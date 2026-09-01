use std::error::Error;
use std::fmt;

use clause_package::{
    ActivationId, AdmissionAuthorizationEvidence, ApplicationId, AuthorityError, AuthorityStore,
    CheckedProcessPackage, ConfigurationId, EffectAttemptId, EffectAttemptOccurrenceV1,
    EffectIntentId, EffectIntentOccurrenceV1, IssuedAdmissionAuthorizationOccurrenceId,
    IssuedEffectAuthorizationOccurrenceId, IssuedEffectAuthorizationV1, ProcessCarrier,
    ProcessPackageId, ProgramRevisionId, RootPolicyAnchor, RunId, RuntimeSessionId,
    StateRevisionId,
};

use super::{
    ExecutableAuthorityFactsV1, ExecutableCandidateV1, ExecutableCarrierErrorV1,
    ExecutableEffectSettlementV1, ExecutableErrorV1, ExecutableInputSourceV1,
    ExecutablePhysicalPlanV1, ExecutableProcessRuntimeV1, ExecutableProjectedObservationV1,
    ExecutableResumptionV1, ExecutableSlotV1, ExecutableStateRevisionV1, ExecutableStepV1,
    ExecutableSuspensionV1, RuntimeAllocationEpochV1, decode_executable_occurrence_v1,
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
    allocation: RuntimeAllocationEpochV1,
    last_admitted: Option<ExecutableStateRevisionV1>,
}

/// Physically owned state from a semantically retired persistent session.
///
/// This carrier exposes no execution or authority surface. Keeping it as a
/// distinct type lets a boundary revoke a session synchronously while moving
/// only destruction of its retained history to a later physical lifecycle
/// point.
pub(crate) struct RetiredPersistentProcessRuntimeV1 {
    runtime: ExecutableProcessRuntimeV1,
}

impl RetiredPersistentProcessRuntimeV1 {
    pub(crate) fn reclaim(&mut self, maximum_entries: usize) -> bool {
        self.runtime.reclaim_retired_entries(maximum_entries)
    }
}

impl PersistentProcessSessionV1 {
    /// Open one native session by transferring ownership of the checked
    /// package and authority store into its runtime.
    pub fn open(
        package: CheckedProcessPackage,
        authority: AuthorityStore,
        application: ApplicationId,
        physical_plan: ExecutablePhysicalPlanV1,
        facts: ExecutableAuthorityFactsV1,
    ) -> Result<Self, PersistentProcessSessionErrorV1> {
        let mut runtime = ExecutableProcessRuntimeV1::instantiate_new(
            package,
            authority,
            application,
            physical_plan,
            facts,
        )?;
        runtime.start_carrier_process(facts)?;
        let allocation = runtime.allocation();
        Ok(Self {
            runtime: Some(runtime),
            session: facts.session,
            program_revision: facts.program_revision,
            world_base: facts.initial_state,
            allocation,
            last_admitted: None,
        })
    }

    /// Rematerialize one exact recorded occurrence family. This is deliberately
    /// separate from `open`: replay preserves the allocation epoch, while a
    /// new run always mints a fresh one.
    pub fn rematerialize(
        package: CheckedProcessPackage,
        authority: AuthorityStore,
        application: ApplicationId,
        physical_plan: ExecutablePhysicalPlanV1,
        facts: ExecutableAuthorityFactsV1,
        allocation: RuntimeAllocationEpochV1,
    ) -> Result<Self, PersistentProcessSessionErrorV1> {
        let mut runtime = ExecutableProcessRuntimeV1::instantiate_rematerialized(
            package,
            authority,
            application,
            physical_plan,
            facts,
            allocation,
        )?;
        runtime.start_carrier_process(facts)?;
        Ok(Self {
            runtime: Some(runtime),
            session: facts.session,
            program_revision: facts.program_revision,
            world_base: facts.initial_state,
            allocation,
            last_admitted: None,
        })
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

    pub fn suspend(&mut self) -> Result<ExecutableSuspensionV1, PersistentProcessSessionErrorV1> {
        Ok(self.runtime_mut()?.suspend_carrier_process()?)
    }

    pub fn resume(&mut self) -> Result<ExecutableResumptionV1, PersistentProcessSessionErrorV1> {
        Ok(self.runtime_mut()?.resume_carrier_process()?)
    }

    pub fn emit_effect_intent(
        &mut self,
    ) -> Result<EffectIntentOccurrenceV1, PersistentProcessSessionErrorV1> {
        Ok(self.runtime_mut()?.emit_carrier_effect_intent()?)
    }

    pub fn pending_effect_intent(
        &self,
    ) -> Result<Option<&EffectIntentOccurrenceV1>, PersistentProcessSessionErrorV1> {
        let Some(intent) = self.runtime()?.pending_carrier_effect_intent() else {
            return Ok(None);
        };
        Ok(self.runtime()?.carrier().carrier().effect_intent(intent))
    }

    pub fn issue_effect_authorization(
        &mut self,
        intent: EffectIntentId,
    ) -> Result<IssuedEffectAuthorizationV1, PersistentProcessSessionErrorV1> {
        Ok(self
            .runtime_mut()?
            .issue_carrier_effect_authorization(intent)?)
    }

    pub fn begin_effect_attempt(
        &mut self,
        authorization: IssuedEffectAuthorizationOccurrenceId,
    ) -> Result<EffectAttemptOccurrenceV1, PersistentProcessSessionErrorV1> {
        Ok(self
            .runtime_mut()?
            .begin_carrier_effect_attempt(authorization)?)
    }

    pub fn settle_effect_attempt(
        &mut self,
        attempt: EffectAttemptId,
        receipt: Option<(u32, Vec<u8>)>,
    ) -> Result<ExecutableEffectSettlementV1, PersistentProcessSessionErrorV1> {
        Ok(self
            .runtime_mut()?
            .settle_carrier_effect_attempt(attempt, receipt)?)
    }

    /// Lower one checked, construct-blind physical observation through the
    /// session's package-Role-indexed physical plan.
    pub fn apply_physical_input(
        &mut self,
        source: &ExecutableInputSourceV1,
    ) -> Result<ExecutableStepV1, PersistentProcessSessionErrorV1> {
        Ok(self.runtime_mut()?.advance_carrier_input(source)?.clone())
    }

    /// Lower the exact fixed tick and emit one candidate without Admission.
    pub fn apply_fixed_tick_and_emit_candidate(
        &mut self,
        fixed_tick_milliseconds: u32,
    ) -> Result<ExecutableStepV1, PersistentProcessSessionErrorV1> {
        Ok(self
            .runtime_mut()?
            .advance_carrier_tick_and_emit_candidate(fixed_tick_milliseconds)?
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

    pub fn issue_candidate_admission_authorization(
        &mut self,
    ) -> Result<IssuedAdmissionAuthorizationOccurrenceId, PersistentProcessSessionErrorV1> {
        Ok(self
            .runtime_mut()?
            .issue_candidate_admission_authorization()?)
    }

    pub fn admit_issued_candidate_with_projection(
        &mut self,
        occurrence: IssuedAdmissionAuthorizationOccurrenceId,
    ) -> Result<
        (
            ExecutableStateRevisionV1,
            Option<ExecutableProjectedObservationV1>,
        ),
        PersistentProcessSessionErrorV1,
    > {
        self.admit_candidate_with_projection(AdmissionAuthorizationEvidence::Issued { occurrence })
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

    /// Admit the current candidate through the sole exact grant already
    /// constituted by this session's admitted Program revision.
    ///
    /// The candidate does not create authority, and this operation neither
    /// establishes nor mutates a root policy. Missing or ambiguous exact
    /// grants fail before Judgment or Admission becomes visible.
    pub fn admit_constituted_candidate_with_projection(
        &mut self,
    ) -> Result<
        (
            ExecutableStateRevisionV1,
            Option<ExecutableProjectedObservationV1>,
        ),
        PersistentProcessSessionErrorV1,
    > {
        let candidate = self
            .candidate()?
            .ok_or(ExecutableCarrierErrorV1::ConstitutiveAdmissionAuthorityUnavailable)?
            .clone();
        let package = self.package()?;
        let exact_scope = clause_package::CheckedStateAdmissionScope {
            package,
            session: self.session,
            base: self.world_base,
            delta: candidate.id,
        };
        if candidate.base != self.world_base {
            return Err(ExecutableCarrierErrorV1::ConstitutiveAdmissionAuthorityUnavailable.into());
        }
        let authorization = self
            .runtime()?
            .carrier()
            .unique_revision_state_admission_authorization(self.program_revision, exact_scope)
            .ok_or(ExecutableCarrierErrorV1::ConstitutiveAdmissionAuthorityUnavailable)?;
        self.admit_candidate_with_projection(AdmissionAuthorizationEvidence::ProgramConstitution {
            revision: self.program_revision,
            authorization,
        })
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
        self.retire_runtime().is_some()
    }

    /// Revoke every live execution and authority surface while transferring
    /// physical ownership of retained history to the caller for later drop.
    pub(crate) fn retire_runtime(&mut self) -> Option<RetiredPersistentProcessRuntimeV1> {
        self.runtime
            .take()
            .map(|runtime| RetiredPersistentProcessRuntimeV1 { runtime })
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

    #[must_use]
    pub const fn allocation(&self) -> RuntimeAllocationEpochV1 {
        self.allocation
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

    pub fn configuration(&self) -> Result<&[ExecutableSlotV1], PersistentProcessSessionErrorV1> {
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

    pub fn authority_facts(
        &self,
    ) -> Result<ExecutableAuthorityFactsV1, PersistentProcessSessionErrorV1> {
        self.runtime()?
            .authority_facts()
            .ok_or(PersistentProcessSessionErrorV1::Carrier(
                ExecutableCarrierErrorV1::NotStarted,
            ))
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
