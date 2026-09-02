//! Bounded canonical-source ingress for the declaration slice exercised by the
//! jump-arena specimen.
//!
//! This is intentionally not a general Clause frontend. It reads the whole
//! source losslessly, lowers only the explicitly named profile, and returns
//! exact typed records for every unsupported top-level production.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::canonical::{
    CanonicalDecodeError, CanonicalEncodeError, ProcessPackageCheckError,
    ProgramSnapshotPreimageV2, check_process_package, decode_process_package,
    derive_program_snapshot_id, encode_process_package,
};
use crate::formation::*;
use crate::hash::domain_hash;
use crate::identity::*;
use crate::process::{CheckedProcessPackage, ProcessPackageV2};
use crate::term::{EqualityContract, Term, TermError, TermScope};

const SOURCE_ARTIFACT_DOMAIN: &str = "clause/source-artifact/v1";
const SOURCE_LOCAL_ALLOCATION_DOMAIN: &str = "clause/source-local-allocation/v1";
const RESERVED_LOCAL_ID: u32 = 0;
const MAX_CANONICAL_TEXT_BYTES: usize = u16::MAX as usize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalSourceArtifactIdV1([u8; IDENTITY_BYTES]);

impl CanonicalSourceArtifactIdV1 {
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; IDENTITY_BYTES] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalSourceOriginV1 {
    pub artifact: CanonicalSourceArtifactIdV1,
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CanonicalSourceProductionV1 {
    Referent,
    Shape,
    ShapeField,
    Relation,
    RelationRole,
    RelationMode,
    Law,
    Derive,
    Assertion,
    Handler,
    HandlerInclude,
    Capability,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalEmissionSlotV1 {
    pub production: CanonicalSourceProductionV1,
    pub local: Vec<u8>,
    pub repetition: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CanonicalFreshBasisV1 {
    ConstitutedProgramChange(ProgramChangeOccurrenceId),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalSemanticProducerV1 {
    pub production: CanonicalSourceProductionV1,
    pub semantic_key: Vec<u8>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CanonicalAllocationSlotV1 {
    Emission(CanonicalEmissionSlotV1),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CanonicalAllocationCollisionDispositionV1 {
    RejectTypedCollision,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CanonicalAllocationCycleDispositionV1 {
    RejectDependencyCycle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalAllocationJudgmentV1 {
    Fresh {
        basis: CanonicalFreshBasisV1,
        producer: CanonicalSemanticProducerV1,
        slot: CanonicalAllocationSlotV1,
        collision: CanonicalAllocationCollisionDispositionV1,
        cycle: CanonicalAllocationCycleDispositionV1,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalAllocatedIdentityV1 {
    Formation(FormationLocalId),
    Capability(CapabilityLocalId),
    RelationSchema(RelationSchemaLocalId),
    Role(LocalRoleRefV2),
    Operator(OperatorLocalId),
    Mode(LocalModeRefV2),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalAllocationV1 {
    pub identity: CanonicalAllocatedIdentityV1,
    /// Counter used only to skip the reserved zero coordinate. It is part of
    /// the deterministic derivation record, never freshness evidence.
    pub derivation_attempt: u32,
    pub judgment: CanonicalAllocationJudgmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalSourceAllocationPlanV1 {
    artifact: CanonicalSourceArtifactIdV1,
    root: ProgramChangeOccurrenceId,
    allocations: Vec<CanonicalAllocationV1>,
}

impl CanonicalSourceAllocationPlanV1 {
    #[must_use]
    pub const fn artifact(&self) -> CanonicalSourceArtifactIdV1 {
        self.artifact
    }

    #[must_use]
    pub const fn root(&self) -> ProgramChangeOccurrenceId {
        self.root
    }

    #[must_use]
    pub fn allocations(&self) -> &[CanonicalAllocationV1] {
        &self.allocations
    }

    fn identity(
        &self,
        producer: &CanonicalSemanticProducerV1,
        slot: &CanonicalEmissionSlotV1,
        domain: AllocationDomain,
    ) -> Option<CanonicalAllocatedIdentityV1> {
        self.allocations.iter().find_map(|allocation| {
            let CanonicalAllocationJudgmentV1::Fresh {
                producer: actual_producer,
                slot: CanonicalAllocationSlotV1::Emission(actual_slot),
                ..
            } = &allocation.judgment;
            (actual_producer == producer
                && actual_slot == slot
                && AllocationDomain::of(allocation.identity) == domain)
                .then_some(allocation.identity)
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalSourceEmissionV1 {
    pub producer: CanonicalSemanticProducerV1,
    pub slot: CanonicalEmissionSlotV1,
    pub origin: CanonicalSourceOriginV1,
    pub allocations: Vec<CanonicalAllocationV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalUnsupportedProductionV1 {
    pub production: CanonicalSourceProductionV1,
    pub origin: CanonicalSourceOriginV1,
    /// Ordered independent emissions visible even though this production is
    /// not lowered by the declaration profile.
    pub emissions: Vec<CanonicalSourceEmissionV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalInputScalarV1 {
    Parameter(u16),
    Number(u64),
}

impl CanonicalInputScalarV1 {
    #[must_use]
    pub const fn as_number(self) -> Option<f64> {
        match self {
            Self::Number(bits) => Some(f64::from_bits(bits)),
            Self::Parameter(_) => None,
        }
    }
}

/// Checked source-owned meaning for the bounded `on input` slice. The source
/// artifact and exact origins remain attached; physical slots and event codes
/// are deliberately absent and are supplied only by a later refinement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalInputHandlerV1 {
    pub artifact: CanonicalSourceArtifactIdV1,
    pub handler_origin: CanonicalSourceOriginV1,
    pub initial_assertion_origin: CanonicalSourceOriginV1,
    pub initial_x: u64,
    pub initial_z: u64,
    pub result_x: CanonicalInputScalarV1,
    pub result_z: CanonicalInputScalarV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalJumpScalarV1 {
    VelocityComponent(u8),
    JumpSpeed,
    Number(u64),
}

/// Checked source-owned meaning for one bounded jump-shaped transition. Source
/// owns its designation, the three prerequisite assertions, the grounded
/// predicate, and every included value; a later physical refinement supplies
/// only entry and slot coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalJumpHandlerV1 {
    pub artifact: CanonicalSourceArtifactIdV1,
    pub handler_origin: CanonicalSourceOriginV1,
    pub velocity_assertion_origin: CanonicalSourceOriginV1,
    pub grounded_assertion_origin: CanonicalSourceOriginV1,
    pub jump_speed_assertion_origin: CanonicalSourceOriginV1,
    pub initial_velocity: [u64; 3],
    pub initial_grounded: bool,
    pub jump_speed: u64,
    pub required_grounded: bool,
    pub result_velocity: [CanonicalJumpScalarV1; 3],
    pub result_grounded: bool,
}

/// One construct-blind scalar value owned by canonical source.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalScalarValueV1 {
    Number(u64),
    Boolean(bool),
    Symbol(Vec<u8>),
    Text(String),
    Referent(CanonicalReferentV1),
    RelationTable(CanonicalRelationTableV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalScalarValueKindV1 {
    Number,
    Boolean,
    Symbol,
    Text,
    Referent,
    RelationTable,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalReferentV1 {
    pub domain: FormationLocalId,
    pub identity: FormationLocalId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalRelationValueKindV1 {
    Number,
    Boolean,
    Symbol,
    Text,
    Referent(FormationLocalId),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalRelationCardinalityV1 {
    One,
    Maybe,
    Many,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalRelationTableV1 {
    pub subject_domain: FormationLocalId,
    pub value_kind: CanonicalRelationValueKindV1,
    pub cardinality: CanonicalRelationCardinalityV1,
    pub rows: BTreeMap<CanonicalReferentV1, BTreeSet<CanonicalScalarValueV1>>,
}

/// One exact checked source-state coordinate. The assertion identifies the
/// nominal state occurrence, the relation roles identify its semantic
/// binding, and `path` identifies a scalar or one checked structured field.
/// None of these identities are inferred from declaration or tuple position.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalStateRefV1 {
    pub assertion: FormationLocalId,
    pub relation: RelationSchemaLocalId,
    pub subject_role: LocalRoleRefV2,
    pub value_role: LocalRoleRefV2,
    pub subject: Vec<u8>,
    pub relation_designation: Vec<u8>,
    pub path: CanonicalStatePathV1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalStatePathV1 {
    Scalar,
    Many,
    Rows,
    Field {
        formation: FormationLocalId,
        designation: Vec<u8>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalStateCellV1 {
    pub state: CanonicalStateRefV1,
    pub initial_value: Option<CanonicalScalarValueV1>,
    pub value_kind: CanonicalScalarValueKindV1,
}

/// Construct-blind checked expression used by physical refinements. State
/// reads name exact `CanonicalStateRefV1` values and external values name
/// declared argument ordinals local to the handler, never physical slots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalExecutableExpressionV1 {
    Constant(CanonicalScalarValueV1),
    State(CanonicalStateRefV1),
    Argument(u16),
    FreshReferent {
        domain: FormationLocalId,
        binder: u16,
    },
    RelationRead(Box<Self>, Box<Self>),
    RelationPresent(Box<Self>, Box<Self>),
    RelationPut(Box<Self>, Box<Self>, Box<Self>),
    RelationInsert(Box<Self>, Box<Self>, Box<Self>),
    RelationRemoveRow(Box<Self>, Box<Self>),
    RelationRemoveValue(Box<Self>, Box<Self>, Box<Self>),
    Concatenate(Box<Self>, Box<Self>),
    Add(Box<Self>, Box<Self>),
    Subtract(Box<Self>, Box<Self>),
    Multiply(Box<Self>, Box<Self>),
    Divide(Box<Self>, Box<Self>),
    Clamp(Box<Self>, Box<Self>, Box<Self>),
    Insert(Box<Self>, Box<Self>),
    Remove(Box<Self>, Box<Self>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalExecutablePredicateV1 {
    Equal(
        CanonicalExecutableExpressionV1,
        CanonicalExecutableExpressionV1,
    ),
    GreaterThan(
        CanonicalExecutableExpressionV1,
        CanonicalExecutableExpressionV1,
    ),
    LessThanOrEqual(
        CanonicalExecutableExpressionV1,
        CanonicalExecutableExpressionV1,
    ),
    Contains(
        CanonicalExecutableExpressionV1,
        CanonicalExecutableExpressionV1,
    ),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalExecutableAssignmentV1 {
    pub target: CanonicalStateRefV1,
    pub value: CanonicalExecutableExpressionV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalExecutableRuleV1 {
    pub predicates: Vec<CanonicalExecutablePredicateV1>,
    pub required_present: Vec<CanonicalStateRefV1>,
    pub required_absent: Vec<CanonicalStateRefV1>,
    pub assignments: Vec<CanonicalExecutableAssignmentV1>,
    pub removals: Vec<CanonicalStateRefV1>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalHandlerTriggerV1 {
    /// A physical adapter supplies this handler's declared arguments.
    External,
    /// A source `on tick` rule consumes the fixed-tick delta-time argument and
    /// precedes dependent automatic reactions in the same commanded chain.
    FixedTickRoot,
    /// A source `derive` materializes one predicate after fixed-tick roots and
    /// before every automatic reaction that may consume it.
    FixedTickDerived,
    /// A source-owned automatic reaction follows fixed-tick roots.
    FixedTick,
}

/// One checked source handler independent of physical entry and slot
/// coordinates. `id` is the handler formation allocated from its semantic
/// producer; `designation` is its checked source designation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalExecutableHandlerV1 {
    pub id: FormationLocalId,
    pub designation: Vec<u8>,
    pub trigger: CanonicalHandlerTriggerV1,
    pub argument_count: u16,
    pub rules: Vec<CanonicalExecutableRuleV1>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalKeyPhaseV1 {
    Down,
    Up,
}

/// One source-owned physical key distinction and its checked handler target.
/// The source names only the browser-standard keyboard code and phase; a
/// physical refinement supplies package Roles and executable entries.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalKeyboardBindingV1 {
    pub code: Vec<u8>,
    pub phase: CanonicalKeyPhaseV1,
    pub handler_designation: Vec<u8>,
}

/// One source-owned scalar physical channel and its one-argument handler.
/// The channel is a stable semantic name; each observation supplies one
/// finite F64 value at execution time.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalScalarInputBindingV1 {
    pub channel: Vec<u8>,
    pub handler_designation: Vec<u8>,
}

/// Construct-blind scalar expression owned by one canonical source handler.
/// Physical state coordinates are deliberately supplied only by refinement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalScalarExpressionV1 {
    Current,
    Parameter(Vec<u8>),
    Number(u64),
    Boolean(bool),
    Symbol(Vec<u8>),
    Text(String),
    Concatenate(Box<Self>, Box<Self>),
    Add(Box<Self>, Box<Self>),
    Subtract(Box<Self>, Box<Self>),
    Multiply(Box<Self>, Box<Self>),
    Divide(Box<Self>, Box<Self>),
    Clamp(Box<Self>, Box<Self>, Box<Self>),
}

/// One construct-blind predicate over the scalar cell and source-bound
/// parameters. Physical state coordinates remain absent until refinement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalScalarPredicateV1 {
    Equal(CanonicalScalarExpressionV1, CanonicalScalarExpressionV1),
    GreaterThan(CanonicalScalarExpressionV1, CanonicalScalarExpressionV1),
    LessThanOrEqual(CanonicalScalarExpressionV1, CanonicalScalarExpressionV1),
}

/// Checked source-owned meaning for the bounded one-cell scalar transition
/// profile. The event designation and relation phrase select no host behavior;
/// a later refinement supplies only one entry and one physical state slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalScalarHandlerV1 {
    pub artifact: CanonicalSourceArtifactIdV1,
    pub handler_origin: CanonicalSourceOriginV1,
    pub initial_assertion_origin: CanonicalSourceOriginV1,
    pub include_origin: CanonicalSourceOriginV1,
    pub initial_value: CanonicalScalarValueV1,
    pub parameters: Vec<Vec<u8>>,
    pub predicates: Vec<CanonicalScalarPredicateV1>,
    pub result: CanonicalScalarExpressionV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalTickValueV1 {
    DeltaTime,
    PositionComponent(u8),
    VelocityComponent(u8),
    IntentComponent(u8),
    Grounded,
    Gravity,
    MoveSpeed,
    FloorHeight,
    MinimumX,
    MaximumX,
    MinimumZ,
    MaximumZ,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalTickExpressionV1 {
    Value(CanonicalTickValueV1),
    Number(u64),
    Add(Box<Self>, Box<Self>),
    Subtract(Box<Self>, Box<Self>),
    Multiply(Box<Self>, Box<Self>),
    Divide(Box<Self>, Box<Self>),
    Clamp(Box<Self>, Box<Self>, Box<Self>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalTickPredicateV1 {
    EqualBoolean(CanonicalTickValueV1, bool),
    GreaterThan(CanonicalTickExpressionV1, CanonicalTickExpressionV1),
    LessThanOrEqual(CanonicalTickExpressionV1, CanonicalTickExpressionV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalTickAssignmentTargetV1 {
    PositionComponent(u8),
    VelocityComponent(u8),
    Grounded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalTickAssignmentValueV1 {
    Number(CanonicalTickExpressionV1),
    Boolean(bool),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalTickAssignmentV1 {
    pub target: CanonicalTickAssignmentTargetV1,
    pub value: CanonicalTickAssignmentValueV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalTickRuleV1 {
    pub handler_origin: CanonicalSourceOriginV1,
    pub include_origins: Vec<CanonicalSourceOriginV1>,
    pub predicates: Vec<CanonicalTickPredicateV1>,
    pub assignments: Vec<CanonicalTickAssignmentV1>,
}

/// Checked source-owned meaning for the three bounded `on tick` branches.
/// The source owns all initial world values, arithmetic, predicates, clamp
/// use, and result grouping. A later physical refinement supplies only the
/// tick entry and configuration coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalTickProgramV1 {
    pub artifact: CanonicalSourceArtifactIdV1,
    pub initial_position: [u64; 3],
    pub initial_velocity: [u64; 3],
    pub initial_intent: [u64; 3],
    pub initial_grounded: bool,
    pub gravity: u64,
    pub move_speed: u64,
    pub floor_height: u64,
    pub minimum_x: u64,
    pub maximum_x: u64,
    pub minimum_z: u64,
    pub maximum_z: u64,
    pub assertion_origins: Vec<CanonicalSourceOriginV1>,
    pub clamp_law_origins: [CanonicalSourceOriginV1; 3],
    pub derive_origins: [CanonicalSourceOriginV1; 3],
    pub rules: Vec<CanonicalTickRuleV1>,
}

#[derive(Clone, Debug)]
pub struct CanonicalSourceCstV1 {
    artifact: CanonicalSourceArtifactIdV1,
    exact_source: Box<[u8]>,
    items: Vec<CstItem>,
}

impl CanonicalSourceCstV1 {
    #[must_use]
    pub const fn artifact(&self) -> CanonicalSourceArtifactIdV1 {
        self.artifact
    }

    #[must_use]
    pub fn exact_source(&self) -> &[u8] {
        &self.exact_source
    }

    #[must_use]
    pub fn source_slice(&self, origin: CanonicalSourceOriginV1) -> Option<&[u8]> {
        if origin.artifact != self.artifact {
            return None;
        }
        let start = usize::try_from(origin.start).ok()?;
        let end = usize::try_from(origin.end).ok()?;
        self.exact_source.get(start..end)
    }
}

#[derive(Debug)]
pub struct CanonicalSourcePackageSliceV1 {
    pub checked_package: CheckedProcessPackage,
    pub emissions: Vec<CanonicalSourceEmissionV1>,
    pub unsupported: Vec<CanonicalUnsupportedProductionV1>,
    pub state_cells: Vec<CanonicalStateCellV1>,
    pub executable_handlers: Vec<CanonicalExecutableHandlerV1>,
    pub keyboard_bindings: Vec<CanonicalKeyboardBindingV1>,
    pub scalar_input_bindings: Vec<CanonicalScalarInputBindingV1>,
    pub input_handler: Option<CanonicalInputHandlerV1>,
    pub jump_handler: Option<CanonicalJumpHandlerV1>,
    pub scalar_handlers: Vec<CanonicalScalarHandlerV1>,
    pub tick_program: Option<CanonicalTickProgramV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalSourceContextV1 {
    pub universe: UniverseId,
    pub semantics: ClauseSemanticsId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalSourceErrorV1 {
    InvalidUtf8,
    TabIndentation {
        offset: u64,
    },
    UnexpectedIndentation {
        origin: CanonicalSourceOriginV1,
    },
    EmptyDesignation {
        origin: CanonicalSourceOriginV1,
    },
    DuplicateDesignation {
        designation: Vec<u8>,
    },
    DuplicateChild {
        producer: Vec<u8>,
        child: Vec<u8>,
    },
    InvalidShapeField {
        origin: CanonicalSourceOriginV1,
    },
    InvalidRelationChild {
        origin: CanonicalSourceOriginV1,
    },
    UnknownModeFormation {
        designation: Vec<u8>,
    },
    UnknownModeCapability {
        designation: Vec<u8>,
    },
    MissingRelationReads {
        designation: Vec<u8>,
    },
    MissingRelationMode {
        designation: Vec<u8>,
    },
    UnknownSubjectRole {
        designation: Vec<u8>,
        role: Vec<u8>,
    },
    InvalidMode {
        origin: CanonicalSourceOriginV1,
    },
    InvalidMembershipGroup {
        origin: CanonicalSourceOriginV1,
    },
    InvalidInputHandler {
        origin: CanonicalSourceOriginV1,
    },
    MissingInputInitialAssertion {
        origin: CanonicalSourceOriginV1,
    },
    AmbiguousInputInitialAssertion {
        origin: CanonicalSourceOriginV1,
    },
    InvalidJumpHandler {
        origin: CanonicalSourceOriginV1,
    },
    MissingJumpInitialAssertion {
        origin: CanonicalSourceOriginV1,
    },
    AmbiguousJumpInitialAssertion {
        origin: CanonicalSourceOriginV1,
    },
    InvalidScalarHandler {
        origin: CanonicalSourceOriginV1,
    },
    InvalidGeneralHandler {
        origin: CanonicalSourceOriginV1,
    },
    InvalidKeyboardBinding {
        origin: CanonicalSourceOriginV1,
    },
    DuplicateKeyboardBinding {
        code: Vec<u8>,
        phase: CanonicalKeyPhaseV1,
    },
    InvalidScalarInputBinding {
        origin: CanonicalSourceOriginV1,
    },
    DuplicateScalarInputBinding {
        channel: Vec<u8>,
    },
    MissingScalarInputHandler {
        designation: Vec<u8>,
    },
    AmbiguousScalarInputHandler {
        designation: Vec<u8>,
    },
    MissingKeyboardHandler {
        designation: Vec<u8>,
    },
    AmbiguousKeyboardHandler {
        designation: Vec<u8>,
    },
    MissingScalarInitialAssertion {
        origin: CanonicalSourceOriginV1,
    },
    AmbiguousScalarInitialAssertion {
        origin: CanonicalSourceOriginV1,
    },
    InvalidTickProfile {
        origin: CanonicalSourceOriginV1,
    },
    MissingTickInitialAssertion {
        origin: CanonicalSourceOriginV1,
    },
    AmbiguousTickInitialAssertion {
        origin: CanonicalSourceOriginV1,
    },
    MissingExecutableBinding {
        origin: CanonicalSourceOriginV1,
    },
    AmbiguousExecutableBinding {
        origin: CanonicalSourceOriginV1,
    },
    UnknownModeRole {
        designation: Vec<u8>,
        role: Vec<u8>,
    },
    NonCanonicalKeyword {
        origin: CanonicalSourceOriginV1,
        keyword: Vec<u8>,
    },
    RepeatedEmissionNeedsPlan {
        slot: CanonicalEmissionSlotV1,
    },
    AllocationArtifactMismatch,
    RecordedPlanMismatch,
    MissingAllocation {
        slot: CanonicalEmissionSlotV1,
        domain: &'static str,
    },
    AllocationCollision {
        identity: CanonicalAllocatedIdentityV1,
        first_producer: CanonicalSemanticProducerV1,
        first_slot: CanonicalAllocationSlotV1,
        second_producer: CanonicalSemanticProducerV1,
        second_slot: CanonicalAllocationSlotV1,
    },
    AllocationDerivationExhausted {
        domain: &'static str,
    },
    Term(TermError),
    Encode(CanonicalEncodeError),
    Decode(CanonicalDecodeError),
    Check(ProcessPackageCheckError),
}

impl fmt::Display for CanonicalSourceErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CanonicalSourceErrorV1 {}

impl From<TermError> for CanonicalSourceErrorV1 {
    fn from(value: TermError) -> Self {
        Self::Term(value)
    }
}

#[derive(Clone, Copy, Debug)]
struct SourceLine<'a> {
    text: &'a str,
    start: usize,
    end: usize,
    indent: usize,
}

#[derive(Clone, Debug)]
struct CstItem {
    origin: CanonicalSourceOriginV1,
    kind: CstKind,
}

#[derive(Clone, Debug)]
enum CstKind {
    Referent {
        designation: Vec<u8>,
    },
    Membership(MembershipCst),
    Capability {
        designation: Vec<u8>,
    },
    Shape {
        designation: Vec<u8>,
        fields: Vec<ShapeField>,
    },
    Relation(RelationCst),
    InputHandler(InputHandlerCst),
    JumpHandler(JumpHandlerCst),
    ScalarHandler(ScalarHandlerCst),
    GeneralHandler(GeneralHandlerCst),
    TickHandler(TickHandlerCst),
    KeyboardBinding(KeyboardBindingCst),
    ScalarInputBinding(ScalarInputBindingCst),
    ClampLaw(ClampLawCst),
    ClampDerive(ClampDeriveCst),
    BooleanLaw(BooleanLawCst),
    BooleanDerive(BooleanDeriveCst),
    VectorAssertion(VectorAssertionCst),
    ShapeAssertion(ShapeAssertionCst),
    BooleanAssertion(BooleanAssertionCst),
    NumberAssertion(NumberAssertionCst),
    SymbolAssertion(SymbolAssertionCst),
    TextAssertion(TextAssertionCst),
    Unsupported(CanonicalUnsupportedProductionV1),
}

#[derive(Clone, Debug)]
struct MembershipCst {
    subject: Vec<u8>,
    domains: Vec<Vec<u8>>,
    emissions: Vec<CanonicalSourceEmissionV1>,
}

#[derive(Clone, Debug)]
struct InputHandlerCst {
    origin: CanonicalSourceOriginV1,
    producer: CanonicalSemanticProducerV1,
    designation: Vec<u8>,
    relation: Vec<u8>,
    result_x: CanonicalInputScalarV1,
    result_z: CanonicalInputScalarV1,
    include_origin: CanonicalSourceOriginV1,
    include_local: Vec<u8>,
}

#[derive(Clone, Debug)]
struct VectorAssertionCst {
    origin: CanonicalSourceOriginV1,
    subject: Vec<u8>,
    relation: Vec<u8>,
    x: u64,
    y: u64,
    z: u64,
}

#[derive(Clone, Debug)]
struct ShapeAssertionCst {
    origin: CanonicalSourceOriginV1,
    subject: Vec<u8>,
    relation: Vec<u8>,
    shape: Vec<u8>,
    fields: Vec<ShapeAssertionFieldCst>,
}

#[derive(Clone, Debug)]
struct ShapeAssertionFieldCst {
    name: Vec<u8>,
    value: CanonicalScalarValueV1,
}

#[derive(Clone, Debug)]
struct BooleanAssertionCst {
    origin: CanonicalSourceOriginV1,
    subject: Vec<u8>,
    relation: Vec<u8>,
    value: bool,
}

#[derive(Clone, Debug)]
struct NumberAssertionCst {
    origin: CanonicalSourceOriginV1,
    subject: Vec<u8>,
    relation: Vec<u8>,
    value: u64,
}

#[derive(Clone, Debug)]
struct SymbolAssertionCst {
    origin: CanonicalSourceOriginV1,
    subject: Vec<u8>,
    relation: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Clone, Debug)]
struct TextAssertionCst {
    origin: CanonicalSourceOriginV1,
    subject: Vec<u8>,
    relation: Vec<u8>,
    value: String,
}

#[derive(Clone, Debug)]
struct HandlerIncludeCst {
    origin: CanonicalSourceOriginV1,
    local: Vec<u8>,
}

#[derive(Clone, Debug)]
struct JumpHandlerCst {
    origin: CanonicalSourceOriginV1,
    producer: CanonicalSemanticProducerV1,
    designation: Vec<u8>,
    velocity_relation: Vec<u8>,
    grounded_relation: Vec<u8>,
    jump_speed_subject: Vec<u8>,
    jump_speed_relation: Vec<u8>,
    required_grounded: bool,
    result_velocity: [CanonicalJumpScalarV1; 3],
    result_grounded: bool,
    includes: [HandlerIncludeCst; 2],
}

#[derive(Clone, Debug)]
struct ScalarHandlerCst {
    origin: CanonicalSourceOriginV1,
    producer: CanonicalSemanticProducerV1,
    designation: Vec<u8>,
    subject: Vec<u8>,
    relation: Vec<u8>,
    field: Option<Vec<u8>>,
    parameters: Vec<Vec<u8>>,
    parameter_sources: Vec<ScalarParameterSourceCst>,
    predicates: Vec<CanonicalScalarPredicateV1>,
    boolean_conditions: Vec<BooleanRelationUseCst>,
    result: CanonicalScalarExpressionV1,
    include: HandlerIncludeCst,
}

#[derive(Clone, Debug)]
struct GeneralHandlerCst {
    origin: CanonicalSourceOriginV1,
    producer: CanonicalSemanticProducerV1,
    designation: Vec<u8>,
    subject: Vec<u8>,
    arguments: Vec<GeneralHandlerArgumentCst>,
    creations: Vec<GeneralReferentCreationCst>,
    parameter_sources: Vec<ScalarParameterSourceCst>,
    membership_sources: Vec<ScalarParameterSourceCst>,
    required_sources: Vec<ScalarParameterSourceCst>,
    scalar_bindings: Vec<ScalarLawBindingCst>,
    predicates: Vec<CanonicalScalarPredicateV1>,
    boolean_conditions: Vec<BooleanRelationUseCst>,
    assignments: Vec<GeneralAssignmentCst>,
    insertions: Vec<GeneralAssignmentCst>,
    removals: Vec<ScalarParameterSourceCst>,
    includes: Vec<HandlerIncludeCst>,
}

#[derive(Clone, Debug)]
struct GeneralReferentCreationCst {
    parameter: Vec<u8>,
    domain: Vec<u8>,
    binder: u16,
}

#[derive(Clone, Debug)]
struct GeneralHandlerArgumentCst {
    designation: Vec<u8>,
    ordinal: u16,
}

#[derive(Clone, Debug)]
struct ScalarLawBindingCst {
    origin: CanonicalSourceOriginV1,
    parameter: Vec<u8>,
    value: CanonicalScalarExpressionV1,
}

#[derive(Clone, Debug)]
struct GeneralAssignmentCst {
    target: ScalarParameterSourceCst,
    value: CanonicalScalarExpressionV1,
}

struct GeneralReplacementCst {
    assignments: Vec<GeneralAssignmentCst>,
    aggregate_binding: Option<Vec<u8>>,
    required_sources: Vec<ScalarParameterSourceCst>,
}

#[derive(Clone, Debug)]
struct KeyboardBindingCst {
    origin: CanonicalSourceOriginV1,
    code: Vec<u8>,
    phase: CanonicalKeyPhaseV1,
    handler_designation: Vec<u8>,
}

#[derive(Clone, Debug)]
struct ScalarInputBindingCst {
    origin: CanonicalSourceOriginV1,
    channel: Vec<u8>,
    handler_designation: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScalarParameterSourceCst {
    parameter: Vec<u8>,
    subject: Vec<u8>,
    relation: Vec<u8>,
    shape: Option<Vec<u8>>,
    field: Option<Vec<u8>>,
}

#[derive(Clone)]
struct ScalarHandlerParts<'a> {
    handler: &'a ScalarHandlerCst,
    initial_origin: CanonicalSourceOriginV1,
    initial_value: CanonicalScalarValueV1,
}

#[derive(Clone, Copy)]
struct JumpHandlerParts<'a> {
    handler: &'a JumpHandlerCst,
    velocity: &'a VectorAssertionCst,
    grounded: &'a BooleanAssertionCst,
    jump_speed: &'a NumberAssertionCst,
}

#[derive(Clone, Debug)]
struct TickHandlerCst {
    origin: CanonicalSourceOriginV1,
    producer: CanonicalSemanticProducerV1,
    designation: Vec<u8>,
    predicates: Vec<CanonicalTickPredicateV1>,
    assignments: Vec<CanonicalTickAssignmentV1>,
    includes: Vec<HandlerIncludeCst>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ClampBranchV1 {
    Lower,
    Interior,
    Upper,
}

#[derive(Clone, Debug)]
struct ClampLawCst {
    origin: CanonicalSourceOriginV1,
    designation: Vec<u8>,
    branch: ClampBranchV1,
}

#[derive(Clone, Debug)]
struct ClampDeriveCst {
    origin: CanonicalSourceOriginV1,
    designation: Vec<u8>,
    branch: ClampBranchV1,
}

#[derive(Clone, Debug)]
struct BooleanRelationUseCst {
    origin: CanonicalSourceOriginV1,
    source: Vec<u8>,
}

#[derive(Clone, Debug)]
struct BooleanLawCst {
    origin: CanonicalSourceOriginV1,
    designation: Vec<u8>,
    parameter_sources: Vec<ScalarParameterSourceCst>,
    predicates: Vec<CanonicalScalarPredicateV1>,
    result: BooleanRelationUseCst,
}

#[derive(Clone, Debug)]
struct BooleanDeriveCst {
    origin: CanonicalSourceOriginV1,
    designation: Vec<u8>,
}

#[derive(Clone, Copy)]
struct TickProgramParts<'a> {
    handlers: [&'a TickHandlerCst; 3],
    laws: [&'a ClampLawCst; 3],
    derives: [&'a ClampDeriveCst; 3],
    position: &'a VectorAssertionCst,
    velocity: &'a VectorAssertionCst,
    intent: &'a VectorAssertionCst,
    grounded: &'a BooleanAssertionCst,
    gravity: &'a NumberAssertionCst,
    move_speed: &'a NumberAssertionCst,
    floor_height: &'a NumberAssertionCst,
    minimum_x: &'a NumberAssertionCst,
    maximum_x: &'a NumberAssertionCst,
    minimum_z: &'a NumberAssertionCst,
    maximum_z: &'a NumberAssertionCst,
}

#[derive(Clone, Debug)]
struct ShapeField {
    name: Vec<u8>,
    domain: Vec<u8>,
    origin: CanonicalSourceOriginV1,
}

#[derive(Clone, Debug)]
struct RelationRoleCst {
    name: Vec<u8>,
    domain: Vec<u8>,
    origin: CanonicalSourceOriginV1,
}

#[derive(Clone, Debug)]
struct RelationModeCst {
    known: Vec<Vec<u8>>,
    produced: Vec<Vec<u8>>,
    cardinality: SourceCardinality,
    reactive_obligation: Option<Vec<u8>>,
    continues_linearly: bool,
    effect: Option<RelationEffectCst>,
    canonical: Vec<u8>,
    origin: CanonicalSourceOriginV1,
}

#[derive(Clone, Debug)]
struct RelationEffectCst {
    action_role: Vec<u8>,
    resource_role: Vec<u8>,
    payload_role: Vec<u8>,
    capability: Vec<u8>,
}

#[derive(Clone, Debug)]
struct RelationCst {
    designation: Vec<u8>,
    surface: Vec<u8>,
    reading: Vec<RelationReadingPartCst>,
    subject: Option<Vec<u8>>,
    roles: Vec<RelationRoleCst>,
    modes: Vec<RelationModeCst>,
}

#[derive(Clone, Debug)]
enum RelationReadingPartCst {
    Literal(Vec<u8>),
    Role(Vec<u8>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceCardinality {
    One,
    Maybe,
    Some,
    Many,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
enum AllocationDomain {
    Formation,
    Capability,
    RelationSchema,
    Role,
    Operator,
    Mode,
}

impl AllocationDomain {
    const fn of(identity: CanonicalAllocatedIdentityV1) -> Self {
        match identity {
            CanonicalAllocatedIdentityV1::Formation(_) => Self::Formation,
            CanonicalAllocatedIdentityV1::Capability(_) => Self::Capability,
            CanonicalAllocatedIdentityV1::RelationSchema(_) => Self::RelationSchema,
            CanonicalAllocatedIdentityV1::Role(_) => Self::Role,
            CanonicalAllocatedIdentityV1::Operator(_) => Self::Operator,
            CanonicalAllocatedIdentityV1::Mode(_) => Self::Mode,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Formation => "FormationLocalId",
            Self::Capability => "CapabilityLocalId",
            Self::RelationSchema => "RelationSchemaLocalId",
            Self::Role => "RoleLocalId",
            Self::Operator => "OperatorLocalId",
            Self::Mode => "ModeLocalId",
        }
    }
}

fn derive_local_coordinate(
    root: ProgramChangeOccurrenceId,
    request: &AllocationRequest,
) -> Result<(u32, u32), CanonicalSourceErrorV1> {
    let producer = encode_semantic_producer(&request.producer);
    let slot = encode_emission_slot(&request.slot);
    let domain = [request.domain as u8];
    for attempt in 0..=u32::MAX {
        let attempt_bytes = attempt.to_be_bytes();
        let digest = domain_hash(
            SOURCE_LOCAL_ALLOCATION_DOMAIN,
            &[root.as_bytes(), &domain, &producer, &slot, &attempt_bytes],
        );
        let coordinate = u32::from_be_bytes(
            digest[..4]
                .try_into()
                .expect("a Clause identity digest contains four coordinate octets"),
        );
        if coordinate != RESERVED_LOCAL_ID {
            return Ok((coordinate, attempt));
        }
    }
    Err(CanonicalSourceErrorV1::AllocationDerivationExhausted {
        domain: request.domain.label(),
    })
}

fn encode_semantic_producer(producer: &CanonicalSemanticProducerV1) -> Vec<u8> {
    let mut bytes = vec![producer.production as u8];
    frame_bytes(&mut bytes, &producer.semantic_key);
    bytes
}

fn encode_emission_slot(slot: &CanonicalEmissionSlotV1) -> Vec<u8> {
    let mut bytes = vec![slot.production as u8];
    frame_bytes(&mut bytes, &slot.local);
    match slot.repetition {
        None => bytes.push(0),
        Some(repetition) => {
            bytes.push(1);
            bytes.extend_from_slice(&repetition.to_be_bytes());
        }
    }
    bytes
}

/// Read exact UTF-8 bytes into the bounded canonical CST profile.
pub fn read_canonical_source_v1(
    exact_source: &[u8],
) -> Result<CanonicalSourceCstV1, CanonicalSourceErrorV1> {
    let source =
        std::str::from_utf8(exact_source).map_err(|_| CanonicalSourceErrorV1::InvalidUtf8)?;
    let artifact =
        CanonicalSourceArtifactIdV1(domain_hash(SOURCE_ARTIFACT_DOMAIN, &[exact_source]));
    let lines = source_lines(source)?;
    let mut items = Vec::new();
    let mut cursor = 0;
    while cursor < lines.len() {
        if lines[cursor].text.trim().is_empty() {
            cursor += 1;
            continue;
        }
        if lines[cursor].indent != 0 {
            return Err(CanonicalSourceErrorV1::UnexpectedIndentation {
                origin: line_origin(artifact, lines[cursor]),
            });
        }
        let start = cursor;
        cursor += 1;
        while cursor < lines.len()
            && (lines[cursor].text.trim().is_empty() || lines[cursor].indent > 0)
        {
            cursor += 1;
        }
        let block = &lines[start..cursor];
        let last = block
            .iter()
            .rfind(|line| !line.text.trim().is_empty())
            .copied()
            .expect("a block begins with a nonblank line");
        let origin = CanonicalSourceOriginV1 {
            artifact,
            start: block[0].start as u64,
            end: last.end as u64,
        };
        items.push(parse_item(artifact, block, origin)?);
    }
    retain_supported_boolean_derive_pairs(&mut items);
    validate_unique_designations(&items)?;
    Ok(CanonicalSourceCstV1 {
        artifact,
        exact_source: exact_source.into(),
        items,
    })
}

/// Project an explicit independent allocation plan. Every product is recorded
/// as `Fresh` against the exact constituted root, semantic producer, and
/// emission slot. Canonical sorting fixes record order only; it never supplies
/// an identity coordinate or continuity evidence.
pub fn plan_independent_canonical_source_allocations_v1(
    cst: &CanonicalSourceCstV1,
    root: ProgramChangeOccurrenceId,
) -> Result<CanonicalSourceAllocationPlanV1, CanonicalSourceErrorV1> {
    build_independent_plan(cst, root)
}

/// Validate and recover an already recorded plan for the same proposal act.
/// This observes the original Fresh judgments and identities; it does not
/// create a second allocation judgment or claim retention.
pub fn rematerialize_canonical_source_allocation_plan_v1(
    cst: &CanonicalSourceCstV1,
    recorded: &CanonicalSourceAllocationPlanV1,
) -> Result<CanonicalSourceAllocationPlanV1, CanonicalSourceErrorV1> {
    if recorded.artifact != cst.artifact {
        return Err(CanonicalSourceErrorV1::AllocationArtifactMismatch);
    }
    let requests = allocation_requests(cst)?;
    if requests.len() != recorded.allocations.len() {
        return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
    }
    let (schema_for_producer, operator_for_producer) =
        derived_container_coordinates(recorded.root, &requests)?;
    for request in &requests {
        let matches = recorded
            .allocations
            .iter()
            .filter(|allocation| {
                let CanonicalAllocationJudgmentV1::Fresh {
                    basis,
                    producer,
                    slot: CanonicalAllocationSlotV1::Emission(slot),
                    collision,
                    cycle,
                } = &allocation.judgment;
                *basis == CanonicalFreshBasisV1::ConstitutedProgramChange(recorded.root)
                    && producer == &request.producer
                    && slot == &request.slot
                    && *collision == CanonicalAllocationCollisionDispositionV1::RejectTypedCollision
                    && *cycle == CanonicalAllocationCycleDispositionV1::RejectDependencyCycle
                    && AllocationDomain::of(allocation.identity) == request.domain
            })
            .collect::<Vec<_>>();
        let [allocation] = matches.as_slice() else {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        };
        let (coordinate, attempt) = derive_local_coordinate(recorded.root, request)?;
        let expected_identity = allocated_identity(
            request,
            coordinate,
            &schema_for_producer,
            &operator_for_producer,
        );
        if allocation.identity != expected_identity || allocation.derivation_attempt != attempt {
            return Err(CanonicalSourceErrorV1::RecordedPlanMismatch);
        }
    }
    validate_unique_allocations(&recorded.allocations)?;
    Ok(recorded.clone())
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AllocationRequest {
    producer: CanonicalSemanticProducerV1,
    slot: CanonicalEmissionSlotV1,
    domain: AllocationDomain,
}

fn build_independent_plan(
    cst: &CanonicalSourceCstV1,
    root: ProgramChangeOccurrenceId,
) -> Result<CanonicalSourceAllocationPlanV1, CanonicalSourceErrorV1> {
    let requested = allocation_requests(cst)?;
    let (schema_for_producer, operator_for_producer) =
        derived_container_coordinates(root, &requested)?;
    let basis = CanonicalFreshBasisV1::ConstitutedProgramChange(root);
    let mut allocations = Vec::with_capacity(requested.len());
    for request in requested {
        let (coordinate, derivation_attempt) = derive_local_coordinate(root, &request)?;
        let identity = allocated_identity(
            &request,
            coordinate,
            &schema_for_producer,
            &operator_for_producer,
        );
        allocations.push(CanonicalAllocationV1 {
            identity,
            derivation_attempt,
            judgment: CanonicalAllocationJudgmentV1::Fresh {
                basis,
                producer: request.producer,
                slot: CanonicalAllocationSlotV1::Emission(request.slot),
                collision: CanonicalAllocationCollisionDispositionV1::RejectTypedCollision,
                cycle: CanonicalAllocationCycleDispositionV1::RejectDependencyCycle,
            },
        });
    }
    validate_unique_allocations(&allocations)?;
    Ok(CanonicalSourceAllocationPlanV1 {
        artifact: cst.artifact,
        root,
        allocations,
    })
}

fn derived_container_coordinates(
    root: ProgramChangeOccurrenceId,
    requests: &[AllocationRequest],
) -> Result<
    (
        BTreeMap<CanonicalSemanticProducerV1, RelationSchemaLocalId>,
        BTreeMap<CanonicalSemanticProducerV1, OperatorLocalId>,
    ),
    CanonicalSourceErrorV1,
> {
    let mut schemas = BTreeMap::new();
    let mut operators = BTreeMap::new();
    for request in requests {
        match request.domain {
            AllocationDomain::RelationSchema => {
                let (coordinate, _) = derive_local_coordinate(root, request)?;
                schemas.insert(
                    request.producer.clone(),
                    RelationSchemaLocalId::new(coordinate),
                );
            }
            AllocationDomain::Operator => {
                let (coordinate, _) = derive_local_coordinate(root, request)?;
                operators.insert(request.producer.clone(), OperatorLocalId::new(coordinate));
            }
            _ => {}
        }
    }
    Ok((schemas, operators))
}

fn allocated_identity(
    request: &AllocationRequest,
    coordinate: u32,
    schema_for_producer: &BTreeMap<CanonicalSemanticProducerV1, RelationSchemaLocalId>,
    operator_for_producer: &BTreeMap<CanonicalSemanticProducerV1, OperatorLocalId>,
) -> CanonicalAllocatedIdentityV1 {
    match request.domain {
        AllocationDomain::Formation => {
            CanonicalAllocatedIdentityV1::Formation(FormationLocalId::new(coordinate))
        }
        AllocationDomain::Capability => {
            CanonicalAllocatedIdentityV1::Capability(CapabilityLocalId::new(coordinate))
        }
        AllocationDomain::RelationSchema => {
            CanonicalAllocatedIdentityV1::RelationSchema(RelationSchemaLocalId::new(coordinate))
        }
        AllocationDomain::Role => CanonicalAllocatedIdentityV1::Role(LocalRoleRefV2 {
            schema: *schema_for_producer
                .get(&request.producer)
                .expect("a relation producer has one derived schema"),
            role: RoleLocalId::new(coordinate),
        }),
        AllocationDomain::Operator => {
            CanonicalAllocatedIdentityV1::Operator(OperatorLocalId::new(coordinate))
        }
        AllocationDomain::Mode => CanonicalAllocatedIdentityV1::Mode(LocalModeRefV2 {
            operator: *operator_for_producer
                .get(&request.producer)
                .expect("a relation producer has one derived operator"),
            mode: ModeLocalId::new(coordinate),
        }),
    }
}

fn allocation_requests(
    cst: &CanonicalSourceCstV1,
) -> Result<Vec<AllocationRequest>, CanonicalSourceErrorV1> {
    let mut requested = Vec::new();
    let mut many_assertions = BTreeSet::new();
    let input = input_handler_parts(cst)?;
    let jump = jump_handler_parts(cst)?;
    let scalar = scalar_handler_parts(cst)?;
    let tick = tick_program_parts(cst)?;
    let relational_handlers = relational_handler_origins(cst);
    for item in &cst.items {
        match &item.kind {
            CstKind::Referent { designation } => {
                requested.push(AllocationRequest {
                    producer: semantic_producer(CanonicalSourceProductionV1::Referent, designation),
                    slot: head_slot(CanonicalSourceProductionV1::Referent),
                    domain: AllocationDomain::Formation,
                });
            }
            CstKind::Membership(membership) => {
                requested.push(AllocationRequest {
                    producer: semantic_producer(
                        CanonicalSourceProductionV1::Referent,
                        &membership.subject,
                    ),
                    slot: head_slot(CanonicalSourceProductionV1::Referent),
                    domain: AllocationDomain::Formation,
                });
            }
            CstKind::Capability { designation } => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Capability, designation);
                let slot = head_slot(CanonicalSourceProductionV1::Capability);
                requested.extend([
                    AllocationRequest {
                        producer: producer.clone(),
                        slot: slot.clone(),
                        domain: AllocationDomain::Formation,
                    },
                    AllocationRequest {
                        producer,
                        slot,
                        domain: AllocationDomain::Capability,
                    },
                ]);
            }
            CstKind::Shape {
                designation,
                fields,
            } => {
                let producer = semantic_producer(CanonicalSourceProductionV1::Shape, designation);
                requested.push(AllocationRequest {
                    producer: producer.clone(),
                    slot: head_slot(CanonicalSourceProductionV1::Shape),
                    domain: AllocationDomain::Formation,
                });
                for field in fields {
                    requested.push(AllocationRequest {
                        producer: producer.clone(),
                        slot: child_slot(CanonicalSourceProductionV1::ShapeField, &field.name),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::Relation(relation) => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Relation, &relation.designation);
                let head = head_slot(CanonicalSourceProductionV1::Relation);
                requested.extend([
                    AllocationRequest {
                        producer: producer.clone(),
                        slot: head.clone(),
                        domain: AllocationDomain::Formation,
                    },
                    AllocationRequest {
                        producer: producer.clone(),
                        slot: head.clone(),
                        domain: AllocationDomain::RelationSchema,
                    },
                    AllocationRequest {
                        producer: producer.clone(),
                        slot: head,
                        domain: AllocationDomain::Operator,
                    },
                ]);
                for role in &relation.roles {
                    requested.push(AllocationRequest {
                        producer: producer.clone(),
                        slot: child_slot(CanonicalSourceProductionV1::RelationRole, &role.name),
                        domain: AllocationDomain::Role,
                    });
                }
                for mode in &relation.modes {
                    requested.push(AllocationRequest {
                        producer: producer.clone(),
                        slot: child_slot(
                            CanonicalSourceProductionV1::RelationMode,
                            &mode.canonical,
                        ),
                        domain: AllocationDomain::Mode,
                    });
                }
            }
            CstKind::InputHandler(handler) => {
                requested.extend([
                    AllocationRequest {
                        producer: handler.producer.clone(),
                        slot: head_slot(CanonicalSourceProductionV1::Handler),
                        domain: AllocationDomain::Formation,
                    },
                    AllocationRequest {
                        producer: handler.producer.clone(),
                        slot: child_slot(
                            CanonicalSourceProductionV1::HandlerInclude,
                            &handler.include_local,
                        ),
                        domain: AllocationDomain::Formation,
                    },
                ]);
            }
            CstKind::JumpHandler(handler) => {
                requested.push(AllocationRequest {
                    producer: handler.producer.clone(),
                    slot: head_slot(CanonicalSourceProductionV1::Handler),
                    domain: AllocationDomain::Formation,
                });
                for include in &handler.includes {
                    requested.push(AllocationRequest {
                        producer: handler.producer.clone(),
                        slot: child_slot(
                            CanonicalSourceProductionV1::HandlerInclude,
                            &include.local,
                        ),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::ScalarHandler(handler) => {
                requested.extend([
                    AllocationRequest {
                        producer: handler.producer.clone(),
                        slot: head_slot(CanonicalSourceProductionV1::Handler),
                        domain: AllocationDomain::Formation,
                    },
                    AllocationRequest {
                        producer: handler.producer.clone(),
                        slot: child_slot(
                            CanonicalSourceProductionV1::HandlerInclude,
                            &handler.include.local,
                        ),
                        domain: AllocationDomain::Formation,
                    },
                ]);
            }
            CstKind::GeneralHandler(handler) => {
                requested.push(AllocationRequest {
                    producer: handler.producer.clone(),
                    slot: head_slot(CanonicalSourceProductionV1::Handler),
                    domain: AllocationDomain::Formation,
                });
                for include in &handler.includes {
                    requested.push(AllocationRequest {
                        producer: handler.producer.clone(),
                        slot: child_slot(
                            CanonicalSourceProductionV1::HandlerInclude,
                            &include.local,
                        ),
                        domain: AllocationDomain::Formation,
                    });
                }
                if relational_handlers.contains(&handler.origin) {
                    continue;
                }
                let concrete_subject = concrete_general_handler_subject(cst, handler)?;
                let mut optional_assertions = BTreeSet::new();
                for insertion in &handler.insertions {
                    let request = AllocationRequest {
                        producer: assertion_producer(&concrete_subject, &insertion.target.relation),
                        slot: head_slot(CanonicalSourceProductionV1::Assertion),
                        domain: AllocationDomain::Formation,
                    };
                    if declared_state_cardinality(cst, &insertion.target.relation)
                        == Some(SourceCardinality::Many)
                    {
                        many_assertions.insert(request);
                    } else {
                        optional_assertions.insert(request);
                    }
                }
                for membership in &handler.membership_sources {
                    many_assertions.insert(AllocationRequest {
                        producer: assertion_producer(&concrete_subject, &membership.relation),
                        slot: head_slot(CanonicalSourceProductionV1::Assertion),
                        domain: AllocationDomain::Formation,
                    });
                }
                requested.extend(optional_assertions);
            }
            CstKind::TickHandler(handler) => {
                requested.push(AllocationRequest {
                    producer: handler.producer.clone(),
                    slot: head_slot(CanonicalSourceProductionV1::Handler),
                    domain: AllocationDomain::Formation,
                });
                for include in &handler.includes {
                    requested.push(AllocationRequest {
                        producer: handler.producer.clone(),
                        slot: child_slot(
                            CanonicalSourceProductionV1::HandlerInclude,
                            &include.local,
                        ),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::ClampLaw(law) => requested.push(AllocationRequest {
                producer: semantic_producer(CanonicalSourceProductionV1::Law, &law.designation),
                slot: head_slot(CanonicalSourceProductionV1::Law),
                domain: AllocationDomain::Formation,
            }),
            CstKind::ClampDerive(derive) => requested.push(AllocationRequest {
                producer: semantic_producer(
                    CanonicalSourceProductionV1::Derive,
                    &derive.designation,
                ),
                slot: head_slot(CanonicalSourceProductionV1::Derive),
                domain: AllocationDomain::Formation,
            }),
            CstKind::BooleanLaw(law) => requested.push(AllocationRequest {
                producer: semantic_producer(CanonicalSourceProductionV1::Law, &law.designation),
                slot: head_slot(CanonicalSourceProductionV1::Law),
                domain: AllocationDomain::Formation,
            }),
            CstKind::BooleanDerive(derive) => requested.push(AllocationRequest {
                producer: semantic_producer(
                    CanonicalSourceProductionV1::Derive,
                    &derive.designation,
                ),
                slot: head_slot(CanonicalSourceProductionV1::Derive),
                domain: AllocationDomain::Formation,
            }),
            CstKind::VectorAssertion(assertion) => {
                if input
                    .as_ref()
                    .is_some_and(|(_, selected)| selected.origin == assertion.origin)
                    || jump.is_some_and(|parts| parts.velocity.origin == assertion.origin)
                    || tick.is_some_and(|parts| {
                        [
                            parts.position.origin,
                            parts.velocity.origin,
                            parts.intent.origin,
                        ]
                        .contains(&assertion.origin)
                    })
                    || scalar.iter().any(|parts| {
                        parts.initial_origin == assertion.origin
                            || parts.handler.parameter_sources.iter().any(|source| {
                                source.subject == assertion.subject
                                    && source.relation == assertion.relation
                                    && source.field.is_some()
                            })
                    })
                    || declared_state_relation(cst, &assertion.relation)
                {
                    requested.push(AllocationRequest {
                        producer: assertion_producer(&assertion.subject, &assertion.relation),
                        slot: head_slot(CanonicalSourceProductionV1::Assertion),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::ShapeAssertion(assertion) => {
                if declared_state_relation(cst, &assertion.relation) {
                    requested.push(AllocationRequest {
                        producer: assertion_producer(&assertion.subject, &assertion.relation),
                        slot: head_slot(CanonicalSourceProductionV1::Assertion),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::BooleanAssertion(assertion) => {
                if jump.is_some_and(|parts| parts.grounded.origin == assertion.origin)
                    || tick.is_some_and(|parts| parts.grounded.origin == assertion.origin)
                    || scalar.iter().any(|parts| {
                        parts.initial_origin == assertion.origin
                            || parts.handler.parameter_sources.iter().any(|source| {
                                source.subject == assertion.subject
                                    && source.relation == assertion.relation
                                    && source.field.is_none()
                            })
                    })
                    || declared_state_relation(cst, &assertion.relation)
                {
                    requested.push(AllocationRequest {
                        producer: assertion_producer(&assertion.subject, &assertion.relation),
                        slot: head_slot(CanonicalSourceProductionV1::Assertion),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::NumberAssertion(assertion) => {
                if jump.is_some_and(|parts| parts.jump_speed.origin == assertion.origin)
                    || scalar.iter().any(|parts| {
                        parts.initial_origin == assertion.origin
                            || parts.handler.parameter_sources.iter().any(|source| {
                                source.subject == assertion.subject
                                    && source.relation == assertion.relation
                                    && source.field.is_none()
                            })
                    })
                    || tick.is_some_and(|parts| {
                        [
                            parts.gravity.origin,
                            parts.move_speed.origin,
                            parts.floor_height.origin,
                            parts.minimum_x.origin,
                            parts.maximum_x.origin,
                            parts.minimum_z.origin,
                            parts.maximum_z.origin,
                        ]
                        .contains(&assertion.origin)
                    })
                    || declared_state_relation(cst, &assertion.relation)
                {
                    requested.push(AllocationRequest {
                        producer: assertion_producer(&assertion.subject, &assertion.relation),
                        slot: head_slot(CanonicalSourceProductionV1::Assertion),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::SymbolAssertion(assertion) => {
                if scalar.iter().any(|parts| {
                    parts.initial_origin == assertion.origin
                        || parts.handler.parameter_sources.iter().any(|source| {
                            source.subject == assertion.subject
                                && source.relation == assertion.relation
                                && source.field.is_none()
                        })
                }) || declared_state_relation(cst, &assertion.relation)
                {
                    requested.push(AllocationRequest {
                        producer: assertion_producer(&assertion.subject, &assertion.relation),
                        slot: head_slot(CanonicalSourceProductionV1::Assertion),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::TextAssertion(assertion) => {
                if scalar.iter().any(|parts| {
                    parts.initial_origin == assertion.origin
                        || parts.handler.parameter_sources.iter().any(|source| {
                            source.subject == assertion.subject
                                && source.relation == assertion.relation
                                && source.field.is_none()
                        })
                }) || declared_state_relation(cst, &assertion.relation)
                {
                    requested.push(AllocationRequest {
                        producer: assertion_producer(&assertion.subject, &assertion.relation),
                        slot: head_slot(CanonicalSourceProductionV1::Assertion),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::KeyboardBinding(_)
            | CstKind::ScalarInputBinding(_)
            | CstKind::Unsupported(_) => {}
        }
    }
    requested.extend(many_assertions);
    requested.sort();
    for pair in requested.windows(2) {
        if pair[0] == pair[1] {
            return Err(CanonicalSourceErrorV1::RepeatedEmissionNeedsPlan {
                slot: pair[0].slot.clone(),
            });
        }
    }
    Ok(requested)
}

fn declared_state_cardinality(
    cst: &CanonicalSourceCstV1,
    surface: &[u8],
) -> Option<SourceCardinality> {
    let relation = cst.items.iter().find_map(|item| match &item.kind {
        CstKind::Relation(relation) if relation.surface == surface => Some(relation),
        _ => None,
    })?;
    let subject = relation.subject.as_ref()?;
    let matching = relation
        .modes
        .iter()
        .filter(|mode| mode.known.iter().any(|role| role == subject))
        .collect::<Vec<_>>();
    let [mode] = matching.as_slice() else {
        return None;
    };
    (mode.produced.len() == 1).then_some(mode.cardinality)
}

fn general_handler_relation_designations(handler: &GeneralHandlerCst) -> BTreeSet<Vec<u8>> {
    handler
        .parameter_sources
        .iter()
        .chain(&handler.membership_sources)
        .chain(&handler.required_sources)
        .chain(&handler.removals)
        .map(|source| source.relation.clone())
        .chain(
            handler
                .assignments
                .iter()
                .chain(&handler.insertions)
                .map(|assignment| assignment.target.relation.clone()),
        )
        .collect()
}

fn relational_handler_origins(cst: &CanonicalSourceCstV1) -> BTreeSet<CanonicalSourceOriginV1> {
    let handlers = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::GeneralHandler(handler) => Some(handler),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut origins = handlers
        .iter()
        .filter(|handler| !handler.creations.is_empty())
        .map(|handler| handler.origin)
        .collect::<BTreeSet<_>>();
    let mut relations = handlers
        .iter()
        .filter(|handler| origins.contains(&handler.origin))
        .flat_map(|handler| general_handler_relation_designations(handler))
        .collect::<BTreeSet<_>>();
    loop {
        let mut changed = false;
        for handler in &handlers {
            let touched = general_handler_relation_designations(handler);
            if !origins.contains(&handler.origin)
                && touched.iter().any(|relation| relations.contains(relation))
            {
                origins.insert(handler.origin);
                changed = true;
            }
            if origins.contains(&handler.origin) {
                let prior = relations.len();
                relations.extend(touched);
                changed |= relations.len() != prior;
            }
        }
        if !changed {
            break;
        }
    }
    origins
}

fn relational_relation_designations(cst: &CanonicalSourceCstV1) -> BTreeSet<Vec<u8>> {
    let origins = relational_handler_origins(cst);
    cst.items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::GeneralHandler(handler) if origins.contains(&handler.origin) => Some(handler),
            _ => None,
        })
        .flat_map(general_handler_relation_designations)
        .collect()
}

fn declared_state_relation(cst: &CanonicalSourceCstV1, surface: &[u8]) -> bool {
    let matching = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::Relation(relation) if relation.surface == surface => Some(relation),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [relation] = matching.as_slice() else {
        return false;
    };
    let Some(subject) = relation.subject.as_ref() else {
        return false;
    };
    let produced = relation
        .modes
        .iter()
        .filter(|mode| mode.known.iter().any(|role| role == subject))
        .flat_map(|mode| mode.produced.iter())
        .collect::<BTreeSet<_>>();
    if produced.len() != 1 {
        return false;
    }
    let value = *produced
        .iter()
        .next()
        .expect("one produced state role was established");
    relation.roles.iter().any(|role| &role.name == value)
}

struct ResolvedStateRelation<'a> {
    relation: &'a RelationCst,
    schema: RelationSchemaLocalId,
    subject_role: LocalRoleRefV2,
    subject_designation: &'a [u8],
    subject_domain: &'a [u8],
    value_role: LocalRoleRefV2,
    value_designation: &'a [u8],
    value_domain: &'a [u8],
}

fn resolved_state_relation<'a>(
    cst: &'a CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    surface: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<ResolvedStateRelation<'a>, CanonicalSourceErrorV1> {
    let matching = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::Relation(relation) if relation.surface == surface => Some(relation),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [relation] = matching.as_slice() else {
        return Err(if matching.is_empty() {
            CanonicalSourceErrorV1::MissingExecutableBinding { origin }
        } else {
            CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin }
        });
    };
    resolved_state_relation_for(plan, relation, origin)
}

fn resolved_state_relation_for<'a>(
    plan: &CanonicalSourceAllocationPlanV1,
    relation: &'a RelationCst,
    origin: CanonicalSourceOriginV1,
) -> Result<ResolvedStateRelation<'a>, CanonicalSourceErrorV1> {
    let subject = relation
        .subject
        .as_ref()
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?;
    let produced = relation
        .modes
        .iter()
        .filter(|mode| mode.known.iter().any(|role| role == subject))
        .flat_map(|mode| mode.produced.iter().cloned())
        .collect::<BTreeSet<_>>();
    let produced = produced.iter().collect::<Vec<_>>();
    let [value] = produced.as_slice() else {
        return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin });
    };
    let value = relation
        .roles
        .iter()
        .find(|role| &role.name == *value)
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?;
    let value_designation = value.name.as_slice();
    let value_domain = value.domain.as_slice();
    let subject_domain = relation
        .roles
        .iter()
        .find(|role| &role.name == subject)
        .map(|role| role.domain.as_slice())
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?;
    let producer = semantic_producer(CanonicalSourceProductionV1::Relation, &relation.designation);
    let head = head_slot(CanonicalSourceProductionV1::Relation);
    let schema = schema_id(plan, &producer, &head)?;
    let role = |name: &[u8]| {
        role_id(
            plan,
            &producer,
            &child_slot(CanonicalSourceProductionV1::RelationRole, name),
        )
    };
    let subject_role = role(subject)?;
    let value_role = role(value_designation)?;
    if subject_role.schema != schema || value_role.schema != schema {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
    }
    Ok(ResolvedStateRelation {
        relation,
        schema,
        subject_role,
        subject_designation: subject,
        subject_domain,
        value_role,
        value_designation,
        value_domain,
    })
}

fn validate_source_shape(
    relation: &ResolvedStateRelation<'_>,
    source: &ScalarParameterSourceCst,
    origin: CanonicalSourceOriginV1,
) -> Result<(), CanonicalSourceErrorV1> {
    match (source.shape.as_deref(), source.field.as_deref()) {
        (None, None) => Ok(()),
        (Some(shape), Some(_)) if shape == relation.value_domain => Ok(()),
        _ => Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin }),
    }
}

fn canonical_state_ref(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    subject: &[u8],
    surface: &[u8],
    field: Option<&[u8]>,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, surface, origin)?;
    let assertion_producer = assertion_producer(subject, surface);
    let assertion = formation_id(
        plan,
        &assertion_producer,
        &head_slot(CanonicalSourceProductionV1::Assertion),
    )?;
    canonical_state_ref_with_identity(cst, plan, assertion, subject, &relation, field, origin)
}

fn canonical_many_state_ref(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    subject: &[u8],
    surface: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, surface, origin)?;
    let assertion = formation_id(
        plan,
        &assertion_producer(subject, surface),
        &head_slot(CanonicalSourceProductionV1::Assertion),
    )?;
    Ok(CanonicalStateRefV1 {
        assertion,
        relation: relation.schema,
        subject_role: relation.subject_role,
        value_role: relation.value_role,
        subject: subject.to_vec(),
        relation_designation: relation.relation.designation.clone(),
        path: CanonicalStatePathV1::Many,
    })
}

fn canonical_state_ref_with_identity(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    assertion: FormationLocalId,
    subject: &[u8],
    relation: &ResolvedStateRelation<'_>,
    field: Option<&[u8]>,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
    let path = if let Some(field) = field {
        let shape = cst
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                CstKind::Shape {
                    designation,
                    fields,
                } if designation == relation.value_domain => Some((designation, fields)),
                _ => None,
            })
            .next()
            .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?;
        let declared = shape
            .1
            .iter()
            .find(|declared| declared.name == field)
            .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?;
        let producer = semantic_producer(CanonicalSourceProductionV1::Shape, shape.0);
        CanonicalStatePathV1::Field {
            formation: formation_id(
                plan,
                &producer,
                &child_slot(CanonicalSourceProductionV1::ShapeField, &declared.name),
            )?,
            designation: declared.name.clone(),
        }
    } else {
        CanonicalStatePathV1::Scalar
    };
    Ok(CanonicalStateRefV1 {
        assertion,
        relation: relation.schema,
        subject_role: relation.subject_role,
        value_role: relation.value_role,
        subject: subject.to_vec(),
        relation_designation: relation.relation.designation.clone(),
        path,
    })
}

fn state_ref_for_origin(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    origin: CanonicalSourceOriginV1,
    field: Option<&[u8]>,
) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
    let item = cst
        .items
        .iter()
        .find(|item| item.origin == origin)
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?;
    match &item.kind {
        CstKind::VectorAssertion(assertion) => canonical_state_ref(
            cst,
            plan,
            &assertion.subject,
            &assertion.relation,
            field,
            origin,
        ),
        CstKind::ShapeAssertion(assertion) => {
            let field = field.ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?;
            let relation = resolved_state_relation(cst, plan, &assertion.relation, origin)?;
            if relation.value_domain != assertion.shape
                || !assertion
                    .fields
                    .iter()
                    .any(|declared| declared.name == field)
            {
                return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
            }
            canonical_state_ref(
                cst,
                plan,
                &assertion.subject,
                &assertion.relation,
                Some(field),
                origin,
            )
        }
        CstKind::BooleanAssertion(assertion) => canonical_state_ref(
            cst,
            plan,
            &assertion.subject,
            &assertion.relation,
            None,
            origin,
        ),
        CstKind::NumberAssertion(assertion) => canonical_state_ref(
            cst,
            plan,
            &assertion.subject,
            &assertion.relation,
            None,
            origin,
        ),
        CstKind::SymbolAssertion(assertion) => canonical_state_ref(
            cst,
            plan,
            &assertion.subject,
            &assertion.relation,
            None,
            origin,
        ),
        CstKind::TextAssertion(assertion) => canonical_state_ref(
            cst,
            plan,
            &assertion.subject,
            &assertion.relation,
            None,
            origin,
        ),
        _ => Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin }),
    }
}

fn validate_shape_assertion(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    assertion: &ShapeAssertionCst,
) -> Result<(), CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, &assertion.relation, assertion.origin)?;
    if relation.value_domain != assertion.shape {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
            origin: assertion.origin,
        });
    }
    let declared = cst
        .items
        .iter()
        .find_map(|item| match &item.kind {
            CstKind::Shape {
                designation,
                fields,
            } if designation == &assertion.shape => Some(fields),
            _ => None,
        })
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding {
            origin: assertion.origin,
        })?;
    if declared.len() != assertion.fields.len() {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
            origin: assertion.origin,
        });
    }
    for (declared, actual) in declared.iter().zip(&assertion.fields) {
        let kind_matches = matches!(
            (declared.domain.as_slice(), &actual.value),
            (b"F64", CanonicalScalarValueV1::Number(_))
                | (b"Bool", CanonicalScalarValueV1::Boolean(_))
        ) || (declared.domain == b"Text"
            && matches!(&actual.value, CanonicalScalarValueV1::Text(_)))
            || (declared.domain != b"F64"
                && declared.domain != b"Bool"
                && declared.domain != b"Text"
                && matches!(&actual.value, CanonicalScalarValueV1::Symbol(_)));
        if declared.name != actual.name || !kind_matches {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
                origin: assertion.origin,
            });
        }
    }
    Ok(())
}

fn general_parameter_state_ref(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &ScalarParameterSourceCst,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, &source.relation, origin)?;
    validate_source_shape(&relation, source, origin)?;
    let variable_subject = source.subject.starts_with(b"?");
    let assertions = cst
        .items
        .iter()
        .filter(|item| match (&item.kind, source.field.as_deref()) {
            (CstKind::VectorAssertion(assertion), Some(_)) => {
                assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject)
            }
            (CstKind::ShapeAssertion(assertion), Some(field)) => {
                assertion.relation == source.relation
                    && source.shape.as_deref() == Some(assertion.shape.as_slice())
                    && assertion
                        .fields
                        .iter()
                        .any(|declared| declared.name == field)
                    && (variable_subject || assertion.subject == source.subject)
            }
            (CstKind::BooleanAssertion(assertion), None) => {
                assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject)
            }
            (CstKind::NumberAssertion(assertion), None) => {
                assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject)
            }
            (CstKind::SymbolAssertion(assertion), None) => {
                assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject)
            }
            (CstKind::TextAssertion(assertion), None) => {
                assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject)
            }
            _ => false,
        })
        .collect::<Vec<_>>();
    let [assertion] = assertions.as_slice() else {
        return Err(if assertions.is_empty() {
            CanonicalSourceErrorV1::MissingExecutableBinding { origin }
        } else {
            CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin }
        });
    };
    state_ref_for_origin(cst, plan, assertion.origin, source.field.as_deref())
}

fn general_target_state_ref(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &ScalarParameterSourceCst,
    entities: &BTreeMap<Vec<u8>, Vec<u8>>,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, &source.relation, origin)?;
    validate_source_shape(&relation, source, origin)?;
    let subject = if source.subject.starts_with(b"?") {
        entities
            .get(&source.subject)
            .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?
            .as_slice()
    } else {
        source.subject.as_slice()
    };
    canonical_state_ref(
        cst,
        plan,
        subject,
        &source.relation,
        source.field.as_deref(),
        origin,
    )
}

fn many_state_ref(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &ScalarParameterSourceCst,
    entities: &BTreeMap<Vec<u8>, Vec<u8>>,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
    if source.field.is_some()
        || state_relation_cardinality(cst, plan, source, origin)? != SourceCardinality::Many
    {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
    }
    let subject = if source.subject.starts_with(b"?") {
        entities
            .get(&source.subject)
            .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?
            .as_slice()
    } else {
        source.subject.as_slice()
    };
    canonical_many_state_ref(cst, plan, subject, &source.relation, origin)
}

fn concrete_general_handler_subject(
    cst: &CanonicalSourceCstV1,
    handler: &GeneralHandlerCst,
) -> Result<Vec<u8>, CanonicalSourceErrorV1> {
    if !handler.subject.starts_with(b"?") {
        return Ok(handler.subject.clone());
    }
    let mut subjects = BTreeSet::new();
    for source in handler
        .parameter_sources
        .iter()
        .chain(&handler.membership_sources)
        .chain(&handler.required_sources)
        .filter(|source| source.subject == handler.subject)
    {
        for item in &cst.items {
            let matching = match (&item.kind, source.field.as_deref()) {
                (CstKind::VectorAssertion(assertion), Some(_)) => {
                    assertion.relation == source.relation
                }
                (CstKind::ShapeAssertion(assertion), Some(field)) => {
                    assertion.relation == source.relation
                        && source.shape.as_deref() == Some(assertion.shape.as_slice())
                        && assertion
                            .fields
                            .iter()
                            .any(|declared| declared.name == field)
                }
                (CstKind::BooleanAssertion(assertion), None) => {
                    assertion.relation == source.relation
                }
                (CstKind::NumberAssertion(assertion), None) => {
                    assertion.relation == source.relation
                }
                (CstKind::SymbolAssertion(assertion), None) => {
                    assertion.relation == source.relation
                }
                (CstKind::TextAssertion(assertion), None) => assertion.relation == source.relation,
                _ => false,
            };
            if matching {
                let subject = match &item.kind {
                    CstKind::VectorAssertion(assertion) => &assertion.subject,
                    CstKind::ShapeAssertion(assertion) => &assertion.subject,
                    CstKind::BooleanAssertion(assertion) => &assertion.subject,
                    CstKind::NumberAssertion(assertion) => &assertion.subject,
                    CstKind::SymbolAssertion(assertion) => &assertion.subject,
                    CstKind::TextAssertion(assertion) => &assertion.subject,
                    _ => unreachable!(),
                };
                subjects.insert(subject.clone());
            }
        }
    }
    if subjects.len() != 1 {
        return Err(if subjects.is_empty() {
            CanonicalSourceErrorV1::MissingExecutableBinding {
                origin: handler.origin,
            }
        } else {
            CanonicalSourceErrorV1::AmbiguousExecutableBinding {
                origin: handler.origin,
            }
        });
    }
    Ok(subjects.into_iter().next().expect("one concrete subject"))
}

fn optional_state_value_kind(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &ScalarParameterSourceCst,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalScalarValueKindV1, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, &source.relation, origin)?;
    validate_source_shape(&relation, source, origin)?;
    let domain = if let Some(field) = &source.field {
        cst.items
            .iter()
            .filter_map(|item| match &item.kind {
                CstKind::Shape {
                    designation,
                    fields,
                } if designation == relation.value_domain => fields
                    .iter()
                    .find(|declared| &declared.name == field)
                    .map(|declared| declared.domain.as_slice()),
                _ => None,
            })
            .next()
            .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?
    } else {
        relation.value_domain
    };
    Ok(match domain {
        b"F64" => CanonicalScalarValueKindV1::Number,
        b"Bool" => CanonicalScalarValueKindV1::Boolean,
        b"Text" => CanonicalScalarValueKindV1::Text,
        _ => CanonicalScalarValueKindV1::Symbol,
    })
}

fn referent_type_id(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    designation: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<FormationLocalId, CanonicalSourceErrorV1> {
    if !cst.items.iter().any(|item| {
        matches!(
            &item.kind,
            CstKind::Referent {
                designation: candidate
            } if candidate == designation
        )
    }) {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
    }
    let producer = semantic_producer(CanonicalSourceProductionV1::Referent, designation);
    formation_id(
        plan,
        &producer,
        &head_slot(CanonicalSourceProductionV1::Referent),
    )
}

fn declared_referent_value(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    designation: &[u8],
    domain: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalReferentV1, CanonicalSourceErrorV1> {
    let matching = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::Membership(membership)
                if membership.subject == designation
                    && membership
                        .domains
                        .iter()
                        .any(|candidate| candidate == domain) =>
            {
                Some(membership)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let [membership] = matching.as_slice() else {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
    };
    let producer = semantic_producer(CanonicalSourceProductionV1::Referent, &membership.subject);
    Ok(CanonicalReferentV1 {
        domain: referent_type_id(cst, plan, domain, origin)?,
        identity: formation_id(
            plan,
            &producer,
            &head_slot(CanonicalSourceProductionV1::Referent),
        )?,
    })
}

fn relation_table_state_ref(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    surface: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, surface, origin)?;
    let producer = semantic_producer(
        CanonicalSourceProductionV1::Relation,
        &relation.relation.designation,
    );
    Ok(CanonicalStateRefV1 {
        assertion: formation_id(
            plan,
            &producer,
            &head_slot(CanonicalSourceProductionV1::Relation),
        )?,
        relation: relation.schema,
        subject_role: relation.subject_role,
        value_role: relation.value_role,
        subject: b"relations".to_vec(),
        relation_designation: relation.relation.designation.clone(),
        path: CanonicalStatePathV1::Rows,
    })
}

fn canonical_relation_table(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    surface: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalRelationTableV1, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, surface, origin)?;
    let subject_domain = referent_type_id(cst, plan, relation.subject_domain, origin)?;
    let value_kind = match relation.value_domain {
        b"F64" => CanonicalRelationValueKindV1::Number,
        b"Bool" => CanonicalRelationValueKindV1::Boolean,
        b"Text" => CanonicalRelationValueKindV1::Text,
        domain => {
            CanonicalRelationValueKindV1::Referent(referent_type_id(cst, plan, domain, origin)?)
        }
    };
    let cardinality = match state_relation_cardinality(
        cst,
        plan,
        &ScalarParameterSourceCst {
            parameter: Vec::new(),
            subject: Vec::new(),
            relation: surface.to_vec(),
            shape: None,
            field: None,
        },
        origin,
    )? {
        SourceCardinality::One => CanonicalRelationCardinalityV1::One,
        SourceCardinality::Maybe => CanonicalRelationCardinalityV1::Maybe,
        SourceCardinality::Many => CanonicalRelationCardinalityV1::Many,
        SourceCardinality::Some => {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
        }
    };
    let mut rows = BTreeMap::<CanonicalReferentV1, BTreeSet<CanonicalScalarValueV1>>::new();
    for item in &cst.items {
        let (subject, value) = match &item.kind {
            CstKind::NumberAssertion(assertion) if assertion.relation == surface => (
                assertion.subject.as_slice(),
                CanonicalScalarValueV1::Number(assertion.value),
            ),
            CstKind::BooleanAssertion(assertion) if assertion.relation == surface => (
                assertion.subject.as_slice(),
                CanonicalScalarValueV1::Boolean(assertion.value),
            ),
            CstKind::TextAssertion(assertion) if assertion.relation == surface => (
                assertion.subject.as_slice(),
                CanonicalScalarValueV1::Text(assertion.value.clone()),
            ),
            CstKind::SymbolAssertion(assertion) if assertion.relation == surface => {
                let CanonicalRelationValueKindV1::Referent(_) = value_kind else {
                    return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
                        origin: item.origin,
                    });
                };
                (
                    assertion.subject.as_slice(),
                    CanonicalScalarValueV1::Referent(declared_referent_value(
                        cst,
                        plan,
                        &assertion.value,
                        relation.value_domain,
                        item.origin,
                    )?),
                )
            }
            _ => continue,
        };
        let subject =
            declared_referent_value(cst, plan, subject, relation.subject_domain, item.origin)?;
        let values = rows.entry(subject).or_default();
        if cardinality != CanonicalRelationCardinalityV1::Many && !values.is_empty() {
            return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding {
                origin: item.origin,
            });
        }
        values.insert(value);
    }
    Ok(CanonicalRelationTableV1 {
        subject_domain,
        value_kind,
        cardinality,
        rows,
    })
}

fn require_optional_state_relation(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &ScalarParameterSourceCst,
    origin: CanonicalSourceOriginV1,
) -> Result<(), CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, &source.relation, origin)?;
    let subject = relation
        .relation
        .subject
        .as_ref()
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?;
    let optional = relation.relation.modes.iter().any(|mode| {
        mode.cardinality == SourceCardinality::Maybe
            && mode.known.iter().any(|role| role == subject)
    });
    if !optional {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
    }
    Ok(())
}

fn state_relation_cardinality(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &ScalarParameterSourceCst,
    origin: CanonicalSourceOriginV1,
) -> Result<SourceCardinality, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, &source.relation, origin)?;
    validate_source_shape(&relation, source, origin)?;
    let matching = relation
        .relation
        .modes
        .iter()
        .filter(|mode| {
            mode.known
                .iter()
                .any(|role| role == relation.subject_designation)
                && mode
                    .produced
                    .iter()
                    .any(|role| role == relation.value_designation)
        })
        .collect::<Vec<_>>();
    let [mode] = matching.as_slice() else {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
    };
    Ok(mode.cardinality)
}

struct ResolvedBooleanRelationUse<'a> {
    relation: &'a RelationCst,
    bindings: BTreeMap<Vec<u8>, Vec<u8>>,
    value: bool,
}

fn resolve_boolean_relation_use<'a>(
    cst: &'a CanonicalSourceCstV1,
    source: &BooleanRelationUseCst,
) -> Result<ResolvedBooleanRelationUse<'a>, CanonicalSourceErrorV1> {
    let text = std::str::from_utf8(&source.source).map_err(|_| {
        CanonicalSourceErrorV1::MissingExecutableBinding {
            origin: source.origin,
        }
    })?;
    let tokens = text
        .split_whitespace()
        .map(str::as_bytes)
        .collect::<Vec<_>>();
    let matches = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::Relation(relation) if relation.reading.len() == tokens.len() => {
                let mut bindings = BTreeMap::new();
                for (part, token) in relation.reading.iter().zip(&tokens) {
                    match part {
                        RelationReadingPartCst::Literal(literal) if literal == token => {}
                        RelationReadingPartCst::Role(role) => {
                            bindings.insert(role.clone(), token.to_vec());
                        }
                        _ => return None,
                    }
                }
                let subject = relation.subject.as_ref()?;
                let produced = relation
                    .modes
                    .iter()
                    .filter(|mode| mode.known.iter().any(|role| role == subject))
                    .flat_map(|mode| mode.produced.iter())
                    .collect::<BTreeSet<_>>();
                if produced.len() != 1 {
                    return None;
                }
                let value_role = produced.into_iter().next()?;
                let value = match bindings.get(value_role)?.as_slice() {
                    b"true" => true,
                    b"false" => false,
                    _ => return None,
                };
                Some(ResolvedBooleanRelationUse {
                    relation,
                    bindings,
                    value,
                })
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let [resolved] = matches.as_slice() else {
        return Err(if matches.is_empty() {
            CanonicalSourceErrorV1::MissingExecutableBinding {
                origin: source.origin,
            }
        } else {
            CanonicalSourceErrorV1::AmbiguousExecutableBinding {
                origin: source.origin,
            }
        });
    };
    Ok(ResolvedBooleanRelationUse {
        relation: resolved.relation,
        bindings: resolved.bindings.clone(),
        value: resolved.value,
    })
}

fn resolve_parameter_states(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    sources: &[ScalarParameterSourceCst],
    origin: CanonicalSourceOriginV1,
) -> Result<
    (
        BTreeMap<Vec<u8>, CanonicalStateRefV1>,
        BTreeMap<Vec<u8>, Vec<u8>>,
    ),
    CanonicalSourceErrorV1,
> {
    let mut parameters = BTreeMap::new();
    let mut entities = BTreeMap::new();
    let mut pending = Vec::new();
    for source in sources {
        let state = match general_parameter_state_ref(cst, plan, source, origin) {
            Ok(state) => state,
            Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. }) => {
                pending.push(source);
                continue;
            }
            Err(error) => return Err(error),
        };
        if source.subject.starts_with(b"?") {
            if let Some(previous) = entities.insert(source.subject.clone(), state.subject.clone())
                && previous != state.subject
            {
                return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin });
            }
        }
        parameters.insert(source.parameter.clone(), state);
    }
    for source in pending {
        let state = general_target_state_ref(cst, plan, source, &entities, origin)?;
        parameters.insert(source.parameter.clone(), state);
    }
    Ok((parameters, entities))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneralStateCandidate {
    state: CanonicalStateRefV1,
    value: Option<CanonicalScalarValueV1>,
}

struct GeneralSourcePlan<'a> {
    source: &'a ScalarParameterSourceCst,
    candidates: Vec<GeneralStateCandidate>,
    subject_domain: Vec<u8>,
    value_domain: Vec<u8>,
    singleton_forward_mode: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneralBindingSolution {
    parameters: BTreeMap<Vec<u8>, CanonicalStateRefV1>,
    entities: BTreeMap<Vec<u8>, Vec<u8>>,
    selector_equalities: Vec<(CanonicalStateRefV1, Vec<u8>)>,
}

fn general_source_value_domain(
    cst: &CanonicalSourceCstV1,
    relation: &ResolvedStateRelation<'_>,
    source: &ScalarParameterSourceCst,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<u8>, CanonicalSourceErrorV1> {
    validate_source_shape(relation, source, origin)?;
    let Some(field) = source.field.as_deref() else {
        return Ok(relation.value_domain.to_vec());
    };
    cst.items
        .iter()
        .find_map(|item| match &item.kind {
            CstKind::Shape {
                designation,
                fields,
            } if designation == relation.value_domain => fields
                .iter()
                .find(|declared| declared.name == field)
                .map(|declared| declared.domain.clone()),
            _ => None,
        })
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })
}

fn singleton_forward_mode(relation: &ResolvedStateRelation<'_>) -> bool {
    let matching = relation
        .relation
        .modes
        .iter()
        .filter(|mode| {
            mode.known
                .iter()
                .any(|role| role == relation.subject_designation)
                && mode
                    .produced
                    .iter()
                    .any(|role| role == relation.value_designation)
        })
        .collect::<Vec<_>>();
    matches!(matching.as_slice(), [mode] if mode.cardinality == SourceCardinality::One)
}

fn general_state_candidates(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &ScalarParameterSourceCst,
    initial_entities: &BTreeMap<Vec<u8>, Vec<u8>>,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<GeneralStateCandidate>, CanonicalSourceErrorV1> {
    let relation = resolved_state_relation(cst, plan, &source.relation, origin)?;
    validate_source_shape(&relation, source, origin)?;
    let variable_subject = source.subject.starts_with(b"?");
    let mut candidates = Vec::new();
    for item in &cst.items {
        let value = match (&item.kind, source.field.as_deref()) {
            (CstKind::VectorAssertion(assertion), Some(field))
                if assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject) =>
            {
                let value = match field {
                    b"x" => assertion.x,
                    b"y" => assertion.y,
                    b"z" => assertion.z,
                    _ => continue,
                };
                CanonicalScalarValueV1::Number(value)
            }
            (CstKind::ShapeAssertion(assertion), Some(field))
                if assertion.relation == source.relation
                    && source.shape.as_deref() == Some(assertion.shape.as_slice())
                    && (variable_subject || assertion.subject == source.subject) =>
            {
                let Some(value) = assertion
                    .fields
                    .iter()
                    .find(|declared| declared.name == field)
                    .map(|declared| declared.value.clone())
                else {
                    continue;
                };
                value
            }
            (CstKind::BooleanAssertion(assertion), None)
                if assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject) =>
            {
                CanonicalScalarValueV1::Boolean(assertion.value)
            }
            (CstKind::NumberAssertion(assertion), None)
                if assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject) =>
            {
                CanonicalScalarValueV1::Number(assertion.value)
            }
            (CstKind::SymbolAssertion(assertion), None)
                if assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject) =>
            {
                CanonicalScalarValueV1::Symbol(assertion.value.clone())
            }
            (CstKind::TextAssertion(assertion), None)
                if assertion.relation == source.relation
                    && (variable_subject || assertion.subject == source.subject) =>
            {
                CanonicalScalarValueV1::Text(assertion.value.clone())
            }
            _ => continue,
        };
        candidates.push(GeneralStateCandidate {
            state: state_ref_for_origin(cst, plan, item.origin, source.field.as_deref())?,
            value: Some(value),
        });
    }
    if candidates.is_empty() {
        let subject = if source.subject.starts_with(b"?") {
            initial_entities.get(&source.subject).map(Vec::as_slice)
        } else {
            Some(source.subject.as_slice())
        };
        if let Some(subject) = subject {
            match canonical_state_ref(
                cst,
                plan,
                subject,
                &source.relation,
                source.field.as_deref(),
                origin,
            ) {
                Ok(state) => candidates.push(GeneralStateCandidate { state, value: None }),
                Err(CanonicalSourceErrorV1::MissingAllocation { .. }) => {}
                Err(error) => return Err(error),
            }
        }
    }
    Ok(candidates)
}

fn search_general_binding_solutions(
    plans: &[GeneralSourcePlan<'_>],
    index: usize,
    parameters: BTreeMap<Vec<u8>, CanonicalStateRefV1>,
    entities: BTreeMap<Vec<u8>, Vec<u8>>,
    solutions: &mut Vec<GeneralBindingSolution>,
) {
    let Some(planned) = plans.get(index) else {
        let mut selector_equalities = plans
            .iter()
            .filter(|planned| {
                plans
                    .iter()
                    .any(|candidate| candidate.source.subject == planned.source.parameter)
            })
            .filter_map(|planned| {
                Some((
                    parameters.get(&planned.source.parameter)?.clone(),
                    entities.get(&planned.source.parameter)?.clone(),
                ))
            })
            .collect::<Vec<_>>();
        selector_equalities.sort();
        selector_equalities.dedup();
        let solution = GeneralBindingSolution {
            parameters,
            entities,
            selector_equalities,
        };
        if !solutions.contains(&solution) {
            solutions.push(solution);
        }
        return;
    };
    for candidate in &planned.candidates {
        let mut next_entities = entities.clone();
        if planned.source.subject.starts_with(b"?") {
            if let Some(bound) = next_entities.get(&planned.source.subject) {
                if bound != &candidate.state.subject {
                    continue;
                }
            } else {
                next_entities.insert(
                    planned.source.subject.clone(),
                    candidate.state.subject.clone(),
                );
            }
        }
        let linked_as_subject = plans
            .iter()
            .any(|source| source.source.subject == planned.source.parameter);
        if linked_as_subject {
            // A state-selected referent is a runtime join, not a compile-time
            // alias for the selector's initial value. The downstream subject
            // candidates bind the referent and the completed solution records
            // the exact selector equality that guards this specialized rule.
        } else if let Some(CanonicalScalarValueV1::Symbol(value)) = &candidate.value {
            if let Some(bound) = next_entities.get(&planned.source.parameter) {
                if bound != value {
                    continue;
                }
            } else {
                next_entities.insert(planned.source.parameter.clone(), value.clone());
            }
        } else if linked_as_subject || next_entities.contains_key(&planned.source.parameter) {
            continue;
        }
        let mut next_parameters = parameters.clone();
        next_parameters.insert(planned.source.parameter.clone(), candidate.state.clone());
        search_general_binding_solutions(
            plans,
            index + 1,
            next_parameters,
            next_entities,
            solutions,
        );
    }
}

fn resolve_general_parameter_states(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    sources: &[ScalarParameterSourceCst],
    handler_subject: &[u8],
    initial_entities: BTreeMap<Vec<u8>, Vec<u8>>,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<GeneralBindingSolution>, CanonicalSourceErrorV1> {
    let mut planned = Vec::with_capacity(sources.len());
    for source in sources {
        let relation = resolved_state_relation(cst, plan, &source.relation, origin)?;
        planned.push(GeneralSourcePlan {
            source,
            candidates: general_state_candidates(cst, plan, source, &initial_entities, origin)?,
            subject_domain: relation.subject_domain.to_vec(),
            value_domain: general_source_value_domain(cst, &relation, source, origin)?,
            singleton_forward_mode: singleton_forward_mode(&relation),
        });
    }

    let producers = planned
        .iter()
        .map(|source| (source.source.parameter.as_slice(), source))
        .collect::<BTreeMap<_, _>>();
    let mut handler_domain = None::<&[u8]>;
    for source in &planned {
        if !source.source.subject.starts_with(b"?") {
            continue;
        }
        if source.source.subject == handler_subject {
            if let Some(expected) = handler_domain {
                if expected != source.subject_domain {
                    return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
                }
            } else {
                handler_domain = Some(&source.subject_domain);
            }
            continue;
        }
        let producer = producers
            .get(source.source.subject.as_slice())
            .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?;
        if !producer.singleton_forward_mode || producer.value_domain != source.subject_domain {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
        }
    }

    let mut solutions = Vec::new();
    search_general_binding_solutions(
        &planned,
        0,
        BTreeMap::new(),
        initial_entities,
        &mut solutions,
    );
    if solutions.is_empty() {
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin })
    } else {
        Ok(solutions)
    }
}

fn concretize_relation_bindings(
    bindings: &BTreeMap<Vec<u8>, Vec<u8>>,
    entities: &BTreeMap<Vec<u8>, Vec<u8>>,
    origin: CanonicalSourceOriginV1,
) -> Result<BTreeMap<Vec<u8>, Vec<u8>>, CanonicalSourceErrorV1> {
    bindings
        .iter()
        .map(|(role, value)| {
            let concrete = if value.starts_with(b"?") {
                entities
                    .get(value)
                    .cloned()
                    .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?
            } else {
                value.clone()
            };
            Ok((role.clone(), concrete))
        })
        .collect()
}

struct ResolvedBooleanDerive<'a> {
    law: &'a BooleanLawCst,
    derive: &'a BooleanDeriveCst,
    state: CanonicalStateRefV1,
    parameters: BTreeMap<Vec<u8>, CanonicalStateRefV1>,
    bindings: BTreeMap<Vec<u8>, Vec<u8>>,
    value: bool,
}

fn resolved_boolean_derives<'a>(
    cst: &'a CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
) -> Result<Vec<ResolvedBooleanDerive<'a>>, CanonicalSourceErrorV1> {
    let derives = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::BooleanDerive(derive) => Some(derive),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut resolved = Vec::with_capacity(derives.len());
    for derive in derives {
        let laws = cst
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                CstKind::BooleanLaw(law) if law.designation == derive.designation => Some(law),
                _ => None,
            })
            .collect::<Vec<_>>();
        let [law] = laws.as_slice() else {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
                origin: derive.origin,
            });
        };
        let (parameters, entities) =
            resolve_parameter_states(cst, plan, &law.parameter_sources, law.origin)?;
        let result = resolve_boolean_relation_use(cst, &law.result)?;
        let bindings =
            concretize_relation_bindings(&result.bindings, &entities, law.result.origin)?;
        let subject_role = result.relation.subject.as_ref().ok_or(
            CanonicalSourceErrorV1::MissingExecutableBinding {
                origin: law.result.origin,
            },
        )?;
        let subject =
            bindings
                .get(subject_role)
                .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding {
                    origin: law.result.origin,
                })?;
        let producer = semantic_producer(CanonicalSourceProductionV1::Derive, &derive.designation);
        let identity = formation_id(
            plan,
            &producer,
            &head_slot(CanonicalSourceProductionV1::Derive),
        )?;
        let relation = resolved_state_relation_for(plan, result.relation, law.result.origin)?;
        let state = canonical_state_ref_with_identity(
            cst,
            plan,
            identity,
            subject,
            &relation,
            None,
            law.result.origin,
        )?;
        resolved.push(ResolvedBooleanDerive {
            law,
            derive,
            state,
            parameters,
            bindings,
            value: result.value,
        });
    }
    Ok(resolved)
}

fn derived_condition_state_ref(
    cst: &CanonicalSourceCstV1,
    condition: &BooleanRelationUseCst,
    entities: &BTreeMap<Vec<u8>, Vec<u8>>,
    derives: &[ResolvedBooleanDerive<'_>],
) -> Result<(CanonicalStateRefV1, bool), CanonicalSourceErrorV1> {
    let condition_use = resolve_boolean_relation_use(cst, condition)?;
    let bindings =
        concretize_relation_bindings(&condition_use.bindings, entities, condition.origin)?;
    let matching = derives
        .iter()
        .filter(|derive| {
            derive.state.relation_designation == condition_use.relation.designation
                && derive.bindings == bindings
                && derive.value == condition_use.value
        })
        .collect::<Vec<_>>();
    let [derive] = matching.as_slice() else {
        return Err(if matching.is_empty() {
            CanonicalSourceErrorV1::MissingExecutableBinding {
                origin: condition.origin,
            }
        } else {
            CanonicalSourceErrorV1::AmbiguousExecutableBinding {
                origin: condition.origin,
            }
        });
    };
    Ok((derive.state.clone(), condition_use.value))
}

fn checked_source_state_cells(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
) -> Result<Vec<CanonicalStateCellV1>, CanonicalSourceErrorV1> {
    let mut cells = BTreeMap::new();
    let relational_relations = relational_relation_designations(cst);
    for item in &cst.items {
        let assertion_relation = match &item.kind {
            CstKind::VectorAssertion(assertion) => Some(assertion.relation.as_slice()),
            CstKind::ShapeAssertion(assertion) => Some(assertion.relation.as_slice()),
            CstKind::BooleanAssertion(assertion) => Some(assertion.relation.as_slice()),
            CstKind::NumberAssertion(assertion) => Some(assertion.relation.as_slice()),
            CstKind::SymbolAssertion(assertion) => Some(assertion.relation.as_slice()),
            CstKind::TextAssertion(assertion) => Some(assertion.relation.as_slice()),
            _ => None,
        };
        if assertion_relation.is_some_and(|relation| relational_relations.contains(relation)) {
            continue;
        }
        let producer = match &item.kind {
            CstKind::VectorAssertion(assertion) => {
                Some(assertion_producer(&assertion.subject, &assertion.relation))
            }
            CstKind::ShapeAssertion(assertion) => {
                Some(assertion_producer(&assertion.subject, &assertion.relation))
            }
            CstKind::BooleanAssertion(assertion) => {
                Some(assertion_producer(&assertion.subject, &assertion.relation))
            }
            CstKind::NumberAssertion(assertion) => {
                Some(assertion_producer(&assertion.subject, &assertion.relation))
            }
            CstKind::SymbolAssertion(assertion) => {
                Some(assertion_producer(&assertion.subject, &assertion.relation))
            }
            CstKind::TextAssertion(assertion) => {
                Some(assertion_producer(&assertion.subject, &assertion.relation))
            }
            _ => None,
        };
        let Some(producer) = producer else { continue };
        let head = head_slot(CanonicalSourceProductionV1::Assertion);
        if plan
            .identity(&producer, &head, AllocationDomain::Formation)
            .is_none()
        {
            continue;
        }
        match &item.kind {
            CstKind::VectorAssertion(assertion) => {
                for (field, value) in [
                    (b"x".as_slice(), assertion.x),
                    (b"y".as_slice(), assertion.y),
                    (b"z".as_slice(), assertion.z),
                ] {
                    let state = state_ref_for_origin(cst, plan, item.origin, Some(field))?;
                    cells.insert(
                        state.clone(),
                        CanonicalStateCellV1 {
                            state,
                            initial_value: Some(CanonicalScalarValueV1::Number(value)),
                            value_kind: CanonicalScalarValueKindV1::Number,
                        },
                    );
                }
            }
            CstKind::ShapeAssertion(assertion) => {
                validate_shape_assertion(cst, plan, assertion)?;
                for field in &assertion.fields {
                    let state =
                        state_ref_for_origin(cst, plan, item.origin, Some(field.name.as_slice()))?;
                    let value_kind = match field.value {
                        CanonicalScalarValueV1::Number(_) => CanonicalScalarValueKindV1::Number,
                        CanonicalScalarValueV1::Boolean(_) => CanonicalScalarValueKindV1::Boolean,
                        CanonicalScalarValueV1::Symbol(_) => CanonicalScalarValueKindV1::Symbol,
                        CanonicalScalarValueV1::Text(_) => CanonicalScalarValueKindV1::Text,
                        CanonicalScalarValueV1::Referent(_) => CanonicalScalarValueKindV1::Referent,
                        CanonicalScalarValueV1::RelationTable(_) => {
                            CanonicalScalarValueKindV1::RelationTable
                        }
                    };
                    cells.insert(
                        state.clone(),
                        CanonicalStateCellV1 {
                            state,
                            initial_value: Some(field.value.clone()),
                            value_kind,
                        },
                    );
                }
            }
            CstKind::BooleanAssertion(assertion) => {
                let state = state_ref_for_origin(cst, plan, item.origin, None)?;
                cells.insert(
                    state.clone(),
                    CanonicalStateCellV1 {
                        state,
                        initial_value: Some(CanonicalScalarValueV1::Boolean(assertion.value)),
                        value_kind: CanonicalScalarValueKindV1::Boolean,
                    },
                );
            }
            CstKind::NumberAssertion(assertion) => {
                let state = state_ref_for_origin(cst, plan, item.origin, None)?;
                cells.insert(
                    state.clone(),
                    CanonicalStateCellV1 {
                        state,
                        initial_value: Some(CanonicalScalarValueV1::Number(assertion.value)),
                        value_kind: CanonicalScalarValueKindV1::Number,
                    },
                );
            }
            CstKind::SymbolAssertion(assertion) => {
                let state = state_ref_for_origin(cst, plan, item.origin, None)?;
                cells.insert(
                    state.clone(),
                    CanonicalStateCellV1 {
                        state,
                        initial_value: Some(CanonicalScalarValueV1::Symbol(
                            assertion.value.clone(),
                        )),
                        value_kind: CanonicalScalarValueKindV1::Symbol,
                    },
                );
            }
            CstKind::TextAssertion(assertion) => {
                let state = state_ref_for_origin(cst, plan, item.origin, None)?;
                cells.insert(
                    state.clone(),
                    CanonicalStateCellV1 {
                        state,
                        initial_value: Some(CanonicalScalarValueV1::Text(assertion.value.clone())),
                        value_kind: CanonicalScalarValueKindV1::Text,
                    },
                );
            }
            _ => unreachable!("the producer filter selected an assertion"),
        }
    }
    for handler in cst.items.iter().filter_map(|item| match &item.kind {
        CstKind::GeneralHandler(handler) => Some(handler),
        _ => None,
    }) {
        if relational_handler_origins(cst).contains(&handler.origin) {
            continue;
        }
        let subject = concrete_general_handler_subject(cst, handler)?;
        for insertion in &handler.insertions {
            let kind = optional_state_value_kind(cst, plan, &insertion.target, handler.origin)?;
            match state_relation_cardinality(cst, plan, &insertion.target, handler.origin)? {
                SourceCardinality::Maybe => {
                    require_optional_state_relation(cst, plan, &insertion.target, handler.origin)?;
                    let state = canonical_state_ref(
                        cst,
                        plan,
                        &subject,
                        &insertion.target.relation,
                        insertion.target.field.as_deref(),
                        handler.origin,
                    )?;
                    cells.entry(state.clone()).or_insert(CanonicalStateCellV1 {
                        state,
                        initial_value: None,
                        value_kind: kind,
                    });
                }
                SourceCardinality::Many if insertion.target.field.is_none() => {
                    let state = canonical_many_state_ref(
                        cst,
                        plan,
                        &subject,
                        &insertion.target.relation,
                        handler.origin,
                    )?;
                    cells.entry(state.clone()).or_insert(CanonicalStateCellV1 {
                        state,
                        initial_value: None,
                        value_kind: kind,
                    });
                }
                _ => {
                    return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
                        origin: handler.origin,
                    });
                }
            }
        }
        for membership in &handler.membership_sources {
            if membership.field.is_some()
                || state_relation_cardinality(cst, plan, membership, handler.origin)?
                    != SourceCardinality::Many
            {
                return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
                    origin: handler.origin,
                });
            }
            let state = canonical_many_state_ref(
                cst,
                plan,
                &subject,
                &membership.relation,
                handler.origin,
            )?;
            cells.entry(state.clone()).or_insert(CanonicalStateCellV1 {
                state,
                initial_value: None,
                value_kind: optional_state_value_kind(cst, plan, membership, handler.origin)?,
            });
        }
    }
    for derive in resolved_boolean_derives(cst, plan)? {
        cells.insert(
            derive.state.clone(),
            CanonicalStateCellV1 {
                state: derive.state,
                initial_value: Some(CanonicalScalarValueV1::Boolean(!derive.value)),
                value_kind: CanonicalScalarValueKindV1::Boolean,
            },
        );
    }
    for relation in relational_relations {
        let origin = cst
            .items
            .iter()
            .find_map(|item| match &item.kind {
                CstKind::Relation(candidate) if candidate.surface == relation => Some(item.origin),
                _ => None,
            })
            .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding {
                origin: CanonicalSourceOriginV1 {
                    artifact: cst.artifact,
                    start: 0,
                    end: 0,
                },
            })?;
        let state = relation_table_state_ref(cst, plan, &relation, origin)?;
        cells.insert(
            state.clone(),
            CanonicalStateCellV1 {
                state,
                initial_value: Some(CanonicalScalarValueV1::RelationTable(
                    canonical_relation_table(cst, plan, &relation, origin)?,
                )),
                value_kind: CanonicalScalarValueKindV1::RelationTable,
            },
        );
    }
    Ok(cells.into_values().collect())
}

fn constant_expression(value: CanonicalScalarValueV1) -> CanonicalExecutableExpressionV1 {
    CanonicalExecutableExpressionV1::Constant(value)
}

fn canonical_input_expression(value: CanonicalInputScalarV1) -> CanonicalExecutableExpressionV1 {
    match value {
        CanonicalInputScalarV1::Parameter(index) => {
            CanonicalExecutableExpressionV1::Argument(index)
        }
        CanonicalInputScalarV1::Number(bits) => {
            constant_expression(CanonicalScalarValueV1::Number(bits))
        }
    }
}

fn canonical_scalar_executable_expression(
    expression: &CanonicalScalarExpressionV1,
    current: &CanonicalStateRefV1,
    parameters: &BTreeMap<Vec<u8>, CanonicalStateRefV1>,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    let pair = |left: &CanonicalScalarExpressionV1,
                right: &CanonicalScalarExpressionV1|
     -> Result<_, CanonicalSourceErrorV1> {
        Ok((
            Box::new(canonical_scalar_executable_expression(
                left, current, parameters, origin,
            )?),
            Box::new(canonical_scalar_executable_expression(
                right, current, parameters, origin,
            )?),
        ))
    };
    Ok(match expression {
        CanonicalScalarExpressionV1::Current => {
            CanonicalExecutableExpressionV1::State(current.clone())
        }
        CanonicalScalarExpressionV1::Parameter(parameter) => {
            CanonicalExecutableExpressionV1::State(
                parameters
                    .get(parameter)
                    .cloned()
                    .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?,
            )
        }
        CanonicalScalarExpressionV1::Number(bits) => {
            constant_expression(CanonicalScalarValueV1::Number(*bits))
        }
        CanonicalScalarExpressionV1::Boolean(value) => {
            constant_expression(CanonicalScalarValueV1::Boolean(*value))
        }
        CanonicalScalarExpressionV1::Symbol(value) => {
            constant_expression(CanonicalScalarValueV1::Symbol(value.clone()))
        }
        CanonicalScalarExpressionV1::Text(value) => {
            constant_expression(CanonicalScalarValueV1::Text(value.clone()))
        }
        CanonicalScalarExpressionV1::Concatenate(left, right) => {
            let (left, right) = pair(left, right)?;
            CanonicalExecutableExpressionV1::Concatenate(left, right)
        }
        CanonicalScalarExpressionV1::Add(left, right) => {
            let (left, right) = pair(left, right)?;
            CanonicalExecutableExpressionV1::Add(left, right)
        }
        CanonicalScalarExpressionV1::Subtract(left, right) => {
            let (left, right) = pair(left, right)?;
            CanonicalExecutableExpressionV1::Subtract(left, right)
        }
        CanonicalScalarExpressionV1::Multiply(left, right) => {
            let (left, right) = pair(left, right)?;
            CanonicalExecutableExpressionV1::Multiply(left, right)
        }
        CanonicalScalarExpressionV1::Divide(left, right) => {
            let (left, right) = pair(left, right)?;
            CanonicalExecutableExpressionV1::Divide(left, right)
        }
        CanonicalScalarExpressionV1::Clamp(value, lower, upper) => {
            CanonicalExecutableExpressionV1::Clamp(
                Box::new(canonical_scalar_executable_expression(
                    value, current, parameters, origin,
                )?),
                Box::new(canonical_scalar_executable_expression(
                    lower, current, parameters, origin,
                )?),
                Box::new(canonical_scalar_executable_expression(
                    upper, current, parameters, origin,
                )?),
            )
        }
    })
}

fn expand_scalar_law_bindings(
    expression: &CanonicalScalarExpressionV1,
    bindings: &BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>,
    expanding: &mut BTreeSet<Vec<u8>>,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalScalarExpressionV1, CanonicalSourceErrorV1> {
    let pair = |left: &CanonicalScalarExpressionV1,
                right: &CanonicalScalarExpressionV1,
                expanding: &mut BTreeSet<Vec<u8>>|
     -> Result<_, CanonicalSourceErrorV1> {
        Ok((
            Box::new(expand_scalar_law_bindings(
                left, bindings, expanding, origin,
            )?),
            Box::new(expand_scalar_law_bindings(
                right, bindings, expanding, origin,
            )?),
        ))
    };
    Ok(match expression {
        CanonicalScalarExpressionV1::Parameter(parameter) if bindings.contains_key(parameter) => {
            if !expanding.insert(parameter.clone()) {
                return Err(CanonicalSourceErrorV1::AmbiguousExecutableBinding { origin });
            }
            let expanded = expand_scalar_law_bindings(
                bindings.get(parameter).expect("checked scalar binding"),
                bindings,
                expanding,
                origin,
            )?;
            expanding.remove(parameter);
            expanded
        }
        CanonicalScalarExpressionV1::Add(left, right) => {
            let (left, right) = pair(left, right, expanding)?;
            CanonicalScalarExpressionV1::Add(left, right)
        }
        CanonicalScalarExpressionV1::Concatenate(left, right) => {
            let (left, right) = pair(left, right, expanding)?;
            CanonicalScalarExpressionV1::Concatenate(left, right)
        }
        CanonicalScalarExpressionV1::Subtract(left, right) => {
            let (left, right) = pair(left, right, expanding)?;
            CanonicalScalarExpressionV1::Subtract(left, right)
        }
        CanonicalScalarExpressionV1::Multiply(left, right) => {
            let (left, right) = pair(left, right, expanding)?;
            CanonicalScalarExpressionV1::Multiply(left, right)
        }
        CanonicalScalarExpressionV1::Divide(left, right) => {
            let (left, right) = pair(left, right, expanding)?;
            CanonicalScalarExpressionV1::Divide(left, right)
        }
        CanonicalScalarExpressionV1::Clamp(value, lower, upper) => {
            CanonicalScalarExpressionV1::Clamp(
                Box::new(expand_scalar_law_bindings(
                    value, bindings, expanding, origin,
                )?),
                Box::new(expand_scalar_law_bindings(
                    lower, bindings, expanding, origin,
                )?),
                Box::new(expand_scalar_law_bindings(
                    upper, bindings, expanding, origin,
                )?),
            )
        }
        expression => expression.clone(),
    })
}

fn canonical_general_executable_expression(
    expression: &CanonicalScalarExpressionV1,
    current: &CanonicalStateRefV1,
    parameters: &BTreeMap<Vec<u8>, CanonicalStateRefV1>,
    arguments: &BTreeMap<Vec<u8>, u16>,
    bindings: &BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    let expression =
        expand_scalar_law_bindings(expression, bindings, &mut BTreeSet::new(), origin)?;
    if let CanonicalScalarExpressionV1::Parameter(parameter) = &expression
        && let Some(ordinal) = arguments.get(parameter)
    {
        return Ok(CanonicalExecutableExpressionV1::Argument(*ordinal));
    }
    let lower_expression = |expression: &CanonicalScalarExpressionV1| {
        canonical_general_executable_expression(
            expression, current, parameters, arguments, bindings, origin,
        )
    };
    let pair = |left: &CanonicalScalarExpressionV1,
                right: &CanonicalScalarExpressionV1|
     -> Result<_, CanonicalSourceErrorV1> {
        Ok((
            Box::new(lower_expression(left)?),
            Box::new(lower_expression(right)?),
        ))
    };
    match &expression {
        CanonicalScalarExpressionV1::Concatenate(left, right) => {
            let (left, right) = pair(left, right)?;
            Ok(CanonicalExecutableExpressionV1::Concatenate(left, right))
        }
        CanonicalScalarExpressionV1::Add(left, right) => {
            let (left, right) = pair(left, right)?;
            Ok(CanonicalExecutableExpressionV1::Add(left, right))
        }
        CanonicalScalarExpressionV1::Subtract(left, right) => {
            let (left, right) = pair(left, right)?;
            Ok(CanonicalExecutableExpressionV1::Subtract(left, right))
        }
        CanonicalScalarExpressionV1::Multiply(left, right) => {
            let (left, right) = pair(left, right)?;
            Ok(CanonicalExecutableExpressionV1::Multiply(left, right))
        }
        CanonicalScalarExpressionV1::Divide(left, right) => {
            let (left, right) = pair(left, right)?;
            Ok(CanonicalExecutableExpressionV1::Divide(left, right))
        }
        CanonicalScalarExpressionV1::Clamp(value, lower, upper) => {
            Ok(CanonicalExecutableExpressionV1::Clamp(
                Box::new(lower_expression(value)?),
                Box::new(lower_expression(lower)?),
                Box::new(lower_expression(upper)?),
            ))
        }
        _ => canonical_scalar_executable_expression(&expression, current, parameters, origin),
    }
}

fn canonical_general_executable_predicates(
    predicates: &[CanonicalScalarPredicateV1],
    current: &CanonicalStateRefV1,
    parameters: &BTreeMap<Vec<u8>, CanonicalStateRefV1>,
    arguments: &BTreeMap<Vec<u8>, u16>,
    bindings: &BTreeMap<Vec<u8>, CanonicalScalarExpressionV1>,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<CanonicalExecutablePredicateV1>, CanonicalSourceErrorV1> {
    predicates
        .iter()
        .map(|predicate| {
            let (left, right, constructor) = match predicate {
                CanonicalScalarPredicateV1::Equal(left, right) => (
                    left,
                    right,
                    CanonicalExecutablePredicateV1::Equal
                        as fn(_, _) -> CanonicalExecutablePredicateV1,
                ),
                CanonicalScalarPredicateV1::GreaterThan(left, right) => (
                    left,
                    right,
                    CanonicalExecutablePredicateV1::GreaterThan
                        as fn(_, _) -> CanonicalExecutablePredicateV1,
                ),
                CanonicalScalarPredicateV1::LessThanOrEqual(left, right) => (
                    left,
                    right,
                    CanonicalExecutablePredicateV1::LessThanOrEqual
                        as fn(_, _) -> CanonicalExecutablePredicateV1,
                ),
            };
            Ok(constructor(
                canonical_general_executable_expression(
                    left, current, parameters, arguments, bindings, origin,
                )?,
                canonical_general_executable_expression(
                    right, current, parameters, arguments, bindings, origin,
                )?,
            ))
        })
        .collect()
}

fn canonical_scalar_executable_predicates(
    predicates: &[CanonicalScalarPredicateV1],
    current: &CanonicalStateRefV1,
    parameters: &BTreeMap<Vec<u8>, CanonicalStateRefV1>,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<CanonicalExecutablePredicateV1>, CanonicalSourceErrorV1> {
    predicates
        .iter()
        .map(|predicate| {
            let (left, right, constructor) = match predicate {
                CanonicalScalarPredicateV1::Equal(left, right) => (
                    left,
                    right,
                    CanonicalExecutablePredicateV1::Equal
                        as fn(_, _) -> CanonicalExecutablePredicateV1,
                ),
                CanonicalScalarPredicateV1::GreaterThan(left, right) => (
                    left,
                    right,
                    CanonicalExecutablePredicateV1::GreaterThan
                        as fn(_, _) -> CanonicalExecutablePredicateV1,
                ),
                CanonicalScalarPredicateV1::LessThanOrEqual(left, right) => (
                    left,
                    right,
                    CanonicalExecutablePredicateV1::LessThanOrEqual
                        as fn(_, _) -> CanonicalExecutablePredicateV1,
                ),
            };
            Ok(constructor(
                canonical_scalar_executable_expression(left, current, parameters, origin)?,
                canonical_scalar_executable_expression(right, current, parameters, origin)?,
            ))
        })
        .collect()
}

fn tick_state_ref(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    parts: TickProgramParts<'_>,
    value: CanonicalTickValueV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    let state = match value {
        CanonicalTickValueV1::DeltaTime => {
            return Ok(CanonicalExecutableExpressionV1::Argument(0));
        }
        CanonicalTickValueV1::PositionComponent(index) => state_ref_for_origin(
            cst,
            plan,
            parts.position.origin,
            Some([b"x", b"y", b"z"][usize::from(index)]),
        )?,
        CanonicalTickValueV1::VelocityComponent(index) => state_ref_for_origin(
            cst,
            plan,
            parts.velocity.origin,
            Some([b"x", b"y", b"z"][usize::from(index)]),
        )?,
        CanonicalTickValueV1::IntentComponent(index) => state_ref_for_origin(
            cst,
            plan,
            parts.intent.origin,
            Some([b"x", b"y", b"z"][usize::from(index)]),
        )?,
        CanonicalTickValueV1::Grounded => {
            state_ref_for_origin(cst, plan, parts.grounded.origin, None)?
        }
        CanonicalTickValueV1::Gravity => {
            state_ref_for_origin(cst, plan, parts.gravity.origin, None)?
        }
        CanonicalTickValueV1::MoveSpeed => {
            state_ref_for_origin(cst, plan, parts.move_speed.origin, None)?
        }
        CanonicalTickValueV1::FloorHeight => {
            state_ref_for_origin(cst, plan, parts.floor_height.origin, None)?
        }
        CanonicalTickValueV1::MinimumX => {
            state_ref_for_origin(cst, plan, parts.minimum_x.origin, None)?
        }
        CanonicalTickValueV1::MaximumX => {
            state_ref_for_origin(cst, plan, parts.maximum_x.origin, None)?
        }
        CanonicalTickValueV1::MinimumZ => {
            state_ref_for_origin(cst, plan, parts.minimum_z.origin, None)?
        }
        CanonicalTickValueV1::MaximumZ => {
            state_ref_for_origin(cst, plan, parts.maximum_z.origin, None)?
        }
    };
    Ok(CanonicalExecutableExpressionV1::State(state))
}

fn tick_executable_expression(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    parts: TickProgramParts<'_>,
    expression: &CanonicalTickExpressionV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    let pair = |left: &CanonicalTickExpressionV1,
                right: &CanonicalTickExpressionV1|
     -> Result<_, CanonicalSourceErrorV1> {
        Ok((
            Box::new(tick_executable_expression(cst, plan, parts, left)?),
            Box::new(tick_executable_expression(cst, plan, parts, right)?),
        ))
    };
    Ok(match expression {
        CanonicalTickExpressionV1::Value(value) => tick_state_ref(cst, plan, parts, *value)?,
        CanonicalTickExpressionV1::Number(bits) => {
            constant_expression(CanonicalScalarValueV1::Number(*bits))
        }
        CanonicalTickExpressionV1::Add(left, right) => {
            let (left, right) = pair(left, right)?;
            CanonicalExecutableExpressionV1::Add(left, right)
        }
        CanonicalTickExpressionV1::Subtract(left, right) => {
            let (left, right) = pair(left, right)?;
            CanonicalExecutableExpressionV1::Subtract(left, right)
        }
        CanonicalTickExpressionV1::Multiply(left, right) => {
            let (left, right) = pair(left, right)?;
            CanonicalExecutableExpressionV1::Multiply(left, right)
        }
        CanonicalTickExpressionV1::Divide(left, right) => {
            let (left, right) = pair(left, right)?;
            CanonicalExecutableExpressionV1::Divide(left, right)
        }
        CanonicalTickExpressionV1::Clamp(value, lower, upper) => {
            CanonicalExecutableExpressionV1::Clamp(
                Box::new(tick_executable_expression(cst, plan, parts, value)?),
                Box::new(tick_executable_expression(cst, plan, parts, lower)?),
                Box::new(tick_executable_expression(cst, plan, parts, upper)?),
            )
        }
    })
}

fn tick_assignment_target(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    parts: TickProgramParts<'_>,
    target: CanonicalTickAssignmentTargetV1,
) -> Result<CanonicalStateRefV1, CanonicalSourceErrorV1> {
    match target {
        CanonicalTickAssignmentTargetV1::PositionComponent(index) => state_ref_for_origin(
            cst,
            plan,
            parts.position.origin,
            Some([b"x", b"y", b"z"][usize::from(index)]),
        ),
        CanonicalTickAssignmentTargetV1::VelocityComponent(index) => state_ref_for_origin(
            cst,
            plan,
            parts.velocity.origin,
            Some([b"x", b"y", b"z"][usize::from(index)]),
        ),
        CanonicalTickAssignmentTargetV1::Grounded => {
            state_ref_for_origin(cst, plan, parts.grounded.origin, None)
        }
    }
}

fn relational_subject_expression(
    variables: &BTreeMap<Vec<u8>, CanonicalExecutableExpressionV1>,
    subject: &[u8],
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    variables
        .get(subject)
        .cloned()
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })
}

fn relational_scalar_expression(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    expression: &CanonicalScalarExpressionV1,
    variables: &BTreeMap<Vec<u8>, CanonicalExecutableExpressionV1>,
    expected_domain: Option<&[u8]>,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    let lower_expression = |expression: &CanonicalScalarExpressionV1,
                            expected_domain: Option<&[u8]>| {
        relational_scalar_expression(cst, plan, expression, variables, expected_domain, origin)
    };
    let pair = |left: &CanonicalScalarExpressionV1,
                right: &CanonicalScalarExpressionV1,
                expected_domain: Option<&[u8]>|
     -> Result<_, CanonicalSourceErrorV1> {
        Ok((
            Box::new(lower_expression(left, expected_domain)?),
            Box::new(lower_expression(right, expected_domain)?),
        ))
    };
    Ok(match expression {
        CanonicalScalarExpressionV1::Current => {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
        }
        CanonicalScalarExpressionV1::Parameter(parameter) => variables
            .get(parameter)
            .cloned()
            .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding { origin })?,
        CanonicalScalarExpressionV1::Number(bits) => {
            constant_expression(CanonicalScalarValueV1::Number(*bits))
        }
        CanonicalScalarExpressionV1::Boolean(value) => {
            constant_expression(CanonicalScalarValueV1::Boolean(*value))
        }
        CanonicalScalarExpressionV1::Symbol(value) => {
            if let Some(domain) =
                expected_domain.filter(|domain| !matches!(*domain, b"F64" | b"Bool" | b"Text"))
            {
                constant_expression(CanonicalScalarValueV1::Referent(declared_referent_value(
                    cst, plan, value, domain, origin,
                )?))
            } else {
                constant_expression(CanonicalScalarValueV1::Symbol(value.clone()))
            }
        }
        CanonicalScalarExpressionV1::Text(value) => {
            constant_expression(CanonicalScalarValueV1::Text(value.clone()))
        }
        CanonicalScalarExpressionV1::Concatenate(left, right) => {
            let (left, right) = pair(left, right, Some(b"Text"))?;
            CanonicalExecutableExpressionV1::Concatenate(left, right)
        }
        CanonicalScalarExpressionV1::Add(left, right) => {
            let (left, right) = pair(left, right, Some(b"F64"))?;
            CanonicalExecutableExpressionV1::Add(left, right)
        }
        CanonicalScalarExpressionV1::Subtract(left, right) => {
            let (left, right) = pair(left, right, Some(b"F64"))?;
            CanonicalExecutableExpressionV1::Subtract(left, right)
        }
        CanonicalScalarExpressionV1::Multiply(left, right) => {
            let (left, right) = pair(left, right, Some(b"F64"))?;
            CanonicalExecutableExpressionV1::Multiply(left, right)
        }
        CanonicalScalarExpressionV1::Divide(left, right) => {
            let (left, right) = pair(left, right, Some(b"F64"))?;
            CanonicalExecutableExpressionV1::Divide(left, right)
        }
        CanonicalScalarExpressionV1::Clamp(value, lower, upper) => {
            CanonicalExecutableExpressionV1::Clamp(
                Box::new(lower_expression(value, Some(b"F64"))?),
                Box::new(lower_expression(lower, Some(b"F64"))?),
                Box::new(lower_expression(upper, Some(b"F64"))?),
            )
        }
    })
}

fn relation_read_expression(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &ScalarParameterSourceCst,
    subject: CanonicalExecutableExpressionV1,
    origin: CanonicalSourceOriginV1,
) -> Result<CanonicalExecutableExpressionV1, CanonicalSourceErrorV1> {
    let table = relation_table_state_ref(cst, plan, &source.relation, origin)?;
    Ok(CanonicalExecutableExpressionV1::RelationRead(
        Box::new(CanonicalExecutableExpressionV1::State(table)),
        Box::new(subject),
    ))
}

fn relational_checked_handler(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    source: &GeneralHandlerCst,
) -> Result<CanonicalExecutableHandlerV1, CanonicalSourceErrorV1> {
    let handler_id = formation_id(
        plan,
        &source.producer,
        &head_slot(CanonicalSourceProductionV1::Handler),
    )?;
    let subject_domain = source
        .parameter_sources
        .iter()
        .chain(&source.membership_sources)
        .chain(&source.required_sources)
        .find(|state| state.subject == source.subject)
        .map(|state| resolved_state_relation(cst, plan, &state.relation, source.origin))
        .transpose()?
        .map(|relation| relation.subject_domain)
        .ok_or(CanonicalSourceErrorV1::MissingExecutableBinding {
            origin: source.origin,
        })?;
    let concrete_subject = match concrete_general_handler_subject(cst, source) {
        Ok(subject) => subject,
        Err(CanonicalSourceErrorV1::MissingExecutableBinding { .. }) => {
            let matching = cst
                .items
                .iter()
                .filter_map(|item| match &item.kind {
                    CstKind::Membership(membership)
                        if membership
                            .domains
                            .iter()
                            .any(|domain| domain == subject_domain) =>
                    {
                        Some(membership.subject.clone())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let [subject] = matching.as_slice() else {
                return Err(if matching.is_empty() {
                    CanonicalSourceErrorV1::MissingExecutableBinding {
                        origin: source.origin,
                    }
                } else {
                    CanonicalSourceErrorV1::AmbiguousExecutableBinding {
                        origin: source.origin,
                    }
                });
            };
            subject.clone()
        }
        Err(error) => return Err(error),
    };
    let mut variables = BTreeMap::from([(
        source.subject.clone(),
        constant_expression(CanonicalScalarValueV1::Referent(declared_referent_value(
            cst,
            plan,
            &concrete_subject,
            subject_domain,
            source.origin,
        )?)),
    )]);
    variables.extend(source.arguments.iter().map(|argument| {
        (
            argument.designation.clone(),
            CanonicalExecutableExpressionV1::Argument(argument.ordinal),
        )
    }));
    for creation in &source.creations {
        variables.insert(
            creation.parameter.clone(),
            CanonicalExecutableExpressionV1::FreshReferent {
                domain: referent_type_id(cst, plan, &creation.domain, source.origin)?,
                binder: creation.binder,
            },
        );
    }

    let mut predicates = Vec::new();
    let mut unresolved = source.parameter_sources.iter().collect::<Vec<_>>();
    while !unresolved.is_empty() {
        let prior = unresolved.len();
        let mut next = Vec::new();
        for parameter in unresolved {
            let Some(subject) = variables.get(&parameter.subject).cloned() else {
                next.push(parameter);
                continue;
            };
            let table = relation_table_state_ref(cst, plan, &parameter.relation, source.origin)?;
            predicates.push(CanonicalExecutablePredicateV1::Equal(
                CanonicalExecutableExpressionV1::RelationPresent(
                    Box::new(CanonicalExecutableExpressionV1::State(table)),
                    Box::new(subject.clone()),
                ),
                constant_expression(CanonicalScalarValueV1::Boolean(true)),
            ));
            variables.insert(
                parameter.parameter.clone(),
                relation_read_expression(cst, plan, parameter, subject, source.origin)?,
            );
        }
        if next.len() == prior {
            return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
                origin: source.origin,
            });
        }
        unresolved = next;
    }

    for membership in &source.membership_sources {
        let subject =
            relational_subject_expression(&variables, &membership.subject, source.origin)?;
        let value =
            relational_subject_expression(&variables, &membership.parameter, source.origin)?;
        let read = relation_read_expression(cst, plan, membership, subject, source.origin)?;
        predicates.push(CanonicalExecutablePredicateV1::Contains(read, value));
    }
    for required in &source.required_sources {
        let subject = relational_subject_expression(&variables, &required.subject, source.origin)?;
        let table = relation_table_state_ref(cst, plan, &required.relation, source.origin)?;
        predicates.push(CanonicalExecutablePredicateV1::Equal(
            CanonicalExecutableExpressionV1::RelationPresent(
                Box::new(CanonicalExecutableExpressionV1::State(table)),
                Box::new(subject),
            ),
            constant_expression(CanonicalScalarValueV1::Boolean(true)),
        ));
    }
    for predicate in &source.predicates {
        let (left, right, constructor) = match predicate {
            CanonicalScalarPredicateV1::Equal(left, right) => (
                left,
                right,
                CanonicalExecutablePredicateV1::Equal as fn(_, _) -> _,
            ),
            CanonicalScalarPredicateV1::GreaterThan(left, right) => (
                left,
                right,
                CanonicalExecutablePredicateV1::GreaterThan as fn(_, _) -> _,
            ),
            CanonicalScalarPredicateV1::LessThanOrEqual(left, right) => (
                left,
                right,
                CanonicalExecutablePredicateV1::LessThanOrEqual as fn(_, _) -> _,
            ),
        };
        let expected = [left, right].into_iter().find_map(|expression| {
            let CanonicalScalarExpressionV1::Parameter(parameter) = expression else {
                return None;
            };
            source
                .parameter_sources
                .iter()
                .find(|candidate| candidate.parameter == *parameter)
                .and_then(|candidate| {
                    resolved_state_relation(cst, plan, &candidate.relation, source.origin)
                        .ok()
                        .map(|relation| relation.value_domain)
                })
        });
        predicates.push(constructor(
            relational_scalar_expression(cst, plan, left, &variables, expected, source.origin)?,
            relational_scalar_expression(cst, plan, right, &variables, expected, source.origin)?,
        ));
    }

    let mut table_assignments =
        BTreeMap::<CanonicalStateRefV1, CanonicalExecutableExpressionV1>::new();
    let mut mutate = |target: &ScalarParameterSourceCst,
                      value: Option<&CanonicalScalarExpressionV1>,
                      insertion: bool,
                      removal: bool|
     -> Result<(), CanonicalSourceErrorV1> {
        let state = relation_table_state_ref(cst, plan, &target.relation, source.origin)?;
        let subject = relational_subject_expression(&variables, &target.subject, source.origin)?;
        let current = table_assignments
            .remove(&state)
            .unwrap_or_else(|| CanonicalExecutableExpressionV1::State(state.clone()));
        let next = if removal {
            if state_relation_cardinality(cst, plan, target, source.origin)?
                == SourceCardinality::Many
            {
                let removed =
                    relational_subject_expression(&variables, &target.parameter, source.origin)?;
                CanonicalExecutableExpressionV1::RelationRemoveValue(
                    Box::new(current),
                    Box::new(subject),
                    Box::new(removed),
                )
            } else {
                CanonicalExecutableExpressionV1::RelationRemoveRow(
                    Box::new(current),
                    Box::new(subject),
                )
            }
        } else {
            let value = value.ok_or(CanonicalSourceErrorV1::InvalidGeneralHandler {
                origin: source.origin,
            })?;
            let relation = resolved_state_relation(cst, plan, &target.relation, source.origin)?;
            let value = relational_scalar_expression(
                cst,
                plan,
                value,
                &variables,
                Some(relation.value_domain),
                source.origin,
            )?;
            if insertion
                && state_relation_cardinality(cst, plan, target, source.origin)?
                    == SourceCardinality::Many
            {
                CanonicalExecutableExpressionV1::RelationInsert(
                    Box::new(current),
                    Box::new(subject),
                    Box::new(value),
                )
            } else {
                if insertion {
                    predicates.push(CanonicalExecutablePredicateV1::Equal(
                        CanonicalExecutableExpressionV1::RelationPresent(
                            Box::new(CanonicalExecutableExpressionV1::State(state.clone())),
                            Box::new(subject.clone()),
                        ),
                        constant_expression(CanonicalScalarValueV1::Boolean(false)),
                    ));
                }
                CanonicalExecutableExpressionV1::RelationPut(
                    Box::new(current),
                    Box::new(subject),
                    Box::new(value),
                )
            }
        };
        table_assignments.insert(state, next);
        Ok(())
    };
    for assignment in &source.assignments {
        mutate(&assignment.target, Some(&assignment.value), false, false)?;
    }
    for insertion in &source.insertions {
        mutate(&insertion.target, Some(&insertion.value), true, false)?;
    }
    for removal in &source.removals {
        mutate(removal, None, false, true)?;
    }

    Ok(CanonicalExecutableHandlerV1 {
        id: handler_id,
        designation: source.designation.clone(),
        trigger: CanonicalHandlerTriggerV1::External,
        argument_count: u16::try_from(source.arguments.len()).map_err(|_| {
            CanonicalSourceErrorV1::InvalidGeneralHandler {
                origin: source.origin,
            }
        })?,
        rules: vec![CanonicalExecutableRuleV1 {
            predicates,
            required_present: vec![],
            required_absent: vec![],
            assignments: table_assignments
                .into_iter()
                .map(|(target, value)| CanonicalExecutableAssignmentV1 { target, value })
                .collect(),
            removals: vec![],
        }],
    })
}

fn checked_executable_handlers(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    input: Option<(&InputHandlerCst, &VectorAssertionCst)>,
    jump: Option<JumpHandlerParts<'_>>,
    scalar: &[ScalarHandlerParts<'_>],
    tick: Option<TickProgramParts<'_>>,
    keyboard: &[CanonicalKeyboardBindingV1],
) -> Result<Vec<CanonicalExecutableHandlerV1>, CanonicalSourceErrorV1> {
    let mut handlers = Vec::new();
    let handler_id = |producer: &CanonicalSemanticProducerV1| {
        formation_id(
            plan,
            producer,
            &head_slot(CanonicalSourceProductionV1::Handler),
        )
    };
    if let Some((source, assertion)) = input {
        let x = state_ref_for_origin(cst, plan, assertion.origin, Some(b"x"))?;
        let z = state_ref_for_origin(cst, plan, assertion.origin, Some(b"z"))?;
        handlers.push(CanonicalExecutableHandlerV1 {
            id: handler_id(&source.producer)?,
            designation: source.designation.clone(),
            trigger: CanonicalHandlerTriggerV1::External,
            argument_count: 2,
            rules: vec![CanonicalExecutableRuleV1 {
                predicates: vec![],
                required_present: vec![],
                required_absent: vec![],
                assignments: vec![
                    CanonicalExecutableAssignmentV1 {
                        target: x,
                        value: canonical_input_expression(source.result_x),
                    },
                    CanonicalExecutableAssignmentV1 {
                        target: z,
                        value: canonical_input_expression(source.result_z),
                    },
                ],
                removals: vec![],
            }],
        });
    }
    if let Some(parts) = jump {
        let velocity = [b"x".as_slice(), b"y".as_slice(), b"z".as_slice()]
            .map(|field| state_ref_for_origin(cst, plan, parts.velocity.origin, Some(field)))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        let grounded = state_ref_for_origin(cst, plan, parts.grounded.origin, None)?;
        let jump_speed = state_ref_for_origin(cst, plan, parts.jump_speed.origin, None)?;
        let jump_expression = |value| match value {
            CanonicalJumpScalarV1::VelocityComponent(index) => velocity
                .get(usize::from(index))
                .cloned()
                .map(CanonicalExecutableExpressionV1::State),
            CanonicalJumpScalarV1::JumpSpeed => {
                Some(CanonicalExecutableExpressionV1::State(jump_speed.clone()))
            }
            CanonicalJumpScalarV1::Number(bits) => {
                Some(constant_expression(CanonicalScalarValueV1::Number(bits)))
            }
        };
        let mut assignments = velocity
            .iter()
            .cloned()
            .zip(parts.handler.result_velocity)
            .map(|(target, value)| {
                Ok(CanonicalExecutableAssignmentV1 {
                    target,
                    value: jump_expression(value).ok_or(
                        CanonicalSourceErrorV1::InvalidJumpHandler {
                            origin: parts.handler.origin,
                        },
                    )?,
                })
            })
            .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?;
        assignments.push(CanonicalExecutableAssignmentV1 {
            target: grounded.clone(),
            value: constant_expression(CanonicalScalarValueV1::Boolean(
                parts.handler.result_grounded,
            )),
        });
        handlers.push(CanonicalExecutableHandlerV1 {
            id: handler_id(&parts.handler.producer)?,
            designation: parts.handler.designation.clone(),
            trigger: CanonicalHandlerTriggerV1::External,
            argument_count: 0,
            rules: vec![CanonicalExecutableRuleV1 {
                predicates: vec![CanonicalExecutablePredicateV1::Equal(
                    CanonicalExecutableExpressionV1::State(grounded),
                    constant_expression(CanonicalScalarValueV1::Boolean(
                        parts.handler.required_grounded,
                    )),
                )],
                required_present: vec![],
                required_absent: vec![],
                assignments,
                removals: vec![],
            }],
        });
    }
    if let Some(parts) = tick {
        for source in parts.handlers {
            let predicates = source
                .predicates
                .iter()
                .map(|predicate| match predicate {
                    CanonicalTickPredicateV1::EqualBoolean(value, expected) => {
                        Ok(CanonicalExecutablePredicateV1::Equal(
                            tick_state_ref(cst, plan, parts, *value)?,
                            constant_expression(CanonicalScalarValueV1::Boolean(*expected)),
                        ))
                    }
                    CanonicalTickPredicateV1::GreaterThan(left, right) => {
                        Ok(CanonicalExecutablePredicateV1::GreaterThan(
                            tick_executable_expression(cst, plan, parts, left)?,
                            tick_executable_expression(cst, plan, parts, right)?,
                        ))
                    }
                    CanonicalTickPredicateV1::LessThanOrEqual(left, right) => {
                        Ok(CanonicalExecutablePredicateV1::LessThanOrEqual(
                            tick_executable_expression(cst, plan, parts, left)?,
                            tick_executable_expression(cst, plan, parts, right)?,
                        ))
                    }
                })
                .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?;
            let assignments = source
                .assignments
                .iter()
                .map(|assignment| {
                    let value = match &assignment.value {
                        CanonicalTickAssignmentValueV1::Number(expression) => {
                            tick_executable_expression(cst, plan, parts, expression)?
                        }
                        CanonicalTickAssignmentValueV1::Boolean(value) => {
                            constant_expression(CanonicalScalarValueV1::Boolean(*value))
                        }
                    };
                    Ok(CanonicalExecutableAssignmentV1 {
                        target: tick_assignment_target(cst, plan, parts, assignment.target)?,
                        value,
                    })
                })
                .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?;
            handlers.push(CanonicalExecutableHandlerV1 {
                id: handler_id(&source.producer)?,
                designation: source.designation.clone(),
                trigger: CanonicalHandlerTriggerV1::FixedTickRoot,
                argument_count: 1,
                rules: vec![CanonicalExecutableRuleV1 {
                    predicates,
                    required_present: vec![],
                    required_absent: vec![],
                    assignments,
                    removals: vec![],
                }],
            });
        }
    }
    let derives = resolved_boolean_derives(cst, plan)?;
    for derive in &derives {
        let predicates = canonical_scalar_executable_predicates(
            &derive.law.predicates,
            &derive.state,
            &derive.parameters,
            derive.law.origin,
        )?;
        handlers.push(CanonicalExecutableHandlerV1 {
            id: derive.state.assertion,
            designation: derive.derive.designation.clone(),
            trigger: CanonicalHandlerTriggerV1::FixedTickDerived,
            argument_count: 0,
            rules: vec![
                CanonicalExecutableRuleV1 {
                    predicates,
                    required_present: vec![],
                    required_absent: vec![],
                    assignments: vec![CanonicalExecutableAssignmentV1 {
                        target: derive.state.clone(),
                        value: constant_expression(CanonicalScalarValueV1::Boolean(derive.value)),
                    }],
                    removals: vec![],
                },
                CanonicalExecutableRuleV1 {
                    predicates: vec![],
                    required_present: vec![],
                    required_absent: vec![],
                    assignments: vec![CanonicalExecutableAssignmentV1 {
                        target: derive.state.clone(),
                        value: constant_expression(CanonicalScalarValueV1::Boolean(!derive.value)),
                    }],
                    removals: vec![],
                },
            ],
        });
    }
    for parts in scalar {
        let current = state_ref_for_origin(
            cst,
            plan,
            parts.initial_origin,
            parts.handler.field.as_deref(),
        )?;
        let (parameters, mut entities) = resolve_parameter_states(
            cst,
            plan,
            &parts.handler.parameter_sources,
            parts.handler.origin,
        )?;
        entities.insert(parts.handler.subject.clone(), current.subject.clone());
        let mut predicates = canonical_scalar_executable_predicates(
            &parts.handler.predicates,
            &current,
            &parameters,
            parts.handler.origin,
        )?;
        predicates.extend(
            parts
                .handler
                .boolean_conditions
                .iter()
                .map(|condition| {
                    let (state, expected) =
                        derived_condition_state_ref(cst, condition, &entities, &derives)?;
                    Ok(CanonicalExecutablePredicateV1::Equal(
                        CanonicalExecutableExpressionV1::State(state),
                        constant_expression(CanonicalScalarValueV1::Boolean(expected)),
                    ))
                })
                .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?,
        );
        handlers.push(CanonicalExecutableHandlerV1 {
            id: handler_id(&parts.handler.producer)?,
            designation: parts.handler.designation.clone(),
            trigger: if keyboard
                .iter()
                .any(|binding| binding.handler_designation == parts.handler.designation)
                || (keyboard.is_empty()
                    && parameters.is_empty()
                    && parts.handler.boolean_conditions.is_empty())
            {
                CanonicalHandlerTriggerV1::External
            } else {
                CanonicalHandlerTriggerV1::FixedTick
            },
            argument_count: 0,
            rules: vec![CanonicalExecutableRuleV1 {
                predicates,
                required_present: vec![],
                required_absent: vec![],
                assignments: vec![CanonicalExecutableAssignmentV1 {
                    target: current.clone(),
                    value: canonical_scalar_executable_expression(
                        &parts.handler.result,
                        &current,
                        &parameters,
                        parts.handler.origin,
                    )?,
                }],
                removals: vec![],
            }],
        });
    }
    for source in cst.items.iter().filter_map(|item| match &item.kind {
        CstKind::GeneralHandler(handler) => Some(handler),
        _ => None,
    }) {
        if relational_handler_origins(cst).contains(&source.origin) {
            continue;
        }
        let current_source = source
            .assignments
            .first()
            .map(|assignment| &assignment.target)
            .or_else(|| source.parameter_sources.first())
            .or_else(|| source.required_sources.first())
            .ok_or(CanonicalSourceErrorV1::InvalidGeneralHandler {
                origin: source.origin,
            })?
            .clone();
        let current = general_parameter_state_ref(cst, plan, &current_source, source.origin)?;
        let initial_entities = BTreeMap::from([(source.subject.clone(), current.subject.clone())]);
        let solutions = resolve_general_parameter_states(
            cst,
            plan,
            &source.parameter_sources,
            &source.subject,
            initial_entities,
            source.origin,
        )?;
        let scalar_bindings = source
            .scalar_bindings
            .iter()
            .map(|binding| (binding.parameter.clone(), binding.value.clone()))
            .collect::<BTreeMap<_, _>>();
        let arguments = source
            .arguments
            .iter()
            .map(|argument| (argument.designation.clone(), argument.ordinal))
            .collect::<BTreeMap<_, _>>();
        if let Some(binding) = source.scalar_bindings.first() {
            validate_clamp_derivation(cst, binding.origin)?;
        }
        let mut rules = Vec::with_capacity(solutions.len());
        for solution in solutions {
            let parameters = solution.parameters;
            let entities = solution.entities;
            let mut required_present = parameters.values().cloned().collect::<Vec<_>>();
            required_present.extend(
                source
                    .required_sources
                    .iter()
                    .map(|required| {
                        general_target_state_ref(cst, plan, required, &entities, source.origin)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
            required_present.sort();
            required_present.dedup();
            let mut optional_insertions = Vec::new();
            let mut many_insertions = Vec::new();
            for insertion in &source.insertions {
                match state_relation_cardinality(cst, plan, &insertion.target, source.origin)? {
                    SourceCardinality::Maybe => optional_insertions.push(insertion),
                    SourceCardinality::Many => many_insertions.push(insertion),
                    _ => {
                        return Err(CanonicalSourceErrorV1::InvalidGeneralHandler {
                            origin: source.origin,
                        });
                    }
                }
            }
            let mut required_absent = optional_insertions
                .iter()
                .map(|insertion| {
                    general_target_state_ref(cst, plan, &insertion.target, &entities, source.origin)
                })
                .collect::<Result<Vec<_>, _>>()?;
            required_absent.sort();
            required_absent.dedup();
            let mut scalar_removals = Vec::new();
            let mut many_removals = Vec::new();
            for removal in &source.removals {
                match state_relation_cardinality(cst, plan, removal, source.origin)? {
                    SourceCardinality::Maybe => scalar_removals.push(removal),
                    SourceCardinality::Many => many_removals.push(removal),
                    _ => {
                        return Err(CanonicalSourceErrorV1::InvalidGeneralHandler {
                            origin: source.origin,
                        });
                    }
                }
            }
            let mut removals = scalar_removals
                .iter()
                .map(|removal| {
                    general_target_state_ref(cst, plan, removal, &entities, source.origin)
                })
                .collect::<Result<Vec<_>, _>>()?;
            removals.sort();
            removals.dedup();
            for binding in &source.scalar_bindings {
                canonical_general_executable_expression(
                    &binding.value,
                    &current,
                    &parameters,
                    &arguments,
                    &scalar_bindings,
                    binding.origin,
                )?;
            }
            let mut predicates = canonical_general_executable_predicates(
                &source.predicates,
                &current,
                &parameters,
                &arguments,
                &scalar_bindings,
                source.origin,
            )?;
            predicates.extend(solution.selector_equalities.into_iter().map(
                |(selector, expected)| {
                    CanonicalExecutablePredicateV1::Equal(
                        CanonicalExecutableExpressionV1::State(selector),
                        constant_expression(CanonicalScalarValueV1::Symbol(expected)),
                    )
                },
            ));
            predicates.extend(
                source
                    .boolean_conditions
                    .iter()
                    .map(|condition| {
                        let (state, expected) =
                            derived_condition_state_ref(cst, condition, &entities, &derives)?;
                        Ok(CanonicalExecutablePredicateV1::Equal(
                            CanonicalExecutableExpressionV1::State(state),
                            constant_expression(CanonicalScalarValueV1::Boolean(expected)),
                        ))
                    })
                    .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?,
            );
            for membership in &source.membership_sources {
                let argument = arguments.get(&membership.parameter).copied().ok_or(
                    CanonicalSourceErrorV1::InvalidGeneralHandler {
                        origin: source.origin,
                    },
                )?;
                let state = many_state_ref(cst, plan, membership, &entities, source.origin)?;
                required_present.push(state.clone());
                predicates.push(CanonicalExecutablePredicateV1::Contains(
                    CanonicalExecutableExpressionV1::State(state),
                    CanonicalExecutableExpressionV1::Argument(argument),
                ));
            }
            required_present.sort();
            required_present.dedup();
            let mut assignments = source
                .assignments
                .iter()
                .map(|assignment| {
                    Ok(CanonicalExecutableAssignmentV1 {
                        target: general_target_state_ref(
                            cst,
                            plan,
                            &assignment.target,
                            &entities,
                            source.origin,
                        )?,
                        value: canonical_general_executable_expression(
                            &assignment.value,
                            &current,
                            &parameters,
                            &arguments,
                            &scalar_bindings,
                            source.origin,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?;
            assignments.extend(
                optional_insertions
                    .iter()
                    .map(|assignment| {
                        Ok(CanonicalExecutableAssignmentV1 {
                            target: general_target_state_ref(
                                cst,
                                plan,
                                &assignment.target,
                                &entities,
                                source.origin,
                            )?,
                            value: canonical_general_executable_expression(
                                &assignment.value,
                                &current,
                                &parameters,
                                &arguments,
                                &scalar_bindings,
                                source.origin,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?,
            );
            for insertion in many_insertions {
                let value = canonical_general_executable_expression(
                    &insertion.value,
                    &current,
                    &parameters,
                    &arguments,
                    &scalar_bindings,
                    source.origin,
                )?;
                let state = many_state_ref(cst, plan, &insertion.target, &entities, source.origin)?;
                if assignments
                    .iter()
                    .any(|assignment| assignment.target == state)
                {
                    return Err(CanonicalSourceErrorV1::InvalidGeneralHandler {
                        origin: source.origin,
                    });
                }
                required_present.push(state.clone());
                assignments.push(CanonicalExecutableAssignmentV1 {
                    target: state.clone(),
                    value: CanonicalExecutableExpressionV1::Insert(
                        Box::new(CanonicalExecutableExpressionV1::State(state)),
                        Box::new(value),
                    ),
                });
            }
            for removal in many_removals {
                let argument = arguments.get(&removal.parameter).copied().ok_or(
                    CanonicalSourceErrorV1::InvalidGeneralHandler {
                        origin: source.origin,
                    },
                )?;
                let state = many_state_ref(cst, plan, removal, &entities, source.origin)?;
                if assignments
                    .iter()
                    .any(|assignment| assignment.target == state)
                {
                    return Err(CanonicalSourceErrorV1::InvalidGeneralHandler {
                        origin: source.origin,
                    });
                }
                required_present.push(state.clone());
                assignments.push(CanonicalExecutableAssignmentV1 {
                    target: state.clone(),
                    value: CanonicalExecutableExpressionV1::Remove(
                        Box::new(CanonicalExecutableExpressionV1::State(state)),
                        Box::new(CanonicalExecutableExpressionV1::Argument(argument)),
                    ),
                });
            }
            required_present.sort();
            required_present.dedup();
            rules.push(CanonicalExecutableRuleV1 {
                predicates,
                required_present,
                required_absent,
                assignments,
                removals,
            });
        }
        handlers.push(CanonicalExecutableHandlerV1 {
            id: handler_id(&source.producer)?,
            designation: source.designation.clone(),
            trigger: if !source.arguments.is_empty()
                || keyboard
                    .iter()
                    .any(|binding| binding.handler_designation == source.designation)
                || (source.predicates.is_empty() && source.boolean_conditions.is_empty())
            {
                CanonicalHandlerTriggerV1::External
            } else {
                CanonicalHandlerTriggerV1::FixedTick
            },
            argument_count: u16::try_from(source.arguments.len()).map_err(|_| {
                CanonicalSourceErrorV1::InvalidGeneralHandler {
                    origin: source.origin,
                }
            })?,
            rules,
        });
    }
    let relational = relational_handler_origins(cst);
    handlers.extend(
        cst.items
            .iter()
            .filter_map(|item| match &item.kind {
                CstKind::GeneralHandler(handler) if relational.contains(&handler.origin) => {
                    Some(relational_checked_handler(cst, plan, handler))
                }
                _ => None,
            })
            .collect::<Result<Vec<_>, _>>()?,
    );
    handlers.sort_by_key(|handler| handler.id);
    Ok(handlers)
}

fn source_keyboard_bindings(
    cst: &CanonicalSourceCstV1,
) -> Result<Vec<CanonicalKeyboardBindingV1>, CanonicalSourceErrorV1> {
    let mut seen = BTreeSet::new();
    let mut bindings = Vec::new();
    for source in cst.items.iter().filter_map(|item| match &item.kind {
        CstKind::KeyboardBinding(binding) => Some(binding),
        _ => None,
    }) {
        if !seen.insert((source.code.clone(), source.phase)) {
            return Err(CanonicalSourceErrorV1::DuplicateKeyboardBinding {
                code: source.code.clone(),
                phase: source.phase,
            });
        }
        bindings.push(CanonicalKeyboardBindingV1 {
            code: source.code.clone(),
            phase: source.phase,
            handler_designation: source.handler_designation.clone(),
        });
    }
    bindings.sort();
    Ok(bindings)
}

fn source_scalar_input_bindings(
    cst: &CanonicalSourceCstV1,
) -> Result<Vec<CanonicalScalarInputBindingV1>, CanonicalSourceErrorV1> {
    let mut seen = BTreeSet::new();
    let mut bindings = Vec::new();
    for source in cst.items.iter().filter_map(|item| match &item.kind {
        CstKind::ScalarInputBinding(binding) => Some(binding),
        _ => None,
    }) {
        if !seen.insert(source.channel.clone()) {
            return Err(CanonicalSourceErrorV1::DuplicateScalarInputBinding {
                channel: source.channel.clone(),
            });
        }
        bindings.push(CanonicalScalarInputBindingV1 {
            channel: source.channel.clone(),
            handler_designation: source.handler_designation.clone(),
        });
    }
    bindings.sort();
    Ok(bindings)
}

fn validate_keyboard_handler_targets(
    cst: &CanonicalSourceCstV1,
    keyboard: &[CanonicalKeyboardBindingV1],
    handlers: &[CanonicalExecutableHandlerV1],
) -> Result<(), CanonicalSourceErrorV1> {
    for binding in keyboard {
        let matching = handlers
            .iter()
            .filter(|handler| handler.designation == binding.handler_designation)
            .collect::<Vec<_>>();
        let [handler] = matching.as_slice() else {
            return Err(if matching.is_empty() {
                CanonicalSourceErrorV1::MissingKeyboardHandler {
                    designation: binding.handler_designation.clone(),
                }
            } else {
                CanonicalSourceErrorV1::AmbiguousKeyboardHandler {
                    designation: binding.handler_designation.clone(),
                }
            });
        };
        if handler.trigger != CanonicalHandlerTriggerV1::External || handler.argument_count != 0 {
            let origin = cst
                .items
                .iter()
                .find_map(|item| match &item.kind {
                    CstKind::KeyboardBinding(source)
                        if source.code == binding.code
                            && source.phase == binding.phase
                            && source.handler_designation == binding.handler_designation =>
                    {
                        Some(source.origin)
                    }
                    _ => None,
                })
                .expect("one checked keyboard binding retains its source origin");
            return Err(CanonicalSourceErrorV1::InvalidKeyboardBinding { origin });
        }
    }
    Ok(())
}

fn validate_scalar_input_handler_targets(
    cst: &CanonicalSourceCstV1,
    scalar_inputs: &[CanonicalScalarInputBindingV1],
    handlers: &[CanonicalExecutableHandlerV1],
) -> Result<(), CanonicalSourceErrorV1> {
    for binding in scalar_inputs {
        let matching = handlers
            .iter()
            .filter(|handler| handler.designation == binding.handler_designation)
            .collect::<Vec<_>>();
        let [handler] = matching.as_slice() else {
            return Err(if matching.is_empty() {
                CanonicalSourceErrorV1::MissingScalarInputHandler {
                    designation: binding.handler_designation.clone(),
                }
            } else {
                CanonicalSourceErrorV1::AmbiguousScalarInputHandler {
                    designation: binding.handler_designation.clone(),
                }
            });
        };
        if handler.trigger != CanonicalHandlerTriggerV1::External || handler.argument_count != 1 {
            let origin = cst
                .items
                .iter()
                .find_map(|item| match &item.kind {
                    CstKind::ScalarInputBinding(source)
                        if source.channel == binding.channel
                            && source.handler_designation == binding.handler_designation =>
                    {
                        Some(source.origin)
                    }
                    _ => None,
                })
                .expect("one checked scalar input binding retains its source origin");
            return Err(CanonicalSourceErrorV1::InvalidScalarInputBinding { origin });
        }
    }
    Ok(())
}

struct CheckedCanonicalSourceExecutionV1 {
    state_cells: Vec<CanonicalStateCellV1>,
    executable_handlers: Vec<CanonicalExecutableHandlerV1>,
    keyboard_bindings: Vec<CanonicalKeyboardBindingV1>,
    scalar_input_bindings: Vec<CanonicalScalarInputBindingV1>,
    input_handler: Option<CanonicalInputHandlerV1>,
    jump_handler: Option<CanonicalJumpHandlerV1>,
    scalar_handlers: Vec<CanonicalScalarHandlerV1>,
    tick_program: Option<CanonicalTickProgramV1>,
}

fn checked_canonical_source_execution_v1(
    cst: &CanonicalSourceCstV1,
    plan: &CanonicalSourceAllocationPlanV1,
    input_parts: Option<(&InputHandlerCst, &VectorAssertionCst)>,
    jump_parts: Option<JumpHandlerParts<'_>>,
    scalar_parts: &[ScalarHandlerParts<'_>],
    tick_parts: Option<TickProgramParts<'_>>,
) -> Result<CheckedCanonicalSourceExecutionV1, CanonicalSourceErrorV1> {
    let input_handler = input_parts
        .as_ref()
        .map(|(handler, assertion)| CanonicalInputHandlerV1 {
            artifact: cst.artifact,
            handler_origin: handler.origin,
            initial_assertion_origin: assertion.origin,
            initial_x: assertion.x,
            initial_z: assertion.z,
            result_x: handler.result_x,
            result_z: handler.result_z,
        });
    let jump_handler = jump_parts.map(|parts| CanonicalJumpHandlerV1 {
        artifact: cst.artifact,
        handler_origin: parts.handler.origin,
        velocity_assertion_origin: parts.velocity.origin,
        grounded_assertion_origin: parts.grounded.origin,
        jump_speed_assertion_origin: parts.jump_speed.origin,
        initial_velocity: [parts.velocity.x, parts.velocity.y, parts.velocity.z],
        initial_grounded: parts.grounded.value,
        jump_speed: parts.jump_speed.value,
        required_grounded: parts.handler.required_grounded,
        result_velocity: parts.handler.result_velocity,
        result_grounded: parts.handler.result_grounded,
    });
    let scalar_handlers = scalar_parts
        .iter()
        .map(|parts| CanonicalScalarHandlerV1 {
            artifact: cst.artifact,
            handler_origin: parts.handler.origin,
            initial_assertion_origin: parts.initial_origin,
            include_origin: parts.handler.include.origin,
            initial_value: parts.initial_value.clone(),
            parameters: parts.handler.parameters.clone(),
            predicates: parts.handler.predicates.clone(),
            result: parts.handler.result.clone(),
        })
        .collect();
    let keyboard_bindings = source_keyboard_bindings(cst)?;
    let scalar_input_bindings = source_scalar_input_bindings(cst)?;
    let state_cells = checked_source_state_cells(cst, plan)?;
    let executable_handlers = checked_executable_handlers(
        cst,
        plan,
        input_parts,
        jump_parts,
        scalar_parts,
        tick_parts,
        &keyboard_bindings,
    )?;
    validate_keyboard_handler_targets(cst, &keyboard_bindings, &executable_handlers)?;
    validate_scalar_input_handler_targets(cst, &scalar_input_bindings, &executable_handlers)?;
    let tick_program = tick_parts.map(|parts| CanonicalTickProgramV1 {
        artifact: cst.artifact,
        initial_position: [parts.position.x, parts.position.y, parts.position.z],
        initial_velocity: [parts.velocity.x, parts.velocity.y, parts.velocity.z],
        initial_intent: [parts.intent.x, parts.intent.y, parts.intent.z],
        initial_grounded: parts.grounded.value,
        gravity: parts.gravity.value,
        move_speed: parts.move_speed.value,
        floor_height: parts.floor_height.value,
        minimum_x: parts.minimum_x.value,
        maximum_x: parts.maximum_x.value,
        minimum_z: parts.minimum_z.value,
        maximum_z: parts.maximum_z.value,
        assertion_origins: vec![
            parts.position.origin,
            parts.velocity.origin,
            parts.intent.origin,
            parts.grounded.origin,
            parts.gravity.origin,
            parts.move_speed.origin,
            parts.floor_height.origin,
            parts.minimum_x.origin,
            parts.maximum_x.origin,
            parts.minimum_z.origin,
            parts.maximum_z.origin,
        ],
        clamp_law_origins: parts.laws.map(|law| law.origin),
        derive_origins: parts.derives.map(|derive| derive.origin),
        rules: parts
            .handlers
            .map(|handler| CanonicalTickRuleV1 {
                handler_origin: handler.origin,
                include_origins: handler
                    .includes
                    .iter()
                    .map(|include| include.origin)
                    .collect(),
                predicates: handler.predicates.clone(),
                assignments: handler.assignments.clone(),
            })
            .into(),
    });
    Ok(CheckedCanonicalSourceExecutionV1 {
        state_cells,
        executable_handlers,
        keyboard_bindings,
        scalar_input_bindings,
        input_handler,
        jump_handler,
        scalar_handlers,
        tick_program,
    })
}

/// Lower the supported declaration slice, then pass it through the existing
/// canonical encoder, decoder, and package checker.
pub fn elaborate_canonical_source_package_v1(
    cst: &CanonicalSourceCstV1,
    context: CanonicalSourceContextV1,
    plan: &CanonicalSourceAllocationPlanV1,
) -> Result<CanonicalSourcePackageSliceV1, CanonicalSourceErrorV1> {
    if plan.artifact != cst.artifact {
        return Err(CanonicalSourceErrorV1::AllocationArtifactMismatch);
    }
    let scope = TermScope {
        universe: context.universe,
        semantics: context.semantics,
    };
    let mut formations = Vec::new();
    let mut schemas = Vec::new();
    let mut capabilities = Vec::new();
    let mut operators = Vec::new();
    let mut emissions = Vec::new();
    let mut unsupported = Vec::new();
    let mut named_formations = BTreeMap::new();
    let mut named_capabilities = BTreeMap::new();
    for item in &cst.items {
        match &item.kind {
            CstKind::Referent { designation } => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Referent, designation);
                let slot = head_slot(CanonicalSourceProductionV1::Referent);
                named_formations.insert(designation.clone(), formation_id(plan, &producer, &slot)?);
            }
            CstKind::Capability { designation } => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Capability, designation);
                let slot = head_slot(CanonicalSourceProductionV1::Capability);
                named_formations.insert(designation.clone(), formation_id(plan, &producer, &slot)?);
                named_capabilities
                    .insert(designation.clone(), capability_id(plan, &producer, &slot)?);
            }
            _ => {}
        }
    }
    let input_parts = input_handler_parts(cst)?;
    let jump_parts = jump_handler_parts(cst)?;
    let scalar_parts = scalar_handler_parts(cst)?;
    let tick_parts = tick_program_parts(cst)?;
    for item in &cst.items {
        match &item.kind {
            CstKind::Referent { designation } => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Referent, designation);
                let slot = head_slot(CanonicalSourceProductionV1::Referent);
                let id = formation_id(plan, &producer, &slot)?;
                formations.push(source_formation(
                    scope,
                    id,
                    cst.source_slice(item.origin).expect("owned origin"),
                    item.origin,
                    "referent",
                )?);
                emissions.push(emission(plan, producer, slot, item.origin));
            }
            CstKind::Membership(membership) => {
                if membership
                    .domains
                    .iter()
                    .any(|domain| !named_formations.contains_key(domain))
                {
                    return Err(CanonicalSourceErrorV1::MissingExecutableBinding {
                        origin: item.origin,
                    });
                }
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Referent, &membership.subject);
                let slot = head_slot(CanonicalSourceProductionV1::Referent);
                let id = formation_id(plan, &producer, &slot)?;
                formations.push(source_formation(
                    scope,
                    id,
                    cst.source_slice(item.origin)
                        .expect("owned membership origin"),
                    item.origin,
                    "referent",
                )?);
                emissions.push(emission(plan, producer, slot, item.origin));
                emissions.extend(membership.emissions.clone());
            }
            CstKind::Capability { designation } => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Capability, designation);
                let slot = head_slot(CanonicalSourceProductionV1::Capability);
                let formation = formation_id(plan, &producer, &slot)?;
                let capability = capability_id(plan, &producer, &slot)?;
                formations.push(source_formation(
                    scope,
                    formation,
                    cst.source_slice(item.origin).expect("owned origin"),
                    item.origin,
                    "capability",
                )?);
                capabilities.push(CapabilityDeclarationPreimageV2 {
                    id: capability,
                    formation,
                    direct_dependencies: vec![],
                });
                emissions.push(emission(plan, producer, slot, item.origin));
            }
            CstKind::Shape {
                designation,
                fields,
            } => {
                let producer = semantic_producer(CanonicalSourceProductionV1::Shape, designation);
                let slot = head_slot(CanonicalSourceProductionV1::Shape);
                let id = formation_id(plan, &producer, &slot)?;
                formations.push(source_formation(
                    scope,
                    id,
                    cst.source_slice(item.origin).expect("owned origin"),
                    item.origin,
                    "shape",
                )?);
                emissions.push(emission(plan, producer.clone(), slot, item.origin));
                for field in fields {
                    let slot = child_slot(CanonicalSourceProductionV1::ShapeField, &field.name);
                    let id = formation_id(plan, &producer, &slot)?;
                    formations.push(FormationJudgmentPreimageV2 {
                        id,
                        context: vec![origin_term(scope, field.origin)?],
                        term: source_term(
                            scope,
                            cst.source_slice(field.origin).expect("owned origin"),
                        )?,
                        target: target(scope, b"clause/source-shape-field-type-v1", &field.domain)?,
                        direct_dependencies: vec![],
                    });
                    emissions.push(emission(plan, producer.clone(), slot, field.origin));
                }
            }
            CstKind::Relation(relation) => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Relation, &relation.designation);
                let slot = head_slot(CanonicalSourceProductionV1::Relation);
                let formation = formation_id(plan, &producer, &slot)?;
                let schema = schema_id(plan, &producer, &slot)?;
                let operator = operator_id(plan, &producer, &slot)?;
                formations.push(source_formation(
                    scope,
                    formation,
                    cst.source_slice(item.origin).expect("owned origin"),
                    item.origin,
                    "relation",
                )?);
                emissions.push(emission(plan, producer.clone(), slot, item.origin));
                let mut roles = Vec::new();
                let mut role_ids = BTreeMap::new();
                for role in &relation.roles {
                    let role_slot =
                        child_slot(CanonicalSourceProductionV1::RelationRole, &role.name);
                    let role_ref = role_id(plan, &producer, &role_slot)?;
                    if role_ref.schema != schema {
                        return Err(CanonicalSourceErrorV1::MissingAllocation {
                            slot: role_slot,
                            domain: AllocationDomain::Role.label(),
                        });
                    }
                    role_ids.insert(role.name.clone(), role_ref.role);
                    roles.push(RoleDeclarationPreimageV2 {
                        id: role_ref.role,
                        target: target(scope, b"clause/source-role-domain-v1", &role.domain)?,
                        cardinality: exactly_one(),
                        direct_dependencies: vec![],
                    });
                    emissions.push(emission(plan, producer.clone(), role_slot, role.origin));
                }
                roles.sort_by_key(|role| role.id);
                let result_domain = target(
                    scope,
                    b"clause/source-relation-result-v1",
                    &relation.designation,
                )?;
                schemas.push(RelationSchemaPreimageV2 {
                    id: schema,
                    roles,
                    constraints: vec![],
                    result_domain: result_domain.clone(),
                    direct_dependencies: vec![],
                });
                let mut modes = Vec::new();
                for mode in &relation.modes {
                    let mode_slot =
                        child_slot(CanonicalSourceProductionV1::RelationMode, &mode.canonical);
                    let mode_ref = mode_id(plan, &producer, &mode_slot)?;
                    if mode_ref.operator != operator {
                        return Err(CanonicalSourceErrorV1::MissingAllocation {
                            slot: mode_slot,
                            domain: AllocationDomain::Mode.label(),
                        });
                    }
                    let mut known_roles = mode
                        .known
                        .iter()
                        .map(|name| role_ids[name])
                        .collect::<Vec<_>>();
                    let mut produced_roles = mode
                        .produced
                        .iter()
                        .map(|name| role_ids[name])
                        .collect::<Vec<_>>();
                    known_roles.sort();
                    produced_roles.sort();
                    let productivity = match &mode.reactive_obligation {
                        Some(designation) => ProductivityContractV2 {
                            kind: ProductivityKindV2::Reactive,
                            obligations: vec![*named_formations.get(designation).ok_or_else(
                                || CanonicalSourceErrorV1::UnknownModeFormation {
                                    designation: designation.clone(),
                                },
                            )?],
                        },
                        None => ProductivityContractV2 {
                            kind: ProductivityKindV2::Partial,
                            obligations: vec![],
                        },
                    };
                    let (effect_intents, capability_requirements) = match &mode.effect {
                        Some(effect) => {
                            let capability = *named_capabilities
                                .get(&effect.capability)
                                .ok_or_else(|| CanonicalSourceErrorV1::UnknownModeCapability {
                                    designation: effect.capability.clone(),
                                })?;
                            (
                                vec![EffectIntentContractPreimageV2 {
                                    intent_domain: target(
                                        scope,
                                        b"clause/source-effect-intent-v1",
                                        &relation.designation,
                                    )?,
                                    action_role: role_ids[&effect.action_role],
                                    resource_role: role_ids[&effect.resource_role],
                                    payload_role: role_ids[&effect.payload_role],
                                    required_capability: capability,
                                }],
                                vec![capability],
                            )
                        }
                        None => (vec![], vec![]),
                    };
                    modes.push(ModePreimageV2 {
                        id: mode_ref.mode,
                        schema,
                        known_roles,
                        produced_roles,
                        static_basis: StaticActivationBasisPreimageV2 {
                            context_requirements: vec![],
                            constitutive_dependencies: vec![],
                        },
                        authorization_requirements: vec![],
                        dynamic_prerequisites: vec![],
                        contract: ModeContractV2 {
                            determinism: DeterminismContractV2::Deterministic,
                            result_cardinality: mode.cardinality.as_contract(),
                            result_order: ResultOrderContractV2::UnorderedFiniteSet,
                            failure_domain: None,
                            state_delta_domain: None,
                            budget_exhaustion_domain: None,
                            effect_intents,
                            formation_checks: vec![],
                            productivity,
                            scheduling_requirements: vec![],
                            resource_requirements: vec![],
                            capability_requirements,
                            continuation: if mode.continues_linearly {
                                ContinuationContractV2::Suspensible {
                                    use_policy: ContinuationUseV2::Linear,
                                    may_handoff: false,
                                    may_cancel: false,
                                }
                            } else {
                                ContinuationContractV2::TerminalOnly { may_cancel: false }
                            },
                        },
                        direct_dependencies: vec![],
                    });
                    emissions.push(emission(plan, producer.clone(), mode_slot, mode.origin));
                }
                modes.sort_by_key(|mode| mode.id);
                operators.push(OperatorPreimageV2 {
                    id: operator,
                    modes,
                    direct_dependencies: vec![],
                });
            }
            CstKind::InputHandler(handler) => {
                let head = head_slot(CanonicalSourceProductionV1::Handler);
                let head_id = formation_id(plan, &handler.producer, &head)?;
                formations.push(source_formation(
                    scope,
                    head_id,
                    cst.source_slice(handler.origin)
                        .expect("owned handler origin"),
                    handler.origin,
                    "input-handler",
                )?);
                emissions.push(emission(
                    plan,
                    handler.producer.clone(),
                    head,
                    handler.origin,
                ));

                let include = child_slot(
                    CanonicalSourceProductionV1::HandlerInclude,
                    &handler.include_local,
                );
                let include_id = formation_id(plan, &handler.producer, &include)?;
                formations.push(source_formation(
                    scope,
                    include_id,
                    cst.source_slice(handler.include_origin)
                        .expect("owned handler include origin"),
                    handler.include_origin,
                    "handler-include",
                )?);
                emissions.push(emission(
                    plan,
                    handler.producer.clone(),
                    include,
                    handler.include_origin,
                ));
            }
            CstKind::JumpHandler(handler) => {
                let head = head_slot(CanonicalSourceProductionV1::Handler);
                let head_id = formation_id(plan, &handler.producer, &head)?;
                formations.push(source_formation(
                    scope,
                    head_id,
                    cst.source_slice(handler.origin)
                        .expect("owned jump handler origin"),
                    handler.origin,
                    "jump-handler",
                )?);
                emissions.push(emission(
                    plan,
                    handler.producer.clone(),
                    head,
                    handler.origin,
                ));
                for include in &handler.includes {
                    let slot =
                        child_slot(CanonicalSourceProductionV1::HandlerInclude, &include.local);
                    let id = formation_id(plan, &handler.producer, &slot)?;
                    formations.push(source_formation(
                        scope,
                        id,
                        cst.source_slice(include.origin)
                            .expect("owned jump include origin"),
                        include.origin,
                        "handler-include",
                    )?);
                    emissions.push(emission(
                        plan,
                        handler.producer.clone(),
                        slot,
                        include.origin,
                    ));
                }
            }
            CstKind::ScalarHandler(handler) => {
                let head = head_slot(CanonicalSourceProductionV1::Handler);
                let head_id = formation_id(plan, &handler.producer, &head)?;
                formations.push(source_formation(
                    scope,
                    head_id,
                    cst.source_slice(handler.origin)
                        .expect("owned scalar handler origin"),
                    handler.origin,
                    "scalar-handler",
                )?);
                emissions.push(emission(
                    plan,
                    handler.producer.clone(),
                    head,
                    handler.origin,
                ));
                let include = child_slot(
                    CanonicalSourceProductionV1::HandlerInclude,
                    &handler.include.local,
                );
                let include_id = formation_id(plan, &handler.producer, &include)?;
                formations.push(source_formation(
                    scope,
                    include_id,
                    cst.source_slice(handler.include.origin)
                        .expect("owned scalar handler include origin"),
                    handler.include.origin,
                    "handler-include",
                )?);
                emissions.push(emission(
                    plan,
                    handler.producer.clone(),
                    include,
                    handler.include.origin,
                ));
            }
            CstKind::GeneralHandler(handler) => {
                let head = head_slot(CanonicalSourceProductionV1::Handler);
                let head_id = formation_id(plan, &handler.producer, &head)?;
                formations.push(source_formation(
                    scope,
                    head_id,
                    cst.source_slice(handler.origin)
                        .expect("owned general handler origin"),
                    handler.origin,
                    "general-handler",
                )?);
                emissions.push(emission(
                    plan,
                    handler.producer.clone(),
                    head,
                    handler.origin,
                ));
                for include in &handler.includes {
                    let slot =
                        child_slot(CanonicalSourceProductionV1::HandlerInclude, &include.local);
                    let id = formation_id(plan, &handler.producer, &slot)?;
                    formations.push(source_formation(
                        scope,
                        id,
                        cst.source_slice(include.origin)
                            .expect("owned general handler include origin"),
                        include.origin,
                        "handler-include",
                    )?);
                    emissions.push(emission(
                        plan,
                        handler.producer.clone(),
                        slot,
                        include.origin,
                    ));
                }
            }
            CstKind::TickHandler(handler) => {
                let head = head_slot(CanonicalSourceProductionV1::Handler);
                let head_id = formation_id(plan, &handler.producer, &head)?;
                formations.push(source_formation(
                    scope,
                    head_id,
                    cst.source_slice(handler.origin)
                        .expect("owned tick handler origin"),
                    handler.origin,
                    "tick-handler",
                )?);
                emissions.push(emission(
                    plan,
                    handler.producer.clone(),
                    head,
                    handler.origin,
                ));
                for include in &handler.includes {
                    let slot =
                        child_slot(CanonicalSourceProductionV1::HandlerInclude, &include.local);
                    let id = formation_id(plan, &handler.producer, &slot)?;
                    formations.push(source_formation(
                        scope,
                        id,
                        cst.source_slice(include.origin)
                            .expect("owned tick include origin"),
                        include.origin,
                        "handler-include",
                    )?);
                    emissions.push(emission(
                        plan,
                        handler.producer.clone(),
                        slot,
                        include.origin,
                    ));
                }
            }
            CstKind::ClampLaw(law) => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Law, &law.designation);
                let slot = head_slot(CanonicalSourceProductionV1::Law);
                let id = formation_id(plan, &producer, &slot)?;
                formations.push(source_formation(
                    scope,
                    id,
                    cst.source_slice(law.origin)
                        .expect("owned clamp law origin"),
                    law.origin,
                    "clamp-law",
                )?);
                emissions.push(emission(plan, producer, slot, law.origin));
            }
            CstKind::ClampDerive(derive) => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Derive, &derive.designation);
                let slot = head_slot(CanonicalSourceProductionV1::Derive);
                let id = formation_id(plan, &producer, &slot)?;
                formations.push(source_formation(
                    scope,
                    id,
                    cst.source_slice(derive.origin)
                        .expect("owned clamp derive origin"),
                    derive.origin,
                    "clamp-derive",
                )?);
                emissions.push(emission(plan, producer, slot, derive.origin));
            }
            CstKind::BooleanLaw(law) => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Law, &law.designation);
                let slot = head_slot(CanonicalSourceProductionV1::Law);
                let id = formation_id(plan, &producer, &slot)?;
                formations.push(source_formation(
                    scope,
                    id,
                    cst.source_slice(law.origin)
                        .expect("owned Boolean law origin"),
                    law.origin,
                    "boolean-law",
                )?);
                emissions.push(emission(plan, producer, slot, law.origin));
            }
            CstKind::BooleanDerive(derive) => {
                let producer =
                    semantic_producer(CanonicalSourceProductionV1::Derive, &derive.designation);
                let slot = head_slot(CanonicalSourceProductionV1::Derive);
                let id = formation_id(plan, &producer, &slot)?;
                formations.push(source_formation(
                    scope,
                    id,
                    cst.source_slice(derive.origin)
                        .expect("owned Boolean derive origin"),
                    derive.origin,
                    "boolean-derive",
                )?);
                emissions.push(emission(plan, producer, slot, derive.origin));
            }
            CstKind::VectorAssertion(assertion) => {
                if input_parts
                    .as_ref()
                    .is_some_and(|(_, selected)| selected.origin == assertion.origin)
                    || jump_parts.is_some_and(|parts| parts.velocity.origin == assertion.origin)
                    || tick_parts.is_some_and(|parts| {
                        [
                            parts.position.origin,
                            parts.velocity.origin,
                            parts.intent.origin,
                        ]
                        .contains(&assertion.origin)
                    })
                    || scalar_parts.iter().any(|parts| {
                        parts.handler.parameter_sources.iter().any(|source| {
                            source.subject == assertion.subject
                                && source.relation == assertion.relation
                                && source.field.is_some()
                        })
                    })
                    || declared_state_relation(cst, &assertion.relation)
                {
                    let producer = assertion_producer(&assertion.subject, &assertion.relation);
                    let slot = head_slot(CanonicalSourceProductionV1::Assertion);
                    let id = formation_id(plan, &producer, &slot)?;
                    formations.push(source_formation(
                        scope,
                        id,
                        cst.source_slice(assertion.origin)
                            .expect("owned initial assertion origin"),
                        assertion.origin,
                        "initial-assertion",
                    )?);
                    emissions.push(emission(plan, producer, slot, assertion.origin));
                } else {
                    unsupported.push(CanonicalUnsupportedProductionV1 {
                        production: CanonicalSourceProductionV1::Assertion,
                        origin: assertion.origin,
                        emissions: vec![],
                    });
                }
            }
            CstKind::ShapeAssertion(assertion) => {
                if declared_state_relation(cst, &assertion.relation) {
                    let producer = assertion_producer(&assertion.subject, &assertion.relation);
                    let slot = head_slot(CanonicalSourceProductionV1::Assertion);
                    let id = formation_id(plan, &producer, &slot)?;
                    formations.push(source_formation(
                        scope,
                        id,
                        cst.source_slice(assertion.origin)
                            .expect("owned shaped assertion origin"),
                        assertion.origin,
                        "initial-assertion",
                    )?);
                    emissions.push(emission(plan, producer, slot, assertion.origin));
                } else {
                    unsupported.push(CanonicalUnsupportedProductionV1 {
                        production: CanonicalSourceProductionV1::Assertion,
                        origin: assertion.origin,
                        emissions: vec![],
                    });
                }
            }
            CstKind::BooleanAssertion(assertion) => {
                if jump_parts.is_some_and(|parts| parts.grounded.origin == assertion.origin)
                    || tick_parts.is_some_and(|parts| parts.grounded.origin == assertion.origin)
                    || scalar_parts.iter().any(|parts| {
                        parts.initial_origin == assertion.origin
                            || parts.handler.parameter_sources.iter().any(|source| {
                                source.subject == assertion.subject
                                    && source.relation == assertion.relation
                                    && source.field.is_none()
                            })
                    })
                    || declared_state_relation(cst, &assertion.relation)
                {
                    let producer = assertion_producer(&assertion.subject, &assertion.relation);
                    let slot = head_slot(CanonicalSourceProductionV1::Assertion);
                    let id = formation_id(plan, &producer, &slot)?;
                    formations.push(source_formation(
                        scope,
                        id,
                        cst.source_slice(assertion.origin)
                            .expect("owned grounded assertion origin"),
                        assertion.origin,
                        "initial-assertion",
                    )?);
                    emissions.push(emission(plan, producer, slot, assertion.origin));
                } else {
                    unsupported.push(CanonicalUnsupportedProductionV1 {
                        production: CanonicalSourceProductionV1::Assertion,
                        origin: assertion.origin,
                        emissions: vec![],
                    });
                }
            }
            CstKind::NumberAssertion(assertion) => {
                if jump_parts.is_some_and(|parts| parts.jump_speed.origin == assertion.origin)
                    || scalar_parts.iter().any(|parts| {
                        parts.initial_origin == assertion.origin
                            || parts.handler.parameter_sources.iter().any(|source| {
                                source.subject == assertion.subject
                                    && source.relation == assertion.relation
                                    && source.field.is_none()
                            })
                    })
                    || tick_parts.is_some_and(|parts| {
                        [
                            parts.gravity.origin,
                            parts.move_speed.origin,
                            parts.floor_height.origin,
                            parts.minimum_x.origin,
                            parts.maximum_x.origin,
                            parts.minimum_z.origin,
                            parts.maximum_z.origin,
                        ]
                        .contains(&assertion.origin)
                    })
                    || declared_state_relation(cst, &assertion.relation)
                {
                    let producer = assertion_producer(&assertion.subject, &assertion.relation);
                    let slot = head_slot(CanonicalSourceProductionV1::Assertion);
                    let id = formation_id(plan, &producer, &slot)?;
                    formations.push(source_formation(
                        scope,
                        id,
                        cst.source_slice(assertion.origin)
                            .expect("owned jump-speed assertion origin"),
                        assertion.origin,
                        "initial-assertion",
                    )?);
                    emissions.push(emission(plan, producer, slot, assertion.origin));
                } else {
                    unsupported.push(CanonicalUnsupportedProductionV1 {
                        production: CanonicalSourceProductionV1::Assertion,
                        origin: assertion.origin,
                        emissions: vec![],
                    });
                }
            }
            CstKind::SymbolAssertion(assertion) => {
                if scalar_parts.iter().any(|parts| {
                    parts.initial_origin == assertion.origin
                        || parts.handler.parameter_sources.iter().any(|source| {
                            source.subject == assertion.subject
                                && source.relation == assertion.relation
                                && source.field.is_none()
                        })
                }) || declared_state_relation(cst, &assertion.relation)
                {
                    let producer = assertion_producer(&assertion.subject, &assertion.relation);
                    let slot = head_slot(CanonicalSourceProductionV1::Assertion);
                    let id = formation_id(plan, &producer, &slot)?;
                    formations.push(source_formation(
                        scope,
                        id,
                        cst.source_slice(assertion.origin)
                            .expect("owned scalar assertion origin"),
                        assertion.origin,
                        "initial-assertion",
                    )?);
                    emissions.push(emission(plan, producer, slot, assertion.origin));
                } else {
                    unsupported.push(CanonicalUnsupportedProductionV1 {
                        production: CanonicalSourceProductionV1::Assertion,
                        origin: assertion.origin,
                        emissions: vec![],
                    });
                }
            }
            CstKind::TextAssertion(assertion) => {
                if scalar_parts.iter().any(|parts| {
                    parts.initial_origin == assertion.origin
                        || parts.handler.parameter_sources.iter().any(|source| {
                            source.subject == assertion.subject
                                && source.relation == assertion.relation
                                && source.field.is_none()
                        })
                }) || declared_state_relation(cst, &assertion.relation)
                {
                    let producer = assertion_producer(&assertion.subject, &assertion.relation);
                    let slot = head_slot(CanonicalSourceProductionV1::Assertion);
                    let id = formation_id(plan, &producer, &slot)?;
                    formations.push(source_formation(
                        scope,
                        id,
                        cst.source_slice(assertion.origin)
                            .expect("owned Text assertion origin"),
                        assertion.origin,
                        "initial-assertion",
                    )?);
                    emissions.push(emission(plan, producer, slot, assertion.origin));
                } else {
                    unsupported.push(CanonicalUnsupportedProductionV1 {
                        production: CanonicalSourceProductionV1::Assertion,
                        origin: assertion.origin,
                        emissions: vec![],
                    });
                }
            }
            CstKind::KeyboardBinding(_) | CstKind::ScalarInputBinding(_) => {}
            CstKind::Unsupported(value) => unsupported.push(value.clone()),
        }
    }
    let (checked_package, checked_execution) = std::thread::scope(|parallel| {
        let execution = parallel.spawn(|| {
            checked_canonical_source_execution_v1(
                cst,
                plan,
                input_parts,
                jump_parts,
                &scalar_parts,
                tick_parts,
            )
        });
        formations.sort_by_key(|formation| formation.id);
        schemas.sort_by_key(|schema| schema.id);
        capabilities.sort_by_key(|capability| capability.id);
        operators.sort_by_key(|operator| operator.id);
        let checked_package: Result<CheckedProcessPackage, CanonicalSourceErrorV1> = (|| {
            let snapshot = ProgramSnapshotPreimageV2 {
                constitution: ProgramConstitutionPreimageV2 {
                    semantics: context.semantics,
                    universe: context.universe,
                    formations,
                    schemas,
                    capabilities,
                    operators,
                    applications: vec![],
                },
                successor_grants: vec![],
                static_execution_grants: vec![],
                state_admission_grants: vec![],
                judgment_authority_grants: vec![],
            };
            let claimed_snapshot =
                derive_program_snapshot_id(&snapshot).map_err(CanonicalSourceErrorV1::Encode)?;
            let package = ProcessPackageV2 {
                claimed_snapshot,
                snapshot,
                initial_state_views: vec![],
                records: vec![],
            };
            let bytes = encode_process_package(&package).map_err(CanonicalSourceErrorV1::Encode)?;
            let decoded = decode_process_package(&bytes).map_err(CanonicalSourceErrorV1::Decode)?;
            let checked = check_process_package(decoded).map_err(CanonicalSourceErrorV1::Check)?;
            Ok(checked)
        })();
        let checked_execution = match execution.join() {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        };
        Ok::<_, CanonicalSourceErrorV1>((checked_package?, checked_execution?))
    })?;
    let CheckedCanonicalSourceExecutionV1 {
        state_cells,
        executable_handlers,
        keyboard_bindings,
        scalar_input_bindings,
        input_handler,
        jump_handler,
        scalar_handlers,
        tick_program,
    } = checked_execution;
    Ok(CanonicalSourcePackageSliceV1 {
        checked_package,
        emissions,
        unsupported,
        state_cells,
        executable_handlers,
        keyboard_bindings,
        scalar_input_bindings,
        input_handler,
        jump_handler,
        scalar_handlers,
        tick_program,
    })
}

impl SourceCardinality {
    const fn as_contract(self) -> CardinalityV2 {
        match self {
            Self::One => CardinalityV2 {
                minimum: 1,
                maximum: Some(1),
            },
            Self::Maybe => CardinalityV2 {
                minimum: 0,
                maximum: Some(1),
            },
            Self::Some => CardinalityV2 {
                minimum: 1,
                maximum: None,
            },
            Self::Many => CardinalityV2 {
                minimum: 0,
                maximum: None,
            },
        }
    }
}

fn source_lines(source: &str) -> Result<Vec<SourceLine<'_>>, CanonicalSourceErrorV1> {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    while start < bytes.len() {
        let newline = bytes[start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| start + offset);
        let raw_end = newline.unwrap_or(bytes.len());
        let end = raw_end
            .checked_sub(usize::from(raw_end > start && bytes[raw_end - 1] == b'\r'))
            .expect("line end remains in bounds");
        let text = &source[start..end];
        if let Some(offset) = text.bytes().position(|byte| byte == b'\t') {
            return Err(CanonicalSourceErrorV1::TabIndentation {
                offset: (start + offset) as u64,
            });
        }
        let indent = text.bytes().take_while(|byte| *byte == b' ').count();
        lines.push(SourceLine {
            text,
            start,
            end,
            indent,
        });
        start = newline.map_or(bytes.len(), |position| position + 1);
    }
    Ok(lines)
}

fn parse_item(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
) -> Result<CstItem, CanonicalSourceErrorV1> {
    let head = block[0].text;
    if let Some(designation) = head.strip_prefix("capability ") {
        require_leaf(block, artifact)?;
        return Ok(CstItem {
            origin,
            kind: CstKind::Capability {
                designation: designation_bytes(designation, origin)?,
            },
        });
    }
    if let Some(designation) = head.strip_prefix("shape ") {
        let designation = designation_bytes(designation, origin)?;
        let mut fields = Vec::new();
        for line in block
            .iter()
            .skip(1)
            .filter(|line| !line.text.trim().is_empty())
        {
            if line.indent != 2 {
                return Err(CanonicalSourceErrorV1::InvalidShapeField {
                    origin: line_origin(artifact, *line),
                });
            }
            let value = &line.text[2..];
            let Some((name, domain)) = value.split_once(": ") else {
                return Err(CanonicalSourceErrorV1::InvalidShapeField {
                    origin: line_origin(artifact, *line),
                });
            };
            if name.is_empty() || domain.is_empty() || name.contains('=') || domain.contains('=') {
                return Err(CanonicalSourceErrorV1::InvalidShapeField {
                    origin: line_origin(artifact, *line),
                });
            }
            fields.push(ShapeField {
                name: name.as_bytes().to_vec(),
                domain: domain.as_bytes().to_vec(),
                origin: line_origin(artifact, *line),
            });
        }
        if fields.is_empty() {
            return Err(CanonicalSourceErrorV1::InvalidShapeField { origin });
        }
        ensure_unique_children(
            &designation,
            fields.iter().map(|field| field.name.as_slice()),
        )?;
        return Ok(CstItem {
            origin,
            kind: CstKind::Shape {
                designation,
                fields,
            },
        });
    }
    if let Some(designation) = head.strip_prefix("relation ") {
        let designation = designation_bytes(designation, origin)?;
        return Ok(CstItem {
            origin,
            kind: CstKind::Relation(parse_relation(artifact, block, designation)?),
        });
    }
    if head.starts_with("bind keyboard ") {
        require_leaf(block, artifact)?;
        let binding = parse_keyboard_binding(head, origin)?;
        return Ok(CstItem {
            origin,
            kind: CstKind::KeyboardBinding(binding),
        });
    }
    if head.starts_with("bind scalar-input ") {
        require_leaf(block, artifact)?;
        let binding = parse_scalar_input_binding(head, origin)?;
        return Ok(CstItem {
            origin,
            kind: CstKind::ScalarInputBinding(binding),
        });
    }
    if head.starts_with("law ") {
        if let Some(law) = parse_clamp_law(block, origin) {
            return Ok(CstItem {
                origin,
                kind: CstKind::ClampLaw(law),
            });
        }
        if let Some(law) = parse_boolean_law(artifact, block, origin)? {
            return Ok(CstItem {
                origin,
                kind: CstKind::BooleanLaw(law),
            });
        }
        return Ok(unsupported_item(
            artifact,
            block,
            origin,
            CanonicalSourceProductionV1::Law,
            vec![],
        )?);
    }
    if head.starts_with("derive ") {
        require_leaf(block, artifact)?;
        if let Some(derive) = parse_clamp_derive(head, origin) {
            return Ok(CstItem {
                origin,
                kind: CstKind::ClampDerive(derive),
            });
        }
        if let Some(designation) = head.strip_prefix("derive ") {
            return Ok(CstItem {
                origin,
                kind: CstKind::BooleanDerive(BooleanDeriveCst {
                    origin,
                    designation: designation_bytes(designation, origin)?,
                }),
            });
        }
        return Ok(unsupported_item(
            artifact,
            block,
            origin,
            CanonicalSourceProductionV1::Derive,
            vec![],
        )?);
    }
    if head.starts_with("on ") {
        if let Some(handler) = parse_input_handler(artifact, block, origin)? {
            return Ok(CstItem {
                origin,
                kind: CstKind::InputHandler(handler),
            });
        }
        if let Some(handler) = parse_jump_handler(artifact, block, origin)? {
            return Ok(CstItem {
                origin,
                kind: CstKind::JumpHandler(handler),
            });
        }
        if let Some(handler) = parse_tick_handler(artifact, block, origin)? {
            return Ok(CstItem {
                origin,
                kind: CstKind::TickHandler(handler),
            });
        }
        if let Some(handler) = parse_scalar_handler(artifact, block, origin)? {
            return Ok(CstItem {
                origin,
                kind: CstKind::ScalarHandler(handler),
            });
        }
        if let Some(handler) = parse_general_handler(artifact, block, origin)? {
            return Ok(CstItem {
                origin,
                kind: CstKind::GeneralHandler(handler),
            });
        }
        let emissions = handler_include_emissions(artifact, block)?;
        return Ok(unsupported_item(
            artifact,
            block,
            origin,
            CanonicalSourceProductionV1::Handler,
            emissions,
        )?);
    }
    require_leaf(block, artifact)?;
    if !head.contains(char::is_whitespace) {
        return Ok(CstItem {
            origin,
            kind: CstKind::Referent {
                designation: designation_bytes(head, origin)?,
            },
        });
    }
    if head.contains('∈') {
        let emissions = membership_group_emissions(artifact, block[0], origin)?;
        let (subject, domains) = parse_membership_group(head, origin)?;
        return Ok(CstItem {
            origin,
            kind: CstKind::Membership(MembershipCst {
                subject,
                domains,
                emissions,
            }),
        });
    }
    if let Some(assertion) = parse_vector_assertion(head, origin)? {
        return Ok(CstItem {
            origin,
            kind: CstKind::VectorAssertion(assertion),
        });
    }
    if let Some(assertion) = parse_shape_assertion(head, origin)? {
        return Ok(CstItem {
            origin,
            kind: CstKind::ShapeAssertion(assertion),
        });
    }
    if let Some(assertion) = parse_boolean_assertion(head, origin) {
        return Ok(CstItem {
            origin,
            kind: CstKind::BooleanAssertion(assertion),
        });
    }
    if let Some(assertion) = parse_number_assertion(head, origin) {
        return Ok(CstItem {
            origin,
            kind: CstKind::NumberAssertion(assertion),
        });
    }
    if let Some(assertion) = parse_text_assertion(head, origin) {
        return Ok(CstItem {
            origin,
            kind: CstKind::TextAssertion(assertion),
        });
    }
    if let Some(assertion) = parse_symbol_assertion(head, origin) {
        return Ok(CstItem {
            origin,
            kind: CstKind::SymbolAssertion(assertion),
        });
    }
    Ok(unsupported_item(
        artifact,
        block,
        origin,
        CanonicalSourceProductionV1::Assertion,
        vec![],
    )?)
}

fn parse_keyboard_binding(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<KeyboardBindingCst, CanonicalSourceErrorV1> {
    let parts = source.split_whitespace().collect::<Vec<_>>();
    let ["bind", "keyboard", code, phase, "to", handler] = parts.as_slice() else {
        return Err(CanonicalSourceErrorV1::InvalidKeyboardBinding { origin });
    };
    if code.is_empty()
        || code.len() > 64
        || !code.as_bytes().iter().all(u8::is_ascii_graphic)
        || handler.is_empty()
        || !handler
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-'))
    {
        return Err(CanonicalSourceErrorV1::InvalidKeyboardBinding { origin });
    }
    let phase = match *phase {
        "down" => CanonicalKeyPhaseV1::Down,
        "up" => CanonicalKeyPhaseV1::Up,
        _ => return Err(CanonicalSourceErrorV1::InvalidKeyboardBinding { origin }),
    };
    Ok(KeyboardBindingCst {
        origin,
        code: code.as_bytes().to_vec(),
        phase,
        handler_designation: handler.as_bytes().to_vec(),
    })
}

fn parse_scalar_input_binding(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<ScalarInputBindingCst, CanonicalSourceErrorV1> {
    let parts = source.split_whitespace().collect::<Vec<_>>();
    let ["bind", "scalar-input", channel, "to", handler] = parts.as_slice() else {
        return Err(CanonicalSourceErrorV1::InvalidScalarInputBinding { origin });
    };
    let valid_designation = |value: &str| {
        !value.is_empty()
            && value.len() <= 64
            && value
                .as_bytes()
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-'))
    };
    if !valid_designation(channel) || !valid_designation(handler) {
        return Err(CanonicalSourceErrorV1::InvalidScalarInputBinding { origin });
    }
    Ok(ScalarInputBindingCst {
        origin,
        channel: channel.as_bytes().to_vec(),
        handler_designation: handler.as_bytes().to_vec(),
    })
}

fn parse_input_handler(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
) -> Result<Option<InputHandlerCst>, CanonicalSourceErrorV1> {
    let Some(header) = block[0].text.strip_prefix("on input ") else {
        return Ok(None);
    };
    let Some((subject, header_vector)) = split_vector_subject(header) else {
        return Err(CanonicalSourceErrorV1::InvalidInputHandler { origin });
    };
    if !subject.starts_with('?') {
        return Err(CanonicalSourceErrorV1::InvalidInputHandler { origin });
    }
    let header_components = parse_vec3_components(header_vector)
        .ok_or(CanonicalSourceErrorV1::InvalidInputHandler { origin })?;
    if !header_components[0].starts_with('?')
        || header_components[1] != "0.0"
        || !header_components[2].starts_with('?')
        || header_components[0] == header_components[2]
    {
        return Err(CanonicalSourceErrorV1::InvalidInputHandler { origin });
    }
    let parameters = [header_components[0], header_components[2]];

    let mut section = "";
    let mut when_line = None;
    let mut withdraw_line = None;
    let mut include = None;
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let trimmed = line.text.trim();
        if line.indent == 2 {
            if trimmed == "admit" {
                return Err(CanonicalSourceErrorV1::NonCanonicalKeyword {
                    origin: line_origin(artifact, *line),
                    keyword: b"admit".to_vec(),
                });
            }
            section = trimmed;
            continue;
        }
        if line.indent != 4 {
            return Err(CanonicalSourceErrorV1::InvalidInputHandler {
                origin: line_origin(artifact, *line),
            });
        }
        match section {
            "when" if when_line.replace(trimmed).is_none() => {}
            "withdraw" if withdraw_line.replace(trimmed).is_none() => {}
            "include"
                if include
                    .replace((trimmed, line_origin(artifact, *line)))
                    .is_none() => {}
            _ => {
                return Err(CanonicalSourceErrorV1::InvalidInputHandler {
                    origin: line_origin(artifact, *line),
                });
            }
        }
    }
    let when_line = when_line.ok_or(CanonicalSourceErrorV1::InvalidInputHandler { origin })?;
    if withdraw_line != Some(when_line) {
        return Err(CanonicalSourceErrorV1::InvalidInputHandler { origin });
    }
    let when_parts = when_line.split_whitespace().collect::<Vec<_>>();
    if when_parts.len() < 3
        || when_parts[0] != subject
        || !when_parts
            .last()
            .is_some_and(|value| value.starts_with('?'))
    {
        return Err(CanonicalSourceErrorV1::InvalidInputHandler { origin });
    }
    let relation = when_parts[1..when_parts.len() - 1].join(" ").into_bytes();
    let (include_line, include_origin) =
        include.ok_or(CanonicalSourceErrorV1::InvalidInputHandler { origin })?;
    let Some((include_prefix, include_vector)) = split_vector_subject(include_line) else {
        return Err(CanonicalSourceErrorV1::InvalidInputHandler { origin });
    };
    let expected_prefix = format!("{subject} {}", String::from_utf8_lossy(&relation));
    if include_prefix != expected_prefix {
        return Err(CanonicalSourceErrorV1::InvalidInputHandler { origin });
    }
    let result = parse_vec3_components(include_vector)
        .ok_or(CanonicalSourceErrorV1::InvalidInputHandler { origin })?;
    if parse_source_number(result[1]) != Some(0.0_f64.to_bits()) {
        return Err(CanonicalSourceErrorV1::InvalidInputHandler { origin });
    }
    let result_x = parse_input_scalar(result[0], parameters)
        .ok_or(CanonicalSourceErrorV1::InvalidInputHandler { origin })?;
    let result_z = parse_input_scalar(result[2], parameters)
        .ok_or(CanonicalSourceErrorV1::InvalidInputHandler { origin })?;
    Ok(Some(InputHandlerCst {
        origin,
        producer: semantic_producer(
            CanonicalSourceProductionV1::Handler,
            &handler_semantic_producer(block),
        ),
        designation: b"input".to_vec(),
        relation,
        result_x,
        result_z,
        include_origin,
        include_local: include_line.as_bytes().to_vec(),
    }))
}

fn parse_jump_handler(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
) -> Result<Option<JumpHandlerCst>, CanonicalSourceErrorV1> {
    let Some(header) = block[0].text.strip_prefix("on ") else {
        return Ok(None);
    };
    let mut header = header.split_whitespace();
    let Some(designation) = header.next() else {
        return Ok(None);
    };
    let Some(subject) = header.next() else {
        return jump_shape_mismatch(designation, origin);
    };
    if header.next().is_some() || !subject.starts_with('?') {
        return jump_shape_mismatch(designation, origin);
    }

    let mut section = "";
    let mut when = Vec::new();
    let mut withdraw = Vec::new();
    let mut include = Vec::new();
    let mut seen_sections = BTreeSet::new();
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let trimmed = line.text.trim();
        if line.indent == 2 {
            if trimmed == "admit" {
                return Err(CanonicalSourceErrorV1::NonCanonicalKeyword {
                    origin: line_origin(artifact, *line),
                    keyword: b"admit".to_vec(),
                });
            }
            if !matches!(trimmed, "when" | "withdraw" | "include") || !seen_sections.insert(trimmed)
            {
                return jump_shape_mismatch(designation, line_origin(artifact, *line));
            }
            section = trimmed;
            continue;
        }
        if line.indent != 4 {
            return jump_shape_mismatch(designation, line_origin(artifact, *line));
        }
        let entry = (trimmed, line_origin(artifact, *line));
        match section {
            "when" => when.push(entry),
            "withdraw" => withdraw.push(entry),
            "include" => include.push(entry),
            _ => {
                return jump_shape_mismatch(designation, line_origin(artifact, *line));
            }
        }
    }
    if when.len() != 3 || withdraw.len() != 2 || include.len() != 2 {
        return jump_shape_mismatch(designation, origin);
    }
    if withdraw[0].0 != when[0].0 || withdraw[1].0 != when[1].0 {
        return jump_shape_mismatch(designation, origin);
    }

    let Some((velocity_prefix, velocity_vector)) = split_vector_subject(when[0].0) else {
        return jump_shape_mismatch(designation, origin);
    };
    let Some(velocity_relation) = velocity_prefix
        .strip_prefix(subject)
        .and_then(|rest| rest.strip_prefix(' '))
        .filter(|rest| !rest.is_empty())
        .map(|relation| relation.as_bytes().to_vec())
    else {
        return jump_shape_mismatch(designation, origin);
    };
    let Some(velocity_parameters) = parse_vec3_components(velocity_vector).filter(|components| {
        components.iter().all(|value| value.starts_with('?'))
            && components.iter().collect::<BTreeSet<_>>().len() == 3
    }) else {
        return jump_shape_mismatch(designation, origin);
    };

    let Some((grounded_subject, grounded_relation, required_grounded)) =
        parse_boolean_clause(when[1].0)
    else {
        return jump_shape_mismatch(designation, origin);
    };
    if grounded_subject != subject {
        return jump_shape_mismatch(designation, origin);
    }

    let jump_parts = when[2].0.split_whitespace().collect::<Vec<_>>();
    if jump_parts.len() < 3
        || !jump_parts
            .last()
            .is_some_and(|value| value.starts_with('?'))
    {
        return jump_shape_mismatch(designation, origin);
    }
    let jump_speed_subject = jump_parts[0].as_bytes().to_vec();
    let jump_speed_relation = jump_parts[1..jump_parts.len() - 1].join(" ").into_bytes();
    let jump_speed_parameter = jump_parts[jump_parts.len() - 1];

    let Some((result_prefix, result_vector)) = split_vector_subject(include[0].0) else {
        return jump_shape_mismatch(designation, origin);
    };
    let expected_result_prefix =
        format!("{subject} {}", String::from_utf8_lossy(&velocity_relation));
    if result_prefix != expected_result_prefix {
        return jump_shape_mismatch(designation, origin);
    }
    let Some(result_components) = parse_vec3_components(result_vector) else {
        return jump_shape_mismatch(designation, origin);
    };
    let Some(result_velocity) = result_components
        .map(|value| parse_jump_scalar(value, velocity_parameters, jump_speed_parameter))
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .and_then(|values| values.try_into().ok())
    else {
        return jump_shape_mismatch(designation, origin);
    };

    let Some((result_subject, result_relation, result_grounded)) =
        parse_boolean_clause(include[1].0)
    else {
        return jump_shape_mismatch(designation, origin);
    };
    if result_subject != subject || result_relation.as_bytes() != grounded_relation.as_bytes() {
        return jump_shape_mismatch(designation, origin);
    }

    Ok(Some(JumpHandlerCst {
        origin,
        producer: semantic_producer(
            CanonicalSourceProductionV1::Handler,
            &handler_semantic_producer(block),
        ),
        designation: designation.as_bytes().to_vec(),
        velocity_relation,
        grounded_relation: grounded_relation.into_bytes(),
        jump_speed_subject,
        jump_speed_relation,
        required_grounded,
        result_velocity,
        result_grounded,
        includes: [
            HandlerIncludeCst {
                origin: include[0].1,
                local: include[0].0.as_bytes().to_vec(),
            },
            HandlerIncludeCst {
                origin: include[1].1,
                local: include[1].0.as_bytes().to_vec(),
            },
        ],
    }))
}

fn jump_shape_mismatch<T>(
    designation: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<Option<T>, CanonicalSourceErrorV1> {
    if designation == "jump" {
        Err(CanonicalSourceErrorV1::InvalidJumpHandler { origin })
    } else {
        Ok(None)
    }
}

fn parse_jump_scalar(
    source: &str,
    velocity_parameters: [&str; 3],
    jump_speed_parameter: &str,
) -> Option<CanonicalJumpScalarV1> {
    velocity_parameters
        .iter()
        .position(|parameter| *parameter == source)
        .and_then(|index| u8::try_from(index).ok())
        .map(CanonicalJumpScalarV1::VelocityComponent)
        .or_else(|| (source == jump_speed_parameter).then_some(CanonicalJumpScalarV1::JumpSpeed))
        .or_else(|| parse_source_number(source).map(CanonicalJumpScalarV1::Number))
}

fn parse_scalar_handler(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
) -> Result<Option<ScalarHandlerCst>, CanonicalSourceErrorV1> {
    let Some(header) = block[0].text.strip_prefix("on ") else {
        return Ok(None);
    };
    let header_parts = header.split_whitespace().collect::<Vec<_>>();
    let [designation, subject] = header_parts.as_slice() else {
        return Ok(None);
    };
    if !subject.starts_with('?') {
        return Ok(None);
    }

    let mut section = "";
    let mut when = Vec::new();
    let mut withdraw = None;
    let mut include = None;
    let mut seen_sections = BTreeSet::new();
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let trimmed = line.text.trim();
        if line.indent == 2 {
            if trimmed == "admit" {
                return Err(CanonicalSourceErrorV1::NonCanonicalKeyword {
                    origin: line_origin(artifact, *line),
                    keyword: b"admit".to_vec(),
                });
            }
            if !matches!(trimmed, "when" | "withdraw" | "include") || !seen_sections.insert(trimmed)
            {
                return Ok(None);
            }
            section = trimmed;
            continue;
        }
        if line.indent != 4 {
            return Ok(None);
        }
        let entry = (trimmed, line_origin(artifact, *line));
        match section {
            "when" => when.push(entry),
            "withdraw" => {
                if withdraw.replace(entry).is_some() {
                    return Ok(None);
                }
            }
            "include" => {
                if include.replace(entry).is_some() {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        }
    }
    let (Some((withdraw, _)), Some((include, include_origin))) = (withdraw, include) else {
        return Ok(None);
    };
    let state_bindings = when
        .iter()
        .filter(|(candidate, _)| *candidate == withdraw)
        .collect::<Vec<_>>();
    let [state_binding] = state_bindings.as_slice() else {
        return Ok(None);
    };
    let (relation, field, current, result_source) =
        if let Some((when_prefix, when_vector)) = split_vector_subject(state_binding.0) {
            let when_prefix_parts = when_prefix.split_whitespace().collect::<Vec<_>>();
            if when_prefix_parts.len() < 2 || when_prefix_parts[0] != *subject {
                return Ok(None);
            }
            let when_components = parse_vec3_components(when_vector).filter(|components| {
                components
                    .iter()
                    .all(|component| component.starts_with('?'))
                    && components.iter().collect::<BTreeSet<_>>().len() == 3
            });
            let Some(when_components) = when_components else {
                return Ok(None);
            };
            let Some((include_prefix, include_vector)) = split_vector_subject(include) else {
                return Ok(None);
            };
            if include_prefix != when_prefix {
                return Ok(None);
            }
            let Some(include_components) = parse_vec3_components(include_vector) else {
                return Ok(None);
            };
            let changed = (0..3)
                .filter(|index| include_components[*index] != when_components[*index])
                .collect::<Vec<_>>();
            let [changed] = changed.as_slice() else {
                return Ok(None);
            };
            (
                when_prefix_parts[1..].join(" "),
                Some([b"x".as_slice(), b"y".as_slice(), b"z".as_slice()][*changed].to_vec()),
                when_components[*changed],
                include_components[*changed],
            )
        } else {
            let when_parts = state_binding.0.split_whitespace().collect::<Vec<_>>();
            if when_parts.len() < 3 || when_parts[0] != *subject {
                return Ok(None);
            }
            let current = *when_parts
                .last()
                .expect("bounded scalar clause has a value");
            let relation = when_parts[1..when_parts.len() - 1].join(" ");
            let include_prefix = format!("{subject} {relation} ");
            let Some(result_source) = include.strip_prefix(&include_prefix) else {
                return Ok(None);
            };
            (relation, None, current, result_source)
        };
    if !current.starts_with('?') {
        return Ok(None);
    }
    let Some(result) = parse_scalar_expression(result_source, current) else {
        return Ok(None);
    };
    let mut declared_parameters = BTreeSet::new();
    let mut parameter_sources = BTreeMap::new();
    let mut predicates = Vec::new();
    let mut boolean_conditions = Vec::new();
    for (condition, condition_origin) in when
        .iter()
        .copied()
        .filter(|(condition, _)| *condition != withdraw)
    {
        if let Some(predicate) = parse_scalar_predicate(condition, current) {
            predicates.push(predicate);
            continue;
        }
        if parse_scalar_law_binding(condition, condition_origin).is_some() {
            return Ok(None);
        }
        if let Some(parameters) = parse_scalar_parameter_declaration(condition, subject) {
            for parameter in parameters {
                declared_parameters.insert(parameter.parameter.clone());
                if parameter_sources
                    .insert(parameter.parameter.clone(), parameter)
                    .is_some()
                {
                    return Ok(None);
                }
            }
            continue;
        }
        if let Some(condition) = parse_boolean_relation_use(condition, condition_origin) {
            boolean_conditions.push(condition);
            continue;
        }
        return Ok(None);
    }
    let mut used_parameters = BTreeSet::new();
    collect_scalar_expression_parameters(&result, &mut used_parameters);
    for predicate in &predicates {
        let (left, right) = match predicate {
            CanonicalScalarPredicateV1::Equal(left, right)
            | CanonicalScalarPredicateV1::GreaterThan(left, right)
            | CanonicalScalarPredicateV1::LessThanOrEqual(left, right) => (left, right),
        };
        collect_scalar_expression_parameters(left, &mut used_parameters);
        collect_scalar_expression_parameters(right, &mut used_parameters);
    }
    if !used_parameters.is_subset(&declared_parameters) {
        return Ok(None);
    }
    let parameters = used_parameters.into_iter().collect::<Vec<_>>();
    let parameter_sources = parameters
        .iter()
        .map(|parameter| parameter_sources.get(parameter).cloned())
        .collect::<Option<Vec<_>>>()
        .ok_or(CanonicalSourceErrorV1::InvalidScalarHandler { origin })?;
    Ok(Some(ScalarHandlerCst {
        origin,
        producer: semantic_producer(
            CanonicalSourceProductionV1::Handler,
            &handler_semantic_producer(block),
        ),
        designation: designation.as_bytes().to_vec(),
        subject: subject.as_bytes().to_vec(),
        relation: relation.into_bytes(),
        field,
        parameters,
        parameter_sources,
        predicates,
        boolean_conditions,
        result,
        include: HandlerIncludeCst {
            origin: include_origin,
            local: include.as_bytes().to_vec(),
        },
    }))
}

fn parse_general_handler(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
) -> Result<Option<GeneralHandlerCst>, CanonicalSourceErrorV1> {
    let Some(header) = block[0].text.strip_prefix("on ") else {
        return Ok(None);
    };
    let header = header.split_whitespace().collect::<Vec<_>>();
    let [designation, subject, argument_designations @ ..] = header.as_slice() else {
        return Ok(None);
    };
    if !subject.starts_with('?') || *designation == "tick" {
        return Ok(None);
    }
    let mut seen_arguments = BTreeSet::new();
    let arguments = argument_designations
        .iter()
        .enumerate()
        .map(|(ordinal, designation)| {
            if !designation.starts_with('?')
                || *designation == *subject
                || !seen_arguments.insert(designation.as_bytes().to_vec())
            {
                return Err(CanonicalSourceErrorV1::InvalidGeneralHandler { origin });
            }
            Ok(GeneralHandlerArgumentCst {
                designation: designation.as_bytes().to_vec(),
                ordinal: u16::try_from(ordinal)
                    .map_err(|_| CanonicalSourceErrorV1::InvalidGeneralHandler { origin })?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut section = "";
    let mut when = Vec::new();
    let mut create = Vec::new();
    let mut withdraw = Vec::new();
    let mut include = Vec::new();
    let mut seen_sections = BTreeSet::new();
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let trimmed = line.text.trim();
        if line.indent == 2 {
            if trimmed == "admit" {
                return Err(CanonicalSourceErrorV1::NonCanonicalKeyword {
                    origin: line_origin(artifact, *line),
                    keyword: b"admit".to_vec(),
                });
            }
            if !matches!(trimmed, "when" | "create" | "withdraw" | "include")
                || !seen_sections.insert(trimmed)
            {
                return Ok(None);
            }
            section = trimmed;
            continue;
        }
        if line.indent != 4 {
            return Ok(None);
        }
        let entry = (trimmed, line_origin(artifact, *line));
        match section {
            "when" => when.push(entry),
            "create" => create.push(entry),
            "withdraw" => withdraw.push(entry),
            "include" => include.push(entry),
            _ => return Ok(None),
        }
    }
    if include.is_empty() {
        return Ok(None);
    }

    let mut seen_creations = BTreeSet::new();
    let creations = create
        .iter()
        .enumerate()
        .map(|(binder, (source, _))| {
            let (parameter, domain) = parse_general_referent_creation(source)
                .ok_or(CanonicalSourceErrorV1::InvalidGeneralHandler { origin })?;
            if parameter == subject.as_bytes()
                || seen_arguments.contains(parameter.as_slice())
                || !seen_creations.insert(parameter.clone())
            {
                return Err(CanonicalSourceErrorV1::InvalidGeneralHandler { origin });
            }
            Ok(GeneralReferentCreationCst {
                parameter,
                domain,
                binder: u16::try_from(binder)
                    .map_err(|_| CanonicalSourceErrorV1::InvalidGeneralHandler { origin })?,
            })
        })
        .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?;

    let mut parameter_sources = BTreeMap::<Vec<u8>, ScalarParameterSourceCst>::new();
    let mut membership_sources = Vec::new();
    let mut scalar_bindings = BTreeMap::<Vec<u8>, ScalarLawBindingCst>::new();
    let mut predicates = Vec::new();
    let mut boolean_conditions = Vec::new();
    for (condition, condition_origin) in &when {
        if let Some(predicate) = parse_scalar_predicate(condition, "") {
            predicates.push(predicate);
            continue;
        }
        if let Some(condition) = parse_boolean_relation_use(condition, *condition_origin) {
            boolean_conditions.push(condition);
            continue;
        }
        if let Some(binding) = parse_scalar_law_binding(condition, *condition_origin) {
            if seen_arguments.contains(&binding.parameter)
                || parameter_sources.contains_key(&binding.parameter)
                || scalar_bindings
                    .insert(binding.parameter.clone(), binding)
                    .is_some()
            {
                return Ok(None);
            }
            continue;
        }
        let Some(sources) = parse_general_state_declaration(condition, subject) else {
            return Ok(None);
        };
        for source in sources {
            if seen_arguments.contains(&source.parameter) {
                membership_sources.push(source);
                continue;
            }
            if scalar_bindings.contains_key(&source.parameter)
                || parameter_sources
                    .insert(source.parameter.clone(), source)
                    .is_some()
            {
                return Ok(None);
            }
        }
    }

    let mut assignments = Vec::new();
    let mut insertions = Vec::new();
    let mut removals = Vec::new();
    let mut required_sources = Vec::new();
    let mut used_includes = BTreeSet::new();
    let mut matched_withdrawals = BTreeSet::new();
    for (withdraw_index, (withdraw, _)) in withdraw.iter().enumerate() {
        if !when.iter().any(|(condition, _)| condition == withdraw) {
            return Ok(None);
        }
        let matching = include
            .iter()
            .enumerate()
            .filter(|(include_index, _)| !used_includes.contains(include_index))
            .filter_map(|(include_index, (include, _))| {
                parse_general_assignments(withdraw, include, subject)
                    .map(|replacement| (include_index, replacement))
            })
            .collect::<Vec<_>>();
        let Some((include_index, replacement)) = matching.first() else {
            continue;
        };
        if matching.len() != 1 {
            return Err(CanonicalSourceErrorV1::InvalidGeneralHandler { origin });
        }
        if let Some(binding) = &replacement.aggregate_binding {
            let Some(source) = parameter_sources.remove(binding) else {
                return Ok(None);
            };
            if source.field.is_some()
                || replacement.required_sources.first().is_none_or(|required| {
                    required.subject != source.subject || required.relation != source.relation
                })
            {
                return Ok(None);
            }
        }
        assignments.extend(replacement.assignments.clone());
        required_sources.extend(replacement.required_sources.clone());
        used_includes.insert(*include_index);
        matched_withdrawals.insert(withdraw_index);
    }
    for (include_index, (include, _)) in include.iter().enumerate() {
        if used_includes.contains(&include_index) {
            continue;
        }
        let Some(mut inserted) = parse_general_insertion(include, subject) else {
            return Ok(None);
        };
        insertions.append(&mut inserted);
    }
    for (withdraw_index, (withdraw, _)) in withdraw.iter().enumerate() {
        if matched_withdrawals.contains(&withdraw_index) {
            continue;
        }
        let Some(mut removed) = parse_general_state_declaration(withdraw, subject) else {
            return Ok(None);
        };
        removals.append(&mut removed);
    }
    let includes = include
        .iter()
        .map(|(include, origin)| HandlerIncludeCst {
            origin: *origin,
            local: include.as_bytes().to_vec(),
        })
        .collect::<Vec<_>>();
    if assignments.is_empty() && insertions.is_empty() && removals.is_empty() {
        return Ok(None);
    }

    let mut used_parameters = BTreeSet::new();
    for predicate in &predicates {
        let (left, right) = match predicate {
            CanonicalScalarPredicateV1::Equal(left, right)
            | CanonicalScalarPredicateV1::GreaterThan(left, right)
            | CanonicalScalarPredicateV1::LessThanOrEqual(left, right) => (left, right),
        };
        collect_scalar_expression_parameters(left, &mut used_parameters);
        collect_scalar_expression_parameters(right, &mut used_parameters);
    }
    for assignment in &assignments {
        collect_scalar_expression_parameters(&assignment.value, &mut used_parameters);
    }
    for insertion in &insertions {
        collect_scalar_expression_parameters(&insertion.value, &mut used_parameters);
    }
    for binding in scalar_bindings.values() {
        collect_scalar_expression_parameters(&binding.value, &mut used_parameters);
    }
    if used_parameters.iter().any(|parameter| {
        !parameter_sources.contains_key(parameter)
            && !scalar_bindings.contains_key(parameter)
            && !seen_arguments.contains(parameter)
            && !seen_creations.contains(parameter)
    }) {
        return Err(CanonicalSourceErrorV1::InvalidGeneralHandler { origin });
    }

    Ok(Some(GeneralHandlerCst {
        origin,
        producer: semantic_producer(
            CanonicalSourceProductionV1::Handler,
            &handler_semantic_producer(block),
        ),
        designation: designation.as_bytes().to_vec(),
        subject: subject.as_bytes().to_vec(),
        arguments,
        creations,
        parameter_sources: parameter_sources.into_values().collect(),
        membership_sources,
        required_sources,
        scalar_bindings: scalar_bindings.into_values().collect(),
        predicates,
        boolean_conditions,
        assignments,
        insertions,
        removals,
        includes,
    }))
}

fn parse_general_referent_creation(source: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let parts = source.split_whitespace().collect::<Vec<_>>();
    let [parameter, "∈", domain] = parts.as_slice() else {
        return None;
    };
    if !parameter.starts_with('?') || domain.starts_with('?') {
        return None;
    }
    Some((parameter.as_bytes().to_vec(), domain.as_bytes().to_vec()))
}

fn parse_general_insertion(
    source: &str,
    _handler_subject: &str,
) -> Option<Vec<GeneralAssignmentCst>> {
    if let Some((prefix, shape, fields)) = split_shape_subject(source) {
        let prefix = prefix.split_whitespace().collect::<Vec<_>>();
        if prefix.len() < 2 {
            return None;
        }
        let subject = prefix[0].as_bytes().to_vec();
        let relation = prefix[1..].join(" ").into_bytes();
        return parse_shape_fields(fields)?
            .into_iter()
            .map(|(field, value)| {
                Some(GeneralAssignmentCst {
                    target: ScalarParameterSourceCst {
                        parameter: Vec::new(),
                        subject: subject.clone(),
                        relation: relation.clone(),
                        shape: Some(shape.as_bytes().to_vec()),
                        field: Some(field.as_bytes().to_vec()),
                    },
                    value: parse_scalar_expression(value, "")?,
                })
            })
            .collect();
    }

    let parts = source.split_whitespace().collect::<Vec<_>>();
    let [subject, relation @ .., value] = parts.as_slice() else {
        return None;
    };
    if relation.is_empty() {
        return None;
    }
    Some(vec![GeneralAssignmentCst {
        target: ScalarParameterSourceCst {
            parameter: Vec::new(),
            subject: subject.as_bytes().to_vec(),
            relation: relation.join(" ").into_bytes(),
            shape: None,
            field: None,
        },
        value: parse_scalar_expression(value, "")?,
    }])
}

fn parse_scalar_law_binding(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Option<ScalarLawBindingCst> {
    let (value, rest) = source.split_once(" clamped between ")?;
    let (lower, rest) = rest.split_once(" and ")?;
    let (upper, parameter) = rest.split_once(" as ")?;
    if !parameter.starts_with('?') || parameter.split_whitespace().count() != 1 {
        return None;
    }
    Some(ScalarLawBindingCst {
        origin,
        parameter: parameter.as_bytes().to_vec(),
        value: CanonicalScalarExpressionV1::Clamp(
            Box::new(parse_scalar_expression(value, "")?),
            Box::new(parse_scalar_expression(lower, "")?),
            Box::new(parse_scalar_expression(upper, "")?),
        ),
    })
}

fn parse_general_state_declaration(
    source: &str,
    _handler_subject: &str,
) -> Option<Vec<ScalarParameterSourceCst>> {
    if let Some((prefix, shape, fields)) = split_shape_subject(source) {
        let prefix = prefix.split_whitespace().collect::<Vec<_>>();
        if prefix.len() < 2 {
            return None;
        }
        let subject = prefix[0].as_bytes().to_vec();
        let relation = prefix[1..].join(" ").into_bytes();
        let fields = parse_shape_fields(fields)?;
        if !fields
            .iter()
            .all(|(_, parameter)| parameter.starts_with('?'))
        {
            return None;
        }
        return fields
            .into_iter()
            .map(|(field, parameter)| {
                Some(ScalarParameterSourceCst {
                    parameter: parameter.as_bytes().to_vec(),
                    subject: subject.clone(),
                    relation: relation.clone(),
                    shape: Some(shape.as_bytes().to_vec()),
                    field: Some(field.as_bytes().to_vec()),
                })
            })
            .collect();
    }

    let parts = source.split_whitespace().collect::<Vec<_>>();
    let parameter = *parts.last()?;
    if parts.len() < 3 || !parameter.starts_with('?') {
        return None;
    }
    Some(vec![ScalarParameterSourceCst {
        parameter: parameter.as_bytes().to_vec(),
        subject: parts[0].as_bytes().to_vec(),
        relation: parts[1..parts.len() - 1].join(" ").into_bytes(),
        shape: None,
        field: None,
    }])
}

fn parse_general_assignments(
    withdraw: &str,
    include: &str,
    handler_subject: &str,
) -> Option<GeneralReplacementCst> {
    let targets = parse_general_state_declaration(withdraw, handler_subject)?;
    if let Some((withdraw_prefix, withdraw_shape, withdraw_fields)) = split_shape_subject(withdraw)
    {
        let (include_prefix, include_shape, include_fields) = split_shape_subject(include)?;
        if include_prefix != withdraw_prefix || include_shape != withdraw_shape {
            return None;
        }
        let current = parse_shape_fields(withdraw_fields)?;
        let result = parse_shape_fields(include_fields)?;
        if current
            .iter()
            .map(|(field, _)| *field)
            .ne(result.iter().map(|(field, _)| *field))
        {
            return None;
        }
        let assignments = targets
            .into_iter()
            .zip(current)
            .zip(result)
            .filter_map(|((target, (_, current)), (_, result))| {
                (current != result).then(|| {
                    Some(GeneralAssignmentCst {
                        target,
                        value: parse_scalar_expression(result, "")?,
                    })
                })
            })
            .collect::<Option<Vec<_>>>()?;
        return Some(GeneralReplacementCst {
            assignments,
            aggregate_binding: None,
            required_sources: vec![],
        });
    }

    let target = targets.into_iter().next()?;
    let prefix = format!(
        "{} {}",
        String::from_utf8_lossy(&target.subject),
        String::from_utf8_lossy(&target.relation)
    );
    if let Some((include_prefix, shape, fields)) = split_shape_subject(include) {
        if include_prefix != prefix {
            return None;
        }
        let aggregate_binding = target.parameter.clone();
        let fields = parse_shape_fields(fields)?;
        let mut required_sources = Vec::with_capacity(fields.len());
        let assignments = fields
            .into_iter()
            .map(|(field, value)| {
                let mut component = target.clone();
                component.parameter.clear();
                component.shape = Some(shape.as_bytes().to_vec());
                component.field = Some(field.as_bytes().to_vec());
                required_sources.push(component.clone());
                Some(GeneralAssignmentCst {
                    target: component,
                    value: parse_scalar_expression(value, "")?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        return Some(GeneralReplacementCst {
            assignments,
            aggregate_binding: Some(aggregate_binding),
            required_sources,
        });
    }
    let value = include.strip_prefix(&format!("{prefix} "))?;
    Some(GeneralReplacementCst {
        assignments: vec![GeneralAssignmentCst {
            target,
            value: parse_scalar_expression(value, "")?,
        }],
        aggregate_binding: None,
        required_sources: vec![],
    })
}

fn parse_scalar_expression(source: &str, current: &str) -> Option<CanonicalScalarExpressionV1> {
    let mut parser = ScalarExpressionParser {
        source: source.as_bytes(),
        cursor: 0,
        current,
    };
    let expression = parser.additive()?;
    parser.skip_spaces();
    (parser.cursor == parser.source.len()).then_some(expression)
}

fn parse_scalar_atom(source: &str, current: &str) -> Option<CanonicalScalarExpressionV1> {
    if source.starts_with('"') {
        parse_text_literal(source).map(CanonicalScalarExpressionV1::Text)
    } else if source == current {
        Some(CanonicalScalarExpressionV1::Current)
    } else if source.starts_with('?') {
        Some(CanonicalScalarExpressionV1::Parameter(
            source.as_bytes().to_vec(),
        ))
    } else if source == "true" {
        Some(CanonicalScalarExpressionV1::Boolean(true))
    } else if source == "false" {
        Some(CanonicalScalarExpressionV1::Boolean(false))
    } else {
        parse_source_number(source)
            .map(CanonicalScalarExpressionV1::Number)
            .or_else(|| {
                Some(CanonicalScalarExpressionV1::Symbol(
                    source.as_bytes().to_vec(),
                ))
            })
    }
}

fn parse_scalar_predicate(source: &str, current: &str) -> Option<CanonicalScalarPredicateV1> {
    let (left, right, constructor) = if let Some((left, right)) =
        split_once_outside_text(source, " >= ")
    {
        (
            right,
            left,
            CanonicalScalarPredicateV1::LessThanOrEqual as fn(_, _) -> CanonicalScalarPredicateV1,
        )
    } else if let Some((left, right)) = split_once_outside_text(source, " <= ") {
        (
            left,
            right,
            CanonicalScalarPredicateV1::LessThanOrEqual as fn(_, _) -> CanonicalScalarPredicateV1,
        )
    } else if let Some((left, right)) = split_once_outside_text(source, " > ") {
        (
            left,
            right,
            CanonicalScalarPredicateV1::GreaterThan as fn(_, _) -> CanonicalScalarPredicateV1,
        )
    } else if let Some((left, right)) = split_once_outside_text(source, " < ") {
        (
            right,
            left,
            CanonicalScalarPredicateV1::GreaterThan as fn(_, _) -> CanonicalScalarPredicateV1,
        )
    } else {
        let (left, right) = split_once_outside_text(source, " = ")?;
        (
            left,
            right,
            CanonicalScalarPredicateV1::Equal as fn(_, _) -> CanonicalScalarPredicateV1,
        )
    };
    Some(constructor(
        parse_scalar_expression(left, current)?,
        parse_scalar_expression(right, current)?,
    ))
}

fn split_once_outside_text<'a>(source: &'a str, separator: &str) -> Option<(&'a str, &'a str)> {
    let bytes = source.as_bytes();
    let separator = separator.as_bytes();
    let mut in_text = false;
    let mut escaped = false;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_text {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_text = false;
            }
        } else if byte == b'"' {
            in_text = true;
        } else if bytes[index..].starts_with(separator) {
            return Some((source.get(..index)?, source.get(index + separator.len()..)?));
        }
        index += 1;
    }
    None
}

struct ScalarExpressionParser<'a> {
    source: &'a [u8],
    cursor: usize,
    current: &'a str,
}

impl ScalarExpressionParser<'_> {
    fn additive(&mut self) -> Option<CanonicalScalarExpressionV1> {
        let mut value = self.multiplicative()?;
        loop {
            self.skip_spaces();
            let operation = if self.take_exact(b"++") {
                Some(2)
            } else {
                self.take_one(&[b'+', b'-'])
                    .map(|operation| usize::from(operation == b'-'))
            };
            let Some(operation) = operation else { break };
            let right = self.multiplicative()?;
            value = match operation {
                0 => CanonicalScalarExpressionV1::Add(Box::new(value), Box::new(right)),
                1 => CanonicalScalarExpressionV1::Subtract(Box::new(value), Box::new(right)),
                2 => CanonicalScalarExpressionV1::Concatenate(Box::new(value), Box::new(right)),
                _ => unreachable!(),
            };
        }
        Some(value)
    }

    fn multiplicative(&mut self) -> Option<CanonicalScalarExpressionV1> {
        let mut value = self.primary()?;
        loop {
            self.skip_spaces();
            let operation = self.take_one(&[b'*', b'/']);
            let Some(operation) = operation else { break };
            let right = self.primary()?;
            value = match operation {
                b'*' => CanonicalScalarExpressionV1::Multiply(Box::new(value), Box::new(right)),
                b'/' => CanonicalScalarExpressionV1::Divide(Box::new(value), Box::new(right)),
                _ => unreachable!(),
            };
        }
        Some(value)
    }

    fn primary(&mut self) -> Option<CanonicalScalarExpressionV1> {
        self.skip_spaces();
        if self.source.get(self.cursor) == Some(&b'"') {
            let start = self.cursor;
            self.cursor += 1;
            let mut escaped = false;
            while let Some(byte) = self.source.get(self.cursor) {
                self.cursor += 1;
                if escaped {
                    escaped = false;
                } else if *byte == b'\\' {
                    escaped = true;
                } else if *byte == b'"' {
                    let literal = std::str::from_utf8(&self.source[start..self.cursor]).ok()?;
                    return parse_text_literal(literal).map(CanonicalScalarExpressionV1::Text);
                }
            }
            return None;
        }
        if self.source.get(self.cursor) == Some(&b'(') {
            self.cursor += 1;
            let value = self.additive()?;
            self.skip_spaces();
            (self.source.get(self.cursor) == Some(&b')')).then(|| self.cursor += 1)?;
            return Some(value);
        }
        let start = self.cursor;
        if self.source.get(self.cursor) == Some(&b'-') {
            self.cursor += 1;
        }
        while let Some(byte) = self.source.get(self.cursor)
            && !byte.is_ascii_whitespace()
            && !matches!(*byte, b'+' | b'*' | b'/' | b'(' | b')')
        {
            self.cursor += 1;
        }
        (self.cursor > start).then_some(())?;
        let atom = std::str::from_utf8(&self.source[start..self.cursor]).ok()?;
        parse_scalar_atom(atom, self.current)
    }

    fn skip_spaces(&mut self) {
        while self
            .source
            .get(self.cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.cursor += 1;
        }
    }

    fn take_one(&mut self, accepted: &[u8]) -> Option<u8> {
        let byte = *self.source.get(self.cursor)?;
        accepted.contains(&byte).then(|| {
            self.cursor += 1;
            byte
        })
    }

    fn take_exact(&mut self, expected: &[u8]) -> bool {
        if !self.source[self.cursor..].starts_with(expected) {
            return false;
        }
        self.cursor += expected.len();
        true
    }
}

fn parse_scalar_parameter_declaration(
    source: &str,
    handler_subject: &str,
) -> Option<Vec<ScalarParameterSourceCst>> {
    if let Some((prefix, vector)) = split_vector_subject(source) {
        let prefix = prefix.split_whitespace().collect::<Vec<_>>();
        if prefix.len() < 2 || (prefix[0].starts_with('?') && prefix[0] != handler_subject) {
            return None;
        }
        let subject = prefix[0].as_bytes().to_vec();
        let relation = prefix[1..].join(" ").into_bytes();
        let components = parse_vec3_components(vector)?;
        return components
            .into_iter()
            .zip([b"x".as_slice(), b"y".as_slice(), b"z".as_slice()])
            .map(|(component, field)| {
                component
                    .starts_with('?')
                    .then(|| ScalarParameterSourceCst {
                        parameter: component.as_bytes().to_vec(),
                        subject: subject.clone(),
                        relation: relation.clone(),
                        shape: Some(b"Vec3".to_vec()),
                        field: Some(field.to_vec()),
                    })
            })
            .collect();
    }

    let parts = source.split_whitespace().collect::<Vec<_>>();
    let parameter = *parts.last()?;
    if parts.len() < 3
        || (parts[0].starts_with('?') && parts[0] != handler_subject)
        || !parameter.starts_with('?')
    {
        return None;
    }
    Some(vec![ScalarParameterSourceCst {
        parameter: parameter.as_bytes().to_vec(),
        subject: parts[0].as_bytes().to_vec(),
        relation: parts[1..parts.len() - 1].join(" ").into_bytes(),
        shape: None,
        field: None,
    }])
}

fn collect_scalar_expression_parameters(
    expression: &CanonicalScalarExpressionV1,
    parameters: &mut BTreeSet<Vec<u8>>,
) {
    match expression {
        CanonicalScalarExpressionV1::Parameter(parameter) => {
            parameters.insert(parameter.clone());
        }
        CanonicalScalarExpressionV1::Concatenate(left, right)
        | CanonicalScalarExpressionV1::Add(left, right)
        | CanonicalScalarExpressionV1::Subtract(left, right)
        | CanonicalScalarExpressionV1::Multiply(left, right)
        | CanonicalScalarExpressionV1::Divide(left, right) => {
            collect_scalar_expression_parameters(left, parameters);
            collect_scalar_expression_parameters(right, parameters);
        }
        CanonicalScalarExpressionV1::Clamp(value, lower, upper) => {
            collect_scalar_expression_parameters(value, parameters);
            collect_scalar_expression_parameters(lower, parameters);
            collect_scalar_expression_parameters(upper, parameters);
        }
        CanonicalScalarExpressionV1::Current
        | CanonicalScalarExpressionV1::Number(_)
        | CanonicalScalarExpressionV1::Boolean(_)
        | CanonicalScalarExpressionV1::Symbol(_)
        | CanonicalScalarExpressionV1::Text(_) => {}
    }
}

fn parse_boolean_relation_use(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Option<BooleanRelationUseCst> {
    parse_boolean_clause(source)?;
    Some(BooleanRelationUseCst {
        origin,
        source: source.as_bytes().to_vec(),
    })
}

fn parse_boolean_law(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
) -> Result<Option<BooleanLawCst>, CanonicalSourceErrorV1> {
    let Some(designation) = block[0].text.strip_prefix("law ") else {
        return Ok(None);
    };
    let designation = designation_bytes(designation, origin)?;
    let mut section = "";
    let mut conditions = Vec::new();
    let mut result = None;
    let mut seen_sections = BTreeSet::new();
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let trimmed = line.text.trim();
        if line.indent == 2 {
            if !matches!(trimmed, "if" | "then") || !seen_sections.insert(trimmed) {
                return Ok(None);
            }
            section = trimmed;
            continue;
        }
        if line.indent != 4 {
            return Ok(None);
        }
        let entry = (trimmed, line_origin(artifact, *line));
        match section {
            "if" => conditions.push(entry),
            "then" if result.replace(entry).is_none() => {}
            _ => return Ok(None),
        }
    }
    let Some((result, result_origin)) = result else {
        return Ok(None);
    };
    let Some(result) = parse_boolean_relation_use(result, result_origin) else {
        return Ok(None);
    };

    let mut parameter_sources = BTreeMap::<Vec<u8>, ScalarParameterSourceCst>::new();
    let mut predicates = Vec::new();
    for (condition, _) in conditions {
        if let Some(predicate) = parse_scalar_predicate(condition, "") {
            predicates.push(predicate);
            continue;
        }
        let Some(sources) = parse_law_state_declaration(condition) else {
            return Ok(None);
        };
        for source in sources {
            if parameter_sources
                .insert(source.parameter.clone(), source)
                .is_some()
            {
                return Ok(None);
            }
        }
    }
    let mut used_parameters = BTreeSet::new();
    for predicate in &predicates {
        let (left, right) = match predicate {
            CanonicalScalarPredicateV1::Equal(left, right)
            | CanonicalScalarPredicateV1::GreaterThan(left, right)
            | CanonicalScalarPredicateV1::LessThanOrEqual(left, right) => (left, right),
        };
        collect_scalar_expression_parameters(left, &mut used_parameters);
        collect_scalar_expression_parameters(right, &mut used_parameters);
    }
    if used_parameters
        .iter()
        .any(|parameter| !parameter_sources.contains_key(parameter))
    {
        return Ok(None);
    }
    Ok(Some(BooleanLawCst {
        origin,
        designation,
        parameter_sources: parameter_sources.into_values().collect(),
        predicates,
        result,
    }))
}

fn parse_law_state_declaration(source: &str) -> Option<Vec<ScalarParameterSourceCst>> {
    if let Some((prefix, vector)) = split_vector_subject(source) {
        let prefix = prefix.split_whitespace().collect::<Vec<_>>();
        if prefix.len() < 2 {
            return None;
        }
        let subject = prefix[0].as_bytes().to_vec();
        let relation = prefix[1..].join(" ").into_bytes();
        let components = parse_vec3_components(vector)?;
        if !components
            .iter()
            .all(|component| component.starts_with('?'))
        {
            return None;
        }
        return components
            .into_iter()
            .zip([b"x".as_slice(), b"y".as_slice(), b"z".as_slice()])
            .map(|(component, field)| {
                Some(ScalarParameterSourceCst {
                    parameter: component.as_bytes().to_vec(),
                    subject: subject.clone(),
                    relation: relation.clone(),
                    shape: Some(b"Vec3".to_vec()),
                    field: Some(field.to_vec()),
                })
            })
            .collect();
    }

    let parts = source.split_whitespace().collect::<Vec<_>>();
    let parameter = *parts.last()?;
    if parts.len() < 3 || !parameter.starts_with('?') {
        return None;
    }
    Some(vec![ScalarParameterSourceCst {
        parameter: parameter.as_bytes().to_vec(),
        subject: parts[0].as_bytes().to_vec(),
        relation: parts[1..parts.len() - 1].join(" ").into_bytes(),
        shape: None,
        field: None,
    }])
}

fn parse_clamp_law(
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
) -> Option<ClampLawCst> {
    let lines = block
        .iter()
        .filter(|line| !line.text.trim().is_empty())
        .map(|line| line.text.trim())
        .collect::<Vec<_>>();
    let (designation, branch, expected): (&str, ClampBranchV1, &[&str]) = match lines.first()? {
        &"law clamp-lower" => (
            "clamp-lower",
            ClampBranchV1::Lower,
            &[
                "law clamp-lower",
                "if",
                "?lower <= ?upper",
                "?value < ?lower",
                "then",
                "?value clamped between ?lower and ?upper as ?lower",
            ],
        ),
        &"law clamp-interior" => (
            "clamp-interior",
            ClampBranchV1::Interior,
            &[
                "law clamp-interior",
                "if",
                "?lower <= ?value",
                "?value <= ?upper",
                "then",
                "?value clamped between ?lower and ?upper as ?value",
            ],
        ),
        &"law clamp-upper" => (
            "clamp-upper",
            ClampBranchV1::Upper,
            &[
                "law clamp-upper",
                "if",
                "?lower <= ?upper",
                "?value > ?upper",
                "then",
                "?value clamped between ?lower and ?upper as ?upper",
            ],
        ),
        _ => return None,
    };
    (lines == expected).then(|| ClampLawCst {
        origin,
        designation: designation.as_bytes().to_vec(),
        branch,
    })
}

fn parse_clamp_derive(head: &str, origin: CanonicalSourceOriginV1) -> Option<ClampDeriveCst> {
    let (designation, branch) = match head {
        "derive clamp-lower" => (b"clamp-lower".as_slice(), ClampBranchV1::Lower),
        "derive clamp-interior" => (b"clamp-interior".as_slice(), ClampBranchV1::Interior),
        "derive clamp-upper" => (b"clamp-upper".as_slice(), ClampBranchV1::Upper),
        _ => return None,
    };
    Some(ClampDeriveCst {
        origin,
        designation: designation.to_vec(),
        branch,
    })
}

fn validate_clamp_derivation(
    cst: &CanonicalSourceCstV1,
    origin: CanonicalSourceOriginV1,
) -> Result<(), CanonicalSourceErrorV1> {
    let mut laws = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::ClampLaw(law) => Some(law),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut derives = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::ClampDerive(derive) => Some(derive),
            _ => None,
        })
        .collect::<Vec<_>>();
    laws.sort_by_key(|law| law.branch);
    derives.sort_by_key(|derive| derive.branch);
    if laws.iter().map(|law| law.branch).ne([
        ClampBranchV1::Lower,
        ClampBranchV1::Interior,
        ClampBranchV1::Upper,
    ]) || derives.iter().map(|derive| derive.branch).ne([
        ClampBranchV1::Lower,
        ClampBranchV1::Interior,
        ClampBranchV1::Upper,
    ]) || laws
        .iter()
        .map(|law| law.designation.as_slice())
        .ne(derives.iter().map(|derive| derive.designation.as_slice()))
    {
        return Err(CanonicalSourceErrorV1::MissingExecutableBinding { origin });
    }
    Ok(())
}

fn parse_tick_handler(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
) -> Result<Option<TickHandlerCst>, CanonicalSourceErrorV1> {
    if block[0].text != "on tick ?player ?dt" {
        return Ok(None);
    }
    let mut section = "";
    let mut when = Vec::new();
    let mut withdraw = Vec::new();
    let mut include = Vec::new();
    let mut seen_sections = BTreeSet::new();
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let trimmed = line.text.trim();
        if line.indent == 2 {
            if trimmed == "admit" {
                return Err(CanonicalSourceErrorV1::NonCanonicalKeyword {
                    origin: line_origin(artifact, *line),
                    keyword: b"admit".to_vec(),
                });
            }
            if !matches!(trimmed, "when" | "withdraw" | "include") || !seen_sections.insert(trimmed)
            {
                return Err(CanonicalSourceErrorV1::InvalidTickProfile {
                    origin: line_origin(artifact, *line),
                });
            }
            section = trimmed;
            continue;
        }
        if line.indent != 4 {
            return Err(CanonicalSourceErrorV1::InvalidTickProfile {
                origin: line_origin(artifact, *line),
            });
        }
        let entry = (trimmed, line_origin(artifact, *line));
        match section {
            "when" => when.push(entry),
            "withdraw" => withdraw.push(entry),
            "include" => include.push(entry),
            _ => {
                return Err(CanonicalSourceErrorV1::InvalidTickProfile {
                    origin: line_origin(artifact, *line),
                });
            }
        }
    }
    if when.first().map(|entry| entry.0) != Some("?dt > 0.0") {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    }
    let grounded = when
        .get(4)
        .and_then(|entry| parse_boolean_clause(entry.0))
        .filter(|(subject, relation, _)| *subject == "?player" && relation == "grounded")
        .map(|(_, _, value)| value)
        .ok_or(CanonicalSourceErrorV1::InvalidTickProfile { origin })?;
    let grounded_branch = grounded && when.len() == 13 && withdraw.len() == 2 && include.len() == 2;
    let airborne_branch =
        !grounded && when.len() == 15 && withdraw.len() == 2 && include.len() == 2;
    let landing_branch = !grounded && when.len() == 15 && withdraw.len() == 3 && include.len() == 3;
    if !grounded_branch && !airborne_branch && !landing_branch {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    }

    let expected_position = if grounded_branch {
        "?player position Vec3 { x: ?position-x, y: ?floor, z: ?position-z }"
    } else {
        "?player position Vec3 { x: ?position-x, y: ?position-y, z: ?position-z }"
    };
    let expected_velocity = if grounded_branch {
        "?player velocity Vec3 { x: ?velocity-x, y: 0.0, z: ?velocity-z }"
    } else {
        "?player velocity Vec3 { x: ?velocity-x, y: ?velocity-y, z: ?velocity-z }"
    };
    let fixed_prefix = [
        expected_position,
        expected_velocity,
        "?player horizontal intent Vec3 { x: ?intent-x, y: 0.0, z: ?intent-z }",
    ];
    if when[1..4].iter().map(|entry| entry.0).ne(fixed_prefix) {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    }
    let constants = if grounded_branch {
        &when[5..11]
    } else {
        &when[5..12]
    };
    let expected_constants: &[&str] = if grounded_branch {
        &[
            "jump-arena move speed ?move-speed",
            "jump-arena floor height ?floor",
            "jump-arena minimum x ?min-x",
            "jump-arena maximum x ?max-x",
            "jump-arena minimum z ?min-z",
            "jump-arena maximum z ?max-z",
        ]
    } else {
        &[
            "jump-arena gravity ?gravity",
            "jump-arena move speed ?move-speed",
            "jump-arena floor height ?floor",
            "jump-arena minimum x ?min-x",
            "jump-arena maximum x ?max-x",
            "jump-arena minimum z ?min-z",
            "jump-arena maximum z ?max-z",
        ]
    };
    let constants_match = constants
        .iter()
        .zip(expected_constants)
        .all(|(actual, expected)| {
            actual.0 == *expected
                || (*expected == "jump-arena move speed ?move-speed"
                    && actual.0 == "?player move speed ?move-speed")
        });
    if !constants_match || constants.len() != expected_constants.len() {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    }
    let clamp_start = if grounded_branch { 11 } else { 12 };
    let mut derived = BTreeMap::new();
    for line in &when[clamp_start..clamp_start + 2] {
        let (name, expression) = parse_tick_clamp_binding(line.0, &derived)
            .ok_or(CanonicalSourceErrorV1::InvalidTickProfile { origin: line.1 })?;
        derived.insert(name, expression);
    }
    if !derived.contains_key("?next-x") || !derived.contains_key("?next-z") {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    }

    let mut predicates = vec![
        parse_tick_comparison(when[0].0, &derived)
            .ok_or(CanonicalSourceErrorV1::InvalidTickProfile { origin: when[0].1 })?,
        CanonicalTickPredicateV1::EqualBoolean(CanonicalTickValueV1::Grounded, grounded),
    ];
    if !grounded_branch {
        predicates.push(
            parse_tick_comparison(when[14].0, &derived)
                .ok_or(CanonicalSourceErrorV1::InvalidTickProfile { origin: when[14].1 })?,
        );
    }
    if withdraw.get(0).map(|entry| entry.0) != Some(expected_position)
        || withdraw.get(1).map(|entry| entry.0) != Some(expected_velocity)
        || (landing_branch
            && withdraw.get(2).map(|entry| entry.0) != Some("?player grounded false"))
    {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    }

    let mut assignments = Vec::new();
    let (position_prefix, position_vector) = include
        .first()
        .and_then(|entry| split_vector_subject(entry.0))
        .ok_or(CanonicalSourceErrorV1::InvalidTickProfile { origin })?;
    let (velocity_prefix, velocity_vector) = include
        .get(1)
        .and_then(|entry| split_vector_subject(entry.0))
        .ok_or(CanonicalSourceErrorV1::InvalidTickProfile { origin })?;
    if position_prefix != "?player position" || velocity_prefix != "?player velocity" {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    }
    for (target, source) in parse_vec3_components(position_vector)
        .ok_or(CanonicalSourceErrorV1::InvalidTickProfile { origin })?
        .into_iter()
        .enumerate()
    {
        assignments.push(CanonicalTickAssignmentV1 {
            target: CanonicalTickAssignmentTargetV1::PositionComponent(target as u8),
            value: CanonicalTickAssignmentValueV1::Number(
                parse_tick_expression(source, &derived).ok_or(
                    CanonicalSourceErrorV1::InvalidTickProfile {
                        origin: include[0].1,
                    },
                )?,
            ),
        });
    }
    for (target, source) in parse_vec3_components(velocity_vector)
        .ok_or(CanonicalSourceErrorV1::InvalidTickProfile { origin })?
        .into_iter()
        .enumerate()
    {
        assignments.push(CanonicalTickAssignmentV1 {
            target: CanonicalTickAssignmentTargetV1::VelocityComponent(target as u8),
            value: CanonicalTickAssignmentValueV1::Number(
                parse_tick_expression(source, &derived).ok_or(
                    CanonicalSourceErrorV1::InvalidTickProfile {
                        origin: include[1].1,
                    },
                )?,
            ),
        });
    }
    if landing_branch {
        if include[2].0 != "?player grounded true" {
            return Err(CanonicalSourceErrorV1::InvalidTickProfile {
                origin: include[2].1,
            });
        }
        assignments.push(CanonicalTickAssignmentV1 {
            target: CanonicalTickAssignmentTargetV1::Grounded,
            value: CanonicalTickAssignmentValueV1::Boolean(true),
        });
    }
    Ok(Some(TickHandlerCst {
        origin,
        producer: semantic_producer(
            CanonicalSourceProductionV1::Handler,
            &handler_semantic_producer(block),
        ),
        designation: b"tick".to_vec(),
        predicates,
        assignments,
        includes: include
            .into_iter()
            .map(|(local, origin)| HandlerIncludeCst {
                origin,
                local: local.as_bytes().to_vec(),
            })
            .collect(),
    }))
}

fn parse_tick_clamp_binding(
    source: &str,
    derived: &BTreeMap<String, CanonicalTickExpressionV1>,
) -> Option<(String, CanonicalTickExpressionV1)> {
    let (value, rest) = source.split_once(" clamped between ")?;
    let (bounds, result) = rest.split_once(" as ")?;
    let (lower, upper) = bounds.split_once(" and ")?;
    Some((
        result.to_owned(),
        CanonicalTickExpressionV1::Clamp(
            Box::new(parse_tick_expression(value, derived)?),
            Box::new(parse_tick_expression(lower, derived)?),
            Box::new(parse_tick_expression(upper, derived)?),
        ),
    ))
}

fn parse_tick_comparison(
    source: &str,
    derived: &BTreeMap<String, CanonicalTickExpressionV1>,
) -> Option<CanonicalTickPredicateV1> {
    if let Some((left, right)) = source.split_once(" <= ") {
        return Some(CanonicalTickPredicateV1::LessThanOrEqual(
            parse_tick_expression(left, derived)?,
            parse_tick_expression(right, derived)?,
        ));
    }
    let (left, right) = source.split_once(" > ")?;
    Some(CanonicalTickPredicateV1::GreaterThan(
        parse_tick_expression(left, derived)?,
        parse_tick_expression(right, derived)?,
    ))
}

fn parse_tick_expression(
    source: &str,
    derived: &BTreeMap<String, CanonicalTickExpressionV1>,
) -> Option<CanonicalTickExpressionV1> {
    let mut parser = TickExpressionParser {
        source: source.as_bytes(),
        cursor: 0,
        derived,
    };
    let expression = parser.additive()?;
    parser.skip_spaces();
    (parser.cursor == parser.source.len()).then_some(expression)
}

struct TickExpressionParser<'a> {
    source: &'a [u8],
    cursor: usize,
    derived: &'a BTreeMap<String, CanonicalTickExpressionV1>,
}

impl TickExpressionParser<'_> {
    fn additive(&mut self) -> Option<CanonicalTickExpressionV1> {
        let mut value = self.multiplicative()?;
        loop {
            self.skip_spaces();
            let operation = self.take_one(&[b'+', b'-']);
            let Some(operation) = operation else { break };
            let right = self.multiplicative()?;
            value = match operation {
                b'+' => CanonicalTickExpressionV1::Add(Box::new(value), Box::new(right)),
                b'-' => CanonicalTickExpressionV1::Subtract(Box::new(value), Box::new(right)),
                _ => unreachable!(),
            };
        }
        Some(value)
    }

    fn multiplicative(&mut self) -> Option<CanonicalTickExpressionV1> {
        let mut value = self.primary()?;
        loop {
            self.skip_spaces();
            let operation = self.take_one(&[b'*', b'/']);
            let Some(operation) = operation else { break };
            let right = self.primary()?;
            value = match operation {
                b'*' => CanonicalTickExpressionV1::Multiply(Box::new(value), Box::new(right)),
                b'/' => CanonicalTickExpressionV1::Divide(Box::new(value), Box::new(right)),
                _ => unreachable!(),
            };
        }
        Some(value)
    }

    fn primary(&mut self) -> Option<CanonicalTickExpressionV1> {
        self.skip_spaces();
        if self.source.get(self.cursor) == Some(&b'(') {
            self.cursor += 1;
            let value = self.additive()?;
            self.skip_spaces();
            (self.source.get(self.cursor) == Some(&b')')).then(|| self.cursor += 1)?;
            return Some(value);
        }
        let start = self.cursor;
        if self.source.get(self.cursor) == Some(&b'-') {
            self.cursor += 1;
        }
        while let Some(byte) = self.source.get(self.cursor)
            && !byte.is_ascii_whitespace()
            && !matches!(*byte, b'+' | b'*' | b'/' | b'(' | b')')
        {
            self.cursor += 1;
        }
        (self.cursor > start).then_some(())?;
        let atom = std::str::from_utf8(&self.source[start..self.cursor]).ok()?;
        if let Some(value) = self.derived.get(atom) {
            return Some(value.clone());
        }
        let value = match atom {
            "?dt" => CanonicalTickValueV1::DeltaTime,
            "?position-x" => CanonicalTickValueV1::PositionComponent(0),
            "?position-y" => CanonicalTickValueV1::PositionComponent(1),
            "?position-z" => CanonicalTickValueV1::PositionComponent(2),
            "?velocity-x" => CanonicalTickValueV1::VelocityComponent(0),
            "?velocity-y" => CanonicalTickValueV1::VelocityComponent(1),
            "?velocity-z" => CanonicalTickValueV1::VelocityComponent(2),
            "?intent-x" => CanonicalTickValueV1::IntentComponent(0),
            "?intent-y" => CanonicalTickValueV1::IntentComponent(1),
            "?intent-z" => CanonicalTickValueV1::IntentComponent(2),
            "?gravity" => CanonicalTickValueV1::Gravity,
            "?move-speed" => CanonicalTickValueV1::MoveSpeed,
            "?floor" => CanonicalTickValueV1::FloorHeight,
            "?min-x" => CanonicalTickValueV1::MinimumX,
            "?max-x" => CanonicalTickValueV1::MaximumX,
            "?min-z" => CanonicalTickValueV1::MinimumZ,
            "?max-z" => CanonicalTickValueV1::MaximumZ,
            _ => return parse_source_number(atom).map(CanonicalTickExpressionV1::Number),
        };
        Some(CanonicalTickExpressionV1::Value(value))
    }

    fn skip_spaces(&mut self) {
        while self
            .source
            .get(self.cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.cursor += 1;
        }
    }

    fn take_one(&mut self, accepted: &[u8]) -> Option<u8> {
        let byte = *self.source.get(self.cursor)?;
        accepted.contains(&byte).then(|| {
            self.cursor += 1;
            byte
        })
    }
}

fn parse_vector_assertion(
    line: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<Option<VectorAssertionCst>, CanonicalSourceErrorV1> {
    let Some((prefix, vector)) = split_vector_subject(line) else {
        return Ok(None);
    };
    let prefix = prefix.split_whitespace().collect::<Vec<_>>();
    if prefix.len() < 2 {
        return Ok(None);
    }
    let components = parse_vec3_components(vector)
        .ok_or(CanonicalSourceErrorV1::InvalidInputHandler { origin })?;
    let Some(x) = parse_source_number(components[0]) else {
        return Ok(None);
    };
    let Some(y) = parse_source_number(components[1]) else {
        return Ok(None);
    };
    let Some(z) = parse_source_number(components[2]) else {
        return Ok(None);
    };
    Ok(Some(VectorAssertionCst {
        origin,
        subject: prefix[0].as_bytes().to_vec(),
        relation: prefix[1..].join(" ").into_bytes(),
        x,
        y,
        z,
    }))
}

fn parse_shape_assertion(
    line: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<Option<ShapeAssertionCst>, CanonicalSourceErrorV1> {
    let Some((prefix, shape, fields)) = split_shape_subject(line) else {
        return Ok(None);
    };
    if shape == "Vec3" {
        return Ok(None);
    }
    let prefix = prefix.split_whitespace().collect::<Vec<_>>();
    if prefix.len() < 2 {
        return Ok(None);
    }
    let mut seen = BTreeSet::new();
    let fields = parse_shape_fields(fields)
        .ok_or(CanonicalSourceErrorV1::InvalidShapeField { origin })?
        .into_iter()
        .map(|(name, value)| {
            if !seen.insert(name.as_bytes().to_vec()) {
                return Err(CanonicalSourceErrorV1::InvalidShapeField { origin });
            }
            let value = if let Some(number) = parse_source_number(value) {
                CanonicalScalarValueV1::Number(number)
            } else if value == "true" {
                CanonicalScalarValueV1::Boolean(true)
            } else if value == "false" {
                CanonicalScalarValueV1::Boolean(false)
            } else if value.starts_with('"') {
                CanonicalScalarValueV1::Text(
                    parse_text_literal(value)
                        .ok_or(CanonicalSourceErrorV1::InvalidShapeField { origin })?,
                )
            } else {
                CanonicalScalarValueV1::Symbol(designation_bytes(value, origin)?)
            };
            Ok(ShapeAssertionFieldCst {
                name: designation_bytes(name, origin)?,
                value,
            })
        })
        .collect::<Result<Vec<_>, CanonicalSourceErrorV1>>()?;
    Ok(Some(ShapeAssertionCst {
        origin,
        subject: prefix[0].as_bytes().to_vec(),
        relation: prefix[1..].join(" ").into_bytes(),
        shape: designation_bytes(shape, origin)?,
        fields,
    }))
}

fn parse_boolean_clause(source: &str) -> Option<(&str, String, bool)> {
    let parts = source.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 3 {
        return None;
    }
    let value = match *parts.last()? {
        "true" => true,
        "false" => false,
        _ => return None,
    };
    Some((parts[0], parts[1..parts.len() - 1].join(" "), value))
}

fn parse_boolean_assertion(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Option<BooleanAssertionCst> {
    let (subject, relation, value) = parse_boolean_clause(source)?;
    Some(BooleanAssertionCst {
        origin,
        subject: subject.as_bytes().to_vec(),
        relation: relation.into_bytes(),
        value,
    })
}

fn parse_number_assertion(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Option<NumberAssertionCst> {
    let parts = source.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 3 {
        return None;
    }
    let value = parse_source_number(parts.last()?)?;
    Some(NumberAssertionCst {
        origin,
        subject: parts[0].as_bytes().to_vec(),
        relation: parts[1..parts.len() - 1].join(" ").into_bytes(),
        value,
    })
}

fn parse_symbol_assertion(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Option<SymbolAssertionCst> {
    if source.contains('"') {
        return None;
    }
    let parts = source.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 3 {
        return None;
    }
    let value = *parts.last()?;
    if value.starts_with('?') || parse_source_number(value).is_some() {
        return None;
    }
    designation_bytes(value, origin)
        .ok()
        .map(|value| SymbolAssertionCst {
            origin,
            subject: parts[0].as_bytes().to_vec(),
            relation: parts[1..parts.len() - 1].join(" ").into_bytes(),
            value,
        })
}

fn parse_text_assertion(source: &str, origin: CanonicalSourceOriginV1) -> Option<TextAssertionCst> {
    let quote = source.find('"')?;
    let prefix = source.get(..quote)?.strip_suffix(' ')?;
    let parts = prefix.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    Some(TextAssertionCst {
        origin,
        subject: parts[0].as_bytes().to_vec(),
        relation: parts[1..].join(" ").into_bytes(),
        value: parse_text_literal(source.get(quote..)?)?,
    })
}

fn parse_text_literal(source: &str) -> Option<String> {
    let inner = source.strip_prefix('"')?.strip_suffix('"')?;
    let mut characters = inner.chars();
    let mut value = String::new();
    while let Some(character) = characters.next() {
        match character {
            '"' => return None,
            '\\' => match characters.next()? {
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                'u' if characters.next()? == '{' => {
                    let mut digits = String::new();
                    loop {
                        let digit = characters.next()?;
                        if digit == '}' {
                            break;
                        }
                        if digits.len() == 6 || !digit.is_ascii_hexdigit() {
                            return None;
                        }
                        digits.push(digit);
                    }
                    if digits.is_empty() {
                        return None;
                    }
                    value.push(char::from_u32(u32::from_str_radix(&digits, 16).ok()?)?);
                }
                _ => return None,
            },
            character if character.is_control() => return None,
            character => value.push(character),
        }
        if value.len() > MAX_CANONICAL_TEXT_BYTES {
            return None;
        }
    }
    Some(value)
}

fn split_shape_subject(source: &str) -> Option<(&str, &str, &str)> {
    let (prefix, fields) = source.split_once(" { ")?;
    let (prefix, shape) = prefix.rsplit_once(' ')?;
    (!prefix.is_empty() && !shape.is_empty()).then_some((prefix, shape, fields))
}

fn parse_shape_fields(fields: &str) -> Option<Vec<(&str, &str)>> {
    let fields = fields.strip_suffix(" }")?;
    if fields.is_empty() {
        return None;
    }
    fields
        .split(", ")
        .map(|field| {
            let (name, value) = field.split_once(": ")?;
            (!name.is_empty() && !value.is_empty()).then_some((name, value))
        })
        .collect()
}

fn split_vector_subject(source: &str) -> Option<(&str, &str)> {
    let (prefix, shape, vector) = split_shape_subject(source)?;
    (shape == "Vec3").then_some((prefix, vector))
}

fn parse_vec3_components(vector: &str) -> Option<[&str; 3]> {
    let vector = vector.strip_suffix(" }")?;
    let mut fields = vector.split(", ");
    let x = fields.next()?.strip_prefix("x: ")?;
    let y = fields.next()?.strip_prefix("y: ")?;
    let z = fields.next()?.strip_prefix("z: ")?;
    fields.next().is_none().then_some([x, y, z])
}

fn parse_source_number(source: &str) -> Option<u64> {
    let value = source.parse::<f64>().ok()?;
    value.is_finite().then_some(value.to_bits())
}

fn parse_input_scalar(source: &str, parameters: [&str; 2]) -> Option<CanonicalInputScalarV1> {
    parameters
        .iter()
        .position(|parameter| *parameter == source)
        .and_then(|index| u16::try_from(index).ok())
        .map(CanonicalInputScalarV1::Parameter)
        .or_else(|| parse_source_number(source).map(CanonicalInputScalarV1::Number))
}

fn parse_relation(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
    designation: Vec<u8>,
) -> Result<RelationCst, CanonicalSourceErrorV1> {
    let mut roles = None;
    let mut surface = None;
    let mut reading_pattern = None;
    let mut modes: Vec<RelationModeCst> = Vec::new();
    let mut subject = None;
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let origin = line_origin(artifact, *line);
        if line.indent == 4 {
            let mode = modes
                .last_mut()
                .ok_or(CanonicalSourceErrorV1::InvalidRelationChild { origin })?;
            parse_mode_contract_child(&line.text[4..], origin, mode)?;
            continue;
        }
        if line.indent != 2 {
            return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
        }
        let child = &line.text[2..];
        if let Some(reading) = child.strip_prefix("reads ") {
            if roles.is_some() {
                return Err(CanonicalSourceErrorV1::DuplicateChild {
                    producer: designation.clone(),
                    child: b"reads".to_vec(),
                });
            }
            roles = Some(parse_reads_roles(reading, origin)?);
            surface = Some(parse_reading_surface(reading, origin)?);
            reading_pattern = Some(parse_relation_reading(reading, origin)?);
        } else if let Some(role) = child.strip_prefix("subject ") {
            if subject.replace(role.as_bytes().to_vec()).is_some() {
                return Err(CanonicalSourceErrorV1::DuplicateChild {
                    producer: designation.clone(),
                    child: b"subject".to_vec(),
                });
            }
        } else if let Some(mode) = child.strip_prefix("mode ") {
            modes.push(parse_mode(mode, origin)?);
        } else {
            return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
        }
    }
    let roles = roles.ok_or_else(|| CanonicalSourceErrorV1::MissingRelationReads {
        designation: designation.clone(),
    })?;
    if modes.is_empty() {
        return Err(CanonicalSourceErrorV1::MissingRelationMode { designation });
    }
    ensure_unique_children(&designation, roles.iter().map(|role| role.name.as_slice()))?;
    let declared = roles
        .iter()
        .map(|role| role.name.as_slice())
        .collect::<BTreeSet<_>>();
    if let Some(ref subject) = subject
        && !declared.contains(subject.as_slice())
    {
        return Err(CanonicalSourceErrorV1::UnknownSubjectRole {
            designation,
            role: subject.clone(),
        });
    }
    for mode in &modes {
        for role in mode.known.iter().chain(&mode.produced) {
            if !declared.contains(role.as_slice()) {
                return Err(CanonicalSourceErrorV1::UnknownModeRole {
                    designation: designation.clone(),
                    role: role.clone(),
                });
            }
        }
        let closed = mode
            .known
            .iter()
            .chain(&mode.produced)
            .map(Vec::as_slice)
            .collect::<BTreeSet<_>>();
        if closed != declared {
            return Err(CanonicalSourceErrorV1::InvalidMode {
                origin: mode.origin,
            });
        }
        if mode.reactive_obligation.is_some() != mode.continues_linearly
            || (mode.effect.is_some() && !mode.continues_linearly)
        {
            return Err(CanonicalSourceErrorV1::InvalidMode {
                origin: mode.origin,
            });
        }
        if let Some(effect) = &mode.effect {
            let known = mode
                .known
                .iter()
                .map(Vec::as_slice)
                .collect::<BTreeSet<_>>();
            if [
                effect.action_role.as_slice(),
                effect.resource_role.as_slice(),
                effect.payload_role.as_slice(),
            ]
            .into_iter()
            .any(|role| !known.contains(role))
            {
                return Err(CanonicalSourceErrorV1::InvalidMode {
                    origin: mode.origin,
                });
            }
        }
    }
    ensure_unique_children(
        &designation,
        modes.iter().map(|mode| mode.canonical.as_slice()),
    )?;
    Ok(RelationCst {
        designation,
        surface: surface.expect("a parsed Reading has one surface phrase"),
        reading: reading_pattern.expect("a parsed Reading has one role pattern"),
        subject,
        roles,
        modes,
    })
}

fn parse_relation_reading(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<RelationReadingPartCst>, CanonicalSourceErrorV1> {
    let mut rest = source;
    let mut parts = Vec::new();
    loop {
        let Some(open) = rest.find('{') else {
            parts.extend(
                rest.split_whitespace()
                    .map(|literal| RelationReadingPartCst::Literal(literal.as_bytes().to_vec())),
            );
            break;
        };
        parts.extend(
            rest[..open]
                .split_whitespace()
                .map(|literal| RelationReadingPartCst::Literal(literal.as_bytes().to_vec())),
        );
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
        };
        let Some((role, _)) = after[..close].split_once(": ") else {
            return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
        };
        parts.push(RelationReadingPartCst::Role(role.as_bytes().to_vec()));
        rest = &after[close + 1..];
    }
    if parts.is_empty() {
        return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
    }
    Ok(parts)
}

fn parse_reading_surface(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<u8>, CanonicalSourceErrorV1> {
    let mut rest = source;
    let mut literal = String::new();
    while let Some(open) = rest.find('{') {
        literal.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
        };
        rest = &after[close + 1..];
    }
    literal.push_str(rest);
    let normalized = literal.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
    }
    Ok(normalized.into_bytes())
}

fn parse_reads_roles(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<RelationRoleCst>, CanonicalSourceErrorV1> {
    let mut rest = source;
    let mut roles = Vec::new();
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
        };
        let Some((name, domain)) = after[..close].split_once(": ") else {
            return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
        };
        if name.is_empty() || domain.is_empty() || name.contains('=') || domain.contains('=') {
            return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
        }
        roles.push(RelationRoleCst {
            name: name.as_bytes().to_vec(),
            domain: domain.as_bytes().to_vec(),
            origin,
        });
        rest = &after[close + 1..];
    }
    if roles.is_empty() || rest.contains('}') {
        return Err(CanonicalSourceErrorV1::InvalidRelationChild { origin });
    }
    Ok(roles)
}

fn parse_mode(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<RelationModeCst, CanonicalSourceErrorV1> {
    let Some(source) = source.strip_prefix("given ") else {
        return Err(CanonicalSourceErrorV1::InvalidMode { origin });
    };
    let Some((orientation, cardinality)) = source.rsplit_once(": ") else {
        return Err(CanonicalSourceErrorV1::InvalidMode { origin });
    };
    let Some((known, produced)) = orientation.split_once(" yields ") else {
        return Err(CanonicalSourceErrorV1::InvalidMode { origin });
    };
    let split = |value: &str| {
        value
            .split_whitespace()
            .map(|role| role.as_bytes().to_vec())
            .collect::<Vec<_>>()
    };
    let known = split(known);
    let produced = split(produced);
    if known.is_empty() || produced.is_empty() {
        return Err(CanonicalSourceErrorV1::InvalidMode { origin });
    }
    let cardinality = match cardinality {
        "one" => SourceCardinality::One,
        "maybe" => SourceCardinality::Maybe,
        "some" => SourceCardinality::Some,
        "many" => SourceCardinality::Many,
        _ => return Err(CanonicalSourceErrorV1::InvalidMode { origin }),
    };
    Ok(RelationModeCst {
        known,
        produced,
        cardinality,
        reactive_obligation: None,
        continues_linearly: false,
        effect: None,
        canonical: source.as_bytes().to_vec(),
        origin,
    })
}

fn parse_mode_contract_child(
    source: &str,
    origin: CanonicalSourceOriginV1,
    mode: &mut RelationModeCst,
) -> Result<(), CanonicalSourceErrorV1> {
    if let Some(obligation) = source.strip_prefix("reactive while ") {
        if mode.reactive_obligation.is_some() {
            return Err(CanonicalSourceErrorV1::InvalidMode { origin });
        }
        mode.reactive_obligation = Some(designation_bytes(obligation, origin)?);
    } else if source == "continues linearly" {
        if mode.continues_linearly {
            return Err(CanonicalSourceErrorV1::InvalidMode { origin });
        }
        mode.continues_linearly = true;
    } else {
        let parts = source.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 8
            || parts[0] != "effect"
            || parts[2] != "resource"
            || parts[4] != "payload"
            || parts[6] != "requires"
            || mode.effect.is_some()
        {
            return Err(CanonicalSourceErrorV1::InvalidMode { origin });
        }
        mode.effect = Some(RelationEffectCst {
            action_role: designation_bytes(parts[1], origin)?,
            resource_role: designation_bytes(parts[3], origin)?,
            payload_role: designation_bytes(parts[5], origin)?,
            capability: designation_bytes(parts[7], origin)?,
        });
    }
    mode.canonical.push(b'\n');
    mode.canonical.extend_from_slice(source.as_bytes());
    Ok(())
}

fn handler_include_emissions(
    artifact: CanonicalSourceArtifactIdV1,
    block: &[SourceLine<'_>],
) -> Result<Vec<CanonicalSourceEmissionV1>, CanonicalSourceErrorV1> {
    let producer = semantic_producer(
        CanonicalSourceProductionV1::Handler,
        &handler_semantic_producer(block),
    );
    let mut in_include = false;
    let mut seen = BTreeSet::new();
    let mut emissions = Vec::new();
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let trimmed = line.text.trim();
        if line.indent == 2 {
            if trimmed == "admit" {
                return Err(CanonicalSourceErrorV1::NonCanonicalKeyword {
                    origin: line_origin(artifact, *line),
                    keyword: b"admit".to_vec(),
                });
            }
            in_include = trimmed == "include";
            continue;
        }
        if line.indent == 4 && in_include {
            let local = trimmed.as_bytes().to_vec();
            let slot = child_slot(CanonicalSourceProductionV1::HandlerInclude, &local);
            if !seen.insert(slot.clone()) {
                return Err(CanonicalSourceErrorV1::RepeatedEmissionNeedsPlan { slot });
            }
            emissions.push(CanonicalSourceEmissionV1 {
                producer: producer.clone(),
                slot,
                origin: line_origin(artifact, *line),
                allocations: vec![],
            });
        }
    }
    Ok(emissions)
}

fn membership_group_emissions(
    artifact: CanonicalSourceArtifactIdV1,
    line: SourceLine<'_>,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<CanonicalSourceEmissionV1>, CanonicalSourceErrorV1> {
    let source = line.text;
    let mut membership_offsets = source.match_indices('∈');
    let Some((operator_offset, _)) = membership_offsets.next() else {
        return Err(CanonicalSourceErrorV1::InvalidMembershipGroup { origin });
    };
    if membership_offsets.next().is_some() {
        return Err(CanonicalSourceErrorV1::InvalidMembershipGroup { origin });
    }

    let left = &source[..operator_offset];
    let right_offset = operator_offset + '∈'.len_utf8();
    let right = &source[right_offset..];
    if !left.ends_with(' ') || !right.starts_with(' ') {
        return Err(CanonicalSourceErrorV1::InvalidMembershipGroup { origin });
    }
    let subject = left.trim_end_matches(' ');
    let subject_bytes = membership_designation_bytes(subject, origin)?;

    let producer = assertion_producer(&subject_bytes, "∈".as_bytes());
    let mut repetitions = BTreeMap::<Vec<u8>, u64>::new();
    let mut emissions = Vec::new();
    let mut segment_offset = 0;
    for segment in right.split(',') {
        let leading = segment.len() - segment.trim_start_matches(' ').len();
        let target = segment.trim_matches(' ');
        if target.is_empty() {
            return Err(CanonicalSourceErrorV1::InvalidMembershipGroup { origin });
        }
        let target_bytes = membership_designation_bytes(target, origin)?;
        let occurrence = repetitions.entry(target_bytes.clone()).or_default();
        let repetition = (*occurrence > 0).then_some(*occurrence);
        *occurrence = occurrence
            .checked_add(1)
            .ok_or(CanonicalSourceErrorV1::InvalidMembershipGroup { origin })?;
        let target_start = line
            .start
            .checked_add(right_offset)
            .and_then(|start| start.checked_add(segment_offset))
            .and_then(|start| start.checked_add(leading))
            .ok_or(CanonicalSourceErrorV1::InvalidMembershipGroup { origin })?;
        let target_end = target_start
            .checked_add(target.len())
            .ok_or(CanonicalSourceErrorV1::InvalidMembershipGroup { origin })?;
        emissions.push(CanonicalSourceEmissionV1 {
            producer: producer.clone(),
            slot: CanonicalEmissionSlotV1 {
                production: CanonicalSourceProductionV1::Assertion,
                local: target_bytes,
                repetition,
            },
            origin: CanonicalSourceOriginV1 {
                artifact,
                start: target_start as u64,
                end: target_end as u64,
            },
            allocations: vec![],
        });
        segment_offset = segment_offset
            .checked_add(segment.len())
            .and_then(|offset| offset.checked_add(1))
            .ok_or(CanonicalSourceErrorV1::InvalidMembershipGroup { origin })?;
    }
    if emissions.is_empty() {
        return Err(CanonicalSourceErrorV1::InvalidMembershipGroup { origin });
    }
    Ok(emissions)
}

fn parse_membership_group(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<(Vec<u8>, Vec<Vec<u8>>), CanonicalSourceErrorV1> {
    let (subject, domains) = source
        .split_once(" ∈ ")
        .ok_or(CanonicalSourceErrorV1::InvalidMembershipGroup { origin })?;
    let subject = membership_designation_bytes(subject, origin)?;
    let domains = domains
        .split(',')
        .map(|domain| membership_designation_bytes(domain.trim(), origin))
        .collect::<Result<Vec<_>, _>>()?;
    if domains.is_empty() {
        return Err(CanonicalSourceErrorV1::InvalidMembershipGroup { origin });
    }
    Ok((subject, domains))
}

fn membership_designation_bytes(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<u8>, CanonicalSourceErrorV1> {
    let bytes = source.as_bytes();
    let valid = bytes
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && bytes
            .iter()
            .skip(1)
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-'));
    if !valid {
        return Err(CanonicalSourceErrorV1::InvalidMembershipGroup { origin });
    }
    Ok(bytes.to_vec())
}

fn handler_semantic_producer(block: &[SourceLine<'_>]) -> Vec<u8> {
    let mut producer = Vec::new();
    frame_bytes(&mut producer, block[0].text.as_bytes());
    let mut in_when = false;
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let trimmed = line.text.trim();
        if line.indent == 2 {
            in_when = trimmed == "when";
        } else if line.indent == 4 && in_when {
            frame_bytes(&mut producer, trimmed.as_bytes());
        }
    }
    producer
}

fn input_handler_parts(
    cst: &CanonicalSourceCstV1,
) -> Result<Option<(&InputHandlerCst, &VectorAssertionCst)>, CanonicalSourceErrorV1> {
    let handlers = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::InputHandler(handler) => Some(handler),
            _ => None,
        })
        .collect::<Vec<_>>();
    let Some(handler) = handlers.first().copied() else {
        return Ok(None);
    };
    if handlers.len() != 1 {
        return Err(CanonicalSourceErrorV1::InvalidInputHandler {
            origin: handler.origin,
        });
    }
    let assertions = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::VectorAssertion(assertion) if assertion.relation == handler.relation => {
                Some(assertion)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    match assertions.as_slice() {
        [assertion] => Ok(Some((handler, *assertion))),
        [] => Err(CanonicalSourceErrorV1::MissingInputInitialAssertion {
            origin: handler.origin,
        }),
        _ => Err(CanonicalSourceErrorV1::AmbiguousInputInitialAssertion {
            origin: handler.origin,
        }),
    }
}

fn jump_handler_parts(
    cst: &CanonicalSourceCstV1,
) -> Result<Option<JumpHandlerParts<'_>>, CanonicalSourceErrorV1> {
    let handlers = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::JumpHandler(handler) => Some(handler),
            _ => None,
        })
        .collect::<Vec<_>>();
    let Some(handler) = handlers.first().copied() else {
        return Ok(None);
    };
    if handlers.len() != 1 {
        return Err(CanonicalSourceErrorV1::InvalidJumpHandler {
            origin: handler.origin,
        });
    }
    let velocities = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::VectorAssertion(assertion)
                if assertion.relation == handler.velocity_relation =>
            {
                Some(assertion)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let grounded = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::BooleanAssertion(assertion)
                if assertion.relation == handler.grounded_relation =>
            {
                Some(assertion)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let jump_speeds = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::NumberAssertion(assertion)
                if assertion.subject == handler.jump_speed_subject
                    && assertion.relation == handler.jump_speed_relation =>
            {
                Some(assertion)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let ([velocity], [grounded], [jump_speed]) = (
        velocities.as_slice(),
        grounded.as_slice(),
        jump_speeds.as_slice(),
    ) else {
        let missing = velocities.is_empty() || grounded.is_empty() || jump_speeds.is_empty();
        return Err(if missing {
            CanonicalSourceErrorV1::MissingJumpInitialAssertion {
                origin: handler.origin,
            }
        } else {
            CanonicalSourceErrorV1::AmbiguousJumpInitialAssertion {
                origin: handler.origin,
            }
        });
    };
    if velocity.subject != grounded.subject {
        return Err(CanonicalSourceErrorV1::InvalidJumpHandler {
            origin: handler.origin,
        });
    }
    Ok(Some(JumpHandlerParts {
        handler,
        velocity,
        grounded,
        jump_speed,
    }))
}

fn scalar_handler_parts(
    cst: &CanonicalSourceCstV1,
) -> Result<Vec<ScalarHandlerParts<'_>>, CanonicalSourceErrorV1> {
    let handlers = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::ScalarHandler(handler) => Some(handler),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut parts = Vec::with_capacity(handlers.len());
    for handler in handlers {
        let assertions = cst
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                CstKind::VectorAssertion(assertion) if assertion.relation == handler.relation => {
                    let value = match handler.field.as_deref()? {
                        b"x" => assertion.x,
                        b"y" => assertion.y,
                        b"z" => assertion.z,
                        _ => return None,
                    };
                    Some((assertion.origin, CanonicalScalarValueV1::Number(value)))
                }
                CstKind::NumberAssertion(assertion) if assertion.relation == handler.relation => {
                    Some((
                        assertion.origin,
                        CanonicalScalarValueV1::Number(assertion.value),
                    ))
                }
                CstKind::BooleanAssertion(assertion) if assertion.relation == handler.relation => {
                    Some((
                        assertion.origin,
                        CanonicalScalarValueV1::Boolean(assertion.value),
                    ))
                }
                CstKind::SymbolAssertion(assertion) if assertion.relation == handler.relation => {
                    Some((
                        assertion.origin,
                        CanonicalScalarValueV1::Symbol(assertion.value.clone()),
                    ))
                }
                CstKind::TextAssertion(assertion) if assertion.relation == handler.relation => {
                    Some((
                        assertion.origin,
                        CanonicalScalarValueV1::Text(assertion.value.clone()),
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        match assertions.as_slice() {
            [(initial_origin, initial_value)]
                if scalar_expression_matches_value(&handler.result, initial_value) =>
            {
                parts.push(ScalarHandlerParts {
                    handler,
                    initial_origin: *initial_origin,
                    initial_value: initial_value.clone(),
                });
            }
            [_] => {
                return Err(CanonicalSourceErrorV1::InvalidScalarHandler {
                    origin: handler.origin,
                });
            }
            [] => {
                return Err(CanonicalSourceErrorV1::MissingScalarInitialAssertion {
                    origin: handler.origin,
                });
            }
            _ => {
                return Err(CanonicalSourceErrorV1::AmbiguousScalarInitialAssertion {
                    origin: handler.origin,
                });
            }
        }
    }
    Ok(parts)
}

fn scalar_expression_matches_value(
    expression: &CanonicalScalarExpressionV1,
    initial: &CanonicalScalarValueV1,
) -> bool {
    match expression {
        CanonicalScalarExpressionV1::Current => true,
        CanonicalScalarExpressionV1::Parameter(_) => true,
        CanonicalScalarExpressionV1::Number(_) => {
            matches!(initial, CanonicalScalarValueV1::Number(_))
        }
        CanonicalScalarExpressionV1::Boolean(_) => {
            matches!(initial, CanonicalScalarValueV1::Boolean(_))
        }
        CanonicalScalarExpressionV1::Symbol(_) => {
            matches!(initial, CanonicalScalarValueV1::Symbol(_))
        }
        CanonicalScalarExpressionV1::Text(_) => {
            matches!(initial, CanonicalScalarValueV1::Text(_))
        }
        CanonicalScalarExpressionV1::Concatenate(left, right) => {
            matches!(initial, CanonicalScalarValueV1::Text(_))
                && scalar_expression_matches_value(left, initial)
                && scalar_expression_matches_value(right, initial)
        }
        CanonicalScalarExpressionV1::Add(left, right)
        | CanonicalScalarExpressionV1::Subtract(left, right)
        | CanonicalScalarExpressionV1::Multiply(left, right)
        | CanonicalScalarExpressionV1::Divide(left, right) => {
            matches!(initial, CanonicalScalarValueV1::Number(_))
                && scalar_expression_matches_value(left, initial)
                && scalar_expression_matches_value(right, initial)
        }
        CanonicalScalarExpressionV1::Clamp(value, lower, upper) => {
            matches!(initial, CanonicalScalarValueV1::Number(_))
                && scalar_expression_matches_value(value, initial)
                && scalar_expression_matches_value(lower, initial)
                && scalar_expression_matches_value(upper, initial)
        }
    }
}

fn tick_program_parts(
    cst: &CanonicalSourceCstV1,
) -> Result<Option<TickProgramParts<'_>>, CanonicalSourceErrorV1> {
    let handlers = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::TickHandler(handler) => Some(handler),
            _ => None,
        })
        .collect::<Vec<_>>();
    if handlers.is_empty() {
        return Ok(None);
    }
    let origin = handlers[0].origin;
    if handlers.len() != 3 {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    }
    let grounded = handlers
        .iter()
        .copied()
        .filter(|handler| {
            handler.predicates.iter().any(|predicate| {
                matches!(
                    predicate,
                    CanonicalTickPredicateV1::EqualBoolean(CanonicalTickValueV1::Grounded, true)
                )
            })
        })
        .collect::<Vec<_>>();
    let landing = handlers
        .iter()
        .copied()
        .filter(|handler| {
            handler.assignments.iter().any(|assignment| {
                matches!(
                    assignment,
                    CanonicalTickAssignmentV1 {
                        target: CanonicalTickAssignmentTargetV1::Grounded,
                        value: CanonicalTickAssignmentValueV1::Boolean(true),
                    }
                )
            })
        })
        .collect::<Vec<_>>();
    let airborne = handlers
        .iter()
        .copied()
        .filter(|handler| {
            handler.predicates.iter().any(|predicate| {
                matches!(
                    predicate,
                    CanonicalTickPredicateV1::EqualBoolean(CanonicalTickValueV1::Grounded, false)
                )
            }) && !handler.assignments.iter().any(|assignment| {
                matches!(assignment.target, CanonicalTickAssignmentTargetV1::Grounded)
            })
        })
        .collect::<Vec<_>>();
    let ([grounded], [airborne], [landing]) =
        (grounded.as_slice(), airborne.as_slice(), landing.as_slice())
    else {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    };

    let mut laws = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::ClampLaw(law) => Some(law),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut derives = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::ClampDerive(derive) => Some(derive),
            _ => None,
        })
        .collect::<Vec<_>>();
    laws.sort_by_key(|law| law.branch);
    derives.sort_by_key(|derive| derive.branch);
    if laws.iter().map(|law| law.branch).ne([
        ClampBranchV1::Lower,
        ClampBranchV1::Interior,
        ClampBranchV1::Upper,
    ]) || derives.iter().map(|derive| derive.branch).ne([
        ClampBranchV1::Lower,
        ClampBranchV1::Interior,
        ClampBranchV1::Upper,
    ]) || laws
        .iter()
        .map(|law| law.designation.as_slice())
        .ne(derives.iter().map(|derive| derive.designation.as_slice()))
    {
        return Err(CanonicalSourceErrorV1::InvalidTickProfile { origin });
    }
    let [lower_law, interior_law, upper_law] = laws.as_slice() else {
        unreachable!()
    };
    let [lower_derive, interior_derive, upper_derive] = derives.as_slice() else {
        unreachable!()
    };

    let vectors = |relation: &[u8]| {
        cst.items
            .iter()
            .filter_map(|item| match &item.kind {
                CstKind::VectorAssertion(assertion) if assertion.relation == relation => {
                    Some(assertion)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let booleans = |relation: &[u8]| {
        cst.items
            .iter()
            .filter_map(|item| match &item.kind {
                CstKind::BooleanAssertion(assertion) if assertion.relation == relation => {
                    Some(assertion)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let numbers = |relation: &[u8]| {
        cst.items
            .iter()
            .filter_map(|item| match &item.kind {
                CstKind::NumberAssertion(assertion)
                    if assertion.subject == b"jump-arena" && assertion.relation == relation =>
                {
                    Some(assertion)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let position = vectors(b"position");
    let velocity = vectors(b"velocity");
    let intent = vectors(b"horizontal intent");
    let grounded_assertions = booleans(b"grounded");
    let gravity = numbers(b"gravity");
    let move_speed = cst
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            CstKind::NumberAssertion(assertion) if assertion.relation == b"move speed" => {
                Some(assertion)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let floor_height = numbers(b"floor height");
    let minimum_x = numbers(b"minimum x");
    let maximum_x = numbers(b"maximum x");
    let minimum_z = numbers(b"minimum z");
    let maximum_z = numbers(b"maximum z");
    let (
        [position],
        [velocity],
        [intent],
        [grounded_assertion],
        [gravity],
        [move_speed],
        [floor_height],
        [minimum_x],
        [maximum_x],
        [minimum_z],
        [maximum_z],
    ) = (
        position.as_slice(),
        velocity.as_slice(),
        intent.as_slice(),
        grounded_assertions.as_slice(),
        gravity.as_slice(),
        move_speed.as_slice(),
        floor_height.as_slice(),
        minimum_x.as_slice(),
        maximum_x.as_slice(),
        minimum_z.as_slice(),
        maximum_z.as_slice(),
    )
    else {
        let missing = [
            position.len(),
            velocity.len(),
            intent.len(),
            grounded_assertions.len(),
            gravity.len(),
            move_speed.len(),
            floor_height.len(),
            minimum_x.len(),
            maximum_x.len(),
            minimum_z.len(),
            maximum_z.len(),
        ]
        .contains(&0);
        return Err(if missing {
            CanonicalSourceErrorV1::MissingTickInitialAssertion { origin }
        } else {
            CanonicalSourceErrorV1::AmbiguousTickInitialAssertion { origin }
        });
    };
    Ok(Some(TickProgramParts {
        handlers: [*grounded, *airborne, *landing],
        laws: [*lower_law, *interior_law, *upper_law],
        derives: [*lower_derive, *interior_derive, *upper_derive],
        position,
        velocity,
        intent,
        grounded: grounded_assertion,
        gravity,
        move_speed,
        floor_height,
        minimum_x,
        maximum_x,
        minimum_z,
        maximum_z,
    }))
}

fn assertion_producer(subject: &[u8], relation: &[u8]) -> CanonicalSemanticProducerV1 {
    let mut key = Vec::new();
    frame_bytes(&mut key, subject);
    frame_bytes(&mut key, relation);
    semantic_producer(CanonicalSourceProductionV1::Assertion, &key)
}

fn frame_bytes(target: &mut Vec<u8>, value: &[u8]) {
    let length = u32::try_from(value.len()).expect("one source line length fits u32");
    target.extend_from_slice(&length.to_be_bytes());
    target.extend_from_slice(value);
}

fn unsupported_item(
    _artifact: CanonicalSourceArtifactIdV1,
    _block: &[SourceLine<'_>],
    origin: CanonicalSourceOriginV1,
    production: CanonicalSourceProductionV1,
    emissions: Vec<CanonicalSourceEmissionV1>,
) -> Result<CstItem, CanonicalSourceErrorV1> {
    Ok(CstItem {
        origin,
        kind: CstKind::Unsupported(CanonicalUnsupportedProductionV1 {
            production,
            origin,
            emissions,
        }),
    })
}

fn require_leaf(
    block: &[SourceLine<'_>],
    artifact: CanonicalSourceArtifactIdV1,
) -> Result<(), CanonicalSourceErrorV1> {
    if let Some(line) = block
        .iter()
        .skip(1)
        .find(|line| !line.text.trim().is_empty())
    {
        return Err(CanonicalSourceErrorV1::UnexpectedIndentation {
            origin: line_origin(artifact, *line),
        });
    }
    Ok(())
}

fn retain_supported_boolean_derive_pairs(items: &mut [CstItem]) {
    let mut counts = BTreeMap::<Vec<u8>, (usize, usize)>::new();
    for item in items.iter() {
        match &item.kind {
            CstKind::BooleanLaw(law) => counts.entry(law.designation.clone()).or_default().0 += 1,
            CstKind::BooleanDerive(derive) => {
                counts.entry(derive.designation.clone()).or_default().1 += 1;
            }
            _ => {}
        }
    }
    for item in items {
        let unsupported = match &item.kind {
            CstKind::BooleanLaw(law) if counts.get(&law.designation).copied() != Some((1, 1)) => {
                Some(CanonicalSourceProductionV1::Law)
            }
            CstKind::BooleanDerive(derive)
                if counts.get(&derive.designation).copied() != Some((1, 1)) =>
            {
                Some(CanonicalSourceProductionV1::Derive)
            }
            _ => None,
        };
        if let Some(production) = unsupported {
            item.kind = CstKind::Unsupported(CanonicalUnsupportedProductionV1 {
                production,
                origin: item.origin,
                emissions: vec![],
            });
        }
    }
}

fn validate_unique_designations(items: &[CstItem]) -> Result<(), CanonicalSourceErrorV1> {
    let mut seen = BTreeSet::new();
    for designation in items.iter().filter_map(|item| match &item.kind {
        CstKind::Referent { designation }
        | CstKind::Capability { designation }
        | CstKind::Shape { designation, .. } => Some(designation),
        CstKind::Membership(membership) => Some(&membership.subject),
        CstKind::Relation(relation) => Some(&relation.designation),
        CstKind::InputHandler(_)
        | CstKind::JumpHandler(_)
        | CstKind::ScalarHandler(_)
        | CstKind::GeneralHandler(_)
        | CstKind::TickHandler(_)
        | CstKind::KeyboardBinding(_)
        | CstKind::ScalarInputBinding(_)
        | CstKind::ClampLaw(_)
        | CstKind::ClampDerive(_)
        | CstKind::BooleanLaw(_)
        | CstKind::BooleanDerive(_)
        | CstKind::VectorAssertion(_)
        | CstKind::ShapeAssertion(_)
        | CstKind::BooleanAssertion(_)
        | CstKind::NumberAssertion(_)
        | CstKind::SymbolAssertion(_)
        | CstKind::TextAssertion(_)
        | CstKind::Unsupported(_) => None,
    }) {
        if !seen.insert(designation.clone()) {
            return Err(CanonicalSourceErrorV1::DuplicateDesignation {
                designation: designation.clone(),
            });
        }
    }
    Ok(())
}

fn validate_unique_allocations(
    allocations: &[CanonicalAllocationV1],
) -> Result<(), CanonicalSourceErrorV1> {
    let mut seen = BTreeMap::new();
    for allocation in allocations {
        let CanonicalAllocationJudgmentV1::Fresh { producer, slot, .. } = &allocation.judgment;
        if let Some((first_producer, first_slot)) =
            seen.insert(allocation.identity, (producer.clone(), slot.clone()))
        {
            return Err(CanonicalSourceErrorV1::AllocationCollision {
                identity: allocation.identity,
                first_producer,
                first_slot,
                second_producer: producer.clone(),
                second_slot: slot.clone(),
            });
        }
    }
    Ok(())
}

fn ensure_unique_children<'a>(
    producer: &[u8],
    children: impl Iterator<Item = &'a [u8]>,
) -> Result<(), CanonicalSourceErrorV1> {
    let mut seen = BTreeSet::new();
    for child in children {
        if !seen.insert(child.to_vec()) {
            return Err(CanonicalSourceErrorV1::DuplicateChild {
                producer: producer.to_vec(),
                child: child.to_vec(),
            });
        }
    }
    Ok(())
}

fn designation_bytes(
    source: &str,
    origin: CanonicalSourceOriginV1,
) -> Result<Vec<u8>, CanonicalSourceErrorV1> {
    if source.is_empty() || source.contains(char::is_whitespace) || source.contains('/') {
        return Err(CanonicalSourceErrorV1::EmptyDesignation { origin });
    }
    Ok(source.as_bytes().to_vec())
}

fn line_origin(
    artifact: CanonicalSourceArtifactIdV1,
    line: SourceLine<'_>,
) -> CanonicalSourceOriginV1 {
    CanonicalSourceOriginV1 {
        artifact,
        start: line.start as u64,
        end: line.end as u64,
    }
}

fn semantic_producer(
    production: CanonicalSourceProductionV1,
    semantic_key: &[u8],
) -> CanonicalSemanticProducerV1 {
    CanonicalSemanticProducerV1 {
        production,
        semantic_key: semantic_key.to_vec(),
    }
}

fn head_slot(production: CanonicalSourceProductionV1) -> CanonicalEmissionSlotV1 {
    CanonicalEmissionSlotV1 {
        production,
        local: b"declaration".to_vec(),
        repetition: None,
    }
}

fn child_slot(production: CanonicalSourceProductionV1, local: &[u8]) -> CanonicalEmissionSlotV1 {
    CanonicalEmissionSlotV1 {
        production,
        local: local.to_vec(),
        repetition: None,
    }
}

fn allocation(
    plan: &CanonicalSourceAllocationPlanV1,
    producer: &CanonicalSemanticProducerV1,
    slot: &CanonicalEmissionSlotV1,
    domain: AllocationDomain,
) -> Result<CanonicalAllocatedIdentityV1, CanonicalSourceErrorV1> {
    plan.identity(producer, slot, domain)
        .ok_or_else(|| CanonicalSourceErrorV1::MissingAllocation {
            slot: slot.clone(),
            domain: domain.label(),
        })
}

fn formation_id(
    plan: &CanonicalSourceAllocationPlanV1,
    producer: &CanonicalSemanticProducerV1,
    slot: &CanonicalEmissionSlotV1,
) -> Result<FormationLocalId, CanonicalSourceErrorV1> {
    match allocation(plan, producer, slot, AllocationDomain::Formation)? {
        CanonicalAllocatedIdentityV1::Formation(id) => Ok(id),
        _ => unreachable!(),
    }
}

fn capability_id(
    plan: &CanonicalSourceAllocationPlanV1,
    producer: &CanonicalSemanticProducerV1,
    slot: &CanonicalEmissionSlotV1,
) -> Result<CapabilityLocalId, CanonicalSourceErrorV1> {
    match allocation(plan, producer, slot, AllocationDomain::Capability)? {
        CanonicalAllocatedIdentityV1::Capability(id) => Ok(id),
        _ => unreachable!(),
    }
}

fn schema_id(
    plan: &CanonicalSourceAllocationPlanV1,
    producer: &CanonicalSemanticProducerV1,
    slot: &CanonicalEmissionSlotV1,
) -> Result<RelationSchemaLocalId, CanonicalSourceErrorV1> {
    match allocation(plan, producer, slot, AllocationDomain::RelationSchema)? {
        CanonicalAllocatedIdentityV1::RelationSchema(id) => Ok(id),
        _ => unreachable!(),
    }
}

fn operator_id(
    plan: &CanonicalSourceAllocationPlanV1,
    producer: &CanonicalSemanticProducerV1,
    slot: &CanonicalEmissionSlotV1,
) -> Result<OperatorLocalId, CanonicalSourceErrorV1> {
    match allocation(plan, producer, slot, AllocationDomain::Operator)? {
        CanonicalAllocatedIdentityV1::Operator(id) => Ok(id),
        _ => unreachable!(),
    }
}

fn role_id(
    plan: &CanonicalSourceAllocationPlanV1,
    producer: &CanonicalSemanticProducerV1,
    slot: &CanonicalEmissionSlotV1,
) -> Result<LocalRoleRefV2, CanonicalSourceErrorV1> {
    match allocation(plan, producer, slot, AllocationDomain::Role)? {
        CanonicalAllocatedIdentityV1::Role(id) => Ok(id),
        _ => unreachable!(),
    }
}

fn mode_id(
    plan: &CanonicalSourceAllocationPlanV1,
    producer: &CanonicalSemanticProducerV1,
    slot: &CanonicalEmissionSlotV1,
) -> Result<LocalModeRefV2, CanonicalSourceErrorV1> {
    match allocation(plan, producer, slot, AllocationDomain::Mode)? {
        CanonicalAllocatedIdentityV1::Mode(id) => Ok(id),
        _ => unreachable!(),
    }
}

fn emission(
    plan: &CanonicalSourceAllocationPlanV1,
    producer: CanonicalSemanticProducerV1,
    slot: CanonicalEmissionSlotV1,
    origin: CanonicalSourceOriginV1,
) -> CanonicalSourceEmissionV1 {
    let allocations = plan
        .allocations
        .iter()
        .filter(|allocation| {
            let CanonicalAllocationJudgmentV1::Fresh {
                producer: actual_producer,
                slot: CanonicalAllocationSlotV1::Emission(actual_slot),
                ..
            } = &allocation.judgment;
            actual_producer == &producer && actual_slot == &slot
        })
        .cloned()
        .collect();
    CanonicalSourceEmissionV1 {
        producer,
        slot,
        origin,
        allocations,
    }
}

fn source_formation(
    scope: TermScope,
    id: FormationLocalId,
    source: &[u8],
    origin: CanonicalSourceOriginV1,
    kind: &str,
) -> Result<FormationJudgmentPreimageV2, CanonicalSourceErrorV1> {
    Ok(FormationJudgmentPreimageV2 {
        id,
        context: vec![origin_term(scope, origin)?],
        term: source_term(scope, source)?,
        target: target(
            scope,
            format!("clause/source-{kind}-type-v1").as_bytes(),
            b"closed",
        )?,
        direct_dependencies: vec![],
    })
}

fn source_term(scope: TermScope, exact_source: &[u8]) -> Result<Term, TermError> {
    Term::atom(
        scope,
        b"clause/canonical-source-slice-v1".to_vec(),
        exact_source.to_vec(),
        EqualityContract::ExactOctetsV1,
    )
}

fn origin_term(scope: TermScope, origin: CanonicalSourceOriginV1) -> Result<Term, TermError> {
    let mut payload = Vec::with_capacity(IDENTITY_BYTES + 16);
    payload.extend_from_slice(origin.artifact.as_bytes());
    payload.extend_from_slice(&origin.start.to_be_bytes());
    payload.extend_from_slice(&origin.end.to_be_bytes());
    Term::atom(
        scope,
        b"clause/source-origin-v1".to_vec(),
        payload,
        EqualityContract::ExactOctetsV1,
    )
}

fn target(
    scope: TermScope,
    type_kind: &[u8],
    payload: &[u8],
) -> Result<FormationTargetV2, TermError> {
    Ok(FormationTargetV2 {
        type_term: Term::atom(
            scope,
            type_kind.to_vec(),
            payload.to_vec(),
            EqualityContract::ExactOctetsV1,
        )?,
        interpretation: Term::atom(
            scope,
            b"clause/canonical-reading-v1".to_vec(),
            b"declaration-profile-v1".to_vec(),
            EqualityContract::ExactOctetsV1,
        )?,
    })
}

const fn exactly_one() -> CardinalityV2 {
    CardinalityV2 {
        minimum: 1,
        maximum: Some(1),
    }
}
