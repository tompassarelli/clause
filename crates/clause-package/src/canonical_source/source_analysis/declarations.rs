use super::*;
use std::collections::HashMap;

pub(in super::super) struct SourceDeclarations {
    pub formations: Vec<FormationJudgmentPreimageV2>,
    pub schemas: Vec<RelationSchemaPreimageV2>,
    pub capabilities: Vec<CapabilityDeclarationPreimageV2>,
    pub operators: Vec<OperatorPreimageV2>,
    pub emissions: Vec<CanonicalSourceEmissionV1>,
    pub denotations: Vec<CanonicalSourceDenotationV1>,
    pub unsupported: Vec<CanonicalUnsupportedProductionV1>,
}

struct Rebinding<'a> {
    edit: &'a CanonicalSourceEditV1,
    identities: HashMap<CanonicalAllocatedIdentityV1, CanonicalAllocatedIdentityV1>,
    allocations: HashMap<CanonicalAllocatedIdentityV1, &'a CanonicalAllocationV1>,
}

impl Rebinding<'_> {
    fn identity(
        &self,
        old: CanonicalAllocatedIdentityV1,
    ) -> Result<CanonicalAllocatedIdentityV1, CanonicalSourceErrorV1> {
        self.identities
            .get(&old)
            .copied()
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)
    }
    fn formation(&self, old: FormationLocalId) -> Result<FormationLocalId, CanonicalSourceErrorV1> {
        match self.identity(CanonicalAllocatedIdentityV1::Formation(old))? {
            CanonicalAllocatedIdentityV1::Formation(new) => Ok(new),
            _ => Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        }
    }
    fn schema(
        &self,
        old: RelationSchemaLocalId,
    ) -> Result<RelationSchemaLocalId, CanonicalSourceErrorV1> {
        match self.identity(CanonicalAllocatedIdentityV1::RelationSchema(old))? {
            CanonicalAllocatedIdentityV1::RelationSchema(new) => Ok(new),
            _ => Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        }
    }
    fn role(
        &self,
        schema: RelationSchemaLocalId,
        role: RoleLocalId,
    ) -> Result<RoleLocalId, CanonicalSourceErrorV1> {
        match self.identity(CanonicalAllocatedIdentityV1::Role(LocalRoleRefV2 {
            schema,
            role,
        }))? {
            CanonicalAllocatedIdentityV1::Role(new) if new.schema == self.schema(schema)? => {
                Ok(new.role)
            }
            _ => Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        }
    }
    fn operator(&self, old: OperatorLocalId) -> Result<OperatorLocalId, CanonicalSourceErrorV1> {
        match self.identity(CanonicalAllocatedIdentityV1::Operator(old))? {
            CanonicalAllocatedIdentityV1::Operator(new) => Ok(new),
            _ => Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        }
    }
    fn mode(
        &self,
        operator: OperatorLocalId,
        mode: ModeLocalId,
    ) -> Result<ModeLocalId, CanonicalSourceErrorV1> {
        match self.identity(CanonicalAllocatedIdentityV1::Mode(LocalModeRefV2 {
            operator,
            mode,
        }))? {
            CanonicalAllocatedIdentityV1::Mode(new)
                if new.operator == self.operator(operator)? =>
            {
                Ok(new.mode)
            }
            _ => Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        }
    }
    fn capability(
        &self,
        old: CapabilityLocalId,
    ) -> Result<CapabilityLocalId, CanonicalSourceErrorV1> {
        match self.identity(CanonicalAllocatedIdentityV1::Capability(old))? {
            CanonicalAllocatedIdentityV1::Capability(new) => Ok(new),
            _ => Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
        }
    }
    fn formations(&self, values: &mut [FormationLocalId]) -> Result<(), CanonicalSourceErrorV1> {
        for value in values.iter_mut() {
            *value = self.formation(*value)?;
        }
        values.sort();
        Ok(())
    }
    fn dependencies(
        &self,
        values: &mut [LocalSemanticDependencyV2],
    ) -> Result<(), CanonicalSourceErrorV1> {
        use LocalSemanticDependencyV2 as D;
        for value in values.iter_mut() {
            match value {
                D::Formation(id) => *id = self.formation(*id)?,
                D::RelationSchema(id) => *id = self.schema(*id)?,
                D::Role(id) => {
                    *id = LocalRoleRefV2 {
                        schema: self.schema(id.schema)?,
                        role: self.role(id.schema, id.role)?,
                    }
                }
                D::Operator(id) => *id = self.operator(*id)?,
                D::Mode(id) => {
                    *id = LocalModeRefV2 {
                        operator: self.operator(id.operator)?,
                        mode: self.mode(id.operator, id.mode)?,
                    }
                }
                D::Capability(id) => *id = self.capability(*id)?,
                D::ExternalReference(_) => {}
                D::Application(_) => return Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
            }
        }
        values.sort();
        Ok(())
    }
    fn origin(
        &self,
        origin: CanonicalSourceOriginV1,
    ) -> Result<CanonicalSourceOriginV1, CanonicalSourceErrorV1> {
        let (_, start, end, length) = self
            .edit
            .scalar_change
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        if origin.artifact != self.edit.old_artifact {
            return translate_origin(self.edit, origin);
        }
        if origin.start <= start && origin.end >= end {
            let new_end = origin
                .end
                .checked_sub(end - start)
                .and_then(|value| value.checked_add(length))
                .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
            return Ok(CanonicalSourceOriginV1 {
                artifact: self.edit.source().artifact(),
                start: origin.start,
                end: new_end,
            });
        }
        translate_origin(self.edit, origin)
    }
    fn emission(
        &self,
        old: &CanonicalSourceEmissionV1,
    ) -> Result<CanonicalSourceEmissionV1, CanonicalSourceErrorV1> {
        if old.allocations.is_empty() {
            return Ok(CanonicalSourceEmissionV1 {
                producer: old.producer.clone(),
                slot: old.slot.clone(),
                origin: translate_origin(self.edit, old.origin)?,
                allocations: vec![],
            });
        }
        let allocations = old
            .allocations
            .iter()
            .map(|allocation| {
                self.allocations
                    .get(&self.identity(allocation.identity)?)
                    .map(|allocation| (*allocation).clone())
                    .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let first = allocations
            .first()
            .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
        let CanonicalAllocationJudgmentV1::Fresh {
            producer,
            slot: CanonicalAllocationSlotV1::Emission(slot),
            ..
        } = &first.judgment;
        if allocations.iter().any(|allocation| {
            let CanonicalAllocationJudgmentV1::Fresh {
                producer: other,
                slot: CanonicalAllocationSlotV1::Emission(other_slot),
                ..
            } = &allocation.judgment;
            producer != other || slot != other_slot
        }) {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
        Ok(CanonicalSourceEmissionV1 {
            producer: producer.clone(),
            slot: slot.clone(),
            origin: self.origin(old.origin)?,
            allocations,
        })
    }
}

pub(in super::super) fn rebind_declarations(
    previous: &CanonicalSourcePackageSliceV1,
    edit: &CanonicalSourceEditV1,
) -> Result<SourceDeclarations, CanonicalSourceErrorV1> {
    let old = previous.checked_package.constitution().preimage();
    let mut identities = edit
        .retained()
        .iter()
        .map(|(old, new)| (*old, *new))
        .collect::<HashMap<_, _>>();
    let mapped = identities.values().copied().collect::<BTreeSet<_>>();
    let missing_old = previous
        .emissions
        .iter()
        .flat_map(|emission| &emission.allocations)
        .map(|allocation| allocation.identity)
        .filter(|id| !identities.contains_key(id))
        .collect::<BTreeSet<_>>();
    let missing_new = edit
        .plan()
        .allocations()
        .iter()
        .map(|allocation| allocation.identity)
        .filter(|id| !mapped.contains(id))
        .collect::<BTreeSet<_>>();
    // The edited effect is newly allocated, never added to occurrence continuity.
    // The checked leaf operation preserves every other allocation request.
    match (
        missing_old.into_iter().collect::<Vec<_>>().as_slice(),
        missing_new.into_iter().collect::<Vec<_>>().as_slice(),
    ) {
        ([], []) => {}
        (
            [old @ CanonicalAllocatedIdentityV1::Formation(_)],
            [new @ CanonicalAllocatedIdentityV1::Formation(_)],
        ) => {
            identities.insert(*old, *new);
        }
        _ => return Err(CanonicalSourceErrorV1::RecordedPlanMismatch),
    }
    let rebind = Rebinding {
        edit,
        identities,
        allocations: edit
            .plan()
            .allocations()
            .iter()
            .map(|allocation| (allocation.identity, allocation))
            .collect(),
    };
    let scope = TermScope {
        universe: old.universe,
        semantics: old.semantics,
    };
    let mut origins = HashMap::new();
    let emissions = previous
        .emissions
        .iter()
        .map(|emission| {
            let next = rebind.emission(emission)?;
            for allocation in &emission.allocations {
                if let CanonicalAllocatedIdentityV1::Formation(id) = allocation.identity {
                    origins.insert(id, next.origin);
                }
            }
            Ok(next)
        })
        .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?;
    let formations = old
        .formations
        .iter()
        .map(|formation| {
            let origin = *origins
                .get(&formation.id)
                .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
            let source = edit
                .source()
                .source_slice(origin)
                .ok_or(CanonicalSourceErrorV1::RecordedPlanMismatch)?;
            let mut next = formation.clone();
            next.id = rebind.formation(formation.id)?;
            next.context = vec![origin_term(scope, origin)?];
            if next.term.as_atom().is_none_or(|atom| {
                atom.kind() != b"clause/canonical-source-slice-v1"
                    || atom.canonical_payload() != source
            }) {
                next.term = source_term(scope, source)?;
            }
            rebind.dependencies(&mut next.direct_dependencies)?;
            Ok(next)
        })
        .collect::<Result<_, CanonicalSourceErrorV1>>()?;
    let mut schemas = old.schemas.clone();
    for schema in &mut schemas {
        let old_schema = schema.id;
        schema.id = rebind.schema(old_schema)?;
        for role in &mut schema.roles {
            role.id = rebind.role(old_schema, role.id)?;
            rebind.dependencies(&mut role.direct_dependencies)?;
        }
        schema.roles.sort_by_key(|role| role.id);
        rebind.formations(&mut schema.constraints)?;
        rebind.dependencies(&mut schema.direct_dependencies)?;
    }
    let mut capabilities = old.capabilities.clone();
    for capability in &mut capabilities {
        capability.id = rebind.capability(capability.id)?;
        capability.formation = rebind.formation(capability.formation)?;
        rebind.dependencies(&mut capability.direct_dependencies)?;
    }
    let mut operators = old.operators.clone();
    for operator in &mut operators {
        let old_operator = operator.id;
        operator.id = rebind.operator(old_operator)?;
        rebind.dependencies(&mut operator.direct_dependencies)?;
        for mode in &mut operator.modes {
            let schema = mode.schema;
            mode.id = rebind.mode(old_operator, mode.id)?;
            mode.schema = rebind.schema(schema)?;
            for roles in [&mut mode.known_roles, &mut mode.produced_roles] {
                for role in roles.iter_mut() {
                    *role = rebind.role(schema, *role)?;
                }
                roles.sort();
            }
            rebind.formations(&mut mode.static_basis.context_requirements)?;
            rebind.dependencies(&mut mode.static_basis.constitutive_dependencies)?;
            for requirement in &mut mode.authorization_requirements {
                requirement.kind = rebind.formation(requirement.kind)?;
            }
            // The source declaration profile emits no dynamic prerequisite clauses.
            if !mode.dynamic_prerequisites.is_empty() {
                return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
            }
            if let ResultOrderContractV2::SelectedBy(id) = &mut mode.contract.result_order {
                *id = rebind.formation(*id)?;
            }
            for intent in &mut mode.contract.effect_intents {
                intent.action_role = rebind.role(schema, intent.action_role)?;
                intent.resource_role = rebind.role(schema, intent.resource_role)?;
                intent.payload_role = rebind.role(schema, intent.payload_role)?;
                intent.required_capability = rebind.capability(intent.required_capability)?;
            }
            rebind.formations(&mut mode.contract.productivity.obligations)?;
            rebind.formations(&mut mode.contract.scheduling_requirements)?;
            rebind.formations(&mut mode.contract.resource_requirements)?;
            for capability in &mut mode.contract.capability_requirements {
                *capability = rebind.capability(*capability)?;
            }
            mode.contract.capability_requirements.sort();
            rebind.dependencies(&mut mode.direct_dependencies)?;
        }
        operator.modes.sort_by_key(|mode| mode.id);
    }
    let mut denotations = previous.denotations.clone();
    for denotation in &mut denotations {
        denotation.origin = rebind.origin(denotation.origin)?;
    }
    let unsupported = previous
        .unsupported
        .iter()
        .map(|production| {
            Ok(CanonicalUnsupportedProductionV1 {
                production: production.production,
                origin: rebind.origin(production.origin)?,
                emissions: production
                    .emissions
                    .iter()
                    .map(|emission| rebind.emission(emission))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect::<Result<_, CanonicalSourceErrorV1>>()?;
    Ok(SourceDeclarations {
        formations,
        schemas,
        capabilities,
        operators,
        emissions,
        denotations,
        unsupported,
    })
}
