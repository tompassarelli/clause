use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use crate::canonical::{
    CanonicalEncodeError, ProgramSnapshotPreimageV2, encode_application_shape_preimage_v2,
    encode_program_snapshot_preimage_v2,
};
use crate::hash::{derive_application_shape_id, derive_program_snapshot_id};
use crate::identity::*;
use crate::provenance::{ActivationPrerequisiteKind, PrerequisiteScope};
use crate::term::Term;

const MAX_V2_ITEMS: usize = 1_000_000;
const MAX_V2_DEPENDENCY_EDGES: usize = 4_000_000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocalRoleRefV2 {
    pub schema: RelationSchemaLocalId,
    pub role: RoleLocalId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocalModeRefV2 {
    pub operator: OperatorLocalId,
    pub mode: ModeLocalId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LocalSemanticDependencyV2 {
    Formation(FormationLocalId),
    RelationSchema(RelationSchemaLocalId),
    Role(LocalRoleRefV2),
    Operator(OperatorLocalId),
    Mode(LocalModeRefV2),
    Application(ApplicationLocalId),
    Capability(CapabilityLocalId),
    ExternalReference(Term),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SemanticDependencyV2 {
    Formation(FormationRefV2),
    RelationSchema(RelationSchemaId),
    Role(RoleId),
    Operator(OperatorRef),
    Mode(ModeId),
    Application(ApplicationId),
    Capability(CapabilityRef),
    ExternalReference(Term),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FormationTargetV2 {
    pub type_term: Term,
    pub interpretation: Term,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FormationJudgmentPreimageV2 {
    pub id: FormationLocalId,
    pub context: Vec<Term>,
    pub term: Term,
    pub target: FormationTargetV2,
    pub direct_dependencies: Vec<LocalSemanticDependencyV2>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CardinalityV2 {
    pub minimum: u32,
    pub maximum: Option<u32>,
}

impl CardinalityV2 {
    #[must_use]
    pub const fn contains(self, count: u32) -> bool {
        count >= self.minimum
            && match self.maximum {
                Some(maximum) => count <= maximum,
                None => true,
            }
    }

    #[must_use]
    pub const fn is_exactly_one(self) -> bool {
        self.minimum == 1 && matches!(self.maximum, Some(1))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RoleDeclarationPreimageV2 {
    pub id: RoleLocalId,
    pub target: FormationTargetV2,
    pub cardinality: CardinalityV2,
    pub direct_dependencies: Vec<LocalSemanticDependencyV2>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationSchemaPreimageV2 {
    pub id: RelationSchemaLocalId,
    pub roles: Vec<RoleDeclarationPreimageV2>,
    pub constraints: Vec<FormationLocalId>,
    pub result_domain: FormationTargetV2,
    pub direct_dependencies: Vec<LocalSemanticDependencyV2>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapabilityDeclarationPreimageV2 {
    pub id: CapabilityLocalId,
    pub formation: FormationLocalId,
    pub direct_dependencies: Vec<LocalSemanticDependencyV2>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DeterminismContractV2 {
    Deterministic,
    ExplicitlyNondeterministic,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ResultOrderContractV2 {
    UnorderedFiniteSet,
    OrderedStream,
    SelectedBy(FormationLocalId),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductivityKindV2 {
    Total,
    Productive,
    Bounded,
    Partial,
    Reactive,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProductivityContractV2 {
    pub kind: ProductivityKindV2,
    pub obligations: Vec<FormationLocalId>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContinuationUseV2 {
    Linear,
    Reusable,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContinuationContractV2 {
    TerminalOnly {
        may_cancel: bool,
    },
    Suspensible {
        use_policy: ContinuationUseV2,
        may_handoff: bool,
        may_cancel: bool,
    },
}

impl ContinuationContractV2 {
    #[must_use]
    pub const fn may_suspend(self) -> bool {
        matches!(self, Self::Suspensible { .. })
    }

    #[must_use]
    pub const fn may_handoff(self) -> bool {
        matches!(
            self,
            Self::Suspensible {
                may_handoff: true,
                ..
            }
        )
    }

    #[must_use]
    pub const fn may_cancel(self) -> bool {
        match self {
            Self::TerminalOnly { may_cancel } | Self::Suspensible { may_cancel, .. } => may_cancel,
        }
    }

    #[must_use]
    pub const fn use_policy(self) -> Option<ContinuationUseV2> {
        match self {
            Self::TerminalOnly { .. } => None,
            Self::Suspensible { use_policy, .. } => Some(use_policy),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EffectIntentContractPreimageV2 {
    pub intent_domain: FormationTargetV2,
    pub required_capability: CapabilityLocalId,
}

/// Static enabling material for `Executable(G, A, M, kappa)`. Constitutive
/// citations belong here as dependencies; they are not causal occurrences.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StaticActivationBasisPreimageV2 {
    pub context_requirements: Vec<FormationLocalId>,
    pub constitutive_dependencies: Vec<LocalSemanticDependencyV2>,
}

/// An exact Mode-owned authorization requirement. The set of requirements may
/// be empty. Actual authorization evidence remains orthogonal to causality.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AuthorizationRequirementPreimageV2 {
    pub kind: FormationLocalId,
    pub cardinality: CardinalityV2,
}

/// A causal prerequisite whose satisfying occurrence must remain visible in
/// the Activation frontier and cannot be erased as a static fixed grant.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DynamicPrerequisiteRequirementPreimageV2 {
    pub slot: PrerequisiteLocalId,
    pub role: Option<RoleLocalId>,
    pub requirement: ActivationPrerequisiteKind,
    pub expected: FormationLocalId,
    pub scope: PrerequisiteScope,
    pub cardinality: CardinalityV2,
    pub cause_projection: Vec<crate::provenance::CauseProjectionEntryV2>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ModeContractV2 {
    pub determinism: DeterminismContractV2,
    pub result_cardinality: CardinalityV2,
    pub result_order: ResultOrderContractV2,
    pub failure_domain: Option<FormationTargetV2>,
    pub state_delta_domain: Option<FormationTargetV2>,
    pub budget_exhaustion_domain: Option<FormationTargetV2>,
    pub effect_intents: Vec<EffectIntentContractPreimageV2>,
    /// Exact domains this Mode is constituted to check and emit as Formation
    /// observations. This is semantic typing capacity, not policy authority.
    pub formation_checks: Vec<FormationTargetV2>,
    pub productivity: ProductivityContractV2,
    pub scheduling_requirements: Vec<FormationLocalId>,
    pub resource_requirements: Vec<FormationLocalId>,
    pub capability_requirements: Vec<CapabilityLocalId>,
    pub continuation: ContinuationContractV2,
}

impl ModeContractV2 {
    #[must_use]
    pub fn is_pure(&self) -> bool {
        self.state_delta_domain.is_none() && self.effect_intents.is_empty()
    }

    #[must_use]
    pub fn is_function(&self) -> bool {
        self.is_pure()
            && self.determinism == DeterminismContractV2::Deterministic
            && self.result_cardinality.is_exactly_one()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ModePreimageV2 {
    pub id: ModeLocalId,
    pub schema: RelationSchemaLocalId,
    pub known_roles: Vec<RoleLocalId>,
    pub produced_roles: Vec<RoleLocalId>,
    pub static_basis: StaticActivationBasisPreimageV2,
    pub authorization_requirements: Vec<AuthorizationRequirementPreimageV2>,
    pub dynamic_prerequisites: Vec<DynamicPrerequisiteRequirementPreimageV2>,
    pub contract: ModeContractV2,
    pub direct_dependencies: Vec<LocalSemanticDependencyV2>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OperatorPreimageV2 {
    pub id: OperatorLocalId,
    pub modes: Vec<ModePreimageV2>,
    pub direct_dependencies: Vec<LocalSemanticDependencyV2>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RoleBindingValuePreimageV2 {
    Known(FormationLocalId),
    Binder(FormationLocalId),
    Produced,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RoleBindingPreimageV2 {
    pub role: RoleLocalId,
    pub occurrence: u32,
    pub value: RoleBindingValuePreimageV2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConstraintDischargePreimageV2 {
    pub constraint: FormationLocalId,
    pub evidence: FormationLocalId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationFormPreimageV2 {
    pub formation: FormationLocalId,
    pub schema: RelationSchemaLocalId,
    pub operator: OperatorLocalId,
    pub eligible_modes: Vec<ModeLocalId>,
    pub bindings: Vec<RoleBindingPreimageV2>,
    pub context_requirements: Vec<FormationLocalId>,
    pub constraint_discharges: Vec<ConstraintDischargePreimageV2>,
    pub result_domain: FormationTargetV2,
    pub direct_dependencies: Vec<LocalSemanticDependencyV2>,
    pub dependency_closure: Vec<LocalSemanticDependencyV2>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationDeclarationPreimageV2 {
    pub id: ApplicationLocalId,
    pub form: ApplicationFormPreimageV2,
}

/// Canonical local-reference material hashed before `ProgramSnapshotId` exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramConstitutionPreimageV2 {
    pub semantics: ClauseSemanticsId,
    pub universe: UniverseId,
    pub formations: Vec<FormationJudgmentPreimageV2>,
    pub schemas: Vec<RelationSchemaPreimageV2>,
    pub capabilities: Vec<CapabilityDeclarationPreimageV2>,
    pub operators: Vec<OperatorPreimageV2>,
    pub applications: Vec<ApplicationDeclarationPreimageV2>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ResolvedRoleBindingValueV2 {
    Known(FormationRefV2),
    Binder(FormationRefV2),
    Produced,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ResolvedRoleBindingV2 {
    pub role: RoleId,
    pub occurrence: u32,
    pub value: ResolvedRoleBindingValueV2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ResolvedConstraintDischargeV2 {
    pub constraint: FormationRefV2,
    pub evidence: FormationRefV2,
}

/// Exact post-snapshot shape input. `claimed_shape` and nominal ApplicationId
/// are deliberately absent.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationShapePreimageV2 {
    pub semantics: ClauseSemanticsId,
    pub snapshot: ProgramSnapshotId,
    pub term: Term,
    pub formation: FormationRefV2,
    pub schema: RelationSchemaId,
    pub operator: OperatorRef,
    pub eligible_modes: Vec<ModeId>,
    pub bindings: Vec<ResolvedRoleBindingV2>,
    pub context_requirements: Vec<FormationRefV2>,
    pub constraint_discharges: Vec<ResolvedConstraintDischargeV2>,
    pub result_domain: FormationTargetV2,
    pub dependency_closure: Vec<SemanticDependencyV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedStaticActivationBasisV2 {
    pub context_requirements: Vec<FormationRefV2>,
    pub constitutive_dependencies: Vec<SemanticDependencyV2>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedAuthorizationRequirementV2 {
    pub kind: FormationRefV2,
    pub cardinality: CardinalityV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedDynamicPrerequisiteRequirementV2 {
    pub slot: PrerequisiteSlotId,
    pub role: Option<RoleId>,
    pub requirement: ActivationPrerequisiteKind,
    pub expected: FormationRefV2,
    pub scope: PrerequisiteScope,
    pub cardinality: CardinalityV2,
    pub cause_projection: Vec<crate::provenance::CauseProjectionEntryV2>,
}

/// Static result of checking `Executable(G, A, M, kappa)`. Authorization
/// evidence and dynamic causal prerequisite occurrences are checked later.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableContractV2 {
    pub application: ApplicationId,
    pub mode: ModeId,
    pub result_domain: FormationTargetV2,
    pub application_context_requirements: Vec<FormationRefV2>,
    pub application_dependency_closure: Vec<SemanticDependencyV2>,
    pub static_basis: ResolvedStaticActivationBasisV2,
    pub authorization_requirements: Vec<ResolvedAuthorizationRequirementV2>,
    pub dynamic_prerequisites: Vec<ResolvedDynamicPrerequisiteRequirementV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedProgramConstitutionV2 {
    snapshot: ProgramSnapshotId,
    exact_snapshot_preimage: Box<[u8]>,
    preimage: ProgramConstitutionPreimageV2,
    application_shapes: BTreeMap<ApplicationLocalId, ApplicationShapeId>,
}

impl ResolvedProgramConstitutionV2 {
    #[must_use]
    pub const fn snapshot(&self) -> ProgramSnapshotId {
        self.snapshot
    }

    #[must_use]
    pub const fn semantics(&self) -> ClauseSemanticsId {
        self.preimage.semantics
    }

    #[must_use]
    pub const fn universe(&self) -> UniverseId {
        self.preimage.universe
    }

    /// The exact canonical snapshot-preimage bytes from which `snapshot` was
    /// derived. They are retained separately because inserting them into the
    /// semantic constitution would make its identity definition recursive.
    #[must_use]
    pub fn exact_snapshot_preimage_bytes(&self) -> &[u8] {
        &self.exact_snapshot_preimage
    }

    #[must_use]
    pub fn preimage(&self) -> &ProgramConstitutionPreimageV2 {
        &self.preimage
    }

    #[must_use]
    pub fn application_shape(&self, application: ApplicationLocalId) -> Option<ApplicationShapeId> {
        self.application_shapes.get(&application).copied()
    }

    #[must_use]
    pub fn formation(&self, id: FormationLocalId) -> Option<&FormationJudgmentPreimageV2> {
        self.preimage
            .formations
            .binary_search_by_key(&id, |formation| formation.id)
            .ok()
            .map(|index| &self.preimage.formations[index])
    }

    #[must_use]
    pub fn application(&self, id: ApplicationLocalId) -> Option<&ApplicationDeclarationPreimageV2> {
        self.preimage
            .applications
            .binary_search_by_key(&id, |application| application.id)
            .ok()
            .map(|index| &self.preimage.applications[index])
    }

    #[must_use]
    pub fn application_by_id(
        &self,
        id: ApplicationId,
    ) -> Option<&ApplicationDeclarationPreimageV2> {
        (id.snapshot == self.snapshot)
            .then(|| self.application(id.local))
            .flatten()
    }

    #[must_use]
    pub fn mode(&self, id: LocalModeRefV2) -> Option<&ModePreimageV2> {
        let operator_index = self
            .preimage
            .operators
            .binary_search_by_key(&id.operator, |operator| operator.id)
            .ok()?;
        let operator = &self.preimage.operators[operator_index];
        let mode_index = operator
            .modes
            .binary_search_by_key(&id.mode, |mode| mode.id)
            .ok()?;
        Some(&operator.modes[mode_index])
    }

    #[must_use]
    pub fn mode_by_id(&self, id: ModeId) -> Option<&ModePreimageV2> {
        (id.operator.snapshot == self.snapshot)
            .then(|| {
                self.mode(LocalModeRefV2 {
                    operator: id.operator.local,
                    mode: id.local,
                })
            })
            .flatten()
    }

    #[must_use]
    pub fn executable_contract_local(
        &self,
        application: ApplicationLocalId,
        mode: LocalModeRefV2,
    ) -> Option<ExecutableContractV2> {
        let application_record = self.application(application)?;
        if application_record.form.operator != mode.operator
            || application_record
                .form
                .eligible_modes
                .binary_search(&mode.mode)
                .is_err()
        {
            return None;
        }
        let mode_record = self.mode(mode)?;
        Some(ExecutableContractV2 {
            application: resolve_application(self.snapshot, application),
            mode: resolve_mode(self.snapshot, mode),
            result_domain: application_record.form.result_domain.clone(),
            application_context_requirements: application_record
                .form
                .context_requirements
                .iter()
                .copied()
                .map(|local| resolve_formation(self.snapshot, local))
                .collect(),
            application_dependency_closure: application_record
                .form
                .dependency_closure
                .iter()
                .cloned()
                .map(|dependency| resolve_dependency(self.snapshot, dependency))
                .collect(),
            static_basis: ResolvedStaticActivationBasisV2 {
                context_requirements: mode_record
                    .static_basis
                    .context_requirements
                    .iter()
                    .copied()
                    .map(|local| resolve_formation(self.snapshot, local))
                    .collect(),
                constitutive_dependencies: mode_record
                    .static_basis
                    .constitutive_dependencies
                    .iter()
                    .cloned()
                    .map(|dependency| resolve_dependency(self.snapshot, dependency))
                    .collect(),
            },
            authorization_requirements: mode_record
                .authorization_requirements
                .iter()
                .map(|requirement| ResolvedAuthorizationRequirementV2 {
                    kind: resolve_formation(self.snapshot, requirement.kind),
                    cardinality: requirement.cardinality,
                })
                .collect(),
            dynamic_prerequisites: mode_record
                .dynamic_prerequisites
                .iter()
                .map(|requirement| ResolvedDynamicPrerequisiteRequirementV2 {
                    slot: PrerequisiteSlotId {
                        mode: resolve_mode(self.snapshot, mode),
                        local: requirement.slot,
                    },
                    role: requirement
                        .role
                        .map(|role| resolve_role(self.snapshot, mode_record.schema, role)),
                    requirement: requirement.requirement,
                    expected: resolve_formation(self.snapshot, requirement.expected),
                    scope: requirement.scope,
                    cardinality: requirement.cardinality,
                    cause_projection: requirement.cause_projection.clone(),
                })
                .collect(),
        })
    }

    #[must_use]
    pub fn executable_contract(
        &self,
        application: ApplicationId,
        mode: ModeId,
    ) -> Option<ExecutableContractV2> {
        if application.snapshot != self.snapshot || mode.operator.snapshot != self.snapshot {
            return None;
        }
        self.executable_contract_local(
            application.local,
            LocalModeRefV2 {
                operator: mode.operator.local,
                mode: mode.local,
            },
        )
    }
}

/// Canonically encodes and checks one complete local-reference snapshot
/// preimage, derives its snapshot identity exactly once, resolves all
/// snapshot-scoped references, and derives every ApplicationShapeId from its
/// canonical resolved form. Callers supply no identity or hashing callback.
pub fn resolve_program_constitution_v2(
    snapshot_preimage: &ProgramSnapshotPreimageV2,
) -> Result<ResolvedProgramConstitutionV2, FormationErrorV2> {
    let preimage = &snapshot_preimage.constitution;
    let indexes = validate_and_index(preimage)?;
    let exact_snapshot_preimage = encode_program_snapshot_preimage_v2(snapshot_preimage)
        .map_err(FormationErrorV2::CanonicalEncoding)?;
    let snapshot = derive_program_snapshot_id(preimage.semantics, &exact_snapshot_preimage);
    let mut application_shapes = BTreeMap::new();
    let mut shape_preimages = BTreeMap::<ApplicationShapeId, ApplicationShapePreimageV2>::new();

    for application in &preimage.applications {
        validate_application(preimage, &indexes, application)?;
        let shape_preimage = resolve_shape_preimage(preimage, &indexes, snapshot, application)?;
        let canonical_shape = encode_application_shape_preimage_v2(&shape_preimage)
            .map_err(FormationErrorV2::CanonicalEncoding)?;
        let derived_shape =
            derive_application_shape_id(preimage.semantics, snapshot, &canonical_shape);
        if let Some(existing) = shape_preimages.get(&derived_shape) {
            if existing != &shape_preimage {
                return Err(FormationErrorV2::ApplicationShapeCollision(derived_shape));
            }
        } else {
            shape_preimages.insert(derived_shape, shape_preimage);
        }
        application_shapes.insert(application.id, derived_shape);
    }

    Ok(ResolvedProgramConstitutionV2 {
        snapshot,
        exact_snapshot_preimage: exact_snapshot_preimage.into_boxed_slice(),
        preimage: preimage.clone(),
        application_shapes,
    })
}

struct IndexesV2<'a> {
    formations: BTreeMap<FormationLocalId, &'a FormationJudgmentPreimageV2>,
    schemas: BTreeMap<RelationSchemaLocalId, &'a RelationSchemaPreimageV2>,
    roles: BTreeMap<LocalRoleRefV2, &'a RoleDeclarationPreimageV2>,
    capabilities: BTreeMap<CapabilityLocalId, &'a CapabilityDeclarationPreimageV2>,
    operators: BTreeMap<OperatorLocalId, &'a OperatorPreimageV2>,
    modes: BTreeMap<LocalModeRefV2, &'a ModePreimageV2>,
    applications: BTreeMap<ApplicationLocalId, &'a ApplicationDeclarationPreimageV2>,
}

#[derive(Clone, Copy)]
struct ConstitutionScopeV2 {
    semantics: ClauseSemanticsId,
    universe: UniverseId,
}

impl From<&ProgramConstitutionPreimageV2> for ConstitutionScopeV2 {
    fn from(preimage: &ProgramConstitutionPreimageV2) -> Self {
        Self {
            semantics: preimage.semantics,
            universe: preimage.universe,
        }
    }
}

fn validate_and_index(
    preimage: &ProgramConstitutionPreimageV2,
) -> Result<IndexesV2<'_>, FormationErrorV2> {
    let scope = ConstitutionScopeV2::from(preimage);
    validate_aggregate_bounds(preimage)?;
    ensure_count(preimage.formations.len(), "formations")?;
    ensure_count(preimage.schemas.len(), "schemas")?;
    ensure_count(preimage.capabilities.len(), "capabilities")?;
    ensure_count(preimage.operators.len(), "operators")?;
    ensure_count(preimage.applications.len(), "applications")?;

    if !is_strictly_sorted_unique_by(&preimage.formations, |item| item.id)
        || !is_strictly_sorted_unique_by(&preimage.schemas, |item| item.id)
        || !is_strictly_sorted_unique_by(&preimage.capabilities, |item| item.id)
        || !is_strictly_sorted_unique_by(&preimage.operators, |item| item.id)
        || !is_strictly_sorted_unique_by(&preimage.applications, |item| item.id)
    {
        return Err(FormationErrorV2::NonCanonicalSet(
            "constitution declarations",
        ));
    }

    let formations: BTreeMap<_, _> = preimage
        .formations
        .iter()
        .map(|formation| (formation.id, formation))
        .collect();
    let schemas: BTreeMap<_, _> = preimage
        .schemas
        .iter()
        .map(|schema| (schema.id, schema))
        .collect();
    let capabilities: BTreeMap<_, _> = preimage
        .capabilities
        .iter()
        .map(|capability| (capability.id, capability))
        .collect();
    let operators: BTreeMap<_, _> = preimage
        .operators
        .iter()
        .map(|operator| (operator.id, operator))
        .collect();
    let applications: BTreeMap<_, _> = preimage
        .applications
        .iter()
        .map(|application| (application.id, application))
        .collect();

    let mut roles = BTreeMap::new();
    for schema in &preimage.schemas {
        ensure_count(schema.roles.len(), "schema roles")?;
        ensure_count(schema.constraints.len(), "schema constraints")?;
        validate_target(scope, &schema.result_domain)?;
        validate_set(&schema.constraints, "schema constraints")?;
        validate_set(&schema.direct_dependencies, "schema dependencies")?;
        if !is_strictly_sorted_unique_by(&schema.roles, |role| role.id) {
            return Err(FormationErrorV2::NonCanonicalSet("schema roles"));
        }
        for role in &schema.roles {
            validate_cardinality(role.cardinality)?;
            validate_target(scope, &role.target)?;
            validate_set(&role.direct_dependencies, "role dependencies")?;
            roles.insert(
                LocalRoleRefV2 {
                    schema: schema.id,
                    role: role.id,
                },
                role,
            );
        }
    }

    let mut modes = BTreeMap::new();
    for operator in &preimage.operators {
        ensure_count(operator.modes.len(), "operator modes")?;
        validate_set(&operator.direct_dependencies, "operator dependencies")?;
        if !is_strictly_sorted_unique_by(&operator.modes, |mode| mode.id) {
            return Err(FormationErrorV2::NonCanonicalSet("operator modes"));
        }
        for mode in &operator.modes {
            modes.insert(
                LocalModeRefV2 {
                    operator: operator.id,
                    mode: mode.id,
                },
                mode,
            );
        }
    }

    let indexes = IndexesV2 {
        formations,
        schemas,
        roles,
        capabilities,
        operators,
        modes,
        applications,
    };

    for formation in &preimage.formations {
        validate_term(scope, &formation.term)?;
        validate_target(scope, &formation.target)?;
        validate_term_set(scope, &formation.context, "formation context")?;
        validate_set(&formation.direct_dependencies, "formation dependencies")?;
        validate_dependencies(scope, &indexes, &formation.direct_dependencies)?;
    }
    for schema in &preimage.schemas {
        for constraint in &schema.constraints {
            require_formation(&indexes, *constraint)?;
        }
        validate_dependencies(scope, &indexes, &schema.direct_dependencies)?;
        for role in &schema.roles {
            validate_dependencies(scope, &indexes, &role.direct_dependencies)?;
        }
    }
    for capability in &preimage.capabilities {
        require_formation(&indexes, capability.formation)?;
        validate_set(&capability.direct_dependencies, "capability dependencies")?;
        validate_dependencies(scope, &indexes, &capability.direct_dependencies)?;
    }
    for operator in &preimage.operators {
        validate_dependencies(scope, &indexes, &operator.direct_dependencies)?;
        for mode in &operator.modes {
            validate_mode(scope, &indexes, operator.id, mode)?;
        }
    }

    Ok(indexes)
}

fn validate_mode(
    scope: ConstitutionScopeV2,
    indexes: &IndexesV2<'_>,
    operator: OperatorLocalId,
    mode: &ModePreimageV2,
) -> Result<(), FormationErrorV2> {
    let schema = indexes
        .schemas
        .get(&mode.schema)
        .copied()
        .ok_or(FormationErrorV2::UnknownSchema(mode.schema))?;
    validate_set(&mode.known_roles, "mode known roles")?;
    validate_set(&mode.produced_roles, "mode produced roles")?;
    validate_set(
        &mode.static_basis.context_requirements,
        "static context requirements",
    )?;
    validate_set(
        &mode.static_basis.constitutive_dependencies,
        "static constitutive dependencies",
    )?;
    validate_set_by_key(
        &mode.authorization_requirements,
        "authorization requirements",
        |requirement| requirement.kind,
    )?;
    validate_set_by_key(
        &mode.dynamic_prerequisites,
        "dynamic prerequisites",
        |requirement| requirement.slot,
    )?;
    validate_set(&mode.direct_dependencies, "mode dependencies")?;

    let known: BTreeSet<_> = mode.known_roles.iter().copied().collect();
    let produced: BTreeSet<_> = mode.produced_roles.iter().copied().collect();
    if !known.is_disjoint(&produced) {
        return Err(FormationErrorV2::ModeRoleOverlap(LocalModeRefV2 {
            operator,
            mode: mode.id,
        }));
    }
    let declared: BTreeSet<_> = schema.roles.iter().map(|role| role.id).collect();
    if known.union(&produced).copied().collect::<BTreeSet<_>>() != declared {
        return Err(FormationErrorV2::ModeRoleClosureMismatch(LocalModeRefV2 {
            operator,
            mode: mode.id,
        }));
    }

    for requirement in &mode.static_basis.context_requirements {
        require_formation(indexes, *requirement)?;
    }
    validate_dependencies(scope, indexes, &mode.static_basis.constitutive_dependencies)?;
    for requirement in &mode.authorization_requirements {
        require_formation(indexes, requirement.kind)?;
        validate_cardinality(requirement.cardinality)?;
    }
    for requirement in &mode.dynamic_prerequisites {
        require_formation(indexes, requirement.expected)?;
        if let Some(role) = requirement.role
            && !declared.contains(&role)
        {
            return Err(FormationErrorV2::UnknownRole(LocalRoleRefV2 {
                schema: mode.schema,
                role,
            }));
        }
        validate_cardinality(requirement.cardinality)?;
        validate_set_by_key(
            &requirement.cause_projection,
            "prerequisite cause projection",
            |entry| entry.component,
        )?;
    }
    validate_mode_contract(scope, indexes, mode)?;
    validate_dependencies(scope, indexes, &mode.direct_dependencies)
}

fn validate_mode_contract(
    scope: ConstitutionScopeV2,
    indexes: &IndexesV2<'_>,
    mode: &ModePreimageV2,
) -> Result<(), FormationErrorV2> {
    let contract = &mode.contract;
    validate_cardinality(contract.result_cardinality)?;
    if let ResultOrderContractV2::SelectedBy(strategy) = contract.result_order {
        require_formation(indexes, strategy)?;
    }
    if let Some(domain) = &contract.failure_domain {
        validate_target(scope, domain)?;
    }
    if let Some(domain) = &contract.state_delta_domain {
        validate_target(scope, domain)?;
    }
    if let Some(domain) = &contract.budget_exhaustion_domain {
        validate_target(scope, domain)?;
        if contract.productivity.kind != ProductivityKindV2::Bounded {
            return Err(FormationErrorV2::BudgetExhaustionDomainRequiresBoundedMode);
        }
    }
    validate_set(&contract.effect_intents, "effect intent contracts")?;
    validate_set(&contract.formation_checks, "formation check targets")?;
    validate_set(
        &contract.productivity.obligations,
        "productivity obligations",
    )?;
    validate_set(&contract.scheduling_requirements, "scheduling requirements")?;
    validate_set(&contract.resource_requirements, "resource requirements")?;
    validate_set(&contract.capability_requirements, "capability requirements")?;
    for obligation in &contract.productivity.obligations {
        require_formation(indexes, *obligation)?;
    }
    for requirement in &contract.scheduling_requirements {
        require_formation(indexes, *requirement)?;
    }
    for requirement in &contract.resource_requirements {
        require_formation(indexes, *requirement)?;
    }
    for capability in &contract.capability_requirements {
        require_capability(indexes, *capability)?;
    }
    for effect in &contract.effect_intents {
        validate_target(scope, &effect.intent_domain)?;
        require_capability(indexes, effect.required_capability)?;
        if contract
            .capability_requirements
            .binary_search(&effect.required_capability)
            .is_err()
        {
            return Err(FormationErrorV2::EffectCapabilityNotRequired(
                effect.required_capability,
            ));
        }
    }
    for target in &contract.formation_checks {
        validate_target(scope, target)?;
    }
    if contract.productivity.kind != ProductivityKindV2::Partial
        && contract.productivity.obligations.is_empty()
    {
        return Err(FormationErrorV2::MissingProductivityObligation(
            contract.productivity.kind,
        ));
    }
    if contract.productivity.kind == ProductivityKindV2::Bounded
        && contract.resource_requirements.is_empty()
    {
        return Err(FormationErrorV2::BoundedModeWithoutResourceRequirement);
    }
    if contract.productivity.kind == ProductivityKindV2::Bounded
        && contract.budget_exhaustion_domain.is_none()
    {
        return Err(FormationErrorV2::BoundedModeWithoutBudgetExhaustionDomain);
    }
    if matches!(
        contract.productivity.kind,
        ProductivityKindV2::Productive | ProductivityKindV2::Reactive
    ) && !contract.continuation.may_suspend()
    {
        return Err(FormationErrorV2::OngoingModeWithoutSuspensibleContinuation(
            contract.productivity.kind,
        ));
    }
    if contract.continuation.may_cancel() && contract.failure_domain.is_none() {
        return Err(FormationErrorV2::CancellationWithoutFailureDomain);
    }
    if contract.determinism == DeterminismContractV2::Deterministic
        && contract.continuation.use_policy() == Some(ContinuationUseV2::Reusable)
    {
        return Err(FormationErrorV2::DeterministicModeCannotReuseContinuation);
    }
    Ok(())
}

fn validate_application(
    preimage: &ProgramConstitutionPreimageV2,
    indexes: &IndexesV2<'_>,
    application: &ApplicationDeclarationPreimageV2,
) -> Result<(), FormationErrorV2> {
    let form = &application.form;
    let formation = require_formation(indexes, form.formation)?;
    let schema = indexes
        .schemas
        .get(&form.schema)
        .copied()
        .ok_or(FormationErrorV2::UnknownSchema(form.schema))?;
    let operator = indexes
        .operators
        .get(&form.operator)
        .copied()
        .ok_or(FormationErrorV2::UnknownOperator(form.operator))?;

    validate_set(&form.eligible_modes, "eligible modes")?;
    validate_set(
        &form.context_requirements,
        "application context requirements",
    )?;
    validate_set(&form.constraint_discharges, "constraint discharges")?;
    validate_set(&form.direct_dependencies, "application direct dependencies")?;
    validate_set(&form.dependency_closure, "application dependency closure")?;
    ensure_count(form.bindings.len(), "application bindings")?;
    if !is_strictly_sorted_unique_by(&form.bindings, |binding| (binding.role, binding.occurrence)) {
        return Err(FormationErrorV2::NonCanonicalSet("application bindings"));
    }
    let scope = ConstitutionScopeV2::from(preimage);
    validate_target(scope, &form.result_domain)?;
    if form.result_domain != schema.result_domain {
        return Err(FormationErrorV2::ResultDomainMismatch(application.id));
    }
    for requirement in &form.context_requirements {
        require_formation(indexes, *requirement)?;
    }
    validate_dependencies(scope, indexes, &form.direct_dependencies)?;
    validate_dependencies(scope, indexes, &form.dependency_closure)?;

    let discharge_conditions: Vec<_> = form
        .constraint_discharges
        .iter()
        .map(|discharge| discharge.constraint)
        .collect();
    if discharge_conditions != schema.constraints {
        return Err(FormationErrorV2::ConstraintDischargeMismatch(
            application.id,
        ));
    }
    for discharge in &form.constraint_discharges {
        let constraint = require_formation(indexes, discharge.constraint)?;
        let evidence = require_formation(indexes, discharge.evidence)?;
        if evidence.target != constraint.target
            || evidence
                .direct_dependencies
                .binary_search(&LocalSemanticDependencyV2::Formation(discharge.constraint))
                .is_err()
        {
            return Err(FormationErrorV2::InvalidConstraintDischargeEvidence {
                application: application.id,
                constraint: discharge.constraint,
                evidence: discharge.evidence,
            });
        }
    }

    let mut role_counts = BTreeMap::<RoleLocalId, u32>::new();
    for binding in &form.bindings {
        let role = indexes
            .roles
            .get(&LocalRoleRefV2 {
                schema: form.schema,
                role: binding.role,
            })
            .copied()
            .ok_or(FormationErrorV2::UnknownRole(LocalRoleRefV2 {
                schema: form.schema,
                role: binding.role,
            }))?;
        let expected_occurrence = *role_counts.get(&binding.role).unwrap_or(&0);
        if binding.occurrence != expected_occurrence {
            return Err(FormationErrorV2::NonContiguousRoleOccurrence {
                role: binding.role,
                expected: expected_occurrence,
                found: binding.occurrence,
            });
        }
        role_counts.insert(binding.role, expected_occurrence + 1);
        match binding.value {
            RoleBindingValuePreimageV2::Known(id) | RoleBindingValuePreimageV2::Binder(id) => {
                let binding_formation = require_formation(indexes, id)?;
                if binding_formation.target != role.target {
                    return Err(FormationErrorV2::RoleFormationMismatch {
                        role: binding.role,
                        formation: id,
                    });
                }
            }
            RoleBindingValuePreimageV2::Produced => {}
        }
    }
    for role in &schema.roles {
        let count = role_counts.get(&role.id).copied().unwrap_or(0);
        if !role.cardinality.contains(count) {
            return Err(FormationErrorV2::RoleCardinalityMismatch {
                role: role.id,
                count,
                declared: role.cardinality,
            });
        }
    }

    for dependency in application_dependencies(form)? {
        if dependency == LocalSemanticDependencyV2::Formation(form.formation) {
            continue;
        }
        if formation
            .direct_dependencies
            .binary_search(&dependency)
            .is_err()
        {
            return Err(FormationErrorV2::ApplicationFormationEvidenceMismatch(
                application.id,
            ));
        }
    }

    let eligible: Vec<_> = operator
        .modes
        .iter()
        .filter(|mode| {
            mode.schema == form.schema
                && is_subset(
                    &mode.static_basis.context_requirements,
                    &form.context_requirements,
                )
                && bindings_match_mode(&form.bindings, mode)
        })
        .map(|mode| mode.id)
        .collect();
    if eligible != form.eligible_modes {
        return Err(FormationErrorV2::EligibleModeSetMismatch(application.id));
    }

    let computed_closure = compute_application_dependency_closure(indexes, application)?;
    if computed_closure != form.dependency_closure {
        return Err(FormationErrorV2::DependencyClosureMismatch(application.id));
    }

    validate_term(scope, &formation.term)
}

fn bindings_match_mode(bindings: &[RoleBindingPreimageV2], mode: &ModePreimageV2) -> bool {
    bindings.iter().all(|binding| {
        if mode.known_roles.binary_search(&binding.role).is_ok() {
            matches!(binding.value, RoleBindingValuePreimageV2::Known(_))
        } else if mode.produced_roles.binary_search(&binding.role).is_ok() {
            matches!(
                binding.value,
                RoleBindingValuePreimageV2::Binder(_) | RoleBindingValuePreimageV2::Produced
            )
        } else {
            false
        }
    })
}

fn compute_application_dependency_closure(
    indexes: &IndexesV2<'_>,
    application: &ApplicationDeclarationPreimageV2,
) -> Result<Vec<LocalSemanticDependencyV2>, FormationErrorV2> {
    let roots = application_dependencies(&application.form)?;
    let mut queue = VecDeque::new();
    queue
        .try_reserve(roots.len())
        .map_err(|_| FormationErrorV2::AllocationFailed("dependency queue"))?;
    let mut discovered = BTreeSet::new();
    for root in roots {
        if discovered.insert(root.clone()) {
            queue.push_back(root);
        }
    }
    let mut graph = BTreeMap::<LocalSemanticDependencyV2, Vec<LocalSemanticDependencyV2>>::new();
    let mut edges = 0usize;
    while let Some(dependency) = queue.pop_front() {
        if discovered.len() > MAX_V2_ITEMS {
            return Err(FormationErrorV2::TooManyItems {
                field: "application dependency closure",
                count: discovered.len(),
                maximum: MAX_V2_ITEMS,
            });
        }
        let outgoing = outgoing_dependencies(indexes, &dependency)?;
        if outgoing.binary_search(&dependency).is_ok() {
            return Err(FormationErrorV2::SelfDependency(dependency));
        }
        charge_edges(&mut edges, outgoing.len())?;
        for child in &outgoing {
            if discovered.insert(child.clone()) {
                if discovered.len() > MAX_V2_ITEMS {
                    return Err(FormationErrorV2::TooManyItems {
                        field: "application dependency closure",
                        count: discovered.len(),
                        maximum: MAX_V2_ITEMS,
                    });
                }
                queue
                    .try_reserve(1)
                    .map_err(|_| FormationErrorV2::AllocationFailed("dependency queue"))?;
                queue.push_back(child.clone());
            }
        }
        graph.insert(dependency, outgoing);
    }
    reject_unanchored_cycles(&discovered, &graph)?;
    Ok(discovered.into_iter().collect())
}

fn outgoing_dependencies(
    indexes: &IndexesV2<'_>,
    dependency: &LocalSemanticDependencyV2,
) -> Result<Vec<LocalSemanticDependencyV2>, FormationErrorV2> {
    let mut dependencies = DependencyListV2::default();
    match dependency {
        LocalSemanticDependencyV2::Formation(id) => {
            dependencies.extend(&require_formation(indexes, *id)?.direct_dependencies)?;
        }
        LocalSemanticDependencyV2::RelationSchema(id) => {
            let schema = indexes
                .schemas
                .get(id)
                .copied()
                .ok_or(FormationErrorV2::UnknownSchema(*id))?;
            dependencies.extend(&schema.direct_dependencies)?;
            for role in &schema.roles {
                dependencies.push(LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
                    schema: *id,
                    role: role.id,
                }))?;
            }
            for constraint in &schema.constraints {
                dependencies.push(LocalSemanticDependencyV2::Formation(*constraint))?;
            }
        }
        LocalSemanticDependencyV2::Role(id) => {
            let role = indexes
                .roles
                .get(id)
                .copied()
                .ok_or(FormationErrorV2::UnknownRole(*id))?;
            dependencies.extend(&role.direct_dependencies)?;
        }
        LocalSemanticDependencyV2::Operator(id) => {
            let operator = indexes
                .operators
                .get(id)
                .copied()
                .ok_or(FormationErrorV2::UnknownOperator(*id))?;
            dependencies.extend(&operator.direct_dependencies)?;
            for mode in &operator.modes {
                dependencies.push(LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
                    operator: *id,
                    mode: mode.id,
                }))?;
            }
        }
        LocalSemanticDependencyV2::Mode(id) => {
            let mode = indexes
                .modes
                .get(id)
                .copied()
                .ok_or(FormationErrorV2::UnknownMode(*id))?;
            dependencies.push(LocalSemanticDependencyV2::RelationSchema(mode.schema))?;
            for role in mode.known_roles.iter().chain(&mode.produced_roles) {
                dependencies.push(LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
                    schema: mode.schema,
                    role: *role,
                }))?;
            }
            for formation in &mode.static_basis.context_requirements {
                dependencies.push(LocalSemanticDependencyV2::Formation(*formation))?;
            }
            dependencies.extend(&mode.static_basis.constitutive_dependencies)?;
            for requirement in &mode.authorization_requirements {
                dependencies.push(LocalSemanticDependencyV2::Formation(requirement.kind))?;
            }
            for requirement in &mode.dynamic_prerequisites {
                dependencies.push(LocalSemanticDependencyV2::Formation(requirement.expected))?;
            }
            if let ResultOrderContractV2::SelectedBy(formation) = mode.contract.result_order {
                dependencies.push(LocalSemanticDependencyV2::Formation(formation))?;
            }
            for effect in &mode.contract.effect_intents {
                dependencies.push(LocalSemanticDependencyV2::Capability(
                    effect.required_capability,
                ))?;
            }
            for formation in mode
                .contract
                .productivity
                .obligations
                .iter()
                .chain(&mode.contract.scheduling_requirements)
                .chain(&mode.contract.resource_requirements)
            {
                dependencies.push(LocalSemanticDependencyV2::Formation(*formation))?;
            }
            for capability in &mode.contract.capability_requirements {
                dependencies.push(LocalSemanticDependencyV2::Capability(*capability))?;
            }
            dependencies.extend(&mode.direct_dependencies)?;
        }
        LocalSemanticDependencyV2::Application(id) => {
            let application = indexes
                .applications
                .get(id)
                .copied()
                .ok_or(FormationErrorV2::UnknownApplication(*id))?;
            dependencies.extend(&application_dependencies(&application.form)?)?;
        }
        LocalSemanticDependencyV2::Capability(id) => {
            let capability = require_capability(indexes, *id)?;
            dependencies.push(LocalSemanticDependencyV2::Formation(capability.formation))?;
            dependencies.extend(&capability.direct_dependencies)?;
        }
        LocalSemanticDependencyV2::ExternalReference(_) => {}
    }
    Ok(dependencies.finish())
}

#[derive(Default)]
struct DependencyListV2 {
    items: Vec<LocalSemanticDependencyV2>,
}

impl DependencyListV2 {
    fn push(&mut self, dependency: LocalSemanticDependencyV2) -> Result<(), FormationErrorV2> {
        self.reserve(1)?;
        self.items.push(dependency);
        Ok(())
    }

    fn extend(
        &mut self,
        dependencies: &[LocalSemanticDependencyV2],
    ) -> Result<(), FormationErrorV2> {
        self.reserve(dependencies.len())?;
        self.items.extend_from_slice(dependencies);
        Ok(())
    }

    fn reserve(&mut self, additional: usize) -> Result<(), FormationErrorV2> {
        let length = self
            .items
            .len()
            .checked_add(additional)
            .ok_or(FormationErrorV2::DependencyEdgeLimitExceeded)?;
        if length > MAX_V2_DEPENDENCY_EDGES {
            return Err(FormationErrorV2::DependencyEdgeLimitExceeded);
        }
        self.items
            .try_reserve(additional)
            .map_err(|_| FormationErrorV2::AllocationFailed("dependency list"))
    }

    fn finish(mut self) -> Vec<LocalSemanticDependencyV2> {
        self.items.sort();
        self.items.dedup();
        self.items
    }
}

fn application_dependencies(
    form: &ApplicationFormPreimageV2,
) -> Result<Vec<LocalSemanticDependencyV2>, FormationErrorV2> {
    let mut dependencies = DependencyListV2::default();
    dependencies.push(LocalSemanticDependencyV2::Formation(form.formation))?;
    dependencies.push(LocalSemanticDependencyV2::RelationSchema(form.schema))?;
    dependencies.push(LocalSemanticDependencyV2::Operator(form.operator))?;
    for mode in &form.eligible_modes {
        dependencies.push(LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
            operator: form.operator,
            mode: *mode,
        }))?;
    }
    for binding in &form.bindings {
        dependencies.push(LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
            schema: form.schema,
            role: binding.role,
        }))?;
        if let RoleBindingValuePreimageV2::Known(formation)
        | RoleBindingValuePreimageV2::Binder(formation) = binding.value
        {
            dependencies.push(LocalSemanticDependencyV2::Formation(formation))?;
        }
    }
    for formation in &form.context_requirements {
        dependencies.push(LocalSemanticDependencyV2::Formation(*formation))?;
    }
    for discharge in &form.constraint_discharges {
        dependencies.push(LocalSemanticDependencyV2::Formation(discharge.constraint))?;
        dependencies.push(LocalSemanticDependencyV2::Formation(discharge.evidence))?;
    }
    dependencies.extend(&form.direct_dependencies)?;
    Ok(dependencies.finish())
}

fn reject_unanchored_cycles(
    nodes: &BTreeSet<LocalSemanticDependencyV2>,
    graph: &BTreeMap<LocalSemanticDependencyV2, Vec<LocalSemanticDependencyV2>>,
) -> Result<(), FormationErrorV2> {
    let mut reverse = BTreeMap::<LocalSemanticDependencyV2, Vec<LocalSemanticDependencyV2>>::new();
    let mut remaining = BTreeMap::<LocalSemanticDependencyV2, usize>::new();
    let mut grounded = BTreeSet::new();
    let mut queue = VecDeque::new();
    for node in nodes {
        let outgoing = graph.get(node).map(Vec::as_slice).unwrap_or(&[]);
        remaining.insert(node.clone(), outgoing.len());
        if outgoing.is_empty() {
            grounded.insert(node.clone());
            queue
                .try_reserve(1)
                .map_err(|_| FormationErrorV2::AllocationFailed("grounding queue"))?;
            queue.push_back(node.clone());
        }
        for dependency in outgoing {
            let dependents = reverse.entry(dependency.clone()).or_default();
            if dependents.len() >= MAX_V2_DEPENDENCY_EDGES {
                return Err(FormationErrorV2::DependencyEdgeLimitExceeded);
            }
            dependents
                .try_reserve(1)
                .map_err(|_| FormationErrorV2::AllocationFailed("reverse dependencies"))?;
            dependents.push(node.clone());
        }
    }
    while let Some(ground) = queue.pop_front() {
        if let Some(dependents) = reverse.get(&ground) {
            for dependent in dependents {
                let count =
                    remaining
                        .get_mut(dependent)
                        .ok_or(FormationErrorV2::InternalInvariant(
                            "dependency grounding counter",
                        ))?;
                *count = count
                    .checked_sub(1)
                    .ok_or(FormationErrorV2::InternalInvariant(
                        "dependency grounding underflow",
                    ))?;
                if *count == 0 && grounded.insert(dependent.clone()) {
                    queue
                        .try_reserve(1)
                        .map_err(|_| FormationErrorV2::AllocationFailed("grounding queue"))?;
                    queue.push_back(dependent.clone());
                }
            }
        }
    }
    if let Some(unanchored) = nodes.iter().find(|node| !grounded.contains(*node)) {
        return Err(FormationErrorV2::UnanchoredDependencyCycle(
            unanchored.clone(),
        ));
    }
    Ok(())
}

fn resolve_shape_preimage(
    preimage: &ProgramConstitutionPreimageV2,
    indexes: &IndexesV2<'_>,
    snapshot: ProgramSnapshotId,
    application: &ApplicationDeclarationPreimageV2,
) -> Result<ApplicationShapePreimageV2, FormationErrorV2> {
    let form = &application.form;
    let formation = require_formation(indexes, form.formation)?;
    Ok(ApplicationShapePreimageV2 {
        semantics: preimage.semantics,
        snapshot,
        term: formation.term.clone(),
        formation: resolve_formation(snapshot, form.formation),
        schema: resolve_schema(snapshot, form.schema),
        operator: resolve_operator(snapshot, form.operator),
        eligible_modes: form
            .eligible_modes
            .iter()
            .copied()
            .map(|mode| {
                resolve_mode(
                    snapshot,
                    LocalModeRefV2 {
                        operator: form.operator,
                        mode,
                    },
                )
            })
            .collect(),
        bindings: form
            .bindings
            .iter()
            .map(|binding| ResolvedRoleBindingV2 {
                role: resolve_role(snapshot, form.schema, binding.role),
                occurrence: binding.occurrence,
                value: match binding.value {
                    RoleBindingValuePreimageV2::Known(id) => {
                        ResolvedRoleBindingValueV2::Known(resolve_formation(snapshot, id))
                    }
                    RoleBindingValuePreimageV2::Binder(id) => {
                        ResolvedRoleBindingValueV2::Binder(resolve_formation(snapshot, id))
                    }
                    RoleBindingValuePreimageV2::Produced => ResolvedRoleBindingValueV2::Produced,
                },
            })
            .collect(),
        context_requirements: form
            .context_requirements
            .iter()
            .copied()
            .map(|local| resolve_formation(snapshot, local))
            .collect(),
        constraint_discharges: form
            .constraint_discharges
            .iter()
            .map(|discharge| ResolvedConstraintDischargeV2 {
                constraint: resolve_formation(snapshot, discharge.constraint),
                evidence: resolve_formation(snapshot, discharge.evidence),
            })
            .collect(),
        result_domain: form.result_domain.clone(),
        dependency_closure: form
            .dependency_closure
            .iter()
            .cloned()
            .map(|dependency| resolve_dependency(snapshot, dependency))
            .collect(),
    })
}

fn validate_dependencies(
    scope: ConstitutionScopeV2,
    indexes: &IndexesV2<'_>,
    dependencies: &[LocalSemanticDependencyV2],
) -> Result<(), FormationErrorV2> {
    ensure_count(dependencies.len(), "dependencies")?;
    for dependency in dependencies {
        match dependency {
            LocalSemanticDependencyV2::Formation(id) => {
                require_formation(indexes, *id)?;
            }
            LocalSemanticDependencyV2::RelationSchema(id) => {
                if !indexes.schemas.contains_key(id) {
                    return Err(FormationErrorV2::UnknownSchema(*id));
                }
            }
            LocalSemanticDependencyV2::Role(id) => {
                if !indexes.roles.contains_key(id) {
                    return Err(FormationErrorV2::UnknownRole(*id));
                }
            }
            LocalSemanticDependencyV2::Operator(id) => {
                if !indexes.operators.contains_key(id) {
                    return Err(FormationErrorV2::UnknownOperator(*id));
                }
            }
            LocalSemanticDependencyV2::Mode(id) => {
                if !indexes.modes.contains_key(id) {
                    return Err(FormationErrorV2::UnknownMode(*id));
                }
            }
            LocalSemanticDependencyV2::Application(id) => {
                if !indexes.applications.contains_key(id) {
                    return Err(FormationErrorV2::UnknownApplication(*id));
                }
            }
            LocalSemanticDependencyV2::Capability(id) => {
                require_capability(indexes, *id)?;
            }
            LocalSemanticDependencyV2::ExternalReference(term) => {
                validate_term(scope, term)?;
            }
        }
    }
    Ok(())
}

fn require_formation<'a>(
    indexes: &IndexesV2<'a>,
    id: FormationLocalId,
) -> Result<&'a FormationJudgmentPreimageV2, FormationErrorV2> {
    indexes
        .formations
        .get(&id)
        .copied()
        .ok_or(FormationErrorV2::UnknownFormation(id))
}

fn require_capability<'a>(
    indexes: &IndexesV2<'a>,
    id: CapabilityLocalId,
) -> Result<&'a CapabilityDeclarationPreimageV2, FormationErrorV2> {
    indexes
        .capabilities
        .get(&id)
        .copied()
        .ok_or(FormationErrorV2::UnknownCapability(id))
}

fn validate_target(
    scope: ConstitutionScopeV2,
    target: &FormationTargetV2,
) -> Result<(), FormationErrorV2> {
    validate_term(scope, &target.type_term)?;
    validate_term(scope, &target.interpretation)
}

fn validate_term_set(
    scope: ConstitutionScopeV2,
    terms: &[Term],
    field: &'static str,
) -> Result<(), FormationErrorV2> {
    validate_set(terms, field)?;
    for term in terms {
        validate_term(scope, term)?;
    }
    Ok(())
}

fn validate_term(scope: ConstitutionScopeV2, term: &Term) -> Result<(), FormationErrorV2> {
    if term.scope().semantics != scope.semantics {
        return Err(FormationErrorV2::TermSemanticsMismatch);
    }
    if term.scope().universe != scope.universe {
        return Err(FormationErrorV2::TermUniverseMismatch);
    }
    Ok(())
}

fn validate_cardinality(cardinality: CardinalityV2) -> Result<(), FormationErrorV2> {
    if cardinality
        .maximum
        .is_some_and(|maximum| maximum < cardinality.minimum)
    {
        return Err(FormationErrorV2::InvalidCardinality(cardinality));
    }
    Ok(())
}

fn ensure_count(count: usize, field: &'static str) -> Result<(), FormationErrorV2> {
    if count > MAX_V2_ITEMS {
        return Err(FormationErrorV2::TooManyItems {
            field,
            count,
            maximum: MAX_V2_ITEMS,
        });
    }
    Ok(())
}

fn validate_aggregate_bounds(
    preimage: &ProgramConstitutionPreimageV2,
) -> Result<(), FormationErrorV2> {
    let mut items = 0usize;
    let mut edges = 0usize;
    charge_items(
        &mut items,
        preimage.formations.len(),
        "constitution aggregate items",
    )?;
    charge_items(
        &mut items,
        preimage.schemas.len(),
        "constitution aggregate items",
    )?;
    charge_items(
        &mut items,
        preimage.capabilities.len(),
        "constitution aggregate items",
    )?;
    charge_items(
        &mut items,
        preimage.operators.len(),
        "constitution aggregate items",
    )?;
    charge_items(
        &mut items,
        preimage.applications.len(),
        "constitution aggregate items",
    )?;

    for formation in &preimage.formations {
        charge_items(
            &mut items,
            formation.context.len(),
            "constitution aggregate items",
        )?;
        charge_items(
            &mut items,
            formation.direct_dependencies.len(),
            "constitution aggregate items",
        )?;
        charge_edges(&mut edges, formation.direct_dependencies.len())?;
    }
    for schema in &preimage.schemas {
        for count in [
            schema.roles.len(),
            schema.constraints.len(),
            schema.direct_dependencies.len(),
        ] {
            charge_items(&mut items, count, "constitution aggregate items")?;
        }
        charge_edges(
            &mut edges,
            schema
                .roles
                .len()
                .checked_add(schema.constraints.len())
                .and_then(|count| count.checked_add(schema.direct_dependencies.len()))
                .ok_or(FormationErrorV2::DependencyEdgeLimitExceeded)?,
        )?;
        for role in &schema.roles {
            charge_items(
                &mut items,
                role.direct_dependencies.len(),
                "constitution aggregate items",
            )?;
            charge_edges(&mut edges, role.direct_dependencies.len())?;
        }
    }
    for capability in &preimage.capabilities {
        charge_items(
            &mut items,
            capability.direct_dependencies.len(),
            "constitution aggregate items",
        )?;
        charge_edges(&mut edges, capability.direct_dependencies.len() + 1)?;
    }
    for operator in &preimage.operators {
        for count in [operator.modes.len(), operator.direct_dependencies.len()] {
            charge_items(&mut items, count, "constitution aggregate items")?;
        }
        charge_edges(
            &mut edges,
            operator
                .modes
                .len()
                .checked_add(operator.direct_dependencies.len())
                .ok_or(FormationErrorV2::DependencyEdgeLimitExceeded)?,
        )?;
        for mode in &operator.modes {
            let counts = [
                mode.known_roles.len(),
                mode.produced_roles.len(),
                mode.static_basis.context_requirements.len(),
                mode.static_basis.constitutive_dependencies.len(),
                mode.authorization_requirements.len(),
                mode.dynamic_prerequisites.len(),
                mode.contract.effect_intents.len(),
                mode.contract.productivity.obligations.len(),
                mode.contract.scheduling_requirements.len(),
                mode.contract.resource_requirements.len(),
                mode.contract.capability_requirements.len(),
                mode.direct_dependencies.len(),
            ];
            for count in counts {
                charge_items(&mut items, count, "constitution aggregate items")?;
                charge_edges(&mut edges, count)?;
            }
            charge_items(
                &mut items,
                mode.contract.formation_checks.len(),
                "constitution aggregate items",
            )?;
            charge_edges(&mut edges, 1)?;
            if matches!(
                mode.contract.result_order,
                ResultOrderContractV2::SelectedBy(_)
            ) {
                charge_edges(&mut edges, 1)?;
            }
        }
    }
    for application in &preimage.applications {
        let form = &application.form;
        let counts = [
            form.eligible_modes.len(),
            form.bindings.len(),
            form.context_requirements.len(),
            form.constraint_discharges.len(),
            form.direct_dependencies.len(),
            form.dependency_closure.len(),
        ];
        for count in counts {
            charge_items(&mut items, count, "constitution aggregate items")?;
        }
        charge_edges(&mut edges, 3)?;
        charge_edges(&mut edges, form.eligible_modes.len())?;
        charge_edges(&mut edges, form.bindings.len().saturating_mul(2))?;
        charge_edges(&mut edges, form.context_requirements.len())?;
        charge_edges(
            &mut edges,
            form.constraint_discharges.len().saturating_mul(2),
        )?;
        charge_edges(&mut edges, form.direct_dependencies.len())?;
    }
    Ok(())
}

fn charge_items(
    aggregate: &mut usize,
    count: usize,
    field: &'static str,
) -> Result<(), FormationErrorV2> {
    *aggregate = aggregate
        .checked_add(count)
        .ok_or(FormationErrorV2::TooManyItems {
            field,
            count: usize::MAX,
            maximum: MAX_V2_ITEMS,
        })?;
    ensure_count(*aggregate, field)
}

fn charge_edges(aggregate: &mut usize, count: usize) -> Result<(), FormationErrorV2> {
    *aggregate = aggregate
        .checked_add(count)
        .ok_or(FormationErrorV2::DependencyEdgeLimitExceeded)?;
    if *aggregate > MAX_V2_DEPENDENCY_EDGES {
        return Err(FormationErrorV2::DependencyEdgeLimitExceeded);
    }
    Ok(())
}

fn validate_set<T: Ord>(values: &[T], field: &'static str) -> Result<(), FormationErrorV2> {
    ensure_count(values.len(), field)?;
    if !is_strictly_sorted_unique(values) {
        return Err(FormationErrorV2::NonCanonicalSet(field));
    }
    Ok(())
}

fn validate_set_by_key<T, K: Ord + Copy>(
    values: &[T],
    field: &'static str,
    key: impl Fn(&T) -> K,
) -> Result<(), FormationErrorV2> {
    ensure_count(values.len(), field)?;
    if !is_strictly_sorted_unique_by(values, key) {
        return Err(FormationErrorV2::NonCanonicalSet(field));
    }
    Ok(())
}

fn is_strictly_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn is_strictly_sorted_unique_by<T, K: Ord + Copy>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

fn is_subset<T: Ord>(required: &[T], available: &[T]) -> bool {
    required
        .iter()
        .all(|requirement| available.binary_search(requirement).is_ok())
}

fn resolve_formation(snapshot: ProgramSnapshotId, local: FormationLocalId) -> FormationRefV2 {
    FormationRefV2 { snapshot, local }
}

fn resolve_schema(snapshot: ProgramSnapshotId, local: RelationSchemaLocalId) -> RelationSchemaId {
    RelationSchemaId { snapshot, local }
}

fn resolve_role(
    snapshot: ProgramSnapshotId,
    schema: RelationSchemaLocalId,
    local: RoleLocalId,
) -> RoleId {
    RoleId {
        schema: resolve_schema(snapshot, schema),
        local,
    }
}

fn resolve_operator(snapshot: ProgramSnapshotId, local: OperatorLocalId) -> OperatorRef {
    OperatorRef { snapshot, local }
}

fn resolve_mode(snapshot: ProgramSnapshotId, local: LocalModeRefV2) -> ModeId {
    ModeId {
        operator: resolve_operator(snapshot, local.operator),
        local: local.mode,
    }
}

fn resolve_application(snapshot: ProgramSnapshotId, local: ApplicationLocalId) -> ApplicationId {
    ApplicationId { snapshot, local }
}

fn resolve_dependency(
    snapshot: ProgramSnapshotId,
    dependency: LocalSemanticDependencyV2,
) -> SemanticDependencyV2 {
    match dependency {
        LocalSemanticDependencyV2::Formation(local) => {
            SemanticDependencyV2::Formation(resolve_formation(snapshot, local))
        }
        LocalSemanticDependencyV2::RelationSchema(local) => {
            SemanticDependencyV2::RelationSchema(resolve_schema(snapshot, local))
        }
        LocalSemanticDependencyV2::Role(local) => {
            SemanticDependencyV2::Role(resolve_role(snapshot, local.schema, local.role))
        }
        LocalSemanticDependencyV2::Operator(local) => {
            SemanticDependencyV2::Operator(resolve_operator(snapshot, local))
        }
        LocalSemanticDependencyV2::Mode(local) => {
            SemanticDependencyV2::Mode(resolve_mode(snapshot, local))
        }
        LocalSemanticDependencyV2::Application(local) => {
            SemanticDependencyV2::Application(resolve_application(snapshot, local))
        }
        LocalSemanticDependencyV2::Capability(local) => {
            SemanticDependencyV2::Capability(CapabilityRef { snapshot, local })
        }
        LocalSemanticDependencyV2::ExternalReference(term) => {
            SemanticDependencyV2::ExternalReference(term)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormationErrorV2 {
    CanonicalEncoding(CanonicalEncodeError),
    NonCanonicalSet(&'static str),
    TooManyItems {
        field: &'static str,
        count: usize,
        maximum: usize,
    },
    DependencyEdgeLimitExceeded,
    InternalInvariant(&'static str),
    AllocationFailed(&'static str),
    SelfDependency(LocalSemanticDependencyV2),
    UnanchoredDependencyCycle(LocalSemanticDependencyV2),
    TermSemanticsMismatch,
    TermUniverseMismatch,
    InvalidCardinality(CardinalityV2),
    UnknownFormation(FormationLocalId),
    UnknownSchema(RelationSchemaLocalId),
    UnknownRole(LocalRoleRefV2),
    UnknownCapability(CapabilityLocalId),
    UnknownOperator(OperatorLocalId),
    UnknownMode(LocalModeRefV2),
    UnknownApplication(ApplicationLocalId),
    ModeRoleOverlap(LocalModeRefV2),
    ModeRoleClosureMismatch(LocalModeRefV2),
    MissingProductivityObligation(ProductivityKindV2),
    BoundedModeWithoutResourceRequirement,
    BoundedModeWithoutBudgetExhaustionDomain,
    BudgetExhaustionDomainRequiresBoundedMode,
    OngoingModeWithoutSuspensibleContinuation(ProductivityKindV2),
    CancellationWithoutFailureDomain,
    DeterministicModeCannotReuseContinuation,
    EffectCapabilityNotRequired(CapabilityLocalId),
    ResultDomainMismatch(ApplicationLocalId),
    ConstraintDischargeMismatch(ApplicationLocalId),
    InvalidConstraintDischargeEvidence {
        application: ApplicationLocalId,
        constraint: FormationLocalId,
        evidence: FormationLocalId,
    },
    ApplicationFormationEvidenceMismatch(ApplicationLocalId),
    NonContiguousRoleOccurrence {
        role: RoleLocalId,
        expected: u32,
        found: u32,
    },
    RoleFormationMismatch {
        role: RoleLocalId,
        formation: FormationLocalId,
    },
    RoleCardinalityMismatch {
        role: RoleLocalId,
        count: u32,
        declared: CardinalityV2,
    },
    EligibleModeSetMismatch(ApplicationLocalId),
    DependencyClosureMismatch(ApplicationLocalId),
    ApplicationShapeCollision(ApplicationShapeId),
}

impl fmt::Display for FormationErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for FormationErrorV2 {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::{EqualityContract, TermScope};

    const ONE: CardinalityV2 = CardinalityV2 {
        minimum: 1,
        maximum: Some(1),
    };

    fn semantics() -> ClauseSemanticsId {
        ClauseSemanticsId::from_bytes([1; IDENTITY_BYTES])
    }

    fn snapshot(byte: u8) -> ProgramSnapshotId {
        ProgramSnapshotId::from_bytes([byte; IDENTITY_BYTES])
    }

    fn term(payload: &[u8]) -> Term {
        Term::atom(
            TermScope {
                universe: UniverseId::from_bytes([2; IDENTITY_BYTES]),
                semantics: semantics(),
            },
            b"clause.test/formation".to_vec(),
            payload.to_vec(),
            EqualityContract::ExactOctetsV1,
        )
        .expect("bounded fixture Term is valid")
    }

    fn target(payload: &[u8]) -> FormationTargetV2 {
        FormationTargetV2 {
            type_term: term(payload),
            interpretation: term(payload),
        }
    }

    fn formation(
        id: u32,
        term_payload: &[u8],
        formation_target: FormationTargetV2,
    ) -> FormationJudgmentPreimageV2 {
        FormationJudgmentPreimageV2 {
            id: FormationLocalId::new(id),
            context: vec![],
            term: term(term_payload),
            target: formation_target,
            direct_dependencies: vec![],
        }
    }

    fn mode() -> ModePreimageV2 {
        ModePreimageV2 {
            id: ModeLocalId::new(1),
            schema: RelationSchemaLocalId::new(1),
            known_roles: vec![RoleLocalId::new(1)],
            produced_roles: vec![],
            static_basis: StaticActivationBasisPreimageV2 {
                context_requirements: vec![],
                constitutive_dependencies: vec![],
            },
            authorization_requirements: vec![],
            dynamic_prerequisites: vec![],
            contract: ModeContractV2 {
                determinism: DeterminismContractV2::Deterministic,
                result_cardinality: ONE,
                result_order: ResultOrderContractV2::UnorderedFiniteSet,
                failure_domain: None,
                state_delta_domain: None,
                budget_exhaustion_domain: None,
                effect_intents: vec![],
                formation_checks: vec![],
                productivity: ProductivityContractV2 {
                    kind: ProductivityKindV2::Partial,
                    obligations: vec![],
                },
                scheduling_requirements: vec![],
                resource_requirements: vec![],
                capability_requirements: vec![],
                continuation: ContinuationContractV2::TerminalOnly { may_cancel: false },
            },
            direct_dependencies: vec![],
        }
    }

    fn application(id: u32, binding: FormationLocalId) -> ApplicationDeclarationPreimageV2 {
        ApplicationDeclarationPreimageV2 {
            id: ApplicationLocalId::new(id),
            form: ApplicationFormPreimageV2 {
                formation: FormationLocalId::new(id),
                schema: RelationSchemaLocalId::new(1),
                operator: OperatorLocalId::new(1),
                eligible_modes: vec![ModeLocalId::new(1)],
                bindings: vec![RoleBindingPreimageV2 {
                    role: RoleLocalId::new(1),
                    occurrence: 0,
                    value: RoleBindingValuePreimageV2::Known(binding),
                }],
                context_requirements: vec![],
                constraint_discharges: vec![],
                result_domain: target(b"result"),
                direct_dependencies: vec![],
                dependency_closure: vec![
                    LocalSemanticDependencyV2::Formation(FormationLocalId::new(id)),
                    LocalSemanticDependencyV2::RelationSchema(RelationSchemaLocalId::new(1)),
                    LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
                        schema: RelationSchemaLocalId::new(1),
                        role: RoleLocalId::new(1),
                    }),
                    LocalSemanticDependencyV2::Operator(OperatorLocalId::new(1)),
                    LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
                        operator: OperatorLocalId::new(1),
                        mode: ModeLocalId::new(1),
                    }),
                ],
            },
        }
    }

    fn constitution() -> ProgramConstitutionPreimageV2 {
        ProgramConstitutionPreimageV2 {
            semantics: semantics(),
            universe: UniverseId::from_bytes([2; IDENTITY_BYTES]),
            formations: vec![FormationJudgmentPreimageV2 {
                direct_dependencies: vec![
                    LocalSemanticDependencyV2::RelationSchema(RelationSchemaLocalId::new(1)),
                    LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
                        schema: RelationSchemaLocalId::new(1),
                        role: RoleLocalId::new(1),
                    }),
                    LocalSemanticDependencyV2::Operator(OperatorLocalId::new(1)),
                    LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
                        operator: OperatorLocalId::new(1),
                        mode: ModeLocalId::new(1),
                    }),
                ],
                ..formation(1, b"application-1", target(b"role"))
            }],
            schemas: vec![RelationSchemaPreimageV2 {
                id: RelationSchemaLocalId::new(1),
                roles: vec![RoleDeclarationPreimageV2 {
                    id: RoleLocalId::new(1),
                    target: target(b"role"),
                    cardinality: ONE,
                    direct_dependencies: vec![],
                }],
                constraints: vec![],
                result_domain: target(b"result"),
                direct_dependencies: vec![],
            }],
            capabilities: vec![],
            operators: vec![OperatorPreimageV2 {
                id: OperatorLocalId::new(1),
                modes: vec![mode()],
                direct_dependencies: vec![],
            }],
            applications: vec![application(1, FormationLocalId::new(1))],
        }
    }

    fn resolve(
        preimage: ProgramConstitutionPreimageV2,
    ) -> Result<ResolvedProgramConstitutionV2, FormationErrorV2> {
        resolve_program_constitution_v2(&ProgramSnapshotPreimageV2 {
            constitution: preimage,
            successor_grants: vec![],
            static_execution_grants: vec![],
            state_admission_grants: vec![],
            judgment_authority_grants: vec![],
        })
    }

    #[test]
    fn pure_mode_accepts_an_empty_authorization_set() {
        let resolved = resolve(constitution()).expect("empty authorization is valid");
        let application = ApplicationId {
            snapshot: resolved.snapshot(),
            local: ApplicationLocalId::new(1),
        };
        let mode = ModeId {
            operator: OperatorRef {
                snapshot: resolved.snapshot(),
                local: OperatorLocalId::new(1),
            },
            local: ModeLocalId::new(1),
        };
        let contract = resolved
            .executable_contract(application, mode)
            .expect("declared application and mode are executable");

        assert!(contract.authorization_requirements.is_empty());
        assert!(!resolved.exact_snapshot_preimage_bytes().is_empty());
        assert!(resolved.application_shape(application.local).is_some());
    }

    #[test]
    fn repeated_role_bindings_require_exact_contiguous_occurrences() {
        let mut valid = constitution();
        valid.schemas[0].roles[0].cardinality = CardinalityV2 {
            minimum: 2,
            maximum: Some(2),
        };
        valid.applications[0]
            .form
            .bindings
            .push(RoleBindingPreimageV2 {
                role: RoleLocalId::new(1),
                occurrence: 1,
                value: RoleBindingValuePreimageV2::Known(FormationLocalId::new(1)),
            });
        resolve(valid.clone()).expect("two occurrence-exact role slots satisfy cardinality");

        valid.applications[0].form.bindings[1].occurrence = 2;
        assert_eq!(
            resolve(valid),
            Err(FormationErrorV2::NonContiguousRoleOccurrence {
                role: RoleLocalId::new(1),
                expected: 1,
                found: 2,
            })
        );
    }

    #[test]
    fn binding_with_the_wrong_formation_target_rejects() {
        let mut preimage = constitution();
        preimage
            .formations
            .push(formation(2, b"wrong binding", target(b"wrong target")));
        preimage.applications[0].form.bindings[0].value =
            RoleBindingValuePreimageV2::Known(FormationLocalId::new(2));

        assert_eq!(
            resolve(preimage),
            Err(FormationErrorV2::RoleFormationMismatch {
                role: RoleLocalId::new(1),
                formation: FormationLocalId::new(2),
            })
        );
    }

    #[test]
    fn application_must_name_the_exact_eligible_mode_set() {
        let mut preimage = constitution();
        preimage.applications[0].form.eligible_modes.clear();

        assert_eq!(
            resolve(preimage),
            Err(FormationErrorV2::EligibleModeSetMismatch(
                ApplicationLocalId::new(1)
            ))
        );
    }

    #[test]
    fn application_must_name_the_exact_dependency_closure() {
        let mut preimage = constitution();
        let extra = LocalSemanticDependencyV2::ExternalReference(term(b"extra"));
        preimage.applications[0].form.direct_dependencies = vec![extra.clone()];
        preimage.formations[0].direct_dependencies.push(extra);

        assert_eq!(
            resolve(preimage),
            Err(FormationErrorV2::DependencyClosureMismatch(
                ApplicationLocalId::new(1)
            ))
        );
    }

    #[test]
    fn static_authorization_and_dynamic_prerequisites_remain_distinct() {
        let mut preimage = constitution();
        preimage
            .formations
            .push(formation(2, b"activation requirement", target(b"role")));
        let mode = &mut preimage.operators[0].modes[0];
        mode.static_basis.context_requirements = vec![FormationLocalId::new(2)];
        mode.authorization_requirements = vec![AuthorizationRequirementPreimageV2 {
            kind: FormationLocalId::new(2),
            cardinality: ONE,
        }];
        mode.dynamic_prerequisites = vec![DynamicPrerequisiteRequirementPreimageV2 {
            slot: PrerequisiteLocalId::new(1),
            role: None,
            requirement: ActivationPrerequisiteKind::Observation,
            expected: FormationLocalId::new(2),
            scope: PrerequisiteScope::SameProgramRevision,
            cardinality: ONE,
            cause_projection: vec![crate::provenance::CauseProjectionEntryV2 {
                component: CauseComponentLocalId::new(1),
                path: crate::provenance::PrerequisiteOccurrencePathV2::BoundOccurrence,
            }],
        }];
        preimage.applications[0].form.context_requirements = vec![FormationLocalId::new(2)];
        preimage.formations[0]
            .direct_dependencies
            .push(LocalSemanticDependencyV2::Formation(FormationLocalId::new(
                2,
            )));
        preimage.formations[0].direct_dependencies.sort_unstable();
        preimage.applications[0].form.dependency_closure.push(
            LocalSemanticDependencyV2::Formation(FormationLocalId::new(2)),
        );
        preimage.applications[0]
            .form
            .dependency_closure
            .sort_unstable();

        let contract = resolve(preimage)
            .expect("orthogonal static, authorization, and causal requirements are valid")
            .executable_contract_local(
                ApplicationLocalId::new(1),
                LocalModeRefV2 {
                    operator: OperatorLocalId::new(1),
                    mode: ModeLocalId::new(1),
                },
            )
            .expect("application selects the mode");
        assert_eq!(contract.static_basis.context_requirements.len(), 1);
        assert_eq!(contract.authorization_requirements.len(), 1);
        assert_eq!(contract.dynamic_prerequisites.len(), 1);
    }

    #[test]
    fn authorization_kinds_cannot_repeat_with_conflicting_cardinality() {
        let mut preimage = constitution();
        preimage.operators[0].modes[0].authorization_requirements = vec![
            AuthorizationRequirementPreimageV2 {
                kind: FormationLocalId::new(1),
                cardinality: CardinalityV2 {
                    minimum: 0,
                    maximum: Some(1),
                },
            },
            AuthorizationRequirementPreimageV2 {
                kind: FormationLocalId::new(1),
                cardinality: ONE,
            },
        ];

        assert_eq!(
            resolve(preimage),
            Err(FormationErrorV2::NonCanonicalSet(
                "authorization requirements"
            ))
        );
    }

    #[test]
    fn distinct_shape_preimages_derive_distinct_identities() {
        let mut preimage = constitution();
        preimage.formations.push(FormationJudgmentPreimageV2 {
            direct_dependencies: vec![
                LocalSemanticDependencyV2::RelationSchema(RelationSchemaLocalId::new(1)),
                LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
                    schema: RelationSchemaLocalId::new(1),
                    role: RoleLocalId::new(1),
                }),
                LocalSemanticDependencyV2::Operator(OperatorLocalId::new(1)),
                LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
                    operator: OperatorLocalId::new(1),
                    mode: ModeLocalId::new(1),
                }),
            ],
            ..formation(2, b"application-2", target(b"role"))
        });
        preimage
            .applications
            .push(application(2, FormationLocalId::new(2)));

        let resolved = resolve(preimage).expect("distinct valid shapes resolve");
        assert_ne!(
            resolved.application_shape(ApplicationLocalId::new(1)),
            resolved.application_shape(ApplicationLocalId::new(2))
        );
    }

    #[test]
    fn external_lookup_rejects_foreign_snapshot_ids() {
        let resolved = resolve(constitution()).expect("fixture constitution resolves");
        let foreign_application = ApplicationId {
            snapshot: snapshot(9),
            local: ApplicationLocalId::new(1),
        };
        let foreign_mode = ModeId {
            operator: OperatorRef {
                snapshot: snapshot(9),
                local: OperatorLocalId::new(1),
            },
            local: ModeLocalId::new(1),
        };

        assert!(resolved.application_by_id(foreign_application).is_none());
        assert!(resolved.mode_by_id(foreign_mode).is_none());
        assert!(
            resolved
                .executable_contract(foreign_application, foreign_mode)
                .is_none()
        );
    }

    #[test]
    fn bounded_productivity_requires_resource_and_typed_exhaustion() {
        let mut preimage = constitution();
        preimage.formations.push(formation(
            2,
            b"bounded resource",
            target(b"bounded resource"),
        ));
        preimage.applications[0].form.dependency_closure.push(
            LocalSemanticDependencyV2::Formation(FormationLocalId::new(2)),
        );
        preimage.applications[0]
            .form
            .dependency_closure
            .sort_unstable();
        preimage.operators[0].modes[0].contract.productivity = ProductivityContractV2 {
            kind: ProductivityKindV2::Bounded,
            obligations: vec![FormationLocalId::new(2)],
        };

        assert_eq!(
            resolve(preimage.clone()),
            Err(FormationErrorV2::BoundedModeWithoutResourceRequirement)
        );
        preimage.operators[0].modes[0]
            .contract
            .resource_requirements = vec![FormationLocalId::new(2)];
        assert_eq!(
            resolve(preimage.clone()),
            Err(FormationErrorV2::BoundedModeWithoutBudgetExhaustionDomain)
        );
        preimage.operators[0].modes[0]
            .contract
            .budget_exhaustion_domain = Some(target(b"budget exhausted"));
        resolve(preimage).expect("resource and typed exhaustion close the bounded contract");
    }

    #[test]
    fn one_grounded_edge_does_not_hide_a_dependency_cycle() {
        let a = LocalSemanticDependencyV2::ExternalReference(term(b"cycle/a"));
        let b = LocalSemanticDependencyV2::ExternalReference(term(b"cycle/b"));
        let leaf = LocalSemanticDependencyV2::ExternalReference(term(b"cycle/leaf"));
        let nodes = [a.clone(), b.clone(), leaf.clone()].into_iter().collect();
        let graph = BTreeMap::from([
            (a.clone(), vec![b.clone(), leaf.clone()]),
            (b.clone(), vec![a.clone()]),
            (leaf, vec![]),
        ]);

        assert!(matches!(
            reject_unanchored_cycles(&nodes, &graph),
            Err(FormationErrorV2::UnanchoredDependencyCycle(node))
                if node == a || node == b
        ));
    }
}
