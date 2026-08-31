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
    pub input_handler: Option<CanonicalInputHandlerV1>,
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
    InvalidInputHandler {
        origin: CanonicalSourceOriginV1,
    },
    MissingInputInitialAssertion {
        origin: CanonicalSourceOriginV1,
    },
    AmbiguousInputInitialAssertion {
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
    Shape {
        designation: Vec<u8>,
        fields: Vec<ShapeField>,
    },
    Relation(RelationCst),
    InputHandler(InputHandlerCst),
    VectorAssertion(VectorAssertionCst),
    Unsupported(CanonicalUnsupportedProductionV1),
}

#[derive(Clone, Debug)]
struct InputHandlerCst {
    origin: CanonicalSourceOriginV1,
    producer: CanonicalSemanticProducerV1,
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
    z: u64,
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
    canonical: Vec<u8>,
    origin: CanonicalSourceOriginV1,
}

#[derive(Clone, Debug)]
struct RelationCst {
    designation: Vec<u8>,
    roles: Vec<RelationRoleCst>,
    modes: Vec<RelationModeCst>,
}

#[derive(Clone, Copy, Debug)]
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
    RelationSchema,
    Role,
    Operator,
    Mode,
}

impl AllocationDomain {
    const fn of(identity: CanonicalAllocatedIdentityV1) -> Self {
        match identity {
            CanonicalAllocatedIdentityV1::Formation(_) => Self::Formation,
            CanonicalAllocatedIdentityV1::RelationSchema(_) => Self::RelationSchema,
            CanonicalAllocatedIdentityV1::Role(_) => Self::Role,
            CanonicalAllocatedIdentityV1::Operator(_) => Self::Operator,
            CanonicalAllocatedIdentityV1::Mode(_) => Self::Mode,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Formation => "FormationLocalId",
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
    let input = input_handler_parts(cst)?;
    for item in &cst.items {
        match &item.kind {
            CstKind::Referent { designation } => {
                requested.push(AllocationRequest {
                    producer: semantic_producer(CanonicalSourceProductionV1::Referent, designation),
                    slot: head_slot(CanonicalSourceProductionV1::Referent),
                    domain: AllocationDomain::Formation,
                });
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
            CstKind::VectorAssertion(assertion) => {
                if input
                    .as_ref()
                    .is_some_and(|(_, selected)| selected.origin == assertion.origin)
                {
                    requested.push(AllocationRequest {
                        producer: assertion_producer(assertion),
                        slot: head_slot(CanonicalSourceProductionV1::Assertion),
                        domain: AllocationDomain::Formation,
                    });
                }
            }
            CstKind::Unsupported(_) => {}
        }
    }
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
    let mut operators = Vec::new();
    let mut emissions = Vec::new();
    let mut unsupported = Vec::new();
    let input_parts = input_handler_parts(cst)?;
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
                            effect_intents: vec![],
                            formation_checks: vec![],
                            productivity: ProductivityContractV2 {
                                kind: ProductivityKindV2::Partial,
                                obligations: vec![],
                            },
                            scheduling_requirements: vec![],
                            resource_requirements: vec![],
                            capability_requirements: vec![],
                            continuation: ContinuationContractV2::TerminalOnly {
                                may_cancel: false,
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
            CstKind::VectorAssertion(assertion) => {
                if input_parts
                    .as_ref()
                    .is_some_and(|(_, selected)| selected.origin == assertion.origin)
                {
                    let producer = assertion_producer(assertion);
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
            CstKind::Unsupported(value) => unsupported.push(value.clone()),
        }
    }
    formations.sort_by_key(|formation| formation.id);
    schemas.sort_by_key(|schema| schema.id);
    operators.sort_by_key(|operator| operator.id);
    let snapshot = ProgramSnapshotPreimageV2 {
        constitution: ProgramConstitutionPreimageV2 {
            semantics: context.semantics,
            universe: context.universe,
            formations,
            schemas,
            capabilities: vec![],
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
    let checked_package = check_process_package(decoded).map_err(CanonicalSourceErrorV1::Check)?;
    Ok(CanonicalSourcePackageSliceV1 {
        checked_package,
        emissions,
        unsupported,
        input_handler,
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
    if let Some(designation) = head.strip_prefix("referent ") {
        require_leaf(block, artifact)?;
        return Ok(CstItem {
            origin,
            kind: CstKind::Referent {
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
    if head.starts_with("law ") {
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
    if let Some(assertion) = parse_vector_assertion(head, origin)? {
        return Ok(CstItem {
            origin,
            kind: CstKind::VectorAssertion(assertion),
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
        relation,
        result_x,
        result_z,
        include_origin,
        include_local: include_line.as_bytes().to_vec(),
    }))
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
    if y != 0.0_f64.to_bits() {
        return Ok(None);
    }
    Ok(Some(VectorAssertionCst {
        origin,
        subject: prefix[0].as_bytes().to_vec(),
        relation: prefix[1..].join(" ").into_bytes(),
        x,
        z,
    }))
}

fn split_vector_subject(source: &str) -> Option<(&str, &str)> {
    let (prefix, vector) = source.split_once(" Vec3 { ")?;
    Some((prefix, vector))
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
    let mut modes = Vec::new();
    let mut subject = None;
    for line in block
        .iter()
        .skip(1)
        .filter(|line| !line.text.trim().is_empty())
    {
        let origin = line_origin(artifact, *line);
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
    if let Some(subject) = subject
        && !declared.contains(subject.as_slice())
    {
        return Err(CanonicalSourceErrorV1::UnknownSubjectRole {
            designation,
            role: subject,
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
    }
    ensure_unique_children(
        &designation,
        modes.iter().map(|mode| mode.canonical.as_slice()),
    )?;
    Ok(RelationCst {
        designation,
        roles,
        modes,
    })
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
        canonical: source.as_bytes().to_vec(),
        origin,
    })
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

fn assertion_producer(assertion: &VectorAssertionCst) -> CanonicalSemanticProducerV1 {
    let mut key = Vec::new();
    frame_bytes(&mut key, &assertion.subject);
    frame_bytes(&mut key, &assertion.relation);
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

fn validate_unique_designations(items: &[CstItem]) -> Result<(), CanonicalSourceErrorV1> {
    let mut seen = BTreeSet::new();
    for designation in items.iter().filter_map(|item| match &item.kind {
        CstKind::Referent { designation } | CstKind::Shape { designation, .. } => Some(designation),
        CstKind::Relation(relation) => Some(&relation.designation),
        CstKind::InputHandler(_) | CstKind::VectorAssertion(_) | CstKind::Unsupported(_) => None,
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
