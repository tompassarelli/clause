//! Explicit source operations, not a text-diff identity heuristic.
use super::*;

/// A snapshot-scoped editable scalar effect. Origins locate source for display
/// and replacement; the artifact and allocated identities select the node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalScalarEffectV1 {
    pub artifact: CanonicalSourceArtifactIdV1,
    pub handler: FormationLocalId,
    pub effect: FormationLocalId,
    pub handler_origin: CanonicalSourceOriginV1,
    pub expression_origin: CanonicalSourceOriginV1,
    pub expression: Vec<u8>,
}

/// The only constructor replays one expression replacement on the exact old
/// tree. Every other allocation is retained by that operation, NOT inferred
/// from equal names, values, source spans, or arbitrary imported text.
/// Local coordinates in the new snapshot are fresh; these pairs explicitly
/// connect their continuing semantic occurrences across snapshot addresses.
#[derive(Clone, Debug)]
pub struct CanonicalScalarEditV1 {
    source: CanonicalSourceCstV1,
    plan: CanonicalSourceAllocationPlanV1,
    retained: BTreeMap<CanonicalAllocatedIdentityV1, CanonicalAllocatedIdentityV1>,
}

impl CanonicalScalarEditV1 {
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
        match self
            .retained
            .get(&CanonicalAllocatedIdentityV1::Formation(old))
        {
            Some(CanonicalAllocatedIdentityV1::Formation(new)) => Ok(*new),
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
        let role = |old| match self.retained.get(&CanonicalAllocatedIdentityV1::Role(old)) {
            Some(CanonicalAllocatedIdentityV1::Role(new)) => Ok(*new),
            _ => Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        };
        let relation = match self
            .retained
            .get(&CanonicalAllocatedIdentityV1::RelationSchema(old.relation))
        {
            Some(CanonicalAllocatedIdentityV1::RelationSchema(new)) => *new,
            _ => return Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        };
        let mut new = old.clone();
        new.assertion = self.formation(old.assertion)?;
        new.relation = relation;
        new.subject_role = role(old.subject_role)?;
        new.value_role = role(old.value_role)?;
        new.subject_identity = old
            .subject_identity
            .map(|value| self.referent(value))
            .transpose()?;
        if let CanonicalStatePathV1::Field { formation, .. } = &mut new.path {
            *formation = self.formation(*formation)?;
        }
        Ok(new)
    }
}

pub fn canonical_scalar_effects_v1(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
) -> Result<Vec<CanonicalScalarEffectV1>, CanonicalSourceErrorV1> {
    rematerialize_canonical_source_allocation_plan_v1(cst, plan)?;
    let mut effects = Vec::new();
    for item in &cst.items {
        let (producer, origin, includes) = match &item.kind {
            CstKind::GeneralHandler(handler) => (
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
            let exact = std::str::from_utf8(
                cst.source_slice(include.origin)
                    .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?,
            )
            .map_err(|_| CanonicalSourceErrorV1::RecordedPlanMismatch)?;
            let line = exact.trim();
            // Structured products need their own field operation; accepting a
            // whole row here would let callers replace a state binding.
            if split_shape_subject(line).is_some() {
                continue;
            }
            let Some((subject, relation, _)) = split_general_scalar_insertion(line) else {
                continue;
            };
            let tail = line[subject.len()..].trim_start();
            let expression = tail[relation.len()..].trim_start();
            let offset = exact.len() - exact.trim_start().len() + line.len() - expression.len();
            effects.push(CanonicalScalarEffectV1 {
                artifact: cst.artifact,
                handler,
                effect: formation_id(
                    plan,
                    producer,
                    &child_slot(CanonicalSourceProductionV1::HandlerInclude, &include.local),
                )?,
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

pub fn replace_canonical_scalar_effect_v1(
    cst: &CanonicalSourceCstV1,
    old_plan: &CanonicalSourceAllocationPlanV1,
    selected: &CanonicalScalarEffectV1,
    replacement: &[u8],
    new_root: ProgramChangeOccurrenceId,
) -> Result<CanonicalScalarEditV1, CanonicalSourceErrorV1> {
    let offered = canonical_scalar_effects_v1(cst, old_plan)?;
    if !offered.contains(selected) || new_root == old_plan.root {
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
    let source = read_canonical_source_v1(&exact)?;
    let plan = build_independent_plan(&source, new_root)?;
    let old_requests = allocation_requests(cst)?;
    let new_requests = allocation_requests(&source)?;
    // The operation cannot add/delete a declaration, membership, state cell,
    // handler, or facet. Verify the parser's emission graph respects that.
    if old_requests.len() != new_requests.len() {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    let mut retained = BTreeMap::new();
    for request in old_requests {
        let old = old_plan
            .identity(&request.producer, &request.slot, request.domain)
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        if old == CanonicalAllocatedIdentityV1::Formation(selected.effect) {
            continue;
        }
        if !new_requests.contains(&request) {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        let new = plan
            .identity(&request.producer, &request.slot, request.domain)
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        retained.insert(old, new);
    }
    Ok(CanonicalScalarEditV1 {
        source,
        plan,
        retained,
    })
}
