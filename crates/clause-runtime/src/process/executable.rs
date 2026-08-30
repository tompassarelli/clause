use std::error::Error;
use std::fmt;

use clause_package::{
    AdmissionOccurrenceId, ApplicationId, CandidateDeltaId, CheckedProcessPackage,
    ConfigurationId, EqualityContract, JudgmentOccurrenceId, LocalSemanticDependencyV2,
    ObservationId, ProcessIngressError, ProcessPackageId, ProcessRecordV2, RunId,
    StateRevisionId, StepId, Term,
};

use super::ProcessRuntime;

const MAGIC: &[u8; 4] = b"CXP1";
const PROGRAM_KIND: &[u8] = b"clause/process-executable-v1";
const CONFIGURATION_KIND: &[u8] = b"clause/process-configuration-v1";
const OCCURRENCE_KIND: &[u8] = b"clause/process-occurrence-v1";
const MAX_PROGRAM_ITEMS: usize = 65_536;
const MAX_EXPRESSION_DEPTH: usize = 64;

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

fn executable_values_term(
    scope: clause_package::TermScope,
    kind: &[u8],
    values: &[ExecutableValueV1],
) -> Result<Term, ExecutableErrorV1> {
    let mut bytes = Vec::new();
    encode_values(&mut bytes, values)?;
    Term::atom(
        scope,
        kind.to_vec(),
        bytes,
        EqualityContract::ExactOctetsV1,
    )
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
    pub fn encode_term(
        &self,
        scope: clause_package::TermScope,
    ) -> Result<Term, ExecutableErrorV1> {
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
        if atom.kind() != PROGRAM_KIND || atom.equality_contract() != EqualityContract::ExactOctetsV1 {
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

pub struct ExecutableProcessRuntimeV1<'a> {
    carrier: ProcessRuntime<'a>,
    package: ProcessPackageId,
    application: ApplicationId,
    run: RunId,
    activation: clause_package::ActivationId,
    configuration_id: ConfigurationId,
    configuration: Vec<ExecutableValueV1>,
    program: ExecutableProgramV1,
    steps: Vec<ExecutableStepV1>,
    candidate: Option<ExecutableCandidateV1>,
    judgment: Option<ExecutableJudgmentV1>,
    admission: Option<ExecutableAdmissionV1>,
    state: Option<ExecutableStateRevisionV1>,
}

impl<'a> ExecutableProcessRuntimeV1<'a> {
    pub fn instantiate(
        package: &'a CheckedProcessPackage,
        authority: &'a clause_package::AuthorityStore,
        application: ApplicationId,
    ) -> Result<Self, ExecutableErrorV1> {
        let declaration = package
            .constitution()
            .application_by_id(application)
            .ok_or(ExecutableErrorV1::UnknownApplication)?;
        let mut executable = None;
        for dependency in &declaration.form.dependency_closure {
            let LocalSemanticDependencyV2::ExternalReference(term) = dependency else {
                continue;
            };
            if term.as_atom().is_some_and(|atom| atom.kind() == PROGRAM_KIND) {
                if executable.replace(term).is_some() {
                    return Err(ExecutableErrorV1::AmbiguousProgram);
                }
            }
        }
        let program = ExecutableProgramV1::decode_term(
            executable.ok_or(ExecutableErrorV1::MissingProgram)?,
        )?;
        let carrier = ProcessRuntime::instantiate(package, authority)
            .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
        if carrier.carrier().application(application).is_none() {
            return Err(ExecutableErrorV1::UnknownApplication);
        }
        Ok(Self {
            carrier,
            package: package.id(),
            application,
            run: RunId::from_bytes(identity_bytes(1)),
            activation: clause_package::ActivationId::from_bytes(identity_bytes(2)),
            configuration_id: ConfigurationId::from_bytes(identity_bytes(3)),
            configuration: program.initial_configuration.clone(),
            program,
            steps: Vec::new(),
            candidate: None,
            judgment: None,
            admission: None,
            state: None,
        })
    }

    pub fn advance(
        &mut self,
        occurrence: ExecutableOccurrenceV1,
    ) -> Result<&ExecutableStepV1, ExecutableErrorV1> {
        if self.candidate.is_some() {
            return Err(ExecutableErrorV1::CandidateAlreadyEmitted);
        }
        let mut selected = None;
        for rule in self.program.rules.iter().filter(|rule| rule.entry == occurrence.entry) {
            let matches = rule.predicates.iter().try_fold(true, |matches, predicate| {
                let value = evaluate(predicate, &self.configuration, &occurrence.arguments)?;
                Ok::<_, ExecutableErrorV1>(matches
                    && value.as_boolean().ok_or(ExecutableErrorV1::TypeMismatch)?)
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
        let ordinal = u8::try_from(self.steps.len() + 1).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
        let before = self.configuration_id;
        let after = ConfigurationId::from_bytes(identity_bytes(ordinal.saturating_add(3)));
        self.configuration_id = after;
        self.configuration = next;
        self.steps.push(ExecutableStepV1 {
            id: StepId::from_bytes(identity_bytes(ordinal.saturating_add(32))),
            before,
            after,
            occurrence,
            rule_applied: selected.is_some(),
        });
        Ok(self.steps.last().expect("Step was just appended"))
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

    pub fn emit_candidate(
        &mut self,
        base: StateRevisionId,
    ) -> Result<&ExecutableCandidateV1, ExecutableErrorV1> {
        if self.candidate.is_some() {
            return Err(ExecutableErrorV1::CandidateAlreadyEmitted);
        }
        let produced_by = self.steps.last().ok_or(ExecutableErrorV1::NoStep)?.id;
        self.candidate = Some(ExecutableCandidateV1 {
            id: CandidateDeltaId::from_bytes(identity_bytes(80)),
            base,
            produced_by,
            configuration: self.configuration.clone(),
        });
        Ok(self.candidate.as_ref().expect("candidate was just installed"))
    }

    pub fn judge(&mut self, accepted: bool) -> Result<&ExecutableJudgmentV1, ExecutableErrorV1> {
        if self.judgment.is_some() {
            return Err(ExecutableErrorV1::AlreadyJudged);
        }
        let candidate = self.candidate.as_ref().ok_or(ExecutableErrorV1::NoCandidate)?;
        self.judgment = Some(ExecutableJudgmentV1 {
            id: JudgmentOccurrenceId::from_bytes(identity_bytes(90)),
            candidate: candidate.id,
            accepted,
        });
        Ok(self.judgment.as_ref().expect("judgment was just installed"))
    }

    pub fn admit(&mut self) -> Result<&ExecutableStateRevisionV1, ExecutableErrorV1> {
        self.admit_with_state_id(StateRevisionId::from_bytes(identity_bytes(99)))
    }

    pub fn admit_with_state_id(
        &mut self,
        state: StateRevisionId,
    ) -> Result<&ExecutableStateRevisionV1, ExecutableErrorV1> {
        if self.state.is_some() {
            return Err(ExecutableErrorV1::AlreadyAdmitted);
        }
        let candidate = self.candidate.as_ref().ok_or(ExecutableErrorV1::NoCandidate)?;
        let judgment = self.judgment.as_ref().ok_or(ExecutableErrorV1::NoJudgment)?;
        if !judgment.accepted || judgment.candidate != candidate.id {
            return Err(ExecutableErrorV1::RejectedJudgment);
        }
        let admission = ExecutableAdmissionV1 {
            id: AdmissionOccurrenceId::from_bytes(identity_bytes(94)),
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

    pub fn observe(&self, slots: &[u16]) -> Result<ExecutableObservationV1, ExecutableErrorV1> {
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
        Ok(ExecutableObservationV1 {
            id: ObservationId::from_bytes(identity_bytes(100)),
            state: state.id,
            value,
        })
    }

    #[must_use]
    pub const fn carrier(&self) -> &ProcessRuntime<'a> {
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
    pub fn steps(&self) -> &[ExecutableStepV1] {
        &self.steps
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

fn validate_program(program: &ExecutableProgramV1) -> Result<(), ExecutableErrorV1> {
    if program.initial_configuration.len() > MAX_PROGRAM_ITEMS || program.rules.len() > MAX_PROGRAM_ITEMS {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    for rule in &program.rules {
        if rule.predicates.len() > MAX_PROGRAM_ITEMS || rule.assignments.len() > MAX_PROGRAM_ITEMS {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        let mut slots = rule.assignments.iter().map(|(slot, _)| *slot).collect::<Vec<_>>();
        slots.sort_unstable();
        if slots.windows(2).any(|pair| pair[0] == pair[1])
            || slots.iter().any(|slot| usize::from(*slot) >= program.initial_configuration.len())
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
        E::Slot(slot) => slots.get(usize::from(*slot)).copied().ok_or(ExecutableErrorV1::UnknownSlot(*slot)),
        E::Argument(argument) => arguments.get(usize::from(*argument)).copied().ok_or(ExecutableErrorV1::UnknownArgument(*argument)),
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
        E::Not(value) => Ok(ExecutableValueV1::Boolean(!boolean(evaluate(value, slots, arguments)?)?)),
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
    if value == 0.0 { 0.0f64.to_bits() } else { value.to_bits() }
}

fn identity_bytes(tag: u8) -> [u8; clause_package::IDENTITY_BYTES] {
    let mut bytes = [0; clause_package::IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[clause_package::IDENTITY_BYTES - 1] = tag;
    bytes
}

fn encode_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), ExecutableErrorV1> {
    let count = u16::try_from(count).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
    bytes.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn encode_values(bytes: &mut Vec<u8>, values: &[ExecutableValueV1]) -> Result<(), ExecutableErrorV1> {
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

fn encode_expression(bytes: &mut Vec<u8>, expression: &ExecutableExpressionV1) -> Result<(), ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    match expression {
        E::Constant(value) => { bytes.push(0); encode_value(bytes, *value); }
        E::Slot(slot) => { bytes.push(1); bytes.extend_from_slice(&slot.to_le_bytes()); }
        E::Argument(argument) => { bytes.push(2); bytes.extend_from_slice(&argument.to_le_bytes()); }
        E::Add(a, b) => encode_binary(bytes, 3, a, b)?,
        E::Subtract(a, b) => encode_binary(bytes, 4, a, b)?,
        E::Multiply(a, b) => encode_binary(bytes, 5, a, b)?,
        E::Divide(a, b) => encode_binary(bytes, 6, a, b)?,
        E::Clamp(a, b, c) => { bytes.push(7); encode_expression(bytes, a)?; encode_expression(bytes, b)?; encode_expression(bytes, c)?; }
        E::GreaterThan(a, b) => encode_binary(bytes, 8, a, b)?,
        E::LessThanOrEqual(a, b) => encode_binary(bytes, 9, a, b)?,
        E::Equal(a, b) => encode_binary(bytes, 10, a, b)?,
        E::And(a, b) => encode_binary(bytes, 11, a, b)?,
        E::Not(value) => { bytes.push(12); encode_expression(bytes, value)?; }
    }
    Ok(())
}

fn encode_binary(bytes: &mut Vec<u8>, tag: u8, left: &ExecutableExpressionV1, right: &ExecutableExpressionV1) -> Result<(), ExecutableErrorV1> {
    bytes.push(tag);
    encode_expression(bytes, left)?;
    encode_expression(bytes, right)
}

struct Decoder<'a> { bytes: &'a [u8], offset: usize }

impl<'a> Decoder<'a> {
    const fn new(bytes: &'a [u8]) -> Self { Self { bytes, offset: 0 } }
    fn take(&mut self, count: usize) -> Result<&'a [u8], ExecutableErrorV1> {
        let end = self.offset.checked_add(count).ok_or(ExecutableErrorV1::MalformedProgram)?;
        let value = self.bytes.get(self.offset..end).ok_or(ExecutableErrorV1::MalformedProgram)?;
        self.offset = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, ExecutableErrorV1> { Ok(self.take(1)?[0]) }
    fn u16(&mut self) -> Result<u16, ExecutableErrorV1> { Ok(u16::from_le_bytes(self.take(2)?.try_into().expect("two bytes"))) }
    fn u64(&mut self) -> Result<u64, ExecutableErrorV1> { Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("eight bytes"))) }
    fn count(&mut self) -> Result<usize, ExecutableErrorV1> { Ok(usize::from(self.u16()?)) }
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
            1 => match self.byte()? { 0 => Ok(ExecutableValueV1::Boolean(false)), 1 => Ok(ExecutableValueV1::Boolean(true)), _ => Err(ExecutableErrorV1::MalformedProgram) },
            _ => Err(ExecutableErrorV1::MalformedProgram),
        }
    }
    fn expression(&mut self, depth: usize) -> Result<ExecutableExpressionV1, ExecutableErrorV1> {
        if depth >= MAX_EXPRESSION_DEPTH { return Err(ExecutableErrorV1::ResourceLimit); }
        use ExecutableExpressionV1 as E;
        let next = depth + 1;
        Ok(match self.byte()? {
            0 => E::Constant(self.value()?),
            1 => E::Slot(self.u16()?),
            2 => E::Argument(self.u16()?),
            3 => E::Add(Box::new(self.expression(next)?), Box::new(self.expression(next)?)),
            4 => E::Subtract(Box::new(self.expression(next)?), Box::new(self.expression(next)?)),
            5 => E::Multiply(Box::new(self.expression(next)?), Box::new(self.expression(next)?)),
            6 => E::Divide(Box::new(self.expression(next)?), Box::new(self.expression(next)?)),
            7 => E::Clamp(Box::new(self.expression(next)?), Box::new(self.expression(next)?), Box::new(self.expression(next)?)),
            8 => E::GreaterThan(Box::new(self.expression(next)?), Box::new(self.expression(next)?)),
            9 => E::LessThanOrEqual(Box::new(self.expression(next)?), Box::new(self.expression(next)?)),
            10 => E::Equal(Box::new(self.expression(next)?), Box::new(self.expression(next)?)),
            11 => E::And(Box::new(self.expression(next)?), Box::new(self.expression(next)?)),
            12 => E::Not(Box::new(self.expression(next)?)),
            _ => return Err(ExecutableErrorV1::MalformedProgram),
        })
    }
    fn is_complete(&self) -> bool { self.offset == self.bytes.len() }
}
