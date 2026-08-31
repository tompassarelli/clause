use std::error::Error;
use std::fmt;

use clause_package::{
    ActivationId, AdmissionOccurrenceId, ApplicationId, CandidateDeltaId, CausalRef,
    ExternalEvidenceRef, JudgmentOccurrenceId, ObservationId, ProcessPackageId, ProgramRevisionId,
    RootPolicyId, RunId, RuntimePolicyId, RuntimeSessionId, StateRevisionId, StepId, StepRef,
};

use super::{
    ExecutableCandidateV1, ExecutablePhysicalPlanIdV1, ExecutableProjectedObservationV1,
    ExecutableResumptionV1, ExecutableStateRevisionV1, ExecutableStepV1, ExecutableSuspensionV1,
    PersistentProcessSessionErrorV1, PersistentProcessSessionV1,
};

/// Exact execution pins captured when one non-authoritative process branch is
/// forked from an admitted world. This is a boundary record over existing
/// Clause identities, not a second source of semantic identity or authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessBranchPinsV1 {
    pub parent_state: StateRevisionId,
    pub program_revision: ProgramRevisionId,
    pub package: ProcessPackageId,
    pub application: ApplicationId,
    pub session: RuntimeSessionId,
    pub runtime_policy: RuntimePolicyId,
    pub root_policy: RootPolicyId,
    pub input_evidence: ExternalEvidenceRef,
    pub physical_plan: ExecutablePhysicalPlanIdV1,
    pub budget_units: u64,
    pub disconnect_tick: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessBranchAncestryV1 {
    pub parent_state: StateRevisionId,
    pub run: RunId,
    pub activation: ActivationId,
    pub disconnect_step: StepId,
    pub suspension_step: StepId,
    pub continuation: clause_package::ContinuationId,
}

/// Construct-blind projection that keeps one entered occurrence attached to
/// the Observation and Step created by its actual execution. It introduces no
/// identity, ordering, or authority beyond those retained records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessCommandEvidenceV1 {
    pub occurrence: Vec<u8>,
    pub step: StepId,
    pub observation: ObservationId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessReconnectEvidenceV1 {
    pub pins: ProcessBranchPinsV1,
    pub ancestry: ProcessBranchAncestryV1,
    pub resumption: ExecutableResumptionV1,
    pub command_evidence: Vec<ProcessCommandEvidenceV1>,
    pub candidate: CandidateDeltaId,
    pub candidate_step: StepId,
}

/// Caller-supplied, exact execution plan selected by a domain policy after it
/// has inspected the reconnect evidence. The generic substrate validates its
/// bindings and executes it against the current authoritative base; it never
/// interprets or merges branch state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedReconnectAdmissionPlanV1 {
    pub branch_candidate: CandidateDeltaId,
    pub authoritative_base: StateRevisionId,
    pub occurrences: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessCausalRecordV1 {
    pub occurrence: CausalRef,
    pub predecessors: Vec<CausalRef>,
}

/// Query result spanning the retained branch evidence and the separately
/// admitted authoritative consequence. Cross-Run log order is intentionally
/// absent; only exact occurrence predecessors are returned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessBranchExplanationV1 {
    pub pins: ProcessBranchPinsV1,
    pub ancestry: ProcessBranchAncestryV1,
    pub resumption: ExecutableResumptionV1,
    pub branch_command_evidence: Vec<ProcessCommandEvidenceV1>,
    pub branch_candidate: CandidateDeltaId,
    pub authoritative_base: StateRevisionId,
    pub authoritative_run: RunId,
    pub authoritative_activation: ActivationId,
    pub authoritative_command_evidence: Vec<ProcessCommandEvidenceV1>,
    pub authoritative_candidate: CandidateDeltaId,
    pub authorization: clause_package::IssuedAdmissionAuthorizationOccurrenceId,
    pub judgment: JudgmentOccurrenceId,
    pub admission: AdmissionOccurrenceId,
    pub successor: StateRevisionId,
    pub causal_records: Vec<ProcessCausalRecordV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessReconnectAdmissionV1 {
    pub state: ExecutableStateRevisionV1,
    pub projection: Option<ExecutableProjectedObservationV1>,
    pub explanation: ProcessBranchExplanationV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessBranchPinV1 {
    ParentState,
    ProgramRevision,
    Package,
    Application,
    Session,
    RuntimePolicy,
    RootPolicy,
    InputEvidence,
    PhysicalPlan,
    Budget,
    Allocation,
    BranchCandidate,
    AuthoritativeBase,
}

#[derive(Debug)]
pub enum ProcessBranchErrorV1 {
    Session(PersistentProcessSessionErrorV1),
    PinMismatch(ProcessBranchPinV1),
    MissingOccurrence,
    AlreadyProposed,
    MissingProposal,
    AlreadyAdjudicated,
    UnexpectedCandidate,
    MissingInputObservation(StepId),
    MissingCausalRecord(CausalRef),
}

impl fmt::Display for ProcessBranchErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Session(error) => write!(formatter, "process branch session failed: {error}"),
            Self::PinMismatch(pin) => write!(formatter, "process branch pin mismatch: {pin:?}"),
            Self::MissingOccurrence => {
                formatter.write_str("process branch requires one exact occurrence")
            }
            Self::AlreadyProposed => {
                formatter.write_str("process branch already retains a reconnect proposal")
            }
            Self::MissingProposal => {
                formatter.write_str("process branch has no reconnect proposal")
            }
            Self::AlreadyAdjudicated => {
                formatter.write_str("process branch already retains an adjudication")
            }
            Self::UnexpectedCandidate => {
                formatter.write_str("process branch candidate does not match its retained evidence")
            }
            Self::MissingInputObservation(step) => {
                write!(
                    formatter,
                    "process branch Step {step:?} lacks its entered input Observation"
                )
            }
            Self::MissingCausalRecord(occurrence) => {
                write!(
                    formatter,
                    "process branch lacks causal record {occurrence:?}"
                )
            }
        }
    }
}

impl Error for ProcessBranchErrorV1 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Session(error) => Some(error),
            _ => None,
        }
    }
}

impl From<PersistentProcessSessionErrorV1> for ProcessBranchErrorV1 {
    fn from(error: PersistentProcessSessionErrorV1) -> Self {
        Self::Session(error)
    }
}

/// One retained non-authoritative process branch. The branch owns its live
/// session and keeps its CandidateDelta hidden; adjudication occurs only in a
/// separately supplied authoritative session.
pub struct ForkedProcessBranchV1 {
    session: PersistentProcessSessionV1,
    pins: ProcessBranchPinsV1,
    ancestry: ProcessBranchAncestryV1,
    suspension: ExecutableSuspensionV1,
    proposal: Option<ProcessReconnectEvidenceV1>,
    explanation: Option<ProcessBranchExplanationV1>,
}

impl ForkedProcessBranchV1 {
    pub fn fork(
        authoritative: &PersistentProcessSessionV1,
        mut branch: PersistentProcessSessionV1,
        disconnect_tick: u64,
        disconnect_occurrence: &[u8],
    ) -> Result<Self, ProcessBranchErrorV1> {
        let authoritative_facts = authoritative.authority_facts()?;
        let branch_facts = branch.authority_facts()?;
        require_same(
            authoritative.world_base() == branch.world_base(),
            ProcessBranchPinV1::ParentState,
        )?;
        require_same(
            authoritative.program_revision() == branch.program_revision(),
            ProcessBranchPinV1::ProgramRevision,
        )?;
        require_same(
            authoritative.package()? == branch.package()?,
            ProcessBranchPinV1::Package,
        )?;
        require_same(
            authoritative.application()? == branch.application()?,
            ProcessBranchPinV1::Application,
        )?;
        require_same(
            authoritative.runtime_session() == branch.runtime_session(),
            ProcessBranchPinV1::Session,
        )?;
        require_same(
            authoritative_facts.policy == branch_facts.policy,
            ProcessBranchPinV1::RuntimePolicy,
        )?;
        require_same(
            authoritative_facts.root_policy == branch_facts.root_policy,
            ProcessBranchPinV1::RootPolicy,
        )?;
        require_same(
            authoritative_facts.occurrence_ingress.evidence
                == branch_facts.occurrence_ingress.evidence,
            ProcessBranchPinV1::InputEvidence,
        )?;
        require_same(
            authoritative.allocation().physical_plan() == branch.allocation().physical_plan(),
            ProcessBranchPinV1::PhysicalPlan,
        )?;
        require_same(
            authoritative_facts.budget_units == branch_facts.budget_units,
            ProcessBranchPinV1::Budget,
        )?;
        require_same(
            authoritative.allocation().root() != branch.allocation().root(),
            ProcessBranchPinV1::Allocation,
        )?;

        let run = branch.run()?;
        let activation = branch.activation()?;
        let disconnect = branch.apply_opaque_input(disconnect_occurrence)?.clone();
        let suspension = branch.suspend()?;
        let pins = ProcessBranchPinsV1 {
            parent_state: branch.world_base(),
            program_revision: branch.program_revision(),
            package: branch.package()?,
            application: branch.application()?,
            session: branch.runtime_session(),
            runtime_policy: branch_facts.policy,
            root_policy: branch_facts.root_policy,
            input_evidence: branch_facts.occurrence_ingress.evidence,
            physical_plan: branch.allocation().physical_plan(),
            budget_units: branch_facts.budget_units,
            disconnect_tick,
        };
        let ancestry = ProcessBranchAncestryV1 {
            parent_state: pins.parent_state,
            run,
            activation,
            disconnect_step: disconnect.id,
            suspension_step: suspension.step,
            continuation: suspension.continuation,
        };
        Ok(Self {
            session: branch,
            pins,
            ancestry,
            suspension,
            proposal: None,
            explanation: None,
        })
    }

    #[must_use]
    pub const fn pins(&self) -> ProcessBranchPinsV1 {
        self.pins
    }

    #[must_use]
    pub const fn ancestry(&self) -> ProcessBranchAncestryV1 {
        self.ancestry
    }

    #[must_use]
    pub const fn suspension(&self) -> ExecutableSuspensionV1 {
        self.suspension
    }

    #[must_use]
    pub fn proposal(&self) -> Option<&ProcessReconnectEvidenceV1> {
        self.proposal.as_ref()
    }

    #[must_use]
    pub fn explanation(&self) -> Option<&ProcessBranchExplanationV1> {
        self.explanation.as_ref()
    }

    pub fn resume_and_propose(
        &mut self,
        occurrences: &[Vec<u8>],
    ) -> Result<ProcessReconnectEvidenceV1, ProcessBranchErrorV1> {
        if self.proposal.is_some() {
            return Err(ProcessBranchErrorV1::AlreadyProposed);
        }
        let (last, prefix) = occurrences
            .split_last()
            .ok_or(ProcessBranchErrorV1::MissingOccurrence)?;
        let resumption = self.session.resume()?;
        let mut command_evidence = Vec::with_capacity(occurrences.len());
        for occurrence in prefix {
            let step = self.session.apply_opaque_input(occurrence)?.clone();
            retain_command_evidence(occurrence, &step, &mut command_evidence)?;
        }
        let step = self
            .session
            .apply_opaque_input_and_emit_candidate(last)?
            .clone();
        retain_command_evidence(last, &step, &mut command_evidence)?;
        let candidate = self
            .session
            .candidate()?
            .cloned()
            .ok_or(ProcessBranchErrorV1::UnexpectedCandidate)?;
        if candidate.base != self.pins.parent_state || candidate.produced_by != step.id {
            return Err(ProcessBranchErrorV1::UnexpectedCandidate);
        }
        let evidence = ProcessReconnectEvidenceV1 {
            pins: self.pins,
            ancestry: self.ancestry,
            resumption,
            command_evidence,
            candidate: candidate.id,
            candidate_step: candidate.produced_by,
        };
        self.proposal = Some(evidence.clone());
        Ok(evidence)
    }

    pub fn adjudicate(
        &mut self,
        authoritative: &mut PersistentProcessSessionV1,
        submitted: &ProcessReconnectEvidenceV1,
        plan: &CheckedReconnectAdmissionPlanV1,
    ) -> Result<ProcessReconnectAdmissionV1, ProcessBranchErrorV1> {
        if self.explanation.is_some() {
            return Err(ProcessBranchErrorV1::AlreadyAdjudicated);
        }
        let retained = self
            .proposal
            .as_ref()
            .ok_or(ProcessBranchErrorV1::MissingProposal)?;
        validate_submission(retained, submitted)?;
        require_same(
            plan.branch_candidate == retained.candidate,
            ProcessBranchPinV1::BranchCandidate,
        )?;
        require_same(
            plan.authoritative_base == authoritative.world_base(),
            ProcessBranchPinV1::AuthoritativeBase,
        )?;
        validate_authoritative(authoritative, self.pins)?;
        let (last, prefix) = plan
            .occurrences
            .split_last()
            .ok_or(ProcessBranchErrorV1::MissingOccurrence)?;
        let authoritative_run = authoritative.run()?;
        let authoritative_activation = authoritative.activation()?;
        let mut command_evidence = Vec::with_capacity(plan.occurrences.len());
        for occurrence in prefix {
            let step = authoritative.apply_opaque_input(occurrence)?.clone();
            retain_command_evidence(occurrence, &step, &mut command_evidence)?;
        }
        let candidate_step = authoritative
            .apply_opaque_input_and_emit_candidate(last)?
            .clone();
        retain_command_evidence(last, &candidate_step, &mut command_evidence)?;
        let candidate = authoritative
            .candidate()?
            .cloned()
            .ok_or(ProcessBranchErrorV1::UnexpectedCandidate)?;
        if candidate.base != plan.authoritative_base || candidate.produced_by != candidate_step.id {
            return Err(ProcessBranchErrorV1::UnexpectedCandidate);
        }
        let authorization = authoritative.issue_candidate_admission_authorization()?;
        let (state, projection) =
            authoritative.admit_issued_candidate_with_projection(authorization)?;
        let decision = authoritative
            .carrier()?
            .decision_by_occurrence(state.admission)
            .cloned()
            .ok_or(ProcessBranchErrorV1::MissingCausalRecord(
                CausalRef::Admission(state.admission),
            ))?;
        let mut causal_records = Vec::new();
        retain_causal(
            self.session.carrier()?,
            CausalRef::Step(StepRef {
                run: self.ancestry.run,
                activation: self.ancestry.activation,
                step: self.ancestry.suspension_step,
            }),
            &mut causal_records,
        )?;
        retain_causal(
            self.session.carrier()?,
            CausalRef::Resumption(retained.resumption.occurrence),
            &mut causal_records,
        )?;
        for evidence in &retained.command_evidence {
            retain_causal(
                self.session.carrier()?,
                CausalRef::Step(StepRef {
                    run: self.ancestry.run,
                    activation: self.ancestry.activation,
                    step: evidence.step,
                }),
                &mut causal_records,
            )?;
        }
        retain_causal(
            self.session.carrier()?,
            CausalRef::CandidateDelta(retained.candidate),
            &mut causal_records,
        )?;
        for evidence in &command_evidence {
            retain_causal(
                authoritative.carrier()?,
                CausalRef::Step(StepRef {
                    run: authoritative_run,
                    activation: authoritative_activation,
                    step: evidence.step,
                }),
                &mut causal_records,
            )?;
        }
        retain_causal(
            authoritative.carrier()?,
            CausalRef::CandidateDelta(candidate.id),
            &mut causal_records,
        )?;
        retain_causal(
            authoritative.carrier()?,
            CausalRef::Judgment(decision.verdict),
            &mut causal_records,
        )?;
        retain_causal(
            authoritative.carrier()?,
            CausalRef::Admission(state.admission),
            &mut causal_records,
        )?;
        let explanation = ProcessBranchExplanationV1 {
            pins: self.pins,
            ancestry: self.ancestry,
            resumption: retained.resumption,
            branch_command_evidence: retained.command_evidence.clone(),
            branch_candidate: retained.candidate,
            authoritative_base: plan.authoritative_base,
            authoritative_run,
            authoritative_activation,
            authoritative_command_evidence: command_evidence,
            authoritative_candidate: candidate.id,
            authorization,
            judgment: decision.verdict,
            admission: state.admission,
            successor: state.id,
            causal_records,
        };
        self.explanation = Some(explanation.clone());
        Ok(ProcessReconnectAdmissionV1 {
            state,
            projection,
            explanation,
        })
    }

    #[must_use]
    pub fn retained_candidate(&self) -> Option<&ExecutableCandidateV1> {
        self.session.candidate().ok().flatten()
    }
}

fn require_same(value: bool, pin: ProcessBranchPinV1) -> Result<(), ProcessBranchErrorV1> {
    if value {
        Ok(())
    } else {
        Err(ProcessBranchErrorV1::PinMismatch(pin))
    }
}

fn retain_command_evidence(
    occurrence: &[u8],
    step: &ExecutableStepV1,
    evidence: &mut Vec<ProcessCommandEvidenceV1>,
) -> Result<(), ProcessBranchErrorV1> {
    let observation = step
        .input_observation
        .ok_or(ProcessBranchErrorV1::MissingInputObservation(step.id))?;
    evidence.push(ProcessCommandEvidenceV1 {
        occurrence: occurrence.to_vec(),
        step: step.id,
        observation,
    });
    Ok(())
}

fn validate_submission(
    retained: &ProcessReconnectEvidenceV1,
    submitted: &ProcessReconnectEvidenceV1,
) -> Result<(), ProcessBranchErrorV1> {
    for (matches, pin) in [
        (
            submitted.pins.parent_state == retained.pins.parent_state,
            ProcessBranchPinV1::ParentState,
        ),
        (
            submitted.pins.program_revision == retained.pins.program_revision,
            ProcessBranchPinV1::ProgramRevision,
        ),
        (
            submitted.pins.package == retained.pins.package,
            ProcessBranchPinV1::Package,
        ),
        (
            submitted.pins.application == retained.pins.application,
            ProcessBranchPinV1::Application,
        ),
        (
            submitted.pins.session == retained.pins.session,
            ProcessBranchPinV1::Session,
        ),
        (
            submitted.pins.runtime_policy == retained.pins.runtime_policy,
            ProcessBranchPinV1::RuntimePolicy,
        ),
        (
            submitted.pins.root_policy == retained.pins.root_policy,
            ProcessBranchPinV1::RootPolicy,
        ),
        (
            submitted.pins.input_evidence == retained.pins.input_evidence,
            ProcessBranchPinV1::InputEvidence,
        ),
        (
            submitted.pins.physical_plan == retained.pins.physical_plan,
            ProcessBranchPinV1::PhysicalPlan,
        ),
        (
            submitted.pins.budget_units == retained.pins.budget_units,
            ProcessBranchPinV1::Budget,
        ),
        (
            submitted.candidate == retained.candidate,
            ProcessBranchPinV1::BranchCandidate,
        ),
    ] {
        require_same(matches, pin)?;
    }
    if submitted != retained {
        return Err(ProcessBranchErrorV1::UnexpectedCandidate);
    }
    Ok(())
}

fn validate_authoritative(
    authoritative: &PersistentProcessSessionV1,
    pins: ProcessBranchPinsV1,
) -> Result<(), ProcessBranchErrorV1> {
    let facts = authoritative.authority_facts()?;
    for (matches, pin) in [
        (
            authoritative.program_revision() == pins.program_revision,
            ProcessBranchPinV1::ProgramRevision,
        ),
        (
            authoritative.package()? == pins.package,
            ProcessBranchPinV1::Package,
        ),
        (
            authoritative.application()? == pins.application,
            ProcessBranchPinV1::Application,
        ),
        (
            authoritative.runtime_session() == pins.session,
            ProcessBranchPinV1::Session,
        ),
        (
            facts.policy == pins.runtime_policy,
            ProcessBranchPinV1::RuntimePolicy,
        ),
        (
            facts.root_policy == pins.root_policy,
            ProcessBranchPinV1::RootPolicy,
        ),
        (
            facts.occurrence_ingress.evidence == pins.input_evidence,
            ProcessBranchPinV1::InputEvidence,
        ),
        (
            authoritative.allocation().physical_plan() == pins.physical_plan,
            ProcessBranchPinV1::PhysicalPlan,
        ),
    ] {
        require_same(matches, pin)?;
    }
    Ok(())
}

fn retain_causal(
    carrier: &clause_package::ProcessCarrier,
    occurrence: CausalRef,
    records: &mut Vec<ProcessCausalRecordV1>,
) -> Result<(), ProcessBranchErrorV1> {
    let predecessors = carrier
        .causal_predecessors(occurrence)
        .ok_or(ProcessBranchErrorV1::MissingCausalRecord(occurrence))?
        .iter()
        .copied()
        .collect();
    records.push(ProcessCausalRecordV1 {
        occurrence,
        predecessors,
    });
    Ok(())
}
