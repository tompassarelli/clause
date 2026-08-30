use std::fmt;

/// Canonical transport width shared by disjoint semantic identity domains.
pub const IDENTITY_BYTES: usize = 32;

macro_rules! opaque_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; IDENTITY_BYTES]);

        impl $name {
            #[must_use]
            pub const fn from_bytes(bytes: [u8; IDENTITY_BYTES]) -> Self {
                Self(bytes)
            }

            #[must_use]
            pub const fn as_bytes(&self) -> &[u8; IDENTITY_BYTES] {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}(", stringify!($name))?;
                for byte in &self.0[..4] {
                    write!(formatter, "{byte:02x}")?;
                }
                formatter.write_str("…)")
            }
        }
    };
}

macro_rules! local_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u32);

        impl $name {
            #[must_use]
            pub const fn new(value: u32) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }
    };
}

opaque_id!(ClauseSemanticsId);
opaque_id!(UniverseId);
opaque_id!(ProgramSnapshotId);
opaque_id!(ProgramId);
opaque_id!(ProgramRevisionId);
opaque_id!(ProgramChangeOccurrenceId);
opaque_id!(RuntimeSessionId);
opaque_id!(RuntimePolicyId);
opaque_id!(StateRevisionId);
opaque_id!(ApplicationShapeId);
opaque_id!(ProcessPackageId);
opaque_id!(ActivationId);
opaque_id!(RunId);
opaque_id!(StepId);
opaque_id!(ConfigurationId);
opaque_id!(ContinuationId);
opaque_id!(ObservationId);
opaque_id!(CandidateDeltaId);
opaque_id!(ExternalTriggerOccurrenceId);
opaque_id!(SessionStartOccurrenceId);
opaque_id!(ResumptionOccurrenceId);
opaque_id!(HandoffOccurrenceId);
opaque_id!(CancellationOccurrenceId);
opaque_id!(AdmissionOccurrenceId);
opaque_id!(RootPolicyId);
opaque_id!(BoundaryRef);
opaque_id!(ExternalEvidenceRef);
opaque_id!(JudgmentOccurrenceId);

local_id!(RelationSchemaLocalId);
local_id!(RoleLocalId);
local_id!(OperatorLocalId);
local_id!(ModeLocalId);
local_id!(ApplicationLocalId);
local_id!(FormationLocalId);
local_id!(CapabilityLocalId);
local_id!(JudgmentLocalId);
local_id!(JudgmentAuthorityLocalId);
local_id!(ExecutionAuthorizationLocalId);
local_id!(AdmissionAuthorizationLocalId);
local_id!(SupportSlotId);
local_id!(ObligationLocalId);

/// One schema declaration in one exact Program snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RelationSchemaId {
    pub snapshot: ProgramSnapshotId,
    pub local: RelationSchemaLocalId,
}

/// One named role declaration in one exact schema.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RoleId {
    pub schema: RelationSchemaId,
    pub local: RoleLocalId,
}

/// One operator declaration in one exact Program snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperatorRef {
    pub snapshot: ProgramSnapshotId,
    pub local: OperatorLocalId,
}

/// One mode declaration under one exact operator.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModeId {
    pub operator: OperatorRef,
    pub local: ModeLocalId,
}

/// One capability declaration in one exact Program snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapabilityRef {
    pub snapshot: ProgramSnapshotId,
    pub local: CapabilityLocalId,
}

/// One checked formation declaration in one exact Program snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FormationRefV2 {
    pub snapshot: ProgramSnapshotId,
    pub local: FormationLocalId,
}

/// One immutable Judgment declaration in one exact Program snapshot.
///
/// This reference is assessed content, not its issuing authority, issuance
/// occurrence, or an authorization to act.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct JudgmentRef {
    pub snapshot: ProgramSnapshotId,
    pub local: JudgmentLocalId,
}

/// One judgment-issuing authority declaration in one exact Program snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct JudgmentAuthorityRef {
    pub snapshot: ProgramSnapshotId,
    pub local: JudgmentAuthorityLocalId,
}

/// One judgment-issuing authority in an irreducible root policy.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RootJudgmentAuthorityRef {
    pub policy: RootPolicyId,
    pub local: JudgmentAuthorityLocalId,
}

/// One nominal Application declaration in one exact Program snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationId {
    pub snapshot: ProgramSnapshotId,
    pub local: ApplicationLocalId,
}

/// One typed execution-authorization declaration in an exact snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExecutionAuthorizationRef {
    pub snapshot: ProgramSnapshotId,
    pub local: ExecutionAuthorizationLocalId,
}

/// One typed admission-authorization declaration in an exact snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdmissionAuthorizationRef {
    pub snapshot: ProgramSnapshotId,
    pub local: AdmissionAuthorizationLocalId,
}

/// One typed execution authorization in an irreducible root policy.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RootExecutionAuthorizationRef {
    pub policy: RootPolicyId,
    pub local: ExecutionAuthorizationLocalId,
}

/// One typed admission authorization in an irreducible root policy.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RootAdmissionAuthorizationRef {
    pub policy: RootPolicyId,
    pub local: AdmissionAuthorizationLocalId,
}

/// One exact obligation within one candidate delta.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObligationId {
    pub delta: CandidateDeltaId,
    pub local: ObligationLocalId,
}
