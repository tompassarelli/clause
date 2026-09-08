//! Reparse one explicit scalar leaf's enclosing block while retaining the
//! unchanged declaration environment. Every retained source span is rebased.
use super::*;

#[derive(Clone, Debug)]
pub(super) struct ParsedSource {
    pub items: Vec<std::sync::Arc<CstItem>>,
    pub scalar_laws: ScalarLawEnvironment,
}

pub(super) struct OriginEdit {
    old: CanonicalSourceArtifactIdV1,
    new: CanonicalSourceArtifactIdV1,
    start: u64,
    end: u64,
    length: u64,
}

impl OriginEdit {
    pub(super) fn origin(&self, origin: &mut CanonicalSourceOriginV1) -> Result<(), CanonicalSourceErrorV1> {
        if origin.artifact != self.old || origin.start > origin.end {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        let shift = |offset: u64| offset.checked_sub(self.end - self.start)
            .and_then(|offset| offset.checked_add(self.length))
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch);
        if origin.end <= self.start {
        } else if origin.start >= self.end {
            origin.start = shift(origin.start)?;
            origin.end = shift(origin.end)?;
        } else if origin.start <= self.start && origin.end >= self.end {
            origin.end = shift(origin.end)?;
        } else {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        origin.artifact = self.new;
        Ok(())
    }

    fn emission(&self, emission: &mut CanonicalSourceEmissionV1) -> Result<(), CanonicalSourceErrorV1> {
        self.origin(&mut emission.origin)
    }

    fn object(&self, object: &mut CanonicalFocusedObjectV1) -> Result<(), CanonicalSourceErrorV1> {
        match object {
            CanonicalFocusedObjectV1::Source(_) => {},
            CanonicalFocusedObjectV1::Fields { fields, .. } => for field in fields {
                self.origin(&mut field.origin)?;
                self.object(&mut field.value)?;
            },
        }
        Ok(())
    }

    fn edge(&self, edge: &mut CanonicalFocusedEdgeV1) -> Result<(), CanonicalSourceErrorV1> {
        self.origin(&mut edge.origin)?;
        self.object(&mut edge.object)
    }

    fn include(&self, include: &mut HandlerIncludeCst) -> Result<(), CanonicalSourceErrorV1> {
        self.origin(&mut include.origin)?;
        if let Some(edge) = &mut include.structured { self.edge(edge)?; }
        Ok(())
    }

    fn bindings(&self, bindings: &mut [ScalarLawBindingCst]) -> Result<(), CanonicalSourceErrorV1> {
        for binding in bindings {
            self.origin(&mut binding.origin)?;
            for case in &mut binding.cases {
                self.origin(&mut case.law_origin)?;
                self.origin(&mut case.derive_origin)?;
                for origin in &mut case.dependency_origins { self.origin(origin)?; }
            }
        }
        Ok(())
    }

    fn relation(&self, relation: &mut RelationCst) -> Result<(), CanonicalSourceErrorV1> {
        if let Some(origin) = &mut relation.contract_origin { self.origin(origin)?; }
        for role in &mut relation.roles { self.origin(&mut role.origin)?; }
        for mode in &mut relation.modes { self.origin(&mut mode.origin)?; }
        Ok(())
    }

    fn item(&self, item: &mut CstItem) -> Result<(), CanonicalSourceErrorV1> {
        self.origin(&mut item.origin)?;
        match &mut item.kind {
            CstKind::Referent { .. } | CstKind::Capability { .. } => {},
            CstKind::Denotation(value) => for emission in &mut value.emissions { self.emission(emission)?; },
            CstKind::Application(value) => self.emission(&mut value.emission)?,
            CstKind::Shape { fields, .. } => for field in fields { self.origin(&mut field.origin)?; },
            CstKind::Relation(value) => self.relation(value)?,
            CstKind::InputHandler(value) => {
                self.origin(&mut value.origin)?;
                self.origin(&mut value.include_origin)?;
            },
            CstKind::ScalarHandler(value) => {
                self.origin(&mut value.origin)?;
                self.include(&mut value.include)?;
                for condition in &mut value.boolean_conditions { self.origin(&mut condition.origin)?; }
            },
            CstKind::GeneralHandler(value) => {
                self.origin(&mut value.origin)?;
                for (_, origin) in &mut value.premises { self.origin(origin)?; }
                for constraint in &mut value.binding_constraints { self.origin(&mut constraint.origin)?; }
                for selector in &mut value.selectors { self.origin(&mut selector.origin)?; }
                for condition in &mut value.boolean_conditions { self.origin(&mut condition.origin)?; }
                self.bindings(&mut value.scalar_bindings)?;
                for sum in &mut value.sums {
                    self.origin(&mut sum.origin)?;
                    for selector in &mut sum.selectors { self.origin(&mut selector.origin)?; }
                    self.bindings(&mut sum.scalar_bindings)?;
                }
                for include in &mut value.includes { self.include(include)?; }
            },
            CstKind::KeyboardBinding(value) => self.origin(&mut value.origin)?,
            CstKind::ScalarInputBinding(value) => self.origin(&mut value.origin)?,
            CstKind::ReferentInputBinding(value) => self.origin(&mut value.origin)?,
            CstKind::ScalarLaw(value) => value.rebase_origins(self)?,
            CstKind::ScalarDerive(value) => self.origin(&mut value.origin)?,
            CstKind::BooleanLaw(value) => {
                self.origin(&mut value.origin)?;
                for selector in &mut value.selectors { self.origin(&mut selector.origin)?; }
                self.origin(&mut value.result.origin)?;
            },
            CstKind::BooleanDerive(value) => self.origin(&mut value.origin)?,
            CstKind::VectorAssertion(value) => self.origin(&mut value.origin)?,
            CstKind::ShapeAssertion(value) => self.origin(&mut value.origin)?,
            CstKind::BooleanAssertion(value) => self.origin(&mut value.origin)?,
            CstKind::NumberAssertion(value) => self.origin(&mut value.origin)?,
            CstKind::SymbolAssertion(value) => self.origin(&mut value.origin)?,
            CstKind::TextAssertion(value) => self.origin(&mut value.origin)?,
            CstKind::Unsupported(value) => {
                self.origin(&mut value.origin)?;
                for emission in &mut value.emissions { self.emission(emission)?; }
            },
        }
        Ok(())
    }
}

pub(super) fn replace_scalar_leaf(
    previous: &CanonicalSourceCstV1,
    selected: &CanonicalScalarEffectV1,
    exact: &[u8],
    replacement_length: usize,
) -> Result<CanonicalSourceCstV1, CanonicalSourceErrorV1> {
    let source = std::str::from_utf8(exact).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
    let artifact = CanonicalSourceArtifactIdV1(domain_hash(SOURCE_ARTIFACT_DOMAIN, &[exact]));
    let edit = OriginEdit { old: previous.artifact, new: artifact,
        start: selected.expression_origin.start, end: selected.expression_origin.end,
        length: replacement_length as u64 };
    let mut parsed = previous.parsed.as_ref().clone();
    let index = parsed.items.iter().position(|item| item.origin == selected.handler_origin
        && matches!(item.kind, CstKind::ScalarHandler(_) | CstKind::GeneralHandler(_)))
        .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
    let mut origin = parsed.items[index].origin;
    edit.origin(&mut origin)?;
    for (other, item) in parsed.items.iter_mut().enumerate() {
        if other != index { edit.item(std::sync::Arc::make_mut(item))?; }
    }
    for item in &mut parsed.scalar_laws.declarations { edit.item(item)?; }
    for relation in &mut parsed.scalar_laws.relations { edit.relation(relation)?; }
    for law in &mut parsed.scalar_laws.laws { law.rebase_origins(&edit)?; }
    for derive in &mut parsed.scalar_laws.derives { edit.origin(&mut derive.origin)?; }
    let lines = source_lines(source)?;
    let block = lines.iter().filter(|line| line.start >= origin.start as usize && line.start < origin.end as usize)
        .copied().collect::<Vec<_>>();
    let replacement = parse_items(artifact, &block, origin, &parsed.scalar_laws, &previous.declared_frontend)?;
    let [replacement] = replacement.as_slice() else { return Err(CanonicalSourceErrorV1::RecordedPlanMismatch); };
    if !matches!(replacement.kind, CstKind::ScalarHandler(_) | CstKind::GeneralHandler(_)) {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    parsed.items[index] = std::sync::Arc::new(replacement.clone());
    let mut vocabularies = previous.vocabularies.clone();
    for vocabulary in &mut vocabularies { edit.origin(&mut vocabulary.origin)?; }
    let mut focuses = previous.subject_focuses.clone();
    for focus in &mut focuses {
        edit.origin(&mut focus.origin)?;
        for edge in &mut focus.edges { edit.edge(edge)?; }
    }
    finish_canonical_source(exact, artifact, &lines, &previous.declared_frontend, parsed, vocabularies, focuses)
}
