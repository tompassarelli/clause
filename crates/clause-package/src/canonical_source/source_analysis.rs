use super::*;

/// A retained analysis belongs to one exact parsed source, allocation root and
/// semantic context. Its checked executable results cannot be replaced by callers.
#[derive(Debug)]
pub struct CheckedCanonicalSourceAnalysisV1 {
    source: CanonicalSourceCstV1,
    plan: CanonicalSourceAllocationPlanV1,
    context: CanonicalSourceContextV1,
    package: CanonicalSourcePackageSliceV1,
    scalar_effects: std::sync::OnceLock<Vec<CanonicalScalarEffectV1>>,
}

pub(super) struct RetainedSourceDerivations<'a> {
    pub handlers: &'a [CanonicalExecutableHandlerV1],
    pub selected: &'a BTreeSet<FormationLocalId>,
    pub formations: BTreeMap<FormationLocalId, &'a FormationJudgmentPreimageV2>,
}

impl CheckedCanonicalSourceAnalysisV1 {
    pub fn new(source: CanonicalSourceCstV1, plan: CanonicalSourceAllocationPlanV1, context: CanonicalSourceContextV1) -> Result<Self, CanonicalSourceErrorV1> {
        rematerialize_canonical_source_allocation_plan_v1(&source, &plan)?;
        let package = elaborate_canonical_source_package_v1(&source, context, &plan)?;
        Ok(Self { source, plan, context, package, scalar_effects: std::sync::OnceLock::new() })
    }
    pub fn source(&self) -> &CanonicalSourceCstV1 { &self.source }
    pub fn plan(&self) -> &CanonicalSourceAllocationPlanV1 { &self.plan }
    pub fn package(&self) -> &CanonicalSourcePackageSliceV1 { &self.package }

    pub fn scalar_effects(&self) -> Result<&[CanonicalScalarEffectV1], CanonicalSourceErrorV1> {
        if let Some(effects) = self.scalar_effects.get() { return Ok(effects); }
        let effects = live_edit::scalar_effects_from_bound_source(&self.source, &self.plan)?;
        let _ = self.scalar_effects.set(effects);
        Ok(self.scalar_effects.get().expect("successful effects analysis initializes the exact source index"))
    }

    pub fn replace_scalar_effect(
        &self,
        handler: FormationLocalId,
        effect: FormationLocalId,
        field_path: &[FormationLocalId],
        replacement: &[u8],
        new_root: ProgramChangeOccurrenceId,
    ) -> Result<CanonicalSourceEditV1, CanonicalSourceErrorV1> {
        let selected = self.scalar_effects()?.iter().find(|selected|
            selected.handler == handler && selected.effect == effect && selected.field_path == field_path)
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        if selected.expression == replacement { return Err(CanonicalSourceErrorV1::RecordedPlanMismatch); }
        live_edit::replace_bound_scalar_effect(&self.source, &self.plan, selected, replacement, new_root)
    }

    pub fn advance(&self, edit: &CanonicalSourceEditV1) -> Result<Self, CanonicalSourceErrorV1> {
        if edit.old_artifact != self.source.artifact() || edit.old_root != self.plan.root() {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        let package = if let Some((changed, _, _, _)) = edit.scalar_change {
            let selected = BTreeSet::from([edit.formation(changed)?]);
            let scalar_handlers = self.source.items.iter().filter_map(|item| match &item.kind {
                CstKind::ScalarHandler(handler) => Some(formation_id(&self.plan, &handler.producer, &head_slot(CanonicalSourceProductionV1::Handler))),
                _ => None,
            }).collect::<Result<BTreeSet<_>, _>>()?;
            let relational_handlers = self.source.items.iter().filter_map(|item| match &item.kind {
                CstKind::GeneralHandler(handler) if relational_handler_origins(&self.source).contains(&handler.origin) => Some(formation_id(&self.plan, &handler.producer, &head_slot(handler.producer.production))),
                _ => None,
            }).collect::<Result<BTreeSet<_>, _>>()?;
            let mut retained = Vec::new();
            for handler in &self.package.executable_handlers {
                if handler.id == changed { continue; }
                let scalar = scalar_handlers.contains(&handler.id);
                let sorted_assignments = scalar || relational_handlers.contains(&handler.id);
                let mut handler = handler.clone();
                handler.id = edit.formation(handler.id)?;
                for rule in &mut handler.rules {
                    for origin in &mut rule.law_origins { *origin = translate_origin(edit, *origin)?; }
                    for predicate in &mut rule.predicates { remap_predicate(edit, predicate)?; }
                    for state in rule.required_present.iter_mut().chain(&mut rule.required_absent).chain(&mut rule.removals) {
                        *state = edit.state(state)?;
                    }
                    rule.required_present.sort();
                    rule.required_absent.sort();
                    rule.removals.sort();
                    for assignment in &mut rule.assignments {
                        assignment.target = edit.state(&assignment.target)?;
                        remap_expression(edit, &mut assignment.value)?;
                    }
                    if sorted_assignments { rule.assignments.sort_by(|a, b| a.target.cmp(&b.target)); }
                }
                if scalar { handler.rules.sort_by(|a, b| a.assignments.first().map(|a| &a.target).cmp(&b.assignments.first().map(|b| &b.target))); }
                retained.push(handler);
            }
            let formations = self.package.checked_package.constitution().preimage().formations.iter()
                .filter_map(|formation| edit.formation(formation.id).ok().map(|id| (id, formation)))
                .collect();
            let derivations = RetainedSourceDerivations { handlers: &retained, selected: &selected, formations };
            elaborate_canonical_source_package_inner(edit.source(), self.context, edit.plan(), Some(&derivations))?
        } else {
            elaborate_canonical_source_package_v1(edit.source(), self.context, edit.plan())?
        };
        Ok(Self { source: edit.source().clone(), plan: edit.plan().clone(), context: self.context, package, scalar_effects: std::sync::OnceLock::new() })
    }
}

fn translate_origin(edit: &CanonicalSourceEditV1, origin: CanonicalSourceOriginV1) -> Result<CanonicalSourceOriginV1, CanonicalSourceErrorV1> {
    let (_, start, end, length) = edit.scalar_change.ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
    if origin.artifact != edit.old_artifact { return Err(CanonicalSourceErrorV1::RecordedPlanMismatch); }
    let shift = |offset: u64| offset.checked_sub(end - start).and_then(|offset| offset.checked_add(length)).ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch);
    let (start, end) = if origin.end <= start { (origin.start, origin.end) }
        else if origin.start >= end { (shift(origin.start)?, shift(origin.end)?) }
        else { return Err(CanonicalSourceErrorV1::RecordedPlanMismatch); };
    Ok(CanonicalSourceOriginV1 { artifact: edit.source().artifact(), start, end })
}

fn remap_value(edit: &CanonicalSourceEditV1, value: &mut CanonicalScalarValueV1) -> Result<(), CanonicalSourceErrorV1> {
    match value {
        CanonicalScalarValueV1::Referent(value) => *value = edit.referent(*value)?,
        CanonicalScalarValueV1::RelationTable(table) => {
            table.subject_domain = edit.formation(table.subject_domain)?;
            if let CanonicalRelationValueKindV1::Referent(domain) = &mut table.value_kind { *domain = edit.formation(*domain)?; }
            table.rows = std::mem::take(&mut table.rows).into_iter().map(|(subject, values)| {
                Ok((edit.referent(subject)?, values.into_iter().map(|mut value| { remap_value(edit, &mut value)?; Ok(value) }).collect::<Result<_, CanonicalSourceErrorV1>>()?))
            }).collect::<Result<_, CanonicalSourceErrorV1>>()?;
        }
        CanonicalScalarValueV1::Number(_) | CanonicalScalarValueV1::Boolean(_) | CanonicalScalarValueV1::Symbol(_) | CanonicalScalarValueV1::Text(_) => {}
    }
    Ok(())
}

fn remap_predicate(edit: &CanonicalSourceEditV1, predicate: &mut CanonicalExecutablePredicateV1) -> Result<(), CanonicalSourceErrorV1> {
    use CanonicalExecutablePredicateV1 as P;
    let (a, b) = match predicate {
        P::RelationMatch(state, a, b) => { *state = edit.state(state)?; (a, b) }
        P::Equal(a, b) | P::GreaterThan(a, b) | P::LessThanOrEqual(a, b) | P::Contains(a, b) => (a, b),
    };
    remap_expression(edit, a)?;
    remap_expression(edit, b)
}

fn remap_expression(edit: &CanonicalSourceEditV1, expression: &mut CanonicalExecutableExpressionV1) -> Result<(), CanonicalSourceErrorV1> {
    use CanonicalExecutableExpressionV1 as E;
    match expression {
        E::Constant(value) => remap_value(edit, value)?,
        E::State(state) => *state = edit.state(state)?,
        E::Argument(_) | E::Binding(_) => {}
        E::FreshReferent { domain, .. } => *domain = edit.formation(*domain)?,
        E::ReferentFacet { value, domain, members } => {
            remap_expression(edit, value)?;
            *domain = edit.formation(*domain)?;
            for member in members.iter_mut() { *member = edit.formation(*member)?; }
            members.sort();
        }
        E::Sum { inputs, predicates, value } => {
            for input in inputs { remap_expression(edit, input)?; }
            for predicate in predicates { remap_predicate(edit, predicate)?; }
            remap_expression(edit, value)?;
        }
        E::MatchesAny(cases) => for case in cases { for predicate in case { remap_predicate(edit, predicate)?; } },
        E::RelationEffects(effects) => for effect in effects {
            use CanonicalRelationEffectV1 as R;
            let (a, b) = match effect { R::Put(a,b) | R::Insert(a,b) | R::Remove(a,b) | R::Accumulate(a,b) => (a,b) };
            remap_expression(edit, a)?; remap_expression(edit, b)?;
        },
        E::SquareRoot(value) | E::Not(value) | E::Accumulate(value) | E::TextTransform(_,value) => remap_expression(edit,value)?,
        E::Conditional(a,b,c) | E::RelationPut(a,b,c) | E::RelationInsert(a,b,c) | E::RelationRemoveValue(a,b,c) => {
            remap_expression(edit,a)?; remap_expression(edit,b)?; remap_expression(edit,c)?;
        }
        E::ContainsText(a,b) | E::StartsWith(a,b) | E::Equal(a,b) | E::GreaterThan(a,b) | E::LessThanOrEqual(a,b)
        | E::RelationRead(a,b) | E::RelationPresent(a,b) | E::RelationRemoveRow(a,b) | E::Concatenate(a,b)
        | E::Add(a,b) | E::Subtract(a,b) | E::Multiply(a,b) | E::Divide(a,b) | E::Insert(a,b) | E::Remove(a,b) => {
            remap_expression(edit,a)?; remap_expression(edit,b)?;
        }
    }
    Ok(())
}
