use std::error::Error;
use std::fmt;

use clause_package::*;
use sha2::{Digest, Sha256};

use super::ProcessRuntime;

const MAGIC: &[u8; 4] = b"CXP1";
const PROGRAM_KIND: &[u8] = b"clause/process-executable-v1";
const CONFIGURATION_KIND: &[u8] = b"clause/process-configuration-v1";
const OCCURRENCE_KIND: &[u8] = b"clause/process-occurrence-v1";
const OCCURRENCE_MAGIC: &[u8; 4] = b"CXO1";
const MAX_PROGRAM_ITEMS: usize = 65_536;
const MAX_EXPRESSION_DEPTH: usize = 64;

#[derive(Clone, Copy, Debug)]
enum RuntimeIdentitySeedV1 {
    Legacy,
    Exact([u8; IDENTITY_BYTES]),
}

#[derive(Clone, Copy, Debug)]
#[repr(u64)]
enum RuntimeIdentityDomainV1 {
    Run = 1,
    Activation = 2,
    Configuration = 3,
    ExternalTrigger = 10,
    CheckerActivation = 22,
    CheckerRun = 30,
    Step = 32,
    CheckerConfigurationBefore = 42,
    CheckerStep = 53,
    CheckerConfigurationAfter = 63,
    Candidate = 80,
    FormationObservation = 84,
    Judgment = 90,
    Admission = 94,
    SyntheticState = 99,
    StateObservation = 100,
    InputObservation = 130,
}

#[derive(Clone, Copy, Debug)]
struct RuntimeIdentityOrdinalsV1 {
    next_run: u64,
    next_activation: u64,
    next_configuration: u64,
    next_step: u64,
    next_input_observation: u64,
    next_state_observation: u64,
    next_checker: u64,
    next_candidate: u64,
}

impl RuntimeIdentityOrdinalsV1 {
    const fn initial() -> Self {
        Self {
            next_run: 1,
            next_activation: 1,
            next_configuration: 1,
            next_step: 1,
            next_input_observation: 1,
            next_state_observation: 0,
            next_checker: 0,
            next_candidate: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutableValueV1 {
    Number(u64),
    Boolean(bool),
}

impl ExecutableValueV1 {
    pub fn number(value: f64) -> Result<Self, ExecutableErrorV1> {
        if !value.is_finite() {
            return Err(ExecutableErrorV1::NumericDomain);
        }
        Ok(Self::Number(canonical_number_bits(value)))
    }

    #[must_use]
    pub fn as_number(self) -> Option<f64> {
        match self {
            Self::Number(bits) => Some(f64::from_bits(bits)),
            Self::Boolean(_) => None,
        }
    }

    #[must_use]
    pub const fn as_boolean(self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(value),
            Self::Number(_) => None,
        }
    }
}

pub fn executable_configuration_term_v1(
    scope: clause_package::TermScope,
    values: &[ExecutableValueV1],
) -> Result<Term, ExecutableErrorV1> {
    executable_values_term(scope, CONFIGURATION_KIND, values)
}

pub fn executable_occurrence_term_v1(
    scope: clause_package::TermScope,
    occurrence: &ExecutableOccurrenceV1,
) -> Result<Term, ExecutableErrorV1> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&occurrence.entry.to_le_bytes());
    encode_values(&mut bytes, &occurrence.arguments)?;
    Term::atom(
        scope,
        OCCURRENCE_KIND.to_vec(),
        bytes,
        EqualityContract::ExactOctetsV1,
    )
    .map_err(|_| ExecutableErrorV1::MalformedProgram)
}

/// Encode one construct-blind external occurrence for transport across a
/// byte-only host boundary.
pub fn encode_executable_occurrence_v1(
    occurrence: &ExecutableOccurrenceV1,
) -> Result<Vec<u8>, ExecutableErrorV1> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(OCCURRENCE_MAGIC);
    bytes.extend_from_slice(&occurrence.entry.to_le_bytes());
    encode_values(&mut bytes, &occurrence.arguments)?;
    Ok(bytes)
}

/// Strictly decode the exact occurrence transport. Dispatch remains on the
/// numeric package entry selected by the checked executable program; this
/// decoder has no Clause construct, game, role, or designation vocabulary.
pub fn decode_executable_occurrence_v1(
    bytes: &[u8],
) -> Result<ExecutableOccurrenceV1, ExecutableErrorV1> {
    let mut decoder = Decoder::new(bytes);
    if decoder.take(OCCURRENCE_MAGIC.len())? != OCCURRENCE_MAGIC {
        return Err(ExecutableErrorV1::MalformedOccurrence);
    }
    let occurrence = ExecutableOccurrenceV1 {
        entry: decoder.u16()?,
        arguments: decoder.values()?,
    };
    if !decoder.is_complete() {
        return Err(ExecutableErrorV1::MalformedOccurrence);
    }
    Ok(occurrence)
}

fn executable_values_term(
    scope: clause_package::TermScope,
    kind: &[u8],
    values: &[ExecutableValueV1],
) -> Result<Term, ExecutableErrorV1> {
    let mut bytes = Vec::new();
    encode_values(&mut bytes, values)?;
    Term::atom(scope, kind.to_vec(), bytes, EqualityContract::ExactOctetsV1)
        .map_err(|_| ExecutableErrorV1::MalformedProgram)
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExecutableExpressionV1 {
    Constant(ExecutableValueV1),
    Slot(u16),
    Argument(u16),
    Add(Box<Self>, Box<Self>),
    Subtract(Box<Self>, Box<Self>),
    Multiply(Box<Self>, Box<Self>),
    Divide(Box<Self>, Box<Self>),
    Clamp(Box<Self>, Box<Self>, Box<Self>),
    GreaterThan(Box<Self>, Box<Self>),
    LessThanOrEqual(Box<Self>, Box<Self>),
    Equal(Box<Self>, Box<Self>),
    And(Box<Self>, Box<Self>),
    Not(Box<Self>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableRuleV1 {
    pub entry: u16,
    pub predicates: Vec<ExecutableExpressionV1>,
    pub assignments: Vec<(u16, ExecutableExpressionV1)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableProgramV1 {
    pub initial_configuration: Vec<ExecutableValueV1>,
    pub rules: Vec<ExecutableRuleV1>,
}

impl ExecutableProgramV1 {
    pub fn encode_term(&self, scope: clause_package::TermScope) -> Result<Term, ExecutableErrorV1> {
        validate_program(self)?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        encode_values(&mut bytes, &self.initial_configuration)?;
        encode_count(&mut bytes, self.rules.len())?;
        for rule in &self.rules {
            bytes.extend_from_slice(&rule.entry.to_le_bytes());
            encode_count(&mut bytes, rule.predicates.len())?;
            for predicate in &rule.predicates {
                encode_expression(&mut bytes, predicate)?;
            }
            encode_count(&mut bytes, rule.assignments.len())?;
            for (slot, expression) in &rule.assignments {
                bytes.extend_from_slice(&slot.to_le_bytes());
                encode_expression(&mut bytes, expression)?;
            }
        }
        Term::atom(
            scope,
            PROGRAM_KIND.to_vec(),
            bytes,
            EqualityContract::ExactOctetsV1,
        )
        .map_err(|_| ExecutableErrorV1::MalformedProgram)
    }

    fn decode_term(term: &Term) -> Result<Self, ExecutableErrorV1> {
        let atom = term.as_atom().ok_or(ExecutableErrorV1::MalformedProgram)?;
        if atom.kind() != PROGRAM_KIND
            || atom.equality_contract() != EqualityContract::ExactOctetsV1
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let mut decoder = Decoder::new(atom.canonical_payload());
        if decoder.take(MAGIC.len())? != MAGIC {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let initial_configuration = decoder.values()?;
        let rule_count = decoder.count()?;
        let mut rules = Vec::with_capacity(rule_count);
        for _ in 0..rule_count {
            let entry = decoder.u16()?;
            let predicate_count = decoder.count()?;
            let mut predicates = Vec::with_capacity(predicate_count);
            for _ in 0..predicate_count {
                predicates.push(decoder.expression(0)?);
            }
            let assignment_count = decoder.count()?;
            let mut assignments = Vec::with_capacity(assignment_count);
            for _ in 0..assignment_count {
                assignments.push((decoder.u16()?, decoder.expression(0)?));
            }
            rules.push(ExecutableRuleV1 {
                entry,
                predicates,
                assignments,
            });
        }
        if !decoder.is_complete() {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let program = Self {
            initial_configuration,
            rules,
        };
        validate_program(&program)?;
        Ok(program)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableOccurrenceV1 {
    pub entry: u16,
    pub arguments: Vec<ExecutableValueV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableStepV1 {
    pub id: StepId,
    pub before: ConfigurationId,
    pub after: ConfigurationId,
    pub input_observation: Option<ObservationId>,
    pub occurrence: ExecutableOccurrenceV1,
    pub rule_applied: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableCandidateV1 {
    pub id: CandidateDeltaId,
    pub base: StateRevisionId,
    pub produced_by: StepId,
    pub configuration: Vec<ExecutableValueV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableJudgmentV1 {
    pub id: JudgmentOccurrenceId,
    pub candidate: CandidateDeltaId,
    pub accepted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableAdmissionV1 {
    pub id: AdmissionOccurrenceId,
    pub candidate: CandidateDeltaId,
    pub judgment: JudgmentOccurrenceId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableStateRevisionV1 {
    pub id: StateRevisionId,
    pub predecessor: StateRevisionId,
    pub admission: AdmissionOccurrenceId,
    pub configuration: Vec<ExecutableValueV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableObservationV1 {
    pub id: ObservationId,
    pub state: StateRevisionId,
    pub value: Vec<ExecutableValueV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableBoundaryFactV1 {
    pub boundary: BoundaryRef,
    pub evidence: ExternalEvidenceRef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableAuthorityFactsV1 {
    pub program_revision: ProgramRevisionId,
    pub session: RuntimeSessionId,
    pub initial_state: StateRevisionId,
    pub policy: RuntimePolicyId,
    pub session_start: SessionStartOccurrenceId,
    pub root_policy: RootPolicyId,
    pub judgment_authority: RootJudgmentAuthorityRef,
    pub occurrence_ingress: ExecutableBoundaryFactV1,
    pub judgment_ingress: ExecutableBoundaryFactV1,
    pub admission_ingress: ExecutableBoundaryFactV1,
    pub budget_units: u64,
}

#[derive(Clone, Debug)]
struct CarrierExecutionV1 {
    facts: ExecutableAuthorityFactsV1,
    mode: ModeId,
    checker_mode: ModeId,
    state_started: bool,
    remaining_budget: u64,
    prior_step: Option<StepRef>,
    epoch_origin: CausalRef,
    state_base_support: SupportSource,
}

#[derive(Clone, Debug)]
struct PreparedCarrierSettlementV1 {
    judgment: JudgmentOccurrenceV2,
    decision: StateAdmissionDecisionV2,
    executable_admission: ExecutableAdmissionV1,
    executable_state: ExecutableStateRevisionV1,
    successor: StateRevision,
}

#[derive(Clone, Copy, Debug)]
enum CheckerOriginV1 {
    Root(RootTrigger),
    ChildOf(StepRef),
}

pub struct ExecutableProcessRuntimeV1 {
    carrier: ProcessRuntime,
    package: ProcessPackageId,
    application: ApplicationId,
    run: RunId,
    activation: clause_package::ActivationId,
    configuration_id: ConfigurationId,
    configuration: Vec<ExecutableValueV1>,
    program: ExecutableProgramV1,
    last_step: Option<ExecutableStepV1>,
    candidate: Option<ExecutableCandidateV1>,
    judgment: Option<ExecutableJudgmentV1>,
    admission: Option<ExecutableAdmissionV1>,
    state: Option<ExecutableStateRevisionV1>,
    carrier_execution: Option<CarrierExecutionV1>,
    identity_seed: RuntimeIdentitySeedV1,
    identity_ordinals: RuntimeIdentityOrdinalsV1,
    active_candidate_ordinal: Option<u64>,
}

impl ExecutableProcessRuntimeV1 {
    pub fn instantiate(
        package: CheckedProcessPackage,
        authority: clause_package::AuthorityStore,
        application: ApplicationId,
    ) -> Result<Self, ExecutableErrorV1> {
        let program = executable_program_v1(&package, application)?;
        let package_id = package.id();
        let carrier = ProcessRuntime::instantiate(package, authority)
            .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
        Self::from_parts(
            carrier,
            package_id,
            application,
            program,
            RuntimeIdentitySeedV1::Legacy,
        )
    }

    pub(super) fn instantiate_session(
        package: CheckedProcessPackage,
        authority: clause_package::AuthorityStore,
        application: ApplicationId,
        identity_seed: [u8; IDENTITY_BYTES],
    ) -> Result<Self, ExecutableErrorV1> {
        let program = executable_program_v1(&package, application)?;
        let package_id = package.id();
        let carrier = ProcessRuntime::instantiate(package, authority)
            .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
        Self::from_parts(
            carrier,
            package_id,
            application,
            program,
            RuntimeIdentitySeedV1::Exact(identity_seed),
        )
    }

    fn from_parts(
        carrier: ProcessRuntime,
        package: ProcessPackageId,
        application: ApplicationId,
        program: ExecutableProgramV1,
        identity_seed: RuntimeIdentitySeedV1,
    ) -> Result<Self, ExecutableErrorV1> {
        if carrier.carrier().application(application).is_none() {
            return Err(ExecutableErrorV1::UnknownApplication);
        }
        let run = RunId::from_bytes(runtime_identity_bytes(
            identity_seed,
            RuntimeIdentityDomainV1::Run,
            0,
        )?);
        let activation = ActivationId::from_bytes(runtime_identity_bytes(
            identity_seed,
            RuntimeIdentityDomainV1::Activation,
            0,
        )?);
        let configuration_id = ConfigurationId::from_bytes(runtime_identity_bytes(
            identity_seed,
            RuntimeIdentityDomainV1::Configuration,
            0,
        )?);
        Ok(Self {
            carrier,
            package,
            application,
            run,
            activation,
            configuration_id,
            configuration: program.initial_configuration.clone(),
            program,
            last_step: None,
            candidate: None,
            judgment: None,
            admission: None,
            state: None,
            carrier_execution: None,
            identity_seed,
            identity_ordinals: RuntimeIdentityOrdinalsV1::initial(),
            active_candidate_ordinal: None,
        })
    }
}

fn executable_program_v1(
    package: &CheckedProcessPackage,
    application: ApplicationId,
) -> Result<ExecutableProgramV1, ExecutableErrorV1> {
    let declaration = package
        .constitution()
        .application_by_id(application)
        .ok_or(ExecutableErrorV1::UnknownApplication)?;
    let mut executable = None;
    for dependency in &declaration.form.dependency_closure {
        let LocalSemanticDependencyV2::ExternalReference(term) = dependency else {
            continue;
        };
        if term
            .as_atom()
            .is_some_and(|atom| atom.kind() == PROGRAM_KIND)
        {
            if executable.replace(term).is_some() {
                return Err(ExecutableErrorV1::AmbiguousProgram);
            }
        }
    }
    let program =
        ExecutableProgramV1::decode_term(executable.ok_or(ExecutableErrorV1::MissingProgram)?)?;
    Ok(program)
}

impl ExecutableProcessRuntimeV1 {
    /// Start the unique stateful, effect-free Mode constituted for this
    /// Application. The caller supplies operational authority facts only;
    /// package structure supplies the executable Mode and all semantic pins.
    pub fn start_carrier_process(
        &mut self,
        facts: ExecutableAuthorityFactsV1,
    ) -> Result<(), ExecutableCarrierErrorV1> {
        if self.carrier_execution.is_some() {
            return Err(ExecutableCarrierErrorV1::AlreadyStarted);
        }
        let constitution = self.carrier.carrier().constitution();
        let declaration = constitution.application_by_id(self.application).ok_or(
            ExecutableCarrierErrorV1::Executable(ExecutableErrorV1::UnknownApplication),
        )?;
        let snapshot = constitution.snapshot();
        let mut selected = None;
        for local in &declaration.form.eligible_modes {
            let mode = ModeId {
                operator: OperatorRef {
                    snapshot,
                    local: declaration.form.operator,
                },
                local: *local,
            };
            let record = constitution
                .mode_by_id(mode)
                .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
            if record.contract.state_delta_domain.is_some()
                && record.contract.effect_intents.is_empty()
            {
                if selected.replace(mode).is_some() {
                    return Err(ExecutableCarrierErrorV1::AmbiguousStatefulMode);
                }
            }
        }
        let mode = selected.ok_or(ExecutableCarrierErrorV1::MissingStatefulMode)?;
        let mode_record = constitution
            .mode_by_id(mode)
            .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
        let executable = constitution
            .executable_contract(self.application, mode)
            .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
        if !executable.authorization_requirements.is_empty()
            || !mode_record.contract.capability_requirements.is_empty()
            || !mode_record.contract.scheduling_requirements.is_empty()
            || !mode_record.contract.resource_requirements.is_empty()
        {
            return Err(ExecutableCarrierErrorV1::UnsupportedSurface);
        }
        let target = mode_record
            .contract
            .state_delta_domain
            .as_ref()
            .expect("selected stateful Mode has a delta domain");
        let mut checker_mode = None;
        for local in &declaration.form.eligible_modes {
            let candidate = ModeId {
                operator: OperatorRef {
                    snapshot,
                    local: declaration.form.operator,
                },
                local: *local,
            };
            if candidate == mode {
                continue;
            }
            let record = constitution
                .mode_by_id(candidate)
                .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
            if record.contract.state_delta_domain.is_none()
                && record.contract.effect_intents.is_empty()
                && record
                    .contract
                    .formation_checks
                    .binary_search(target)
                    .is_ok()
            {
                if checker_mode.replace(candidate).is_some() {
                    return Err(ExecutableCarrierErrorV1::AmbiguousCheckerMode);
                }
            }
        }
        let checker_mode = checker_mode.ok_or(ExecutableCarrierErrorV1::MissingCheckerMode)?;
        self.carrier_execution = Some(CarrierExecutionV1 {
            facts,
            mode,
            checker_mode,
            state_started: false,
            remaining_budget: facts.budget_units,
            prior_step: None,
            epoch_origin: CausalRef::SessionStart(facts.session_start),
            state_base_support: SupportSource::SessionStart(facts.session_start),
        });
        Ok(())
    }

    pub fn advance_carrier_occurrence(
        &mut self,
        occurrence: ExecutableOccurrenceV1,
    ) -> Result<&ExecutableStepV1, ExecutableCarrierErrorV1> {
        self.advance_carrier_occurrence_inner(occurrence, false)
    }

    pub fn advance_carrier_occurrence_and_emit_candidate(
        &mut self,
        occurrence: ExecutableOccurrenceV1,
    ) -> Result<&ExecutableStepV1, ExecutableCarrierErrorV1> {
        self.advance_carrier_occurrence_inner(occurrence, true)
    }

    fn advance_carrier_occurrence_inner(
        &mut self,
        occurrence: ExecutableOccurrenceV1,
        emit_candidate: bool,
    ) -> Result<&ExecutableStepV1, ExecutableCarrierErrorV1> {
        let execution = self
            .carrier_execution
            .as_ref()
            .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
        if execution.remaining_budget == 0 {
            return Err(ExecutableCarrierErrorV1::BudgetExhausted);
        }
        if self.candidate.is_some() {
            return Err(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::CandidateAlreadyEmitted,
            ));
        }
        let (step_ordinal, next_step_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_step)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (configuration_ordinal, next_configuration_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_configuration)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (observation_ordinal, next_observation_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_input_observation)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let occurrence_id = ObservationId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::InputObservation,
                observation_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let (next_configuration, mut bridge_step) = self
            .prepare_step(occurrence, step_ordinal, configuration_ordinal)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        bridge_step.input_observation = Some(occurrence_id);
        let execution = self
            .carrier_execution
            .as_ref()
            .expect("execution remains started");
        let facts = execution.facts;
        let prior_step = execution.prior_step;
        let mode = execution.mode;
        let remaining_budget = execution.remaining_budget;
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let mut entered_occurrence = EnteredObservationV2 {
            observation: ObservationProposalV2::Value {
                id: occurrence_id,
                value: executable_occurrence_term_v1(scope, &bridge_step.occurrence)
                    .map_err(ExecutableCarrierErrorV1::Executable)?,
                supports: vec![],
            },
            provenance: EnteredThrough {
                boundary: facts.occurrence_ingress.boundary,
                evidence: facts.occurrence_ingress.evidence,
                causes: prior_step.map_or_else(
                    || vec![execution.epoch_origin],
                    |step| vec![CausalRef::Step(step)],
                ),
            },
        };
        let reference = StepRef {
            run: self.run,
            activation: self.activation,
            step: bridge_step.id,
        };
        let mut ingress = Vec::new();
        let mut next_checker_ordinal = self.identity_ordinals.next_checker;
        let mut activation_prerequisite = None;
        if !execution.state_started {
            let trigger = ExternalTriggerOccurrenceId::from_bytes(
                runtime_identity_bytes(
                    self.identity_seed,
                    RuntimeIdentityDomainV1::ExternalTrigger,
                    0,
                )
                .map_err(ExecutableCarrierErrorV1::Executable)?,
            );
            ingress.push(ProcessRecordV2::ExternalTrigger(
                ExternalTriggerOccurrenceV2 {
                    id: trigger,
                    provenance: EnteredThrough {
                        boundary: facts.occurrence_ingress.boundary,
                        evidence: facts.occurrence_ingress.evidence,
                        causes: vec![],
                    },
                },
            ));
            entered_occurrence.provenance.causes = vec![CausalRef::ExternalTrigger(trigger)];
            ingress.push(ProcessRecordV2::EnteredObservation(
                entered_occurrence.clone(),
            ));
            if !emit_candidate {
                let (checker_ordinal, after_checker_ordinal) =
                    stage_runtime_ordinal(next_checker_ordinal)
                        .map_err(ExecutableCarrierErrorV1::Executable)?;
                next_checker_ordinal = after_checker_ordinal;
                let (checker_records, formation, _) = self.prepare_formation_checker(
                    execution.checker_mode,
                    facts,
                    &self.configuration,
                    &self.configuration,
                    CheckerOriginV1::Root(runtime_root_trigger(execution.epoch_origin)?),
                    execution.state_base_support,
                    checker_ordinal,
                    facts.budget_units,
                )?;
                ingress.extend(checker_records);
                activation_prerequisite = Some(activation_formation_prerequisite(
                    self.carrier.carrier().constitution(),
                    self.application,
                    mode,
                    formation,
                )?);
            }
        } else {
            ingress.push(ProcessRecordV2::EnteredObservation(entered_occurrence));
        }
        let mut candidate_checker_step = None;
        let staged_candidate = if emit_candidate {
            let (candidate_ordinal, next_candidate_ordinal) =
                stage_runtime_ordinal(self.identity_ordinals.next_candidate)
                    .map_err(ExecutableCarrierErrorV1::Executable)?;
            let id = CandidateDeltaId::from_bytes(
                runtime_identity_bytes(
                    self.identity_seed,
                    RuntimeIdentityDomainV1::Candidate,
                    candidate_ordinal,
                )
                .map_err(ExecutableCarrierErrorV1::Executable)?,
            );
            let candidate = ExecutableCandidateV1 {
                id,
                base: facts.initial_state,
                produced_by: reference.step,
                configuration: next_configuration.clone(),
            };
            let configuration = executable_configuration_term_v1(scope, &next_configuration)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
            let (checker_ordinal, after_checker_ordinal) =
                stage_runtime_ordinal(next_checker_ordinal)
                    .map_err(ExecutableCarrierErrorV1::Executable)?;
            next_checker_ordinal = after_checker_ordinal;
            let checker_origin = if execution.state_started {
                CheckerOriginV1::ChildOf(prior_step.ok_or(ExecutableCarrierErrorV1::Executable(
                    ExecutableErrorV1::NoStep,
                ))?)
            } else {
                CheckerOriginV1::Root(runtime_root_trigger(execution.epoch_origin)?)
            };
            let (checker_records, formation, checker_step) = self.prepare_formation_checker(
                execution.checker_mode,
                facts,
                &self.configuration,
                &next_configuration,
                checker_origin,
                SupportSource::Observation(occurrence_id),
                checker_ordinal,
                remaining_budget,
            )?;
            ingress.extend(checker_records);
            if execution.state_started {
                candidate_checker_step = Some(checker_step);
            } else {
                activation_prerequisite = Some(activation_formation_prerequisite(
                    self.carrier.carrier().constitution(),
                    self.application,
                    mode,
                    formation,
                )?);
            }
            let carrier_candidate = CandidateDeltaV2 {
                id,
                base: facts.initial_state,
                delta: DomainBoundTermV2 {
                    term: configuration.clone(),
                    evidence: formation,
                },
                proposed_payload: configuration,
                evidence: vec![SupportUse {
                    slot: SupportSlotId::new(0),
                    role: runtime_role_term(scope, b"clause/process-state-base-v1")?,
                    source: execution.state_base_support,
                }],
                obligations: vec![],
            };
            Some((
                candidate_ordinal,
                next_candidate_ordinal,
                candidate,
                carrier_candidate,
            ))
        } else {
            None
        };
        if !execution.state_started {
            let prerequisite = activation_prerequisite
                .ok_or(ExecutableCarrierErrorV1::MissingCheckerPrerequisite)?;
            ingress.push(ProcessRecordV2::Activation(ActivationProposalV2 {
                id: self.activation,
                application: self.application,
                mode,
                pins: activation_pins_v1(
                    self.carrier.carrier().constitution(),
                    self.application,
                    mode,
                    facts,
                    true,
                )?,
                static_basis: ActivationStaticBasis {
                    execution_authorizations: vec![],
                    judgment_authorities: vec![],
                },
                causes: ActivationCauseFrontierV2 {
                    origin: ActivationOrigin::RootedBy(RootTrigger::SessionStart(
                        facts.session_start,
                    )),
                    prerequisites: vec![prerequisite],
                },
                membership: RunMembership::RootOf(self.run),
                initial_configuration: ConfigurationProposal {
                    id: self.configuration_id,
                    value: executable_configuration_term_v1(scope, &self.configuration)
                        .map_err(ExecutableCarrierErrorV1::Executable)?,
                },
            }));
        }
        let after_budget = remaining_budget - 1;
        let mut causes = vec![prior_step.map_or(
            StepCause::ActivationStart(self.activation),
            StepCause::PriorStep,
        )];
        if let Some(checker_step) = candidate_checker_step {
            causes.push(StepCause::PriorStep(checker_step));
            causes.sort_unstable();
        }
        let step = StepProposalV2 {
            id: reference.step,
            run: reference.run,
            activation: reference.activation,
            before: bridge_step.before,
            after: ConfigurationProposal {
                id: bridge_step.after,
                value: executable_configuration_term_v1(scope, &next_configuration)
                    .map_err(ExecutableCarrierErrorV1::Executable)?,
            },
            observed_state: Some(facts.initial_state),
            budget: StepBudgetTransitionV2 {
                before: Budget {
                    remaining_units: remaining_budget,
                },
                consumed_units: 1,
                after: Budget {
                    remaining_units: after_budget,
                },
            },
            causes,
            observations: vec![],
            candidate_delta: staged_candidate
                .as_ref()
                .map(|(_, _, _, candidate)| candidate.clone()),
            outcome: StepOutcomeProposalV2::Progress,
        };
        ingress.push(ProcessRecordV2::Steps(vec![step]));
        self.carrier
            .apply_ingress(&ingress)
            .map_err(ExecutableCarrierErrorV1::Ingress)?;

        self.configuration = next_configuration;
        self.configuration_id = bridge_step.after;
        self.last_step = Some(bridge_step);
        self.identity_ordinals.next_step = next_step_ordinal;
        self.identity_ordinals.next_configuration = next_configuration_ordinal;
        self.identity_ordinals.next_input_observation = next_observation_ordinal;
        self.identity_ordinals.next_checker = next_checker_ordinal;
        if let Some((candidate_ordinal, next_candidate_ordinal, candidate, _)) = staged_candidate {
            self.candidate = Some(candidate);
            self.active_candidate_ordinal = Some(candidate_ordinal);
            self.identity_ordinals.next_candidate = next_candidate_ordinal;
        }
        let execution = self
            .carrier_execution
            .as_mut()
            .expect("execution remains started");
        execution.remaining_budget = after_budget;
        execution.prior_step = Some(reference);
        execution.state_started = true;
        Ok(self
            .last_step
            .as_ref()
            .expect("accepted Step remains retained"))
    }

    pub fn advance(
        &mut self,
        occurrence: ExecutableOccurrenceV1,
    ) -> Result<&ExecutableStepV1, ExecutableErrorV1> {
        if self.candidate.is_some() {
            return Err(ExecutableErrorV1::CandidateAlreadyEmitted);
        }
        let (step_ordinal, next_step_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_step)?;
        let (configuration_ordinal, next_configuration_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_configuration)?;
        let (next, step) = self.prepare_step(occurrence, step_ordinal, configuration_ordinal)?;
        self.configuration_id = step.after;
        self.configuration = next;
        self.last_step = Some(step);
        self.identity_ordinals.next_step = next_step_ordinal;
        self.identity_ordinals.next_configuration = next_configuration_ordinal;
        Ok(self.last_step.as_ref().expect("Step was just installed"))
    }

    fn prepare_step(
        &self,
        occurrence: ExecutableOccurrenceV1,
        step_ordinal: u64,
        configuration_ordinal: u64,
    ) -> Result<(Vec<ExecutableValueV1>, ExecutableStepV1), ExecutableErrorV1> {
        let mut selected = None;
        for rule in self
            .program
            .rules
            .iter()
            .filter(|rule| rule.entry == occurrence.entry)
        {
            let matches = rule
                .predicates
                .iter()
                .try_fold(true, |matches, predicate| {
                    let value = evaluate(predicate, &self.configuration, &occurrence.arguments)?;
                    Ok::<_, ExecutableErrorV1>(
                        matches && value.as_boolean().ok_or(ExecutableErrorV1::TypeMismatch)?,
                    )
                })?;
            if matches {
                selected = Some(rule);
                break;
            }
        }
        let mut next = self.configuration.clone();
        if let Some(rule) = selected {
            for (slot, expression) in &rule.assignments {
                let value = evaluate(expression, &self.configuration, &occurrence.arguments)?;
                let target = next
                    .get_mut(usize::from(*slot))
                    .ok_or(ExecutableErrorV1::UnknownSlot(*slot))?;
                *target = value;
            }
        }
        let before = self.configuration_id;
        let after = ConfigurationId::from_bytes(runtime_identity_bytes(
            self.identity_seed,
            RuntimeIdentityDomainV1::Configuration,
            configuration_ordinal,
        )?);
        let step = ExecutableStepV1 {
            id: StepId::from_bytes(runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::Step,
                step_ordinal,
            )?),
            before,
            after,
            input_observation: None,
            occurrence,
            rule_applied: selected.is_some(),
        };
        Ok((next, step))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the checker proposal must bind every exact semantic and causal input"
    )]
    fn prepare_formation_checker(
        &self,
        checker_mode: ModeId,
        mut facts: ExecutableAuthorityFactsV1,
        before: &[ExecutableValueV1],
        subject: &[ExecutableValueV1],
        origin: CheckerOriginV1,
        support: SupportSource,
        ordinal: u64,
        budget_units: u64,
    ) -> Result<(Vec<ProcessRecordV2>, ObservationId, StepRef), ExecutableCarrierErrorV1> {
        if budget_units == 0 {
            return Err(ExecutableCarrierErrorV1::BudgetExhausted);
        }
        let constitution = self.carrier.carrier().constitution();
        let state_mode = self
            .carrier
            .carrier()
            .constitution()
            .mode_by_id(
                self.carrier_execution
                    .as_ref()
                    .ok_or(ExecutableCarrierErrorV1::NotStarted)?
                    .mode,
            )
            .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
        let target = state_mode
            .contract
            .state_delta_domain
            .clone()
            .ok_or(ExecutableCarrierErrorV1::MissingStatefulMode)?;
        let scope = TermScope {
            universe: constitution.universe(),
            semantics: constitution.semantics(),
        };
        let checker_activation = ActivationId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::CheckerActivation,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let checker_run = RunId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::CheckerRun,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let checker_before = ConfigurationId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::CheckerConfigurationBefore,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let checker_after = ConfigurationId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::CheckerConfigurationAfter,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let formation = ObservationId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::FormationObservation,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let subject = executable_configuration_term_v1(scope, subject)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        facts.budget_units = budget_units;
        let (activation_origin, membership, run) = match origin {
            CheckerOriginV1::Root(root) => (
                ActivationOrigin::RootedBy(root),
                RunMembership::RootOf(checker_run),
                checker_run,
            ),
            CheckerOriginV1::ChildOf(parent) => (
                ActivationOrigin::ChildOf {
                    run: parent.run,
                    parent_activation: parent.activation,
                    parent_step: parent.step,
                },
                RunMembership::ChildIn(parent.run),
                parent.run,
            ),
        };
        let checker_step = StepRef {
            run,
            activation: checker_activation,
            step: StepId::from_bytes(
                runtime_identity_bytes(
                    self.identity_seed,
                    RuntimeIdentityDomainV1::CheckerStep,
                    ordinal,
                )
                .map_err(ExecutableCarrierErrorV1::Executable)?,
            ),
        };
        Ok((
            vec![
                ProcessRecordV2::Activation(ActivationProposalV2 {
                    id: checker_activation,
                    application: self.application,
                    mode: checker_mode,
                    pins: activation_pins_v1(
                        constitution,
                        self.application,
                        checker_mode,
                        facts,
                        true,
                    )?,
                    static_basis: ActivationStaticBasis {
                        execution_authorizations: vec![],
                        judgment_authorities: vec![],
                    },
                    causes: ActivationCauseFrontierV2 {
                        origin: activation_origin,
                        prerequisites: vec![],
                    },
                    membership,
                    initial_configuration: ConfigurationProposal {
                        id: checker_before,
                        value: executable_configuration_term_v1(scope, before)
                            .map_err(ExecutableCarrierErrorV1::Executable)?,
                    },
                }),
                ProcessRecordV2::Steps(vec![StepProposalV2 {
                    id: checker_step.step,
                    run,
                    activation: checker_activation,
                    before: checker_before,
                    after: ConfigurationProposal {
                        id: checker_after,
                        value: subject.clone(),
                    },
                    observed_state: Some(facts.initial_state),
                    budget: StepBudgetTransitionV2 {
                        before: Budget {
                            remaining_units: budget_units,
                        },
                        consumed_units: 1,
                        after: Budget {
                            remaining_units: budget_units - 1,
                        },
                    },
                    causes: vec![StepCause::ActivationStart(checker_activation)],
                    observations: vec![ObservationProposalV2::Formation {
                        id: formation,
                        subject,
                        target,
                        supports: vec![SupportUse {
                            slot: SupportSlotId::new(0),
                            role: runtime_role_term(scope, b"clause/process-formation-support-v1")?,
                            source: support,
                        }],
                    }],
                    candidate_delta: None,
                    outcome: StepOutcomeProposalV2::Progress,
                }]),
            ],
            formation,
            checker_step,
        ))
    }

    /// Submit bridge-produced canonical records to the checked carrier. This
    /// is the only mutation path by which executable output becomes Clause
    /// process state; executable trace values alone carry no authority.
    pub fn apply_carrier_ingress(
        &mut self,
        records: &[ProcessRecordV2],
    ) -> Result<(), ProcessIngressError> {
        self.carrier.apply_ingress(records)
    }

    pub(super) fn establish_root_policy(
        &mut self,
        policy: RootPolicyAnchor,
    ) -> Result<(), AuthorityError> {
        self.carrier.establish_root_policy(policy)
    }

    fn prepare_carrier_settlement(
        &self,
        authorization: AdmissionAuthorizationEvidence,
    ) -> Result<PreparedCarrierSettlementV1, ExecutableCarrierErrorV1> {
        let execution = self
            .carrier_execution
            .as_ref()
            .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
        let facts = execution.facts;
        let producer = execution
            .prior_step
            .ok_or(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::NoStep,
            ))?;
        let candidate = self
            .candidate
            .as_ref()
            .ok_or(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::NoCandidate,
            ))?
            .clone();
        let candidate_ordinal =
            self.active_candidate_ordinal
                .ok_or(ExecutableCarrierErrorV1::Executable(
                    ExecutableErrorV1::NoCandidate,
                ))?;
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let judgment_id = JudgmentOccurrenceId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::Judgment,
                candidate_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let judgment = JudgmentOccurrenceV2 {
            body: JudgmentOccurrenceBodyV2 {
                id: judgment_id,
                judgment: AdmissionJudgment {
                    delta: candidate.id,
                    session: facts.session,
                    policy: facts.policy,
                    claim: AdmissionJudgmentClaim::Verdict(AdmissionDisposition::Admit),
                },
                authority: JudgmentAuthorityEvidence::IrreducibleRoot {
                    policy: facts.root_policy,
                    authority: facts.judgment_authority,
                },
                supports: vec![SupportUse {
                    slot: SupportSlotId::new(0),
                    role: runtime_role_term(scope, b"clause/process-candidate-producer-v1")?,
                    source: SupportSource::Step(producer),
                }],
            },
            provenance: OccurrenceProvenance::EnteredThrough(EnteredThrough {
                boundary: facts.judgment_ingress.boundary,
                evidence: facts.judgment_ingress.evidence,
                causes: vec![CausalRef::CandidateDelta(candidate.id)],
            }),
        };
        let admission_id = AdmissionOccurrenceId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::Admission,
                candidate_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let payload = executable_configuration_term_v1(scope, &candidate.configuration)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        let mut successor = StateRevision {
            id: StateRevisionId::from_bytes(
                runtime_identity_bytes(
                    self.identity_seed,
                    RuntimeIdentityDomainV1::SyntheticState,
                    candidate_ordinal,
                )
                .map_err(ExecutableCarrierErrorV1::Executable)?,
            ),
            session: facts.session,
            predecessor: Some(candidate.base),
            cause: StateRevisionCause::Admission {
                occurrence: admission_id,
                run: producer.run,
                activation: producer.activation,
                step: producer.step,
            },
            canonical_state_snapshot: canonical_term_bytes(&payload)
                .map_err(|_| ExecutableCarrierErrorV1::UnsupportedSurface)?
                .into_boxed_slice(),
            payload,
            policy: facts.policy,
            semantics: scope.semantics,
        };
        successor.id = successor.derived_id();
        let decision = StateAdmissionDecisionV2 {
            occurrence: admission_id,
            delta: candidate.id,
            authorization,
            evidence: vec![SupportUse {
                slot: SupportSlotId::new(0),
                role: runtime_role_term(scope, b"clause/process-admission-verdict-v1")?,
                source: SupportSource::Judgment(judgment_id),
            }],
            verdict: judgment_id,
            obligation_judgments: vec![],
            provenance: EnteredThrough {
                boundary: facts.admission_ingress.boundary,
                evidence: facts.admission_ingress.evidence,
                causes: vec![
                    CausalRef::CandidateDelta(candidate.id),
                    CausalRef::Judgment(judgment_id),
                ],
            },
            outcome: StateAdmissionOutcomeV2::Admit(successor.clone()),
        };
        Ok(PreparedCarrierSettlementV1 {
            judgment,
            decision,
            executable_admission: ExecutableAdmissionV1 {
                id: admission_id,
                candidate: candidate.id,
                judgment: judgment_id,
            },
            executable_state: ExecutableStateRevisionV1 {
                id: successor.id,
                predecessor: candidate.base,
                admission: admission_id,
                configuration: candidate.configuration,
            },
            successor,
        })
    }

    /// Issue the carrier Judgment and Admission for the one computed
    /// candidate, deriving the successor identity from its complete preimage.
    pub fn settle_carrier_process(
        &mut self,
        authorization: AdmissionAuthorizationEvidence,
    ) -> Result<&ExecutableStateRevisionV1, ExecutableCarrierErrorV1> {
        if self.state.is_some() {
            return Err(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::AlreadyAdmitted,
            ));
        }
        let prepared = self.prepare_carrier_settlement(authorization)?;
        let judgment_id = prepared.executable_admission.judgment;
        let candidate_id = prepared.executable_admission.candidate;
        self.carrier
            .apply_ingress(&[
                ProcessRecordV2::Judgment(prepared.judgment),
                ProcessRecordV2::AdmissionDecision(prepared.decision),
            ])
            .map_err(ExecutableCarrierErrorV1::Ingress)?;
        self.judgment = Some(ExecutableJudgmentV1 {
            id: judgment_id,
            candidate: candidate_id,
            accepted: true,
        });
        self.admission = Some(prepared.executable_admission);
        self.state = Some(prepared.executable_state);
        Ok(self.state.as_ref().expect("settled State is retained"))
    }

    pub(super) fn settle_carrier_process_and_start_epoch(
        &mut self,
        authorization: AdmissionAuthorizationEvidence,
    ) -> Result<ExecutableStateRevisionV1, ExecutableCarrierErrorV1> {
        if self.state.is_some() {
            return Err(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::AlreadyAdmitted,
            ));
        }
        let prepared = self.prepare_carrier_settlement(authorization)?;
        let (facts, mode, checker_mode, remaining_budget) = {
            let execution = self
                .carrier_execution
                .as_ref()
                .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
            (
                execution.facts,
                execution.mode,
                execution.checker_mode,
                execution.remaining_budget,
            )
        };
        let (run_ordinal, next_run_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_run)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (activation_ordinal, next_activation_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_activation)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (configuration_ordinal, next_configuration_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_configuration)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let next_run = RunId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::Run,
                run_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let next_activation = ActivationId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::Activation,
                activation_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let next_configuration = ConfigurationId::from_bytes(
            runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::Configuration,
                configuration_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let mut next_facts = facts;
        next_facts.initial_state = prepared.successor.id;
        next_facts.budget_units = remaining_budget;
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let (checker_ordinal, next_checker_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_checker)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (checker_records, formation, _) = self.prepare_formation_checker(
            checker_mode,
            next_facts,
            &prepared.executable_state.configuration,
            &prepared.executable_state.configuration,
            CheckerOriginV1::Root(RootTrigger::Admitted(prepared.executable_admission.id)),
            SupportSource::Admission(prepared.executable_admission.id),
            checker_ordinal,
            remaining_budget,
        )?;
        let prerequisite = activation_formation_prerequisite(
            self.carrier.carrier().constitution(),
            self.application,
            mode,
            formation,
        )?;
        let next_epoch = ProcessRecordV2::Activation(ActivationProposalV2 {
            id: next_activation,
            application: self.application,
            mode,
            pins: activation_pins_v1(
                self.carrier.carrier().constitution(),
                self.application,
                mode,
                next_facts,
                true,
            )?,
            static_basis: ActivationStaticBasis {
                execution_authorizations: vec![],
                judgment_authorities: vec![],
            },
            causes: ActivationCauseFrontierV2 {
                origin: ActivationOrigin::RootedBy(RootTrigger::Admitted(
                    prepared.executable_admission.id,
                )),
                prerequisites: vec![prerequisite],
            },
            membership: RunMembership::RootOf(next_run),
            initial_configuration: ConfigurationProposal {
                id: next_configuration,
                value: executable_configuration_term_v1(
                    scope,
                    &prepared.executable_state.configuration,
                )
                .map_err(ExecutableCarrierErrorV1::Executable)?,
            },
        });
        let mut ingress = vec![
            ProcessRecordV2::Judgment(prepared.judgment),
            ProcessRecordV2::AdmissionDecision(prepared.decision),
        ];
        ingress.extend(checker_records);
        ingress.push(next_epoch);
        self.carrier
            .apply_ingress(&ingress)
            .map_err(ExecutableCarrierErrorV1::Ingress)?;

        let admitted = prepared.executable_state;
        let admission = prepared.executable_admission.id;
        self.run = next_run;
        self.activation = next_activation;
        self.configuration_id = next_configuration;
        self.configuration = admitted.configuration.clone();
        self.candidate = None;
        self.judgment = None;
        self.admission = None;
        self.state = None;
        self.active_candidate_ordinal = None;
        self.identity_ordinals.next_run = next_run_ordinal;
        self.identity_ordinals.next_activation = next_activation_ordinal;
        self.identity_ordinals.next_configuration = next_configuration_ordinal;
        self.identity_ordinals.next_checker = next_checker_ordinal;
        let execution = self
            .carrier_execution
            .as_mut()
            .expect("persistent settlement retains its execution");
        execution.facts = next_facts;
        execution.remaining_budget = remaining_budget;
        execution.prior_step = None;
        execution.state_started = true;
        execution.epoch_origin = CausalRef::Admission(admission);
        execution.state_base_support = SupportSource::Admission(admission);
        Ok(admitted)
    }

    /// Project selected values and enter the projection as an Observation
    /// causally pinned to the exact Admission that created its State.
    pub fn observe_carrier_state(
        &mut self,
        slots: &[u16],
    ) -> Result<ExecutableObservationV1, ExecutableCarrierErrorV1> {
        let observation = self
            .observe(slots)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        let execution = self
            .carrier_execution
            .as_ref()
            .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
        let facts = execution.facts;
        let admission = self
            .admission
            .as_ref()
            .ok_or(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::NoAdmission,
            ))?;
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let value = executable_values_term(scope, CONFIGURATION_KIND, &observation.value)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        let state_role = Term::atom(
            scope,
            b"clause/process-observed-state-v1".to_vec(),
            observation.state.as_bytes().to_vec(),
            EqualityContract::ExactOctetsV1,
        )
        .map_err(|_| ExecutableCarrierErrorV1::UnsupportedSurface)?;
        self.carrier
            .apply_ingress(&[ProcessRecordV2::EnteredObservation(EnteredObservationV2 {
                observation: ObservationProposalV2::Value {
                    id: observation.id,
                    value,
                    supports: vec![SupportUse {
                        slot: SupportSlotId::new(0),
                        role: state_role,
                        source: SupportSource::Admission(admission.id),
                    }],
                },
                provenance: EnteredThrough {
                    boundary: facts.occurrence_ingress.boundary,
                    evidence: facts.occurrence_ingress.evidence,
                    causes: vec![CausalRef::Admission(admission.id)],
                },
            })])
            .map_err(ExecutableCarrierErrorV1::Ingress)?;
        Ok(observation)
    }

    pub fn emit_candidate(
        &mut self,
        base: StateRevisionId,
    ) -> Result<&ExecutableCandidateV1, ExecutableErrorV1> {
        if self.candidate.is_some() {
            return Err(ExecutableErrorV1::CandidateAlreadyEmitted);
        }
        let produced_by = self.last_step.as_ref().ok_or(ExecutableErrorV1::NoStep)?.id;
        let (candidate_ordinal, next_candidate_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_candidate)?;
        self.candidate = Some(ExecutableCandidateV1 {
            id: CandidateDeltaId::from_bytes(runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::Candidate,
                candidate_ordinal,
            )?),
            base,
            produced_by,
            configuration: self.configuration.clone(),
        });
        self.active_candidate_ordinal = Some(candidate_ordinal);
        self.identity_ordinals.next_candidate = next_candidate_ordinal;
        Ok(self
            .candidate
            .as_ref()
            .expect("candidate was just installed"))
    }

    pub fn judge(&mut self, accepted: bool) -> Result<&ExecutableJudgmentV1, ExecutableErrorV1> {
        if self.judgment.is_some() {
            return Err(ExecutableErrorV1::AlreadyJudged);
        }
        let candidate = self
            .candidate
            .as_ref()
            .ok_or(ExecutableErrorV1::NoCandidate)?;
        let candidate_ordinal = self
            .active_candidate_ordinal
            .ok_or(ExecutableErrorV1::NoCandidate)?;
        self.judgment = Some(ExecutableJudgmentV1 {
            id: JudgmentOccurrenceId::from_bytes(runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::Judgment,
                candidate_ordinal,
            )?),
            candidate: candidate.id,
            accepted,
        });
        Ok(self.judgment.as_ref().expect("judgment was just installed"))
    }

    pub fn admit(&mut self) -> Result<&ExecutableStateRevisionV1, ExecutableErrorV1> {
        let candidate_ordinal = self
            .active_candidate_ordinal
            .ok_or(ExecutableErrorV1::NoCandidate)?;
        self.admit_with_state_id(StateRevisionId::from_bytes(runtime_identity_bytes(
            self.identity_seed,
            RuntimeIdentityDomainV1::SyntheticState,
            candidate_ordinal,
        )?))
    }

    pub fn admit_with_state_id(
        &mut self,
        state: StateRevisionId,
    ) -> Result<&ExecutableStateRevisionV1, ExecutableErrorV1> {
        if self.state.is_some() {
            return Err(ExecutableErrorV1::AlreadyAdmitted);
        }
        let candidate = self
            .candidate
            .as_ref()
            .ok_or(ExecutableErrorV1::NoCandidate)?;
        let judgment = self
            .judgment
            .as_ref()
            .ok_or(ExecutableErrorV1::NoJudgment)?;
        if !judgment.accepted || judgment.candidate != candidate.id {
            return Err(ExecutableErrorV1::RejectedJudgment);
        }
        let candidate_ordinal = self
            .active_candidate_ordinal
            .ok_or(ExecutableErrorV1::NoCandidate)?;
        let admission = ExecutableAdmissionV1 {
            id: AdmissionOccurrenceId::from_bytes(runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::Admission,
                candidate_ordinal,
            )?),
            candidate: candidate.id,
            judgment: judgment.id,
        };
        self.state = Some(ExecutableStateRevisionV1 {
            id: state,
            predecessor: candidate.base,
            admission: admission.id,
            configuration: candidate.configuration.clone(),
        });
        self.admission = Some(admission);
        Ok(self.state.as_ref().expect("state was just installed"))
    }

    pub fn observe(&mut self, slots: &[u16]) -> Result<ExecutableObservationV1, ExecutableErrorV1> {
        let state = self.state.as_ref().ok_or(ExecutableErrorV1::NoAdmission)?;
        let value = slots
            .iter()
            .map(|slot| {
                state
                    .configuration
                    .get(usize::from(*slot))
                    .copied()
                    .ok_or(ExecutableErrorV1::UnknownSlot(*slot))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (ordinal, next_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_state_observation)?;
        let observation = ExecutableObservationV1 {
            id: ObservationId::from_bytes(runtime_identity_bytes(
                self.identity_seed,
                RuntimeIdentityDomainV1::StateObservation,
                ordinal,
            )?),
            state: state.id,
            value,
        };
        self.identity_ordinals.next_state_observation = next_ordinal;
        Ok(observation)
    }

    #[must_use]
    pub const fn carrier(&self) -> &ProcessRuntime {
        &self.carrier
    }

    #[must_use]
    pub const fn package(&self) -> ProcessPackageId {
        self.package
    }

    #[must_use]
    pub const fn application(&self) -> ApplicationId {
        self.application
    }

    #[must_use]
    pub const fn run(&self) -> RunId {
        self.run
    }

    #[must_use]
    pub const fn activation(&self) -> clause_package::ActivationId {
        self.activation
    }

    #[must_use]
    pub fn configuration(&self) -> &[ExecutableValueV1] {
        &self.configuration
    }

    #[must_use]
    pub const fn configuration_id(&self) -> ConfigurationId {
        self.configuration_id
    }

    #[must_use]
    pub const fn last_step(&self) -> Option<&ExecutableStepV1> {
        self.last_step.as_ref()
    }

    #[must_use]
    pub const fn candidate(&self) -> Option<&ExecutableCandidateV1> {
        self.candidate.as_ref()
    }

    #[must_use]
    pub const fn judgment(&self) -> Option<&ExecutableJudgmentV1> {
        self.judgment.as_ref()
    }

    #[must_use]
    pub const fn admission(&self) -> Option<&ExecutableAdmissionV1> {
        self.admission.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutableErrorV1 {
    MissingProgram,
    AmbiguousProgram,
    MalformedProgram,
    MalformedOccurrence,
    UnknownApplication,
    CarrierRejected,
    ResourceLimit,
    UnknownSlot(u16),
    UnknownArgument(u16),
    TypeMismatch,
    NumericDomain,
    CandidateAlreadyEmitted,
    NoStep,
    NoCandidate,
    AlreadyJudged,
    NoJudgment,
    RejectedJudgment,
    AlreadyAdmitted,
    NoAdmission,
}

impl fmt::Display for ExecutableErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for ExecutableErrorV1 {}

#[derive(Debug)]
pub enum ExecutableCarrierErrorV1 {
    Executable(ExecutableErrorV1),
    Ingress(ProcessIngressError),
    AlreadyStarted,
    NotStarted,
    MissingStatefulMode,
    AmbiguousStatefulMode,
    MissingCheckerMode,
    AmbiguousCheckerMode,
    MissingCheckerPrerequisite,
    MissingFormationEvidence,
    BudgetExhausted,
    UnsupportedSurface,
}

impl fmt::Display for ExecutableCarrierErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for ExecutableCarrierErrorV1 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Executable(error) => Some(error),
            Self::Ingress(error) => Some(error),
            _ => None,
        }
    }
}

fn runtime_role_term(scope: TermScope, kind: &[u8]) -> Result<Term, ExecutableCarrierErrorV1> {
    Term::atom(
        scope,
        kind.to_vec(),
        Vec::new(),
        EqualityContract::ExactOctetsV1,
    )
    .map_err(|_| ExecutableCarrierErrorV1::UnsupportedSurface)
}

fn runtime_root_trigger(cause: CausalRef) -> Result<RootTrigger, ExecutableCarrierErrorV1> {
    match cause {
        CausalRef::SessionStart(start) => Ok(RootTrigger::SessionStart(start)),
        CausalRef::Admission(admission) => Ok(RootTrigger::Admitted(admission)),
        _ => Err(ExecutableCarrierErrorV1::UnsupportedSurface),
    }
}

fn activation_formation_prerequisite(
    constitution: &ResolvedProgramConstitutionV2,
    application: ApplicationId,
    mode: ModeId,
    observation: ObservationId,
) -> Result<ActivationPrerequisiteUseV2, ExecutableCarrierErrorV1> {
    let state_contract = constitution
        .executable_contract(application, mode)
        .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
    let prerequisite = state_contract
        .dynamic_prerequisites
        .iter()
        .find(|requirement| {
            requirement.occurrence_kind == ActivationPrerequisiteKind::Observation
                && requirement.cardinality.contains(1)
                && matches!(
                    requirement.scope,
                    PrerequisiteScope::SameSemantics | PrerequisiteScope::SameProgramRevision
                )
        })
        .ok_or(ExecutableCarrierErrorV1::MissingCheckerPrerequisite)?;
    Ok(ActivationPrerequisiteUseV2 {
        kind: prerequisite.kind,
        prerequisite: ActivationPrerequisite::Observation(observation),
    })
}

fn activation_pins_v1(
    constitution: &ResolvedProgramConstitutionV2,
    application: ApplicationId,
    mode: ModeId,
    facts: ExecutableAuthorityFactsV1,
    stateful: bool,
) -> Result<ActivationPins, ExecutableCarrierErrorV1> {
    let executable = constitution
        .executable_contract(application, mode)
        .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
    let mode_record = constitution
        .mode_by_id(mode)
        .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
    if !executable.authorization_requirements.is_empty()
        || !mode_record.contract.capability_requirements.is_empty()
        || !mode_record.contract.scheduling_requirements.is_empty()
        || !mode_record.contract.resource_requirements.is_empty()
    {
        return Err(ExecutableCarrierErrorV1::UnsupportedSurface);
    }
    let mut context_requirements = executable.application_context_requirements;
    context_requirements.extend(executable.static_basis.context_requirements);
    context_requirements.sort_unstable();
    context_requirements.dedup();
    let mut constitutive_dependencies = executable.application_dependency_closure;
    constitutive_dependencies.extend(executable.static_basis.constitutive_dependencies);
    constitutive_dependencies.sort_unstable();
    constitutive_dependencies.dedup();
    Ok(ActivationPins {
        semantics: constitution.semantics(),
        snapshot: constitution.snapshot(),
        program_revision: facts.program_revision,
        runtime_session: stateful.then_some(facts.session),
        observed_state: stateful.then_some(facts.initial_state),
        runtime_policy: stateful.then_some(facts.policy),
        context_requirements,
        constitutive_dependencies,
        capabilities: vec![],
        scheduling_requirements: vec![],
        resource_requirements: vec![],
        cancellation_scope: CancellationScope::Activation,
        budget: Budget {
            remaining_units: facts.budget_units,
        },
    })
}

fn validate_program(program: &ExecutableProgramV1) -> Result<(), ExecutableErrorV1> {
    if program.initial_configuration.len() > MAX_PROGRAM_ITEMS
        || program.rules.len() > MAX_PROGRAM_ITEMS
    {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    for rule in &program.rules {
        if rule.predicates.len() > MAX_PROGRAM_ITEMS || rule.assignments.len() > MAX_PROGRAM_ITEMS {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        let mut slots = rule
            .assignments
            .iter()
            .map(|(slot, _)| *slot)
            .collect::<Vec<_>>();
        slots.sort_unstable();
        if slots.windows(2).any(|pair| pair[0] == pair[1])
            || slots
                .iter()
                .any(|slot| usize::from(*slot) >= program.initial_configuration.len())
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
    Ok(())
}

fn evaluate(
    expression: &ExecutableExpressionV1,
    slots: &[ExecutableValueV1],
    arguments: &[ExecutableValueV1],
) -> Result<ExecutableValueV1, ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    match expression {
        E::Constant(value) => Ok(*value),
        E::Slot(slot) => slots
            .get(usize::from(*slot))
            .copied()
            .ok_or(ExecutableErrorV1::UnknownSlot(*slot)),
        E::Argument(argument) => arguments
            .get(usize::from(*argument))
            .copied()
            .ok_or(ExecutableErrorV1::UnknownArgument(*argument)),
        E::Add(left, right) => numeric2(left, right, slots, arguments, |a, b| a + b),
        E::Subtract(left, right) => numeric2(left, right, slots, arguments, |a, b| a - b),
        E::Multiply(left, right) => numeric2(left, right, slots, arguments, |a, b| a * b),
        E::Divide(left, right) => {
            let denominator = number(evaluate(right, slots, arguments)?)?;
            if denominator == 0.0 {
                return Err(ExecutableErrorV1::NumericDomain);
            }
            let numerator = number(evaluate(left, slots, arguments)?)?;
            ExecutableValueV1::number(numerator / denominator)
        }
        E::Clamp(value, lower, upper) => {
            let value = number(evaluate(value, slots, arguments)?)?;
            let lower = number(evaluate(lower, slots, arguments)?)?;
            let upper = number(evaluate(upper, slots, arguments)?)?;
            if lower > upper {
                return Err(ExecutableErrorV1::NumericDomain);
            }
            ExecutableValueV1::number(value.clamp(lower, upper))
        }
        E::GreaterThan(left, right) => compare(left, right, slots, arguments, |a, b| a > b),
        E::LessThanOrEqual(left, right) => compare(left, right, slots, arguments, |a, b| a <= b),
        E::Equal(left, right) => Ok(ExecutableValueV1::Boolean(
            evaluate(left, slots, arguments)? == evaluate(right, slots, arguments)?,
        )),
        E::And(left, right) => Ok(ExecutableValueV1::Boolean(
            boolean(evaluate(left, slots, arguments)?)?
                && boolean(evaluate(right, slots, arguments)?)?,
        )),
        E::Not(value) => Ok(ExecutableValueV1::Boolean(!boolean(evaluate(
            value, slots, arguments,
        )?)?)),
    }
}

fn numeric2(
    left: &ExecutableExpressionV1,
    right: &ExecutableExpressionV1,
    slots: &[ExecutableValueV1],
    arguments: &[ExecutableValueV1],
    operation: impl FnOnce(f64, f64) -> f64,
) -> Result<ExecutableValueV1, ExecutableErrorV1> {
    let left = number(evaluate(left, slots, arguments)?)?;
    let right = number(evaluate(right, slots, arguments)?)?;
    ExecutableValueV1::number(operation(left, right))
}

fn compare(
    left: &ExecutableExpressionV1,
    right: &ExecutableExpressionV1,
    slots: &[ExecutableValueV1],
    arguments: &[ExecutableValueV1],
    operation: impl FnOnce(f64, f64) -> bool,
) -> Result<ExecutableValueV1, ExecutableErrorV1> {
    Ok(ExecutableValueV1::Boolean(operation(
        number(evaluate(left, slots, arguments)?)?,
        number(evaluate(right, slots, arguments)?)?,
    )))
}

fn number(value: ExecutableValueV1) -> Result<f64, ExecutableErrorV1> {
    value.as_number().ok_or(ExecutableErrorV1::TypeMismatch)
}

fn boolean(value: ExecutableValueV1) -> Result<bool, ExecutableErrorV1> {
    value.as_boolean().ok_or(ExecutableErrorV1::TypeMismatch)
}

fn canonical_number_bits(value: f64) -> u64 {
    if value == 0.0 {
        0.0f64.to_bits()
    } else {
        value.to_bits()
    }
}

fn stage_runtime_ordinal(ordinal: u64) -> Result<(u64, u64), ExecutableErrorV1> {
    let next = ordinal
        .checked_add(1)
        .ok_or(ExecutableErrorV1::ResourceLimit)?;
    Ok((ordinal, next))
}

fn runtime_identity_bytes(
    seed: RuntimeIdentitySeedV1,
    domain: RuntimeIdentityDomainV1,
    ordinal: u64,
) -> Result<[u8; clause_package::IDENTITY_BYTES], ExecutableErrorV1> {
    match seed {
        RuntimeIdentitySeedV1::Legacy => {
            let tag = (domain as u64)
                .checked_add(ordinal)
                .and_then(|value| u8::try_from(value).ok())
                .ok_or(ExecutableErrorV1::ResourceLimit)?;
            let mut bytes = [0; clause_package::IDENTITY_BYTES];
            bytes[0] = tag;
            bytes[clause_package::IDENTITY_BYTES - 1] = tag;
            Ok(bytes)
        }
        RuntimeIdentitySeedV1::Exact(identity_seed) => {
            let domain = (domain as u64).to_be_bytes();
            let ordinal = ordinal.to_be_bytes();
            Ok(runtime_domain_hash(
                "clause/runtime-identity/v1",
                &[&identity_seed, &domain, &ordinal],
            ))
        }
    }
}

fn runtime_domain_hash(domain: &str, components: &[&[u8]]) -> [u8; clause_package::IDENTITY_BYTES] {
    debug_assert!(domain.is_ascii());
    let mut hasher = Sha256::new();
    let domain_bytes = domain.as_bytes();
    let domain_length =
        u32::try_from(domain_bytes.len()).expect("fixed runtime hash domain fits U32");
    hasher.update(domain_length.to_be_bytes());
    hasher.update(domain_bytes);
    for component in components {
        let component_length =
            u64::try_from(component.len()).expect("a Rust slice length fits U64");
        hasher.update(component_length.to_be_bytes());
        hasher.update(component);
    }
    hasher.finalize().into()
}

pub(super) fn persistent_candidate_id_v1(
    session: RuntimeSessionId,
    ordinal: u64,
) -> CandidateDeltaId {
    persistent_candidate_id_from_seed_v1(*session.as_bytes(), ordinal)
}

pub(super) fn persistent_candidate_id_from_seed_v1(
    identity_seed: [u8; IDENTITY_BYTES],
    ordinal: u64,
) -> CandidateDeltaId {
    CandidateDeltaId::from_bytes(
        runtime_identity_bytes(
            RuntimeIdentitySeedV1::Exact(identity_seed),
            RuntimeIdentityDomainV1::Candidate,
            ordinal,
        )
        .expect("session identity derivation accepts every u64 ordinal"),
    )
}

fn encode_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), ExecutableErrorV1> {
    let count = u16::try_from(count).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
    bytes.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn encode_values(
    bytes: &mut Vec<u8>,
    values: &[ExecutableValueV1],
) -> Result<(), ExecutableErrorV1> {
    encode_count(bytes, values.len())?;
    for value in values {
        encode_value(bytes, *value);
    }
    Ok(())
}

fn encode_value(bytes: &mut Vec<u8>, value: ExecutableValueV1) {
    match value {
        ExecutableValueV1::Number(bits) => {
            bytes.push(0);
            bytes.extend_from_slice(&bits.to_le_bytes());
        }
        ExecutableValueV1::Boolean(value) => {
            bytes.extend_from_slice(&[1, u8::from(value)]);
        }
    }
}

fn encode_expression(
    bytes: &mut Vec<u8>,
    expression: &ExecutableExpressionV1,
) -> Result<(), ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    match expression {
        E::Constant(value) => {
            bytes.push(0);
            encode_value(bytes, *value);
        }
        E::Slot(slot) => {
            bytes.push(1);
            bytes.extend_from_slice(&slot.to_le_bytes());
        }
        E::Argument(argument) => {
            bytes.push(2);
            bytes.extend_from_slice(&argument.to_le_bytes());
        }
        E::Add(a, b) => encode_binary(bytes, 3, a, b)?,
        E::Subtract(a, b) => encode_binary(bytes, 4, a, b)?,
        E::Multiply(a, b) => encode_binary(bytes, 5, a, b)?,
        E::Divide(a, b) => encode_binary(bytes, 6, a, b)?,
        E::Clamp(a, b, c) => {
            bytes.push(7);
            encode_expression(bytes, a)?;
            encode_expression(bytes, b)?;
            encode_expression(bytes, c)?;
        }
        E::GreaterThan(a, b) => encode_binary(bytes, 8, a, b)?,
        E::LessThanOrEqual(a, b) => encode_binary(bytes, 9, a, b)?,
        E::Equal(a, b) => encode_binary(bytes, 10, a, b)?,
        E::And(a, b) => encode_binary(bytes, 11, a, b)?,
        E::Not(value) => {
            bytes.push(12);
            encode_expression(bytes, value)?;
        }
    }
    Ok(())
}

fn encode_binary(
    bytes: &mut Vec<u8>,
    tag: u8,
    left: &ExecutableExpressionV1,
    right: &ExecutableExpressionV1,
) -> Result<(), ExecutableErrorV1> {
    bytes.push(tag);
    encode_expression(bytes, left)?;
    encode_expression(bytes, right)
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], ExecutableErrorV1> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        self.offset = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, ExecutableErrorV1> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, ExecutableErrorV1> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }
    fn u64(&mut self) -> Result<u64, ExecutableErrorV1> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight bytes"),
        ))
    }
    fn count(&mut self) -> Result<usize, ExecutableErrorV1> {
        Ok(usize::from(self.u16()?))
    }
    fn values(&mut self) -> Result<Vec<ExecutableValueV1>, ExecutableErrorV1> {
        let count = self.count()?;
        (0..count).map(|_| self.value()).collect()
    }
    fn value(&mut self) -> Result<ExecutableValueV1, ExecutableErrorV1> {
        match self.byte()? {
            0 => {
                let value = f64::from_bits(self.u64()?);
                ExecutableValueV1::number(value)
            }
            1 => match self.byte()? {
                0 => Ok(ExecutableValueV1::Boolean(false)),
                1 => Ok(ExecutableValueV1::Boolean(true)),
                _ => Err(ExecutableErrorV1::MalformedProgram),
            },
            _ => Err(ExecutableErrorV1::MalformedProgram),
        }
    }
    fn expression(&mut self, depth: usize) -> Result<ExecutableExpressionV1, ExecutableErrorV1> {
        if depth >= MAX_EXPRESSION_DEPTH {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        use ExecutableExpressionV1 as E;
        let next = depth + 1;
        Ok(match self.byte()? {
            0 => E::Constant(self.value()?),
            1 => E::Slot(self.u16()?),
            2 => E::Argument(self.u16()?),
            3 => E::Add(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            4 => E::Subtract(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            5 => E::Multiply(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            6 => E::Divide(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            7 => E::Clamp(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            8 => E::GreaterThan(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            9 => E::LessThanOrEqual(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            10 => E::Equal(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            11 => E::And(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            12 => E::Not(Box::new(self.expression(next)?)),
            _ => return Err(ExecutableErrorV1::MalformedProgram),
        })
    }
    fn is_complete(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
