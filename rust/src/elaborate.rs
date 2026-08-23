use crate::frontend::{self, ShapePart, TermKind};
use crate::kernel::{self, Clause, Intent, Model, Relation, Role, Sentence, Term};

pub fn program(program: frontend::Program) -> kernel::Result<Model> {
    if program.models.len() != 1 || program.queries.len() != 1 {
        return Err(kernel::KernelError::new(
            "M2 requires exactly one model and one query",
        ));
    }

    let relations = program
        .relations
        .into_iter()
        .map(relation)
        .collect::<kernel::Result<Vec<_>>>()?;
    let intents = program
        .intents
        .into_iter()
        .map(|intent| {
            Intent::new(
                intent.name,
                clause(intent.desired.relation, intent.desired.roles)?,
            )
        })
        .collect::<kernel::Result<Vec<_>>>()?;

    let model = program
        .models
        .into_iter()
        .next()
        .expect("one model checked");
    let query = program
        .queries
        .into_iter()
        .next()
        .expect("one query checked");
    if query.model != model.name {
        return Err(kernel::KernelError::new(
            "query does not name the admitted model",
        ));
    }

    let facts = model
        .facts
        .into_iter()
        .map(|fact| clause(fact.relation, fact.roles))
        .collect::<kernel::Result<Vec<_>>>()?;
    let query = clause(query.relation, query.roles)?;
    Model::with_intents(relations, facts, query, intents, "ascending")
}

/// Elaborate one already-resolved closed frontend operation clause.  Operation
/// execution remains in the kernel; this only carries the source clause across
/// the frontend/kernel boundary used by the M3 canary.
pub fn operation(operation: &frontend::Operation) -> kernel::Result<Clause> {
    clause(
        operation.clause.relation.clone(),
        operation.clause.roles.clone(),
    )
}

fn relation(declaration: frontend::RelationDecl) -> kernel::Result<Relation> {
    let mut parts = declaration.sentence.parts.into_iter();
    let left = match parts.next() {
        Some(ShapePart::Role { name, .. }) => name,
        _ => return Err(kernel::KernelError::new("sentence must begin with a role")),
    };
    let literal = match parts.next() {
        Some(ShapePart::Literal { text, .. }) if !text.trim().is_empty() => text.trim().to_owned(),
        _ => {
            return Err(kernel::KernelError::new(
                "sentence must contain one literal relation",
            ));
        }
    };
    let right = match parts.next() {
        Some(ShapePart::Role { name, .. }) if parts.next().is_none() => name,
        _ => return Err(kernel::KernelError::new("sentence must end with one role")),
    };

    let roles = declaration
        .roles
        .into_iter()
        .map(|role| Role::new(role.name, role.ty))
        .collect::<kernel::Result<Vec<_>>>()?;
    let cardinality = match declaration.mode.cardinality {
        frontend::Cardinality::One => kernel::Cardinality::One,
        frontend::Cardinality::Maybe => kernel::Cardinality::Maybe,
        frontend::Cardinality::Some => kernel::Cardinality::Some,
        frontend::Cardinality::Many => kernel::Cardinality::Many,
    };
    let mode = kernel::Mode::finite(declaration.mode.known, declaration.mode.sought, cardinality)?;
    Relation::new(
        declaration.name,
        roles,
        Sentence::new(left, literal, right)?,
        vec![mode],
    )
}

fn clause(
    relation: String,
    roles: std::collections::BTreeMap<String, frontend::Term>,
) -> kernel::Result<Clause> {
    let roles = roles
        .into_iter()
        .map(|(name, term)| {
            let term = match term.kind {
                TermKind::Text(text) => Term::literal(text)?,
                TermKind::Variable(name) => Term::variable(name)?,
            };
            Ok((name, term))
        })
        .collect::<kernel::Result<Vec<_>>>()?;
    Clause::new(relation, roles)
}
