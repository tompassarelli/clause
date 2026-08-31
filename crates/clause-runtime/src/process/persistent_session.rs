use std::error::Error;
use std::fmt;

use clause_package::{
    ActivationId, AdmissionAuthorizationEvidence, ApplicationId, AuthorityError, AuthorityStore,
    CandidateDeltaId, CheckedProcessPackage, ConfigurationId, ProcessCarrier, ProcessPackageId,
    ProgramRevisionId, RootPolicyAnchor, RunId, RuntimeSessionId, StateRevisionId,
};

use super::executable::{persistent_candidate_id_from_seed_v1, persistent_candidate_id_v1};
use super::{
    ExecutableAuthorityFactsV1, ExecutableCandidateV1, ExecutableCarrierErrorV1, ExecutableErrorV1,
    ExecutableProcessRuntimeV1, ExecutableProjectedObservationV1, ExecutableStateRevisionV1,
    ExecutableStepV1, ExecutableValueV1, decode_executable_occurrence_v1,
};

/// One native, long-lived execution binding for an exact package, Program
/// revision, RuntimeSession, and current admitted world base.
///
/// All fields are private. Commands retain one concrete executable runtime;
/// they never rebuild it, clone its package, or replace its authority store.
pub struct PersistentProcessSessionV1 {
    runtime: Option<ExecutableProcessRuntimeV1>,
    session: RuntimeSessionId,
    program_revision: ProgramRevisionId,
    world_base: StateRevisionId,
    last_admitted: Option<ExecutableStateRevisionV1>,
}

impl PersistentProcessSessionV1 {
    /// Open one native session by transferring ownership of the checked
    /// package and authority store into its runtime.
    pub fn open(
        package: CheckedProcessPackage,
        authority: AuthorityStore,
        application: ApplicationId,
        facts: ExecutableAuthorityFactsV1,
    ) -> Result<Self, PersistentProcessSessionErrorV1> {
        Self::open_with_identity_seed(
            package,
            authority,
            application,
            facts,
            *facts.session.as_bytes(),
        )
    }

    /// Open one native session with an explicit nominal allocation seed.
    /// The seed affects runtime occurrence identities only; the separately
    /// typed RuntimeSession remains the continuity and authority pin.
    pub fn open_with_identity_seed(
        package: CheckedProcessPackage,
        authority: AuthorityStore,
        application: ApplicationId,
        facts: ExecutableAuthorityFactsV1,
        identity_seed: [u8; clause_package::IDENTITY_BYTES],
    ) -> Result<Self, PersistentProcessSessionErrorV1> {
        let mut runtime = ExecutableProcessRuntimeV1::instantiate_session(
            package,
            authority,
            application,
            identity_seed,
        )?;
        runtime.start_carrier_process(facts)?;
        Ok(Self {
            runtime: Some(runtime),
            session: facts.session,
            program_revision: facts.program_revision,
            world_base: facts.initial_state,
            last_admitted: None,
        })
    }

    /// Derive the exact candidate identity that external Admission authority
    /// must scope for one zero-based session candidate ordinal.
    #[must_use]
    pub fn candidate_id_for(session: RuntimeSessionId, candidate_ordinal: u64) -> CandidateDeltaId {
        persistent_candidate_id_v1(session, candidate_ordinal)
    }

    /// Derive a candidate identity for a session opened with an explicit
    /// nominal allocation seed.
    #[must_use]
    pub fn candidate_id_for_seed(
        identity_seed: [u8; clause_package::IDENTITY_BYTES],
        candidate_ordinal: u64,
    ) -> CandidateDeltaId {
        persistent_candidate_id_from_seed_v1(identity_seed, candidate_ordinal)
    }

    /// Decode and execute one construct-blind occurrence without requesting
    /// Admission. Its entered Observation and local Step are committed only if
    /// their complete carrier batch succeeds.
    pub fn apply_opaque_input(
        &mut self,
        exact_occurrence: &[u8],
    ) -> Result<ExecutableStepV1, PersistentProcessSessionErrorV1> {
        let occurrence = decode_executable_occurrence_v1(exact_occurrence)?;
        Ok(self
            .runtime_mut()?
            .advance_carrier_occurrence(occurrence)?
            .clone())
    }

    /// Execute one construct-blind occurrence and attach the immutable
    /// candidate delta to that exact local Step. No Judgment, AdmissionDecision,
    /// or StateRevision is created by this command.
    pub fn apply_opaque_input_and_emit_candidate(
        &mut self,
        exact_occurrence: &[u8],
    ) -> Result<ExecutableStepV1, PersistentProcessSessionErrorV1> {
        let occurrence = decode_executable_occurrence_v1(exact_occurrence)?;
        Ok(self
            .runtime_mut()?
            .advance_carrier_occurrence_and_emit_candidate(occurrence)?
            .clone())
    }

    /// Atomically enter the prepared Judgment, AdmissionDecision, and the
    /// successor-pinned admitted-root Activation. Only after that carrier batch
    /// succeeds does the live session install the new Run/Activation epoch.
    pub fn admit_candidate(
        &mut self,
        authorization: AdmissionAuthorizationEvidence,
    ) -> Result<ExecutableStateRevisionV1, PersistentProcessSessionErrorV1> {
        self.admit_candidate_with_projection(authorization)
            .map(|(admitted, _)| admitted)
    }

    /// Atomically admit the candidate, enter its package-declared derived
    /// Observation when present, and install the successor execution epoch.
    pub fn admit_candidate_with_projection(
        &mut self,
        authorization: AdmissionAuthorizationEvidence,
    ) -> Result<
        (
            ExecutableStateRevisionV1,
            Option<ExecutableProjectedObservationV1>,
        ),
        PersistentProcessSessionErrorV1,
    > {
        let (admitted, projection) = self
            .runtime_mut()?
            .settle_carrier_process_project_and_start_epoch(authorization)?;
        self.world_base = admitted.id;
        self.last_admitted = Some(admitted.clone());
        Ok((admitted, projection))
    }

    /// Establish one caller-supplied root policy on the runtime-owned
    /// authority store. This is the only authority mutation exposed by the
    /// persistent session; each Admission still receives its exact typed
    /// evidence separately.
    pub fn establish_root_policy(
        &mut self,
        policy: RootPolicyAnchor,
    ) -> Result<(), PersistentProcessSessionErrorV1> {
        self.runtime_mut()?
            .establish_root_policy(policy)
            .map_err(PersistentProcessSessionErrorV1::Authority)
    }

    /// Deterministically retire the owned runtime. Disposal is idempotent, and
    /// every later command or live-state query fails closed with `Disposed`.
    pub fn dispose(&mut self) -> bool {
        self.runtime.take().is_some()
    }

    #[must_use]
    pub const fn is_disposed(&self) -> bool {
        self.runtime.is_none()
    }

    #[must_use]
    pub const fn runtime_session(&self) -> RuntimeSessionId {
        self.session
    }

    #[must_use]
    pub const fn program_revision(&self) -> ProgramRevisionId {
        self.program_revision
    }

    #[must_use]
    pub const fn world_base(&self) -> StateRevisionId {
        self.world_base
    }

    pub fn package(&self) -> Result<ProcessPackageId, PersistentProcessSessionErrorV1> {
        Ok(self.runtime()?.package())
    }

    pub fn application(&self) -> Result<ApplicationId, PersistentProcessSessionErrorV1> {
        Ok(self.runtime()?.application())
    }

    pub fn run(&self) -> Result<RunId, PersistentProcessSessionErrorV1> {
        Ok(self.runtime()?.run())
    }

    pub fn activation(&self) -> Result<ActivationId, PersistentProcessSessionErrorV1> {
        Ok(self.runtime()?.activation())
    }

    pub fn configuration_id(&self) -> Result<ConfigurationId, PersistentProcessSessionErrorV1> {
        Ok(self.runtime()?.configuration_id())
    }

    pub fn configuration(&self) -> Result<&[ExecutableValueV1], PersistentProcessSessionErrorV1> {
        Ok(self.runtime()?.configuration())
    }

    pub fn candidate(
        &self,
    ) -> Result<Option<&ExecutableCandidateV1>, PersistentProcessSessionErrorV1> {
        Ok(self.runtime()?.candidate())
    }

    pub fn last_admitted(&self) -> Option<&ExecutableStateRevisionV1> {
        self.last_admitted.as_ref()
    }

    pub fn carrier(&self) -> Result<&ProcessCarrier, PersistentProcessSessionErrorV1> {
        Ok(self.runtime()?.carrier().carrier())
    }

    fn runtime(&self) -> Result<&ExecutableProcessRuntimeV1, PersistentProcessSessionErrorV1> {
        self.runtime
            .as_ref()
            .ok_or(PersistentProcessSessionErrorV1::Disposed)
    }

    fn runtime_mut(
        &mut self,
    ) -> Result<&mut ExecutableProcessRuntimeV1, PersistentProcessSessionErrorV1> {
        self.runtime
            .as_mut()
            .ok_or(PersistentProcessSessionErrorV1::Disposed)
    }
}

#[derive(Debug)]
pub enum PersistentProcessSessionErrorV1 {
    Executable(ExecutableErrorV1),
    Carrier(ExecutableCarrierErrorV1),
    Authority(AuthorityError),
    Disposed,
}

impl fmt::Display for PersistentProcessSessionErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Executable(error) => write!(formatter, "persistent executable error: {error}"),
            Self::Carrier(error) => write!(formatter, "persistent carrier error: {error}"),
            Self::Authority(error) => write!(formatter, "persistent authority error: {error}"),
            Self::Disposed => formatter.write_str("persistent process session is disposed"),
        }
    }
}

impl Error for PersistentProcessSessionErrorV1 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Executable(error) => Some(error),
            Self::Carrier(error) => Some(error),
            Self::Authority(error) => Some(error),
            Self::Disposed => None,
        }
    }
}

impl From<ExecutableErrorV1> for PersistentProcessSessionErrorV1 {
    fn from(error: ExecutableErrorV1) -> Self {
        Self::Executable(error)
    }
}

impl From<ExecutableCarrierErrorV1> for PersistentProcessSessionErrorV1 {
    fn from(error: ExecutableCarrierErrorV1) -> Self {
        Self::Carrier(error)
    }
}
