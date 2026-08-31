use std::fmt;

use crate::authority::{
    JudgmentAuthorityScope, RevisionJudgmentAuthorityGrant, RevisionStateAdmissionGrant,
    RevisionStaticExecutionGrant, RevisionSuccessorGrant, StateAdmissionScope,
    StaticExecutionScope, SuccessorAdmissionScope,
};
use crate::formation::*;
use crate::hash::{
    derive_process_package_id, derive_program_snapshot_id as derive_program_snapshot_id_from_bytes,
};
use crate::identity::*;
use crate::process::*;
use crate::provenance::*;
use crate::term::{
    Atom, EqualityContract, MAX_TERM_DEPTH, MAX_TERM_NODES, Term, TermScope, TermValueRef,
};

const MAGIC: &[u8; 4] = b"CLPV";
const VERSION: u8 = 2;
const MAX_LIST_ITEMS: u32 = 1_000_000;
const MAX_CANONICAL_BYTES: usize = 256 * 1024 * 1024;
// These are decoder-local resource refusals, not judgments that otherwise
// canonical bytes have a different constitutional spelling.
const MAX_DECODE_NODES: usize = 2 * MAX_TERM_NODES;
const MAX_DECODE_ALLOCATION_BYTES: usize = 512 * 1024 * 1024;

/// One snapshot-local successor grant. Its authorization reference is resolved
/// only after the enclosing snapshot identity has been derived.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionSuccessorGrantPreimageV2 {
    pub authorization: AdmissionAuthorizationLocalId,
    pub scope: SuccessorAdmissionScope,
}

/// One snapshot-local static execution grant. No field can name the snapshot
/// being derived, so the snapshot preimage is not self-referential.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionStaticExecutionGrantPreimageV2 {
    pub authorization: ExecutionAuthorizationLocalId,
    pub kind: FormationLocalId,
    pub application: ApplicationLocalId,
    pub mode: LocalModeRefV2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionStateAdmissionGrantPreimageV2 {
    pub authorization: AdmissionAuthorizationLocalId,
    pub scope: StateAdmissionScope,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionJudgmentAuthorityGrantPreimageV2 {
    pub authority: JudgmentAuthorityLocalId,
    pub scope: JudgmentAuthorityScope,
}

/// Complete canonical material from which one ProgramSnapshotId is derived.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramSnapshotPreimageV2 {
    pub constitution: ProgramConstitutionPreimageV2,
    pub successor_grants: Vec<RevisionSuccessorGrantPreimageV2>,
    pub static_execution_grants: Vec<RevisionStaticExecutionGrantPreimageV2>,
    pub state_admission_grants: Vec<RevisionStateAdmissionGrantPreimageV2>,
    pub judgment_authority_grants: Vec<RevisionJudgmentAuthorityGrantPreimageV2>,
}

impl ProgramSnapshotPreimageV2 {
    fn resolve_successor_grants(&self, snapshot: ProgramSnapshotId) -> Vec<RevisionSuccessorGrant> {
        self.successor_grants
            .iter()
            .map(|grant| RevisionSuccessorGrant {
                authorization: AdmissionAuthorizationRef {
                    snapshot,
                    local: grant.authorization,
                },
                scope: grant.scope,
            })
            .collect()
    }

    fn resolve_static_execution_grants(
        &self,
        snapshot: ProgramSnapshotId,
    ) -> Vec<RevisionStaticExecutionGrant> {
        self.static_execution_grants
            .iter()
            .map(|grant| RevisionStaticExecutionGrant {
                authorization: ExecutionAuthorizationRef {
                    snapshot,
                    local: grant.authorization,
                },
                scope: StaticExecutionScope {
                    kind: FormationRefV2 {
                        snapshot,
                        local: grant.kind,
                    },
                    application: ApplicationId {
                        snapshot,
                        local: grant.application,
                    },
                    mode: ModeId {
                        operator: OperatorRef {
                            snapshot,
                            local: grant.mode.operator,
                        },
                        local: grant.mode.mode,
                    },
                },
            })
            .collect()
    }

    fn resolve_state_admission_grants(
        &self,
        snapshot: ProgramSnapshotId,
    ) -> Vec<RevisionStateAdmissionGrant> {
        self.state_admission_grants
            .iter()
            .map(|grant| RevisionStateAdmissionGrant {
                authorization: AdmissionAuthorizationRef {
                    snapshot,
                    local: grant.authorization,
                },
                scope: grant.scope,
            })
            .collect()
    }

    fn resolve_judgment_authority_grants(
        &self,
        snapshot: ProgramSnapshotId,
    ) -> Vec<RevisionJudgmentAuthorityGrant> {
        self.judgment_authority_grants
            .iter()
            .map(|grant| RevisionJudgmentAuthorityGrant {
                authority: JudgmentAuthorityRef {
                    snapshot,
                    local: grant.authority,
                },
                scope: grant.scope,
            })
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalEncodeError {
    LengthExceedsU32 { field: &'static str, length: usize },
    ListTooLong { count: usize, maximum: u32 },
    TermDepthExceeded { maximum: usize },
    EncodedBytesTooLong { maximum: usize },
    NonCanonicalOrder(&'static str),
}

impl fmt::Display for CanonicalEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CanonicalEncodeError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalDecodeError {
    UnexpectedEof {
        offset: usize,
        needed: usize,
        remaining: usize,
    },
    WrongMagic {
        found: Vec<u8>,
    },
    UnsupportedVersion {
        offset: usize,
        found: u8,
    },
    UnknownTag {
        offset: usize,
        construct: &'static str,
        found: u8,
    },
    ListTooLong {
        offset: usize,
        count: u32,
    },
    InputTooLong {
        length: usize,
        maximum: usize,
    },
    DeclaredLengthExceedsInput {
        offset: usize,
        count: u32,
        remaining: usize,
    },
    AllocationFailed {
        offset: usize,
        count: u32,
    },
    NodeBudgetExceeded {
        offset: usize,
    },
    AllocationBudgetExceeded {
        offset: usize,
        requested: usize,
    },
    TermDepthExceeded {
        offset: usize,
    },
    InvalidAtom {
        offset: usize,
    },
    TrailingBytes {
        offset: usize,
        remaining: usize,
    },
    NonCanonical(CanonicalEncodeError),
}

impl fmt::Display for CanonicalDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CanonicalDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NonCanonical(error) => Some(error),
            _ => None,
        }
    }
}

struct Encoder {
    bytes: Vec<u8>,
    byte_limit_exceeded: bool,
}

impl Encoder {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            byte_limit_exceeded: false,
        }
    }

    fn u8(&mut self, value: u8) {
        self.fixed(&[value]);
    }

    fn u32(&mut self, value: u32) {
        self.fixed(&value.to_be_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.fixed(&value.to_be_bytes());
    }

    fn fixed(&mut self, value: &[u8]) {
        if self.byte_limit_exceeded {
            return;
        }
        let Some(length) = self.bytes.len().checked_add(value.len()) else {
            self.byte_limit_exceeded = true;
            return;
        };
        if length > MAX_CANONICAL_BYTES {
            self.byte_limit_exceeded = true;
            return;
        }
        self.bytes.extend_from_slice(value);
    }

    fn blob(&mut self, field: &'static str, value: &[u8]) -> Result<(), CanonicalEncodeError> {
        let length =
            u32::try_from(value.len()).map_err(|_| CanonicalEncodeError::LengthExceedsU32 {
                field,
                length: value.len(),
            })?;
        self.u32(length);
        self.fixed(value);
        Ok(())
    }

    fn finish(self) -> Result<Vec<u8>, CanonicalEncodeError> {
        if self.byte_limit_exceeded {
            Err(CanonicalEncodeError::EncodedBytesTooLong {
                maximum: MAX_CANONICAL_BYTES,
            })
        } else {
            Ok(self.bytes)
        }
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
    remaining_nodes: usize,
    remaining_allocation_bytes: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            remaining_nodes: MAX_DECODE_NODES,
            remaining_allocation_bytes: MAX_DECODE_ALLOCATION_BYTES,
        }
    }

    fn offset(&self) -> usize {
        self.position
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    fn charge_nodes(&mut self, count: usize) -> Result<(), CanonicalDecodeError> {
        self.remaining_nodes = self.remaining_nodes.checked_sub(count).ok_or(
            CanonicalDecodeError::NodeBudgetExceeded {
                offset: self.offset(),
            },
        )?;
        Ok(())
    }

    fn charge_allocation(&mut self, bytes: usize) -> Result<(), CanonicalDecodeError> {
        self.remaining_allocation_bytes = self
            .remaining_allocation_bytes
            .checked_sub(bytes)
            .ok_or(CanonicalDecodeError::AllocationBudgetExceeded {
                offset: self.offset(),
                requested: bytes,
            })?;
        Ok(())
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], CanonicalDecodeError> {
        let remaining = self.remaining();
        if length > remaining {
            return Err(CanonicalDecodeError::UnexpectedEof {
                offset: self.offset(),
                needed: length,
                remaining,
            });
        }
        let start = self.position;
        self.position += length;
        Ok(&self.bytes[start..self.position])
    }

    fn u8(&mut self) -> Result<u8, CanonicalDecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, CanonicalDecodeError> {
        let bytes: [u8; 4] = self
            .take(4)?
            .try_into()
            .expect("a four-byte cursor slice has length four");
        Ok(u32::from_be_bytes(bytes))
    }

    fn u64(&mut self) -> Result<u64, CanonicalDecodeError> {
        let bytes: [u8; 8] = self
            .take(8)?
            .try_into()
            .expect("an eight-byte cursor slice has length eight");
        Ok(u64::from_be_bytes(bytes))
    }

    fn blob(&mut self) -> Result<Vec<u8>, CanonicalDecodeError> {
        let length = self.u32()? as usize;
        self.charge_allocation(length)?;
        Ok(self.take(length)?.to_vec())
    }
}

trait Wire: Sized {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError>;
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError>;
}

impl Wire for bool {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(u8::from(*self));
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "Boolean",
                found,
            }),
        }
    }
}

impl Wire for u8 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(*self);
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        cursor.u8()
    }
}

impl Wire for u32 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u32(*self);
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        cursor.u32()
    }
}

impl Wire for u64 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u64(*self);
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        cursor.u64()
    }
}

impl<T: Wire> Wire for Option<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            None => encoder.u8(0),
            Some(value) => {
                encoder.u8(1);
                value.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(None),
            1 => Ok(Some(T::decode(cursor)?)),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "Option",
                found,
            }),
        }
    }
}

impl Wire for Box<[u8]> {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.blob("octet string", self)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(cursor.blob()?.into_boxed_slice())
    }
}

impl<T: Wire> Wire for Vec<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        if self.len() > MAX_LIST_ITEMS as usize {
            return Err(CanonicalEncodeError::ListTooLong {
                count: self.len(),
                maximum: MAX_LIST_ITEMS,
            });
        }
        let count =
            u32::try_from(self.len()).map_err(|_| CanonicalEncodeError::LengthExceedsU32 {
                field: "list",
                length: self.len(),
            })?;
        encoder.u32(count);
        for value in self {
            value.encode(encoder)?;
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        let count = cursor.u32()?;
        if count > MAX_LIST_ITEMS {
            return Err(CanonicalDecodeError::ListTooLong { offset, count });
        }
        // Every private Wire implementation consumes at least one byte. This
        // floor prevents a declared count from driving bulk allocation before
        // the input can possibly contain that many values.
        if count as usize > cursor.remaining() {
            return Err(CanonicalDecodeError::DeclaredLengthExceedsInput {
                offset,
                count,
                remaining: cursor.remaining(),
            });
        }
        cursor.charge_nodes(count as usize)?;
        let mut values = Vec::new();
        for _ in 0..count {
            let value = T::decode(cursor)?;
            let old_capacity = values.capacity();
            values
                .try_reserve(1)
                .map_err(|_| CanonicalDecodeError::AllocationFailed { offset, count })?;
            let added_capacity = values.capacity() - old_capacity;
            let added_bytes = added_capacity.checked_mul(std::mem::size_of::<T>()).ok_or(
                CanonicalDecodeError::AllocationBudgetExceeded {
                    offset,
                    requested: usize::MAX,
                },
            )?;
            // Charge the allocator's visible capacity growth rather than a
            // guessed multiplier. A rejected growth is dropped immediately.
            cursor.charge_allocation(added_bytes)?;
            values.push(value);
        }
        Ok(values)
    }
}

macro_rules! wire_opaque_id {
    ($($name:ident),+ $(,)?) => {
        $(
            impl Wire for $name {
                fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
                    encoder.fixed(self.as_bytes());
                    Ok(())
                }

                fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
                    let bytes: [u8; IDENTITY_BYTES] = cursor
                        .take(IDENTITY_BYTES)?
                        .try_into()
                        .expect("an identity cursor slice has the fixed identity width");
                    Ok(Self::from_bytes(bytes))
                }
            }
        )+
    };
}

wire_opaque_id!(
    ClauseSemanticsId,
    UniverseId,
    ProgramSnapshotId,
    ProgramId,
    ProgramRevisionId,
    ProgramChangeOccurrenceId,
    RuntimeSessionId,
    RuntimePolicyId,
    StateRevisionId,
    ApplicationShapeId,
    ProcessPackageId,
    ActivationId,
    RunId,
    StepId,
    ConfigurationId,
    ContinuationId,
    ObservationId,
    CandidateDeltaId,
    ExternalTriggerOccurrenceId,
    SessionStartOccurrenceId,
    ResumptionOccurrenceId,
    HandoffOccurrenceId,
    CancellationOccurrenceId,
    AdmissionOccurrenceId,
    BoundaryRef,
    ExternalEvidenceRef,
    JudgmentOccurrenceId,
    RootPolicyId,
);

macro_rules! wire_local_id {
    ($($name:ident),+ $(,)?) => {
        $(
            impl Wire for $name {
                fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
                    encoder.u32(self.get());
                    Ok(())
                }

                fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
                    Ok(Self::new(cursor.u32()?))
                }
            }
        )+
    };
}

wire_local_id!(
    RelationSchemaLocalId,
    RoleLocalId,
    OperatorLocalId,
    ModeLocalId,
    ApplicationLocalId,
    FormationLocalId,
    CapabilityLocalId,
    JudgmentLocalId,
    JudgmentAuthorityLocalId,
    ExecutionAuthorizationLocalId,
    AdmissionAuthorizationLocalId,
    PrerequisiteLocalId,
    CauseComponentLocalId,
    SupportSlotId,
    ObligationLocalId,
    BoundaryPermissionLocalId,
);

impl Wire for RelationSchemaId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.snapshot.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            snapshot: ProgramSnapshotId::decode(cursor)?,
            local: RelationSchemaLocalId::decode(cursor)?,
        })
    }
}

impl Wire for RoleId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.schema.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            schema: RelationSchemaId::decode(cursor)?,
            local: RoleLocalId::decode(cursor)?,
        })
    }
}

impl Wire for OperatorRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.snapshot.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            snapshot: ProgramSnapshotId::decode(cursor)?,
            local: OperatorLocalId::decode(cursor)?,
        })
    }
}

impl Wire for ModeId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.operator.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            operator: OperatorRef::decode(cursor)?,
            local: ModeLocalId::decode(cursor)?,
        })
    }
}

impl Wire for PrerequisiteSlotId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.mode.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            mode: ModeId::decode(cursor)?,
            local: PrerequisiteLocalId::decode(cursor)?,
        })
    }
}

impl Wire for ApplicationId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.snapshot.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            snapshot: ProgramSnapshotId::decode(cursor)?,
            local: ApplicationLocalId::decode(cursor)?,
        })
    }
}

impl Wire for CapabilityRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.snapshot.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            snapshot: ProgramSnapshotId::decode(cursor)?,
            local: CapabilityLocalId::decode(cursor)?,
        })
    }
}

impl Wire for JudgmentRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.snapshot.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            snapshot: ProgramSnapshotId::decode(cursor)?,
            local: JudgmentLocalId::decode(cursor)?,
        })
    }
}

impl Wire for JudgmentAuthorityRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.snapshot.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            snapshot: ProgramSnapshotId::decode(cursor)?,
            local: JudgmentAuthorityLocalId::decode(cursor)?,
        })
    }
}

impl Wire for RootJudgmentAuthorityRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.policy.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            policy: RootPolicyId::decode(cursor)?,
            local: JudgmentAuthorityLocalId::decode(cursor)?,
        })
    }
}

impl Wire for ObligationId {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.delta.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            delta: CandidateDeltaId::decode(cursor)?,
            local: ObligationLocalId::decode(cursor)?,
        })
    }
}

impl Wire for ExecutionAuthorizationRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.snapshot.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            snapshot: ProgramSnapshotId::decode(cursor)?,
            local: ExecutionAuthorizationLocalId::decode(cursor)?,
        })
    }
}

impl Wire for AdmissionAuthorizationRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.snapshot.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            snapshot: ProgramSnapshotId::decode(cursor)?,
            local: AdmissionAuthorizationLocalId::decode(cursor)?,
        })
    }
}

impl Wire for RootExecutionAuthorizationRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.policy.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            policy: RootPolicyId::decode(cursor)?,
            local: ExecutionAuthorizationLocalId::decode(cursor)?,
        })
    }
}

impl Wire for RootAdmissionAuthorizationRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        self.policy.encode(encoder)?;
        self.local.encode(encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            policy: RootPolicyId::decode(cursor)?,
            local: AdmissionAuthorizationLocalId::decode(cursor)?,
        })
    }
}

impl Wire for Term {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encode_term(self, encoder, 0)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        decode_term(cursor, 0)
    }
}

fn encode_term(
    term: &Term,
    encoder: &mut Encoder,
    depth: usize,
) -> Result<(), CanonicalEncodeError> {
    if depth > MAX_TERM_DEPTH {
        return Err(CanonicalEncodeError::TermDepthExceeded {
            maximum: MAX_TERM_DEPTH,
        });
    }
    if depth == 0 {
        term.scope().universe.encode(encoder)?;
        term.scope().semantics.encode(encoder)?;
    }
    match term.value() {
        TermValueRef::Atom(atom) => {
            encoder.u8(0);
            encoder.blob("atom kind", atom.kind())?;
            encoder.blob("atom canonical payload", atom.canonical_payload())?;
            atom.equality_contract().encode(encoder)?;
        }
        TermValueRef::RawTriple(triple) => {
            encoder.u8(1);
            for slot in triple.slots() {
                encode_term_value(slot, encoder, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn encode_term_value(
    term: &Term,
    encoder: &mut Encoder,
    depth: usize,
) -> Result<(), CanonicalEncodeError> {
    if depth > MAX_TERM_DEPTH {
        return Err(CanonicalEncodeError::TermDepthExceeded {
            maximum: MAX_TERM_DEPTH,
        });
    }
    match term.value() {
        TermValueRef::Atom(atom) => {
            encoder.u8(0);
            encoder.blob("atom kind", atom.kind())?;
            encoder.blob("atom canonical payload", atom.canonical_payload())?;
            atom.equality_contract().encode(encoder)?;
        }
        TermValueRef::RawTriple(triple) => {
            encoder.u8(1);
            for slot in triple.slots() {
                encode_term_value(slot, encoder, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn decode_term(cursor: &mut Cursor<'_>, depth: usize) -> Result<Term, CanonicalDecodeError> {
    let scope = TermScope {
        universe: UniverseId::decode(cursor)?,
        semantics: ClauseSemanticsId::decode(cursor)?,
    };
    decode_term_value(cursor, scope, depth)
}

fn decode_term_value(
    cursor: &mut Cursor<'_>,
    scope: TermScope,
    depth: usize,
) -> Result<Term, CanonicalDecodeError> {
    if depth > MAX_TERM_DEPTH {
        return Err(CanonicalDecodeError::TermDepthExceeded {
            offset: cursor.offset(),
        });
    }
    cursor.charge_nodes(1)?;
    let offset = cursor.offset();
    match cursor.u8()? {
        0 => {
            let kind = cursor.blob()?;
            let payload = cursor.blob()?;
            let equality = EqualityContract::decode(cursor)?;
            Atom::from_canonical_parts(kind, payload, equality)
                .map(|atom| Term::from_atom(scope, atom))
                .map_err(|_| CanonicalDecodeError::InvalidAtom { offset })
        }
        1 => {
            cursor.charge_allocation(3 * std::mem::size_of::<Term>())?;
            Term::raw_triple([
                decode_term_value(cursor, scope, depth + 1)?,
                decode_term_value(cursor, scope, depth + 1)?,
                decode_term_value(cursor, scope, depth + 1)?,
            ])
            .map_err(|_| CanonicalDecodeError::InvalidAtom { offset })
        }
        found => Err(CanonicalDecodeError::UnknownTag {
            offset,
            construct: "Term",
            found,
        }),
    }
}

impl Wire for EqualityContract {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::ExactOctetsV1 => 0,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::ExactOctetsV1),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "EqualityContract",
                found,
            }),
        }
    }
}

macro_rules! wire_struct {
    ($type:ty { $($field:ident),+ $(,)? }) => {
        impl Wire for $type {
            fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
                $(self.$field.encode(encoder)?;)+
                Ok(())
            }

            fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
                Ok(Self {
                    $($field: Wire::decode(cursor)?,)+
                })
            }
        }
    };
}

wire_struct!(FormationRefV2 { snapshot, local });
wire_struct!(LocalRoleRefV2 { schema, role });
wire_struct!(LocalModeRefV2 { operator, mode });

impl Wire for LocalSemanticDependencyV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Formation(value) => tagged(encoder, 0, value),
            Self::RelationSchema(value) => tagged(encoder, 1, value),
            Self::Role(value) => tagged(encoder, 2, value),
            Self::Operator(value) => tagged(encoder, 3, value),
            Self::Mode(value) => tagged(encoder, 4, value),
            Self::Application(value) => tagged(encoder, 5, value),
            Self::Capability(value) => tagged(encoder, 6, value),
            Self::ExternalReference(value) => tagged(encoder, 7, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        Ok(match cursor.u8()? {
            0 => Self::Formation(FormationLocalId::decode(cursor)?),
            1 => Self::RelationSchema(RelationSchemaLocalId::decode(cursor)?),
            2 => Self::Role(LocalRoleRefV2::decode(cursor)?),
            3 => Self::Operator(OperatorLocalId::decode(cursor)?),
            4 => Self::Mode(LocalModeRefV2::decode(cursor)?),
            5 => Self::Application(ApplicationLocalId::decode(cursor)?),
            6 => Self::Capability(CapabilityLocalId::decode(cursor)?),
            7 => Self::ExternalReference(Term::decode(cursor)?),
            found => return Err(unknown_tag(offset, "LocalSemanticDependencyV2", found)),
        })
    }
}

impl Wire for SemanticDependencyV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Formation(value) => tagged(encoder, 0, value),
            Self::RelationSchema(value) => tagged(encoder, 1, value),
            Self::Role(value) => tagged(encoder, 2, value),
            Self::Operator(value) => tagged(encoder, 3, value),
            Self::Mode(value) => tagged(encoder, 4, value),
            Self::Application(value) => tagged(encoder, 5, value),
            Self::Capability(value) => tagged(encoder, 6, value),
            Self::ExternalReference(value) => tagged(encoder, 7, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        Ok(match cursor.u8()? {
            0 => Self::Formation(FormationRefV2::decode(cursor)?),
            1 => Self::RelationSchema(RelationSchemaId::decode(cursor)?),
            2 => Self::Role(RoleId::decode(cursor)?),
            3 => Self::Operator(OperatorRef::decode(cursor)?),
            4 => Self::Mode(ModeId::decode(cursor)?),
            5 => Self::Application(ApplicationId::decode(cursor)?),
            6 => Self::Capability(CapabilityRef::decode(cursor)?),
            7 => Self::ExternalReference(Term::decode(cursor)?),
            found => return Err(unknown_tag(offset, "SemanticDependencyV2", found)),
        })
    }
}

wire_struct!(FormationTargetV2 {
    type_term,
    interpretation,
});
wire_struct!(FormationJudgmentPreimageV2 {
    id,
    context,
    term,
    target,
    direct_dependencies,
});
wire_struct!(CardinalityV2 { minimum, maximum });
wire_struct!(RoleDeclarationPreimageV2 {
    id,
    target,
    cardinality,
    direct_dependencies,
});
wire_struct!(RelationSchemaPreimageV2 {
    id,
    roles,
    constraints,
    result_domain,
    direct_dependencies,
});
wire_struct!(CapabilityDeclarationPreimageV2 {
    id,
    formation,
    direct_dependencies,
});

impl Wire for DeterminismContractV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::Deterministic => 0,
            Self::ExplicitlyNondeterministic => 1,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Deterministic),
            1 => Ok(Self::ExplicitlyNondeterministic),
            found => Err(unknown_tag(offset, "DeterminismContractV2", found)),
        }
    }
}

impl Wire for ResultOrderContractV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::UnorderedFiniteSet => encoder.u8(0),
            Self::OrderedStream => encoder.u8(1),
            Self::SelectedBy(formation) => {
                encoder.u8(2);
                formation.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::UnorderedFiniteSet),
            1 => Ok(Self::OrderedStream),
            2 => Ok(Self::SelectedBy(FormationLocalId::decode(cursor)?)),
            found => Err(unknown_tag(offset, "ResultOrderContractV2", found)),
        }
    }
}

impl Wire for ProductivityKindV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::Total => 0,
            Self::Productive => 1,
            Self::Bounded => 2,
            Self::Partial => 3,
            Self::Reactive => 4,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Total),
            1 => Ok(Self::Productive),
            2 => Ok(Self::Bounded),
            3 => Ok(Self::Partial),
            4 => Ok(Self::Reactive),
            found => Err(unknown_tag(offset, "ProductivityKindV2", found)),
        }
    }
}

wire_struct!(ProductivityContractV2 { kind, obligations });

impl Wire for ContinuationUseV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::Linear => 0,
            Self::Reusable => 1,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Linear),
            1 => Ok(Self::Reusable),
            found => Err(unknown_tag(offset, "ContinuationUseV2", found)),
        }
    }
}

impl Wire for ContinuationContractV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::TerminalOnly { may_cancel } => {
                encoder.u8(0);
                may_cancel.encode(encoder)?;
            }
            Self::Suspensible {
                use_policy,
                may_handoff,
                may_cancel,
            } => {
                encoder.u8(1);
                use_policy.encode(encoder)?;
                may_handoff.encode(encoder)?;
                may_cancel.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::TerminalOnly {
                may_cancel: bool::decode(cursor)?,
            }),
            1 => Ok(Self::Suspensible {
                use_policy: ContinuationUseV2::decode(cursor)?,
                may_handoff: bool::decode(cursor)?,
                may_cancel: bool::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "ContinuationContractV2", found)),
        }
    }
}

wire_struct!(EffectIntentContractPreimageV2 {
    intent_domain,
    required_capability,
});
wire_struct!(StaticActivationBasisPreimageV2 {
    context_requirements,
    constitutive_dependencies,
});
wire_struct!(AuthorizationRequirementPreimageV2 { kind, cardinality });
wire_struct!(DynamicPrerequisiteRequirementPreimageV2 {
    slot,
    role,
    requirement,
    expected,
    scope,
    cardinality,
    cause_projection,
});
wire_struct!(ModeContractV2 {
    determinism,
    result_cardinality,
    result_order,
    failure_domain,
    state_delta_domain,
    budget_exhaustion_domain,
    effect_intents,
    formation_checks,
    productivity,
    scheduling_requirements,
    resource_requirements,
    capability_requirements,
    continuation,
});
wire_struct!(ModePreimageV2 {
    id,
    schema,
    known_roles,
    produced_roles,
    static_basis,
    authorization_requirements,
    dynamic_prerequisites,
    contract,
    direct_dependencies,
});
wire_struct!(OperatorPreimageV2 {
    id,
    modes,
    direct_dependencies,
});

impl Wire for RoleBindingValuePreimageV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Known(value) => tagged(encoder, 0, value),
            Self::Binder(value) => tagged(encoder, 1, value),
            Self::Produced => {
                encoder.u8(2);
                Ok(())
            }
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Known(FormationLocalId::decode(cursor)?)),
            1 => Ok(Self::Binder(FormationLocalId::decode(cursor)?)),
            2 => Ok(Self::Produced),
            found => Err(unknown_tag(offset, "RoleBindingValuePreimageV2", found)),
        }
    }
}

wire_struct!(RoleBindingPreimageV2 {
    role,
    occurrence,
    value,
});
wire_struct!(ConstraintDischargePreimageV2 {
    constraint,
    evidence,
});
wire_struct!(ApplicationFormPreimageV2 {
    formation,
    schema,
    operator,
    eligible_modes,
    bindings,
    context_requirements,
    constraint_discharges,
    result_domain,
    direct_dependencies,
    dependency_closure,
});
wire_struct!(ApplicationDeclarationPreimageV2 { id, form });
wire_struct!(ProgramConstitutionPreimageV2 {
    semantics,
    universe,
    formations,
    schemas,
    capabilities,
    operators,
    applications,
});

impl Wire for ResolvedRoleBindingValueV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Known(value) => tagged(encoder, 0, value),
            Self::Binder(value) => tagged(encoder, 1, value),
            Self::Produced => {
                encoder.u8(2);
                Ok(())
            }
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Known(FormationRefV2::decode(cursor)?)),
            1 => Ok(Self::Binder(FormationRefV2::decode(cursor)?)),
            2 => Ok(Self::Produced),
            found => Err(unknown_tag(offset, "ResolvedRoleBindingValueV2", found)),
        }
    }
}

wire_struct!(ResolvedRoleBindingV2 {
    role,
    occurrence,
    value,
});
wire_struct!(ResolvedConstraintDischargeV2 {
    constraint,
    evidence,
});
wire_struct!(ApplicationShapePreimageV2 {
    semantics,
    snapshot,
    term,
    formation,
    schema,
    operator,
    eligible_modes,
    bindings,
    context_requirements,
    constraint_discharges,
    result_domain,
    dependency_closure,
});
wire_struct!(SuccessorAdmissionScope {
    semantics,
    program,
    snapshot,
    change,
});
wire_struct!(RevisionSuccessorGrantPreimageV2 {
    authorization,
    scope,
});
wire_struct!(RevisionStaticExecutionGrantPreimageV2 {
    authorization,
    kind,
    application,
    mode,
});
wire_struct!(StateAdmissionScope {
    session,
    base,
    delta,
});
wire_struct!(JudgmentAuthorityScope {
    semantics,
    session,
    policy,
});
wire_struct!(RevisionStateAdmissionGrantPreimageV2 {
    authorization,
    scope,
});
wire_struct!(RevisionJudgmentAuthorityGrantPreimageV2 { authority, scope });
wire_struct!(ProgramSnapshotPreimageV2 {
    constitution,
    successor_grants,
    static_execution_grants,
    state_admission_grants,
    judgment_authority_grants,
});

fn tagged<T: Wire>(encoder: &mut Encoder, tag: u8, value: &T) -> Result<(), CanonicalEncodeError> {
    encoder.u8(tag);
    value.encode(encoder)
}

fn unknown_tag(offset: usize, construct: &'static str, found: u8) -> CanonicalDecodeError {
    CanonicalDecodeError::UnknownTag {
        offset,
        construct,
        found,
    }
}

impl Wire for JudgmentAuthorityEvidence {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::ProgramConstitution {
                revision,
                authority,
            } => {
                encoder.u8(0);
                revision.encode(encoder)?;
                authority.encode(encoder)?;
            }
            Self::IrreducibleRoot { policy, authority } => {
                encoder.u8(1);
                policy.encode(encoder)?;
                authority.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::ProgramConstitution {
                revision: ProgramRevisionId::decode(cursor)?,
                authority: JudgmentAuthorityRef::decode(cursor)?,
            }),
            1 => Ok(Self::IrreducibleRoot {
                policy: RootPolicyId::decode(cursor)?,
                authority: RootJudgmentAuthorityRef::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "JudgmentAuthorityEvidence", found)),
        }
    }
}

wire_struct!(StepRef {
    run,
    activation,
    step,
});

impl Wire for CausalRef {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::SessionStart(value) => tagged(encoder, 0, value),
            Self::ExternalTrigger(value) => tagged(encoder, 1, value),
            Self::Resumption(value) => tagged(encoder, 2, value),
            Self::Handoff(value) => tagged(encoder, 3, value),
            Self::Cancellation(value) => tagged(encoder, 4, value),
            Self::Step(value) => tagged(encoder, 5, value),
            Self::Observation(value) => tagged(encoder, 6, value),
            Self::CandidateDelta(value) => tagged(encoder, 7, value),
            Self::Judgment(value) => tagged(encoder, 8, value),
            Self::Admission(value) => tagged(encoder, 9, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        Ok(match cursor.u8()? {
            0 => Self::SessionStart(SessionStartOccurrenceId::decode(cursor)?),
            1 => Self::ExternalTrigger(ExternalTriggerOccurrenceId::decode(cursor)?),
            2 => Self::Resumption(ResumptionOccurrenceId::decode(cursor)?),
            3 => Self::Handoff(HandoffOccurrenceId::decode(cursor)?),
            4 => Self::Cancellation(CancellationOccurrenceId::decode(cursor)?),
            5 => Self::Step(StepRef::decode(cursor)?),
            6 => Self::Observation(ObservationId::decode(cursor)?),
            7 => Self::CandidateDelta(CandidateDeltaId::decode(cursor)?),
            8 => Self::Judgment(JudgmentOccurrenceId::decode(cursor)?),
            9 => Self::Admission(AdmissionOccurrenceId::decode(cursor)?),
            found => return Err(unknown_tag(offset, "CausalRef", found)),
        })
    }
}

wire_struct!(EnteredThrough {
    boundary,
    evidence,
    permission,
    payload,
    supports,
    causes,
});

impl Wire for OccurrenceProvenance {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::ProducedBy(value) => tagged(encoder, 0, value),
            Self::EnteredThrough(value) => tagged(encoder, 1, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::ProducedBy(StepRef::decode(cursor)?)),
            1 => Ok(Self::EnteredThrough(EnteredThrough::decode(cursor)?)),
            found => Err(unknown_tag(offset, "OccurrenceProvenance", found)),
        }
    }
}

wire_struct!(ExternalTriggerOccurrenceV2 { id, provenance });
wire_struct!(ResumptionOccurrenceBodyV2 {
    id,
    continuation,
    run,
    activation,
    pins,
});
wire_struct!(ResumptionOccurrenceV2 { body, provenance });
wire_struct!(HandoffOccurrenceBodyV2 {
    id,
    continuation,
    run,
    activation,
    pins,
});
wire_struct!(HandoffOccurrenceV2 { body, provenance });
wire_struct!(CancellationOccurrenceBodyV2 { id, target, pins });
wire_struct!(CancellationOccurrenceV2 { body, provenance });
wire_struct!(ExecutionAuthorizationUseV2 { kind, evidence });
wire_struct!(ActivationStaticBasis {
    execution_authorizations,
    judgment_authorities,
});

impl Wire for ActivationPrerequisite {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Observation(value) => tagged(encoder, 0, value),
            Self::Admission(value) => tagged(encoder, 1, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Observation(ObservationId::decode(cursor)?)),
            1 => Ok(Self::Admission(AdmissionOccurrenceId::decode(cursor)?)),
            found => Err(unknown_tag(offset, "ActivationPrerequisite", found)),
        }
    }
}

impl Wire for ActivationPrerequisiteKind {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::Observation => 0,
            Self::Admission => 1,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Observation),
            1 => Ok(Self::Admission),
            found => Err(unknown_tag(offset, "ActivationPrerequisiteKind", found)),
        }
    }
}

impl Wire for PrerequisiteScope {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::SameSemantics => 0,
            Self::SameProgramRevision => 1,
            Self::SameRuntimeSession => 2,
            Self::SameObservedState => 3,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::SameSemantics),
            1 => Ok(Self::SameProgramRevision),
            2 => Ok(Self::SameRuntimeSession),
            3 => Ok(Self::SameObservedState),
            found => Err(unknown_tag(offset, "PrerequisiteScope", found)),
        }
    }
}

impl Wire for PrerequisiteOccurrencePathV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::BoundOccurrence => 0,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::BoundOccurrence),
            found => Err(unknown_tag(offset, "PrerequisiteOccurrencePathV2", found)),
        }
    }
}

wire_struct!(CauseProjectionEntryV2 { component, path });
wire_struct!(DynamicPrerequisiteBindingV2 {
    slot,
    ordinal,
    value,
});
wire_struct!(ActivationOccurrenceCauseV2 {
    slot,
    ordinal,
    component,
    occurrence,
});

impl Wire for SupportSource {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::SessionStart(value) => tagged(encoder, 0, value),
            Self::ExternalTrigger(value) => tagged(encoder, 1, value),
            Self::Resumption(value) => tagged(encoder, 2, value),
            Self::Handoff(value) => tagged(encoder, 3, value),
            Self::Cancellation(value) => tagged(encoder, 4, value),
            Self::Step(value) => tagged(encoder, 5, value),
            Self::Observation(value) => tagged(encoder, 6, value),
            Self::Judgment(value) => tagged(encoder, 7, value),
            Self::Admission(value) => tagged(encoder, 8, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        Ok(match cursor.u8()? {
            0 => Self::SessionStart(SessionStartOccurrenceId::decode(cursor)?),
            1 => Self::ExternalTrigger(ExternalTriggerOccurrenceId::decode(cursor)?),
            2 => Self::Resumption(ResumptionOccurrenceId::decode(cursor)?),
            3 => Self::Handoff(HandoffOccurrenceId::decode(cursor)?),
            4 => Self::Cancellation(CancellationOccurrenceId::decode(cursor)?),
            5 => Self::Step(StepRef::decode(cursor)?),
            6 => Self::Observation(ObservationId::decode(cursor)?),
            7 => Self::Judgment(JudgmentOccurrenceId::decode(cursor)?),
            8 => Self::Admission(AdmissionOccurrenceId::decode(cursor)?),
            found => return Err(unknown_tag(offset, "SupportSource", found)),
        })
    }
}

wire_struct!(SupportUse { slot, role, source });
wire_struct!(CandidateObligation { id, requirement });
wire_struct!(CandidateDeltaV2 {
    id,
    base,
    delta,
    proposed_payload,
    evidence,
    obligations,
});

impl Wire for AdmissionDisposition {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::Admit => 0,
            Self::Reject => 1,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Admit),
            1 => Ok(Self::Reject),
            found => Err(unknown_tag(offset, "AdmissionDisposition", found)),
        }
    }
}

impl Wire for ObligationStatus {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::Satisfied => 0,
            Self::Unsatisfied => 1,
            Self::Deferred => 2,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Satisfied),
            1 => Ok(Self::Unsatisfied),
            2 => Ok(Self::Deferred),
            found => Err(unknown_tag(offset, "ObligationStatus", found)),
        }
    }
}

impl Wire for AdmissionJudgmentClaim {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Verdict(value) => tagged(encoder, 0, value),
            Self::Obligation { obligation, status } => {
                encoder.u8(1);
                obligation.encode(encoder)?;
                status.encode(encoder)
            }
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Verdict(AdmissionDisposition::decode(cursor)?)),
            1 => Ok(Self::Obligation {
                obligation: ObligationId::decode(cursor)?,
                status: ObligationStatus::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "AdmissionJudgmentClaim", found)),
        }
    }
}

wire_struct!(AdmissionJudgment {
    delta,
    session,
    policy,
    claim,
});
wire_struct!(JudgmentOccurrenceBodyV2 {
    id,
    judgment,
    authority,
    supports,
});
wire_struct!(JudgmentOccurrenceV2 { body, provenance });
wire_struct!(ObligationJudgmentUse {
    obligation,
    judgment,
});
wire_struct!(AdmissionRejectionV2 { reason });

impl Wire for StateAdmissionOutcomeV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Admit(value) => tagged(encoder, 0, value),
            Self::Reject(value) => tagged(encoder, 1, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Admit(StateRevision::decode(cursor)?)),
            1 => Ok(Self::Reject(AdmissionRejectionV2::decode(cursor)?)),
            found => Err(unknown_tag(offset, "StateAdmissionOutcomeV2", found)),
        }
    }
}

wire_struct!(StateAdmissionDecisionV2 {
    occurrence,
    delta,
    authorization,
    evidence,
    verdict,
    obligation_judgments,
    provenance,
    outcome,
});

impl Wire for ExecutionAuthorizationEvidence {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::ProgramConstitution {
                revision,
                authorization,
            } => {
                encoder.u8(0);
                revision.encode(encoder)?;
                authorization.encode(encoder)?;
            }
            Self::IrreducibleRoot {
                policy,
                authorization,
            } => {
                encoder.u8(1);
                policy.encode(encoder)?;
                authorization.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::ProgramConstitution {
                revision: ProgramRevisionId::decode(cursor)?,
                authorization: ExecutionAuthorizationRef::decode(cursor)?,
            }),
            1 => Ok(Self::IrreducibleRoot {
                policy: RootPolicyId::decode(cursor)?,
                authorization: RootExecutionAuthorizationRef::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "ExecutionAuthorizationEvidence", found)),
        }
    }
}

impl Wire for AdmissionAuthorizationEvidence {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::ProgramConstitution {
                revision,
                authorization,
            } => {
                encoder.u8(0);
                revision.encode(encoder)?;
                authorization.encode(encoder)?;
            }
            Self::IrreducibleRoot {
                policy,
                authorization,
            } => {
                encoder.u8(1);
                policy.encode(encoder)?;
                authorization.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::ProgramConstitution {
                revision: ProgramRevisionId::decode(cursor)?,
                authorization: AdmissionAuthorizationRef::decode(cursor)?,
            }),
            1 => Ok(Self::IrreducibleRoot {
                policy: RootPolicyId::decode(cursor)?,
                authorization: RootAdmissionAuthorizationRef::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "AdmissionAuthorizationEvidence", found)),
        }
    }
}

wire_struct!(Budget { remaining_units });
wire_struct!(StepBudgetTransitionV2 {
    before,
    consumed_units,
    after,
});

impl Wire for CheckedConstitutionBinding {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Candidate { package, snapshot } => {
                encoder.u8(0);
                package.encode(encoder)?;
                snapshot.encode(encoder)?;
            }
            Self::Admitted { revision } => {
                encoder.u8(1);
                revision.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Candidate {
                package: ProcessPackageId::decode(cursor)?,
                snapshot: ProgramSnapshotId::decode(cursor)?,
            }),
            1 => Ok(Self::Admitted {
                revision: ProgramRevisionId::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "CheckedConstitutionBinding", found)),
        }
    }
}

impl Wire for CancellationScope {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::Activation => 0,
            Self::Run => 1,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Activation),
            1 => Ok(Self::Run),
            found => Err(unknown_tag(offset, "CancellationScope", found)),
        }
    }
}

wire_struct!(ActivationPins {
    semantics,
    snapshot,
    constitution,
    runtime_session,
    observed_state,
    runtime_policy,
    context_requirements,
    constitutive_dependencies,
    capabilities,
    scheduling_requirements,
    resource_requirements,
    cancellation_scope,
    budget,
});

impl Wire for RootTrigger {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::External(value) => tagged(encoder, 0, value),
            Self::SessionStart(value) => tagged(encoder, 1, value),
            Self::Admitted(value) => tagged(encoder, 2, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::External(ExternalTriggerOccurrenceId::decode(cursor)?)),
            1 => Ok(Self::SessionStart(SessionStartOccurrenceId::decode(
                cursor,
            )?)),
            2 => Ok(Self::Admitted(AdmissionOccurrenceId::decode(cursor)?)),
            found => Err(unknown_tag(offset, "RootTrigger", found)),
        }
    }
}

impl Wire for ActivationOrigin {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::RootedBy(value) => tagged(encoder, 0, value),
            Self::ChildOf {
                run,
                parent_activation,
                parent_step,
            } => {
                encoder.u8(1);
                run.encode(encoder)?;
                parent_activation.encode(encoder)?;
                parent_step.encode(encoder)?;
                Ok(())
            }
            Self::HandoffFrom {
                run,
                parent_activation,
                parent_step,
                continuation,
                handoff,
            } => {
                encoder.u8(2);
                run.encode(encoder)?;
                parent_activation.encode(encoder)?;
                parent_step.encode(encoder)?;
                continuation.encode(encoder)?;
                handoff.encode(encoder)?;
                Ok(())
            }
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::RootedBy(RootTrigger::decode(cursor)?)),
            1 => Ok(Self::ChildOf {
                run: RunId::decode(cursor)?,
                parent_activation: ActivationId::decode(cursor)?,
                parent_step: StepId::decode(cursor)?,
            }),
            2 => Ok(Self::HandoffFrom {
                run: RunId::decode(cursor)?,
                parent_activation: ActivationId::decode(cursor)?,
                parent_step: StepId::decode(cursor)?,
                continuation: ContinuationId::decode(cursor)?,
                handoff: HandoffOccurrenceId::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "ActivationOrigin", found)),
        }
    }
}

impl Wire for RunMembership {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::RootOf(value) => tagged(encoder, 0, value),
            Self::ChildIn(value) => tagged(encoder, 1, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::RootOf(RunId::decode(cursor)?)),
            1 => Ok(Self::ChildIn(RunId::decode(cursor)?)),
            found => Err(unknown_tag(offset, "RunMembership", found)),
        }
    }
}

wire_struct!(ActivationCauseFrontierV2 {
    origin,
    prerequisite_occurrences,
});
wire_struct!(ConfigurationProposal { id, value });
wire_struct!(ActivationProposalV2 {
    id,
    application,
    mode,
    pins,
    static_basis,
    prerequisite_bindings,
    causes,
    membership,
    initial_configuration,
});
wire_struct!(ContinuationPins {
    run,
    activation,
    application,
    mode,
    activation_pins,
    remaining_budget,
});
wire_struct!(ContinuationProposalV2 {
    id,
    emitted_by,
    pins,
    remainder,
});

impl Wire for CancellationTarget {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Activation(value) => tagged(encoder, 0, value),
            Self::Run(value) => tagged(encoder, 1, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Activation(ActivationId::decode(cursor)?)),
            1 => Ok(Self::Run(RunId::decode(cursor)?)),
            found => Err(unknown_tag(offset, "CancellationTarget", found)),
        }
    }
}

impl Wire for ContinuationTakeupOccurrence {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Resumption(value) => tagged(encoder, 0, value),
            Self::Handoff(value) => tagged(encoder, 1, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Resumption(ResumptionOccurrenceId::decode(cursor)?)),
            1 => Ok(Self::Handoff(HandoffOccurrenceId::decode(cursor)?)),
            found => Err(unknown_tag(offset, "ContinuationTakeupOccurrence", found)),
        }
    }
}

impl Wire for StepCause {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::ActivationStart(value) => tagged(encoder, 0, value),
            Self::PriorStep(value) => tagged(encoder, 1, value),
            Self::ContinuationTakeup {
                continuation,
                occurrence,
            } => {
                encoder.u8(2);
                continuation.encode(encoder)?;
                occurrence.encode(encoder)?;
                Ok(())
            }
            Self::CancellationRequest(value) => tagged(encoder, 3, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::ActivationStart(ActivationId::decode(cursor)?)),
            1 => Ok(Self::PriorStep(StepRef::decode(cursor)?)),
            2 => Ok(Self::ContinuationTakeup {
                continuation: ContinuationId::decode(cursor)?,
                occurrence: ContinuationTakeupOccurrence::decode(cursor)?,
            }),
            3 => Ok(Self::CancellationRequest(CancellationOccurrenceId::decode(
                cursor,
            )?)),
            found => Err(unknown_tag(offset, "StepCause", found)),
        }
    }
}

impl Wire for TruthVerdict {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::True => 0,
            Self::False => 1,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::True),
            1 => Ok(Self::False),
            found => Err(unknown_tag(offset, "TruthVerdict", found)),
        }
    }
}

impl Wire for ObservationProposalV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Value {
                id,
                value,
                supports,
            } => {
                encoder.u8(0);
                id.encode(encoder)?;
                value.encode(encoder)?;
                supports.encode(encoder)?;
            }
            Self::Truth {
                id,
                verdict,
                proposition,
                supports,
            } => {
                encoder.u8(1);
                id.encode(encoder)?;
                verdict.encode(encoder)?;
                proposition.encode(encoder)?;
                supports.encode(encoder)?;
            }
            Self::Formation {
                id,
                subject,
                target,
                supports,
            } => {
                encoder.u8(2);
                id.encode(encoder)?;
                subject.encode(encoder)?;
                target.encode(encoder)?;
                supports.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Value {
                id: ObservationId::decode(cursor)?,
                value: Term::decode(cursor)?,
                supports: Vec::<SupportUse>::decode(cursor)?,
            }),
            1 => Ok(Self::Truth {
                id: ObservationId::decode(cursor)?,
                verdict: TruthVerdict::decode(cursor)?,
                proposition: Term::decode(cursor)?,
                supports: Vec::<SupportUse>::decode(cursor)?,
            }),
            2 => Ok(Self::Formation {
                id: ObservationId::decode(cursor)?,
                subject: Term::decode(cursor)?,
                target: FormationTargetV2::decode(cursor)?,
                supports: Vec::<SupportUse>::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "ObservationProposalV2", found)),
        }
    }
}

wire_struct!(TruthAbsenceV2 {
    proposition,
    search_scope,
    completion_evidence,
});

impl Wire for StepObservationOutcomeV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Observed(observation) => tagged(encoder, 0, observation),
            Self::Absent(absence) => tagged(encoder, 1, absence),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Observed(ObservationProposalV2::decode(cursor)?)),
            1 => Ok(Self::Absent(TruthAbsenceV2::decode(cursor)?)),
            found => Err(unknown_tag(offset, "StepObservationOutcomeV2", found)),
        }
    }
}

wire_struct!(EnteredObservationV2 {
    observation,
    provenance,
});
wire_struct!(DomainBoundTermV2 { term, evidence });

impl Wire for StepOutcomeProposalV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Progress => {
                encoder.u8(0);
                Ok(())
            }
            Self::Suspend(value) => tagged(encoder, 1, value),
            Self::Return(value) => tagged(encoder, 2, value),
            Self::Fail(value) => tagged(encoder, 3, value),
            Self::Cancel(value) => tagged(encoder, 4, value),
            Self::BudgetExhausted {
                exhaustion,
                continuation,
                obligations,
            } => {
                encoder.u8(5);
                exhaustion.encode(encoder)?;
                continuation.encode(encoder)?;
                obligations.encode(encoder)
            }
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Progress),
            1 => Ok(Self::Suspend(ContinuationProposalV2::decode(cursor)?)),
            2 => Ok(Self::Return(DomainBoundTermV2::decode(cursor)?)),
            3 => Ok(Self::Fail(DomainBoundTermV2::decode(cursor)?)),
            4 => Ok(Self::Cancel(CancellationOccurrenceId::decode(cursor)?)),
            5 => Ok(Self::BudgetExhausted {
                exhaustion: DomainBoundTermV2::decode(cursor)?,
                continuation: Option::<ContinuationProposalV2>::decode(cursor)?,
                obligations: Vec::<Term>::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "StepOutcomeProposalV2", found)),
        }
    }
}

wire_struct!(StepProposalV2 {
    id,
    run,
    activation,
    before,
    after,
    observed_state,
    budget,
    causes,
    observation_outcomes,
    candidate_delta,
    outcome,
});

impl Wire for StateRevisionCause {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::SessionStart(value) => tagged(encoder, 0, value),
            Self::Admission {
                occurrence,
                run,
                activation,
                step,
            } => {
                encoder.u8(1);
                occurrence.encode(encoder)?;
                run.encode(encoder)?;
                activation.encode(encoder)?;
                step.encode(encoder)?;
                Ok(())
            }
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::SessionStart(SessionStartOccurrenceId::decode(
                cursor,
            )?)),
            1 => Ok(Self::Admission {
                occurrence: AdmissionOccurrenceId::decode(cursor)?,
                run: RunId::decode(cursor)?,
                activation: ActivationId::decode(cursor)?,
                step: StepId::decode(cursor)?,
            }),
            found => Err(unknown_tag(offset, "StateRevisionCause", found)),
        }
    }
}

wire_struct!(StateRevision {
    id,
    session,
    predecessor,
    cause,
    payload,
    canonical_state_snapshot,
    policy,
    semantics,
});

impl Wire for ProcessRecordV2 {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::ExternalTrigger(value) => tagged(encoder, 0, value),
            Self::EnteredObservation(value) => tagged(encoder, 1, value),
            Self::Activation(value) => tagged(encoder, 2, value),
            Self::Resumption(value) => tagged(encoder, 3, value),
            Self::Handoff(value) => tagged(encoder, 4, value),
            Self::Cancellation(value) => tagged(encoder, 5, value),
            Self::Steps(value) => tagged(encoder, 6, value),
            Self::Judgment(value) => tagged(encoder, 7, value),
            Self::AdmissionDecision(value) => tagged(encoder, 8, value),
        }
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        Ok(match cursor.u8()? {
            0 => Self::ExternalTrigger(ExternalTriggerOccurrenceV2::decode(cursor)?),
            1 => Self::EnteredObservation(EnteredObservationV2::decode(cursor)?),
            2 => Self::Activation(ActivationProposalV2::decode(cursor)?),
            3 => Self::Resumption(ResumptionOccurrenceV2::decode(cursor)?),
            4 => Self::Handoff(HandoffOccurrenceV2::decode(cursor)?),
            5 => Self::Cancellation(CancellationOccurrenceV2::decode(cursor)?),
            6 => Self::Steps(Vec::<StepProposalV2>::decode(cursor)?),
            7 => Self::Judgment(JudgmentOccurrenceV2::decode(cursor)?),
            8 => Self::AdmissionDecision(StateAdmissionDecisionV2::decode(cursor)?),
            found => return Err(unknown_tag(offset, "ProcessRecordV2", found)),
        })
    }
}

wire_struct!(InitialStateViewV2 {
    session,
    payload,
    canonical_state_snapshot,
});
wire_struct!(ProcessPackageV2 {
    claimed_snapshot,
    snapshot,
    initial_state_views,
    records,
});

/// One strictly decoded process-v2 package. The candidate cannot be extracted
/// by value; checking must consume this exact byte/value binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedProcessPackage {
    exact_bytes: Box<[u8]>,
    candidate: ProcessPackageV2,
}

impl DecodedProcessPackage {
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.exact_bytes
    }

    #[must_use]
    pub fn candidate(&self) -> &ProcessPackageV2 {
        &self.candidate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessPackageCheckError {
    Canonical(CanonicalEncodeError),
    SnapshotIdMismatch {
        claimed: ProgramSnapshotId,
        derived: ProgramSnapshotId,
    },
    InitialStateSnapshotMismatch(RuntimeSessionId),
    Formation(FormationErrorV2),
    Process(ProcessError),
}

impl fmt::Display for ProcessPackageCheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "process package check rejected: {self:?}")
    }
}

impl std::error::Error for ProcessPackageCheckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Canonical(error) => Some(error),
            Self::Formation(error) => Some(error),
            Self::Process(error) => Some(error),
            Self::SnapshotIdMismatch { .. } | Self::InitialStateSnapshotMismatch(_) => None,
        }
    }
}

/// Encode one inert process-v2 candidate. No authority is established.
pub fn encode_process_package(package: &ProcessPackageV2) -> Result<Vec<u8>, CanonicalEncodeError> {
    validate_package_order(package)?;
    let mut encoder = Encoder::new();
    encoder.fixed(MAGIC);
    encoder.u8(VERSION);
    package.encode(&mut encoder)?;
    encoder.finish()
}

/// Decode exactly CLPV version 2. Version 1 has no live compatibility path.
pub fn decode_process_package(bytes: &[u8]) -> Result<DecodedProcessPackage, CanonicalDecodeError> {
    if bytes.len() > MAX_CANONICAL_BYTES {
        return Err(CanonicalDecodeError::InputTooLong {
            length: bytes.len(),
            maximum: MAX_CANONICAL_BYTES,
        });
    }

    let mut cursor = Cursor::new(bytes);
    let found_magic = cursor.take(MAGIC.len())?;
    if found_magic != MAGIC {
        return Err(CanonicalDecodeError::WrongMagic {
            found: found_magic.to_vec(),
        });
    }
    let version_offset = cursor.offset();
    let version = cursor.u8()?;
    if version != VERSION {
        return Err(CanonicalDecodeError::UnsupportedVersion {
            offset: version_offset,
            found: version,
        });
    }

    let candidate = ProcessPackageV2::decode(&mut cursor)?;
    if cursor.remaining() != 0 {
        return Err(CanonicalDecodeError::TrailingBytes {
            offset: cursor.offset(),
            remaining: cursor.remaining(),
        });
    }

    let canonical =
        encode_process_package(&candidate).map_err(CanonicalDecodeError::NonCanonical)?;
    if canonical.as_slice() != bytes {
        return Err(CanonicalDecodeError::NonCanonical(
            CanonicalEncodeError::NonCanonicalOrder("process package spelling"),
        ));
    }
    cursor.charge_allocation(canonical.len())?;
    Ok(DecodedProcessPackage {
        exact_bytes: canonical.into_boxed_slice(),
        candidate,
    })
}

/// Check one exact decoded package. This derives all content identities while
/// leaving revision, runtime, boundary, and admission authority external.
pub fn check_process_package(
    decoded: DecodedProcessPackage,
) -> Result<CheckedProcessPackage, ProcessPackageCheckError> {
    let DecodedProcessPackage {
        exact_bytes,
        candidate,
    } = decoded;
    let ProcessPackageV2 {
        claimed_snapshot,
        snapshot,
        initial_state_views,
        records,
    } = candidate;

    for view in &initial_state_views {
        let encoded =
            canonical_term_bytes(&view.payload).map_err(ProcessPackageCheckError::Canonical)?;
        if encoded.as_slice() != view.canonical_state_snapshot.as_ref() {
            return Err(ProcessPackageCheckError::InitialStateSnapshotMismatch(
                view.session,
            ));
        }
    }

    let canonical_snapshot_preimage =
        encode_wire(&snapshot).map_err(ProcessPackageCheckError::Canonical)?;
    let semantics = snapshot.constitution.semantics;
    let constitution =
        resolve_program_constitution_v2(&snapshot).map_err(ProcessPackageCheckError::Formation)?;
    let derived_snapshot = constitution.snapshot();
    if claimed_snapshot != derived_snapshot {
        return Err(ProcessPackageCheckError::SnapshotIdMismatch {
            claimed: claimed_snapshot,
            derived: derived_snapshot,
        });
    }

    let successor_grants = snapshot.resolve_successor_grants(derived_snapshot);
    let static_execution_grants = snapshot.resolve_static_execution_grants(derived_snapshot);
    let state_admission_grants = snapshot.resolve_state_admission_grants(derived_snapshot);
    let judgment_authority_grants = snapshot.resolve_judgment_authority_grants(derived_snapshot);
    let package_id = derive_process_package_id(semantics, &exact_bytes);

    CheckedProcessPackage::from_checked_parts(
        package_id,
        exact_bytes.into_vec(),
        canonical_snapshot_preimage,
        constitution,
        successor_grants,
        static_execution_grants,
        state_admission_grants,
        judgment_authority_grants,
        initial_state_views,
        records,
    )
    .map_err(ProcessPackageCheckError::Process)
}

fn encode_wire<T: Wire>(value: &T) -> Result<Vec<u8>, CanonicalEncodeError> {
    let mut encoder = Encoder::new();
    value.encode(&mut encoder)?;
    encoder.finish()
}

pub(crate) fn encode_program_snapshot_preimage_v2(
    snapshot: &ProgramSnapshotPreimageV2,
) -> Result<Vec<u8>, CanonicalEncodeError> {
    validate_snapshot_order(snapshot)?;
    encode_wire(snapshot)
}

/// Derive the inert content identity for one canonical local-reference
/// snapshot preimage. Identity agreement grants no authority.
pub fn derive_program_snapshot_id(
    snapshot: &ProgramSnapshotPreimageV2,
) -> Result<ProgramSnapshotId, CanonicalEncodeError> {
    let bytes = encode_program_snapshot_preimage_v2(snapshot)?;
    Ok(derive_program_snapshot_id_from_bytes(
        snapshot.constitution.semantics,
        &bytes,
    ))
}

/// Canonical process-v2 bytes for one scoped Term. This helper is inert and
/// exists so external State anchors do not reimplement the Term format.
pub fn canonical_term_bytes(term: &Term) -> Result<Vec<u8>, CanonicalEncodeError> {
    encode_wire(term)
}

/// Decode one exact canonical scoped Term with no surrounding package.
///
/// This is the inverse of [`canonical_term_bytes`]. The resulting Term is
/// inert; decoding it grants no authority and creates no Observation.
pub fn decode_canonical_term_bytes(bytes: &[u8]) -> Result<Term, CanonicalDecodeError> {
    if bytes.len() > MAX_CANONICAL_BYTES {
        return Err(CanonicalDecodeError::InputTooLong {
            length: bytes.len(),
            maximum: MAX_CANONICAL_BYTES,
        });
    }
    let mut cursor = Cursor::new(bytes);
    let term = Term::decode(&mut cursor)?;
    if cursor.remaining() != 0 {
        return Err(CanonicalDecodeError::TrailingBytes {
            offset: cursor.offset(),
            remaining: cursor.remaining(),
        });
    }
    let canonical = canonical_term_bytes(&term).map_err(CanonicalDecodeError::NonCanonical)?;
    if canonical.as_slice() != bytes {
        return Err(CanonicalDecodeError::NonCanonical(
            CanonicalEncodeError::NonCanonicalOrder("Term spelling"),
        ));
    }
    Ok(term)
}

/// Canonical list payload used only to price cumulative live ingress. It is
/// not a restart format and carries no identity or authority.
pub(crate) fn canonical_process_record_bytes(
    records: &[ProcessRecordV2],
) -> Result<Vec<u8>, CanonicalEncodeError> {
    if records.len() > MAX_LIST_ITEMS as usize {
        return Err(CanonicalEncodeError::ListTooLong {
            count: records.len(),
            maximum: MAX_LIST_ITEMS,
        });
    }
    for record in records {
        validate_record_order_v2(record)?;
    }
    let count =
        u32::try_from(records.len()).map_err(|_| CanonicalEncodeError::LengthExceedsU32 {
            field: "process record batch",
            length: records.len(),
        })?;
    let mut encoder = Encoder::new();
    encoder.u32(count);
    for record in records {
        record.encode(&mut encoder)?;
    }
    encoder.finish()
}

pub(crate) fn encode_application_shape_preimage_v2(
    shape: &ApplicationShapePreimageV2,
) -> Result<Vec<u8>, CanonicalEncodeError> {
    encode_wire(shape)
}

fn validate_package_order(package: &ProcessPackageV2) -> Result<(), CanonicalEncodeError> {
    validate_snapshot_order(&package.snapshot)?;
    ensure_by_key(
        &package.initial_state_views,
        "initial State views",
        |view| view.session,
    )?;
    for record in &package.records {
        validate_record_order_v2(record)?;
    }
    Ok(())
}

fn validate_snapshot_order(
    snapshot: &ProgramSnapshotPreimageV2,
) -> Result<(), CanonicalEncodeError> {
    let constitution = &snapshot.constitution;
    ensure_by_key(&constitution.formations, "formations", |value| value.id)?;
    ensure_by_key(&constitution.schemas, "schemas", |value| value.id)?;
    ensure_by_key(&constitution.capabilities, "capabilities", |value| value.id)?;
    ensure_by_key(&constitution.operators, "operators", |value| value.id)?;
    ensure_by_key(&constitution.applications, "applications", |value| value.id)?;

    for formation in &constitution.formations {
        ensure_sorted(&formation.context, "formation context")?;
        ensure_sorted(
            &formation.direct_dependencies,
            "formation direct dependencies",
        )?;
    }
    for schema in &constitution.schemas {
        ensure_by_key(&schema.roles, "schema roles", |value| value.id)?;
        ensure_sorted(&schema.constraints, "schema constraints")?;
        ensure_sorted(&schema.direct_dependencies, "schema direct dependencies")?;
        for role in &schema.roles {
            ensure_sorted(&role.direct_dependencies, "role direct dependencies")?;
        }
    }
    for capability in &constitution.capabilities {
        ensure_sorted(
            &capability.direct_dependencies,
            "capability direct dependencies",
        )?;
    }
    for operator in &constitution.operators {
        ensure_by_key(&operator.modes, "operator modes", |value| value.id)?;
        ensure_sorted(
            &operator.direct_dependencies,
            "operator direct dependencies",
        )?;
        for mode in &operator.modes {
            ensure_sorted(&mode.known_roles, "known roles")?;
            ensure_sorted(&mode.produced_roles, "produced roles")?;
            ensure_sorted(
                &mode.static_basis.context_requirements,
                "static context requirements",
            )?;
            ensure_sorted(
                &mode.static_basis.constitutive_dependencies,
                "constitutive dependencies",
            )?;
            ensure_by_key(
                &mode.authorization_requirements,
                "authorization requirements",
                |value| value.kind,
            )?;
            ensure_by_key(
                &mode.dynamic_prerequisites,
                "dynamic prerequisites",
                |value| value.slot,
            )?;
            for requirement in &mode.dynamic_prerequisites {
                ensure_by_key(
                    &requirement.cause_projection,
                    "prerequisite cause projection",
                    |entry| entry.component,
                )?;
            }
            ensure_sorted(&mode.contract.effect_intents, "effect intents")?;
            ensure_sorted(&mode.contract.formation_checks, "formation check targets")?;
            ensure_sorted(
                &mode.contract.productivity.obligations,
                "productivity obligations",
            )?;
            ensure_sorted(
                &mode.contract.scheduling_requirements,
                "scheduling requirements",
            )?;
            ensure_sorted(
                &mode.contract.resource_requirements,
                "resource requirements",
            )?;
            ensure_sorted(
                &mode.contract.capability_requirements,
                "capability requirements",
            )?;
            ensure_sorted(&mode.direct_dependencies, "mode direct dependencies")?;
        }
    }
    for application in &constitution.applications {
        let form = &application.form;
        ensure_sorted(&form.eligible_modes, "eligible modes")?;
        ensure_by_key(&form.bindings, "role bindings", |value| {
            (value.role, value.occurrence)
        })?;
        ensure_sorted(&form.context_requirements, "application context")?;
        ensure_sorted(&form.constraint_discharges, "constraint discharges")?;
        ensure_sorted(&form.direct_dependencies, "application direct dependencies")?;
        ensure_sorted(&form.dependency_closure, "application dependency closure")?;
    }

    ensure_sorted(&snapshot.successor_grants, "successor grants")?;
    ensure_sorted(&snapshot.static_execution_grants, "static execution grants")?;
    ensure_sorted(&snapshot.state_admission_grants, "State admission grants")?;
    ensure_sorted(
        &snapshot.judgment_authority_grants,
        "Judgment authority grants",
    )
}

fn validate_record_order_v2(record: &ProcessRecordV2) -> Result<(), CanonicalEncodeError> {
    match record {
        ProcessRecordV2::ExternalTrigger(value) => validate_entered(&value.provenance),
        ProcessRecordV2::EnteredObservation(value) => {
            validate_observation_order(&value.observation)?;
            validate_entered(&value.provenance)
        }
        ProcessRecordV2::Activation(value) => {
            validate_activation_pins_order(&value.pins)?;
            ensure_sorted(
                &value.static_basis.execution_authorizations,
                "execution authorization uses",
            )?;
            ensure_sorted(
                &value.static_basis.judgment_authorities,
                "Judgment authority uses",
            )?;
            ensure_by_key(
                &value.prerequisite_bindings,
                "dynamic prerequisite bindings",
                |binding| (binding.slot, binding.ordinal),
            )?;
            ensure_by_key(
                &value.causes.prerequisite_occurrences,
                "Activation prerequisite causes",
                |cause| (cause.slot, cause.ordinal, cause.component),
            )
        }
        ProcessRecordV2::Resumption(value) => {
            validate_continuation_pins_order(&value.body.pins)?;
            validate_occurrence(&value.provenance)
        }
        ProcessRecordV2::Handoff(value) => {
            validate_continuation_pins_order(&value.body.pins)?;
            validate_occurrence(&value.provenance)
        }
        ProcessRecordV2::Cancellation(value) => {
            validate_activation_pins_order(&value.body.pins)?;
            validate_occurrence(&value.provenance)
        }
        ProcessRecordV2::Steps(steps) => {
            for step in steps {
                ensure_sorted(&step.causes, "Step causes")?;
                ensure_sorted(&step.observation_outcomes, "Step observation outcomes")?;
                for outcome in &step.observation_outcomes {
                    match outcome {
                        StepObservationOutcomeV2::Observed(observation) => {
                            validate_observation_order(observation)?;
                        }
                        StepObservationOutcomeV2::Absent(absence) => {
                            validate_supports(&absence.completion_evidence)?;
                        }
                    }
                }
                if let Some(delta) = &step.candidate_delta {
                    validate_delta_order(delta)?;
                }
                match &step.outcome {
                    StepOutcomeProposalV2::Suspend(continuation) => {
                        validate_continuation_pins_order(&continuation.pins)?;
                    }
                    StepOutcomeProposalV2::BudgetExhausted {
                        continuation,
                        obligations,
                        ..
                    } => {
                        if let Some(continuation) = continuation {
                            validate_continuation_pins_order(&continuation.pins)?;
                        }
                        ensure_sorted(obligations, "budget obligations")?;
                    }
                    StepOutcomeProposalV2::Progress
                    | StepOutcomeProposalV2::Return(_)
                    | StepOutcomeProposalV2::Fail(_)
                    | StepOutcomeProposalV2::Cancel(_) => {}
                }
            }
            Ok(())
        }
        ProcessRecordV2::Judgment(value) => {
            validate_supports(&value.body.supports)?;
            validate_occurrence(&value.provenance)
        }
        ProcessRecordV2::AdmissionDecision(value) => {
            validate_supports(&value.evidence)?;
            ensure_by_key(
                &value.obligation_judgments,
                "obligation Judgments",
                |use_| use_.obligation,
            )?;
            validate_entered(&value.provenance)
        }
    }
}

fn validate_continuation_pins_order(pins: &ContinuationPins) -> Result<(), CanonicalEncodeError> {
    validate_activation_pins_order(&pins.activation_pins)
}

fn validate_activation_pins_order(pins: &ActivationPins) -> Result<(), CanonicalEncodeError> {
    ensure_sorted(&pins.capabilities, "Activation capabilities")?;
    ensure_sorted(
        &pins.context_requirements,
        "Activation context requirements",
    )?;
    ensure_sorted(
        &pins.constitutive_dependencies,
        "Activation constitutive dependencies",
    )?;
    ensure_sorted(
        &pins.scheduling_requirements,
        "Activation scheduling requirements",
    )?;
    ensure_sorted(
        &pins.resource_requirements,
        "Activation resource requirements",
    )
}

fn validate_observation_order(
    observation: &ObservationProposalV2,
) -> Result<(), CanonicalEncodeError> {
    validate_supports(observation.supports())
}

fn validate_delta_order(delta: &CandidateDeltaV2) -> Result<(), CanonicalEncodeError> {
    validate_supports(&delta.evidence)?;
    ensure_by_key(&delta.obligations, "candidate obligations", |value| {
        value.id
    })
}

fn validate_supports(supports: &[SupportUse]) -> Result<(), CanonicalEncodeError> {
    ensure_by_key(supports, "support slots", |value| value.slot)
}

fn validate_occurrence(provenance: &OccurrenceProvenance) -> Result<(), CanonicalEncodeError> {
    match provenance {
        OccurrenceProvenance::ProducedBy(_) => Ok(()),
        OccurrenceProvenance::EnteredThrough(value) => validate_entered(value),
    }
}

fn validate_entered(provenance: &EnteredThrough) -> Result<(), CanonicalEncodeError> {
    ensure_sorted(&provenance.causes, "entered occurrence causes")
}

fn ensure_sorted<T: Ord>(values: &[T], field: &'static str) -> Result<(), CanonicalEncodeError> {
    if values.windows(2).all(|pair| pair[0] < pair[1]) {
        Ok(())
    } else {
        Err(CanonicalEncodeError::NonCanonicalOrder(field))
    }
}

fn ensure_by_key<T, K: Ord>(
    values: &[T],
    field: &'static str,
    key: impl Fn(&T) -> K,
) -> Result<(), CanonicalEncodeError> {
    if values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1])) {
        Ok(())
    } else {
        Err(CanonicalEncodeError::NonCanonicalOrder(field))
    }
}
