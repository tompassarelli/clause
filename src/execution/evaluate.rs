use std::collections::{BTreeMap, BTreeSet};

use crate::{
    intrinsic::{Intrinsic, IntrinsicRole},
    kernel::{ContentId, KernelError, ReferentId, RelationalContent, Result, Revision, Term},
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ActiveKey {
    Definition(ReferentId),
    Application(ContentId),
}

pub(super) fn evaluate(revision: &Revision, term: &Term) -> Result<Term> {
    Evaluator::new(revision).term(term)
}

#[cfg(test)]
pub(super) fn evaluate_with_dispatches(revision: &Revision, term: &Term) -> Result<(Term, usize)> {
    let mut evaluator = Evaluator::new(revision);
    let result = evaluator.term(term)?;
    Ok((result, evaluator.application_dispatches))
}

struct Evaluator<'a> {
    revision: &'a Revision,
    definitions: BTreeMap<ReferentId, Term>,
    applications: BTreeMap<ContentId, Term>,
    active: BTreeSet<ActiveKey>,
    #[cfg(test)]
    application_dispatches: usize,
}

impl<'a> Evaluator<'a> {
    fn new(revision: &'a Revision) -> Self {
        Self {
            revision,
            definitions: BTreeMap::new(),
            applications: BTreeMap::new(),
            active: BTreeSet::new(),
            #[cfg(test)]
            application_dispatches: 0,
        }
    }

    fn term(&mut self, term: &Term) -> Result<Term> {
        match term {
            Term::Referent(id) => self.definition(id),
            Term::Pattern(_) => Err(KernelError::new(
                "pattern cannot be evaluated as a pure term",
            )),
            Term::Application(id) => self.application(id),
            Term::F32(_) | Term::Int(_) | Term::Bool(_) => Ok(term.clone()),
            Term::Product(fields) => Term::product(
                fields
                    .iter()
                    .map(|(label, value)| Ok((label.clone(), self.term(value)?)))
                    .collect::<Result<BTreeMap<_, _>>>()?,
            ),
            Term::Sum { tag, value } => Term::sum(tag.clone(), self.term(value)?),
            Term::Sequence(values) => Term::sequence(
                values
                    .iter()
                    .map(|value| self.term(value))
                    .collect::<Result<Vec<_>>>()?,
            ),
        }
    }

    fn definition(&mut self, id: &ReferentId) -> Result<Term> {
        if let Some(term) = self.definitions.get(id) {
            return Ok(term.clone());
        }
        let Some(denotation) = self
            .revision
            .model()
            .definition(id)
            .map(|definition| definition.denotation().clone())
        else {
            return Ok(Term::referent(id.clone()));
        };
        let key = ActiveKey::Definition(id.clone());
        if !self.active.insert(key.clone()) {
            return Err(KernelError::new(
                "pure definition evaluation contains a cycle",
            ));
        }
        let result = self.term(&denotation);
        self.active.remove(&key);
        if let Ok(term) = &result {
            self.definitions.insert(id.clone(), term.clone());
        }
        result
    }

    fn application(&mut self, id: &ContentId) -> Result<Term> {
        if let Some(term) = self.applications.get(id) {
            return Ok(term.clone());
        }
        let content = self
            .revision
            .model()
            .content(id)
            .cloned()
            .ok_or_else(|| KernelError::new("pure application names undeclared content"))?;
        let key = ActiveKey::Application(id.clone());
        if !self.active.insert(key.clone()) {
            return Err(KernelError::new(
                "pure application evaluation contains a cycle",
            ));
        }
        #[cfg(test)]
        {
            self.application_dispatches += 1;
        }
        let result = self.dispatch(&content);
        self.active.remove(&key);
        if let Ok(term) = &result {
            self.applications.insert(id.clone(), term.clone());
        }
        result
    }

    fn dispatch(&mut self, content: &RelationalContent) -> Result<Term> {
        let intrinsic = Intrinsic::from_relation(content.relation()).ok_or_else(|| {
            KernelError::new("authored relations are not executable pure intrinsics")
        })?;
        require_exact_roles(content, intrinsic)?;
        match intrinsic {
            Intrinsic::Conditional => self.conditional(content),
            Intrinsic::Map => self.map(content),
            Intrinsic::Length => {
                let input = self.normalized_role(content, intrinsic, IntrinsicRole::Input)?;
                length(&input)
            }
            Intrinsic::Equal | Intrinsic::NotEqual => {
                let (left, right) = self.normalized_binary(content, intrinsic)?;
                Ok(Term::boolean(if intrinsic == Intrinsic::Equal {
                    left == right
                } else {
                    left != right
                }))
            }
            Intrinsic::LessThan
            | Intrinsic::LessOrEqual
            | Intrinsic::GreaterThan
            | Intrinsic::GreaterOrEqual => {
                let (left, right) = self.normalized_binary(content, intrinsic)?;
                comparison(intrinsic, &left, &right)
            }
            Intrinsic::Add | Intrinsic::Subtract | Intrinsic::Multiply | Intrinsic::Divide => {
                let (left, right) = self.normalized_binary(content, intrinsic)?;
                arithmetic(intrinsic, &left, &right)
            }
        }
    }

    fn normalized_role(
        &mut self,
        content: &RelationalContent,
        intrinsic: Intrinsic,
        role: IntrinsicRole,
    ) -> Result<Term> {
        let term = content
            .roles()
            .get(&intrinsic.role(role))
            .expect("exact intrinsic role validation precedes dispatch")
            .clone();
        self.term(&term)
    }

    fn normalized_binary(
        &mut self,
        content: &RelationalContent,
        intrinsic: Intrinsic,
    ) -> Result<(Term, Term)> {
        let left = self.normalized_role(content, intrinsic, IntrinsicRole::Left)?;
        let right = self.normalized_role(content, intrinsic, IntrinsicRole::Right)?;
        Ok((left, right))
    }

    fn conditional(&mut self, content: &RelationalContent) -> Result<Term> {
        let intrinsic = Intrinsic::Conditional;
        let condition = self.normalized_role(content, intrinsic, IntrinsicRole::Condition)?;
        let selected = match condition {
            Term::Bool(true) => IntrinsicRole::Then,
            Term::Bool(false) => IntrinsicRole::Else,
            _ => {
                return Err(KernelError::new(
                    "conditional condition must evaluate to Bool",
                ));
            }
        };
        self.normalized_role(content, intrinsic, selected)
    }

    fn map(&mut self, content: &RelationalContent) -> Result<Term> {
        let intrinsic = Intrinsic::Map;
        let mapper = self.normalized_role(content, intrinsic, IntrinsicRole::Mapper)?;
        if mapper != Term::referent(Intrinsic::Length.callable_identity()) {
            return Err(KernelError::new(
                "map requires the exact intrinsic length mapper",
            ));
        }
        let sequence = self.normalized_role(content, intrinsic, IntrinsicRole::Sequence)?;
        let Term::Sequence(values) = sequence else {
            return Err(KernelError::new("map requires a Sequence input"));
        };
        Term::sequence(values.iter().map(length).collect::<Result<Vec<_>>>()?)
    }
}

fn require_exact_roles(content: &RelationalContent, intrinsic: Intrinsic) -> Result<()> {
    let expected = intrinsic
        .input_roles()
        .iter()
        .map(|role| intrinsic.role(*role))
        .collect::<BTreeSet<_>>();
    if content.roles().keys().cloned().collect::<BTreeSet<_>>() == expected {
        Ok(())
    } else {
        Err(KernelError::new(
            "pure intrinsic application must provide exactly its input roles",
        ))
    }
}

fn arithmetic(intrinsic: Intrinsic, left: &Term, right: &Term) -> Result<Term> {
    match (left, right) {
        (Term::F32(left), Term::F32(right)) => {
            arithmetic_f32(intrinsic, left.value(), right.value())
        }
        (Term::Int(left), Term::Int(right)) => arithmetic_int(intrinsic, *left, *right),
        (Term::Product(_), Term::Product(_))
            if matches!(intrinsic, Intrinsic::Add | Intrinsic::Subtract) =>
        {
            tuple_pair(intrinsic, left, right)
        }
        (Term::Product(_), Term::F32(_) | Term::Int(_))
            if matches!(intrinsic, Intrinsic::Multiply | Intrinsic::Divide) =>
        {
            tuple_scalar(intrinsic, left, right)
        }
        _ => Err(KernelError::new(
            "arithmetic requires same-kind scalars or canonical numeric tuples",
        )),
    }
}

fn arithmetic_f32(intrinsic: Intrinsic, left: f32, right: f32) -> Result<Term> {
    if intrinsic == Intrinsic::Divide && right == 0.0 {
        return Err(KernelError::new("division by zero"));
    }
    let value = match intrinsic {
        Intrinsic::Add => left + right,
        Intrinsic::Subtract => left - right,
        Intrinsic::Multiply => left * right,
        Intrinsic::Divide => left / right,
        _ => return Err(KernelError::new("invalid arithmetic intrinsic")),
    };
    Term::f32(value).map_err(|_| KernelError::new("F32 arithmetic produced a nonfinite result"))
}

fn arithmetic_int(intrinsic: Intrinsic, left: i64, right: i64) -> Result<Term> {
    let value = match intrinsic {
        Intrinsic::Add => left.checked_add(right),
        Intrinsic::Subtract => left.checked_sub(right),
        Intrinsic::Multiply => left.checked_mul(right),
        Intrinsic::Divide => left.checked_div(right),
        _ => return Err(KernelError::new("invalid arithmetic intrinsic")),
    }
    .ok_or_else(|| {
        if intrinsic == Intrinsic::Divide && right == 0 {
            KernelError::new("division by zero")
        } else {
            KernelError::new("Int arithmetic overflow")
        }
    })?;
    Ok(Term::int(value))
}

fn comparison(intrinsic: Intrinsic, left: &Term, right: &Term) -> Result<Term> {
    let result = match (left, right) {
        (Term::F32(left), Term::F32(right)) => compare(intrinsic, left.value(), right.value()),
        (Term::Int(left), Term::Int(right)) => compare(intrinsic, *left, *right),
        _ => {
            return Err(KernelError::new(
                "numeric comparison requires same-kind scalar operands",
            ));
        }
    }?;
    Ok(Term::boolean(result))
}

fn compare<T: PartialOrd>(intrinsic: Intrinsic, left: T, right: T) -> Result<bool> {
    match intrinsic {
        Intrinsic::LessThan => Ok(left < right),
        Intrinsic::LessOrEqual => Ok(left <= right),
        Intrinsic::GreaterThan => Ok(left > right),
        Intrinsic::GreaterOrEqual => Ok(left >= right),
        _ => Err(KernelError::new("invalid comparison intrinsic")),
    }
}

enum NumericTuple {
    F32(Vec<f32>),
    Int(Vec<i64>),
}

fn numeric_tuple(term: &Term) -> Result<NumericTuple> {
    let Term::Product(fields) = term else {
        return Err(KernelError::new("numeric tuple must be a Product"));
    };
    let values = fields.values().collect::<Vec<_>>();
    for (index, label) in fields.keys().enumerate() {
        if label.as_str() != format!("_{index:020}") {
            return Err(KernelError::new(
                "numeric tuple requires canonical contiguous ordinal labels",
            ));
        }
    }
    match values.first() {
        Some(Term::F32(_)) => values
            .into_iter()
            .map(|value| match value {
                Term::F32(value) => Ok(value.value()),
                _ => Err(KernelError::new(
                    "numeric tuple leaves must have one scalar kind",
                )),
            })
            .collect::<Result<Vec<_>>>()
            .map(NumericTuple::F32),
        Some(Term::Int(_)) => values
            .into_iter()
            .map(|value| match value {
                Term::Int(value) => Ok(*value),
                _ => Err(KernelError::new(
                    "numeric tuple leaves must have one scalar kind",
                )),
            })
            .collect::<Result<Vec<_>>>()
            .map(NumericTuple::Int),
        Some(_) => Err(KernelError::new(
            "numeric tuple leaves must be numeric scalars",
        )),
        None => Err(KernelError::new("numeric tuple must be nonempty")),
    }
}

fn tuple_pair(intrinsic: Intrinsic, left: &Term, right: &Term) -> Result<Term> {
    match (numeric_tuple(left)?, numeric_tuple(right)?) {
        (NumericTuple::F32(left), NumericTuple::F32(right)) if left.len() == right.len() => {
            Term::tuple(
                left.into_iter()
                    .zip(right)
                    .map(|(left, right)| arithmetic_f32(intrinsic, left, right))
                    .collect::<Result<Vec<_>>>()?,
            )
        }
        (NumericTuple::Int(left), NumericTuple::Int(right)) if left.len() == right.len() => {
            Term::tuple(
                left.into_iter()
                    .zip(right)
                    .map(|(left, right)| arithmetic_int(intrinsic, left, right))
                    .collect::<Result<Vec<_>>>()?,
            )
        }
        _ => Err(KernelError::new(
            "tuple arithmetic requires equal shapes and matching numeric leaf kinds",
        )),
    }
}

fn tuple_scalar(intrinsic: Intrinsic, tuple: &Term, scalar: &Term) -> Result<Term> {
    match (numeric_tuple(tuple)?, scalar) {
        (NumericTuple::F32(values), Term::F32(scalar)) => Term::tuple(
            values
                .into_iter()
                .map(|value| arithmetic_f32(intrinsic, value, scalar.value()))
                .collect::<Result<Vec<_>>>()?,
        ),
        (NumericTuple::Int(values), Term::Int(scalar)) => Term::tuple(
            values
                .into_iter()
                .map(|value| arithmetic_int(intrinsic, value, *scalar))
                .collect::<Result<Vec<_>>>()?,
        ),
        _ => Err(KernelError::new(
            "tuple scaling requires a scalar matching the tuple leaf kind",
        )),
    }
}

fn length(term: &Term) -> Result<Term> {
    let sum = match numeric_tuple(term)? {
        NumericTuple::F32(values) => values
            .into_iter()
            .map(|value| f64::from(value).powi(2))
            .sum::<f64>(),
        NumericTuple::Int(values) => values
            .into_iter()
            .map(|value| (value as f64).powi(2))
            .sum::<f64>(),
    };
    Term::f32(sum.sqrt() as f32)
        .map_err(|_| KernelError::new("tuple length produced a nonfinite F32 result"))
}
