use std::fmt;

use crate::identity::*;
use crate::process::*;
use crate::term::{Atom, RawTriple, Term};

const MAGIC: &[u8; 4] = b"CLPV";
const VERSION: u8 = 1;
const MAX_LIST_ITEMS: u32 = 1_000_000;
const MAX_TERM_DEPTH: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedProcessVector {
    exact_bytes: Box<[u8]>,
    vector: ProcessVector,
}

impl DecodedProcessVector {
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.exact_bytes
    }

    #[must_use]
    pub fn vector(&self) -> &ProcessVector {
        &self.vector
    }

    #[must_use]
    pub fn into_vector(self) -> ProcessVector {
        self.vector
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalEncodeError {
    LengthExceedsU32 { field: &'static str, length: usize },
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
    AllocationFailed {
        offset: usize,
        count: u32,
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
}

impl Encoder {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn fixed(&mut self, value: &[u8]) {
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
}

#[derive(Clone, Copy)]
struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn offset(self) -> usize {
        self.position
    }

    fn remaining(self) -> usize {
        self.bytes.len() - self.position
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

impl<T: Wire> Wire for Vec<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
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
        let mut values = Vec::new();
        values
            .try_reserve_exact(count as usize)
            .map_err(|_| CanonicalDecodeError::AllocationFailed { offset, count })?;
        for _ in 0..count {
            values.push(T::decode(cursor)?);
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
    ProgramSnapshotId,
    ProgramId,
    ProgramRevisionId,
    ProgramChangeOccurrenceId,
    RuntimeSessionId,
    RuntimePolicyId,
    StateRevisionId,
    ApplicationShapeId,
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
    ExecutionAuthorizationLocalId,
    AdmissionAuthorizationLocalId,
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
        encode_term(self, encoder)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        decode_term(cursor, 0)
    }
}

fn encode_term(term: &Term, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
    match term {
        Term::Atom(atom) => {
            encoder.u8(0);
            encoder.blob("atom kind", atom.kind())?;
            encoder.blob("atom canonical payload", atom.canonical_payload())?;
            encoder.blob("atom equality contract", atom.equality_contract())?;
        }
        Term::RawTriple(triple) => {
            encoder.u8(1);
            for slot in triple.slots() {
                encode_term(slot, encoder)?;
            }
        }
    }
    Ok(())
}

fn decode_term(cursor: &mut Cursor<'_>, depth: usize) -> Result<Term, CanonicalDecodeError> {
    if depth > MAX_TERM_DEPTH {
        return Err(CanonicalDecodeError::TermDepthExceeded {
            offset: cursor.offset(),
        });
    }
    let offset = cursor.offset();
    match cursor.u8()? {
        0 => {
            let kind = cursor.blob()?;
            let payload = cursor.blob()?;
            let equality = cursor.blob()?;
            Atom::from_canonical_parts(kind, payload, equality)
                .map(Term::Atom)
                .map_err(|_| CanonicalDecodeError::InvalidAtom { offset })
        }
        1 => Ok(Term::RawTriple(RawTriple::new([
            decode_term(cursor, depth + 1)?,
            decode_term(cursor, depth + 1)?,
            decode_term(cursor, depth + 1)?,
        ]))),
        found => Err(CanonicalDecodeError::UnknownTag {
            offset,
            construct: "Term",
            found,
        }),
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

impl Wire for ModeStateContract {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u8(match self {
            Self::Pure => 0,
            Self::ProposesState => 1,
        });
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        decode_tag(
            cursor,
            "ModeStateContract",
            &[Self::Pure, Self::ProposesState],
        )
    }
}

impl Wire for RoleBindingValue {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Known(term) => {
                encoder.u8(0);
                term.encode(encoder)?;
            }
            Self::Produced => encoder.u8(1),
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Known(Term::decode(cursor)?)),
            1 => Ok(Self::Produced),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "RoleBindingValue",
                found,
            }),
        }
    }
}

wire_struct!(RoleBinding { role, value });
wire_struct!(RelationSchemaDeclaration { id, roles });
wire_struct!(ModeDeclaration {
    id,
    schema,
    known_roles,
    produced_roles,
    context_requirements,
    state_contract,
    may_suspend,
    may_cancel,
});
wire_struct!(OperatorDeclaration { id, modes });
wire_struct!(ProgramConstitutionCandidate {
    semantics,
    snapshot,
    schemas,
    operators,
});
wire_struct!(ApplicationFormCandidate {
    term,
    schema,
    operator,
    eligible_modes,
    bindings,
    context_requirements,
});

impl Wire for ApplicationAllocationAuthority {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::ProgramRevision(id) => {
                encoder.u8(0);
                id.encode(encoder)?;
            }
            Self::RootPolicy(id) => {
                encoder.u8(1);
                id.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::ProgramRevision(ProgramRevisionId::decode(cursor)?)),
            1 => Ok(Self::RootPolicy(RootPolicyId::decode(cursor)?)),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "ApplicationAllocationAuthority",
                found,
            }),
        }
    }
}

wire_struct!(ApplicationProposal {
    id,
    form,
    allocation_authority,
});
wire_struct!(ExecutionScope { application, mode });
wire_struct!(AdmissionScope { session });
wire_struct!(ProgramExecutionAuthorization { reference, scope });
wire_struct!(ProgramAdmissionAuthorization { reference, scope });
wire_struct!(AuthoritativeProgramRevision {
    id,
    program,
    snapshot,
    semantics,
    predecessor,
    change,
    execution_authorizations,
    admission_authorizations,
});
wire_struct!(RootExecutionAuthorization { reference, scope });
wire_struct!(RootAdmissionAuthorization { reference, scope });
wire_struct!(RootPolicy {
    id,
    semantics,
    snapshot_scope,
    execution_authorizations,
    admission_authorizations,
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
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "ExecutionAuthorizationEvidence",
                found,
            }),
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
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "AdmissionAuthorizationEvidence",
                found,
            }),
        }
    }
}

wire_struct!(RuntimeSession {
    id,
    program_revision,
    semantics,
    policy,
    start,
    initial_state,
});

impl Wire for StateRevisionCause {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::SessionStart(start) => {
                encoder.u8(0);
                start.encode(encoder)?;
            }
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
            }
        }
        Ok(())
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
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "StateRevisionCause",
                found,
            }),
        }
    }
}

wire_struct!(StateRevision {
    id,
    session,
    predecessor,
    cause,
    payload,
    policy,
    semantics,
});

impl Wire for Budget {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        encoder.u64(self.remaining_units);
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        Ok(Self {
            remaining_units: cursor.u64()?,
        })
    }
}

wire_struct!(ActivationPins {
    semantics,
    snapshot,
    program_revision,
    runtime_session,
    observed_state,
    runtime_policy,
    budget,
});

impl Wire for RootTrigger {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::External(id) => {
                encoder.u8(0);
                id.encode(encoder)?;
            }
            Self::SessionStart(id) => {
                encoder.u8(1);
                id.encode(encoder)?;
            }
            Self::Admitted(id) => {
                encoder.u8(2);
                id.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::External(ExternalTriggerOccurrenceId::decode(cursor)?)),
            1 => Ok(Self::SessionStart(SessionStartOccurrenceId::decode(
                cursor,
            )?)),
            2 => Ok(Self::Admitted(AdmissionOccurrenceId::decode(cursor)?)),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "RootTrigger",
                found,
            }),
        }
    }
}

impl Wire for ActivationOrigin {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::RootedBy(trigger) => {
                encoder.u8(0);
                trigger.encode(encoder)?;
            }
            Self::ChildOf {
                run,
                parent_activation,
                parent_step,
            } => {
                encoder.u8(1);
                run.encode(encoder)?;
                parent_activation.encode(encoder)?;
                parent_step.encode(encoder)?;
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
            }
        }
        Ok(())
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
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "ActivationOrigin",
                found,
            }),
        }
    }
}

wire_struct!(ActivationCauseFrontier {
    origin,
    authorization,
});

impl Wire for RunMembership {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::RootOf(id) => {
                encoder.u8(0);
                id.encode(encoder)?;
            }
            Self::ChildIn(id) => {
                encoder.u8(1);
                id.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::RootOf(RunId::decode(cursor)?)),
            1 => Ok(Self::ChildIn(RunId::decode(cursor)?)),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "RunMembership",
                found,
            }),
        }
    }
}

wire_struct!(ConfigurationProposal { id, value });
wire_struct!(ActivationProposal {
    id,
    application,
    mode,
    pins,
    causes,
    membership,
    initial_configuration,
});
wire_struct!(ContinuationPins {
    run,
    activation,
    application,
    mode,
    semantics,
    snapshot,
    program_revision,
    runtime_session,
    observed_state,
    runtime_policy,
    remaining_budget,
});
wire_struct!(ContinuationProposal {
    id,
    emitted_by,
    pins,
    remainder,
    linear,
});
wire_struct!(ResumptionOccurrenceProposal {
    id,
    continuation,
    run,
    activation,
    pins,
});
wire_struct!(HandoffOccurrenceProposal {
    id,
    continuation,
    run,
    activation,
    pins,
});

impl Wire for CancellationTarget {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Activation(id) => {
                encoder.u8(0);
                id.encode(encoder)?;
            }
            Self::Run(id) => {
                encoder.u8(1);
                id.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Activation(ActivationId::decode(cursor)?)),
            1 => Ok(Self::Run(RunId::decode(cursor)?)),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "CancellationTarget",
                found,
            }),
        }
    }
}

wire_struct!(CancellationOccurrenceProposal { id, target });

impl Wire for ContinuationTakeupOccurrence {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Resumption(id) => {
                encoder.u8(0);
                id.encode(encoder)?;
            }
            Self::Handoff(id) => {
                encoder.u8(1);
                id.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Resumption(ResumptionOccurrenceId::decode(cursor)?)),
            1 => Ok(Self::Handoff(HandoffOccurrenceId::decode(cursor)?)),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "ContinuationTakeupOccurrence",
                found,
            }),
        }
    }
}

impl Wire for StepCause {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::ActivationStart(id) => {
                encoder.u8(0);
                id.encode(encoder)?;
            }
            Self::PriorStep {
                run,
                activation,
                step,
            } => {
                encoder.u8(1);
                run.encode(encoder)?;
                activation.encode(encoder)?;
                step.encode(encoder)?;
            }
            Self::ContinuationTakeup {
                continuation,
                occurrence,
            } => {
                encoder.u8(2);
                continuation.encode(encoder)?;
                occurrence.encode(encoder)?;
            }
            Self::CancellationRequest(id) => {
                encoder.u8(3);
                id.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::ActivationStart(ActivationId::decode(cursor)?)),
            1 => Ok(Self::PriorStep {
                run: RunId::decode(cursor)?,
                activation: ActivationId::decode(cursor)?,
                step: StepId::decode(cursor)?,
            }),
            2 => Ok(Self::ContinuationTakeup {
                continuation: ContinuationId::decode(cursor)?,
                occurrence: ContinuationTakeupOccurrence::decode(cursor)?,
            }),
            3 => Ok(Self::CancellationRequest(CancellationOccurrenceId::decode(
                cursor,
            )?)),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "StepCause",
                found,
            }),
        }
    }
}

wire_struct!(ObservationProposal { id, value });
wire_struct!(CandidateDeltaProposal {
    id,
    base,
    proposed_payload,
    evidence,
});

impl Wire for StepOutcomeProposal {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Progress => encoder.u8(0),
            Self::Suspend(continuation) => {
                encoder.u8(1);
                continuation.encode(encoder)?;
            }
            Self::Return(value) => {
                encoder.u8(2);
                value.encode(encoder)?;
            }
            Self::Cancel(cancellation) => {
                encoder.u8(3);
                cancellation.encode(encoder)?;
            }
            Self::BudgetExhausted {
                continuation,
                obligations,
            } => {
                encoder.u8(4);
                continuation.encode(encoder)?;
                obligations.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Progress),
            1 => Ok(Self::Suspend(ContinuationProposal::decode(cursor)?)),
            2 => Ok(Self::Return(Term::decode(cursor)?)),
            3 => Ok(Self::Cancel(CancellationOccurrenceId::decode(cursor)?)),
            4 => Ok(Self::BudgetExhausted {
                continuation: Option::<ContinuationProposal>::decode(cursor)?,
                obligations: Vec::<Term>::decode(cursor)?,
            }),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "StepOutcomeProposal",
                found,
            }),
        }
    }
}

wire_struct!(StepProposal {
    id,
    run,
    activation,
    before,
    after,
    observed_state,
    causes,
    observations,
    candidate_delta,
    outcome,
});
wire_struct!(StateAdmissionProposal {
    occurrence,
    delta,
    authorization,
    successor,
});

impl Wire for ProcessRecord {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), CanonicalEncodeError> {
        match self {
            Self::Application(value) => {
                encoder.u8(0);
                value.encode(encoder)?;
            }
            Self::Activation(value) => {
                encoder.u8(1);
                value.encode(encoder)?;
            }
            Self::Resumption(value) => {
                encoder.u8(2);
                value.encode(encoder)?;
            }
            Self::Handoff(value) => {
                encoder.u8(3);
                value.encode(encoder)?;
            }
            Self::Cancellation(value) => {
                encoder.u8(4);
                value.encode(encoder)?;
            }
            Self::Steps(value) => {
                encoder.u8(5);
                value.encode(encoder)?;
            }
            Self::AdmitState(value) => {
                encoder.u8(6);
                value.encode(encoder)?;
            }
        }
        Ok(())
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, CanonicalDecodeError> {
        let offset = cursor.offset();
        match cursor.u8()? {
            0 => Ok(Self::Application(ApplicationProposal::decode(cursor)?)),
            1 => Ok(Self::Activation(ActivationProposal::decode(cursor)?)),
            2 => Ok(Self::Resumption(ResumptionOccurrenceProposal::decode(
                cursor,
            )?)),
            3 => Ok(Self::Handoff(HandoffOccurrenceProposal::decode(cursor)?)),
            4 => Ok(Self::Cancellation(CancellationOccurrenceProposal::decode(
                cursor,
            )?)),
            5 => Ok(Self::Steps(Vec::<StepProposal>::decode(cursor)?)),
            6 => Ok(Self::AdmitState(StateAdmissionProposal::decode(cursor)?)),
            found => Err(CanonicalDecodeError::UnknownTag {
                offset,
                construct: "ProcessRecord",
                found,
            }),
        }
    }
}

wire_struct!(ProcessVector {
    constitution,
    program_revisions,
    root_policies,
    sessions,
    initial_states,
    records,
});

/// Encode one exact process-v1 vector. Encoding does not check or authorize its
/// semantic proposals.
pub fn encode_process_vector(vector: &ProcessVector) -> Result<Vec<u8>, CanonicalEncodeError> {
    validate_canonical_order(vector)?;
    let mut encoder = Encoder::new();
    encoder.fixed(MAGIC);
    encoder.u8(VERSION);
    vector.encode(&mut encoder)?;
    Ok(encoder.bytes)
}

/// Strictly decode one inert process-v1 vector and bind it to its exact bytes.
pub fn decode_process_vector(bytes: &[u8]) -> Result<DecodedProcessVector, CanonicalDecodeError> {
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
    let vector = ProcessVector::decode(&mut cursor)?;
    if cursor.remaining() != 0 {
        return Err(CanonicalDecodeError::TrailingBytes {
            offset: cursor.offset(),
            remaining: cursor.remaining(),
        });
    }
    let canonical = encode_process_vector(&vector).map_err(CanonicalDecodeError::NonCanonical)?;
    if canonical != bytes {
        return Err(CanonicalDecodeError::NonCanonical(
            CanonicalEncodeError::NonCanonicalOrder("wire spelling"),
        ));
    }
    Ok(DecodedProcessVector {
        exact_bytes: bytes.into(),
        vector,
    })
}

fn validate_canonical_order(vector: &ProcessVector) -> Result<(), CanonicalEncodeError> {
    ensure_by_key(&vector.constitution.schemas, "schemas", |value| value.id)?;
    ensure_by_key(&vector.constitution.operators, "operators", |value| {
        value.id
    })?;
    for schema in &vector.constitution.schemas {
        ensure_sorted(&schema.roles, "schema roles")?;
    }
    for operator in &vector.constitution.operators {
        ensure_by_key(&operator.modes, "operator modes", |value| value.id)?;
        for mode in &operator.modes {
            ensure_sorted(&mode.known_roles, "known roles")?;
            ensure_sorted(&mode.produced_roles, "produced roles")?;
            ensure_sorted(&mode.context_requirements, "mode context requirements")?;
        }
    }
    ensure_by_key(&vector.program_revisions, "program revisions", |value| {
        value.id
    })?;
    ensure_by_key(&vector.root_policies, "root policies", |value| value.id)?;
    ensure_by_key(&vector.sessions, "runtime sessions", |value| value.id)?;
    ensure_by_key(&vector.initial_states, "initial states", |value| value.id)?;
    for revision in &vector.program_revisions {
        ensure_sorted(
            &revision.execution_authorizations,
            "program execution authorizations",
        )?;
        ensure_sorted(
            &revision.admission_authorizations,
            "program admission authorizations",
        )?;
    }
    for policy in &vector.root_policies {
        ensure_sorted(
            &policy.execution_authorizations,
            "root execution authorizations",
        )?;
        ensure_sorted(
            &policy.admission_authorizations,
            "root admission authorizations",
        )?;
    }
    for record in &vector.records {
        validate_record_order(record)?;
    }
    Ok(())
}

fn validate_record_order(record: &ProcessRecord) -> Result<(), CanonicalEncodeError> {
    match record {
        ProcessRecord::Application(proposal) => {
            ensure_sorted(&proposal.form.eligible_modes, "eligible modes")?;
            ensure_sorted(&proposal.form.bindings, "role bindings")?;
            ensure_sorted(
                &proposal.form.context_requirements,
                "application context requirements",
            )?;
        }
        ProcessRecord::Steps(proposals) => {
            for proposal in proposals {
                ensure_sorted(&proposal.causes, "step causes")?;
                ensure_sorted(&proposal.observations, "observations")?;
                if let Some(delta) = &proposal.candidate_delta {
                    ensure_sorted(&delta.evidence, "candidate delta evidence")?;
                }
                if let StepOutcomeProposal::BudgetExhausted { obligations, .. } = &proposal.outcome
                {
                    ensure_sorted(obligations, "budget obligations")?;
                }
            }
        }
        ProcessRecord::Activation(_)
        | ProcessRecord::Resumption(_)
        | ProcessRecord::Handoff(_)
        | ProcessRecord::Cancellation(_)
        | ProcessRecord::AdmitState(_) => {}
    }
    Ok(())
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

fn decode_tag<T: Copy>(
    cursor: &mut Cursor<'_>,
    construct: &'static str,
    values: &[T],
) -> Result<T, CanonicalDecodeError> {
    let offset = cursor.offset();
    let tag = cursor.u8()?;
    values
        .get(tag as usize)
        .copied()
        .ok_or(CanonicalDecodeError::UnknownTag {
            offset,
            construct,
            found: tag,
        })
}
