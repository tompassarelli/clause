use super::*;
mod declarations;
pub(super) use declarations::rebind_declarations;

/// A retained analysis belongs to one exact parsed source, allocation root and
/// semantic context. Its checked executable results cannot be replaced by callers.
#[derive(Debug)]
pub struct CheckedCanonicalSourceAnalysisV1 {
    source: CanonicalSourceCstV1,
    plan: CanonicalSourceAllocationPlanV1,
    context: CanonicalSourceContextV1,
    package: CanonicalSourcePackageSliceV1,
    scalar_effects: std::sync::OnceLock<Vec<CanonicalScalarEffectV1>>,
    retained_rules: Option<(ProgramChangeOccurrenceId, BTreeMap<(FormationLocalId, usize), (FormationLocalId, usize)>)>,
}

pub(super) struct RetainedSourceDerivations<'a> {
    pub analysis: &'a CheckedCanonicalSourceAnalysisV1,
    pub retained_rules: &'a mut BTreeMap<(FormationLocalId, usize), (FormationLocalId, usize)>,
    pub selected: &'a BTreeSet<FormationLocalId>,
    pub previous: &'a CanonicalSourcePackageSliceV1,
    pub edit: &'a CanonicalSourceEditV1,
}

impl CheckedCanonicalSourceAnalysisV1 {
    pub fn new(source: CanonicalSourceCstV1, plan: CanonicalSourceAllocationPlanV1, context: CanonicalSourceContextV1) -> Result<Self, CanonicalSourceErrorV1> {
        rematerialize_canonical_source_allocation_plan_v1(&source, &plan)?;
        let package = elaborate_canonical_source_package_v1(&source, context, &plan)?;
        Ok(Self { source, plan, context, package, scalar_effects: std::sync::OnceLock::new(), retained_rules: None })
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

    /// Checked correspondence for unchanged rules from the preceding source.
    /// Rebinding can reorder rules, so ordinals alone do not identify them.
    pub fn retained_rule(&self, old_root: ProgramChangeOccurrenceId, handler: FormationLocalId, rule: usize) -> Option<(FormationLocalId, usize)> {
        let (root, rules) = self.retained_rules.as_ref()?;
        (*root == old_root).then(|| rules.get(&(handler, rule)).copied()).flatten()
    }

    pub fn advance(&self, edit: &CanonicalSourceEditV1) -> Result<Self, CanonicalSourceErrorV1> {
        if edit.old_artifact != self.source.artifact() || edit.old_root != self.plan.root() {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        let mut retained_rules = BTreeMap::new();
        let package = if let Some((changed, _, _, _)) = edit.scalar_change {
            let selected = BTreeSet::from([edit.formation(changed)?]);
            let derivations = RetainedSourceDerivations { analysis: self, retained_rules: &mut retained_rules, selected: &selected, previous: &self.package, edit };
            elaborate_canonical_source_package_inner(edit.source(), self.context, edit.plan(), Some(derivations))?
        } else {
            elaborate_canonical_source_package_v1(edit.source(), self.context, edit.plan())?
        };
        Ok(Self { source: edit.source().clone(), plan: edit.plan().clone(), context: self.context, package, scalar_effects: std::sync::OnceLock::new(), retained_rules: Some((self.plan.root(), retained_rules)) })
    }
}

impl RetainedSourceDerivations<'_> {
    pub(super) fn handlers(&mut self) -> Result<Vec<CanonicalExecutableHandlerV1>, CanonicalSourceErrorV1> {
        let analysis = self.analysis;
        let edit = self.edit;
        let retained_rules = &mut self.retained_rules;
        let (changed, _, _, _) = edit.scalar_change.ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        let scalar_handlers = analysis.source.items.iter().filter_map(|item| match &item.kind {
            CstKind::ScalarHandler(handler) => Some(formation_id(&analysis.plan, &handler.producer, &head_slot(CanonicalSourceProductionV1::Handler))),
            _ => None,
        }).collect::<Result<BTreeSet<_>, _>>()?;
        let relational_handlers = analysis.source.items.iter().filter_map(|item| match &item.kind {
            CstKind::GeneralHandler(handler) if relational_handler_origins(&analysis.source).contains(&handler.origin) => Some(formation_id(&analysis.plan, &handler.producer, &head_slot(handler.producer.production))),
            _ => None,
        }).collect::<Result<BTreeSet<_>, _>>()?;
        let mut retained = Vec::new();
        let mut member_sets = BTreeMap::new();
        for handler in &analysis.package.executable_handlers {
            if handler.id == changed { continue; }
            let scalar = scalar_handlers.contains(&handler.id);
            let sorted_assignments = scalar || relational_handlers.contains(&handler.id);
            let old_handler = handler.id;
            let mut handler = handler.clone();
            handler.id = edit.formation(handler.id)?;
            for rule in &mut handler.rules {
                for origin in &mut rule.law_origins { *origin = translate_origin(edit, *origin)?; }
                for predicate in &mut rule.predicates { remap_predicate(edit, predicate, &mut member_sets)?; }
                for state in rule.required_present.iter_mut().chain(&mut rule.required_absent).chain(&mut rule.removals) {
                    edit.rebind_state(state)?;
                }
                rule.required_present.sort();
                rule.required_absent.sort();
                rule.removals.sort();
                for assignment in &mut rule.assignments {
                    edit.rebind_state(&mut assignment.target)?;
                    remap_expression(edit, &mut assignment.value, &mut member_sets)?;
                }
                if sorted_assignments { rule.assignments.sort_by(|a, b| a.target.cmp(&b.target)); }
            }
            let mut rules = handler.rules.into_iter().enumerate().collect::<Vec<_>>();
            if scalar { rules.sort_by(|(_, a), (_, b)| a.assignments.first().map(|a| &a.target).cmp(&b.assignments.first().map(|b| &b.target))); }
            handler.rules = rules.into_iter().enumerate().map(|(new_index, (old_index, rule))| {
                retained_rules.insert((handler.id, new_index), (old_handler, old_index));
                rule
            }).collect();
            retained.push(handler);
        }
        Ok(retained)
    }
}

fn translate_origin(edit: &CanonicalSourceEditV1, origin: CanonicalSourceOriginV1) -> Result<CanonicalSourceOriginV1, CanonicalSourceErrorV1> {
    let (_, start, end, length) = edit.scalar_change.ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
    if origin.artifact != edit.old_artifact {
        return edit.source().imported_sources.get(&origin.artifact)
            .filter(|source| origin.start <= origin.end && origin.end <= source.len() as u64)
            .map(|_| origin).ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
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
        CanonicalScalarValueV1::Sequence(values) => for value in values { remap_value(edit, value)?; },
        CanonicalScalarValueV1::Record(fields) => for value in fields.values_mut() { remap_value(edit, value)?; },
    }
    Ok(())
}

fn remap_predicate(edit: &CanonicalSourceEditV1, predicate: &mut CanonicalExecutablePredicateV1, member_sets: &mut BTreeMap<Vec<FormationLocalId>, Vec<FormationLocalId>>) -> Result<(), CanonicalSourceErrorV1> {
    use CanonicalExecutablePredicateV1 as P;
    let (a, b) = match predicate {
        P::RelationMatch(state, a, b) => { edit.rebind_state(state)?; (a, b) }
        P::Equal(a, b) | P::GreaterThan(a, b) | P::LessThanOrEqual(a, b) | P::Contains(a, b) => (a, b),
    };
    remap_expression(edit, a, member_sets)?;
    remap_expression(edit, b, member_sets)
}

fn remap_expression(edit: &CanonicalSourceEditV1, expression: &mut CanonicalExecutableExpressionV1, member_sets: &mut BTreeMap<Vec<FormationLocalId>, Vec<FormationLocalId>>) -> Result<(), CanonicalSourceErrorV1> {
    use CanonicalExecutableExpressionV1 as E;
    match expression {
        E::Lambda { body: value, .. } | E::Widen { value, .. } => remap_expression(edit, value, member_sets)?,
        E::Match { value, cases } => {
            remap_expression(edit, value, member_sets)?;
            for (_, _, body) in cases { remap_expression(edit, body, member_sets)?; }
        }
        E::Constant(value) => remap_value(edit, value)?,
        E::State(state) => edit.rebind_state(state)?,
        E::Argument(_) | E::Binding(_) | E::EmptySequence(_) => {}
        E::Sequence(values) | E::Foreign { arguments: values, .. } => for value in values { remap_expression(edit, value, member_sets)?; },
        E::Record(fields) => for value in fields.values_mut() { remap_expression(edit, value, member_sets)?; },
        E::Let { value, body, .. } | E::SequenceMap { source: value, body, .. } => {
            remap_expression(edit, value, member_sets)?; remap_expression(edit, body, member_sets)?;
        },
        E::SequenceFold { source, initial, body, .. } => {
            remap_expression(edit, source, member_sets)?; remap_expression(edit, initial, member_sets)?; remap_expression(edit, body, member_sets)?;
        },
        E::TextCodepoint(value) | E::TextFromCodepoint(value) | E::TextCharacters(value) | E::ParseIntegerPrefix(value) | E::SequenceRange(value) | E::SequenceCount(value) | E::SequenceSort(value) | E::ScalarText(value) | E::Field(value, _) => remap_expression(edit, value, member_sets)?,
        E::Apply(a, b) | E::SequenceAt(a, b) | E::SequenceDrop(a, b) | E::SequenceJoin(a, b) | E::Dictionary(a, b) | E::SequenceAppend(a, b) => {
            remap_expression(edit, a, member_sets)?; remap_expression(edit, b, member_sets)?;
        },
        E::Require(a, b, c) => {
            remap_expression(edit, a, member_sets)?; remap_expression(edit, b, member_sets)?; remap_expression(edit, c, member_sets)?;
        },
        E::FreshReferent { domain, .. } => *domain = edit.formation(*domain)?,
        E::ReferentFacet { value, domain, members } => {
            remap_expression(edit, value, member_sets)?;
            *domain = edit.formation(*domain)?;
            // Repeated facets carry the same complete member set. Reuse only
            // an exact set translated successfully under this one checked edit.
            if let Some(retained) = member_sets.get(members.as_slice()) {
                members.clone_from(retained);
            } else {
                let old = members.clone();
                for member in members.iter_mut() { *member = edit.formation(*member)?; }
                members.sort();
                member_sets.insert(old, members.clone());
            }
        }
        E::Sum { inputs, predicates, value } => {
            for input in inputs { remap_expression(edit, input, member_sets)?; }
            for predicate in predicates { remap_predicate(edit, predicate, member_sets)?; }
            remap_expression(edit, value, member_sets)?;
        }
        E::MatchesAny(cases) => for case in cases { for predicate in case { remap_predicate(edit, predicate, member_sets)?; } },
        E::RelationEffects(effects) => for effect in effects {
            use CanonicalRelationEffectV1 as R;
            let (a, b) = match effect { R::Put(a,b) | R::Insert(a,b) | R::Remove(a,b) | R::Accumulate(a,b) => (a,b) };
            remap_expression(edit, a, member_sets)?; remap_expression(edit, b, member_sets)?;
        },
        E::SquareRoot(value) | E::Not(value) | E::Accumulate(value) | E::TextTransform(_,value) => remap_expression(edit, value, member_sets)?,
        E::Conditional(a,b,c) | E::RelationPut(a,b,c) | E::RelationInsert(a,b,c) | E::RelationRemoveValue(a,b,c) => {
            remap_expression(edit, a, member_sets)?; remap_expression(edit, b, member_sets)?; remap_expression(edit, c, member_sets)?;
        }
        E::TextSplit(a,b) | E::ContainsText(a,b) | E::StartsWith(a,b) | E::Equal(a,b) | E::GreaterThan(a,b) | E::LessThanOrEqual(a,b)
        | E::RelationRead(a,b) | E::RelationPresent(a,b) | E::RelationRemoveRow(a,b) | E::Concatenate(a,b)
        | E::Add(a,b) | E::Subtract(a,b) | E::Multiply(a,b) | E::Divide(a,b) | E::Insert(a,b) | E::Remove(a,b) => {
            remap_expression(edit, a, member_sets)?; remap_expression(edit, b, member_sets)?;
        }
    }
    Ok(())
}
