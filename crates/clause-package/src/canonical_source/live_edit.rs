//! Explicit source operations, not a text-diff identity heuristic.
use super::*;

/// A snapshot-scoped scalar expression in a handler effect. Origins locate
/// source for display and replay; allocated identities select the occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalScalarEffectV1 {
    pub artifact: CanonicalSourceArtifactIdV1,
    pub handler: FormationLocalId,
    pub effect: FormationLocalId,
    /// Empty for a scalar effect; otherwise the declared ShapeField identities
    /// from the enclosing product to this leaf. Equal values are not addresses.
    pub field_path: Vec<FormationLocalId>,
    pub handler_origin: CanonicalSourceOriginV1,
    pub expression_origin: CanonicalSourceOriginV1,
    pub expression: Vec<u8>,
}

/// Constructors replay explicit replacements on the exact old tree.
/// Unchanged allocations are retained by that operation, NOT inferred
/// from equal names, values, source spans, or arbitrary imported text.
/// Local coordinates in the new snapshot are fresh; these pairs explicitly
/// connect their continuing semantic occurrences across snapshot addresses.
#[derive(Clone, Debug)]
pub struct CanonicalSourceEditV1 {
    pub(super) old_artifact: CanonicalSourceArtifactIdV1,
    pub(super) old_root: ProgramChangeOccurrenceId,
    pub(super) scalar_change: Option<(FormationLocalId, u64, u64, u64)>,
    source: CanonicalSourceCstV1,
    plan: CanonicalSourceAllocationPlanV1,
    retained: BTreeMap<CanonicalAllocatedIdentityV1, CanonicalAllocatedIdentityV1>,
    retained_index: std::sync::OnceLock<std::collections::HashMap<CanonicalAllocatedIdentityV1, CanonicalAllocatedIdentityV1>>,
}

impl CanonicalSourceEditV1 {
    fn retained_identity(&self, old: CanonicalAllocatedIdentityV1) -> Option<CanonicalAllocatedIdentityV1> {
        self.retained_index.get_or_init(|| self.retained.iter().map(|(old, new)| (*old, *new)).collect())
            .get(&old).copied()
    }

    pub fn source(&self) -> &CanonicalSourceCstV1 {
        &self.source
    }
    pub fn plan(&self) -> &CanonicalSourceAllocationPlanV1 {
        &self.plan
    }

    pub fn retained(
        &self,
    ) -> &BTreeMap<CanonicalAllocatedIdentityV1, CanonicalAllocatedIdentityV1> {
        &self.retained
    }

    pub fn formation(
        &self,
        old: FormationLocalId,
    ) -> Result<FormationLocalId, CanonicalSourceErrorV1> {
        match self.retained_identity(CanonicalAllocatedIdentityV1::Formation(old)) {
            Some(CanonicalAllocatedIdentityV1::Formation(new)) => Ok(new),
            _ => Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        }
    }

    pub fn referent(
        &self,
        old: CanonicalReferentV1,
    ) -> Result<CanonicalReferentV1, CanonicalSourceErrorV1> {
        Ok(CanonicalReferentV1 {
            domain: self.formation(old.domain)?,
            identity: self.formation(old.identity)?,
        })
    }

    pub fn state(
        &self,
        old: &CanonicalStateRefV1,
    ) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
        let mut new = old.clone();
        self.rebind_state(&mut new)?;
        Ok(new)
    }

    pub(super) fn rebind_state(&self, state: &mut CanonicalStateRefV1) -> Result<(), CanonicalSourceErrorV1> {
        let role = |old| match self.retained_identity(CanonicalAllocatedIdentityV1::Role(old)) {
            Some(CanonicalAllocatedIdentityV1::Role(new)) => Ok(new),
            _ => Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        };
        let relation = match self.retained_identity(CanonicalAllocatedIdentityV1::RelationSchema(state.relation)) {
            Some(CanonicalAllocatedIdentityV1::RelationSchema(new)) => new,
            _ => return Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        };
        let assertion = self.formation(state.assertion)?;
        let subject_role = role(state.subject_role)?;
        let value_role = role(state.value_role)?;
        let subject_identity = state.subject_identity.map(|value| self.referent(value)).transpose()?;
        let field = match &state.path {
            CanonicalStatePathV1::Field { formation, .. } => Some(self.formation(*formation)?),
            _ => None,
        };
        state.assertion = assertion;
        state.relation = relation;
        state.subject_role = subject_role;
        state.value_role = value_role;
        state.subject_identity = subject_identity;
        if let (CanonicalStatePathV1::Field { formation, .. }, Some(new)) = (&mut state.path, field) {
            *formation = new;
        }
        Ok(())
    }

}

pub fn canonical_scalar_effects_v1(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
) -> Result<Vec<CanonicalScalarEffectV1>, CanonicalSourceErrorV1> {
    rematerialize_canonical_source_allocation_plan_v1(cst, plan)?;
    scalar_effects_from_bound_source(cst, plan)
}

pub(super) fn scalar_effects_from_bound_source(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
) -> Result<Vec<CanonicalScalarEffectV1>, CanonicalSourceErrorV1> {
    let mut effects = Vec::new();
    for item in &cst.items {
        let (producer, origin, includes) = match &item.kind {
            CstKind::GeneralHandler(handler) if !handler.derivation => (
                &handler.producer,
                handler.origin,
                handler.includes.as_slice(),
            ),
            CstKind::ScalarHandler(handler) => (
                &handler.producer,
                handler.origin,
                std::slice::from_ref(&handler.include),
            ),
            _ => continue,
        };
        let handler = formation_id(
            plan,
            producer,
            &head_slot(CanonicalSourceProductionV1::Handler),
        )?;
        for include in includes {
            let effect = formation_id(
                plan,
                producer,
                &child_slot(CanonicalSourceProductionV1::HandlerInclude, &include.local),
            )?;
            let selected = CanonicalScalarEffectV1 {
                artifact: cst.artifact,
                handler,
                effect,
                field_path: vec![],
                handler_origin: origin,
                expression_origin: include.origin,
                expression: vec![],
            };
            if let Some(edge) = &include.structured {
                structured_effects(cst, plan, &edge.object, &selected, &mut effects)?;
                continue;
            }
            let exact = std::str::from_utf8(
                cst.source_slice(include.origin)
                    .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?,
            )
            .map_err(|_| CanonicalSourceErrorV1::RecordedPlanMismatch)?;
            let line = std::str::from_utf8(&include.local)
                .map_err(|_| CanonicalSourceErrorV1::RecordedPlanMismatch)?;
            let line = line.strip_prefix("accumulate ").unwrap_or(line);
            if let Some((_, shape, fields)) = split_shape_subject(exact.trim()) {
                for (name, expression) in parse_shape_fields(fields)
                    .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?
                {
                    let mut field = selected.clone();
                    field
                        .field_path
                        .push(declared_field(plan, shape.as_bytes(), name.as_bytes())?);
                    // Both slices belong to this exact parsed clause. Their
                    // offset is projection data, never occurrence selection.
                    let offset = expression.as_ptr().addr() - exact.as_ptr().addr();
                    field.expression_origin.start += offset as u64;
                    field.expression_origin.end =
                        field.expression_origin.start + expression.len() as u64;
                    field.expression = expression.as_bytes().to_vec();
                    effects.push(field);
                }
                continue;
            }
            let Some((subject, relation, _)) = split_general_scalar_insertion(line) else {
                continue;
            };
            let tail = line[subject.len()..].trim_start();
            let expression = tail[relation.len()..].trim_start();
            // Focus changes the printed prefix, not the parsed expression.
            // Only a contiguous exact source suffix admits this scalar edit;
            // multiline and structured values require their own operations.
            let Some(prefix) = exact.trim_end().strip_suffix(expression) else {
                continue;
            };
            let offset = prefix.len();
            effects.push(CanonicalScalarEffectV1 {
                artifact: cst.artifact,
                handler,
                effect,
                field_path: vec![],
                handler_origin: origin,
                expression_origin: CanonicalSourceOriginV1 {
                    artifact: cst.artifact,
                    start: include.origin.start + offset as u64,
                    end: include.origin.start + (offset + expression.len()) as u64,
                },
                expression: expression.as_bytes().to_vec(),
            });
        }
    }
    Ok(effects)
}

fn declared_field(
    plan: &CanonicalSourceAllocationPlanV1,
    shape: &[u8],
    field: &[u8],
) -> Result<FormationLocalId, CanonicalSourceErrorV1> {
    formation_id(
        plan,
        &semantic_producer(CanonicalSourceProductionV1::Shape, shape),
        &child_slot(CanonicalSourceProductionV1::ShapeField, field),
    )
}

fn structured_effects(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    object: &CanonicalFocusedObjectV1,
    parent: &CanonicalScalarEffectV1,
    effects: &mut Vec<CanonicalScalarEffectV1>,
) -> Result<(), CanonicalSourceErrorV1> {
    let CanonicalFocusedObjectV1::Fields { shape, fields } = object else {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    };
    for field in fields {
        let mut selected = parent.clone();
        selected
            .field_path
            .push(declared_field(plan, shape, &field.name)?);
        match &field.value {
            CanonicalFocusedObjectV1::Fields { .. } => {
                structured_effects(cst, plan, &field.value, &selected, effects)?;
            }
            CanonicalFocusedObjectV1::Source(expression) => {
                let exact = cst
                    .source_slice(field.origin)
                    .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
                let Some(prefix) = exact.trim_ascii_end().strip_suffix(expression.as_slice())
                else {
                    continue;
                };
                selected.expression_origin = CanonicalSourceOriginV1 {
                    artifact: cst.artifact,
                    start: field.origin.start + prefix.len() as u64,
                    end: field.origin.start + (prefix.len() + expression.len()) as u64,
                };
                selected.expression = expression.clone();
                effects.push(selected);
            }
        }
    }
    Ok(())
}

pub fn replace_canonical_scalar_effect_v1(
    cst: &CanonicalSourceCstV1,
    old_plan: &CanonicalSourceAllocationPlanV1,
    selected: &CanonicalScalarEffectV1,
    replacement: &[u8],
    new_root: ProgramChangeOccurrenceId,
) -> Result<CanonicalSourceEditV1, CanonicalSourceErrorV1> {
    let offered = canonical_scalar_effects_v1(cst, old_plan)?;
    if !offered.contains(selected) {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    replace_bound_scalar_effect(cst, old_plan, selected, replacement, new_root)
}

pub(super) fn replace_bound_scalar_effect(
    cst: &CanonicalSourceCstV1,
    old_plan: &CanonicalSourceAllocationPlanV1,
    selected: &CanonicalScalarEffectV1,
    replacement: &[u8],
    new_root: ProgramChangeOccurrenceId,
) -> Result<CanonicalSourceEditV1, CanonicalSourceErrorV1> {
    if new_root == old_plan.root {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    let expression = std::str::from_utf8(replacement)
        .map_err(|_| CanonicalSourceErrorV1::RecordedPlanMismatch)?;
    if expression.len() > MAX_CANONICAL_TEXT_BYTES
        || expression.contains(['\n', '\r'])
        || expression.trim() != expression
        || parse_scalar_expression(expression, "").is_none()
    {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    let start = usize::try_from(selected.expression_origin.start)
        .map_err(|_| CanonicalSourceErrorV1::RecordedPlanMismatch)?;
    let end = usize::try_from(selected.expression_origin.end)
        .map_err(|_| CanonicalSourceErrorV1::RecordedPlanMismatch)?;
    let mut exact = Vec::with_capacity(cst.exact_source.len() + replacement.len());
    exact.extend_from_slice(&cst.exact_source[..start]);
    exact.extend_from_slice(replacement);
    exact.extend_from_slice(&cst.exact_source[end..]);
    let source = incremental_read::replace_scalar_leaf(cst, selected, &exact, replacement.len())?;
    let plan = build_independent_plan(&source, new_root)?;
    let old_requests = allocation_requests(cst)?;
    let new_requests = allocation_requests(&source)?;
    let handler_producer = |tree: &CanonicalSourceCstV1| {
        tree.items.iter().find_map(|item| {
            if item.origin.start != selected.handler_origin.start {
                return None;
            }
            match &item.kind {
                CstKind::InputHandler(handler) => Some(handler.producer.clone()),
                CstKind::ScalarHandler(handler) => Some(handler.producer.clone()),
                CstKind::GeneralHandler(handler) => Some(handler.producer.clone()),
                _ => None,
            }
        }).ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)
    };
    // Replaying this exact leaf edit explicitly continues its enclosing handler;
    // independent reads use the complete clause, including effects, as its key.
    let old_producer = handler_producer(cst)?;
    let new_producer = handler_producer(&source)?;
    let continued_requests = old_requests.iter().map(|request| {
        let mut continued = request.clone();
        if continued.producer == old_producer {
            continued.producer = new_producer.clone();
        }
        continued
    }).collect::<Vec<_>>();
    // The operation cannot add/delete a declaration, membership, state cell,
    // handler, or facet. Verify the parser's emission graph respects that.
    if old_requests.len() != new_requests.len() {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    let mut retained = BTreeMap::new();
    for (request, continued) in old_requests.iter().zip(&continued_requests) {
        let old = old_plan
            .identity(&request.producer, &request.slot, request.domain)
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        if old == CanonicalAllocatedIdentityV1::Formation(selected.effect) {
            if !selected.field_path.is_empty() {
                // The single leaf replacement explicitly continues its parent
                // occurrence. All other requests must survive unchanged. This
                // is operation replay, not matching an imported tree by text.
                let continuing = new_requests
                    .iter()
                    .filter(|candidate| *candidate == continued || !continued_requests.contains(candidate))
                    .collect::<Vec<_>>();
                let [new] = continuing.as_slice() else {
                    return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
                };
                if new.producer != continued.producer
                    || new.domain != request.domain
                    || new.slot.production != CanonicalSourceProductionV1::HandlerInclude
                {
                    return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
                }
                retained.insert(
                    old,
                    plan.identity(&new.producer, &new.slot, new.domain)
                        .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?,
                );
            }
            continue;
        }
        if new_requests.binary_search(continued).is_err() {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        let new = plan
            .identity(&continued.producer, &continued.slot, continued.domain)
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        retained.insert(old, new);
    }
    Ok(CanonicalSourceEditV1 {
        old_artifact: cst.artifact(), old_root: old_plan.root(),
        scalar_change: Some((selected.handler, selected.expression_origin.start, selected.expression_origin.end, replacement.len() as u64)),
        source,
        plan,
        retained,
        retained_index: std::sync::OnceLock::new(),
    })
}

/// A checked occurrence offered for whole-item replacement. The identity
/// selects it; the origin and exact bytes are replay/display data, not identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalEditableSourceItemV1 {
    pub identity: CanonicalAllocatedIdentityV1,
    pub production: CanonicalSourceProductionV1,
    pub origin: CanonicalSourceOriginV1,
    pub source: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalSourceItemReplacementV1 {
    pub selected: CanonicalEditableSourceItemV1,
    pub replacement: Vec<u8>,
}

fn editable_item_producer(item: &CstItem) -> Option<CanonicalSemanticProducerV1> {
    Some(match &item.kind {
        CstKind::Relation(relation) if relation.contract_origin.is_none() => {
            semantic_producer(CanonicalSourceProductionV1::Relation, &relation.designation)
        }
        CstKind::ScalarLaw(law) => {
            semantic_producer(CanonicalSourceProductionV1::Law, &law.designation)
        }
        CstKind::BooleanLaw(law) => {
            semantic_producer(CanonicalSourceProductionV1::Law, &law.designation)
        }
        CstKind::GeneralHandler(handler) => handler.producer.clone(),
        CstKind::ScalarHandler(handler) => handler.producer.clone(),
        CstKind::InputHandler(handler) => handler.producer.clone(),
        _ => return None,
    })
}

pub fn canonical_editable_source_items_v1(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
) -> Result<Vec<CanonicalEditableSourceItemV1>, CanonicalSourceErrorV1> {
    rematerialize_canonical_source_allocation_plan_v1(cst, plan)?;
    let mut offered = Vec::new();
    for item in &cst.items {
        let Some(producer) = editable_item_producer(item) else {
            continue;
        };
        offered.push(CanonicalEditableSourceItemV1 {
            identity: CanonicalAllocatedIdentityV1::Formation(formation_id(
                plan,
                &producer,
                &head_slot(producer.production),
            )?),
            production: producer.production,
            origin: item.origin,
            source: cst
                .source_slice(item.origin)
                .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?
                .to_vec(),
        });
        if let CstKind::Relation(relation) = &item.kind {
            for mode in &relation.modes {
                offered.push(CanonicalEditableSourceItemV1 {
                    identity: plan
                        .identity(
                            &producer,
                            &child_slot(CanonicalSourceProductionV1::RelationMode, &mode.canonical),
                            AllocationDomain::Mode,
                        )
                        .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?,
                    production: CanonicalSourceProductionV1::RelationMode,
                    origin: mode.origin,
                    source: cst
                        .source_slice(mode.origin)
                        .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?
                        .to_vec(),
                });
            }
        }
    }
    Ok(offered)
}

/// Replace a nonoverlapping batch on one exact tree. Each replacement must
/// produce exactly one construct of the selected kind. Its head continues;
/// its children are fresh. Copied constructs retain their allocations by
/// operation replay, including handlers whose expanded laws have changed.
/// State-schema changes are not licensed: runtime continuity checks must still
/// account for every live slot and referent before admitting the successor.
pub fn replace_canonical_source_items_v1(
    cst: &CanonicalSourceCstV1,
    old_plan: &CanonicalSourceAllocationPlanV1,
    replacements: &[CanonicalSourceItemReplacementV1],
    new_root: ProgramChangeOccurrenceId,
) -> Result<CanonicalSourceEditV1, CanonicalSourceErrorV1> {
    let reject = CanonicalSourceErrorV1::RecordedPlanMismatch;
    let offered = canonical_editable_source_items_v1(cst, old_plan)?;
    if replacements.is_empty() || new_root == old_plan.root {
        return Err(reject);
    }
    if replacements
        .iter()
        .any(|op| !offered.contains(&op.selected))
    {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    let mut operations = replacements
        .iter()
        .filter(|op| op.selected.source != op.replacement)
        .collect::<Vec<_>>();
    operations.sort_by_key(|op| op.selected.origin.start);
    let mut exact = Vec::new();
    let mut cursor = 0;
    let mut segments = Vec::new();
    for op in &operations {
        if !offered.contains(&op.selected) {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        let start = op.selected.origin.start as usize;
        let end = op.selected.origin.end as usize;
        if start < cursor || op.replacement.is_empty() {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        exact.extend_from_slice(&cst.exact_source[cursor..start]);
        let new_start = exact.len() as u64;
        exact.extend_from_slice(&op.replacement);
        segments.push((op.selected.origin, new_start, exact.len() as u64));
        cursor = end;
    }
    exact.extend_from_slice(&cst.exact_source[cursor..]);
    if exact.as_slice() == cst.exact_source.as_ref() {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    let source = read_canonical_source_with_declared_frontend_v1(&exact, &cst.declared_frontend)?;
    let plan = build_independent_plan(&source, new_root)?;
    let new_offered = canonical_editable_source_items_v1(&source, &plan)?;
    let translated_origin =
        |old: CanonicalSourceOriginV1| -> Result<CanonicalSourceOriginV1, CanonicalSourceErrorV1> {
            let mut delta = 0i128;
            for (replaced, start, end) in &segments {
                if old == *replaced {
                    return Ok(CanonicalSourceOriginV1 {
                        artifact: source.artifact,
                        start: *start,
                        end: *end,
                    });
                }
                if old.start < replaced.end && replaced.start < old.end {
                    return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
                }
                if replaced.end <= old.start {
                    delta += i128::from(end - start) - i128::from(replaced.end - replaced.start);
                }
            }
            Ok(CanonicalSourceOriginV1 {
                artifact: source.artifact,
                start: (i128::from(old.start) + delta)
                    .try_into()
                    .map_err(|_| CanonicalSourceErrorV1::RecordedPlanMismatch)?,
                end: (i128::from(old.end) + delta)
                    .try_into()
                    .map_err(|_| CanonicalSourceErrorV1::RecordedPlanMismatch)?,
            })
        };
    let mut retained = BTreeMap::new();
    for old in &offered {
        let origin = translated_origin(old.origin)?;
        let matches = new_offered
            .iter()
            .filter(|item| item.origin == origin && item.production == old.production)
            .collect::<Vec<_>>();
        let [new] = matches.as_slice() else {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        };
        retained.insert(old.identity, new.identity);
    }
    let mut producers = BTreeMap::new();
    for old in &cst.items {
        let Some(old_producer) = editable_item_producer(old) else {
            continue;
        };
        let origin = translated_origin(old.origin)?;
        let new = source
            .items
            .iter()
            .find(|item| item.origin == origin)
            .and_then(|item| editable_item_producer(item))
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        if old_producer.production != new.production {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        let replaced = operations.iter().any(|op| op.selected.origin == old.origin);
        producers.insert(old_producer, (new, replaced));
    }
    let new_requests = allocation_requests(&source)?;
    for request in allocation_requests(cst)? {
        let old = old_plan
            .identity(&request.producer, &request.slot, request.domain)
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        if retained.contains_key(&old) {
            continue;
        }
        let mut continued = request.clone();
        if let Some((new, replaced)) = producers.get(&request.producer) {
            if *replaced {
                continue;
            }
            continued.producer = new.clone();
        }
        if new_requests.binary_search(&continued).is_err() {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        let new = plan
            .identity(&continued.producer, &continued.slot, continued.domain)
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        retained.insert(old, new);
    }
    if retained.values().collect::<BTreeSet<_>>().len() != retained.len() {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    Ok(CanonicalSourceEditV1 {
        old_artifact: cst.artifact(), old_root: old_plan.root(), scalar_change: None,
        source,
        plan,
        retained,
        retained_index: std::sync::OnceLock::new(),
    })
}
