use super::*;
use clause_package::CanonicalSourceOriginV1;
use std::collections::HashMap;

#[derive(Clone, Eq, Hash, PartialEq)]
struct RuleSource {
    handler: FormationLocalId,
    designation: Vec<u8>,
    origin: Option<CanonicalSourceOriginV1>,
    laws: Vec<CanonicalSourceOriginV1>,
}

/// Only fragments used by this exact generation remain owned. A successor
/// borrows the previous index while building and keeps no generation chain.
pub(in super::super) struct SourceMetadataGraph {
    scope: TermScope,
    end: Term,
    fields: HashMap<Vec<u8>, Term>,
    texts: HashMap<String, Term>,
    numbers: HashMap<u64, Term>,
    origins: HashMap<CanonicalSourceOriginV1, Term>,
    rules: HashMap<RuleSource, Term>,
    row_header: Option<Term>,
    row_roles: HashMap<LocalRoleRefV2, Term>,
    referents: BTreeMap<ExecutableReferentV1, Term>,
}

pub(in super::super) struct SourceMetadataBuilder<'a> {
    graph: SourceMetadataGraph,
    previous: Option<&'a SourceMetadataGraph>,
}

impl<'a> SourceMetadataBuilder<'a> {
    pub(super) fn new(
        scope: TermScope,
        previous: Option<&'a SourceMetadataGraph>,
    ) -> Result<Self, ExecutableErrorV1> {
        if previous.is_some_and(|previous| previous.scope != scope) {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let end = match previous {
            Some(previous) => previous.end.clone(),
            None => projection_literal(scope, b"clause/js-object-end-v1", &[])?,
        };
        Ok(Self {
            graph: SourceMetadataGraph {
                scope,
                end,
                fields: HashMap::new(),
                texts: HashMap::new(),
                numbers: HashMap::new(),
                origins: HashMap::new(),
                rules: HashMap::new(),
                row_header: None,
                row_roles: HashMap::new(),
                referents: BTreeMap::new(),
            },
            previous,
        })
    }

    pub(super) fn finish(self) -> SourceMetadataGraph {
        self.graph
    }

    fn field(&mut self, name: &[u8]) -> Result<Term, ExecutableErrorV1> {
        if let Some(term) = self.graph.fields.get(name) {
            return Ok(term.clone());
        }
        let term = match self.previous.and_then(|previous| previous.fields.get(name)) {
            Some(term) => term.clone(),
            None => projection_literal(self.graph.scope, b"clause/js-field-v1", name)?,
        };
        self.graph.fields.insert(name.to_vec(), term.clone());
        Ok(term)
    }

    fn text(&mut self, value: &str) -> Result<Term, ExecutableErrorV1> {
        if let Some(term) = self.graph.texts.get(value) {
            return Ok(term.clone());
        }
        let term = match self.previous.and_then(|previous| previous.texts.get(value)) {
            Some(term) => term.clone(),
            None => diagnostic_text(self.graph.scope, value)?,
        };
        self.graph.texts.insert(value.to_owned(), term.clone());
        Ok(term)
    }

    fn number(&mut self, value: f64) -> Result<Term, ExecutableErrorV1> {
        let bits = value.to_bits();
        if let Some(term) = self.graph.numbers.get(&bits) {
            return Ok(term.clone());
        }
        let term = match self
            .previous
            .and_then(|previous| previous.numbers.get(&bits))
        {
            Some(term) => term.clone(),
            None => diagnostic_number(self.graph.scope, value)?,
        };
        self.graph.numbers.insert(bits, term.clone());
        Ok(term)
    }

    pub(in super::super) fn object(
        &mut self,
        fields: Vec<(Vec<u8>, Term)>,
    ) -> Result<Term, ExecutableErrorV1> {
        let mut rest = self.graph.end.clone();
        for (field, value) in fields.into_iter().rev() {
            rest = Term::triple([self.field(&field)?, value, rest])
                .map_err(|_| ExecutableErrorV1::MalformedProgram)?;
        }
        Ok(rest)
    }

    pub(in super::super) fn referent(
        &mut self,
        value: ExecutableReferentV1,
    ) -> Result<Term, ExecutableErrorV1> {
        if let Some(term) = self.graph.referents.get(&value) {
            return Ok(term.clone());
        }
        let term = match self
            .previous
            .and_then(|previous| previous.referents.get(&value))
        {
            Some(term) => term.clone(),
            None => projected_scalar_value_term(
                self.graph.scope,
                &ExecutableValueV1::Referent(value.clone()),
            )?,
        };
        self.graph.referents.insert(value, term.clone());
        Ok(term)
    }

    pub(in super::super) fn row(
        &mut self,
        role: LocalRoleRefV2,
        subject: ExecutableReferentV1,
    ) -> Result<Term, ExecutableErrorV1> {
        let header = match &self.graph.row_header {
            Some(term) => term.clone(),
            None => {
                let term = match self
                    .previous
                    .and_then(|previous| previous.row_header.as_ref())
                {
                    Some(term) => term.clone(),
                    None => projection_literal(
                        self.graph.scope,
                        relational_projection::ROW_SELECTOR,
                        &[],
                    )?,
                };
                self.graph.row_header = Some(term.clone());
                term
            }
        };
        let selector = match self.graph.row_roles.get(&role) {
            Some(term) => term.clone(),
            None => {
                let term = match self
                    .previous
                    .and_then(|previous| previous.row_roles.get(&role))
                {
                    Some(term) => term.clone(),
                    None => executable_projection_role_term_v1(
                        self.graph.scope,
                        role,
                        ExecutableValueKindV1::RelationTable,
                    )?,
                };
                self.graph.row_roles.insert(role, term.clone());
                term
            }
        };
        Term::triple([header, selector, self.referent(subject)?])
            .map_err(|_| ExecutableErrorV1::MalformedProgram)
    }

    fn origin(&mut self, origin: CanonicalSourceOriginV1) -> Result<Term, ExecutableErrorV1> {
        if let Some(term) = self.graph.origins.get(&origin) {
            return Ok(term.clone());
        }
        let term = match self
            .previous
            .and_then(|previous| previous.origins.get(&origin))
        {
            Some(term) => term.clone(),
            None => {
                let fields = vec![
                    (
                        b"artifact".to_vec(),
                        self.text(&hex_identity(origin.artifact.as_bytes()))?,
                    ),
                    (b"start".to_vec(), self.number(origin.start as f64)?),
                    (b"end".to_vec(), self.number(origin.end as f64)?),
                ];
                self.object(fields)?
            }
        };
        self.graph.origins.insert(origin, term.clone());
        Ok(term)
    }

    fn rule(&mut self, source: RuleSource) -> Result<Term, ExecutableErrorV1> {
        if let Some(term) = self.graph.rules.get(&source) {
            return Ok(term.clone());
        }
        let term = match self
            .previous
            .and_then(|previous| previous.rules.get(&source))
        {
            Some(term) => term.clone(),
            None => {
                let laws = source
                    .laws
                    .iter()
                    .enumerate()
                    .map(|(index, origin)| {
                        Ok((index.to_string().into_bytes(), self.origin(*origin)?))
                    })
                    .collect::<Result<_, ExecutableErrorV1>>()?;
                let mut fields = vec![
                    (
                        b"handler".to_vec(),
                        self.number(source.handler.get() as f64)?,
                    ),
                    (
                        b"designation".to_vec(),
                        self.text(&String::from_utf8_lossy(&source.designation))?,
                    ),
                    (b"laws".to_vec(), self.object(laws)?),
                ];
                if let Some(origin) = source.origin {
                    fields.push((b"origin".to_vec(), self.origin(origin)?));
                }
                self.object(fields)?
            }
        };
        self.graph.rules.insert(source, term.clone());
        Ok(term)
    }

    fn index(&mut self, fields: Vec<(usize, Term)>) -> Result<Term, ExecutableErrorV1> {
        let mut pages = BTreeMap::<usize, Vec<(Vec<u8>, Term)>>::new();
        for (index, value) in fields {
            pages
                .entry(index / 64)
                .or_default()
                .push(((index % 64).to_string().into_bytes(), value));
        }
        let pages = pages
            .into_iter()
            .map(|(page, fields)| Ok((page.to_string().into_bytes(), self.object(fields)?)))
            .collect::<Result<_, ExecutableErrorV1>>()?;
        self.object(pages)
    }

    fn state(
        &mut self,
        binding: &ExecutableCanonicalStateBindingV1,
    ) -> Result<Term, ExecutableErrorV1> {
        let state = &binding.state;
        let mut fields = vec![
            (b"slot".to_vec(), self.number(binding.slot as f64)?),
            (
                b"subject".to_vec(),
                self.text(&String::from_utf8_lossy(&state.subject))?,
            ),
            (
                b"relation".to_vec(),
                self.text(&String::from_utf8_lossy(&state.relation_designation))?,
            ),
            (
                b"assertion".to_vec(),
                self.number(state.assertion.get() as f64)?,
            ),
            (
                b"schema".to_vec(),
                self.number(state.relation.get() as f64)?,
            ),
            (
                b"subject-role".to_vec(),
                self.number(state.subject_role.role.get() as f64)?,
            ),
            (
                b"value-role".to_vec(),
                self.number(state.value_role.role.get() as f64)?,
            ),
        ];
        if let Some(referent) = state.subject_identity {
            fields.push((
                b"referent".to_vec(),
                projected_scalar_value_term(
                    self.graph.scope,
                    &ExecutableValueV1::Referent(ExecutableReferentV1::declared(
                        referent.domain.get(),
                        referent.identity.get(),
                    )),
                )?,
            ));
        }
        if let clause_package::CanonicalStatePathV1::Field {
            formation,
            designation,
        } = &state.path
        {
            fields.push((
                b"field".to_vec(),
                self.text(&String::from_utf8_lossy(designation))?,
            ));
            fields.push((
                b"field-formation".to_vec(),
                self.number(formation.get() as f64)?,
            ));
        }
        self.object(fields)
    }

    pub(super) fn source(
        &mut self,
        package: &clause_package::CanonicalSourcePackageSliceV1,
        artifact: clause_package::CanonicalSourceArtifactIdV1,
        states: &[ExecutableCanonicalStateBindingV1],
    ) -> Result<Term, ExecutableErrorV1> {
        let mut handlers = package.executable_handlers.iter().collect::<Vec<_>>();
        handlers.sort_by_key(|handler| handler.id);
        let mut rules = Vec::new();
        for handler in handlers {
            let origin = package
                .emissions
                .iter()
                .find(|emission| {
                    emission.allocations.iter().any(|allocation| {
                        allocation.identity == CanonicalAllocatedIdentityV1::Formation(handler.id)
                    })
                })
                .map(|emission| emission.origin);
            for rule in &handler.rules {
                rules.push(self.rule(RuleSource {
                    handler: handler.id,
                    designation: handler.designation.clone(),
                    origin,
                    laws: rule.law_origins.clone(),
                })?);
            }
        }
        // State metadata is keyed by retained physical slot, not source order.
        let state_records = states
            .iter()
            .map(|binding| Ok((usize::from(binding.slot), self.state(binding)?)))
            .collect::<Result<Vec<_>, ExecutableErrorV1>>()?;
        let fields = vec![
            (
                b"artifact".to_vec(),
                self.text(&hex_identity(artifact.as_bytes()))?,
            ),
            (
                b"snapshot".to_vec(),
                self.text(&hex_identity(
                    package.checked_package.constitution().snapshot().as_bytes(),
                ))?,
            ),
            (
                b"rules".to_vec(),
                self.index(rules.into_iter().enumerate().collect())?,
            ),
            (b"states".to_vec(), self.index(state_records)?),
        ];
        self.object(fields)
    }
}
