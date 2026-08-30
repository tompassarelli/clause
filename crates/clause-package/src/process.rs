use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::identity::*;
use crate::term::Term;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ModeStateContract {
    Pure,
    ProposesState,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RoleBindingValue {
    Known(Term),
    Produced,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RoleBinding {
    pub role: RoleId,
    pub value: RoleBindingValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationSchemaDeclaration {
    pub id: RelationSchemaId,
    pub roles: Vec<RoleId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeDeclaration {
    pub id: ModeId,
    pub schema: RelationSchemaId,
    pub known_roles: Vec<RoleId>,
    pub produced_roles: Vec<RoleId>,
    pub context_requirements: Vec<Term>,
    pub state_contract: ModeStateContract,
    pub may_suspend: bool,
    pub may_cancel: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorDeclaration {
    pub id: OperatorRef,
    pub modes: Vec<ModeDeclaration>,
}

/// Candidate local-reference process constitution. Construction is inert.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramConstitutionCandidate {
    pub semantics: ClauseSemanticsId,
    pub snapshot: ProgramSnapshotId,
    pub schemas: Vec<RelationSchemaDeclaration>,
    pub operators: Vec<OperatorDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationFormCandidate {
    pub term: Term,
    pub schema: RelationSchemaId,
    pub operator: OperatorRef,
    pub eligible_modes: Vec<ModeId>,
    pub bindings: Vec<RoleBinding>,
    pub context_requirements: Vec<Term>,
}

/// A checked, closed configured application possibility. It is not nominal and
/// does not execute or authorize anything.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationForm {
    term: Term,
    schema: RelationSchemaId,
    operator: OperatorRef,
    eligible_modes: Vec<ModeId>,
    bindings: Vec<RoleBinding>,
    context_requirements: Vec<Term>,
}

impl ApplicationForm {
    #[must_use]
    pub fn term(&self) -> &Term {
        &self.term
    }

    #[must_use]
    pub const fn schema(&self) -> RelationSchemaId {
        self.schema
    }

    #[must_use]
    pub const fn operator(&self) -> OperatorRef {
        self.operator
    }

    #[must_use]
    pub fn eligible_modes(&self) -> &[ModeId] {
        &self.eligible_modes
    }

    #[must_use]
    pub fn bindings(&self) -> &[RoleBinding] {
        &self.bindings
    }

    #[must_use]
    pub fn context_requirements(&self) -> &[Term] {
        &self.context_requirements
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationAllocationAuthority {
    ProgramRevision(ProgramRevisionId),
    RootPolicy(RootPolicyId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProposal {
    pub id: ApplicationId,
    pub form: ApplicationFormCandidate,
    pub allocation_authority: ApplicationAllocationAuthority,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Application {
    id: ApplicationId,
    form: ApplicationForm,
    allocation_authority: ApplicationAllocationAuthority,
}

impl Application {
    #[must_use]
    pub const fn id(&self) -> ApplicationId {
        self.id
    }

    #[must_use]
    pub fn form(&self) -> &ApplicationForm {
        &self.form
    }

    #[must_use]
    pub const fn allocation_authority(&self) -> ApplicationAllocationAuthority {
        self.allocation_authority
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutionScope {
    pub application: ApplicationId,
    pub mode: ModeId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AdmissionScope {
    pub session: RuntimeSessionId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProgramExecutionAuthorization {
    pub reference: ExecutionAuthorizationRef,
    pub scope: ExecutionScope,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProgramAdmissionAuthorization {
    pub reference: AdmissionAuthorizationRef,
    pub scope: AdmissionScope,
}

/// A Program revision supplied as already authoritative to this carrier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeProgramRevision {
    pub id: ProgramRevisionId,
    pub program: ProgramId,
    pub snapshot: ProgramSnapshotId,
    pub semantics: ClauseSemanticsId,
    pub predecessor: Option<ProgramRevisionId>,
    pub change: ProgramChangeOccurrenceId,
    pub execution_authorizations: Vec<ProgramExecutionAuthorization>,
    pub admission_authorizations: Vec<ProgramAdmissionAuthorization>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RootExecutionAuthorization {
    pub reference: RootExecutionAuthorizationRef,
    pub scope: ExecutionScope,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RootAdmissionAuthorization {
    pub reference: RootAdmissionAuthorizationRef,
    pub scope: AdmissionScope,
}

/// An independently established irreducible policy supplied to the carrier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootPolicy {
    pub id: RootPolicyId,
    pub semantics: ClauseSemanticsId,
    pub snapshot_scope: ProgramSnapshotId,
    pub execution_authorizations: Vec<RootExecutionAuthorization>,
    pub admission_authorizations: Vec<RootAdmissionAuthorization>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSession {
    pub id: RuntimeSessionId,
    pub program_revision: ProgramRevisionId,
    pub semantics: ClauseSemanticsId,
    pub policy: RuntimePolicyId,
    pub start: SessionStartOccurrenceId,
    pub initial_state: StateRevisionId,
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
    pub policy: RuntimePolicyId,
    pub semantics: ClauseSemanticsId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Budget {
    pub remaining_units: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivationPins {
    pub semantics: ClauseSemanticsId,
    pub snapshot: ProgramSnapshotId,
    pub program_revision: ProgramRevisionId,
    pub runtime_session: Option<RuntimeSessionId>,
    pub observed_state: Option<StateRevisionId>,
    pub runtime_policy: Option<RuntimePolicyId>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivationCauseFrontier {
    pub origin: ActivationOrigin,
    pub authorization: ExecutionAuthorizationEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationProposal {
    pub id: ConfigurationId,
    pub value: Term,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationProposal {
    pub id: ActivationId,
    pub application: ApplicationId,
    pub mode: ModeId,
    pub pins: ActivationPins,
    pub causes: ActivationCauseFrontier,
    pub membership: RunMembership,
    pub initial_configuration: ConfigurationProposal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activation {
    id: ActivationId,
    application: ApplicationId,
    mode: ModeId,
    pins: ActivationPins,
    causes: ActivationCauseFrontier,
    membership: RunMembership,
    initial_configuration: ConfigurationId,
}

impl Activation {
    #[must_use]
    pub const fn id(&self) -> ActivationId {
        self.id
    }

    #[must_use]
    pub const fn application(&self) -> ApplicationId {
        self.application
    }

    #[must_use]
    pub const fn mode(&self) -> ModeId {
        self.mode
    }

    #[must_use]
    pub const fn pins(&self) -> ActivationPins {
        self.pins
    }

    #[must_use]
    pub const fn causes(&self) -> ActivationCauseFrontier {
        self.causes
    }

    #[must_use]
    pub const fn membership(&self) -> RunMembership {
        self.membership
    }

    #[must_use]
    pub const fn initial_configuration(&self) -> ConfigurationId {
        self.initial_configuration
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Configuration {
    pub id: ConfigurationId,
    pub activation: ActivationId,
    pub value: Term,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContinuationPins {
    pub run: RunId,
    pub activation: ActivationId,
    pub application: ApplicationId,
    pub mode: ModeId,
    pub semantics: ClauseSemanticsId,
    pub snapshot: ProgramSnapshotId,
    pub program_revision: ProgramRevisionId,
    pub runtime_session: Option<RuntimeSessionId>,
    pub observed_state: Option<StateRevisionId>,
    pub runtime_policy: Option<RuntimePolicyId>,
    pub remaining_budget: Budget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationProposal {
    pub id: ContinuationId,
    pub emitted_by: StepId,
    pub pins: ContinuationPins,
    pub remainder: Term,
    pub linear: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Continuation {
    proposal: ContinuationProposal,
    consumed: bool,
}

impl Continuation {
    #[must_use]
    pub fn proposal(&self) -> &ContinuationProposal {
        &self.proposal
    }

    #[must_use]
    pub const fn consumed(&self) -> bool {
        self.consumed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResumptionOccurrenceProposal {
    pub id: ResumptionOccurrenceId,
    pub continuation: ContinuationId,
    pub run: RunId,
    pub activation: ActivationId,
    pub pins: ContinuationPins,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffOccurrenceProposal {
    pub id: HandoffOccurrenceId,
    pub continuation: ContinuationId,
    pub run: RunId,
    pub activation: ActivationId,
    pub pins: ContinuationPins,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CancellationTarget {
    Activation(ActivationId),
    Run(RunId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancellationOccurrenceProposal {
    pub id: CancellationOccurrenceId,
    pub target: CancellationTarget,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContinuationTakeupOccurrence {
    Resumption(ResumptionOccurrenceId),
    Handoff(HandoffOccurrenceId),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StepCause {
    ActivationStart(ActivationId),
    PriorStep {
        run: RunId,
        activation: ActivationId,
        step: StepId,
    },
    ContinuationTakeup {
        continuation: ContinuationId,
        occurrence: ContinuationTakeupOccurrence,
    },
    CancellationRequest(CancellationOccurrenceId),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObservationProposal {
    pub id: ObservationId,
    pub value: Term,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    pub id: ObservationId,
    pub run: RunId,
    pub activation: ActivationId,
    pub step: StepId,
    pub value: Term,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateDeltaProposal {
    pub id: CandidateDeltaId,
    pub base: StateRevisionId,
    pub proposed_payload: Term,
    pub evidence: Vec<Term>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateDelta {
    pub proposal: CandidateDeltaProposal,
    pub produced_by: StepId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepOutcomeProposal {
    Progress,
    Suspend(ContinuationProposal),
    Return(Term),
    Cancel(CancellationOccurrenceId),
    BudgetExhausted {
        continuation: Option<ContinuationProposal>,
        obligations: Vec<Term>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepProposal {
    pub id: StepId,
    pub run: RunId,
    pub activation: ActivationId,
    pub before: ConfigurationId,
    pub after: ConfigurationProposal,
    pub observed_state: Option<StateRevisionId>,
    /// A nonempty canonical set. The checker rejects duplicate or unsorted input.
    pub causes: Vec<StepCause>,
    pub observations: Vec<ObservationProposal>,
    pub candidate_delta: Option<CandidateDeltaProposal>,
    pub outcome: StepOutcomeProposal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Step {
    proposal: StepProposal,
}

impl Step {
    #[must_use]
    pub fn proposal(&self) -> &StepProposal {
        &self.proposal
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateAdmissionProposal {
    pub occurrence: AdmissionOccurrenceId,
    pub delta: CandidateDeltaId,
    pub authorization: AdmissionAuthorizationEvidence,
    pub successor: StateRevision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionOccurrence {
    pub id: AdmissionOccurrenceId,
    pub delta: CandidateDeltaId,
    pub base: StateRevisionId,
    pub successor: StateRevisionId,
    pub authorization: AdmissionAuthorizationEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessRecord {
    Application(ApplicationProposal),
    Activation(ActivationProposal),
    Resumption(ResumptionOccurrenceProposal),
    Handoff(HandoffOccurrenceProposal),
    Cancellation(CancellationOccurrenceProposal),
    Steps(Vec<StepProposal>),
    AdmitState(StateAdmissionProposal),
}

/// Self-contained deterministic input for process-v1 carrier vectors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessVector {
    pub constitution: ProgramConstitutionCandidate,
    pub program_revisions: Vec<AuthoritativeProgramRevision>,
    pub root_policies: Vec<RootPolicy>,
    pub sessions: Vec<RuntimeSession>,
    pub initial_states: Vec<StateRevision>,
    pub records: Vec<ProcessRecord>,
}

#[derive(Clone, Debug)]
struct ProgramConstitution {
    semantics: ClauseSemanticsId,
    snapshot: ProgramSnapshotId,
    schemas: BTreeMap<RelationSchemaId, RelationSchemaDeclaration>,
    operators: BTreeMap<OperatorRef, OperatorDeclaration>,
    modes: BTreeMap<ModeId, ModeDeclaration>,
}

impl ProgramConstitution {
    fn check(candidate: ProgramConstitutionCandidate) -> Result<Self, ProcessError> {
        let mut schemas = BTreeMap::new();
        for schema in candidate.schemas {
            if schema.id.snapshot != candidate.snapshot {
                return Err(ProcessError::DeclarationSnapshotMismatch);
            }
            if !is_strictly_sorted_unique(&schema.roles) {
                return Err(ProcessError::NonCanonicalSet("schema roles"));
            }
            if schema.roles.iter().any(|role| role.schema != schema.id) {
                return Err(ProcessError::RoleSchemaMismatch);
            }
            if schemas.insert(schema.id, schema).is_some() {
                return Err(ProcessError::DuplicateSchema);
            }
        }

        let mut operators = BTreeMap::new();
        let mut modes = BTreeMap::new();
        for operator in candidate.operators {
            if operator.id.snapshot != candidate.snapshot {
                return Err(ProcessError::DeclarationSnapshotMismatch);
            }
            if operators.contains_key(&operator.id) {
                return Err(ProcessError::DuplicateOperator);
            }
            for mode in &operator.modes {
                if mode.id.operator != operator.id {
                    return Err(ProcessError::ModeOperatorMismatch);
                }
                let schema = schemas
                    .get(&mode.schema)
                    .ok_or(ProcessError::UnknownSchema(mode.schema))?;
                if !is_strictly_sorted_unique(&mode.known_roles)
                    || !is_strictly_sorted_unique(&mode.produced_roles)
                    || !is_strictly_sorted_unique(&mode.context_requirements)
                {
                    return Err(ProcessError::NonCanonicalSet("mode contract"));
                }
                let known: BTreeSet<_> = mode.known_roles.iter().copied().collect();
                let produced: BTreeSet<_> = mode.produced_roles.iter().copied().collect();
                if !known.is_disjoint(&produced) {
                    return Err(ProcessError::RoleDirectionOverlap);
                }
                let declared: BTreeSet<_> = schema.roles.iter().copied().collect();
                if known.union(&produced).copied().collect::<BTreeSet<_>>() != declared {
                    return Err(ProcessError::ModeRoleClosureMismatch);
                }
                if modes.insert(mode.id, mode.clone()).is_some() {
                    return Err(ProcessError::DuplicateMode);
                }
            }
            operators.insert(operator.id, operator);
        }

        Ok(Self {
            semantics: candidate.semantics,
            snapshot: candidate.snapshot,
            schemas,
            operators,
            modes,
        })
    }

    fn check_form(
        &self,
        candidate: ApplicationFormCandidate,
    ) -> Result<ApplicationForm, ProcessError> {
        if candidate.schema.snapshot != self.snapshot
            || candidate.operator.snapshot != self.snapshot
        {
            return Err(ProcessError::ApplicationSnapshotMismatch);
        }
        let schema = self
            .schemas
            .get(&candidate.schema)
            .ok_or(ProcessError::UnknownSchema(candidate.schema))?;
        let operator = self
            .operators
            .get(&candidate.operator)
            .ok_or(ProcessError::UnknownOperator(candidate.operator))?;

        if !is_strictly_sorted_unique(&candidate.eligible_modes)
            || !is_strictly_sorted_unique(&candidate.bindings)
            || !is_strictly_sorted_unique(&candidate.context_requirements)
        {
            return Err(ProcessError::NonCanonicalApplicationForm);
        }
        if candidate
            .bindings
            .iter()
            .any(|binding| binding.role.schema != candidate.schema)
        {
            return Err(ProcessError::RoleSchemaMismatch);
        }
        let bound_roles: BTreeSet<_> = candidate
            .bindings
            .iter()
            .map(|binding| binding.role)
            .collect();
        let declared_roles: BTreeSet<_> = schema.roles.iter().copied().collect();
        if bound_roles != declared_roles {
            return Err(ProcessError::ApplicationRoleClosureMismatch);
        }

        let eligible: Vec<_> = operator
            .modes
            .iter()
            .filter(|mode| {
                mode.schema == candidate.schema
                    && mode.context_requirements == candidate.context_requirements
                    && mode.known_roles.iter().all(|role| {
                        candidate.bindings.iter().any(|binding| {
                            binding.role == *role
                                && matches!(binding.value, RoleBindingValue::Known(_))
                        })
                    })
                    && mode.produced_roles.iter().all(|role| {
                        candidate.bindings.iter().any(|binding| {
                            binding.role == *role
                                && matches!(binding.value, RoleBindingValue::Produced)
                        })
                    })
            })
            .map(|mode| mode.id)
            .collect();
        if candidate.eligible_modes != eligible {
            return Err(ProcessError::EligibleModeSetMismatch);
        }

        Ok(ApplicationForm {
            term: candidate.term,
            schema: candidate.schema,
            operator: candidate.operator,
            eligible_modes: candidate.eligible_modes,
            bindings: candidate.bindings,
            context_requirements: candidate.context_requirements,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ProcessCarrier {
    constitution: ProgramConstitution,
    program_revisions: BTreeMap<ProgramRevisionId, AuthoritativeProgramRevision>,
    root_policies: BTreeMap<RootPolicyId, RootPolicy>,
    sessions: BTreeMap<RuntimeSessionId, RuntimeSession>,
    states: BTreeMap<StateRevisionId, StateRevision>,
    applications: BTreeMap<ApplicationId, Application>,
    runs: BTreeMap<RunId, ActivationId>,
    activations: BTreeMap<ActivationId, Activation>,
    configurations: BTreeMap<ConfigurationId, Configuration>,
    steps: BTreeMap<StepId, Step>,
    observations: BTreeMap<ObservationId, Observation>,
    continuations: BTreeMap<ContinuationId, Continuation>,
    resumptions: BTreeMap<ResumptionOccurrenceId, ResumptionOccurrenceProposal>,
    handoffs: BTreeMap<HandoffOccurrenceId, HandoffOccurrenceProposal>,
    cancellations: BTreeMap<CancellationOccurrenceId, CancellationOccurrenceProposal>,
    candidate_deltas: BTreeMap<CandidateDeltaId, CandidateDelta>,
    admissions: BTreeMap<AdmissionOccurrenceId, AdmissionOccurrence>,
}

impl ProcessCarrier {
    pub fn from_vector_prelude(vector: &ProcessVector) -> Result<Self, ProcessError> {
        let constitution = ProgramConstitution::check(vector.constitution.clone())?;

        let mut program_revisions = BTreeMap::new();
        for revision in &vector.program_revisions {
            if revision.snapshot != constitution.snapshot
                || revision.semantics != constitution.semantics
            {
                return Err(ProcessError::AuthorityPinMismatch);
            }
            validate_program_authorizations(revision)?;
            if program_revisions
                .insert(revision.id, revision.clone())
                .is_some()
            {
                return Err(ProcessError::DuplicateProgramRevision);
            }
        }

        let mut root_policies = BTreeMap::new();
        for policy in &vector.root_policies {
            if policy.snapshot_scope != constitution.snapshot
                || policy.semantics != constitution.semantics
            {
                return Err(ProcessError::AuthorityPinMismatch);
            }
            validate_root_authorizations(policy)?;
            if root_policies.insert(policy.id, policy.clone()).is_some() {
                return Err(ProcessError::DuplicateRootPolicy);
            }
        }

        let mut states = BTreeMap::new();
        for state in &vector.initial_states {
            if state.semantics != constitution.semantics {
                return Err(ProcessError::StatePinMismatch);
            }
            if states.insert(state.id, state.clone()).is_some() {
                return Err(ProcessError::DuplicateStateRevision(state.id));
            }
        }

        let mut sessions = BTreeMap::new();
        for session in &vector.sessions {
            if session.semantics != constitution.semantics
                || !program_revisions.contains_key(&session.program_revision)
            {
                return Err(ProcessError::SessionPinMismatch);
            }
            let state = states
                .get(&session.initial_state)
                .ok_or(ProcessError::UnknownStateRevision(session.initial_state))?;
            if state.session != session.id
                || state.predecessor.is_some()
                || state.policy != session.policy
                || state.semantics != session.semantics
                || state.cause != StateRevisionCause::SessionStart(session.start)
            {
                return Err(ProcessError::InitialStateMismatch);
            }
            if sessions.insert(session.id, session.clone()).is_some() {
                return Err(ProcessError::DuplicateRuntimeSession);
            }
        }
        if states
            .values()
            .any(|state| !sessions.contains_key(&state.session))
        {
            return Err(ProcessError::StatePinMismatch);
        }

        Ok(Self {
            constitution,
            program_revisions,
            root_policies,
            sessions,
            states,
            applications: BTreeMap::new(),
            runs: BTreeMap::new(),
            activations: BTreeMap::new(),
            configurations: BTreeMap::new(),
            steps: BTreeMap::new(),
            observations: BTreeMap::new(),
            continuations: BTreeMap::new(),
            resumptions: BTreeMap::new(),
            handoffs: BTreeMap::new(),
            cancellations: BTreeMap::new(),
            candidate_deltas: BTreeMap::new(),
            admissions: BTreeMap::new(),
        })
    }

    pub fn replay(vector: &ProcessVector) -> Result<Self, ProcessError> {
        let mut carrier = Self::from_vector_prelude(vector)?;
        for record in &vector.records {
            carrier.apply(record.clone())?;
        }
        Ok(carrier)
    }

    pub fn apply(&mut self, record: ProcessRecord) -> Result<(), ProcessError> {
        match record {
            ProcessRecord::Application(proposal) => self.allocate_application(proposal),
            ProcessRecord::Activation(proposal) => self.activate(proposal),
            ProcessRecord::Resumption(proposal) => self.record_resumption(proposal),
            ProcessRecord::Handoff(proposal) => self.record_handoff(proposal),
            ProcessRecord::Cancellation(proposal) => self.record_cancellation(proposal),
            ProcessRecord::Steps(proposals) => self.apply_steps(proposals),
            ProcessRecord::AdmitState(proposal) => self.admit_state(proposal),
        }
    }

    pub fn allocate_application(
        &mut self,
        proposal: ApplicationProposal,
    ) -> Result<(), ProcessError> {
        if self.applications.contains_key(&proposal.id) {
            return Err(ProcessError::DuplicateApplication(proposal.id));
        }
        if proposal.id.snapshot != self.constitution.snapshot {
            return Err(ProcessError::ApplicationSnapshotMismatch);
        }
        match proposal.allocation_authority {
            ApplicationAllocationAuthority::ProgramRevision(revision) => {
                let revision = self
                    .program_revisions
                    .get(&revision)
                    .ok_or(ProcessError::UnknownProgramRevision(revision))?;
                if revision.snapshot != proposal.id.snapshot {
                    return Err(ProcessError::AuthorityPinMismatch);
                }
            }
            ApplicationAllocationAuthority::RootPolicy(policy) => {
                let policy = self
                    .root_policies
                    .get(&policy)
                    .ok_or(ProcessError::UnknownRootPolicy(policy))?;
                if policy.snapshot_scope != proposal.id.snapshot {
                    return Err(ProcessError::AuthorityPinMismatch);
                }
            }
        }
        let form = self.constitution.check_form(proposal.form)?;
        self.applications.insert(
            proposal.id,
            Application {
                id: proposal.id,
                form,
                allocation_authority: proposal.allocation_authority,
            },
        );
        Ok(())
    }

    pub fn activate(&mut self, proposal: ActivationProposal) -> Result<(), ProcessError> {
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
        if !application.form.eligible_modes.contains(&proposal.mode) {
            return Err(ProcessError::ModeNotEligible(proposal.mode));
        }
        if proposal.pins.semantics != self.constitution.semantics
            || proposal.pins.snapshot != self.constitution.snapshot
            || proposal.application.snapshot != proposal.pins.snapshot
        {
            return Err(ProcessError::ActivationPinMismatch);
        }
        let pinned_revision = self
            .program_revisions
            .get(&proposal.pins.program_revision)
            .ok_or(ProcessError::UnknownProgramRevision(
                proposal.pins.program_revision,
            ))?;
        if pinned_revision.snapshot != proposal.pins.snapshot
            || pinned_revision.semantics != proposal.pins.semantics
        {
            return Err(ProcessError::ActivationPinMismatch);
        }
        self.validate_runtime_pins(proposal.pins)?;
        self.validate_execution_authorization(
            proposal.causes.authorization,
            proposal.pins.program_revision,
            ExecutionScope {
                application: proposal.application,
                mode: proposal.mode,
            },
        )?;

        let run = proposal.membership.run();
        match (proposal.causes.origin, proposal.membership) {
            (ActivationOrigin::RootedBy(_), RunMembership::RootOf(root_run)) => {
                if root_run != run || self.runs.contains_key(&run) {
                    return Err(ProcessError::RunMembershipMismatch);
                }
            }
            (
                ActivationOrigin::ChildOf {
                    run: cause_run,
                    parent_activation,
                    parent_step,
                },
                RunMembership::ChildIn(member_run),
            ) => {
                self.validate_parent_step(cause_run, parent_activation, parent_step)?;
                if cause_run != member_run || !self.runs.contains_key(&member_run) {
                    return Err(ProcessError::RunMembershipMismatch);
                }
            }
            (
                ActivationOrigin::HandoffFrom {
                    run: cause_run,
                    parent_activation,
                    parent_step,
                    continuation,
                    handoff,
                },
                RunMembership::ChildIn(member_run),
            ) => {
                self.validate_parent_step(cause_run, parent_activation, parent_step)?;
                let handoff_record = self
                    .handoffs
                    .get(&handoff)
                    .ok_or(ProcessError::UnknownHandoff(handoff))?;
                if cause_run != member_run
                    || handoff_record.continuation != continuation
                    || !self.continuations.contains_key(&continuation)
                {
                    return Err(ProcessError::RunMembershipMismatch);
                }
            }
            _ => return Err(ProcessError::RunMembershipMismatch),
        }

        if matches!(proposal.membership, RunMembership::RootOf(_)) {
            self.runs.insert(run, proposal.id);
        }
        self.configurations.insert(
            proposal.initial_configuration.id,
            Configuration {
                id: proposal.initial_configuration.id,
                activation: proposal.id,
                value: proposal.initial_configuration.value,
            },
        );
        self.activations.insert(
            proposal.id,
            Activation {
                id: proposal.id,
                application: proposal.application,
                mode: proposal.mode,
                pins: proposal.pins,
                causes: proposal.causes,
                membership: proposal.membership,
                initial_configuration: proposal.initial_configuration.id,
            },
        );
        Ok(())
    }

    pub fn record_resumption(
        &mut self,
        proposal: ResumptionOccurrenceProposal,
    ) -> Result<(), ProcessError> {
        if self.resumptions.contains_key(&proposal.id) {
            return Err(ProcessError::DuplicateResumption(proposal.id));
        }
        self.validate_continuation_occurrence(
            proposal.continuation,
            proposal.run,
            proposal.activation,
            proposal.pins,
        )?;
        self.resumptions.insert(proposal.id, proposal);
        Ok(())
    }

    pub fn record_handoff(
        &mut self,
        proposal: HandoffOccurrenceProposal,
    ) -> Result<(), ProcessError> {
        if self.handoffs.contains_key(&proposal.id) {
            return Err(ProcessError::DuplicateHandoff(proposal.id));
        }
        self.validate_continuation_occurrence(
            proposal.continuation,
            proposal.run,
            proposal.activation,
            proposal.pins,
        )?;
        self.handoffs.insert(proposal.id, proposal);
        Ok(())
    }

    pub fn record_cancellation(
        &mut self,
        proposal: CancellationOccurrenceProposal,
    ) -> Result<(), ProcessError> {
        if self.cancellations.contains_key(&proposal.id) {
            return Err(ProcessError::DuplicateCancellation(proposal.id));
        }
        match proposal.target {
            CancellationTarget::Activation(activation) => {
                if !self.activations.contains_key(&activation) {
                    return Err(ProcessError::UnknownActivation(activation));
                }
            }
            CancellationTarget::Run(run) => {
                if !self.runs.contains_key(&run) {
                    return Err(ProcessError::UnknownRun(run));
                }
            }
        }
        self.cancellations.insert(proposal.id, proposal);
        Ok(())
    }

    /// Validate a batch in full, including its causal graph, before allocating
    /// any Step or output identity.
    pub fn apply_steps(&mut self, proposals: Vec<StepProposal>) -> Result<(), ProcessError> {
        self.preflight_step_batch(&proposals)?;
        let mut checked = self.clone();
        for proposal in proposals {
            checked.apply_one_step(proposal)?;
        }
        *self = checked;
        Ok(())
    }

    fn preflight_step_batch(&self, proposals: &[StepProposal]) -> Result<(), ProcessError> {
        if proposals.is_empty() {
            return Err(ProcessError::EmptyStepBatch);
        }
        let mut candidate_positions = BTreeMap::new();
        for (position, proposal) in proposals.iter().enumerate() {
            if self.steps.contains_key(&proposal.id)
                || candidate_positions.insert(proposal.id, position).is_some()
            {
                return Err(ProcessError::DuplicateStep(proposal.id));
            }
            for cause in &proposal.causes {
                if let StepCause::PriorStep { step, .. } = cause {
                    if *step == proposal.id {
                        return Err(ProcessError::SelfStepCause(proposal.id));
                    }
                    if !self.steps.contains_key(step) && !candidate_positions.contains_key(step) {
                        // A later batch member may not have been visited yet.
                        if !proposals.iter().any(|candidate| candidate.id == *step) {
                            return Err(ProcessError::StepCauseNotConstituted(*step));
                        }
                    }
                }
            }
        }

        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for proposal in proposals {
            detect_candidate_cycle(
                proposal.id,
                proposals,
                &candidate_positions,
                &mut visiting,
                &mut visited,
            )?;
        }

        for (position, proposal) in proposals.iter().enumerate() {
            for cause in &proposal.causes {
                if let StepCause::PriorStep { step, .. } = cause
                    && let Some(cause_position) = candidate_positions.get(step)
                    && *cause_position >= position
                {
                    return Err(ProcessError::FutureStepCause(*step));
                }
            }
        }
        Ok(())
    }

    fn apply_one_step(&mut self, proposal: StepProposal) -> Result<(), ProcessError> {
        if self.steps.contains_key(&proposal.id) {
            return Err(ProcessError::DuplicateStep(proposal.id));
        }
        if proposal.causes.is_empty() {
            return Err(ProcessError::EmptyStepCauseFrontier);
        }
        if !is_strictly_sorted_unique(&proposal.causes) {
            return Err(ProcessError::NonCanonicalSet("step cause frontier"));
        }
        if !is_strictly_sorted_unique(&proposal.observations) {
            return Err(ProcessError::NonCanonicalSet("step observations"));
        }
        if self.configurations.contains_key(&proposal.after.id) {
            return Err(ProcessError::DuplicateConfiguration(proposal.after.id));
        }
        for observation in &proposal.observations {
            if self.observations.contains_key(&observation.id) {
                return Err(ProcessError::DuplicateObservation(observation.id));
            }
        }
        if let Some(delta) = &proposal.candidate_delta
            && self.candidate_deltas.contains_key(&delta.id)
        {
            return Err(ProcessError::DuplicateCandidateDelta(delta.id));
        }

        let activation = self
            .activations
            .get(&proposal.activation)
            .ok_or(ProcessError::UnknownActivation(proposal.activation))?
            .clone();
        if activation.membership.run() != proposal.run {
            return Err(ProcessError::StepOwnerMismatch);
        }
        let before = self
            .configurations
            .get(&proposal.before)
            .ok_or(ProcessError::UnknownConfiguration(proposal.before))?;
        if before.activation != proposal.activation {
            return Err(ProcessError::ConfigurationOwnerMismatch);
        }
        if proposal.observed_state != activation.pins.observed_state {
            return Err(ProcessError::StepWorldPinMismatch);
        }

        let is_first = !self
            .steps
            .values()
            .any(|step| step.proposal.activation == proposal.activation);
        if is_first {
            if proposal.causes != [StepCause::ActivationStart(proposal.activation)] {
                return Err(ProcessError::InvalidFirstStepFrontier);
            }
        } else if proposal
            .causes
            .iter()
            .any(|cause| matches!(cause, StepCause::ActivationStart(_)))
        {
            return Err(ProcessError::ActivationStartAfterFirstStep);
        }

        let mode = self
            .constitution
            .modes
            .get(&activation.mode)
            .expect("an accepted Activation retains a declared mode")
            .clone();
        let mut continuations_to_consume = Vec::new();
        for cause in &proposal.causes {
            match *cause {
                StepCause::ActivationStart(owner) => {
                    if owner != proposal.activation {
                        return Err(ProcessError::StepOwnerMismatch);
                    }
                }
                StepCause::PriorStep {
                    run,
                    activation,
                    step,
                } => {
                    if step == proposal.id {
                        return Err(ProcessError::SelfStepCause(step));
                    }
                    let predecessor = self
                        .steps
                        .get(&step)
                        .ok_or(ProcessError::StepCauseNotConstituted(step))?;
                    if predecessor.proposal.run != run
                        || predecessor.proposal.activation != activation
                        || run != proposal.run
                    {
                        return Err(ProcessError::StepCauseOwnerMismatch(step));
                    }
                }
                StepCause::ContinuationTakeup {
                    continuation,
                    occurrence,
                } => {
                    if !mode.may_suspend {
                        return Err(ProcessError::CauseNotPermittedByMode);
                    }
                    let continuation_record = self
                        .continuations
                        .get(&continuation)
                        .ok_or(ProcessError::UnknownContinuation(continuation))?;
                    if continuation_record.proposal.pins.run != proposal.run
                        || continuation_record.proposal.pins.activation != proposal.activation
                        || continuation_record.consumed && continuation_record.proposal.linear
                    {
                        return Err(ProcessError::ContinuationPinMismatch);
                    }
                    let occurrence_matches = match occurrence {
                        ContinuationTakeupOccurrence::Resumption(id) => {
                            self.resumptions.get(&id).is_some_and(|record| {
                                record.continuation == continuation
                                    && record.run == proposal.run
                                    && record.activation == proposal.activation
                                    && record.pins == continuation_record.proposal.pins
                            })
                        }
                        ContinuationTakeupOccurrence::Handoff(id) => {
                            self.handoffs.get(&id).is_some_and(|record| {
                                record.continuation == continuation
                                    && record.run == proposal.run
                                    && record.activation == proposal.activation
                                    && record.pins == continuation_record.proposal.pins
                            })
                        }
                    };
                    if !occurrence_matches {
                        return Err(ProcessError::ContinuationTakeupMismatch);
                    }
                    let emitting_step_is_cause = proposal.causes.iter().any(|candidate| {
                        matches!(
                            candidate,
                            StepCause::PriorStep { step, .. }
                                if *step == continuation_record.proposal.emitted_by
                        )
                    });
                    if !emitting_step_is_cause {
                        return Err(ProcessError::ContinuationEmitterNotCited);
                    }
                    continuations_to_consume.push(continuation);
                }
                StepCause::CancellationRequest(cancellation) => {
                    if !mode.may_cancel {
                        return Err(ProcessError::CauseNotPermittedByMode);
                    }
                    let cancellation = self
                        .cancellations
                        .get(&cancellation)
                        .ok_or(ProcessError::UnknownCancellation(cancellation))?;
                    let applies = match cancellation.target {
                        CancellationTarget::Activation(target) => target == proposal.activation,
                        CancellationTarget::Run(target) => target == proposal.run,
                    };
                    if !applies {
                        return Err(ProcessError::CancellationTargetMismatch);
                    }
                }
            }
        }

        self.validate_step_outcome(&proposal, &activation, &mode)?;
        if let Some(delta) = &proposal.candidate_delta {
            if mode.state_contract != ModeStateContract::ProposesState {
                return Err(ProcessError::PureModeProposedState);
            }
            let observed = activation
                .pins
                .observed_state
                .ok_or(ProcessError::StatefulModeMissingWorld)?;
            if delta.base != observed {
                return Err(ProcessError::CandidateDeltaBaseMismatch);
            }
        }

        self.configurations.insert(
            proposal.after.id,
            Configuration {
                id: proposal.after.id,
                activation: proposal.activation,
                value: proposal.after.value.clone(),
            },
        );
        for observation in &proposal.observations {
            self.observations.insert(
                observation.id,
                Observation {
                    id: observation.id,
                    run: proposal.run,
                    activation: proposal.activation,
                    step: proposal.id,
                    value: observation.value.clone(),
                },
            );
        }
        if let Some(delta) = &proposal.candidate_delta {
            self.candidate_deltas.insert(
                delta.id,
                CandidateDelta {
                    proposal: delta.clone(),
                    produced_by: proposal.id,
                },
            );
        }
        if let Some(continuation) = outcome_continuation(&proposal.outcome) {
            self.continuations.insert(
                continuation.id,
                Continuation {
                    proposal: continuation.clone(),
                    consumed: false,
                },
            );
        }
        for continuation in continuations_to_consume {
            self.continuations
                .get_mut(&continuation)
                .expect("validated continuation remains present")
                .consumed = true;
        }
        self.steps.insert(proposal.id, Step { proposal });
        Ok(())
    }

    fn validate_step_outcome(
        &self,
        proposal: &StepProposal,
        activation: &Activation,
        mode: &ModeDeclaration,
    ) -> Result<(), ProcessError> {
        match &proposal.outcome {
            StepOutcomeProposal::Progress | StepOutcomeProposal::Return(_) => Ok(()),
            StepOutcomeProposal::Suspend(continuation) => {
                if !mode.may_suspend {
                    return Err(ProcessError::OutcomeNotPermittedByMode);
                }
                self.validate_new_continuation(proposal, activation, continuation)
            }
            StepOutcomeProposal::Cancel(cancellation) => {
                if !mode.may_cancel
                    || !proposal
                        .causes
                        .contains(&StepCause::CancellationRequest(*cancellation))
                {
                    return Err(ProcessError::OutcomeNotPermittedByMode);
                }
                Ok(())
            }
            StepOutcomeProposal::BudgetExhausted {
                continuation,
                obligations,
            } => {
                if !is_strictly_sorted_unique(obligations) {
                    return Err(ProcessError::NonCanonicalSet("budget obligations"));
                }
                if let Some(continuation) = continuation {
                    if !mode.may_suspend {
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
        proposal: &StepProposal,
        activation: &Activation,
        continuation: &ContinuationProposal,
    ) -> Result<(), ProcessError> {
        if self.continuations.contains_key(&continuation.id) {
            return Err(ProcessError::DuplicateContinuation(continuation.id));
        }
        let mut expected_pins = expected_continuation_pins(activation);
        expected_pins.remaining_budget = continuation.pins.remaining_budget;
        if continuation.emitted_by != proposal.id || continuation.pins != expected_pins {
            return Err(ProcessError::ContinuationPinMismatch);
        }
        if continuation.pins.remaining_budget.remaining_units
            > activation.pins.budget.remaining_units
        {
            return Err(ProcessError::ContinuationBudgetIncreased);
        }
        Ok(())
    }

    pub fn admit_state(&mut self, proposal: StateAdmissionProposal) -> Result<(), ProcessError> {
        if self.admissions.contains_key(&proposal.occurrence) {
            return Err(ProcessError::DuplicateAdmission(proposal.occurrence));
        }
        if self.states.contains_key(&proposal.successor.id) {
            return Err(ProcessError::DuplicateStateRevision(proposal.successor.id));
        }
        let delta = self
            .candidate_deltas
            .get(&proposal.delta)
            .ok_or(ProcessError::UnknownCandidateDelta(proposal.delta))?;
        let base = self
            .states
            .get(&delta.proposal.base)
            .ok_or(ProcessError::UnknownStateRevision(delta.proposal.base))?;
        self.validate_admission_authorization(
            proposal.authorization,
            AdmissionScope {
                session: base.session,
            },
        )?;
        let producing_step = self
            .steps
            .get(&delta.produced_by)
            .expect("accepted candidate delta retains its producing Step");
        let expected_cause = StateRevisionCause::Admission {
            occurrence: proposal.occurrence,
            run: producing_step.proposal.run,
            activation: producing_step.proposal.activation,
            step: producing_step.proposal.id,
        };
        if proposal.successor.session != base.session
            || proposal.successor.predecessor != Some(base.id)
            || proposal.successor.cause != expected_cause
            || proposal.successor.payload != delta.proposal.proposed_payload
            || proposal.successor.policy != base.policy
            || proposal.successor.semantics != base.semantics
        {
            return Err(ProcessError::SuccessorStateMismatch);
        }

        self.admissions.insert(
            proposal.occurrence,
            AdmissionOccurrence {
                id: proposal.occurrence,
                delta: proposal.delta,
                base: base.id,
                successor: proposal.successor.id,
                authorization: proposal.authorization,
            },
        );
        self.states
            .insert(proposal.successor.id, proposal.successor);
        Ok(())
    }

    fn validate_runtime_pins(&self, pins: ActivationPins) -> Result<(), ProcessError> {
        match (
            pins.runtime_session,
            pins.observed_state,
            pins.runtime_policy,
        ) {
            (None, None, None) => Ok(()),
            (Some(session_id), Some(state_id), Some(policy)) => {
                let session = self
                    .sessions
                    .get(&session_id)
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

    fn validate_execution_authorization(
        &self,
        evidence: ExecutionAuthorizationEvidence,
        pinned_revision: ProgramRevisionId,
        scope: ExecutionScope,
    ) -> Result<(), ProcessError> {
        match evidence {
            ExecutionAuthorizationEvidence::ProgramConstitution {
                revision,
                authorization,
            } => {
                if revision != pinned_revision {
                    return Err(ProcessError::AuthorityPinMismatch);
                }
                let revision = self
                    .program_revisions
                    .get(&revision)
                    .ok_or(ProcessError::UnknownProgramRevision(revision))?;
                if authorization.snapshot != revision.snapshot
                    || !revision
                        .execution_authorizations
                        .iter()
                        .any(|entry| entry.reference == authorization && entry.scope == scope)
                {
                    return Err(ProcessError::UnauthorizedExecution);
                }
            }
            ExecutionAuthorizationEvidence::IrreducibleRoot {
                policy,
                authorization,
            } => {
                let policy_record = self
                    .root_policies
                    .get(&policy)
                    .ok_or(ProcessError::UnknownRootPolicy(policy))?;
                if authorization.policy != policy
                    || !policy_record
                        .execution_authorizations
                        .iter()
                        .any(|entry| entry.reference == authorization && entry.scope == scope)
                {
                    return Err(ProcessError::UnauthorizedExecution);
                }
            }
        }
        Ok(())
    }

    fn validate_admission_authorization(
        &self,
        evidence: AdmissionAuthorizationEvidence,
        scope: AdmissionScope,
    ) -> Result<(), ProcessError> {
        match evidence {
            AdmissionAuthorizationEvidence::ProgramConstitution {
                revision,
                authorization,
            } => {
                let revision = self
                    .program_revisions
                    .get(&revision)
                    .ok_or(ProcessError::UnknownProgramRevision(revision))?;
                if authorization.snapshot != revision.snapshot
                    || !revision
                        .admission_authorizations
                        .iter()
                        .any(|entry| entry.reference == authorization && entry.scope == scope)
                {
                    return Err(ProcessError::UnauthorizedAdmission);
                }
            }
            AdmissionAuthorizationEvidence::IrreducibleRoot {
                policy,
                authorization,
            } => {
                let policy_record = self
                    .root_policies
                    .get(&policy)
                    .ok_or(ProcessError::UnknownRootPolicy(policy))?;
                if authorization.policy != policy
                    || !policy_record
                        .admission_authorizations
                        .iter()
                        .any(|entry| entry.reference == authorization && entry.scope == scope)
                {
                    return Err(ProcessError::UnauthorizedAdmission);
                }
            }
        }
        Ok(())
    }

    fn validate_parent_step(
        &self,
        run: RunId,
        activation: ActivationId,
        step: StepId,
    ) -> Result<(), ProcessError> {
        let step = self
            .steps
            .get(&step)
            .ok_or(ProcessError::StepCauseNotConstituted(step))?;
        if step.proposal.run != run || step.proposal.activation != activation {
            return Err(ProcessError::StepOwnerMismatch);
        }
        Ok(())
    }

    fn validate_continuation_occurrence(
        &self,
        continuation: ContinuationId,
        run: RunId,
        activation: ActivationId,
        pins: ContinuationPins,
    ) -> Result<(), ProcessError> {
        let continuation = self
            .continuations
            .get(&continuation)
            .ok_or(ProcessError::UnknownContinuation(continuation))?;
        if continuation.proposal.pins != pins || pins.run != run || pins.activation != activation {
            return Err(ProcessError::ContinuationPinMismatch);
        }
        Ok(())
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
    pub fn step(&self, id: StepId) -> Option<&Step> {
        self.steps.get(&id)
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
    pub fn state_revision(&self, id: StateRevisionId) -> Option<&StateRevision> {
        self.states.get(&id)
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
    pub fn state_revision_count(&self) -> usize {
        self.states.len()
    }

    #[must_use]
    pub fn admission_count(&self) -> usize {
        self.admissions.len()
    }
}

fn validate_program_authorizations(
    revision: &AuthoritativeProgramRevision,
) -> Result<(), ProcessError> {
    let mut execution = BTreeSet::new();
    for entry in &revision.execution_authorizations {
        if entry.reference.snapshot != revision.snapshot || !execution.insert(entry.reference) {
            return Err(ProcessError::MalformedAuthority);
        }
    }
    let mut admission = BTreeSet::new();
    for entry in &revision.admission_authorizations {
        if entry.reference.snapshot != revision.snapshot || !admission.insert(entry.reference) {
            return Err(ProcessError::MalformedAuthority);
        }
    }
    Ok(())
}

fn validate_root_authorizations(policy: &RootPolicy) -> Result<(), ProcessError> {
    let mut execution = BTreeSet::new();
    for entry in &policy.execution_authorizations {
        if entry.reference.policy != policy.id || !execution.insert(entry.reference) {
            return Err(ProcessError::MalformedAuthority);
        }
    }
    let mut admission = BTreeSet::new();
    for entry in &policy.admission_authorizations {
        if entry.reference.policy != policy.id || !admission.insert(entry.reference) {
            return Err(ProcessError::MalformedAuthority);
        }
    }
    Ok(())
}

fn expected_continuation_pins(activation: &Activation) -> ContinuationPins {
    ContinuationPins {
        run: activation.membership.run(),
        activation: activation.id,
        application: activation.application,
        mode: activation.mode,
        semantics: activation.pins.semantics,
        snapshot: activation.pins.snapshot,
        program_revision: activation.pins.program_revision,
        runtime_session: activation.pins.runtime_session,
        observed_state: activation.pins.observed_state,
        runtime_policy: activation.pins.runtime_policy,
        remaining_budget: activation.pins.budget,
    }
}

fn outcome_continuation(outcome: &StepOutcomeProposal) -> Option<&ContinuationProposal> {
    match outcome {
        StepOutcomeProposal::Suspend(continuation) => Some(continuation),
        StepOutcomeProposal::BudgetExhausted {
            continuation: Some(continuation),
            ..
        } => Some(continuation),
        StepOutcomeProposal::Progress
        | StepOutcomeProposal::Return(_)
        | StepOutcomeProposal::Cancel(_)
        | StepOutcomeProposal::BudgetExhausted {
            continuation: None, ..
        } => None,
    }
}

fn detect_candidate_cycle(
    step: StepId,
    proposals: &[StepProposal],
    positions: &BTreeMap<StepId, usize>,
    visiting: &mut BTreeSet<StepId>,
    visited: &mut BTreeSet<StepId>,
) -> Result<(), ProcessError> {
    if visited.contains(&step) {
        return Ok(());
    }
    if !visiting.insert(step) {
        return Err(ProcessError::CausalCycle(step));
    }
    let proposal = &proposals[*positions
        .get(&step)
        .expect("candidate cycle traversal starts from a candidate Step")];
    for cause in &proposal.causes {
        if let StepCause::PriorStep { step: cause, .. } = cause
            && positions.contains_key(cause)
        {
            detect_candidate_cycle(*cause, proposals, positions, visiting, visited)?;
        }
    }
    visiting.remove(&step);
    visited.insert(step);
    Ok(())
}

fn is_strictly_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessError {
    DeclarationSnapshotMismatch,
    DuplicateSchema,
    DuplicateOperator,
    DuplicateMode,
    RoleSchemaMismatch,
    RoleDirectionOverlap,
    ModeRoleClosureMismatch,
    ModeOperatorMismatch,
    UnknownSchema(RelationSchemaId),
    UnknownOperator(OperatorRef),
    NonCanonicalSet(&'static str),
    NonCanonicalApplicationForm,
    ApplicationSnapshotMismatch,
    ApplicationRoleClosureMismatch,
    EligibleModeSetMismatch,
    DuplicateProgramRevision,
    DuplicateRootPolicy,
    DuplicateRuntimeSession,
    DuplicateApplication(ApplicationId),
    DuplicateActivation(ActivationId),
    DuplicateStep(StepId),
    DuplicateConfiguration(ConfigurationId),
    DuplicateObservation(ObservationId),
    DuplicateContinuation(ContinuationId),
    DuplicateCandidateDelta(CandidateDeltaId),
    DuplicateStateRevision(StateRevisionId),
    DuplicateAdmission(AdmissionOccurrenceId),
    DuplicateResumption(ResumptionOccurrenceId),
    DuplicateHandoff(HandoffOccurrenceId),
    DuplicateCancellation(CancellationOccurrenceId),
    UnknownProgramRevision(ProgramRevisionId),
    UnknownRootPolicy(RootPolicyId),
    UnknownRuntimeSession(RuntimeSessionId),
    UnknownStateRevision(StateRevisionId),
    UnknownApplication(ApplicationId),
    UnknownActivation(ActivationId),
    UnknownRun(RunId),
    UnknownConfiguration(ConfigurationId),
    UnknownContinuation(ContinuationId),
    UnknownResumption(ResumptionOccurrenceId),
    UnknownHandoff(HandoffOccurrenceId),
    UnknownCancellation(CancellationOccurrenceId),
    UnknownCandidateDelta(CandidateDeltaId),
    AuthorityPinMismatch,
    MalformedAuthority,
    UnauthorizedExecution,
    UnauthorizedAdmission,
    SessionPinMismatch,
    StatePinMismatch,
    InitialStateMismatch,
    ModeNotEligible(ModeId),
    ActivationPinMismatch,
    IncompleteRuntimePins,
    RunMembershipMismatch,
    EmptyStepBatch,
    EmptyStepCauseFrontier,
    SelfStepCause(StepId),
    FutureStepCause(StepId),
    StepCauseNotConstituted(StepId),
    CausalCycle(StepId),
    StepOwnerMismatch,
    StepCauseOwnerMismatch(StepId),
    ConfigurationOwnerMismatch,
    StepWorldPinMismatch,
    InvalidFirstStepFrontier,
    ActivationStartAfterFirstStep,
    CauseNotPermittedByMode,
    OutcomeNotPermittedByMode,
    ContinuationPinMismatch,
    ContinuationBudgetIncreased,
    ContinuationTakeupMismatch,
    ContinuationEmitterNotCited,
    CancellationTargetMismatch,
    PureModeProposedState,
    StatefulModeMissingWorld,
    CandidateDeltaBaseMismatch,
    SuccessorStateMismatch,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ProcessError {}
