//! Process-v2 checked carrier.
//!
//! Canonical package data is inert. A [`CheckedProcessPackage`] retains its
//! exact decoded bytes and checked snapshot binding; replay additionally
//! requires an independently established [`AuthorityStore`].

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::authority::{
    AuthorityStore, CheckedSnapshotAuthorityInput, RevisionJudgmentAuthorityGrant,
    RevisionStateAdmissionGrant, RevisionStaticExecutionGrant, RevisionSuccessorGrant,
};
use crate::canonical::ProgramSnapshotPreimageV2;
use crate::formation::{
    ContinuationUseV2, FormationTargetV2, ResolvedProgramConstitutionV2, SemanticDependencyV2,
};
use crate::identity::*;
use crate::provenance::{
    ActivationPrerequisite, ActivationPrerequisiteKind, ActivationStaticBasis,
    CancellationOccurrenceV2, CausalRef, EnteredOccurrenceKind, EnteredThrough,
    ExternalTriggerOccurrenceV2, HandoffOccurrenceV2, JudgmentAuthorityEvidence,
    JudgmentOccurrenceV2, OccurrenceProvenance, PrerequisiteScope, ResumptionOccurrenceV2,
    StateAdmissionDecisionV2, StateAdmissionOutcomeV2, StepRef, SupportSource, SupportUse,
};
use crate::term::Term;

const MAX_PROCESS_RECORDS: usize = 1_000_000;
const MAX_RUNS: usize = 1_000_000;
const MAX_ACTIVATIONS: usize = 1_000_000;
const MAX_CONFIGURATIONS: usize = 1_000_000;
const MAX_STEP_BATCH_ITEMS: usize = 1_000_000;
const MAX_CARRIER_BYTES: usize = 256 * 1024 * 1024;
const MAX_STEP_FRONTIER_ITEMS: usize = 1_000_000;
const MAX_STEP_OBSERVATIONS: usize = 1_000_000;
const MAX_CAUSAL_OCCURRENCES: usize = 4_000_000;
const MAX_CAUSAL_EDGES: usize = 4_000_000;

/// Candidate package. Its claimed snapshot and records remain inert.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessPackageV2 {
    pub claimed_snapshot: ProgramSnapshotId,
    pub snapshot: ProgramSnapshotPreimageV2,
    pub initial_state_views: Vec<InitialStateViewV2>,
    pub records: Vec<ProcessRecordV2>,
}

/// Exact decoded view of a separately authoritative initial State revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitialStateViewV2 {
    pub session: RuntimeSessionId,
    pub payload: Term,
    pub canonical_state_snapshot: Box<[u8]>,
}

/// Exact-byte-bound checked package. Construction is crate-private.
#[derive(Clone, Debug)]
pub struct CheckedProcessPackage {
    id: ProcessPackageId,
    exact_bytes: Box<[u8]>,
    canonical_snapshot_preimage: Box<[u8]>,
    constitution: ResolvedProgramConstitutionV2,
    authority_input: CheckedSnapshotAuthorityInput,
    initial_state_views: Box<[InitialStateViewV2]>,
    records: Box<[ProcessRecordV2]>,
}

impl CheckedProcessPackage {
    #[expect(
        clippy::too_many_arguments,
        reason = "the checked package boundary retains every exact decoded component"
    )]
    pub(crate) fn from_checked_parts(
        id: ProcessPackageId,
        exact_bytes: Vec<u8>,
        canonical_snapshot_preimage: Vec<u8>,
        constitution: ResolvedProgramConstitutionV2,
        successor_grants: Vec<RevisionSuccessorGrant>,
        static_execution_grants: Vec<RevisionStaticExecutionGrant>,
        state_admission_grants: Vec<RevisionStateAdmissionGrant>,
        judgment_authority_grants: Vec<RevisionJudgmentAuthorityGrant>,
        initial_state_views: Vec<InitialStateViewV2>,
        records: Vec<ProcessRecordV2>,
    ) -> Result<Self, ProcessError> {
        validate_record_batch_bounds(&records, false, MAX_PROCESS_RECORDS, MAX_STEP_BATCH_ITEMS)?;
        let authority_input =
            CheckedSnapshotAuthorityInput::from_checked_process_package_parts_with_governance(
                id,
                constitution.semantics(),
                constitution.snapshot(),
                canonical_snapshot_preimage.clone(),
                successor_grants,
                static_execution_grants,
                state_admission_grants,
                judgment_authority_grants,
                Some(&constitution),
            )
            .map_err(ProcessError::Authority)?;
        Ok(Self {
            id,
            exact_bytes: exact_bytes.into_boxed_slice(),
            canonical_snapshot_preimage: canonical_snapshot_preimage.into_boxed_slice(),
            constitution,
            authority_input,
            initial_state_views: initial_state_views.into_boxed_slice(),
            records: records.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> ProcessPackageId {
        self.id
    }

    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.exact_bytes
    }

    #[must_use]
    pub fn canonical_snapshot_preimage(&self) -> &[u8] {
        &self.canonical_snapshot_preimage
    }

    #[must_use]
    pub fn constitution(&self) -> &ResolvedProgramConstitutionV2 {
        &self.constitution
    }

    #[must_use]
    pub fn authority_input(&self) -> &CheckedSnapshotAuthorityInput {
        &self.authority_input
    }

    #[must_use]
    pub fn initial_state_views(&self) -> &[InitialStateViewV2] {
        &self.initial_state_views
    }

    #[must_use]
    pub fn records(&self) -> &[ProcessRecordV2] {
        &self.records
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutionScope {
    pub application: ApplicationId,
    pub mode: ModeId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ExecutionAuthorizationEvidence {
    ProgramConstitution {
        revision: ProgramRevisionId,
        authorization: ExecutionAuthorizationRef,
    },
    IrreducibleRoot {
        policy: RootPolicyId,
        authorization: RootExecutionAuthorizationRef,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AdmissionAuthorizationEvidence {
    ProgramConstitution {
        revision: ProgramRevisionId,
        authorization: AdmissionAuthorizationRef,
    },
    IrreducibleRoot {
        policy: RootPolicyId,
        authorization: RootAdmissionAuthorizationRef,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Budget {
    pub remaining_units: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CancellationScope {
    Activation,
    Run,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActivationPins {
    pub semantics: ClauseSemanticsId,
    pub snapshot: ProgramSnapshotId,
    pub program_revision: ProgramRevisionId,
    pub runtime_session: Option<RuntimeSessionId>,
    pub observed_state: Option<StateRevisionId>,
    pub runtime_policy: Option<RuntimePolicyId>,
    pub context_requirements: Vec<FormationRefV2>,
    pub constitutive_dependencies: Vec<SemanticDependencyV2>,
    pub capabilities: Vec<CapabilityRef>,
    pub scheduling_requirements: Vec<FormationRefV2>,
    pub resource_requirements: Vec<FormationRefV2>,
    pub cancellation_scope: CancellationScope,
    pub budget: Budget,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RootTrigger {
    External(ExternalTriggerOccurrenceId),
    SessionStart(SessionStartOccurrenceId),
    Admitted(AdmissionOccurrenceId),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationOrigin {
    RootedBy(RootTrigger),
    ChildOf {
        run: RunId,
        parent_activation: ActivationId,
        parent_step: StepId,
    },
    HandoffFrom {
        run: RunId,
        parent_activation: ActivationId,
        parent_step: StepId,
        continuation: ContinuationId,
        handoff: HandoffOccurrenceId,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RunMembership {
    RootOf(RunId),
    ChildIn(RunId),
}

impl RunMembership {
    #[must_use]
    pub const fn run(self) -> RunId {
        match self {
            Self::RootOf(run) | Self::ChildIn(run) => run,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActivationPrerequisiteUseV2 {
    pub kind: FormationRefV2,
    pub prerequisite: ActivationPrerequisite,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationCauseFrontierV2 {
    pub origin: ActivationOrigin,
    pub prerequisites: Vec<ActivationPrerequisiteUseV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationProposal {
    pub id: ConfigurationId,
    pub value: Term,
}

/// The exact serial custody edge that constitutes one Configuration.
/// Semantic Step causality remains independent and may be empty after the
/// first Step when this predecessor supplies configuration succession.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ConfigurationPredecessorV2 {
    ActivationStart(ActivationId),
    ConfigurationAfter(StepRef),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationProposalV2 {
    pub id: ActivationId,
    pub application: ApplicationId,
    pub mode: ModeId,
    pub pins: ActivationPins,
    pub static_basis: ActivationStaticBasis,
    pub causes: ActivationCauseFrontierV2,
    pub membership: RunMembership,
    pub initial_configuration: ConfigurationProposal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationTerminal {
    Returned,
    Failed,
    Cancelled,
    BudgetExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationStatus {
    Ready,
    Live,
    Suspended(ContinuationId),
    Transferred(ContinuationId),
    Terminal(ActivationTerminal),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Application {
    id: ApplicationId,
    shape: ApplicationShapeId,
}

impl Application {
    #[must_use]
    pub const fn id(&self) -> ApplicationId {
        self.id
    }

    #[must_use]
    pub const fn shape(&self) -> ApplicationShapeId {
        self.shape
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activation {
    proposal: ActivationProposalV2,
    status: ActivationStatus,
    latest_configuration: ConfigurationId,
    start_causes: Box<[CausalRef]>,
    remaining_budget: Budget,
}

impl Activation {
    #[must_use]
    pub fn proposal(&self) -> &ActivationProposalV2 {
        &self.proposal
    }

    #[must_use]
    pub const fn id(&self) -> ActivationId {
        self.proposal.id
    }

    #[must_use]
    pub const fn application(&self) -> ApplicationId {
        self.proposal.application
    }

    #[must_use]
    pub const fn mode(&self) -> ModeId {
        self.proposal.mode
    }

    #[must_use]
    pub fn pins(&self) -> &ActivationPins {
        &self.proposal.pins
    }

    #[must_use]
    pub const fn membership(&self) -> RunMembership {
        self.proposal.membership
    }

    #[must_use]
    pub const fn status(&self) -> ActivationStatus {
        self.status
    }

    #[must_use]
    pub const fn latest_configuration(&self) -> ConfigurationId {
        self.latest_configuration
    }

    #[must_use]
    pub fn start_causes(&self) -> &[CausalRef] {
        &self.start_causes
    }

    #[must_use]
    pub const fn remaining_budget(&self) -> Budget {
        self.remaining_budget
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Configuration {
    pub id: ConfigurationId,
    pub activation: ActivationId,
    pub predecessor: ConfigurationPredecessorV2,
    pub value: Term,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContinuationPins {
    pub run: RunId,
    pub activation: ActivationId,
    pub application: ApplicationId,
    pub mode: ModeId,
    pub activation_pins: ActivationPins,
    pub remaining_budget: Budget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationProposalV2 {
    pub id: ContinuationId,
    pub emitted_by: StepId,
    pub pins: ContinuationPins,
    pub remainder: Term,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Continuation {
    proposal: ContinuationProposalV2,
    use_policy: ContinuationUseV2,
    takeups: BTreeSet<ContinuationTakeupOccurrence>,
}

impl Continuation {
    #[must_use]
    pub fn proposal(&self) -> &ContinuationProposalV2 {
        &self.proposal
    }

    #[must_use]
    pub const fn use_policy(&self) -> ContinuationUseV2 {
        self.use_policy
    }

    #[must_use]
    pub fn takeups(&self) -> &BTreeSet<ContinuationTakeupOccurrence> {
        &self.takeups
    }

    #[must_use]
    pub fn consumed(&self) -> bool {
        self.use_policy == ContinuationUseV2::Linear && !self.takeups.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CancellationTarget {
    Activation(ActivationId),
    Run(RunId),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContinuationTakeupOccurrence {
    Resumption(ResumptionOccurrenceId),
    Handoff(HandoffOccurrenceId),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StepCause {
    ActivationStart(ActivationId),
    PriorStep(StepRef),
    ContinuationTakeup {
        continuation: ContinuationId,
        occurrence: ContinuationTakeupOccurrence,
    },
    CancellationRequest(CancellationOccurrenceId),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TruthVerdict {
    True,
    False,
    Absent,
}

/// `Absent` is explicit but has no ObservationId and creates no occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "an Observation proposal owns its exact typed formation target without hidden allocation"
)]
pub enum ObservationProposalV2 {
    Value {
        id: ObservationId,
        value: Term,
        supports: Vec<SupportUse>,
    },
    Truth {
        id: Option<ObservationId>,
        verdict: TruthVerdict,
        proposition: Term,
        supports: Vec<SupportUse>,
    },
    Formation {
        id: ObservationId,
        subject: Term,
        target: FormationTargetV2,
        supports: Vec<SupportUse>,
    },
}

impl ObservationProposalV2 {
    #[must_use]
    pub const fn occurrence_id(&self) -> Option<ObservationId> {
        match self {
            Self::Value { id, .. } | Self::Formation { id, .. } => Some(*id),
            Self::Truth { id, .. } => *id,
        }
    }

    #[must_use]
    pub fn supports(&self) -> &[SupportUse] {
        match self {
            Self::Value { supports, .. }
            | Self::Truth { supports, .. }
            | Self::Formation { supports, .. } => supports,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnteredObservationV2 {
    pub observation: ObservationProposalV2,
    pub provenance: EnteredThrough,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    pub id: ObservationId,
    pub content: ObservationContentV2,
    pub provenance: OccurrenceProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "an Observation owns its exact typed formation target without hidden allocation"
)]
pub enum ObservationContentV2 {
    Value {
        value: Term,
        supports: Vec<SupportUse>,
    },
    Truth {
        verdict: TruthVerdict,
        proposition: Term,
        supports: Vec<SupportUse>,
    },
    Formation {
        subject: Term,
        target: FormationTargetV2,
        supports: Vec<SupportUse>,
    },
}

/// One runtime Term tied to occurrence-exact prior Formation evidence. The
/// carrier preserves the exact domain/evidence relation; it does not infer
/// host-language meaning or let a Step establish its own output domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainBoundTermV2 {
    pub term: Term,
    pub evidence: ObservationId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepOutcomeProposalV2 {
    Progress,
    Suspend(ContinuationProposalV2),
    Return(DomainBoundTermV2),
    Fail(DomainBoundTermV2),
    Cancel(CancellationOccurrenceId),
    BudgetExhausted {
        exhaustion: DomainBoundTermV2,
        continuation: Option<ContinuationProposalV2>,
        obligations: Vec<Term>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepProposalV2 {
    pub id: StepId,
    pub run: RunId,
    pub activation: ActivationId,
    pub before: ConfigurationId,
    pub after: ConfigurationProposal,
    pub observed_state: Option<StateRevisionId>,
    pub budget: StepBudgetTransitionV2,
    pub causes: Vec<StepCause>,
    pub observations: Vec<ObservationProposalV2>,
    pub candidate_delta: Option<crate::provenance::CandidateDeltaV2>,
    pub outcome: StepOutcomeProposalV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepBudgetTransitionV2 {
    pub before: Budget,
    pub consumed_units: u64,
    pub after: Budget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Step {
    proposal: StepProposalV2,
}

impl Step {
    #[must_use]
    pub fn proposal(&self) -> &StepProposalV2 {
        &self.proposal
    }

    #[must_use]
    pub const fn reference(&self) -> StepRef {
        StepRef {
            run: self.proposal.run,
            activation: self.proposal.activation,
            step: self.proposal.id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateDelta {
    pub proposal: crate::provenance::CandidateDeltaV2,
    pub produced_by: StepRef,
    pub package: ProcessPackageId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateRevisionCause {
    SessionStart(SessionStartOccurrenceId),
    Admission {
        occurrence: AdmissionOccurrenceId,
        run: RunId,
        activation: ActivationId,
        step: StepId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateRevision {
    pub id: StateRevisionId,
    pub session: RuntimeSessionId,
    pub predecessor: Option<StateRevisionId>,
    pub cause: StateRevisionCause,
    pub payload: Term,
    pub canonical_state_snapshot: Box<[u8]>,
    pub policy: RuntimePolicyId,
    pub semantics: ClauseSemanticsId,
}

impl StateRevision {
    /// Derive the identity committed by this complete revision preimage.
    #[must_use]
    pub fn derived_id(&self) -> StateRevisionId {
        derive_successor_state_id(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessRecordV2 {
    ExternalTrigger(ExternalTriggerOccurrenceV2),
    EnteredObservation(EnteredObservationV2),
    Activation(ActivationProposalV2),
    Resumption(ResumptionOccurrenceV2),
    Handoff(HandoffOccurrenceV2),
    Cancellation(CancellationOccurrenceV2),
    Steps(Vec<StepProposalV2>),
    Judgment(JudgmentOccurrenceV2),
    AdmissionDecision(StateAdmissionDecisionV2),
}

/// Current cumulative carrier usage at the constitutional live limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessResourceUsageV2 {
    pub base_records: usize,
    pub accepted_ingress_records: usize,
    pub base_package_bytes: usize,
    pub accepted_ingress_bytes: usize,
    pub runs: usize,
    pub activations: usize,
    pub configurations: usize,
}

#[derive(Clone, Debug)]
pub struct ProcessCarrier {
    package: ProcessPackageId,
    exact_package_bytes: Box<[u8]>,
    constitution: ResolvedProgramConstitutionV2,
    applications: BTreeMap<ApplicationId, Application>,
    activations: BTreeMap<ActivationId, Activation>,
    runs: BTreeMap<RunId, ActivationId>,
    run_members: BTreeMap<RunId, BTreeSet<ActivationId>>,
    configurations: BTreeMap<ConfigurationId, Configuration>,
    steps: BTreeMap<StepId, Step>,
    observations: BTreeMap<ObservationId, Observation>,
    continuations: BTreeMap<ContinuationId, Continuation>,
    candidate_deltas: BTreeMap<CandidateDeltaId, CandidateDelta>,
    judgments: BTreeMap<JudgmentOccurrenceId, JudgmentOccurrenceV2>,
    decisions: BTreeMap<CandidateDeltaId, StateAdmissionDecisionV2>,
    decisions_by_occurrence: BTreeMap<AdmissionOccurrenceId, CandidateDeltaId>,
    states: BTreeMap<StateRevisionId, StateRevision>,
    external_triggers: BTreeMap<ExternalTriggerOccurrenceId, ExternalTriggerOccurrenceV2>,
    resumptions: BTreeMap<ResumptionOccurrenceId, ResumptionOccurrenceV2>,
    handoffs: BTreeMap<HandoffOccurrenceId, HandoffOccurrenceV2>,
    cancellations: BTreeMap<CancellationOccurrenceId, CancellationOccurrenceV2>,
    causal_predecessors: BTreeMap<CausalRef, BTreeSet<CausalRef>>,
    causal_edge_count: usize,
    base_record_count: usize,
    applied_base_record_count: usize,
    accepted_ingress_record_count: usize,
    accepted_ingress_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RecordBatchCardinality {
    records: usize,
    runs: usize,
    activations: usize,
    configurations: usize,
    steps: usize,
}

struct PreparedActivationOrigin {
    causes: Vec<CausalRef>,
}

struct StepUndo {
    activation: ActivationId,
    previous_status: ActivationStatus,
    previous_configuration: ConfigurationId,
    previous_budget: Budget,
    step: StepId,
    configuration: ConfigurationId,
    observations: Vec<ObservationId>,
    candidate_delta: Option<CandidateDeltaId>,
    continuation: Option<ContinuationId>,
    continuation_takeup: Option<(ContinuationId, ContinuationTakeupOccurrence)>,
    causal_occurrences: Vec<CausalRef>,
}

enum RecordUndo {
    None,
    ExternalTrigger(ExternalTriggerOccurrenceId),
    Observation(ObservationId),
    Activation {
        activation: ActivationId,
        run: RunId,
        root: bool,
        configuration: ConfigurationId,
    },
    Resumption(ResumptionOccurrenceId),
    Handoff(HandoffOccurrenceId),
    Cancellation(CancellationOccurrenceId),
    Steps(Vec<StepUndo>),
    Judgment(JudgmentOccurrenceId),
    Admission {
        delta: CandidateDeltaId,
        occurrence: AdmissionOccurrenceId,
        state: Option<StateRevisionId>,
    },
}

struct RecordApplyFailure {
    step_index: Option<usize>,
    cause: Box<ProcessError>,
}

impl From<ProcessError> for RecordApplyFailure {
    fn from(cause: ProcessError) -> Self {
        Self {
            step_index: None,
            cause: Box::new(cause),
        }
    }
}

struct StepApplyFailure {
    step_index: Option<usize>,
    cause: Box<ProcessError>,
}

impl From<StepApplyFailure> for RecordApplyFailure {
    fn from(failure: StepApplyFailure) -> Self {
        Self {
            step_index: failure.step_index,
            cause: failure.cause,
        }
    }
}

impl ProcessCarrier {
    pub fn replay(
        package: &CheckedProcessPackage,
        authority: &AuthorityStore,
    ) -> Result<Self, ProcessError> {
        let mut carrier = Self::instantiate(package, authority)?;
        while carrier.advance_package(package, authority)?.is_some() {}
        Ok(carrier)
    }

    /// Instantiate the checked package without executing any of its process
    /// records. The complete record surface is rejected up front when it asks
    /// for execution that the serial carrier does not support.
    pub fn instantiate(
        package: &CheckedProcessPackage,
        authority: &AuthorityStore,
    ) -> Result<Self, ProcessError> {
        let mut carrier = Self::empty(package)?;
        carrier.preflight_supported_records(package.records())?;
        carrier.bind_initial_states(package.initial_state_views(), authority)?;
        Ok(carrier)
    }

    /// Execute the next record from the exact checked package, in package
    /// order. No caller-supplied semantic selector participates in dispatch.
    pub fn advance_package(
        &mut self,
        package: &CheckedProcessPackage,
        authority: &AuthorityStore,
    ) -> Result<Option<ProcessRecordV2>, ProcessError> {
        if package.id != self.package
            || package.exact_bytes.as_ref() != self.exact_package_bytes.as_ref()
            || package.records.len() != self.base_record_count
        {
            return Err(ProcessError::PackageBindingMismatch);
        }
        let Some(record) = package.records.get(self.applied_base_record_count).cloned() else {
            return Ok(None);
        };
        self.apply_record(record.clone(), authority)?;
        self.applied_base_record_count = self
            .applied_base_record_count
            .checked_add(1)
            .expect("checked package record count is bounded");
        Ok(Some(record))
    }

    fn empty(package: &CheckedProcessPackage) -> Result<Self, ProcessError> {
        let mut applications = BTreeMap::new();
        for declaration in &package.constitution.preimage().applications {
            let id = ApplicationId {
                snapshot: package.constitution.snapshot(),
                local: declaration.id,
            };
            let shape = package
                .constitution
                .application_shape(declaration.id)
                .ok_or(ProcessError::UnknownApplication(id))?;
            applications.insert(id, Application { id, shape });
        }
        Ok(Self {
            package: package.id,
            exact_package_bytes: package.exact_bytes.clone(),
            constitution: package.constitution.clone(),
            applications,
            activations: BTreeMap::new(),
            runs: BTreeMap::new(),
            run_members: BTreeMap::new(),
            configurations: BTreeMap::new(),
            steps: BTreeMap::new(),
            observations: BTreeMap::new(),
            continuations: BTreeMap::new(),
            candidate_deltas: BTreeMap::new(),
            judgments: BTreeMap::new(),
            decisions: BTreeMap::new(),
            decisions_by_occurrence: BTreeMap::new(),
            states: BTreeMap::new(),
            external_triggers: BTreeMap::new(),
            resumptions: BTreeMap::new(),
            handoffs: BTreeMap::new(),
            cancellations: BTreeMap::new(),
            causal_predecessors: BTreeMap::new(),
            causal_edge_count: 0,
            base_record_count: package.records.len(),
            applied_base_record_count: 0,
            accepted_ingress_record_count: 0,
            accepted_ingress_bytes: 0,
        })
    }

    fn bind_initial_states(
        &mut self,
        views: &[InitialStateViewV2],
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        let mut sessions = BTreeSet::new();
        for view in views {
            if !sessions.insert(view.session) {
                return Err(ProcessError::DuplicateInitialStateView(view.session));
            }
            let session = authority
                .runtime_session(view.session)
                .ok_or(ProcessError::UnknownRuntimeSession(view.session))?;
            let revision = authority.revision(session.program_revision).ok_or(
                ProcessError::UnknownProgramRevision(session.program_revision),
            )?;
            if revision.package() != self.package
                || revision.claim().preimage.snapshot != self.constitution.snapshot()
                || revision.claim().preimage.semantics != self.constitution.semantics()
                || session.semantics != self.constitution.semantics()
                || session.initial_state.canonical_state_snapshot.as_ref()
                    != view.canonical_state_snapshot.as_ref()
                || view.payload.scope().semantics != session.semantics
                || view.payload.scope().universe != self.constitution.universe()
            {
                return Err(ProcessError::InitialStateBindingMismatch(view.session));
            }
            let id = session.initial_state_id();
            let state = StateRevision {
                id,
                session: view.session,
                predecessor: None,
                cause: StateRevisionCause::SessionStart(session.start),
                payload: view.payload.clone(),
                canonical_state_snapshot: view.canonical_state_snapshot.clone(),
                policy: session.policy,
                semantics: session.semantics,
            };
            self.validate_state_payload_binding(&state)?;
            self.states.insert(id, state);
            self.causal_predecessors
                .entry(CausalRef::SessionStart(session.start))
                .or_default();
        }
        Ok(())
    }

    fn apply_record(
        &mut self,
        record: ProcessRecordV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        self.apply_record_with_undo(record, authority)
            .map(|_| ())
            .map_err(|failure| *failure.cause)
    }

    /// Apply one nonempty bounded live-ingress batch through the same checks as
    /// package replay.
    ///
    /// Authority is read-only, and every derived carrier mutation is rolled
    /// back if any later record or nested Step is rejected. Accepted ingress
    /// does not extend the carrier's [`ProcessPackageId`] or exact package
    /// bytes; those continue to identify only the checked replay package.
    pub fn apply_ingress(
        &mut self,
        records: &[ProcessRecordV2],
        authority: &AuthorityStore,
    ) -> Result<(), ProcessIngressError> {
        let cardinality = self.preflight_ingress_cardinality(records)?;
        let ingress_bytes = crate::canonical::canonical_process_record_bytes(records)
            .map_err(|cause| ProcessIngressError::Batch {
                cause: Box::new(ProcessError::Canonical(cause)),
            })?
            .len();
        self.apply_prepared_ingress(records, cardinality, ingress_bytes, authority)
    }

    fn preflight_ingress_cardinality(
        &self,
        records: &[ProcessRecordV2],
    ) -> Result<RecordBatchCardinality, ProcessIngressError> {
        self.preflight_supported_records(records)
            .map_err(|cause| ProcessIngressError::Batch {
                cause: Box::new(cause),
            })?;
        let cardinality =
            validate_record_batch_bounds(records, true, MAX_PROCESS_RECORDS, MAX_STEP_BATCH_ITEMS)
                .map_err(|cause| ProcessIngressError::Batch {
                    cause: Box::new(cause),
                })?;
        let retained_records = checked_resource_add(
            self.base_record_count,
            self.accepted_ingress_record_count,
            MAX_PROCESS_RECORDS,
            ProcessResourceKindV2::Record,
        )
        .and_then(|current| {
            checked_resource_add(
                current,
                cardinality.records,
                MAX_PROCESS_RECORDS,
                ProcessResourceKindV2::Record,
            )
        })
        .map_err(|cause| ProcessIngressError::Batch {
            cause: Box::new(cause),
        })?;
        debug_assert!(retained_records <= MAX_PROCESS_RECORDS);
        for (current, growth, maximum, kind) in [
            (
                self.runs.len(),
                cardinality.runs,
                MAX_RUNS,
                ProcessResourceKindV2::Run,
            ),
            (
                self.activations.len(),
                cardinality.activations,
                MAX_ACTIVATIONS,
                ProcessResourceKindV2::Activation,
            ),
            (
                self.configurations.len(),
                cardinality.configurations,
                MAX_CONFIGURATIONS,
                ProcessResourceKindV2::Configuration,
            ),
        ] {
            checked_resource_add(current, growth, maximum, kind).map_err(|cause| {
                ProcessIngressError::Batch {
                    cause: Box::new(cause),
                }
            })?;
        }
        Ok(cardinality)
    }

    fn preflight_supported_records(&self, records: &[ProcessRecordV2]) -> Result<(), ProcessError> {
        for record in records {
            match record {
                ProcessRecordV2::Activation(proposal) => {
                    let supported_membership = match (proposal.causes.origin, proposal.membership) {
                        (ActivationOrigin::RootedBy(_), RunMembership::RootOf(_)) => true,
                        (
                            ActivationOrigin::ChildOf { run: origin, .. },
                            RunMembership::ChildIn(member),
                        ) => origin == member,
                        _ => false,
                    };
                    if !supported_membership {
                        return Err(ProcessError::ChildActivationUnsupported);
                    }
                    let mode = self.mode_contract(proposal.mode)?;
                    if !mode.contract.effect_intents.is_empty() {
                        return Err(ProcessError::EffectfulModeUnsupported(proposal.mode));
                    }
                }
                ProcessRecordV2::Handoff(_) => {
                    return Err(ProcessError::HandoffUnsupported);
                }
                ProcessRecordV2::Steps(steps) => {
                    for step in steps {
                        if let Some(activation) = self.activations.get(&step.activation) {
                            let mode = self.mode_contract(activation.mode())?;
                            if !mode.contract.effect_intents.is_empty() {
                                return Err(ProcessError::EffectfulModeUnsupported(
                                    activation.mode(),
                                ));
                            }
                        }
                    }
                }
                ProcessRecordV2::ExternalTrigger(_)
                | ProcessRecordV2::EnteredObservation(_)
                | ProcessRecordV2::Resumption(_)
                | ProcessRecordV2::Cancellation(_)
                | ProcessRecordV2::Judgment(_)
                | ProcessRecordV2::AdmissionDecision(_) => {}
            }
        }
        Ok(())
    }

    fn apply_prepared_ingress(
        &mut self,
        records: &[ProcessRecordV2],
        cardinality: RecordBatchCardinality,
        ingress_bytes: usize,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessIngressError> {
        let next_ingress_bytes = self
            .accepted_ingress_bytes
            .checked_add(ingress_bytes)
            .unwrap_or(usize::MAX);
        let carrier_bytes = self
            .exact_package_bytes
            .len()
            .checked_add(next_ingress_bytes)
            .unwrap_or(usize::MAX);
        if carrier_bytes > MAX_CARRIER_BYTES {
            return Err(ProcessIngressError::Batch {
                cause: Box::new(ProcessError::IngressByteLimitExceeded {
                    count: carrier_bytes,
                    maximum: MAX_CARRIER_BYTES,
                }),
            });
        }
        let mut undo = Vec::new();
        undo.try_reserve_exact(records.len())
            .map_err(|_| ProcessIngressError::Batch {
                cause: Box::new(ProcessError::TransactionAllocationFailed),
            })?;
        for (record_index, record) in records.iter().enumerate() {
            match self.apply_record_with_undo(record.clone(), authority) {
                Ok(record_undo) => undo.push(record_undo),
                Err(failure) => {
                    for record_undo in undo.into_iter().rev() {
                        self.rollback_record(record_undo);
                    }
                    return Err(match failure.step_index {
                        Some(step_index) => ProcessIngressError::Step {
                            record_index,
                            step_index,
                            cause: failure.cause,
                        },
                        None => ProcessIngressError::Record {
                            record_index,
                            cause: failure.cause,
                        },
                    });
                }
            }
        }
        self.accepted_ingress_record_count = self
            .accepted_ingress_record_count
            .checked_add(cardinality.records)
            .expect("preflight bounded accepted ingress records");
        self.accepted_ingress_bytes = next_ingress_bytes;
        Ok(())
    }

    fn apply_record_with_undo(
        &mut self,
        record: ProcessRecordV2,
        authority: &AuthorityStore,
    ) -> Result<RecordUndo, RecordApplyFailure> {
        match record {
            ProcessRecordV2::ExternalTrigger(occurrence) => {
                let id = occurrence.id;
                self.add_external_trigger(occurrence, authority)?;
                Ok(RecordUndo::ExternalTrigger(id))
            }
            ProcessRecordV2::EnteredObservation(observation) => {
                let id = observation.observation.occurrence_id();
                self.add_entered_observation(observation, authority)?;
                Ok(id.map_or(RecordUndo::None, RecordUndo::Observation))
            }
            ProcessRecordV2::Activation(proposal) => {
                let activation = proposal.id;
                let run = proposal.membership.run();
                let root = matches!(proposal.membership, RunMembership::RootOf(_));
                let configuration = proposal.initial_configuration.id;
                self.activate(proposal, authority)?;
                Ok(RecordUndo::Activation {
                    activation,
                    run,
                    root,
                    configuration,
                })
            }
            ProcessRecordV2::Resumption(occurrence) => {
                let id = occurrence.body.id;
                self.add_resumption(occurrence, authority)?;
                Ok(RecordUndo::Resumption(id))
            }
            ProcessRecordV2::Handoff(occurrence) => {
                let id = occurrence.body.id;
                self.add_handoff(occurrence, authority)?;
                Ok(RecordUndo::Handoff(id))
            }
            ProcessRecordV2::Cancellation(occurrence) => {
                let id = occurrence.body.id;
                self.add_cancellation(occurrence, authority)?;
                Ok(RecordUndo::Cancellation(id))
            }
            ProcessRecordV2::Steps(proposals) => self
                .apply_steps_with_undo(&proposals)
                .map(RecordUndo::Steps)
                .map_err(RecordApplyFailure::from),
            ProcessRecordV2::Judgment(judgment) => {
                let id = judgment.body.id;
                self.add_judgment(judgment, authority)?;
                Ok(RecordUndo::Judgment(id))
            }
            ProcessRecordV2::AdmissionDecision(decision) => {
                let delta = decision.delta;
                let occurrence = decision.occurrence;
                let state = match &decision.outcome {
                    StateAdmissionOutcomeV2::Admit(successor) => Some(successor.id),
                    StateAdmissionOutcomeV2::Reject(_) => None,
                };
                self.decide_state(decision, authority)?;
                Ok(RecordUndo::Admission {
                    delta,
                    occurrence,
                    state,
                })
            }
        }
    }

    fn rollback_record(&mut self, undo: RecordUndo) {
        match undo {
            RecordUndo::None => {}
            RecordUndo::ExternalTrigger(id) => {
                self.external_triggers.remove(&id);
                self.remove_causal(CausalRef::ExternalTrigger(id));
            }
            RecordUndo::Observation(id) => {
                self.observations.remove(&id);
                self.remove_causal(CausalRef::Observation(id));
            }
            RecordUndo::Activation {
                activation,
                run,
                root,
                configuration,
            } => {
                self.activations.remove(&activation);
                self.configurations.remove(&configuration);
                if root {
                    self.runs.remove(&run);
                    self.run_members.remove(&run);
                } else if let Some(members) = self.run_members.get_mut(&run) {
                    members.remove(&activation);
                }
            }
            RecordUndo::Resumption(id) => {
                self.resumptions.remove(&id);
                self.remove_causal(CausalRef::Resumption(id));
            }
            RecordUndo::Handoff(id) => {
                self.handoffs.remove(&id);
                self.remove_causal(CausalRef::Handoff(id));
            }
            RecordUndo::Cancellation(id) => {
                self.cancellations.remove(&id);
                self.remove_causal(CausalRef::Cancellation(id));
            }
            RecordUndo::Steps(steps) => {
                for step in steps.into_iter().rev() {
                    self.rollback_step(step);
                }
            }
            RecordUndo::Judgment(id) => {
                self.judgments.remove(&id);
                self.remove_causal(CausalRef::Judgment(id));
            }
            RecordUndo::Admission {
                delta,
                occurrence,
                state,
            } => {
                if let Some(state) = state {
                    self.states.remove(&state);
                }
                self.decisions.remove(&delta);
                self.decisions_by_occurrence.remove(&occurrence);
                self.remove_causal(CausalRef::Admission(occurrence));
            }
        }
    }

    fn remove_causal(&mut self, occurrence: CausalRef) {
        if let Some(causes) = self.causal_predecessors.remove(&occurrence) {
            self.causal_edge_count -= causes.len();
        }
    }

    fn add_external_trigger(
        &mut self,
        occurrence: ExternalTriggerOccurrenceV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        if self.external_triggers.contains_key(&occurrence.id) {
            return Err(ProcessError::DuplicateExternalTrigger(occurrence.id));
        }
        self.validate_entered(
            &occurrence.provenance,
            EnteredOccurrenceKind::ExternalTrigger,
            authority,
        )?;
        let reference = CausalRef::ExternalTrigger(occurrence.id);
        self.register_causal(reference, occurrence.provenance.causes.clone())?;
        self.external_triggers.insert(occurrence.id, occurrence);
        Ok(())
    }

    fn add_entered_observation(
        &mut self,
        entered: EnteredObservationV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        self.validate_entered(
            &entered.provenance,
            EnteredOccurrenceKind::Observation,
            authority,
        )?;
        let Some(id) = entered.observation.occurrence_id() else {
            self.validate_observation_proposal(&entered.observation)?;
            return Ok(());
        };
        if self.observations.contains_key(&id) {
            return Err(ProcessError::DuplicateObservation(id));
        }
        let content = self.validate_observation_proposal(&entered.observation)?;
        self.register_causal(
            CausalRef::Observation(id),
            entered.provenance.causes.clone(),
        )?;
        self.observations.insert(
            id,
            Observation {
                id,
                content,
                provenance: OccurrenceProvenance::EnteredThrough(entered.provenance),
            },
        );
        Ok(())
    }

    fn add_resumption(
        &mut self,
        occurrence: ResumptionOccurrenceV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        let id = occurrence.body.id;
        if self.resumptions.contains_key(&id) {
            return Err(ProcessError::DuplicateResumption(id));
        }
        self.validate_continuation_occurrence(
            occurrence.body.continuation,
            occurrence.body.run,
            occurrence.body.activation,
            &occurrence.body.pins,
        )?;
        let OccurrenceProvenance::EnteredThrough(entered) = &occurrence.provenance else {
            return Err(ProcessError::ResumptionRequiresFreshIngress);
        };
        self.validate_entered(entered, EnteredOccurrenceKind::Resumption, authority)?;
        let activation = self
            .activations
            .get(&occurrence.body.activation)
            .ok_or(ProcessError::UnknownActivation(occurrence.body.activation))?;
        self.validate_entered_consumer_pins(entered, activation.pins(), authority)?;
        let causes = self.occurrence_causes(&occurrence.provenance)?;
        self.register_causal(CausalRef::Resumption(id), causes)?;
        self.resumptions.insert(id, occurrence);
        Ok(())
    }

    fn add_handoff(
        &mut self,
        _occurrence: HandoffOccurrenceV2,
        _authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        Err(ProcessError::HandoffUnsupported)
    }

    fn add_cancellation(
        &mut self,
        occurrence: CancellationOccurrenceV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        let id = occurrence.body.id;
        if self.cancellations.contains_key(&id) {
            return Err(ProcessError::DuplicateCancellation(id));
        }
        self.validate_activation_pins(&occurrence.body.pins, authority)?;
        match occurrence.body.target {
            CancellationTarget::Activation(activation) => {
                let activation = self
                    .activations
                    .get(&activation)
                    .ok_or(ProcessError::UnknownActivation(activation))?;
                if activation.pins() != &occurrence.body.pins
                    || activation.pins().cancellation_scope != CancellationScope::Activation
                {
                    return Err(ProcessError::CancellationScopeMismatch);
                }
            }
            CancellationTarget::Run(run) => {
                let members = self
                    .run_members
                    .get(&run)
                    .ok_or(ProcessError::UnknownRun(run))?;
                let mut matched = false;
                for activation in members {
                    let activation =
                        self.activations
                            .get(activation)
                            .ok_or(ProcessError::InternalInvariant(
                                "run member missing Activation",
                            ))?;
                    if activation.pins() == &occurrence.body.pins {
                        matched = true;
                        if activation.pins().cancellation_scope != CancellationScope::Run {
                            return Err(ProcessError::CancellationScopeMismatch);
                        }
                    }
                }
                if !matched {
                    return Err(ProcessError::CancellationScopeMismatch);
                }
            }
        }
        self.validate_occurrence_provenance(
            &occurrence.provenance,
            EnteredOccurrenceKind::Cancellation,
            authority,
        )?;
        self.validate_occurrence_consumer_pins(
            &occurrence.provenance,
            &occurrence.body.pins,
            authority,
        )?;
        let causes = self.occurrence_causes(&occurrence.provenance)?;
        self.register_causal(CausalRef::Cancellation(id), causes)?;
        self.cancellations.insert(id, occurrence);
        Ok(())
    }

    fn validate_entered(
        &self,
        entered: &EnteredThrough,
        kind: EnteredOccurrenceKind,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        crate::provenance::validate_entered_through(entered)?;
        if !authority.external_provenance_is_anchored(entered.boundary, entered.evidence) {
            return Err(ProcessError::UnanchoredExternalProvenance {
                boundary: entered.boundary,
                evidence: entered.evidence,
            });
        }
        let boundary = authority
            .boundary(entered.boundary)
            .ok_or(ProcessError::UnknownBoundary(entered.boundary))?;
        let revision = authority.revision(boundary.program_revision).ok_or(
            ProcessError::UnknownProgramRevision(boundary.program_revision),
        )?;
        if boundary.semantics != self.constitution.semantics()
            || boundary.snapshot != self.constitution.snapshot()
            || revision.package() != self.package
            || revision.claim().preimage.semantics != boundary.semantics
            || revision.claim().preimage.snapshot != boundary.snapshot
        {
            return Err(ProcessError::BoundaryPinMismatch(entered.boundary));
        }
        if boundary.permits.binary_search(&kind).is_err() {
            return Err(ProcessError::BoundaryDoesNotPermit {
                boundary: entered.boundary,
                kind,
            });
        }
        self.validate_causal_frontier(&entered.causes)
    }

    fn validate_entered_consumer_pins(
        &self,
        entered: &EnteredThrough,
        pins: &ActivationPins,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        let boundary = authority
            .boundary(entered.boundary)
            .ok_or(ProcessError::UnknownBoundary(entered.boundary))?;
        if boundary.semantics != pins.semantics
            || boundary.snapshot != pins.snapshot
            || boundary.program_revision != pins.program_revision
            || boundary.runtime_session != pins.runtime_session
            || boundary.runtime_policy != pins.runtime_policy
        {
            return Err(ProcessError::BoundaryConsumerPinMismatch(entered.boundary));
        }
        Ok(())
    }

    fn validate_occurrence_consumer_pins(
        &self,
        provenance: &OccurrenceProvenance,
        pins: &ActivationPins,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        match provenance {
            OccurrenceProvenance::ProducedBy(step) => {
                let producer = self
                    .activations
                    .get(&step.activation)
                    .ok_or(ProcessError::UnknownActivation(step.activation))?;
                if producer.pins().semantics != pins.semantics
                    || producer.pins().snapshot != pins.snapshot
                    || producer.pins().program_revision != pins.program_revision
                    || producer.pins().runtime_session != pins.runtime_session
                    || producer.pins().observed_state != pins.observed_state
                    || producer.pins().runtime_policy != pins.runtime_policy
                {
                    return Err(ProcessError::OccurrenceConsumerPinMismatch);
                }
                Ok(())
            }
            OccurrenceProvenance::EnteredThrough(entered) => {
                self.validate_entered_consumer_pins(entered, pins, authority)
            }
        }
    }

    fn validate_occurrence_provenance(
        &self,
        provenance: &OccurrenceProvenance,
        entered_kind: EnteredOccurrenceKind,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        crate::provenance::validate_occurrence_provenance(provenance)?;
        match provenance {
            OccurrenceProvenance::ProducedBy(step) => {
                self.require_step_ref(*step)?;
                Ok(())
            }
            OccurrenceProvenance::EnteredThrough(entered) => {
                self.validate_entered(entered, entered_kind, authority)
            }
        }
    }

    fn occurrence_causes(
        &self,
        provenance: &OccurrenceProvenance,
    ) -> Result<Vec<CausalRef>, ProcessError> {
        match provenance {
            OccurrenceProvenance::ProducedBy(step) => {
                self.require_step_ref(*step)?;
                Ok(vec![CausalRef::Step(*step)])
            }
            OccurrenceProvenance::EnteredThrough(entered) => Ok(entered.causes.clone()),
        }
    }

    fn validate_runtime_term(&self, term: &Term) -> Result<(), ProcessError> {
        let scope = term.scope();
        if scope.semantics != self.constitution.semantics()
            || scope.universe != self.constitution.universe()
        {
            return Err(ProcessError::RuntimeTermScopeMismatch);
        }
        Ok(())
    }

    fn validate_support_terms(&self, supports: &[SupportUse]) -> Result<(), ProcessError> {
        for support in supports {
            self.validate_runtime_term(&support.role)?;
        }
        Ok(())
    }

    fn validate_state_payload_binding(&self, state: &StateRevision) -> Result<(), ProcessError> {
        self.validate_runtime_term(&state.payload)?;
        let canonical = crate::canonical::canonical_term_bytes(&state.payload)
            .map_err(ProcessError::Canonical)?;
        if canonical.as_slice() != state.canonical_state_snapshot.as_ref() {
            return Err(ProcessError::StatePayloadSnapshotMismatch(state.id));
        }
        Ok(())
    }

    fn activate(
        &mut self,
        proposal: ActivationProposalV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        if self.activations.contains_key(&proposal.id) {
            return Err(ProcessError::DuplicateActivation(proposal.id));
        }
        if self
            .configurations
            .contains_key(&proposal.initial_configuration.id)
        {
            return Err(ProcessError::DuplicateConfiguration(
                proposal.initial_configuration.id,
            ));
        }
        let application = self
            .applications
            .get(&proposal.application)
            .ok_or(ProcessError::UnknownApplication(proposal.application))?;
        let executable = self
            .constitution
            .executable_contract(proposal.application, proposal.mode)
            .ok_or(ProcessError::ModeNotEligible(proposal.mode))?;
        if !self
            .mode_contract(proposal.mode)?
            .contract
            .effect_intents
            .is_empty()
        {
            return Err(ProcessError::EffectfulModeUnsupported(proposal.mode));
        }
        if application.id.snapshot != proposal.pins.snapshot
            || proposal.pins.snapshot != self.constitution.snapshot()
            || proposal.pins.semantics != self.constitution.semantics()
        {
            return Err(ProcessError::ActivationPinMismatch);
        }
        self.validate_runtime_term(&proposal.initial_configuration.value)?;
        self.validate_activation_pins(&proposal.pins, authority)?;
        self.validate_static_basis(&proposal, &executable, authority)?;
        self.validate_dynamic_prerequisites(&proposal, &executable, authority)?;
        let prepared_origin = self.validate_activation_origin(&proposal, authority)?;

        let run = proposal.membership.run();
        match proposal.membership {
            RunMembership::RootOf(run_id) => {
                if !matches!(proposal.causes.origin, ActivationOrigin::RootedBy(_)) {
                    return Err(ProcessError::RunMembershipMismatch);
                }
                if self.runs.contains_key(&run_id) {
                    return Err(ProcessError::DuplicateRun(run_id));
                }
            }
            RunMembership::ChildIn(run_id) => {
                if !matches!(
                    proposal.causes.origin,
                    ActivationOrigin::ChildOf { run, .. } if run == run_id
                ) || !self.runs.contains_key(&run_id)
                {
                    return Err(ProcessError::RunMembershipMismatch);
                }
            }
        }
        if let RunMembership::RootOf(run_id) = proposal.membership {
            self.runs.insert(run_id, proposal.id);
        }
        self.run_members.entry(run).or_default().insert(proposal.id);
        self.configurations.insert(
            proposal.initial_configuration.id,
            Configuration {
                id: proposal.initial_configuration.id,
                activation: proposal.id,
                predecessor: ConfigurationPredecessorV2::ActivationStart(proposal.id),
                value: proposal.initial_configuration.value.clone(),
            },
        );
        let activation_id = proposal.id;
        let configuration = proposal.initial_configuration.id;
        let initial_budget = proposal.pins.budget;
        self.activations.insert(
            activation_id,
            Activation {
                proposal,
                status: ActivationStatus::Ready,
                latest_configuration: configuration,
                start_causes: prepared_origin.causes.into_boxed_slice(),
                remaining_budget: initial_budget,
            },
        );
        Ok(())
    }

    fn validate_activation_pins(
        &self,
        pins: &ActivationPins,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        let revision = authority
            .revision(pins.program_revision)
            .ok_or(ProcessError::UnknownProgramRevision(pins.program_revision))?;
        if revision.package() != self.package
            || revision.claim().preimage.snapshot != pins.snapshot
            || revision.claim().preimage.semantics != pins.semantics
        {
            return Err(ProcessError::ActivationPinMismatch);
        }
        if !is_strictly_sorted_unique(&pins.context_requirements)
            || !is_strictly_sorted_unique(&pins.constitutive_dependencies)
            || !is_strictly_sorted_unique(&pins.capabilities)
            || !is_strictly_sorted_unique(&pins.scheduling_requirements)
            || !is_strictly_sorted_unique(&pins.resource_requirements)
        {
            return Err(ProcessError::NonCanonicalSet("activation pins"));
        }
        match (
            pins.runtime_session,
            pins.observed_state,
            pins.runtime_policy,
        ) {
            (None, None, None) => Ok(()),
            (Some(session_id), Some(state_id), Some(policy)) => {
                let session = authority
                    .runtime_session(session_id)
                    .ok_or(ProcessError::UnknownRuntimeSession(session_id))?;
                let state = self
                    .states
                    .get(&state_id)
                    .ok_or(ProcessError::UnknownStateRevision(state_id))?;
                if session.program_revision != pins.program_revision
                    || session.semantics != pins.semantics
                    || session.policy != policy
                    || state.session != session_id
                    || state.policy != policy
                    || state.semantics != pins.semantics
                {
                    return Err(ProcessError::ActivationPinMismatch);
                }
                Ok(())
            }
            _ => Err(ProcessError::IncompleteRuntimePins),
        }
    }

    fn validate_static_basis(
        &self,
        proposal: &ActivationProposalV2,
        executable: &crate::formation::ExecutableContractV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        crate::provenance::validate_activation_static_basis(&proposal.static_basis)?;
        let mut expected_context = executable.application_context_requirements.clone();
        expected_context.extend_from_slice(&executable.static_basis.context_requirements);
        expected_context.sort_unstable();
        expected_context.dedup();
        let mut expected_dependencies = executable.application_dependency_closure.clone();
        expected_dependencies.extend_from_slice(&executable.static_basis.constitutive_dependencies);
        expected_dependencies.sort_unstable();
        expected_dependencies.dedup();
        if proposal.pins.context_requirements != expected_context
            || proposal.pins.constitutive_dependencies != expected_dependencies
        {
            return Err(ProcessError::StaticBasisMismatch);
        }
        if proposal.pins.capabilities
            != self
                .mode_contract(proposal.mode)?
                .contract
                .capability_requirements
                .iter()
                .map(|local| CapabilityRef {
                    snapshot: proposal.pins.snapshot,
                    local: *local,
                })
                .collect::<Vec<_>>()
            || proposal.pins.scheduling_requirements
                != self
                    .mode_contract(proposal.mode)?
                    .contract
                    .scheduling_requirements
                    .iter()
                    .map(|local| FormationRefV2 {
                        snapshot: proposal.pins.snapshot,
                        local: *local,
                    })
                    .collect::<Vec<_>>()
            || proposal.pins.resource_requirements
                != self
                    .mode_contract(proposal.mode)?
                    .contract
                    .resource_requirements
                    .iter()
                    .map(|local| FormationRefV2 {
                        snapshot: proposal.pins.snapshot,
                        local: *local,
                    })
                    .collect::<Vec<_>>()
        {
            return Err(ProcessError::StaticBasisMismatch);
        }

        for requirement in &executable.authorization_requirements {
            let actual = proposal
                .static_basis
                .execution_authorizations
                .iter()
                .filter(|use_| use_.kind == requirement.kind)
                .count();
            let actual = u32::try_from(actual)
                .map_err(|_| ProcessError::AuthorizationCardinalityMismatch(requirement.kind))?;
            if !requirement.cardinality.contains(actual) {
                return Err(ProcessError::AuthorizationCardinalityMismatch(
                    requirement.kind,
                ));
            }
        }
        for use_ in &proposal.static_basis.execution_authorizations {
            executable
                .authorization_requirements
                .iter()
                .find(|requirement| requirement.kind == use_.kind)
                .ok_or(ProcessError::UnexpectedExecutionAuthorization(use_.kind))?;
            self.validate_execution_authorization(use_, proposal, authority)?;
        }
        self.validate_judgment_authority_basis(proposal, authority)
    }

    fn validate_execution_authorization(
        &self,
        use_: &crate::provenance::ExecutionAuthorizationUseV2,
        proposal: &ActivationProposalV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        let scope = match use_.evidence {
            ExecutionAuthorizationEvidence::ProgramConstitution {
                revision,
                authorization,
            } => {
                if revision != proposal.pins.program_revision {
                    return Err(ProcessError::AuthorityPinMismatch);
                }
                authority.revision_static_execution_scope(revision, authorization)
            }
            ExecutionAuthorizationEvidence::IrreducibleRoot {
                policy,
                authorization,
            } => authority.root_static_execution_scope(policy, authorization),
        }
        .ok_or(ProcessError::UnauthorizedExecution)?;
        if scope.kind != use_.kind
            || scope.application != proposal.application
            || scope.mode != proposal.mode
        {
            return Err(ProcessError::UnauthorizedExecution);
        }
        Ok(())
    }

    fn validate_judgment_authority_basis(
        &self,
        proposal: &ActivationProposalV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        for evidence in &proposal.static_basis.judgment_authorities {
            let scope = match *evidence {
                JudgmentAuthorityEvidence::ProgramConstitution {
                    revision,
                    authority: reference,
                } => {
                    if revision != proposal.pins.program_revision {
                        return Err(ProcessError::AuthorityPinMismatch);
                    }
                    authority.revision_judgment_authority_scope(revision, reference)
                }
                JudgmentAuthorityEvidence::IrreducibleRoot {
                    policy,
                    authority: reference,
                } => authority.root_judgment_authority_scope(policy, reference),
            }
            .ok_or(ProcessError::UnauthorizedJudgment)?;
            if scope.semantics != proposal.pins.semantics
                || Some(scope.session) != proposal.pins.runtime_session
                || Some(scope.policy) != proposal.pins.runtime_policy
            {
                return Err(ProcessError::UnauthorizedJudgment);
            }
        }
        Ok(())
    }

    fn validate_dynamic_prerequisites(
        &self,
        proposal: &ActivationProposalV2,
        executable: &crate::formation::ExecutableContractV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        if !is_strictly_sorted_unique(&proposal.causes.prerequisites) {
            return Err(ProcessError::NonCanonicalSet("activation prerequisites"));
        }
        for requirement in &executable.dynamic_prerequisites {
            let actual = proposal
                .causes
                .prerequisites
                .iter()
                .filter(|use_| use_.kind == requirement.kind)
                .count();
            let actual = u32::try_from(actual)
                .map_err(|_| ProcessError::PrerequisiteCardinalityMismatch(requirement.kind))?;
            if !requirement.cardinality.contains(actual) {
                return Err(ProcessError::PrerequisiteCardinalityMismatch(
                    requirement.kind,
                ));
            }
        }
        for use_ in &proposal.causes.prerequisites {
            let requirement = executable
                .dynamic_prerequisites
                .iter()
                .find(|requirement| requirement.kind == use_.kind)
                .ok_or(ProcessError::UnexpectedPrerequisite(use_.kind))?;
            if use_.prerequisite.kind() != requirement.occurrence_kind {
                return Err(ProcessError::PrerequisiteOccurrenceKindMismatch {
                    requirement: use_.kind,
                    expected: requirement.occurrence_kind,
                    actual: use_.prerequisite.kind(),
                });
            }
            match use_.prerequisite {
                ActivationPrerequisite::Observation(id) => {
                    let observation = self
                        .observations
                        .get(&id)
                        .ok_or(ProcessError::UnknownObservation(id))?;
                    self.validate_prerequisite_scope(
                        &observation.provenance,
                        &proposal.pins,
                        requirement.scope,
                        authority,
                    )?;
                }
                ActivationPrerequisite::Admission(id) => {
                    let delta = self
                        .decisions_by_occurrence
                        .get(&id)
                        .ok_or(ProcessError::UnknownAdmission(id))?;
                    let decision = self
                        .decisions
                        .get(delta)
                        .expect("admission occurrence index retains its decision");
                    if !matches!(decision.outcome, StateAdmissionOutcomeV2::Admit(_)) {
                        return Err(ProcessError::RejectedDecisionIsNotAdmission(id));
                    }
                    self.validate_entered_consumer_pins(
                        &decision.provenance,
                        &proposal.pins,
                        authority,
                    )?;
                    if requirement.scope == PrerequisiteScope::SameObservedState {
                        let StateAdmissionOutcomeV2::Admit(successor) = &decision.outcome else {
                            unreachable!("rejected decision was refused above")
                        };
                        if proposal.pins.observed_state != Some(successor.id) {
                            return Err(ProcessError::PrerequisiteScopeMismatch);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_prerequisite_scope(
        &self,
        provenance: &OccurrenceProvenance,
        pins: &ActivationPins,
        scope: PrerequisiteScope,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        match provenance {
            OccurrenceProvenance::ProducedBy(step) => {
                let activation = self
                    .activations
                    .get(&step.activation)
                    .ok_or(ProcessError::UnknownActivation(step.activation))?;
                let producer = activation.pins();
                let matches = match scope {
                    PrerequisiteScope::SameSemantics => producer.semantics == pins.semantics,
                    PrerequisiteScope::SameProgramRevision => {
                        producer.semantics == pins.semantics
                            && producer.program_revision == pins.program_revision
                    }
                    PrerequisiteScope::SameRuntimeSession => {
                        producer.semantics == pins.semantics
                            && producer.program_revision == pins.program_revision
                            && producer.runtime_session == pins.runtime_session
                            && producer.runtime_policy == pins.runtime_policy
                    }
                    PrerequisiteScope::SameObservedState => {
                        producer.semantics == pins.semantics
                            && producer.program_revision == pins.program_revision
                            && producer.runtime_session == pins.runtime_session
                            && producer.runtime_policy == pins.runtime_policy
                            && producer.observed_state == pins.observed_state
                    }
                };
                if !matches {
                    return Err(ProcessError::PrerequisiteScopeMismatch);
                }
            }
            OccurrenceProvenance::EnteredThrough(entered) => {
                self.validate_entered_consumer_pins(entered, pins, authority)?;
                if scope == PrerequisiteScope::SameObservedState {
                    return Err(ProcessError::PrerequisiteScopeMismatch);
                }
            }
        }
        Ok(())
    }

    fn validate_activation_origin(
        &self,
        proposal: &ActivationProposalV2,
        authority: &AuthorityStore,
    ) -> Result<PreparedActivationOrigin, ProcessError> {
        let mut causes = Vec::with_capacity(proposal.causes.prerequisites.len() + 2);
        match proposal.causes.origin {
            ActivationOrigin::RootedBy(RootTrigger::External(id)) => {
                let trigger = self
                    .external_triggers
                    .get(&id)
                    .ok_or(ProcessError::UnknownExternalTrigger(id))?;
                self.validate_entered_consumer_pins(
                    &trigger.provenance,
                    &proposal.pins,
                    authority,
                )?;
                causes.push(CausalRef::ExternalTrigger(id));
            }
            ActivationOrigin::RootedBy(RootTrigger::SessionStart(start)) => {
                let session_id = proposal
                    .pins
                    .runtime_session
                    .ok_or(ProcessError::RootTriggerPinMismatch)?;
                let state_id = proposal
                    .pins
                    .observed_state
                    .ok_or(ProcessError::RootTriggerPinMismatch)?;
                let state = self
                    .states
                    .get(&state_id)
                    .ok_or(ProcessError::UnknownStateRevision(state_id))?;
                if state.session != session_id
                    || state.predecessor.is_some()
                    || state.cause != StateRevisionCause::SessionStart(start)
                {
                    return Err(ProcessError::RootTriggerPinMismatch);
                }
                causes.push(CausalRef::SessionStart(start));
            }
            ActivationOrigin::RootedBy(RootTrigger::Admitted(admission)) => {
                let delta = self
                    .decisions_by_occurrence
                    .get(&admission)
                    .ok_or(ProcessError::UnknownAdmission(admission))?;
                let decision = self
                    .decisions
                    .get(delta)
                    .expect("admission index retains its decision");
                let StateAdmissionOutcomeV2::Admit(successor) = &decision.outcome else {
                    return Err(ProcessError::RejectedDecisionIsNotAdmission(admission));
                };
                if proposal.pins.observed_state != Some(successor.id)
                    || proposal.pins.runtime_session != Some(successor.session)
                    || proposal.pins.runtime_policy != Some(successor.policy)
                {
                    return Err(ProcessError::RootTriggerPinMismatch);
                }
                causes.push(CausalRef::Admission(admission));
            }
            ActivationOrigin::ChildOf {
                run,
                parent_activation,
                parent_step,
            } => {
                let parent = StepRef {
                    run,
                    activation: parent_activation,
                    step: parent_step,
                };
                self.require_step_ref(parent)?;
                if proposal.membership != RunMembership::ChildIn(run) {
                    return Err(ProcessError::RunMembershipMismatch);
                }
                causes.push(CausalRef::Step(parent));
            }
            ActivationOrigin::HandoffFrom { .. } => {
                return Err(ProcessError::HandoffUnsupported);
            }
        }
        for prerequisite in &proposal.causes.prerequisites {
            causes.push(match prerequisite.prerequisite {
                ActivationPrerequisite::Observation(id) => CausalRef::Observation(id),
                ActivationPrerequisite::Admission(id) => CausalRef::Admission(id),
            });
        }
        causes.sort_unstable();
        causes.dedup();
        self.validate_causal_frontier(&causes)?;
        Ok(PreparedActivationOrigin { causes })
    }

    fn mode_contract(&self, id: ModeId) -> Result<&crate::formation::ModePreimageV2, ProcessError> {
        self.constitution
            .mode_by_id(id)
            .ok_or(ProcessError::ModeNotEligible(id))
    }

    fn validate_continuation_occurrence(
        &self,
        continuation: ContinuationId,
        run: RunId,
        activation: ActivationId,
        pins: &ContinuationPins,
    ) -> Result<(), ProcessError> {
        let continuation_record = self
            .continuations
            .get(&continuation)
            .ok_or(ProcessError::UnknownContinuation(continuation))?;
        if &continuation_record.proposal.pins != pins
            || pins.run != run
            || pins.activation != activation
        {
            return Err(ProcessError::ContinuationPinMismatch);
        }
        let activation_record = self
            .activations
            .get(&activation)
            .ok_or(ProcessError::UnknownActivation(activation))?;
        if activation_record.status != ActivationStatus::Suspended(continuation) {
            return Err(ProcessError::ActivationNotSuspended(activation));
        }
        Ok(())
    }

    fn validate_observation_proposal(
        &self,
        proposal: &ObservationProposalV2,
    ) -> Result<ObservationContentV2, ProcessError> {
        crate::provenance::validate_support_uses(proposal.supports())?;
        match proposal {
            ObservationProposalV2::Value {
                value, supports, ..
            } => {
                self.validate_runtime_term(value)?;
                self.validate_supports(supports)?;
                Ok(ObservationContentV2::Value {
                    value: value.clone(),
                    supports: supports.clone(),
                })
            }
            ObservationProposalV2::Truth {
                id,
                verdict,
                proposition,
                supports,
            } => {
                self.validate_runtime_term(proposition)?;
                match verdict {
                    TruthVerdict::Absent => {
                        if id.is_some() || !supports.is_empty() {
                            return Err(ProcessError::MalformedAbsentObservation);
                        }
                        Ok(ObservationContentV2::Truth {
                            verdict: *verdict,
                            proposition: proposition.clone(),
                            supports: Vec::new(),
                        })
                    }
                    TruthVerdict::True | TruthVerdict::False => {
                        if id.is_none() || supports.is_empty() {
                            return Err(ProcessError::UnsupportedTruthVerdict);
                        }
                        self.validate_supports(supports)?;
                        Ok(ObservationContentV2::Truth {
                            verdict: *verdict,
                            proposition: proposition.clone(),
                            supports: supports.clone(),
                        })
                    }
                }
            }
            ObservationProposalV2::Formation {
                subject,
                target,
                supports,
                ..
            } => {
                self.validate_runtime_term(subject)?;
                self.validate_runtime_term(&target.type_term)?;
                self.validate_runtime_term(&target.interpretation)?;
                self.validate_supports(supports)?;
                Ok(ObservationContentV2::Formation {
                    subject: subject.clone(),
                    target: target.clone(),
                    supports: supports.clone(),
                })
            }
        }
    }

    fn validate_domain_bound_term(
        &self,
        proposal: &StepProposalV2,
        activation: &Activation,
        value: &DomainBoundTermV2,
        expected_domain: &FormationTargetV2,
    ) -> Result<(), ProcessError> {
        self.validate_runtime_term(&value.term)?;
        let evidence = self
            .observations
            .get(&value.evidence)
            .ok_or(ProcessError::MissingPriorFormationEvidence(value.evidence))?;
        let ObservationContentV2::Formation {
            subject,
            target,
            supports,
        } = &evidence.content
        else {
            return Err(ProcessError::FormationEvidenceKindMismatch(value.evidence));
        };
        if subject != &value.term || target != expected_domain {
            return Err(ProcessError::FormationEvidenceDomainMismatch(
                value.evidence,
            ));
        }
        let OccurrenceProvenance::ProducedBy(producer) = evidence.provenance else {
            return Err(ProcessError::FormationEvidenceRequiresCheckerStep(
                value.evidence,
            ));
        };
        if producer.activation == activation.id() {
            return Err(ProcessError::FormationEvidenceRequiresDistinctActivation(
                value.evidence,
            ));
        }
        self.require_step_ref(producer)?;
        let producer_activation = self
            .activations
            .get(&producer.activation)
            .ok_or(ProcessError::UnknownActivation(producer.activation))?;
        if self
            .mode_contract(producer_activation.mode())?
            .contract
            .formation_checks
            .binary_search(target)
            .is_err()
        {
            return Err(ProcessError::FormationEvidenceCheckerTargetMismatch(
                value.evidence,
            ));
        }
        let activation_has_evidence =
            activation
                .proposal
                .causes
                .prerequisites
                .iter()
                .any(|prerequisite| {
                    prerequisite.prerequisite == ActivationPrerequisite::Observation(value.evidence)
                });
        let step_has_evidence = proposal.causes.contains(&StepCause::PriorStep(producer));
        if !activation_has_evidence && !step_has_evidence {
            return Err(ProcessError::FormationEvidenceNotCausal(value.evidence));
        }
        self.validate_supports(supports)
    }

    fn validate_produced_observation_contract(
        &self,
        observation: &ObservationProposalV2,
        mode: &crate::formation::ModePreimageV2,
    ) -> Result<(), ProcessError> {
        let ObservationProposalV2::Formation { target, .. } = observation else {
            return Ok(());
        };
        if mode
            .contract
            .formation_checks
            .binary_search(target)
            .is_err()
        {
            return Err(ProcessError::FormationObservationNotDeclared);
        }
        Ok(())
    }

    fn validate_supports(&self, supports: &[SupportUse]) -> Result<(), ProcessError> {
        crate::provenance::validate_support_uses(supports)?;
        self.validate_support_terms(supports)?;
        for support in supports {
            match support.source {
                SupportSource::SessionStart(start) => {
                    self.require_causal(CausalRef::SessionStart(start))?;
                }
                SupportSource::ExternalTrigger(id) => {
                    self.require_causal(CausalRef::ExternalTrigger(id))?;
                }
                SupportSource::Resumption(id) => {
                    self.require_causal(CausalRef::Resumption(id))?;
                }
                SupportSource::Handoff(id) => {
                    self.require_causal(CausalRef::Handoff(id))?;
                }
                SupportSource::Cancellation(id) => {
                    self.require_causal(CausalRef::Cancellation(id))?;
                }
                SupportSource::Step(step) => {
                    self.require_step_ref(step)?;
                }
                SupportSource::Observation(id) => {
                    self.require_causal(CausalRef::Observation(id))?;
                }
                SupportSource::Judgment(id) => {
                    self.require_causal(CausalRef::Judgment(id))?;
                }
                SupportSource::Admission(id) => {
                    self.require_causal(CausalRef::Admission(id))?;
                }
            }
        }
        Ok(())
    }

    fn apply_steps_with_undo(
        &mut self,
        proposals: &[StepProposalV2],
    ) -> Result<Vec<StepUndo>, StepApplyFailure> {
        if proposals.is_empty() {
            return Err(StepApplyFailure {
                step_index: None,
                cause: Box::new(ProcessError::EmptyStepBatch),
            });
        }
        if proposals.len() > MAX_STEP_BATCH_ITEMS {
            return Err(StepApplyFailure {
                step_index: None,
                cause: Box::new(ProcessError::StepBatchTooLarge(proposals.len())),
            });
        }
        let mut undo = Vec::new();
        undo.try_reserve_exact(proposals.len())
            .map_err(|_| StepApplyFailure {
                step_index: None,
                cause: Box::new(ProcessError::TransactionAllocationFailed),
            })?;
        for (step_index, proposal) in proposals.iter().enumerate() {
            match self.apply_step(proposal.clone()) {
                Ok(step_undo) => undo.push(step_undo),
                Err(cause) => {
                    for step_undo in undo.into_iter().rev() {
                        self.rollback_step(step_undo);
                    }
                    return Err(StepApplyFailure {
                        step_index: Some(step_index),
                        cause: Box::new(cause),
                    });
                }
            }
        }
        Ok(undo)
    }

    fn apply_step(&mut self, proposal: StepProposalV2) -> Result<StepUndo, ProcessError> {
        if self.steps.contains_key(&proposal.id) {
            return Err(ProcessError::DuplicateStep(proposal.id));
        }
        if self.configurations.contains_key(&proposal.after.id) {
            return Err(ProcessError::DuplicateConfiguration(proposal.after.id));
        }
        if proposal.causes.len() > MAX_STEP_FRONTIER_ITEMS {
            return Err(ProcessError::StepFrontierTooLarge(proposal.causes.len()));
        }
        if proposal.observations.len() > MAX_STEP_OBSERVATIONS {
            return Err(ProcessError::StepObservationFrontierTooLarge(
                proposal.observations.len(),
            ));
        }
        if !proposal.causes.is_empty() && !is_strictly_sorted_unique(&proposal.causes) {
            return Err(ProcessError::NonCanonicalSet("step causes"));
        }
        self.validate_runtime_term(&proposal.after.value)?;
        let activation = self
            .activations
            .get(&proposal.activation)
            .ok_or(ProcessError::UnknownActivation(proposal.activation))?
            .clone();
        if activation.membership().run() != proposal.run
            || activation.latest_configuration != proposal.before
            || self
                .configurations
                .get(&proposal.before)
                .is_none_or(|configuration| configuration.activation != proposal.activation)
        {
            return Err(ProcessError::StepOwnerMismatch);
        }
        match activation.status {
            ActivationStatus::Terminal(_) => {
                return Err(ProcessError::ActivationAlreadyTerminal(proposal.activation));
            }
            ActivationStatus::Transferred(_) => {
                return Err(ProcessError::ActivationAlreadyTransferred(
                    proposal.activation,
                ));
            }
            ActivationStatus::Ready | ActivationStatus::Live | ActivationStatus::Suspended(_) => {}
        }
        if proposal.observed_state != activation.pins().observed_state {
            return Err(ProcessError::StepWorldPinMismatch);
        }
        let mode = self.mode_contract(activation.mode())?.clone();
        let executable = self
            .constitution
            .executable_contract(activation.application(), activation.mode())
            .ok_or(ProcessError::ModeNotEligible(activation.mode()))?;
        self.validate_step_budget(&proposal, &activation, &mode)?;
        let continuation_takeup = self.validate_step_frontier(&proposal, &activation, &mode)?;
        if !is_strictly_sorted_unique_by(&proposal.observations, |item| item.occurrence_id()) {
            return Err(ProcessError::NonCanonicalSet("step observations"));
        }
        let mut prepared_observations = Vec::with_capacity(proposal.observations.len());
        for observation in &proposal.observations {
            if let Some(id) = observation.occurrence_id()
                && self.observations.contains_key(&id)
            {
                return Err(ProcessError::DuplicateObservation(id));
            }
            self.validate_produced_observation_contract(observation, &mode)?;
            let content = self.validate_observation_proposal(observation)?;
            if let Some(id) = observation.occurrence_id() {
                prepared_observations.push((id, content));
            }
        }
        if let Some(delta) = &proposal.candidate_delta {
            crate::provenance::validate_candidate_delta(delta)?;
            self.validate_runtime_term(&delta.delta.term)?;
            self.validate_runtime_term(&delta.proposed_payload)?;
            self.validate_support_terms(&delta.evidence)?;
            for obligation in &delta.obligations {
                self.validate_runtime_term(&obligation.requirement)?;
            }
            if self.candidate_deltas.contains_key(&delta.id) {
                return Err(ProcessError::DuplicateCandidateDelta(delta.id));
            }
            let base = activation
                .pins()
                .observed_state
                .ok_or(ProcessError::StatefulModeMissingWorld)?;
            if mode.contract.state_delta_domain.is_none() || delta.base != base {
                return Err(ProcessError::CandidateDeltaBaseMismatch);
            }
            self.validate_domain_bound_term(
                &proposal,
                &activation,
                &delta.delta,
                mode.contract
                    .state_delta_domain
                    .as_ref()
                    .expect("stateful Mode checked above"),
            )?;
            self.validate_supports(&delta.evidence)?;
        } else if mode.contract.state_delta_domain.is_some()
            && matches!(proposal.outcome, StepOutcomeProposalV2::Return(_))
        {
            return Err(ProcessError::StatefulModeMissingDelta);
        }
        self.validate_step_outcome(&proposal, &activation, &executable, &mode)?;

        let prepared_continuation = outcome_continuation(&proposal.outcome).map(|continuation| {
            let use_policy = mode
                .contract
                .continuation
                .use_policy()
                .expect("a validated continuation outcome has a use policy");
            (continuation.clone(), use_policy)
        });

        let reference = StepRef {
            run: proposal.run,
            activation: proposal.activation,
            step: proposal.id,
        };
        let mut causes = self.step_causal_refs(&proposal.causes)?;
        causes.extend(self.step_domain_evidence_refs(&proposal));
        causes.sort_unstable();
        causes.dedup();
        self.validate_step_causal_insertions(
            reference,
            &causes,
            &prepared_observations,
            proposal.candidate_delta.as_ref().map(|delta| delta.id),
        )?;

        let mut causal_occurrences = Vec::with_capacity(
            1 + prepared_observations.len() + usize::from(proposal.candidate_delta.is_some()),
        );
        causal_occurrences.push(CausalRef::Step(reference));
        causal_occurrences.extend(
            prepared_observations
                .iter()
                .map(|(id, _)| CausalRef::Observation(*id)),
        );
        if let Some(delta) = &proposal.candidate_delta {
            causal_occurrences.push(CausalRef::CandidateDelta(delta.id));
        }
        let undo = StepUndo {
            activation: activation.id(),
            previous_status: activation.status,
            previous_configuration: activation.latest_configuration,
            previous_budget: activation.remaining_budget,
            step: reference.step,
            configuration: proposal.after.id,
            observations: prepared_observations.iter().map(|(id, _)| *id).collect(),
            candidate_delta: proposal.candidate_delta.as_ref().map(|delta| delta.id),
            continuation: prepared_continuation
                .as_ref()
                .map(|(continuation, _)| continuation.id),
            continuation_takeup,
            causal_occurrences,
        };

        self.insert_prevalidated_causal(CausalRef::Step(reference), causes);
        self.configurations.insert(
            proposal.after.id,
            Configuration {
                id: proposal.after.id,
                activation: proposal.activation,
                predecessor: ConfigurationPredecessorV2::ConfigurationAfter(reference),
                value: proposal.after.value.clone(),
            },
        );
        for (id, content) in prepared_observations {
            self.insert_prevalidated_causal(
                CausalRef::Observation(id),
                vec![CausalRef::Step(reference)],
            );
            self.observations.insert(
                id,
                Observation {
                    id,
                    content,
                    provenance: OccurrenceProvenance::ProducedBy(reference),
                },
            );
        }
        if let Some(delta) = &proposal.candidate_delta {
            self.insert_prevalidated_causal(
                CausalRef::CandidateDelta(delta.id),
                vec![CausalRef::Step(reference)],
            );
            self.candidate_deltas.insert(
                delta.id,
                CandidateDelta {
                    proposal: delta.clone(),
                    produced_by: reference,
                    package: self.package,
                },
            );
        }
        if let Some((continuation, use_policy)) = prepared_continuation {
            self.continuations.insert(
                continuation.id,
                Continuation {
                    proposal: continuation,
                    use_policy,
                    takeups: BTreeSet::new(),
                },
            );
        }
        if let Some((continuation, occurrence)) = continuation_takeup {
            let continuation_record = self
                .continuations
                .get_mut(&continuation)
                .expect("validated continuation takeup retains its continuation");
            assert!(
                continuation_record.takeups.insert(occurrence),
                "validated continuation takeup remains unique"
            );
        }
        let next_status = activation_status_after_outcome(&proposal.outcome);
        let activation_record = self
            .activations
            .get_mut(&proposal.activation)
            .expect("validated Activation remains present");
        activation_record.latest_configuration = proposal.after.id;
        activation_record.status = next_status;
        activation_record.remaining_budget = proposal.budget.after;
        self.steps.insert(proposal.id, Step { proposal });
        Ok(undo)
    }

    fn rollback_step(&mut self, undo: StepUndo) {
        if let Some((continuation, occurrence)) = undo.continuation_takeup
            && let Some(record) = self.continuations.get_mut(&continuation)
        {
            record.takeups.remove(&occurrence);
        }
        if let Some(continuation) = undo.continuation {
            self.continuations.remove(&continuation);
        }
        if let Some(delta) = undo.candidate_delta {
            self.candidate_deltas.remove(&delta);
        }
        for observation in undo.observations {
            self.observations.remove(&observation);
        }
        self.steps.remove(&undo.step);
        self.configurations.remove(&undo.configuration);
        for occurrence in undo.causal_occurrences.into_iter().rev() {
            if let Some(causes) = self.causal_predecessors.remove(&occurrence) {
                self.causal_edge_count -= causes.len();
            }
        }
        if let Some(activation) = self.activations.get_mut(&undo.activation) {
            activation.status = undo.previous_status;
            activation.latest_configuration = undo.previous_configuration;
            activation.remaining_budget = undo.previous_budget;
        }
    }

    fn validate_step_budget(
        &self,
        proposal: &StepProposalV2,
        activation: &Activation,
        mode: &crate::formation::ModePreimageV2,
    ) -> Result<(), ProcessError> {
        if proposal.budget.before != activation.remaining_budget() {
            return Err(ProcessError::StepBudgetBeforeMismatch);
        }
        let after = proposal
            .budget
            .before
            .remaining_units
            .checked_sub(proposal.budget.consumed_units)
            .ok_or(ProcessError::StepBudgetUnderflow)?;
        if proposal.budget.after.remaining_units != after {
            return Err(ProcessError::StepBudgetAfterMismatch);
        }
        let exhausted = matches!(
            proposal.outcome,
            StepOutcomeProposalV2::BudgetExhausted { .. }
        );
        if exhausted && after != 0 {
            return Err(ProcessError::BudgetExhaustionRequiresZero);
        }
        if !exhausted && after == 0 {
            return Err(ProcessError::ZeroBudgetRequiresExhaustion);
        }
        if mode.contract.productivity.kind == crate::formation::ProductivityKindV2::Bounded
            && proposal.budget.consumed_units == 0
            && matches!(
                proposal.outcome,
                StepOutcomeProposalV2::Progress | StepOutcomeProposalV2::Suspend(_)
            )
        {
            return Err(ProcessError::BoundedProgressRequiresConsumption);
        }
        Ok(())
    }

    fn validate_step_frontier(
        &self,
        proposal: &StepProposalV2,
        activation: &Activation,
        mode: &crate::formation::ModePreimageV2,
    ) -> Result<Option<(ContinuationId, ContinuationTakeupOccurrence)>, ProcessError> {
        let first = matches!(activation.status, ActivationStatus::Ready);
        if first {
            let expected = match proposal.outcome {
                StepOutcomeProposalV2::Cancel(cancellation) => vec![
                    StepCause::ActivationStart(proposal.activation),
                    StepCause::CancellationRequest(cancellation),
                ],
                _ => vec![StepCause::ActivationStart(proposal.activation)],
            };
            if proposal.causes != expected {
                return Err(ProcessError::InvalidFirstStepFrontier);
            }
        } else {
            if proposal
                .causes
                .iter()
                .any(|cause| matches!(cause, StepCause::ActivationStart(_)))
            {
                return Err(ProcessError::ActivationStartAfterFirstStep);
            }
        }
        let mut continuation_takeup = None;
        for cause in &proposal.causes {
            match *cause {
                StepCause::ActivationStart(id) => {
                    if !first || id != proposal.activation {
                        return Err(ProcessError::ActivationStartAfterFirstStep);
                    }
                }
                StepCause::PriorStep(step) => {
                    self.require_step_ref(step)?;
                    if step.run != proposal.run {
                        return Err(ProcessError::StepCauseOwnerMismatch(step.step));
                    }
                }
                StepCause::ContinuationTakeup {
                    continuation,
                    occurrence,
                } => {
                    if !matches!(activation.status, ActivationStatus::Suspended(id) if id == continuation)
                    {
                        return Err(ProcessError::UnexpectedContinuationTakeup);
                    }
                    self.validate_continuation_takeup(
                        continuation,
                        occurrence,
                        proposal.run,
                        proposal.activation,
                        mode,
                    )?;
                    if continuation_takeup
                        .replace((continuation, occurrence))
                        .is_some()
                    {
                        return Err(ProcessError::UnexpectedContinuationTakeup);
                    }
                }
                StepCause::CancellationRequest(id) => {
                    let cancellation = self
                        .cancellations
                        .get(&id)
                        .ok_or(ProcessError::UnknownCancellation(id))?;
                    self.validate_cancellation_consumer(cancellation, proposal, activation, mode)?;
                }
            }
        }
        if let ActivationStatus::Suspended(continuation) = activation.status
            && continuation_takeup.is_none()
        {
            return Err(ProcessError::SuspendedActivationNeedsTakeup {
                activation: proposal.activation,
                continuation,
            });
        }
        Ok(continuation_takeup)
    }

    fn validate_cancellation_consumer(
        &self,
        cancellation: &CancellationOccurrenceV2,
        proposal: &StepProposalV2,
        activation: &Activation,
        mode: &crate::formation::ModePreimageV2,
    ) -> Result<(), ProcessError> {
        if !mode.contract.continuation.may_cancel() {
            return Err(ProcessError::OutcomeNotPermittedByMode);
        }
        if activation.pins() != &cancellation.body.pins {
            return Err(ProcessError::CancellationScopeMismatch);
        }
        match cancellation.body.target {
            CancellationTarget::Activation(target) => {
                if target != proposal.activation {
                    return Err(ProcessError::CancellationTargetMismatch);
                }
                if activation.pins().cancellation_scope != CancellationScope::Activation {
                    return Err(ProcessError::CancellationScopeMismatch);
                }
            }
            CancellationTarget::Run(target) => {
                if target != proposal.run {
                    return Err(ProcessError::CancellationTargetMismatch);
                }
                if activation.pins().cancellation_scope != CancellationScope::Run {
                    return Err(ProcessError::CancellationScopeMismatch);
                }
            }
        }
        Ok(())
    }

    fn validate_continuation_takeup(
        &self,
        continuation: ContinuationId,
        occurrence: ContinuationTakeupOccurrence,
        run: RunId,
        activation: ActivationId,
        mode: &crate::formation::ModePreimageV2,
    ) -> Result<(), ProcessError> {
        let continuation_record = self
            .continuations
            .get(&continuation)
            .ok_or(ProcessError::UnknownContinuation(continuation))?;
        if continuation_record.use_policy == ContinuationUseV2::Linear
            && !continuation_record.takeups.is_empty()
        {
            return Err(ProcessError::LinearContinuationAlreadyTaken(continuation));
        }
        if continuation_record.takeups.contains(&occurrence) {
            return Err(ProcessError::DuplicateContinuationTakeup(continuation));
        }
        match occurrence {
            ContinuationTakeupOccurrence::Resumption(id) => {
                let record = self
                    .resumptions
                    .get(&id)
                    .ok_or(ProcessError::UnknownResumption(id))?;
                if record.body.continuation != continuation
                    || record.body.run != run
                    || record.body.activation != activation
                {
                    return Err(ProcessError::ContinuationTakeupMismatch);
                }
            }
            ContinuationTakeupOccurrence::Handoff(id) => {
                if !mode.contract.continuation.may_handoff() {
                    return Err(ProcessError::OutcomeNotPermittedByMode);
                }
                let record = self
                    .handoffs
                    .get(&id)
                    .ok_or(ProcessError::UnknownHandoff(id))?;
                if record.body.continuation != continuation
                    || record.body.run != run
                    || record.body.activation != activation
                {
                    return Err(ProcessError::ContinuationTakeupMismatch);
                }
            }
        }
        Ok(())
    }

    fn validate_step_outcome(
        &self,
        proposal: &StepProposalV2,
        activation: &Activation,
        executable: &crate::formation::ExecutableContractV2,
        mode: &crate::formation::ModePreimageV2,
    ) -> Result<(), ProcessError> {
        match &proposal.outcome {
            StepOutcomeProposalV2::Progress => Ok(()),
            StepOutcomeProposalV2::Return(value) => self.validate_domain_bound_term(
                proposal,
                activation,
                value,
                &executable.result_domain,
            ),
            StepOutcomeProposalV2::Fail(value) => {
                let domain = mode
                    .contract
                    .failure_domain
                    .as_ref()
                    .ok_or(ProcessError::OutcomeNotPermittedByMode)?;
                self.validate_domain_bound_term(proposal, activation, value, domain)
            }
            StepOutcomeProposalV2::Suspend(continuation) => {
                if !mode.contract.continuation.may_suspend() {
                    return Err(ProcessError::OutcomeNotPermittedByMode);
                }
                self.validate_new_continuation(proposal, activation, continuation)
            }
            StepOutcomeProposalV2::Cancel(cancellation) => {
                if !mode.contract.continuation.may_cancel()
                    || !proposal
                        .causes
                        .contains(&StepCause::CancellationRequest(*cancellation))
                {
                    return Err(ProcessError::OutcomeNotPermittedByMode);
                }
                Ok(())
            }
            StepOutcomeProposalV2::BudgetExhausted {
                exhaustion,
                continuation,
                obligations,
            } => {
                if mode.contract.productivity.kind != crate::formation::ProductivityKindV2::Bounded
                {
                    return Err(ProcessError::OutcomeNotPermittedByMode);
                }
                let domain = mode
                    .contract
                    .budget_exhaustion_domain
                    .as_ref()
                    .ok_or(ProcessError::OutcomeNotPermittedByMode)?;
                self.validate_domain_bound_term(proposal, activation, exhaustion, domain)?;
                if !is_strictly_sorted_unique(obligations) {
                    return Err(ProcessError::NonCanonicalSet("budget obligations"));
                }
                for obligation in obligations {
                    self.validate_runtime_term(obligation)?;
                }
                if let Some(continuation) = continuation {
                    if !mode.contract.continuation.may_suspend() {
                        return Err(ProcessError::OutcomeNotPermittedByMode);
                    }
                    self.validate_new_continuation(proposal, activation, continuation)?;
                }
                Ok(())
            }
        }
    }

    fn validate_new_continuation(
        &self,
        proposal: &StepProposalV2,
        activation: &Activation,
        continuation: &ContinuationProposalV2,
    ) -> Result<(), ProcessError> {
        if self.continuations.contains_key(&continuation.id) {
            return Err(ProcessError::DuplicateContinuation(continuation.id));
        }
        let mut expected = expected_continuation_pins(activation);
        expected.remaining_budget = proposal.budget.after;
        if continuation.emitted_by != proposal.id || continuation.pins != expected {
            return Err(ProcessError::ContinuationPinMismatch);
        }
        self.validate_runtime_term(&continuation.remainder)?;
        if continuation.remainder != proposal.after.value {
            return Err(ProcessError::ContinuationRemainderMismatch);
        }
        Ok(())
    }

    fn add_judgment(
        &mut self,
        occurrence: JudgmentOccurrenceV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        let id = occurrence.body.id;
        if self.judgments.contains_key(&id) {
            return Err(ProcessError::DuplicateJudgment(id));
        }
        crate::provenance::validate_judgment_occurrence(&occurrence)?;
        self.validate_support_terms(&occurrence.body.supports)?;
        self.validate_supports(&occurrence.body.supports)?;
        self.validate_occurrence_provenance(
            &occurrence.provenance,
            EnteredOccurrenceKind::Judgment,
            authority,
        )?;
        let base = self
            .candidate_deltas
            .get(&occurrence.body.judgment.delta)
            .and_then(|delta| self.states.get(&delta.proposal.base))
            .ok_or(ProcessError::UnknownCandidateDelta(
                occurrence.body.judgment.delta,
            ))?;
        let session = authority
            .runtime_session(base.session)
            .ok_or(ProcessError::UnknownRuntimeSession(base.session))?;
        let expected_revision = match &occurrence.provenance {
            OccurrenceProvenance::ProducedBy(step) => {
                let producer = self
                    .activations
                    .get(&step.activation)
                    .ok_or(ProcessError::UnknownActivation(step.activation))?;
                if !producer
                    .proposal
                    .static_basis
                    .judgment_authorities
                    .contains(&occurrence.body.authority)
                {
                    return Err(ProcessError::JudgmentAuthorityNotInProducerBasis {
                        judgment: id,
                        activation: step.activation,
                    });
                }
                producer.pins().program_revision
            }
            OccurrenceProvenance::EnteredThrough(entered) => {
                self.validate_governance_boundary(entered, base, authority)?;
                session.program_revision
            }
        };
        let scope = match occurrence.body.authority {
            JudgmentAuthorityEvidence::ProgramConstitution {
                revision,
                authority: reference,
            } => {
                if revision != expected_revision {
                    return Err(ProcessError::JudgmentProgramRevisionMismatch {
                        judgment: id,
                        expected: expected_revision,
                        actual: revision,
                    });
                }
                authority.revision_judgment_authority_scope(revision, reference)
            }
            JudgmentAuthorityEvidence::IrreducibleRoot {
                policy,
                authority: reference,
            } => authority.root_judgment_authority_scope(policy, reference),
        }
        .ok_or(ProcessError::UnauthorizedJudgment)?;
        if scope.semantics != base.semantics
            || scope.session != base.session
            || scope.policy != base.policy
        {
            return Err(ProcessError::UnauthorizedJudgment);
        }
        let candidate_cause = CausalRef::CandidateDelta(occurrence.body.judgment.delta);
        let causes = match &occurrence.provenance {
            OccurrenceProvenance::ProducedBy(step) => {
                let mut causes = vec![CausalRef::Step(*step), candidate_cause];
                causes.sort_unstable();
                causes.dedup();
                causes
            }
            OccurrenceProvenance::EnteredThrough(entered) => {
                if entered.causes.binary_search(&candidate_cause).is_err() {
                    return Err(ProcessError::MissingJudgmentCandidateCause {
                        judgment: id,
                        delta: occurrence.body.judgment.delta,
                    });
                }
                entered.causes.clone()
            }
        };
        self.register_causal(CausalRef::Judgment(id), causes)?;
        self.judgments.insert(id, occurrence);
        Ok(())
    }

    fn decide_state(
        &mut self,
        decision: StateAdmissionDecisionV2,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        if self.decisions.contains_key(&decision.delta) {
            return Err(ProcessError::CandidateAlreadyDecided(decision.delta));
        }
        if self
            .decisions_by_occurrence
            .contains_key(&decision.occurrence)
        {
            return Err(ProcessError::DuplicateAdmissionDecision(
                decision.occurrence,
            ));
        }
        let candidate = self
            .candidate_deltas
            .get(&decision.delta)
            .ok_or(ProcessError::UnknownCandidateDelta(decision.delta))?
            .clone();
        if candidate.package != self.package {
            return Err(ProcessError::PackageBindingMismatch);
        }
        let base = self
            .states
            .get(&candidate.proposal.base)
            .ok_or(ProcessError::UnknownStateRevision(candidate.proposal.base))?
            .clone();
        self.validate_entered(
            &decision.provenance,
            EnteredOccurrenceKind::AdmissionDecision,
            authority,
        )?;
        self.validate_governance_boundary(&decision.provenance, &base, authority)?;
        self.validate_supports(&decision.evidence)?;
        match &decision.outcome {
            StateAdmissionOutcomeV2::Admit(successor) => {
                self.validate_state_payload_binding(successor)?;
            }
            StateAdmissionOutcomeV2::Reject(rejection) => {
                self.validate_runtime_term(&rejection.reason)?;
            }
        }
        let verdict = self
            .judgments
            .get(&decision.verdict)
            .ok_or(ProcessError::UnknownJudgment(decision.verdict))?
            .clone();
        let obligation_judgments = decision
            .obligation_judgments
            .iter()
            .map(|use_| {
                self.judgments
                    .get(&use_.judgment)
                    .cloned()
                    .ok_or(ProcessError::UnknownJudgment(use_.judgment))
            })
            .collect::<Result<Vec<_>, _>>()?;
        crate::provenance::validate_state_admission_decision_inputs(
            &candidate.proposal,
            &decision,
            &verdict,
            &obligation_judgments,
            crate::provenance::AdmissionDecisionContext {
                base: &base,
                producer: candidate.produced_by,
                prior_decision: None,
            },
        )?;
        self.validate_state_admission_authorization(&decision, &base, authority)?;
        if decision
            .provenance
            .causes
            .binary_search(&CausalRef::CandidateDelta(decision.delta))
            .is_err()
        {
            return Err(ProcessError::MissingAdmissionCandidateCause(
                decision.occurrence,
            ));
        }
        if decision
            .provenance
            .causes
            .binary_search(&CausalRef::Judgment(decision.verdict))
            .is_err()
        {
            return Err(ProcessError::MissingAdmissionVerdictCause {
                admission: decision.occurrence,
                judgment: decision.verdict,
            });
        }
        for use_ in &decision.obligation_judgments {
            if decision
                .provenance
                .causes
                .binary_search(&CausalRef::Judgment(use_.judgment))
                .is_err()
            {
                return Err(ProcessError::MissingAdmissionObligationCause {
                    admission: decision.occurrence,
                    judgment: use_.judgment,
                });
            }
        }
        if let StateAdmissionOutcomeV2::Admit(successor) = &decision.outcome {
            if self.states.contains_key(&successor.id) {
                return Err(ProcessError::DuplicateStateRevision(successor.id));
            }
            let derived = derive_successor_state_id(successor);
            if derived != successor.id {
                return Err(ProcessError::StateRevisionIdMismatch {
                    claimed: successor.id,
                    derived,
                });
            }
        }
        let causes = decision.provenance.causes.clone();
        self.register_causal(CausalRef::Admission(decision.occurrence), causes)?;
        if let StateAdmissionOutcomeV2::Admit(successor) = &decision.outcome {
            self.states.insert(successor.id, successor.clone());
        }
        self.decisions_by_occurrence
            .insert(decision.occurrence, decision.delta);
        self.decisions.insert(decision.delta, decision);
        Ok(())
    }

    fn validate_state_admission_authorization(
        &self,
        decision: &StateAdmissionDecisionV2,
        base: &StateRevision,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        let scope = match decision.authorization {
            AdmissionAuthorizationEvidence::ProgramConstitution {
                revision,
                authorization,
            } => {
                let expected = authority
                    .runtime_session(base.session)
                    .ok_or(ProcessError::UnknownRuntimeSession(base.session))?
                    .program_revision;
                if revision != expected {
                    return Err(ProcessError::AdmissionProgramRevisionMismatch {
                        admission: decision.occurrence,
                        expected,
                        actual: revision,
                    });
                }
                authority.revision_state_admission_scope(revision, authorization)
            }
            AdmissionAuthorizationEvidence::IrreducibleRoot {
                policy,
                authorization,
            } => authority.root_state_admission_scope(policy, authorization),
        }
        .ok_or(ProcessError::UnauthorizedAdmission)?;
        if scope.package != self.package
            || scope.session != base.session
            || scope.base != base.id
            || scope.delta != decision.delta
        {
            return Err(ProcessError::UnauthorizedAdmission);
        }
        Ok(())
    }

    fn validate_governance_boundary(
        &self,
        entered: &EnteredThrough,
        base: &StateRevision,
        authority: &AuthorityStore,
    ) -> Result<(), ProcessError> {
        let session = authority
            .runtime_session(base.session)
            .ok_or(ProcessError::UnknownRuntimeSession(base.session))?;
        let boundary = authority
            .boundary(entered.boundary)
            .ok_or(ProcessError::UnknownBoundary(entered.boundary))?;
        if boundary.semantics != base.semantics
            || boundary.snapshot != self.constitution.snapshot()
            || boundary.program_revision != session.program_revision
            || boundary.runtime_session != Some(base.session)
            || boundary.runtime_policy != Some(base.policy)
        {
            return Err(ProcessError::BoundaryConsumerPinMismatch(entered.boundary));
        }
        Ok(())
    }

    fn step_causal_refs(&self, causes: &[StepCause]) -> Result<Vec<CausalRef>, ProcessError> {
        let mut refs = Vec::with_capacity(causes.len() * 2);
        for cause in causes {
            match *cause {
                StepCause::ActivationStart(activation) => {
                    let activation = self
                        .activations
                        .get(&activation)
                        .ok_or(ProcessError::UnknownActivation(activation))?;
                    refs.extend_from_slice(&activation.start_causes);
                }
                StepCause::PriorStep(step) => {
                    self.require_step_ref(step)?;
                    refs.push(CausalRef::Step(step));
                }
                StepCause::ContinuationTakeup {
                    continuation,
                    occurrence,
                } => {
                    let emitted = self
                        .continuations
                        .get(&continuation)
                        .ok_or(ProcessError::UnknownContinuation(continuation))?
                        .proposal
                        .emitted_by;
                    let emitted_ref = self
                        .steps
                        .get(&emitted)
                        .ok_or(ProcessError::UnknownStep(emitted))?
                        .reference();
                    refs.push(CausalRef::Step(emitted_ref));
                    refs.push(match occurrence {
                        ContinuationTakeupOccurrence::Resumption(id) => CausalRef::Resumption(id),
                        ContinuationTakeupOccurrence::Handoff(id) => CausalRef::Handoff(id),
                    });
                }
                StepCause::CancellationRequest(id) => refs.push(CausalRef::Cancellation(id)),
            }
        }
        refs.sort_unstable();
        refs.dedup();
        Ok(refs)
    }

    fn step_domain_evidence_refs(&self, proposal: &StepProposalV2) -> Vec<CausalRef> {
        let mut refs = Vec::with_capacity(2);
        if let Some(delta) = &proposal.candidate_delta {
            refs.push(CausalRef::Observation(delta.delta.evidence));
        }
        match &proposal.outcome {
            StepOutcomeProposalV2::Return(value) | StepOutcomeProposalV2::Fail(value) => {
                refs.push(CausalRef::Observation(value.evidence));
            }
            StepOutcomeProposalV2::BudgetExhausted { exhaustion, .. } => {
                refs.push(CausalRef::Observation(exhaustion.evidence));
            }
            StepOutcomeProposalV2::Progress
            | StepOutcomeProposalV2::Suspend(_)
            | StepOutcomeProposalV2::Cancel(_) => {}
        }
        refs
    }

    fn validate_causal_frontier(&self, causes: &[CausalRef]) -> Result<(), ProcessError> {
        if !is_strictly_sorted_unique(causes) {
            return Err(ProcessError::NonCanonicalSet("causal frontier"));
        }
        for cause in causes {
            self.require_causal(*cause)?;
        }
        Ok(())
    }

    fn register_causal(
        &mut self,
        occurrence: CausalRef,
        causes: Vec<CausalRef>,
    ) -> Result<(), ProcessError> {
        if self.causal_predecessors.contains_key(&occurrence) {
            return Err(ProcessError::DuplicateCausalOccurrence(occurrence));
        }
        if self.causal_predecessors.len() >= MAX_CAUSAL_OCCURRENCES {
            return Err(ProcessError::CausalOccurrenceLimitExceeded);
        }
        self.validate_causal_frontier(&causes)?;
        let next_count = self
            .causal_edge_count
            .checked_add(causes.len())
            .ok_or(ProcessError::CausalFrontierTooLarge)?;
        if next_count > MAX_CAUSAL_EDGES {
            return Err(ProcessError::CausalFrontierTooLarge);
        }
        for cause in &causes {
            if *cause == occurrence {
                return Err(ProcessError::CausalCycle(occurrence));
            }
        }
        self.causal_predecessors
            .insert(occurrence, causes.into_iter().collect());
        self.causal_edge_count = next_count;
        Ok(())
    }

    fn validate_step_causal_insertions(
        &self,
        step: StepRef,
        step_causes: &[CausalRef],
        observations: &[(ObservationId, ObservationContentV2)],
        candidate_delta: Option<CandidateDeltaId>,
    ) -> Result<(), ProcessError> {
        let step_occurrence = CausalRef::Step(step);
        if self.causal_predecessors.contains_key(&step_occurrence) {
            return Err(ProcessError::DuplicateCausalOccurrence(step_occurrence));
        }
        self.validate_causal_frontier(step_causes)?;
        if step_causes.contains(&step_occurrence) {
            return Err(ProcessError::CausalCycle(step_occurrence));
        }

        for (id, _) in observations {
            let occurrence = CausalRef::Observation(*id);
            if self.causal_predecessors.contains_key(&occurrence) {
                return Err(ProcessError::DuplicateCausalOccurrence(occurrence));
            }
        }
        if let Some(id) = candidate_delta {
            let occurrence = CausalRef::CandidateDelta(id);
            if self.causal_predecessors.contains_key(&occurrence) {
                return Err(ProcessError::DuplicateCausalOccurrence(occurrence));
            }
        }

        let emitted_occurrences = observations
            .len()
            .checked_add(usize::from(candidate_delta.is_some()))
            .and_then(|count| count.checked_add(1))
            .ok_or(ProcessError::CausalOccurrenceLimitExceeded)?;
        let next_occurrence_count = self
            .causal_predecessors
            .len()
            .checked_add(emitted_occurrences)
            .ok_or(ProcessError::CausalOccurrenceLimitExceeded)?;
        if next_occurrence_count > MAX_CAUSAL_OCCURRENCES {
            return Err(ProcessError::CausalOccurrenceLimitExceeded);
        }

        let emitted_edges = step_causes
            .len()
            .checked_add(observations.len())
            .and_then(|count| count.checked_add(usize::from(candidate_delta.is_some())))
            .ok_or(ProcessError::CausalFrontierTooLarge)?;
        let next_edge_count = self
            .causal_edge_count
            .checked_add(emitted_edges)
            .ok_or(ProcessError::CausalFrontierTooLarge)?;
        if next_edge_count > MAX_CAUSAL_EDGES {
            return Err(ProcessError::CausalFrontierTooLarge);
        }
        Ok(())
    }

    fn insert_prevalidated_causal(&mut self, occurrence: CausalRef, causes: Vec<CausalRef>) {
        let edge_count = causes.len();
        assert!(
            self.causal_predecessors
                .insert(occurrence, causes.into_iter().collect())
                .is_none(),
            "prevalidated causal occurrence remains unique"
        );
        self.causal_edge_count = self
            .causal_edge_count
            .checked_add(edge_count)
            .expect("prevalidated causal edge count remains bounded");
    }

    fn require_causal(&self, reference: CausalRef) -> Result<(), ProcessError> {
        if self.causal_predecessors.contains_key(&reference) {
            Ok(())
        } else {
            Err(ProcessError::UnknownCausalOccurrence(reference))
        }
    }

    fn require_step_ref(&self, reference: StepRef) -> Result<(), ProcessError> {
        let step = self
            .steps
            .get(&reference.step)
            .ok_or(ProcessError::UnknownStep(reference.step))?;
        if step.reference() != reference {
            return Err(ProcessError::StepOwnerMismatch);
        }
        Ok(())
    }

    #[must_use]
    pub const fn package_id(&self) -> ProcessPackageId {
        self.package
    }

    #[must_use]
    pub fn exact_package_bytes(&self) -> &[u8] {
        &self.exact_package_bytes
    }

    #[must_use]
    pub fn constitution(&self) -> &ResolvedProgramConstitutionV2 {
        &self.constitution
    }

    #[must_use]
    pub fn application(&self, id: ApplicationId) -> Option<&Application> {
        self.applications.get(&id)
    }

    #[must_use]
    pub fn activation(&self, id: ActivationId) -> Option<&Activation> {
        self.activations.get(&id)
    }

    #[must_use]
    pub fn run_root(&self, id: RunId) -> Option<&ActivationId> {
        self.runs.get(&id)
    }

    #[must_use]
    pub fn configuration(&self, id: ConfigurationId) -> Option<&Configuration> {
        self.configurations.get(&id)
    }

    #[must_use]
    pub fn step(&self, id: StepId) -> Option<&Step> {
        self.steps.get(&id)
    }

    #[must_use]
    pub fn observation(&self, id: ObservationId) -> Option<&Observation> {
        self.observations.get(&id)
    }

    #[must_use]
    pub fn continuation(&self, id: ContinuationId) -> Option<&Continuation> {
        self.continuations.get(&id)
    }

    #[must_use]
    pub fn candidate_delta(&self, id: CandidateDeltaId) -> Option<&CandidateDelta> {
        self.candidate_deltas.get(&id)
    }

    #[must_use]
    pub fn decision(&self, id: CandidateDeltaId) -> Option<&StateAdmissionDecisionV2> {
        self.decisions.get(&id)
    }

    #[must_use]
    pub fn decision_by_occurrence(
        &self,
        id: AdmissionOccurrenceId,
    ) -> Option<&StateAdmissionDecisionV2> {
        self.decisions_by_occurrence
            .get(&id)
            .and_then(|delta| self.decisions.get(delta))
    }

    #[must_use]
    pub fn external_trigger(
        &self,
        id: ExternalTriggerOccurrenceId,
    ) -> Option<&ExternalTriggerOccurrenceV2> {
        self.external_triggers.get(&id)
    }

    #[must_use]
    pub fn resumption(&self, id: ResumptionOccurrenceId) -> Option<&ResumptionOccurrenceV2> {
        self.resumptions.get(&id)
    }

    #[must_use]
    pub fn cancellation(&self, id: CancellationOccurrenceId) -> Option<&CancellationOccurrenceV2> {
        self.cancellations.get(&id)
    }

    #[must_use]
    pub fn judgment(&self, id: JudgmentOccurrenceId) -> Option<&JudgmentOccurrenceV2> {
        self.judgments.get(&id)
    }

    #[must_use]
    pub fn causal_predecessors(&self, id: CausalRef) -> Option<&BTreeSet<CausalRef>> {
        self.causal_predecessors.get(&id)
    }

    #[must_use]
    pub const fn accepted_ingress_record_count(&self) -> usize {
        self.accepted_ingress_record_count
    }

    #[must_use]
    pub const fn accepted_ingress_bytes(&self) -> usize {
        self.accepted_ingress_bytes
    }

    #[must_use]
    pub fn resource_usage(&self) -> ProcessResourceUsageV2 {
        ProcessResourceUsageV2 {
            base_records: self.base_record_count,
            accepted_ingress_records: self.accepted_ingress_record_count,
            base_package_bytes: self.exact_package_bytes.len(),
            accepted_ingress_bytes: self.accepted_ingress_bytes,
            runs: self.runs.len(),
            activations: self.activations.len(),
            configurations: self.configurations.len(),
        }
    }

    #[must_use]
    pub const fn applied_package_record_count(&self) -> usize {
        self.applied_base_record_count
    }

    #[must_use]
    pub fn state_revision(&self, id: StateRevisionId) -> Option<&StateRevision> {
        self.states.get(&id)
    }

    #[must_use]
    pub fn run_members(&self, id: RunId) -> Option<&BTreeSet<ActivationId>> {
        self.run_members.get(&id)
    }

    #[must_use]
    pub fn application_count(&self) -> usize {
        self.applications.len()
    }

    #[must_use]
    pub fn activation_count(&self) -> usize {
        self.activations.len()
    }

    #[must_use]
    pub fn run_count(&self) -> usize {
        self.runs.len()
    }

    #[must_use]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    #[must_use]
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    #[must_use]
    pub fn continuation_count(&self) -> usize {
        self.continuations.len()
    }

    #[must_use]
    pub fn candidate_delta_count(&self) -> usize {
        self.candidate_deltas.len()
    }

    #[must_use]
    pub fn decision_count(&self) -> usize {
        self.decisions.len()
    }

    #[must_use]
    pub fn state_revision_count(&self) -> usize {
        self.states.len()
    }
}

fn expected_continuation_pins(activation: &Activation) -> ContinuationPins {
    ContinuationPins {
        run: activation.membership().run(),
        activation: activation.id(),
        application: activation.application(),
        mode: activation.mode(),
        activation_pins: activation.pins().clone(),
        remaining_budget: activation.pins().budget,
    }
}

fn outcome_continuation(outcome: &StepOutcomeProposalV2) -> Option<&ContinuationProposalV2> {
    match outcome {
        StepOutcomeProposalV2::Suspend(continuation) => Some(continuation),
        StepOutcomeProposalV2::BudgetExhausted {
            continuation: Some(continuation),
            ..
        } => Some(continuation),
        StepOutcomeProposalV2::Progress
        | StepOutcomeProposalV2::Return(_)
        | StepOutcomeProposalV2::Fail(_)
        | StepOutcomeProposalV2::Cancel(_)
        | StepOutcomeProposalV2::BudgetExhausted {
            continuation: None, ..
        } => None,
    }
}

fn activation_status_after_outcome(outcome: &StepOutcomeProposalV2) -> ActivationStatus {
    match outcome {
        StepOutcomeProposalV2::Progress => ActivationStatus::Live,
        StepOutcomeProposalV2::Suspend(continuation) => {
            ActivationStatus::Suspended(continuation.id)
        }
        StepOutcomeProposalV2::Return(_) => {
            ActivationStatus::Terminal(ActivationTerminal::Returned)
        }
        StepOutcomeProposalV2::Fail(_) => ActivationStatus::Terminal(ActivationTerminal::Failed),
        StepOutcomeProposalV2::Cancel(_) => {
            ActivationStatus::Terminal(ActivationTerminal::Cancelled)
        }
        StepOutcomeProposalV2::BudgetExhausted {
            continuation: Some(continuation),
            ..
        } => ActivationStatus::Suspended(continuation.id),
        StepOutcomeProposalV2::BudgetExhausted {
            continuation: None, ..
        } => ActivationStatus::Terminal(ActivationTerminal::BudgetExhausted),
    }
}

fn derive_successor_state_id(state: &StateRevision) -> StateRevisionId {
    let cause = match state.cause {
        StateRevisionCause::SessionStart(start) => {
            crate::hash::StateRevisionCausePreimage::SessionStart(start)
        }
        StateRevisionCause::Admission {
            occurrence,
            run,
            activation,
            step,
        } => crate::hash::StateRevisionCausePreimage::Admission {
            occurrence,
            run,
            activation,
            step,
        },
    };
    crate::hash::derive_state_revision_id(
        state.semantics,
        state.session,
        state.predecessor,
        cause,
        &state.canonical_state_snapshot,
        state.policy,
    )
}

fn is_strictly_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn validate_record_batch_bounds(
    records: &[ProcessRecordV2],
    reject_empty: bool,
    maximum_records: usize,
    maximum_steps: usize,
) -> Result<RecordBatchCardinality, ProcessError> {
    if reject_empty && records.is_empty() {
        return Err(ProcessError::EmptyIngressBatch);
    }
    if records.len() > maximum_records {
        return Err(ProcessError::RecordLimitExceeded {
            count: records.len(),
            maximum: maximum_records,
        });
    }
    let mut cardinality = RecordBatchCardinality {
        records: records.len(),
        ..RecordBatchCardinality::default()
    };
    for record in records {
        match record {
            ProcessRecordV2::Activation(activation) => {
                cardinality.activations = checked_resource_add(
                    cardinality.activations,
                    1,
                    MAX_ACTIVATIONS,
                    ProcessResourceKindV2::Activation,
                )?;
                cardinality.configurations = checked_resource_add(
                    cardinality.configurations,
                    1,
                    MAX_CONFIGURATIONS,
                    ProcessResourceKindV2::Configuration,
                )?;
                if matches!(activation.membership, RunMembership::RootOf(_)) {
                    cardinality.runs = checked_resource_add(
                        cardinality.runs,
                        1,
                        MAX_RUNS,
                        ProcessResourceKindV2::Run,
                    )?;
                }
            }
            ProcessRecordV2::Steps(steps) => {
                cardinality.steps = checked_resource_add(
                    cardinality.steps,
                    steps.len(),
                    maximum_steps,
                    ProcessResourceKindV2::Step,
                )?;
                cardinality.configurations = checked_resource_add(
                    cardinality.configurations,
                    steps.len(),
                    MAX_CONFIGURATIONS,
                    ProcessResourceKindV2::Configuration,
                )?;
            }
            ProcessRecordV2::ExternalTrigger(_)
            | ProcessRecordV2::EnteredObservation(_)
            | ProcessRecordV2::Resumption(_)
            | ProcessRecordV2::Handoff(_)
            | ProcessRecordV2::Cancellation(_)
            | ProcessRecordV2::Judgment(_)
            | ProcessRecordV2::AdmissionDecision(_) => {}
        }
    }
    Ok(cardinality)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessResourceKindV2 {
    Record,
    Run,
    Activation,
    Configuration,
    Step,
}

fn checked_resource_add(
    current: usize,
    growth: usize,
    maximum: usize,
    kind: ProcessResourceKindV2,
) -> Result<usize, ProcessError> {
    let total = current.checked_add(growth).unwrap_or(usize::MAX);
    if total > maximum {
        return Err(match kind {
            ProcessResourceKindV2::Record => ProcessError::RecordLimitExceeded {
                count: total,
                maximum,
            },
            ProcessResourceKindV2::Run => ProcessError::RunLimitExceeded {
                count: total,
                maximum,
            },
            ProcessResourceKindV2::Activation => ProcessError::ActivationLimitExceeded {
                count: total,
                maximum,
            },
            ProcessResourceKindV2::Configuration => ProcessError::ConfigurationLimitExceeded {
                count: total,
                maximum,
            },
            ProcessResourceKindV2::Step => ProcessError::StepBatchTooLarge(total),
        });
    }
    Ok(total)
}

#[cfg(test)]
fn checked_aggregate_step_count(
    counts: impl IntoIterator<Item = usize>,
    maximum: usize,
) -> Result<usize, ProcessError> {
    let mut total = 0;
    for count in counts {
        total = checked_resource_add(total, count, maximum, ProcessResourceKindV2::Step)?;
    }
    Ok(total)
}

fn is_strictly_sorted_unique_by<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessError {
    RecordLimitExceeded {
        count: usize,
        maximum: usize,
    },
    RunLimitExceeded {
        count: usize,
        maximum: usize,
    },
    ActivationLimitExceeded {
        count: usize,
        maximum: usize,
    },
    ConfigurationLimitExceeded {
        count: usize,
        maximum: usize,
    },
    IngressByteLimitExceeded {
        count: usize,
        maximum: usize,
    },
    TransactionAllocationFailed,
    EmptyIngressBatch,
    Authority(crate::authority::AuthorityError),
    Canonical(crate::canonical::CanonicalEncodeError),
    Provenance(crate::provenance::ProvenanceError),
    UnknownApplication(ApplicationId),
    DuplicateInitialStateView(RuntimeSessionId),
    UnknownRuntimeSession(RuntimeSessionId),
    UnknownProgramRevision(ProgramRevisionId),
    InitialStateBindingMismatch(RuntimeSessionId),
    DuplicateExternalTrigger(ExternalTriggerOccurrenceId),
    DuplicateObservation(ObservationId),
    DuplicateResumption(ResumptionOccurrenceId),
    DuplicateHandoff(HandoffOccurrenceId),
    DuplicateCancellation(CancellationOccurrenceId),
    DuplicateActivation(ActivationId),
    DuplicateRun(RunId),
    DuplicateConfiguration(ConfigurationId),
    DuplicateStep(StepId),
    DuplicateContinuation(ContinuationId),
    DuplicateContinuationTakeup(ContinuationId),
    DuplicateCandidateDelta(CandidateDeltaId),
    DuplicateJudgment(JudgmentOccurrenceId),
    DuplicateAdmissionDecision(AdmissionOccurrenceId),
    DuplicateStateRevision(StateRevisionId),
    UnknownBoundary(BoundaryRef),
    UnknownStateRevision(StateRevisionId),
    UnknownExternalTrigger(ExternalTriggerOccurrenceId),
    UnknownActivation(ActivationId),
    UnknownRun(RunId),
    UnknownStep(StepId),
    UnknownObservation(ObservationId),
    UnknownContinuation(ContinuationId),
    UnknownResumption(ResumptionOccurrenceId),
    ResumptionRequiresFreshIngress,
    UnknownHandoff(HandoffOccurrenceId),
    UnknownCancellation(CancellationOccurrenceId),
    UnknownCandidateDelta(CandidateDeltaId),
    UnknownJudgment(JudgmentOccurrenceId),
    UnknownAdmission(AdmissionOccurrenceId),
    UnanchoredExternalProvenance {
        boundary: BoundaryRef,
        evidence: ExternalEvidenceRef,
    },
    BoundaryPinMismatch(BoundaryRef),
    BoundaryConsumerPinMismatch(BoundaryRef),
    OccurrenceConsumerPinMismatch,
    BoundaryDoesNotPermit {
        boundary: BoundaryRef,
        kind: EnteredOccurrenceKind,
    },
    ActivationPinMismatch,
    IncompleteRuntimePins,
    AuthorityPinMismatch,
    StaticBasisMismatch,
    AuthorizationCardinalityMismatch(FormationRefV2),
    PrerequisiteCardinalityMismatch(FormationRefV2),
    PrerequisiteOccurrenceKindMismatch {
        requirement: FormationRefV2,
        expected: ActivationPrerequisiteKind,
        actual: ActivationPrerequisiteKind,
    },
    UnexpectedExecutionAuthorization(FormationRefV2),
    UnexpectedPrerequisite(FormationRefV2),
    UnauthorizedExecution,
    UnauthorizedJudgment,
    UnauthorizedAdmission,
    JudgmentAuthorityNotInProducerBasis {
        judgment: JudgmentOccurrenceId,
        activation: ActivationId,
    },
    JudgmentProgramRevisionMismatch {
        judgment: JudgmentOccurrenceId,
        expected: ProgramRevisionId,
        actual: ProgramRevisionId,
    },
    AdmissionProgramRevisionMismatch {
        admission: AdmissionOccurrenceId,
        expected: ProgramRevisionId,
        actual: ProgramRevisionId,
    },
    MissingJudgmentCandidateCause {
        judgment: JudgmentOccurrenceId,
        delta: CandidateDeltaId,
    },
    MissingAdmissionCandidateCause(AdmissionOccurrenceId),
    MissingAdmissionVerdictCause {
        admission: AdmissionOccurrenceId,
        judgment: JudgmentOccurrenceId,
    },
    MissingAdmissionObligationCause {
        admission: AdmissionOccurrenceId,
        judgment: JudgmentOccurrenceId,
    },
    ModeNotEligible(ModeId),
    RootTriggerPinMismatch,
    RunMembershipMismatch,
    ChildActivationUnsupported,
    HandoffUnsupported,
    EffectfulModeUnsupported(ModeId),
    StepOwnerMismatch,
    StepCauseOwnerMismatch(StepId),
    StepWorldPinMismatch,
    InvalidFirstStepFrontier,
    ActivationStartAfterFirstStep,
    ActivationAlreadyTerminal(ActivationId),
    ActivationAlreadyTransferred(ActivationId),
    ActivationNotSuspended(ActivationId),
    SuspendedActivationNeedsTakeup {
        activation: ActivationId,
        continuation: ContinuationId,
    },
    UnexpectedContinuationTakeup,
    CancellationTargetMismatch,
    CancellationScopeMismatch,
    ContinuationTakeupMismatch,
    ContinuationPinMismatch,
    ContinuationRemainderMismatch,
    LinearContinuationAlreadyTaken(ContinuationId),
    OutcomeNotPermittedByMode,
    EmptyStepBatch,
    StepBatchTooLarge(usize),
    StepFrontierTooLarge(usize),
    StepObservationFrontierTooLarge(usize),
    StepBudgetBeforeMismatch,
    StepBudgetUnderflow,
    StepBudgetAfterMismatch,
    BudgetExhaustionRequiresZero,
    ZeroBudgetRequiresExhaustion,
    BoundedProgressRequiresConsumption,
    StatefulModeMissingWorld,
    StatefulModeMissingDelta,
    CandidateDeltaBaseMismatch,
    CandidateAlreadyDecided(CandidateDeltaId),
    RejectedDecisionIsNotAdmission(AdmissionOccurrenceId),
    PackageBindingMismatch,
    StateRevisionIdMismatch {
        claimed: StateRevisionId,
        derived: StateRevisionId,
    },
    StatePayloadSnapshotMismatch(StateRevisionId),
    RuntimeTermScopeMismatch,
    MalformedAbsentObservation,
    UnsupportedTruthVerdict,
    MissingPriorFormationEvidence(ObservationId),
    FormationEvidenceNotCausal(ObservationId),
    FormationEvidenceRequiresCheckerStep(ObservationId),
    FormationEvidenceRequiresDistinctActivation(ObservationId),
    FormationEvidenceCheckerTargetMismatch(ObservationId),
    FormationObservationNotDeclared,
    FormationEvidenceKindMismatch(ObservationId),
    FormationEvidenceDomainMismatch(ObservationId),
    PrerequisiteScopeMismatch,
    NonCanonicalSet(&'static str),
    UnknownCausalOccurrence(CausalRef),
    DuplicateCausalOccurrence(CausalRef),
    CausalFrontierTooLarge,
    CausalOccurrenceLimitExceeded,
    CausalCycle(CausalRef),
    InternalInvariant(&'static str),
}

/// One rejected live-ingress batch. Record and Step indexes are zero-based.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessIngressError {
    Batch {
        cause: Box<ProcessError>,
    },
    Record {
        record_index: usize,
        cause: Box<ProcessError>,
    },
    Step {
        record_index: usize,
        step_index: usize,
        cause: Box<ProcessError>,
    },
}

impl fmt::Display for ProcessIngressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "process ingress rejected: {self:?}")
    }
}

impl std::error::Error for ProcessIngressError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let cause = match self {
            Self::Batch { cause } | Self::Record { cause, .. } | Self::Step { cause, .. } => cause,
        };
        Some(cause.as_ref())
    }
}

impl fmt::Display for ProcessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ProcessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Authority(error) => Some(error),
            Self::Canonical(error) => Some(error),
            Self::Provenance(error) => Some(error),
            _ => None,
        }
    }
}

impl From<crate::provenance::ProvenanceError> for ProcessError {
    fn from(error: crate::provenance::ProvenanceError) -> Self {
        Self::Provenance(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{ProcessError, checked_aggregate_step_count};

    #[test]
    fn aggregate_step_count_is_checked_across_record_boundaries() {
        assert_eq!(checked_aggregate_step_count([2, 3], 5), Ok(5));
        assert_eq!(
            checked_aggregate_step_count([2, 4], 5),
            Err(ProcessError::StepBatchTooLarge(6))
        );
        assert_eq!(
            checked_aggregate_step_count([usize::MAX, 1], usize::MAX),
            Err(ProcessError::StepBatchTooLarge(usize::MAX))
        );
    }
}
