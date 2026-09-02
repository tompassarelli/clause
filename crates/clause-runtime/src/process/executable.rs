use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::{Mutex, OnceLock};

use clause_package::*;
use sha2::{Digest, Sha256};

use super::ProcessRuntime;

const PHYSICAL_PLAN_MAGIC_V1: &[u8; 4] = b"CPP1";
const ALLOCATION_EPOCH_MAGIC_V1: &[u8; 4] = b"RAE1";
const CONFIGURATION_KIND: &[u8] = b"clause/process-configuration-v1";
const OCCURRENCE_KIND: &[u8] = b"clause/process-occurrence-v1";
const OCCURRENCE_MAGIC: &[u8; 4] = b"CXO1";
const PROJECTION_ROLE_KIND: &[u8] = b"clause/process-projection-role-v1";
const PROJECTED_NUMBER_KIND: &[u8] = b"clause/process-projected-f64-v1";
const PROJECTED_BOOLEAN_KIND: &[u8] = b"clause/process-projected-bool-v1";
const PROJECTED_SYMBOL_KIND: &[u8] = b"clause/process-projected-symbol-v1";
const PROJECTED_SET_KIND: &[u8] = b"clause/process-projected-set-v1";
const PROJECTED_SET_END_KIND: &[u8] = b"clause/process-projected-set-end-v1";
const MAX_EXECUTABLE_SYMBOL_BYTES: usize = 64;
const MAX_PROGRAM_ITEMS: usize = 65_536;
const MAX_EXPRESSION_DEPTH: usize = 64;
const MAX_INPUT_CODE_BYTES: usize = 64;

static ALLOCATED_RUNTIME_ROOTS_V1: OnceLock<Mutex<BTreeSet<[u8; IDENTITY_BYTES]>>> =
    OnceLock::new();

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
    IssuedAdmissionAuthorization = 140,
    Continuation = 150,
    Resumption = 151,
    EffectIntent = 160,
    EffectAuthorization = 161,
    EffectAttempt = 162,
    EffectReceipt = 163,
    EffectJudgment = 164,
    EffectObservation = 165,
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
    next_admission_authorization: u64,
    next_continuation: u64,
    next_resumption: u64,
    next_effect_intent: u64,
    next_effect_authorization: u64,
    next_effect_attempt: u64,
    next_effect_receipt: u64,
    next_effect_judgment: u64,
    next_effect_observation: u64,
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
            next_admission_authorization: 0,
            next_continuation: 0,
            next_resumption: 0,
            next_effect_intent: 0,
            next_effect_authorization: 0,
            next_effect_attempt: 0,
            next_effect_receipt: 0,
            next_effect_judgment: 0,
            next_effect_observation: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ExecutableValueV1 {
    Number(u64),
    Boolean(bool),
    Symbol(ExecutableSymbolV1),
    Set(ExecutableSetV1),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutableSymbolV1 {
    length: u8,
    bytes: [u8; MAX_EXECUTABLE_SYMBOL_BYTES],
}

impl ExecutableSymbolV1 {
    pub fn new(value: &[u8]) -> Result<Self, ExecutableErrorV1> {
        let length = u8::try_from(value.len()).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
        if value.is_empty()
            || value.len() > MAX_EXECUTABLE_SYMBOL_BYTES
            || !value.iter().all(u8::is_ascii_graphic)
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let mut bytes = [0; MAX_EXECUTABLE_SYMBOL_BYTES];
        bytes[..value.len()].copy_from_slice(value);
        Ok(Self { length, bytes })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..usize::from(self.length)]
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ExecutableValueKindV1 {
    Number = 0,
    Boolean = 1,
    Symbol = 2,
    NumberSet = 3,
    BooleanSet = 4,
    SymbolSet = 5,
}

impl ExecutableValueKindV1 {
    const fn set_kind(self) -> Option<Self> {
        match self {
            Self::Number => Some(Self::NumberSet),
            Self::Boolean => Some(Self::BooleanSet),
            Self::Symbol => Some(Self::SymbolSet),
            Self::NumberSet | Self::BooleanSet | Self::SymbolSet => None,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutableSetV1 {
    element_kind: ExecutableValueKindV1,
    values: BTreeSet<ExecutableValueV1>,
}

impl ExecutableSetV1 {
    fn empty(element_kind: ExecutableValueKindV1) -> Result<Self, ExecutableErrorV1> {
        element_kind
            .set_kind()
            .ok_or(ExecutableErrorV1::TypeMismatch)?;
        Ok(Self {
            element_kind,
            values: BTreeSet::new(),
        })
    }

    fn inserted(&self, value: ExecutableValueV1) -> Result<Self, ExecutableErrorV1> {
        if value.kind() != self.element_kind {
            return Err(ExecutableErrorV1::TypeMismatch);
        }
        let mut next = self.clone();
        next.values.insert(value);
        Ok(next)
    }

    fn contains(&self, value: &ExecutableValueV1) -> Result<bool, ExecutableErrorV1> {
        if value.kind() != self.element_kind {
            return Err(ExecutableErrorV1::TypeMismatch);
        }
        Ok(self.values.contains(value))
    }

    fn removed(&self, value: &ExecutableValueV1) -> Result<Self, ExecutableErrorV1> {
        if value.kind() != self.element_kind {
            return Err(ExecutableErrorV1::TypeMismatch);
        }
        let mut next = self.clone();
        next.values.remove(value);
        Ok(next)
    }
}

/// One fixed semantic state coordinate whose relation fact may be absent.
/// Absence belongs to the configuration structure, never to the scalar value
/// domain, so no domain value can be mistaken for a missing fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutableSlotV1 {
    Absent(ExecutableValueKindV1),
    Present(ExecutableValueV1),
}

impl ExecutableSlotV1 {
    #[must_use]
    pub fn kind(&self) -> ExecutableValueKindV1 {
        match self {
            Self::Absent(kind) => *kind,
            Self::Present(value) => value.kind(),
        }
    }

    #[must_use]
    pub const fn value(&self) -> Option<&ExecutableValueV1> {
        match self {
            Self::Absent(_) => None,
            Self::Present(value) => Some(value),
        }
    }

    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        self.value().and_then(ExecutableValueV1::as_number)
    }

    #[must_use]
    pub const fn as_boolean(&self) -> Option<bool> {
        match self.value() {
            Some(value) => value.as_boolean(),
            None => None,
        }
    }
}

impl From<ExecutableValueV1> for ExecutableSlotV1 {
    fn from(value: ExecutableValueV1) -> Self {
        Self::Present(value)
    }
}

impl PartialEq<ExecutableValueV1> for ExecutableSlotV1 {
    fn eq(&self, other: &ExecutableValueV1) -> bool {
        matches!(self, Self::Present(value) if value == other)
    }
}

impl PartialEq<ExecutableSlotV1> for ExecutableValueV1 {
    fn eq(&self, other: &ExecutableSlotV1) -> bool {
        other == self
    }
}

impl ExecutableValueV1 {
    #[must_use]
    pub const fn kind(&self) -> ExecutableValueKindV1 {
        match self {
            Self::Number(_) => ExecutableValueKindV1::Number,
            Self::Boolean(_) => ExecutableValueKindV1::Boolean,
            Self::Symbol(_) => ExecutableValueKindV1::Symbol,
            Self::Set(set) => match set.element_kind {
                ExecutableValueKindV1::Number => ExecutableValueKindV1::NumberSet,
                ExecutableValueKindV1::Boolean => ExecutableValueKindV1::BooleanSet,
                ExecutableValueKindV1::Symbol => ExecutableValueKindV1::SymbolSet,
                ExecutableValueKindV1::NumberSet
                | ExecutableValueKindV1::BooleanSet
                | ExecutableValueKindV1::SymbolSet => unreachable!(),
            },
        }
    }

    fn empty_set(element_kind: ExecutableValueKindV1) -> Result<Self, ExecutableErrorV1> {
        ExecutableSetV1::empty(element_kind).map(Self::Set)
    }
}

/// One checked semantic-role to physical-configuration refinement.
///
/// The slot is package-owned materialization data. Consumers select projected
/// values by Role identity; neither the Wasm boundary nor browser may infer
/// meaning from this physical index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableProjectionBindingV1 {
    pub role: LocalRoleRefV2,
    pub slot: u16,
    pub value_kind: ExecutableValueKindV1,
}

/// One package-declared derived-Observation shape. Role placeholder Atoms in
/// `template` are replaced by exact typed values only after Admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableProjectionV1 {
    pub bindings: Vec<ExecutableProjectionBindingV1>,
    pub template: Term,
}

/// One construct-blind physical input distinction. The browser reports this
/// shape; only the checked physical plan relates it to a Clause Role and an
/// executable occurrence.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ExecutableInputSourceV1 {
    Keyboard {
        code: Vec<u8>,
        phase: ExecutableKeyPhaseV1,
    },
    Scalar {
        channel: Vec<u8>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ExecutableKeyPhaseV1 {
    Down = 0,
    Up = 1,
}

/// A package-role-indexed realization of one physical input distinction.
/// `occurrence` is a generic rule-machine occurrence, not a host callback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableInputBindingV1 {
    pub role: LocalRoleRefV2,
    pub source: ExecutableInputSourceV1,
    pub occurrence: ExecutableOccurrenceV1,
}

/// The fixed tick is a separate package Role and ordered executable entry
/// chain. Every entry receives the exact tick duration in seconds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableTickBindingV1 {
    pub role: LocalRoleRefV2,
    pub entries: Vec<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableInputPlanV1 {
    pub events: Vec<ExecutableInputBindingV1>,
    pub tick: ExecutableTickBindingV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableInputObservationV1 {
    pub sequence: u64,
    pub source: ExecutableInputSourceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableInputConfigurationV1 {
    pub revision: u64,
    pub fixed_tick_milliseconds: u32,
    pub observations: Vec<ExecutableInputObservationV1>,
}

/// Construct a typed role leaf for an executable projection template.
pub fn executable_projection_role_term_v1(
    scope: TermScope,
    role: LocalRoleRefV2,
    value_kind: ExecutableValueKindV1,
) -> Result<Term, ExecutableErrorV1> {
    let mut payload = Vec::with_capacity(9);
    payload.extend_from_slice(&role.schema.get().to_le_bytes());
    payload.extend_from_slice(&role.role.get().to_le_bytes());
    payload.push(value_kind as u8);
    Term::atom(
        scope,
        PROJECTION_ROLE_KIND.to_vec(),
        payload,
        EqualityContract::ExactOctetsV1,
    )
    .map_err(|_| ExecutableErrorV1::MalformedProgram)
}

impl ExecutableValueV1 {
    pub fn number(value: f64) -> Result<Self, ExecutableErrorV1> {
        if !value.is_finite() {
            return Err(ExecutableErrorV1::NumericDomain);
        }
        Ok(Self::Number(canonical_number_bits(value)))
    }

    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(bits) => Some(f64::from_bits(*bits)),
            Self::Boolean(_) | Self::Symbol(_) | Self::Set(_) => None,
        }
    }

    #[must_use]
    pub const fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            Self::Number(_) | Self::Symbol(_) | Self::Set(_) => None,
        }
    }

    pub fn symbol(value: &[u8]) -> Result<Self, ExecutableErrorV1> {
        ExecutableSymbolV1::new(value).map(Self::Symbol)
    }

    #[must_use]
    pub fn as_symbol(&self) -> Option<&[u8]> {
        match self {
            Self::Symbol(value) => Some(value.as_bytes()),
            Self::Number(_) | Self::Boolean(_) | Self::Set(_) => None,
        }
    }
}

pub fn executable_configuration_term_v1(
    scope: clause_package::TermScope,
    values: &[ExecutableSlotV1],
) -> Result<Term, ExecutableErrorV1> {
    let mut bytes = Vec::new();
    encode_slots(&mut bytes, values)?;
    Term::atom(
        scope,
        CONFIGURATION_KIND.to_vec(),
        bytes,
        EqualityContract::ExactOctetsV1,
    )
    .map_err(|_| ExecutableErrorV1::MalformedProgram)
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
    SetInsert(Box<Self>, Box<Self>),
    SetContains(Box<Self>, Box<Self>),
    SetRemove(Box<Self>, Box<Self>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableRuleV1 {
    pub entry: u16,
    pub predicates: Vec<ExecutableExpressionV1>,
    pub required_present: Vec<u16>,
    pub required_absent: Vec<u16>,
    pub assignments: Vec<(u16, ExecutableExpressionV1)>,
    pub removals: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableProgramV1 {
    pub initial_configuration: Vec<ExecutableValueV1>,
    pub rules: Vec<ExecutableRuleV1>,
    pub projection: Option<ExecutableProjectionV1>,
}

/// Exact checked source-state to physical-coordinate refinement selected by
/// the generic planner. The semantic state reference remains available for
/// identity, diagnostics, and projection; `slot` is only physical data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableCanonicalStateBindingV1 {
    pub state: CanonicalStateRefV1,
    pub projection_role: LocalRoleRefV2,
    pub slot: u16,
}

/// Exact checked handler to physical-entry refinement selected without host
/// mechanic names or source traversal ordinals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableCanonicalHandlerBindingV1 {
    pub handler: FormationLocalId,
    pub trigger: CanonicalHandlerTriggerV1,
    pub argument_count: u16,
    pub entry: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableCanonicalProgramV1 {
    pub program: ExecutableProgramV1,
    pub states: Vec<ExecutableCanonicalStateBindingV1>,
    pub handlers: Vec<ExecutableCanonicalHandlerBindingV1>,
}

/// Refine the checked construct-blind source IR into one portable scalar
/// program. Slot and entry ordinals are allocated only after sorting exact
/// semantic state/handler identities. Source order, mechanic names, and host
/// callback registries do not participate.
pub fn lower_canonical_executable_program_v1(
    scope: TermScope,
    state_cells: &[CanonicalStateCellV1],
    handlers: &[CanonicalExecutableHandlerV1],
    projection_roles: &[LocalRoleRefV2],
) -> Result<ExecutableCanonicalProgramV1, ExecutableErrorV1> {
    if state_cells.is_empty()
        || handlers.is_empty()
        || state_cells.len() > MAX_PROGRAM_ITEMS
        || handlers.len() > MAX_PROGRAM_ITEMS
        || projection_roles.len() < state_cells.len()
    {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let mut seen_states = BTreeSet::new();
    if state_cells
        .iter()
        .any(|cell| !seen_states.insert(cell.state.clone()))
    {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let mut ordered_states = state_cells.to_vec();
    ordered_states.sort_by(|left, right| {
        canonical_cell_initially_present(right)
            .cmp(&canonical_cell_initially_present(left))
            .then_with(|| left.state.cmp(&right.state))
    });
    let mut roles = projection_roles.to_vec();
    roles.sort();
    roles.dedup();
    if roles.len() < ordered_states.len() {
        return Err(ExecutableErrorV1::MalformedProgram);
    }

    let mut slots = BTreeMap::new();
    let mut state_bindings = Vec::with_capacity(ordered_states.len());
    let mut initial_configuration = Vec::with_capacity(ordered_states.len());
    let mut roles = roles.into_iter();
    for (ordinal, cell) in ordered_states.iter().enumerate() {
        let slot = u16::try_from(ordinal).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
        if slots.insert(cell.state.clone(), slot).is_some() {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        if let Some(value) = &cell.initial_value {
            initial_configuration.push(lower_scalar_value(value)?);
        } else if matches!(cell.state.path, CanonicalStatePathV1::Many) {
            initial_configuration.push(ExecutableValueV1::empty_set(lower_scalar_value_kind(
                cell.value_kind,
            ))?);
        }
        state_bindings.push(ExecutableCanonicalStateBindingV1 {
            state: cell.state.clone(),
            projection_role: roles.next().ok_or(ExecutableErrorV1::MalformedProgram)?,
            slot,
        });
    }

    let mut ordered_handlers = handlers.to_vec();
    ordered_handlers.sort_by_key(|handler| handler.id);
    if ordered_handlers
        .windows(2)
        .any(|pair| pair[0].id == pair[1].id)
    {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let mut rules = Vec::new();
    let mut handler_bindings = Vec::with_capacity(ordered_handlers.len());
    for (ordinal, handler) in ordered_handlers.iter().enumerate() {
        let entry = u16::try_from(ordinal).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
        handler_bindings.push(ExecutableCanonicalHandlerBindingV1 {
            handler: handler.id,
            trigger: handler.trigger,
            argument_count: handler.argument_count,
            entry,
        });
        for source_rule in &handler.rules {
            let predicates = source_rule
                .predicates
                .iter()
                .map(|predicate| lower_canonical_predicate(predicate, &slots))
                .collect::<Result<Vec<_>, _>>()?;
            let assignments = source_rule
                .assignments
                .iter()
                .map(|assignment| {
                    Ok((
                        *slots
                            .get(&assignment.target)
                            .ok_or(ExecutableErrorV1::MalformedProgram)?,
                        lower_canonical_expression(&assignment.value, &slots, 0)?,
                    ))
                })
                .collect::<Result<Vec<_>, ExecutableErrorV1>>()?;
            rules.push(ExecutableRuleV1 {
                entry,
                predicates,
                required_present: source_rule
                    .required_present
                    .iter()
                    .map(|state| {
                        slots
                            .get(state)
                            .copied()
                            .ok_or(ExecutableErrorV1::MalformedProgram)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                required_absent: source_rule
                    .required_absent
                    .iter()
                    .map(|state| {
                        slots
                            .get(state)
                            .copied()
                            .ok_or(ExecutableErrorV1::MalformedProgram)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                assignments,
                removals: source_rule
                    .removals
                    .iter()
                    .map(|state| {
                        slots
                            .get(state)
                            .copied()
                            .ok_or(ExecutableErrorV1::MalformedProgram)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            });
        }
    }
    let projection = canonical_source_projection(scope, &ordered_states, &state_bindings)?;
    let program = ExecutableProgramV1 {
        initial_configuration,
        rules,
        projection: Some(projection),
    };
    validate_program(&program)?;
    Ok(ExecutableCanonicalProgramV1 {
        program,
        states: state_bindings,
        handlers: handler_bindings,
    })
}

fn canonical_cell_initially_present(cell: &CanonicalStateCellV1) -> bool {
    cell.initial_value.is_some() || matches!(cell.state.path, CanonicalStatePathV1::Many)
}

fn lower_canonical_expression(
    expression: &CanonicalExecutableExpressionV1,
    slots: &BTreeMap<CanonicalStateRefV1, u16>,
    depth: usize,
) -> Result<ExecutableExpressionV1, ExecutableErrorV1> {
    if depth >= MAX_EXPRESSION_DEPTH {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let pair = |left: &CanonicalExecutableExpressionV1,
                right: &CanonicalExecutableExpressionV1|
     -> Result<_, ExecutableErrorV1> {
        Ok((
            Box::new(lower_canonical_expression(left, slots, depth + 1)?),
            Box::new(lower_canonical_expression(right, slots, depth + 1)?),
        ))
    };
    Ok(match expression {
        CanonicalExecutableExpressionV1::Constant(value) => {
            ExecutableExpressionV1::Constant(lower_scalar_value(value)?)
        }
        CanonicalExecutableExpressionV1::State(state) => ExecutableExpressionV1::Slot(
            *slots
                .get(state)
                .ok_or(ExecutableErrorV1::MalformedProgram)?,
        ),
        CanonicalExecutableExpressionV1::Argument(argument) => {
            ExecutableExpressionV1::Argument(*argument)
        }
        CanonicalExecutableExpressionV1::Add(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::Add(left, right)
        }
        CanonicalExecutableExpressionV1::Subtract(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::Subtract(left, right)
        }
        CanonicalExecutableExpressionV1::Multiply(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::Multiply(left, right)
        }
        CanonicalExecutableExpressionV1::Divide(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::Divide(left, right)
        }
        CanonicalExecutableExpressionV1::Clamp(value, lower, upper) => {
            ExecutableExpressionV1::Clamp(
                Box::new(lower_canonical_expression(value, slots, depth + 1)?),
                Box::new(lower_canonical_expression(lower, slots, depth + 1)?),
                Box::new(lower_canonical_expression(upper, slots, depth + 1)?),
            )
        }
        CanonicalExecutableExpressionV1::Insert(set, value) => ExecutableExpressionV1::SetInsert(
            Box::new(lower_canonical_expression(set, slots, depth + 1)?),
            Box::new(lower_canonical_expression(value, slots, depth + 1)?),
        ),
        CanonicalExecutableExpressionV1::Remove(set, value) => ExecutableExpressionV1::SetRemove(
            Box::new(lower_canonical_expression(set, slots, depth + 1)?),
            Box::new(lower_canonical_expression(value, slots, depth + 1)?),
        ),
    })
}

fn lower_canonical_predicate(
    predicate: &CanonicalExecutablePredicateV1,
    slots: &BTreeMap<CanonicalStateRefV1, u16>,
) -> Result<ExecutableExpressionV1, ExecutableErrorV1> {
    let pair = |left, right| {
        Ok::<_, ExecutableErrorV1>((
            Box::new(lower_canonical_expression(left, slots, 0)?),
            Box::new(lower_canonical_expression(right, slots, 0)?),
        ))
    };
    Ok(match predicate {
        CanonicalExecutablePredicateV1::Equal(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::Equal(left, right)
        }
        CanonicalExecutablePredicateV1::GreaterThan(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::GreaterThan(left, right)
        }
        CanonicalExecutablePredicateV1::LessThanOrEqual(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::LessThanOrEqual(left, right)
        }
        CanonicalExecutablePredicateV1::Contains(set, value) => {
            let (set, value) = pair(set, value)?;
            ExecutableExpressionV1::SetContains(set, value)
        }
    })
}

fn projection_literal(
    scope: TermScope,
    kind: &[u8],
    payload: &[u8],
) -> Result<Term, ExecutableErrorV1> {
    Term::atom(
        scope,
        kind.to_vec(),
        payload.to_vec(),
        EqualityContract::ExactOctetsV1,
    )
    .map_err(|_| ExecutableErrorV1::MalformedProgram)
}

fn projection_object(
    scope: TermScope,
    fields: Vec<(Vec<u8>, Term)>,
) -> Result<Term, ExecutableErrorV1> {
    let mut rest = projection_literal(scope, b"clause/js-object-end-v1", &[])?;
    for (field, value) in fields.into_iter().rev() {
        rest = Term::triple([
            projection_literal(scope, b"clause/js-field-v1", &field)?,
            value,
            rest,
        ])
        .map_err(|_| ExecutableErrorV1::MalformedProgram)?;
    }
    Ok(rest)
}

fn canonical_source_projection(
    scope: TermScope,
    cells: &[CanonicalStateCellV1],
    bindings: &[ExecutableCanonicalStateBindingV1],
) -> Result<ExecutableProjectionV1, ExecutableErrorV1> {
    let by_state = bindings
        .iter()
        .map(|binding| (binding.state.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut subjects = BTreeMap::<Vec<u8>, BTreeMap<Vec<u8>, Vec<&CanonicalStateCellV1>>>::new();
    for cell in cells {
        subjects
            .entry(cell.state.subject.clone())
            .or_default()
            .entry(cell.state.relation_designation.clone())
            .or_default()
            .push(cell);
    }
    let mut subject_fields = Vec::with_capacity(subjects.len());
    for (subject, relations) in subjects {
        let mut relation_fields = Vec::with_capacity(relations.len());
        for (relation, mut cells) in relations {
            cells.sort_by(|left, right| left.state.path.cmp(&right.state.path));
            let value = if cells.len() == 1
                && matches!(
                    cells[0].state.path,
                    CanonicalStatePathV1::Scalar | CanonicalStatePathV1::Many
                ) {
                let binding = by_state
                    .get(&cells[0].state)
                    .ok_or(ExecutableErrorV1::MalformedProgram)?;
                let value_kind = lower_scalar_value_kind(cells[0].value_kind);
                executable_projection_role_term_v1(
                    scope,
                    binding.projection_role,
                    if matches!(cells[0].state.path, CanonicalStatePathV1::Many) {
                        value_kind
                            .set_kind()
                            .ok_or(ExecutableErrorV1::MalformedProgram)?
                    } else {
                        value_kind
                    },
                )?
            } else {
                let fields = cells
                    .into_iter()
                    .map(|cell| {
                        let CanonicalStatePathV1::Field { designation, .. } = &cell.state.path
                        else {
                            return Err(ExecutableErrorV1::MalformedProgram);
                        };
                        let binding = by_state
                            .get(&cell.state)
                            .ok_or(ExecutableErrorV1::MalformedProgram)?;
                        Ok((
                            designation.clone(),
                            executable_projection_role_term_v1(
                                scope,
                                binding.projection_role,
                                lower_scalar_value_kind(cell.value_kind),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, ExecutableErrorV1>>()?;
                projection_object(scope, fields)?
            };
            relation_fields.push((relation, value));
        }
        subject_fields.push((subject, projection_object(scope, relation_fields)?));
    }
    Ok(ExecutableProjectionV1 {
        bindings: cells
            .iter()
            .map(|cell| {
                let binding = by_state
                    .get(&cell.state)
                    .ok_or(ExecutableErrorV1::MalformedProgram)?;
                let value_kind = lower_scalar_value_kind(cell.value_kind);
                Ok(ExecutableProjectionBindingV1 {
                    role: binding.projection_role,
                    slot: binding.slot,
                    value_kind: if matches!(cell.state.path, CanonicalStatePathV1::Many) {
                        value_kind
                            .set_kind()
                            .ok_or(ExecutableErrorV1::MalformedProgram)?
                    } else {
                        value_kind
                    },
                })
            })
            .collect::<Result<Vec<_>, ExecutableErrorV1>>()?,
        template: projection_object(scope, subject_fields)?,
    })
}

/// Physical slot realization for the source-owned X/Z input result. Event
/// arguments remain ordered by the source handler's declared Vec3 fields;
/// this record supplies only the target-specific configuration coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableCanonicalInputBindingV1 {
    pub entry: u16,
    pub x_slot: u16,
    pub z_slot: u16,
}

/// Refine the package-owned bounded `on input` meaning into CPP1. The source
/// owns both initial values and result expressions; Rust supplies only the
/// physical entry and slots. Existing rules at that entry are rejected so a
/// host-authored semantic implementation cannot silently remain in force.
pub fn lower_canonical_input_handler_v1(
    program: &mut ExecutableProgramV1,
    source: &CanonicalInputHandlerV1,
    binding: ExecutableCanonicalInputBindingV1,
) -> Result<(), ExecutableErrorV1> {
    if program.rules.iter().any(|rule| rule.entry == binding.entry) {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let x = usize::from(binding.x_slot);
    let z = usize::from(binding.z_slot);
    let Some(initial_x) = program.initial_configuration.get_mut(x) else {
        return Err(ExecutableErrorV1::MalformedProgram);
    };
    *initial_x = ExecutableValueV1::Number(source.initial_x);
    let Some(initial_z) = program.initial_configuration.get_mut(z) else {
        return Err(ExecutableErrorV1::MalformedProgram);
    };
    *initial_z = ExecutableValueV1::Number(source.initial_z);

    let expression = |value| match value {
        CanonicalInputScalarV1::Parameter(index) => ExecutableExpressionV1::Argument(index),
        CanonicalInputScalarV1::Number(bits) => {
            ExecutableExpressionV1::Constant(ExecutableValueV1::Number(bits))
        }
    };
    let rule = ExecutableRuleV1 {
        entry: binding.entry,
        predicates: vec![],
        required_present: vec![],
        required_absent: vec![],
        assignments: vec![
            (binding.x_slot, expression(source.result_x)),
            (binding.z_slot, expression(source.result_z)),
        ],
        removals: vec![],
    };
    let insertion = program
        .rules
        .iter()
        .position(|existing| existing.entry > rule.entry)
        .unwrap_or(program.rules.len());
    program.rules.insert(insertion, rule);
    Ok(())
}

/// Physical coordinates for the source-owned `on jump` result and its three
/// prerequisite assertion values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableCanonicalJumpBindingV1 {
    pub entry: u16,
    pub velocity_slots: [u16; 3],
    pub grounded_slot: u16,
    pub jump_speed_slot: u16,
}

/// Refine the package-owned bounded `on jump` meaning into CPP1. Rust selects
/// physical coordinates only; source owns initial values, predicate, and
/// included results. Any preexisting rule at the entry fails closed.
pub fn lower_canonical_jump_handler_v1(
    program: &mut ExecutableProgramV1,
    source: &CanonicalJumpHandlerV1,
    binding: ExecutableCanonicalJumpBindingV1,
) -> Result<(), ExecutableErrorV1> {
    if program.rules.iter().any(|rule| rule.entry == binding.entry) {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let distinct_slots = binding
        .velocity_slots
        .into_iter()
        .chain([binding.grounded_slot, binding.jump_speed_slot])
        .collect::<BTreeSet<_>>();
    if distinct_slots.len() != 5 {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    for (slot, value) in binding
        .velocity_slots
        .into_iter()
        .zip(source.initial_velocity)
    {
        let Some(target) = program.initial_configuration.get_mut(usize::from(slot)) else {
            return Err(ExecutableErrorV1::MalformedProgram);
        };
        *target = ExecutableValueV1::Number(value);
    }
    let Some(grounded) = program
        .initial_configuration
        .get_mut(usize::from(binding.grounded_slot))
    else {
        return Err(ExecutableErrorV1::MalformedProgram);
    };
    *grounded = ExecutableValueV1::Boolean(source.initial_grounded);
    let Some(jump_speed) = program
        .initial_configuration
        .get_mut(usize::from(binding.jump_speed_slot))
    else {
        return Err(ExecutableErrorV1::MalformedProgram);
    };
    *jump_speed = ExecutableValueV1::Number(source.jump_speed);

    let expression = |value| match value {
        CanonicalJumpScalarV1::VelocityComponent(index) => binding
            .velocity_slots
            .get(usize::from(index))
            .copied()
            .map(ExecutableExpressionV1::Slot),
        CanonicalJumpScalarV1::JumpSpeed => {
            Some(ExecutableExpressionV1::Slot(binding.jump_speed_slot))
        }
        CanonicalJumpScalarV1::Number(bits) => Some(ExecutableExpressionV1::Constant(
            ExecutableValueV1::Number(bits),
        )),
    };
    let mut assignments = Vec::with_capacity(4);
    for (slot, value) in binding
        .velocity_slots
        .into_iter()
        .zip(source.result_velocity)
    {
        assignments.push((
            slot,
            expression(value).ok_or(ExecutableErrorV1::MalformedProgram)?,
        ));
    }
    assignments.push((
        binding.grounded_slot,
        ExecutableExpressionV1::Constant(ExecutableValueV1::Boolean(source.result_grounded)),
    ));
    let rule = ExecutableRuleV1 {
        entry: binding.entry,
        predicates: vec![ExecutableExpressionV1::Equal(
            Box::new(ExecutableExpressionV1::Slot(binding.grounded_slot)),
            Box::new(ExecutableExpressionV1::Constant(
                ExecutableValueV1::Boolean(source.required_grounded),
            )),
        )],
        required_present: vec![],
        required_absent: vec![],
        assignments,
        removals: vec![],
    };
    let insertion = program
        .rules
        .iter()
        .position(|existing| existing.entry > rule.entry)
        .unwrap_or(program.rules.len());
    program.rules.insert(insertion, rule);
    Ok(())
}

/// Physical coordinates for one construct-blind scalar source transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableCanonicalScalarParameterBindingV1 {
    pub parameter: Vec<u8>,
    pub slot: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableCanonicalScalarBindingV1 {
    pub entry: u16,
    pub state_slot: u16,
    pub parameters: Vec<ExecutableCanonicalScalarParameterBindingV1>,
}

/// Refine a checked one-cell numeric source transition into CPP1. Rust sees
/// only a generic expression and physical coordinates; event and relation
/// designations select no host branch.
pub fn lower_canonical_scalar_handler_v1(
    program: &mut ExecutableProgramV1,
    source: &CanonicalScalarHandlerV1,
    binding: ExecutableCanonicalScalarBindingV1,
) -> Result<(), ExecutableErrorV1> {
    if program.rules.iter().any(|rule| rule.entry == binding.entry) {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let Some(initial) = program
        .initial_configuration
        .get_mut(usize::from(binding.state_slot))
    else {
        return Err(ExecutableErrorV1::MalformedProgram);
    };
    *initial = lower_scalar_value(&source.initial_value)?;
    let source_parameters = source.parameters.iter().cloned().collect::<BTreeSet<_>>();
    if source_parameters.len() != source.parameters.len()
        || binding.parameters.len() != source.parameters.len()
    {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let mut parameter_slots = BTreeMap::new();
    for parameter in &binding.parameters {
        if !source_parameters.contains(&parameter.parameter)
            || program
                .initial_configuration
                .get(usize::from(parameter.slot))
                .is_none()
            || parameter_slots
                .insert(parameter.parameter.clone(), parameter.slot)
                .is_some()
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
    if parameter_slots.len() != source_parameters.len() {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let assignment =
        lower_scalar_expression(&source.result, binding.state_slot, &parameter_slots, 0)?;
    let predicates = source
        .predicates
        .iter()
        .map(|predicate| {
            let (left, right, constructor) = match predicate {
                CanonicalScalarPredicateV1::Equal(left, right) => (
                    left,
                    right,
                    ExecutableExpressionV1::Equal as fn(_, _) -> ExecutableExpressionV1,
                ),
                CanonicalScalarPredicateV1::GreaterThan(left, right) => (
                    left,
                    right,
                    ExecutableExpressionV1::GreaterThan as fn(_, _) -> ExecutableExpressionV1,
                ),
                CanonicalScalarPredicateV1::LessThanOrEqual(left, right) => (
                    left,
                    right,
                    ExecutableExpressionV1::LessThanOrEqual as fn(_, _) -> ExecutableExpressionV1,
                ),
            };
            Ok(constructor(
                Box::new(lower_scalar_expression(
                    left,
                    binding.state_slot,
                    &parameter_slots,
                    0,
                )?),
                Box::new(lower_scalar_expression(
                    right,
                    binding.state_slot,
                    &parameter_slots,
                    0,
                )?),
            ))
        })
        .collect::<Result<Vec<_>, ExecutableErrorV1>>()?;
    let rule = ExecutableRuleV1 {
        entry: binding.entry,
        predicates,
        required_present: vec![],
        required_absent: vec![],
        assignments: vec![(binding.state_slot, assignment)],
        removals: vec![],
    };
    let insertion = program
        .rules
        .iter()
        .position(|existing| existing.entry > rule.entry)
        .unwrap_or(program.rules.len());
    program.rules.insert(insertion, rule);
    Ok(())
}

fn lower_scalar_expression(
    expression: &CanonicalScalarExpressionV1,
    state_slot: u16,
    parameter_slots: &BTreeMap<Vec<u8>, u16>,
    depth: usize,
) -> Result<ExecutableExpressionV1, ExecutableErrorV1> {
    if depth >= MAX_EXPRESSION_DEPTH {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let pair = |left: &CanonicalScalarExpressionV1,
                right: &CanonicalScalarExpressionV1|
     -> Result<_, ExecutableErrorV1> {
        Ok((
            Box::new(lower_scalar_expression(
                left,
                state_slot,
                parameter_slots,
                depth + 1,
            )?),
            Box::new(lower_scalar_expression(
                right,
                state_slot,
                parameter_slots,
                depth + 1,
            )?),
        ))
    };
    Ok(match expression {
        CanonicalScalarExpressionV1::Current => ExecutableExpressionV1::Slot(state_slot),
        CanonicalScalarExpressionV1::Parameter(parameter) => ExecutableExpressionV1::Slot(
            *parameter_slots
                .get(parameter)
                .ok_or(ExecutableErrorV1::MalformedProgram)?,
        ),
        CanonicalScalarExpressionV1::Number(bits) => {
            ExecutableExpressionV1::Constant(ExecutableValueV1::Number(*bits))
        }
        CanonicalScalarExpressionV1::Boolean(value) => {
            ExecutableExpressionV1::Constant(ExecutableValueV1::Boolean(*value))
        }
        CanonicalScalarExpressionV1::Symbol(value) => {
            ExecutableExpressionV1::Constant(ExecutableValueV1::symbol(value)?)
        }
        CanonicalScalarExpressionV1::Add(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::Add(left, right)
        }
        CanonicalScalarExpressionV1::Subtract(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::Subtract(left, right)
        }
        CanonicalScalarExpressionV1::Multiply(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::Multiply(left, right)
        }
        CanonicalScalarExpressionV1::Divide(left, right) => {
            let (left, right) = pair(left, right)?;
            ExecutableExpressionV1::Divide(left, right)
        }
        CanonicalScalarExpressionV1::Clamp(value, lower, upper) => ExecutableExpressionV1::Clamp(
            Box::new(lower_scalar_expression(
                value,
                state_slot,
                parameter_slots,
                depth + 1,
            )?),
            Box::new(lower_scalar_expression(
                lower,
                state_slot,
                parameter_slots,
                depth + 1,
            )?),
            Box::new(lower_scalar_expression(
                upper,
                state_slot,
                parameter_slots,
                depth + 1,
            )?),
        ),
    })
}

fn lower_scalar_value(
    value: &CanonicalScalarValueV1,
) -> Result<ExecutableValueV1, ExecutableErrorV1> {
    match value {
        CanonicalScalarValueV1::Number(bits) => Ok(ExecutableValueV1::Number(*bits)),
        CanonicalScalarValueV1::Boolean(value) => Ok(ExecutableValueV1::Boolean(*value)),
        CanonicalScalarValueV1::Symbol(value) => ExecutableValueV1::symbol(value),
    }
}

const fn lower_scalar_value_kind(
    kind: clause_package::CanonicalScalarValueKindV1,
) -> ExecutableValueKindV1 {
    match kind {
        clause_package::CanonicalScalarValueKindV1::Number => ExecutableValueKindV1::Number,
        clause_package::CanonicalScalarValueKindV1::Boolean => ExecutableValueKindV1::Boolean,
        clause_package::CanonicalScalarValueKindV1::Symbol => ExecutableValueKindV1::Symbol,
    }
}

/// Physical coordinates for the source-owned three-branch `on tick` program.
/// The binding contains no gameplay constants or expressions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableCanonicalTickBindingV1 {
    pub entry: u16,
    pub delta_time_argument: u16,
    pub position_slots: [u16; 3],
    pub velocity_slots: [u16; 3],
    pub intent_slots: [u16; 3],
    pub grounded_slot: u16,
    pub gravity_slot: u16,
    pub move_speed_slot: u16,
    pub floor_height_slot: u16,
    pub minimum_x_slot: u16,
    pub maximum_x_slot: u16,
    pub minimum_z_slot: u16,
    pub maximum_z_slot: u16,
}

/// Refine the checked source-owned tick slice into CPP1. Rust supplies only
/// generic physical coordinates and scalar primitives; all initial values,
/// predicates, arithmetic, clamp use, and assignments come from source.
pub fn lower_canonical_tick_program_v1(
    program: &mut ExecutableProgramV1,
    source: &CanonicalTickProgramV1,
    binding: ExecutableCanonicalTickBindingV1,
) -> Result<(), ExecutableErrorV1> {
    if program.rules.iter().any(|rule| rule.entry == binding.entry) {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let slots = binding
        .position_slots
        .into_iter()
        .chain(binding.velocity_slots)
        .chain(binding.intent_slots)
        .chain([
            binding.grounded_slot,
            binding.gravity_slot,
            binding.move_speed_slot,
            binding.floor_height_slot,
            binding.minimum_x_slot,
            binding.maximum_x_slot,
            binding.minimum_z_slot,
            binding.maximum_z_slot,
        ])
        .collect::<BTreeSet<_>>();
    if slots.len() != 17 {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let initial = binding
        .position_slots
        .into_iter()
        .zip(source.initial_position)
        .chain(
            binding
                .velocity_slots
                .into_iter()
                .zip(source.initial_velocity),
        )
        .chain(binding.intent_slots.into_iter().zip(source.initial_intent))
        .map(|(slot, bits)| (slot, ExecutableValueV1::Number(bits)))
        .chain([
            (
                binding.grounded_slot,
                ExecutableValueV1::Boolean(source.initial_grounded),
            ),
            (
                binding.gravity_slot,
                ExecutableValueV1::Number(source.gravity),
            ),
            (
                binding.move_speed_slot,
                ExecutableValueV1::Number(source.move_speed),
            ),
            (
                binding.floor_height_slot,
                ExecutableValueV1::Number(source.floor_height),
            ),
            (
                binding.minimum_x_slot,
                ExecutableValueV1::Number(source.minimum_x),
            ),
            (
                binding.maximum_x_slot,
                ExecutableValueV1::Number(source.maximum_x),
            ),
            (
                binding.minimum_z_slot,
                ExecutableValueV1::Number(source.minimum_z),
            ),
            (
                binding.maximum_z_slot,
                ExecutableValueV1::Number(source.maximum_z),
            ),
        ]);
    for (slot, value) in initial {
        let Some(target) = program.initial_configuration.get_mut(usize::from(slot)) else {
            return Err(ExecutableErrorV1::MalformedProgram);
        };
        *target = value;
    }

    let lower_expression =
        |expression: &CanonicalTickExpressionV1| lower_tick_expression(expression, binding, 0);
    let mut rules = Vec::with_capacity(source.rules.len());
    for source_rule in &source.rules {
        let mut predicates = Vec::with_capacity(source_rule.predicates.len());
        for predicate in &source_rule.predicates {
            predicates.push(match predicate {
                CanonicalTickPredicateV1::EqualBoolean(value, expected) => {
                    ExecutableExpressionV1::Equal(
                        Box::new(lower_tick_value(*value, binding)?),
                        Box::new(ExecutableExpressionV1::Constant(
                            ExecutableValueV1::Boolean(*expected),
                        )),
                    )
                }
                CanonicalTickPredicateV1::GreaterThan(left, right) => {
                    ExecutableExpressionV1::GreaterThan(
                        Box::new(lower_expression(left)?),
                        Box::new(lower_expression(right)?),
                    )
                }
                CanonicalTickPredicateV1::LessThanOrEqual(left, right) => {
                    ExecutableExpressionV1::LessThanOrEqual(
                        Box::new(lower_expression(left)?),
                        Box::new(lower_expression(right)?),
                    )
                }
            });
        }
        let mut assignments = Vec::with_capacity(source_rule.assignments.len());
        for assignment in &source_rule.assignments {
            let slot = match assignment.target {
                CanonicalTickAssignmentTargetV1::PositionComponent(index) => {
                    binding.position_slots.get(usize::from(index)).copied()
                }
                CanonicalTickAssignmentTargetV1::VelocityComponent(index) => {
                    binding.velocity_slots.get(usize::from(index)).copied()
                }
                CanonicalTickAssignmentTargetV1::Grounded => Some(binding.grounded_slot),
            }
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
            let value = match &assignment.value {
                CanonicalTickAssignmentValueV1::Number(expression) => lower_expression(expression)?,
                CanonicalTickAssignmentValueV1::Boolean(value) => {
                    ExecutableExpressionV1::Constant(ExecutableValueV1::Boolean(*value))
                }
            };
            assignments.push((slot, value));
        }
        rules.push(ExecutableRuleV1 {
            entry: binding.entry,
            predicates,
            required_present: vec![],
            required_absent: vec![],
            assignments,
            removals: vec![],
        });
    }
    let insertion = program
        .rules
        .iter()
        .position(|existing| existing.entry > binding.entry)
        .unwrap_or(program.rules.len());
    program.rules.splice(insertion..insertion, rules);
    Ok(())
}

fn lower_tick_value(
    value: CanonicalTickValueV1,
    binding: ExecutableCanonicalTickBindingV1,
) -> Result<ExecutableExpressionV1, ExecutableErrorV1> {
    let slot = match value {
        CanonicalTickValueV1::DeltaTime => {
            return Ok(ExecutableExpressionV1::Argument(
                binding.delta_time_argument,
            ));
        }
        CanonicalTickValueV1::PositionComponent(index) => {
            binding.position_slots.get(usize::from(index)).copied()
        }
        CanonicalTickValueV1::VelocityComponent(index) => {
            binding.velocity_slots.get(usize::from(index)).copied()
        }
        CanonicalTickValueV1::IntentComponent(index) => {
            binding.intent_slots.get(usize::from(index)).copied()
        }
        CanonicalTickValueV1::Grounded => Some(binding.grounded_slot),
        CanonicalTickValueV1::Gravity => Some(binding.gravity_slot),
        CanonicalTickValueV1::MoveSpeed => Some(binding.move_speed_slot),
        CanonicalTickValueV1::FloorHeight => Some(binding.floor_height_slot),
        CanonicalTickValueV1::MinimumX => Some(binding.minimum_x_slot),
        CanonicalTickValueV1::MaximumX => Some(binding.maximum_x_slot),
        CanonicalTickValueV1::MinimumZ => Some(binding.minimum_z_slot),
        CanonicalTickValueV1::MaximumZ => Some(binding.maximum_z_slot),
    };
    Ok(ExecutableExpressionV1::Slot(
        slot.ok_or(ExecutableErrorV1::MalformedProgram)?,
    ))
}

fn lower_tick_expression(
    expression: &CanonicalTickExpressionV1,
    binding: ExecutableCanonicalTickBindingV1,
    depth: usize,
) -> Result<ExecutableExpressionV1, ExecutableErrorV1> {
    if depth >= MAX_EXPRESSION_DEPTH {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let lower_pair = |left: &CanonicalTickExpressionV1, right: &CanonicalTickExpressionV1| {
        Ok::<_, ExecutableErrorV1>((
            Box::new(lower_tick_expression(left, binding, depth + 1)?),
            Box::new(lower_tick_expression(right, binding, depth + 1)?),
        ))
    };
    Ok(match expression {
        CanonicalTickExpressionV1::Value(value) => lower_tick_value(*value, binding)?,
        CanonicalTickExpressionV1::Number(bits) => {
            ExecutableExpressionV1::Constant(ExecutableValueV1::Number(*bits))
        }
        CanonicalTickExpressionV1::Add(left, right) => {
            let (left, right) = lower_pair(left, right)?;
            ExecutableExpressionV1::Add(left, right)
        }
        CanonicalTickExpressionV1::Subtract(left, right) => {
            let (left, right) = lower_pair(left, right)?;
            ExecutableExpressionV1::Subtract(left, right)
        }
        CanonicalTickExpressionV1::Multiply(left, right) => {
            let (left, right) = lower_pair(left, right)?;
            ExecutableExpressionV1::Multiply(left, right)
        }
        CanonicalTickExpressionV1::Divide(left, right) => {
            let (left, right) = lower_pair(left, right)?;
            ExecutableExpressionV1::Divide(left, right)
        }
        CanonicalTickExpressionV1::Clamp(value, lower, upper) => ExecutableExpressionV1::Clamp(
            Box::new(lower_tick_expression(value, binding, depth + 1)?),
            Box::new(lower_tick_expression(lower, binding, depth + 1)?),
            Box::new(lower_tick_expression(upper, binding, depth + 1)?),
        ),
    })
}

/// The exact accepted lowering/refinement contract implemented by the plan.
/// This prototype recognizes only its closed rule-machine realization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExecutableRefinementV1 {
    ClosedApplicationRuleMachineV1 = 1,
}

/// The complete target/profile/ABI/strategy understood by this reversible
/// physical experiment. Adding another realization requires another explicit
/// variant; runtime selection never falls back to a host name or Term kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExecutablePhysicalTargetV1 {
    PortableScalarInterpreterV1 = 1,
}

/// One physical plan outside ProgramSnapshot identity. The exact semantic
/// shape and Mode are retained as its refinement obligation; `program` is a
/// physical realization and is never inserted into semantic dependency
/// closure.
#[derive(Clone, Debug, PartialEq)]
pub struct ExecutablePhysicalPlanV1 {
    pub application_shape: ApplicationShapeId,
    pub mode: ModeId,
    pub refinement: ExecutableRefinementV1,
    pub target: ExecutablePhysicalTargetV1,
    pub input: Option<ExecutableInputPlanV1>,
    pub program: ExecutableProgramV1,
}

/// Exact byte identity of one physical plan artifact. It is not Application,
/// Activation, semantic shape, revision, or authority identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutablePhysicalPlanIdV1([u8; IDENTITY_BYTES]);

impl ExecutablePhysicalPlanIdV1 {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; IDENTITY_BYTES]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; IDENTITY_BYTES] {
        &self.0
    }
}

#[derive(Clone, Debug)]
struct CheckedExecutablePhysicalPlanV1 {
    id: ExecutablePhysicalPlanIdV1,
    plan: ExecutablePhysicalPlanV1,
}

/// Encode one exact physical plan. These bytes are transported beside the
/// checked process package and never participate in ProgramSnapshot identity.
pub fn encode_executable_physical_plan_v1(
    plan: &ExecutablePhysicalPlanV1,
) -> Result<Vec<u8>, ExecutableErrorV1> {
    validate_program(&plan.program)?;
    validate_input_plan_shape(plan.input.as_ref(), &plan.program)?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(PHYSICAL_PLAN_MAGIC_V1);
    bytes.extend_from_slice(plan.application_shape.as_bytes());
    bytes.extend_from_slice(plan.mode.operator.snapshot.as_bytes());
    bytes.extend_from_slice(&plan.mode.operator.local.get().to_le_bytes());
    bytes.extend_from_slice(&plan.mode.local.get().to_le_bytes());
    bytes.push(plan.refinement as u8);
    bytes.push(plan.target as u8);
    encode_input_plan(&mut bytes, plan.input.as_ref())?;
    encode_program_body(&mut bytes, &plan.program)?;
    Ok(bytes)
}

/// Decode one exact physical plan without granting it semantic standing. The
/// runtime separately checks its shape/Mode refinement against the package.
pub fn decode_executable_physical_plan_v1(
    bytes: &[u8],
) -> Result<ExecutablePhysicalPlanV1, ExecutableErrorV1> {
    let mut decoder = Decoder::new(bytes);
    if decoder.take(PHYSICAL_PLAN_MAGIC_V1.len())? != PHYSICAL_PLAN_MAGIC_V1 {
        return Err(ExecutableErrorV1::MalformedPhysicalPlan);
    }
    let application_shape = ApplicationShapeId::from_bytes(decoder.identity()?);
    let snapshot = ProgramSnapshotId::from_bytes(decoder.identity()?);
    let operator = OperatorLocalId::new(decoder.u32()?);
    let mode = ModeLocalId::new(decoder.u32()?);
    let refinement = match decoder.byte()? {
        1 => ExecutableRefinementV1::ClosedApplicationRuleMachineV1,
        _ => return Err(ExecutableErrorV1::UnsupportedPhysicalRefinement),
    };
    let target = match decoder.byte()? {
        1 => ExecutablePhysicalTargetV1::PortableScalarInterpreterV1,
        _ => return Err(ExecutableErrorV1::UnsupportedPhysicalTarget),
    };
    let input = decode_input_plan(&mut decoder)?;
    let program = decode_program_body(&mut decoder)?;
    if !decoder.is_complete() {
        return Err(ExecutableErrorV1::MalformedPhysicalPlan);
    }
    let plan = ExecutablePhysicalPlanV1 {
        application_shape,
        mode: ModeId {
            operator: OperatorRef {
                snapshot,
                local: operator,
            },
            local: mode,
        },
        refinement,
        target,
        input,
        program,
    };
    validate_program(&plan.program)?;
    validate_input_plan_shape(plan.input.as_ref(), &plan.program)?;
    Ok(plan)
}

fn encode_input_source(
    bytes: &mut Vec<u8>,
    source: &ExecutableInputSourceV1,
) -> Result<(), ExecutableErrorV1> {
    match source {
        ExecutableInputSourceV1::Keyboard { code, phase } => {
            bytes.push(0);
            encode_count(bytes, code.len())?;
            bytes.extend_from_slice(code);
            bytes.push(*phase as u8);
        }
        ExecutableInputSourceV1::Scalar { channel } => {
            bytes.push(1);
            encode_count(bytes, channel.len())?;
            bytes.extend_from_slice(channel);
        }
    }
    Ok(())
}

fn decode_input_source(
    decoder: &mut Decoder<'_>,
) -> Result<ExecutableInputSourceV1, ExecutableErrorV1> {
    match decoder.byte()? {
        0 => {
            let length = decoder.count()?;
            if length == 0 || length > MAX_INPUT_CODE_BYTES {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            let code = decoder.take(length)?.to_vec();
            let phase = match decoder.byte()? {
                0 => ExecutableKeyPhaseV1::Down,
                1 => ExecutableKeyPhaseV1::Up,
                _ => return Err(ExecutableErrorV1::MalformedProgram),
            };
            Ok(ExecutableInputSourceV1::Keyboard { code, phase })
        }
        1 => {
            let length = decoder.count()?;
            if length == 0 || length > MAX_INPUT_CODE_BYTES {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            Ok(ExecutableInputSourceV1::Scalar {
                channel: decoder.take(length)?.to_vec(),
            })
        }
        _ => Err(ExecutableErrorV1::MalformedProgram),
    }
}

fn encode_input_plan(
    bytes: &mut Vec<u8>,
    input: Option<&ExecutableInputPlanV1>,
) -> Result<(), ExecutableErrorV1> {
    let Some(input) = input else {
        bytes.push(0);
        return Ok(());
    };
    bytes.push(1);
    encode_count(bytes, input.events.len())?;
    for binding in &input.events {
        bytes.extend_from_slice(&binding.role.schema.get().to_le_bytes());
        bytes.extend_from_slice(&binding.role.role.get().to_le_bytes());
        encode_input_source(bytes, &binding.source)?;
        bytes.extend_from_slice(&binding.occurrence.entry.to_le_bytes());
        encode_values(bytes, &binding.occurrence.arguments)?;
    }
    bytes.extend_from_slice(&input.tick.role.schema.get().to_le_bytes());
    bytes.extend_from_slice(&input.tick.role.role.get().to_le_bytes());
    encode_count(bytes, input.tick.entries.len())?;
    for entry in &input.tick.entries {
        bytes.extend_from_slice(&entry.to_le_bytes());
    }
    Ok(())
}

fn decode_input_plan(
    decoder: &mut Decoder<'_>,
) -> Result<Option<ExecutableInputPlanV1>, ExecutableErrorV1> {
    match decoder.byte()? {
        0 => Ok(None),
        1 => {
            let count = decoder.count()?;
            let mut events = Vec::with_capacity(count);
            for _ in 0..count {
                events.push(ExecutableInputBindingV1 {
                    role: LocalRoleRefV2 {
                        schema: RelationSchemaLocalId::new(decoder.u32()?),
                        role: RoleLocalId::new(decoder.u32()?),
                    },
                    source: decode_input_source(decoder)?,
                    occurrence: ExecutableOccurrenceV1 {
                        entry: decoder.u16()?,
                        arguments: decoder.values()?,
                    },
                });
            }
            Ok(Some(ExecutableInputPlanV1 {
                events,
                tick: ExecutableTickBindingV1 {
                    role: LocalRoleRefV2 {
                        schema: RelationSchemaLocalId::new(decoder.u32()?),
                        role: RoleLocalId::new(decoder.u32()?),
                    },
                    entries: {
                        let count = decoder.count()?;
                        let mut entries = Vec::with_capacity(count);
                        for _ in 0..count {
                            entries.push(decoder.u16()?);
                        }
                        entries
                    },
                },
            }))
        }
        _ => Err(ExecutableErrorV1::MalformedProgram),
    }
}

fn encode_program_body(
    bytes: &mut Vec<u8>,
    program: &ExecutableProgramV1,
) -> Result<(), ExecutableErrorV1> {
    encode_values(bytes, &program.initial_configuration)?;
    encode_count(bytes, program.rules.len())?;
    for rule in &program.rules {
        bytes.extend_from_slice(&rule.entry.to_le_bytes());
        encode_count(bytes, rule.predicates.len())?;
        for predicate in &rule.predicates {
            encode_expression(bytes, predicate)?;
        }
        for slots in [&rule.required_present, &rule.required_absent] {
            encode_count(bytes, slots.len())?;
            for slot in slots {
                bytes.extend_from_slice(&slot.to_le_bytes());
            }
        }
        encode_count(bytes, rule.assignments.len())?;
        for (slot, expression) in &rule.assignments {
            bytes.extend_from_slice(&slot.to_le_bytes());
            encode_expression(bytes, expression)?;
        }
        encode_count(bytes, rule.removals.len())?;
        for slot in &rule.removals {
            bytes.extend_from_slice(&slot.to_le_bytes());
        }
    }
    encode_projection(bytes, program.projection.as_ref())
}

fn decode_program_body(
    decoder: &mut Decoder<'_>,
) -> Result<ExecutableProgramV1, ExecutableErrorV1> {
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
        let decode_slots = |decoder: &mut Decoder<'_>| -> Result<Vec<u16>, ExecutableErrorV1> {
            let count = decoder.count()?;
            (0..count).map(|_| decoder.u16()).collect()
        };
        let required_present = decode_slots(decoder)?;
        let required_absent = decode_slots(decoder)?;
        let assignment_count = decoder.count()?;
        let mut assignments = Vec::with_capacity(assignment_count);
        for _ in 0..assignment_count {
            assignments.push((decoder.u16()?, decoder.expression(0)?));
        }
        let removals = decode_slots(decoder)?;
        rules.push(ExecutableRuleV1 {
            entry,
            predicates,
            required_present,
            required_absent,
            assignments,
            removals,
        });
    }
    Ok(ExecutableProgramV1 {
        initial_configuration,
        rules,
        projection: decode_projection(decoder)?,
    })
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
    pub configuration: Vec<ExecutableSlotV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableSuspensionV1 {
    pub step: StepId,
    pub continuation: ContinuationId,
    pub run: RunId,
    pub activation: ActivationId,
    pub before: ConfigurationId,
    pub after: ConfigurationId,
    pub remaining_budget: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableResumptionV1 {
    pub occurrence: ResumptionOccurrenceId,
    pub step: StepId,
    pub continuation: ContinuationId,
    pub run: RunId,
    pub activation: ActivationId,
    pub before: ConfigurationId,
    pub after: ConfigurationId,
    pub remaining_budget: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableEffectSettlementV1 {
    pub intent: EffectIntentId,
    pub attempt: EffectAttemptId,
    pub receipt: Option<EffectReceiptId>,
    pub observation: Option<ObservationId>,
    pub judgment: EffectJudgmentOccurrenceId,
    pub disposition: EffectJudgmentDispositionV1,
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
    pub configuration: Vec<ExecutableSlotV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableObservationV1 {
    pub id: ObservationId,
    pub state: StateRevisionId,
    pub value: Vec<ExecutableSlotV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableProjectedObservationV1 {
    pub id: ObservationId,
    pub state: StateRevisionId,
    pub term: Term,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableBoundaryFactV1 {
    pub boundary: BoundaryRef,
    pub evidence: ExternalEvidenceRef,
    pub permission: BoundaryPermissionLocalId,
}

pub const EXECUTABLE_TRIGGER_PERMISSION_V1: BoundaryPermissionLocalId =
    BoundaryPermissionLocalId::new(0);
pub const EXECUTABLE_OBSERVATION_PERMISSION_V1: BoundaryPermissionLocalId =
    BoundaryPermissionLocalId::new(1);
pub const EXECUTABLE_JUDGMENT_PERMISSION_V1: BoundaryPermissionLocalId =
    BoundaryPermissionLocalId::new(2);
pub const EXECUTABLE_ADMISSION_PERMISSION_V1: BoundaryPermissionLocalId =
    BoundaryPermissionLocalId::new(3);
pub const EXECUTABLE_ADMISSION_ISSUANCE_PERMISSION_V1: BoundaryPermissionLocalId =
    BoundaryPermissionLocalId::new(4);
pub const EXECUTABLE_RESUMPTION_PERMISSION_V1: BoundaryPermissionLocalId =
    BoundaryPermissionLocalId::new(5);

#[must_use]
pub fn executable_occurrence_boundary_anchor_v1(
    boundary: BoundaryRef,
    payload: FormationTargetV2,
    pins: BoundaryPins,
) -> BoundaryAnchor {
    let at_most_one = CardinalityV2 {
        minimum: 0,
        maximum: Some(1),
    };
    BoundaryAnchor {
        boundary,
        permissions: vec![
            BoundaryOccurrencePermissionV2 {
                id: EXECUTABLE_TRIGGER_PERMISSION_V1,
                kind: EnteredOccurrenceKind::ExternalTrigger,
                payload: payload.clone(),
                pins,
                cause_schema: vec![],
                support_schema: vec![],
                replay: BoundaryReplayPolicyV2::OneShot,
            },
            BoundaryOccurrencePermissionV2 {
                id: EXECUTABLE_OBSERVATION_PERMISSION_V1,
                kind: EnteredOccurrenceKind::Observation,
                payload: payload.clone(),
                pins,
                cause_schema: vec![
                    BoundaryCauseRequirementV2 {
                        kind: EnteredCauseKindV2::ExternalTrigger,
                        cardinality: at_most_one,
                    },
                    BoundaryCauseRequirementV2 {
                        kind: EnteredCauseKindV2::Step,
                        cardinality: at_most_one,
                    },
                    BoundaryCauseRequirementV2 {
                        kind: EnteredCauseKindV2::Admission,
                        cardinality: at_most_one,
                    },
                ],
                support_schema: vec![],
                replay: BoundaryReplayPolicyV2::Repeatable {
                    maximum_occurrences: None,
                },
            },
        ],
    }
}

#[must_use]
pub fn executable_state_boundary_anchor_v1(
    boundary: BoundaryRef,
    payload: FormationTargetV2,
    pins: BoundaryPins,
) -> BoundaryAnchor {
    let exactly_one = CardinalityV2 {
        minimum: 1,
        maximum: Some(1),
    };
    BoundaryAnchor {
        boundary,
        permissions: vec![
            BoundaryOccurrencePermissionV2 {
                id: EXECUTABLE_JUDGMENT_PERMISSION_V1,
                kind: EnteredOccurrenceKind::Judgment,
                payload: payload.clone(),
                pins,
                cause_schema: vec![BoundaryCauseRequirementV2 {
                    kind: EnteredCauseKindV2::CandidateDelta,
                    cardinality: exactly_one,
                }],
                support_schema: vec![],
                replay: BoundaryReplayPolicyV2::Repeatable {
                    maximum_occurrences: None,
                },
            },
            BoundaryOccurrencePermissionV2 {
                id: EXECUTABLE_ADMISSION_PERMISSION_V1,
                kind: EnteredOccurrenceKind::AdmissionDecision,
                payload: payload.clone(),
                pins,
                cause_schema: vec![
                    BoundaryCauseRequirementV2 {
                        kind: EnteredCauseKindV2::CandidateDelta,
                        cardinality: exactly_one,
                    },
                    BoundaryCauseRequirementV2 {
                        kind: EnteredCauseKindV2::Judgment,
                        cardinality: CardinalityV2 {
                            minimum: 1,
                            maximum: None,
                        },
                    },
                ],
                support_schema: vec![],
                replay: BoundaryReplayPolicyV2::Repeatable {
                    maximum_occurrences: None,
                },
            },
            BoundaryOccurrencePermissionV2 {
                id: EXECUTABLE_ADMISSION_ISSUANCE_PERMISSION_V1,
                kind: EnteredOccurrenceKind::AdmissionAuthorization,
                payload: payload.clone(),
                pins,
                cause_schema: vec![BoundaryCauseRequirementV2 {
                    kind: EnteredCauseKindV2::CandidateDelta,
                    cardinality: exactly_one,
                }],
                support_schema: vec![],
                replay: BoundaryReplayPolicyV2::Repeatable {
                    maximum_occurrences: None,
                },
            },
            BoundaryOccurrencePermissionV2 {
                id: EXECUTABLE_RESUMPTION_PERMISSION_V1,
                kind: EnteredOccurrenceKind::Resumption,
                payload,
                pins,
                cause_schema: vec![BoundaryCauseRequirementV2 {
                    kind: EnteredCauseKindV2::Step,
                    cardinality: exactly_one,
                }],
                support_schema: vec![],
                replay: BoundaryReplayPolicyV2::Repeatable {
                    maximum_occurrences: None,
                },
            },
        ],
    }
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
    pub admission_authorization_issuer: RootAdmissionAuthorizationIssuerRef,
    pub trigger_ingress: ExecutableBoundaryFactV1,
    pub occurrence_ingress: ExecutableBoundaryFactV1,
    pub resumption_ingress: ExecutableBoundaryFactV1,
    pub judgment_ingress: ExecutableBoundaryFactV1,
    pub admission_issuance_ingress: ExecutableBoundaryFactV1,
    pub admission_ingress: ExecutableBoundaryFactV1,
    pub budget_units: u64,
}

/// Provenance-bearing allocation root for one recorded runtime occurrence
/// family. `fresh` is minted only by the runtime; the exact record may later
/// be supplied only to rematerialize that same occurrence family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeAllocationEpochV1 {
    root: [u8; IDENTITY_BYTES],
    occurrence: SessionStartOccurrenceId,
    session: RuntimeSessionId,
    constitution: ProgramRevisionId,
    package: ProcessPackageId,
    application: ApplicationId,
    application_shape: ApplicationShapeId,
    mode: ModeId,
    physical_plan: ExecutablePhysicalPlanIdV1,
}

impl RuntimeAllocationEpochV1 {
    /// Reconstitute the exact allocation evidence for an occurrence already
    /// recorded outside this process. `root` is never accepted by the new-run
    /// path; this constructor exists only for explicit rematerialization.
    pub fn recorded_for(
        root: [u8; IDENTITY_BYTES],
        package: &CheckedProcessPackage,
        application: ApplicationId,
        physical_plan: &ExecutablePhysicalPlanV1,
        facts: ExecutableAuthorityFactsV1,
    ) -> Result<Self, ExecutableErrorV1> {
        let plan = check_executable_physical_plan_v1(
            package.constitution(),
            application,
            physical_plan.clone(),
        )?;
        Ok(Self {
            root,
            occurrence: facts.session_start,
            session: facts.session,
            constitution: facts.program_revision,
            package: package.id(),
            application,
            application_shape: plan.plan.application_shape,
            mode: plan.plan.mode,
            physical_plan: plan.id,
        })
    }

    #[must_use]
    pub const fn root(&self) -> &[u8; IDENTITY_BYTES] {
        &self.root
    }

    #[must_use]
    pub const fn occurrence(&self) -> SessionStartOccurrenceId {
        self.occurrence
    }

    #[must_use]
    pub const fn session(&self) -> RuntimeSessionId {
        self.session
    }

    #[must_use]
    pub const fn constitution(&self) -> ProgramRevisionId {
        self.constitution
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
    pub const fn physical_plan(&self) -> ExecutablePhysicalPlanIdV1 {
        self.physical_plan
    }

    /// Derive an exact subordinate candidate identity from this fully typed
    /// recorded allocation root. This is used only to scope authority before
    /// rematerializing the recorded occurrence.
    #[must_use]
    pub fn candidate_id(&self, ordinal: u64) -> CandidateDeltaId {
        CandidateDeltaId::from_bytes(
            runtime_identity_bytes(self.root, RuntimeIdentityDomainV1::Candidate, ordinal)
                .expect("a recorded allocation accepts every u64 candidate ordinal"),
        )
    }

    #[must_use]
    pub const fn application_shape(&self) -> ApplicationShapeId {
        self.application_shape
    }

    #[must_use]
    pub const fn mode(&self) -> ModeId {
        self.mode
    }

    fn allocate_fresh(
        package: ProcessPackageId,
        application: ApplicationId,
        plan: &CheckedExecutablePhysicalPlanV1,
        facts: ExecutableAuthorityFactsV1,
    ) -> Result<Self, ExecutableErrorV1> {
        let mode_operator = plan.plan.mode.operator.local.get().to_be_bytes();
        let mode = plan.plan.mode.local.get().to_be_bytes();
        let application_local = application.local.get().to_be_bytes();
        let roots = ALLOCATED_RUNTIME_ROOTS_V1.get_or_init(|| Mutex::new(BTreeSet::new()));
        let root = loop {
            let mut fresh = [0_u8; IDENTITY_BYTES];
            getrandom::fill(&mut fresh).map_err(|_| ExecutableErrorV1::AllocationUnavailable)?;
            let candidate = runtime_domain_hash(
                "clause/runtime-allocation-epoch/v1",
                &[
                    &fresh,
                    facts.session_start.as_bytes(),
                    facts.session.as_bytes(),
                    facts.program_revision.as_bytes(),
                    package.as_bytes(),
                    application.snapshot.as_bytes(),
                    &application_local,
                    plan.plan.application_shape.as_bytes(),
                    plan.plan.mode.operator.snapshot.as_bytes(),
                    &mode_operator,
                    &mode,
                    plan.id.as_bytes(),
                ],
            );
            let mut allocated = roots
                .lock()
                .map_err(|_| ExecutableErrorV1::AllocationUnavailable)?;
            if allocated.insert(candidate) {
                break candidate;
            }
        };
        Ok(Self {
            root,
            occurrence: facts.session_start,
            session: facts.session,
            constitution: facts.program_revision,
            package,
            application,
            application_shape: plan.plan.application_shape,
            mode: plan.plan.mode,
            physical_plan: plan.id,
        })
    }

    fn validate_rematerialization(
        self,
        package: ProcessPackageId,
        application: ApplicationId,
        plan: &CheckedExecutablePhysicalPlanV1,
        facts: ExecutableAuthorityFactsV1,
    ) -> Result<Self, ExecutableErrorV1> {
        if self.occurrence != facts.session_start
            || self.session != facts.session
            || self.constitution != facts.program_revision
            || self.package != package
            || self.application != application
            || self.application_shape != plan.plan.application_shape
            || self.mode != plan.plan.mode
            || self.physical_plan != plan.id
        {
            return Err(ExecutableErrorV1::AllocationBindingMismatch);
        }
        ALLOCATED_RUNTIME_ROOTS_V1
            .get_or_init(|| Mutex::new(BTreeSet::new()))
            .lock()
            .map_err(|_| ExecutableErrorV1::AllocationUnavailable)?
            .insert(self.root);
        Ok(self)
    }
}

/// Encode the exact provenance record required to rematerialize one recorded
/// occurrence family. Possessing these bytes does not license a new run.
#[must_use]
pub fn encode_runtime_allocation_epoch_v1(epoch: RuntimeAllocationEpochV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(ALLOCATION_EPOCH_MAGIC_V1);
    bytes.extend_from_slice(&epoch.root);
    bytes.extend_from_slice(epoch.occurrence.as_bytes());
    bytes.extend_from_slice(epoch.session.as_bytes());
    bytes.extend_from_slice(epoch.constitution.as_bytes());
    bytes.extend_from_slice(epoch.package.as_bytes());
    bytes.extend_from_slice(epoch.application.snapshot.as_bytes());
    bytes.extend_from_slice(&epoch.application.local.get().to_le_bytes());
    bytes.extend_from_slice(epoch.application_shape.as_bytes());
    bytes.extend_from_slice(epoch.mode.operator.snapshot.as_bytes());
    bytes.extend_from_slice(&epoch.mode.operator.local.get().to_le_bytes());
    bytes.extend_from_slice(&epoch.mode.local.get().to_le_bytes());
    bytes.extend_from_slice(epoch.physical_plan.as_bytes());
    bytes
}

/// Decode an allocation record for the explicit rematerialization path. Every
/// binding is rechecked against the package, authority facts, and physical
/// plan before any runtime identity is derived from it.
pub fn decode_runtime_allocation_epoch_v1(
    bytes: &[u8],
) -> Result<RuntimeAllocationEpochV1, ExecutableErrorV1> {
    let mut decoder = Decoder::new(bytes);
    if decoder.take(ALLOCATION_EPOCH_MAGIC_V1.len())? != ALLOCATION_EPOCH_MAGIC_V1 {
        return Err(ExecutableErrorV1::MalformedAllocationEpoch);
    }
    let epoch = RuntimeAllocationEpochV1 {
        root: decoder.identity()?,
        occurrence: SessionStartOccurrenceId::from_bytes(decoder.identity()?),
        session: RuntimeSessionId::from_bytes(decoder.identity()?),
        constitution: ProgramRevisionId::from_bytes(decoder.identity()?),
        package: ProcessPackageId::from_bytes(decoder.identity()?),
        application: ApplicationId {
            snapshot: ProgramSnapshotId::from_bytes(decoder.identity()?),
            local: ApplicationLocalId::new(decoder.u32()?),
        },
        application_shape: ApplicationShapeId::from_bytes(decoder.identity()?),
        mode: ModeId {
            operator: OperatorRef {
                snapshot: ProgramSnapshotId::from_bytes(decoder.identity()?),
                local: OperatorLocalId::new(decoder.u32()?),
            },
            local: ModeLocalId::new(decoder.u32()?),
        },
        physical_plan: ExecutablePhysicalPlanIdV1(decoder.identity()?),
    };
    if !decoder.is_complete() {
        return Err(ExecutableErrorV1::MalformedAllocationEpoch);
    }
    Ok(epoch)
}

#[derive(Clone, Debug)]
struct CarrierExecutionV1 {
    facts: ExecutableAuthorityFactsV1,
    mode: ModeId,
    checker_mode: Option<ModeId>,
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
    configuration: Vec<ExecutableSlotV1>,
    input: Option<ExecutableInputPlanV1>,
    program: ExecutableProgramV1,
    physical_plan: ExecutablePhysicalPlanIdV1,
    physical_mode: ModeId,
    allocation: RuntimeAllocationEpochV1,
    last_step: Option<ExecutableStepV1>,
    candidate: Option<ExecutableCandidateV1>,
    judgment: Option<ExecutableJudgmentV1>,
    admission: Option<ExecutableAdmissionV1>,
    state: Option<ExecutableStateRevisionV1>,
    carrier_execution: Option<CarrierExecutionV1>,
    identity_ordinals: RuntimeIdentityOrdinalsV1,
    active_candidate_ordinal: Option<u64>,
    issued_admission_authorization: Option<IssuedAdmissionAuthorizationOccurrenceId>,
    suspended_continuation: Option<ContinuationId>,
    pending_effect_intent: Option<EffectIntentId>,
    active_effect_attempt: Option<EffectAttemptId>,
}

impl ExecutableProcessRuntimeV1 {
    /// Instantiate a new occurrence family from one exact physical plan. The
    /// runtime mints a fresh allocation epoch after checking the plan's exact
    /// ApplicationShape/Mode refinement against the package.
    pub fn instantiate_new(
        package: CheckedProcessPackage,
        authority: clause_package::AuthorityStore,
        application: ApplicationId,
        physical_plan: ExecutablePhysicalPlanV1,
        facts: ExecutableAuthorityFactsV1,
    ) -> Result<Self, ExecutableErrorV1> {
        let physical_plan =
            check_executable_physical_plan_v1(package.constitution(), application, physical_plan)?;
        let package_id = package.id();
        let allocation = RuntimeAllocationEpochV1::allocate_fresh(
            package_id,
            application,
            &physical_plan,
            facts,
        )?;
        let carrier = ProcessRuntime::instantiate(package, authority)
            .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
        Self::from_parts(carrier, package_id, application, physical_plan, allocation)
    }

    pub(crate) fn reclaim_retired_entries(&mut self, maximum_entries: usize) -> bool {
        self.carrier.reclaim_retired_entries(maximum_entries)
    }

    /// Rematerialize an already-recorded occurrence family. The exact
    /// allocation root is preserved only after all typed provenance, semantic
    /// shape, Mode, and physical-plan bindings match.
    pub fn instantiate_rematerialized(
        package: CheckedProcessPackage,
        authority: clause_package::AuthorityStore,
        application: ApplicationId,
        physical_plan: ExecutablePhysicalPlanV1,
        facts: ExecutableAuthorityFactsV1,
        allocation: RuntimeAllocationEpochV1,
    ) -> Result<Self, ExecutableErrorV1> {
        let physical_plan =
            check_executable_physical_plan_v1(package.constitution(), application, physical_plan)?;
        let package_id = package.id();
        let allocation = allocation.validate_rematerialization(
            package_id,
            application,
            &physical_plan,
            facts,
        )?;
        let carrier = ProcessRuntime::instantiate(package, authority)
            .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
        Self::from_parts(carrier, package_id, application, physical_plan, allocation)
    }

    fn from_parts(
        carrier: ProcessRuntime,
        package: ProcessPackageId,
        application: ApplicationId,
        physical_plan: CheckedExecutablePhysicalPlanV1,
        allocation: RuntimeAllocationEpochV1,
    ) -> Result<Self, ExecutableErrorV1> {
        if carrier.carrier().application(application).is_none() {
            return Err(ExecutableErrorV1::UnknownApplication);
        }
        let identity_root = allocation.root;
        let run = RunId::from_bytes(runtime_identity_bytes(
            identity_root,
            RuntimeIdentityDomainV1::Run,
            0,
        )?);
        let activation = ActivationId::from_bytes(runtime_identity_bytes(
            identity_root,
            RuntimeIdentityDomainV1::Activation,
            0,
        )?);
        let configuration_id = ConfigurationId::from_bytes(runtime_identity_bytes(
            identity_root,
            RuntimeIdentityDomainV1::Configuration,
            0,
        )?);
        let physical_mode = physical_plan.plan.mode;
        let input = physical_plan.plan.input;
        let program = physical_plan.plan.program;
        let configuration = materialize_initial_configuration(&program)?;
        Ok(Self {
            carrier,
            package,
            application,
            run,
            activation,
            configuration_id,
            configuration,
            input,
            program,
            physical_plan: physical_plan.id,
            physical_mode,
            allocation,
            last_step: None,
            candidate: None,
            judgment: None,
            admission: None,
            state: None,
            carrier_execution: None,
            identity_ordinals: RuntimeIdentityOrdinalsV1::initial(),
            active_candidate_ordinal: None,
            issued_admission_authorization: None,
            suspended_continuation: None,
            pending_effect_intent: None,
            active_effect_attempt: None,
        })
    }
}

fn check_executable_physical_plan_v1(
    constitution: &ResolvedProgramConstitutionV2,
    application: ApplicationId,
    plan: ExecutablePhysicalPlanV1,
) -> Result<CheckedExecutablePhysicalPlanV1, ExecutableErrorV1> {
    let shape = constitution
        .application_shape(application.local)
        .filter(|_| application.snapshot == constitution.snapshot())
        .ok_or(ExecutableErrorV1::UnknownApplication)?;
    if shape != plan.application_shape {
        return Err(ExecutableErrorV1::PhysicalShapeMismatch);
    }
    if constitution
        .executable_contract(application, plan.mode)
        .is_none()
    {
        return Err(ExecutableErrorV1::PhysicalModeMismatch);
    }
    validate_program(&plan.program)?;
    validate_projection_roles(constitution, &plan.program)?;
    validate_input_roles(constitution, plan.input.as_ref())?;
    let exact = encode_executable_physical_plan_v1(&plan)?;
    let id = ExecutablePhysicalPlanIdV1(runtime_domain_hash(
        "clause/executable-physical-plan/v1",
        &[&exact],
    ));
    Ok(CheckedExecutablePhysicalPlanV1 { id, plan })
}

fn exact_role_exists(constitution: &ResolvedProgramConstitutionV2, role: LocalRoleRefV2) -> bool {
    constitution
        .preimage()
        .schemas
        .iter()
        .find(|schema| schema.id == role.schema)
        .and_then(|schema| {
            schema
                .roles
                .iter()
                .find(|declared| declared.id == role.role)
        })
        .is_some_and(|declared| declared.cardinality.is_exactly_one())
}

fn validate_input_roles(
    constitution: &ResolvedProgramConstitutionV2,
    input: Option<&ExecutableInputPlanV1>,
) -> Result<(), ExecutableErrorV1> {
    let Some(input) = input else {
        return Ok(());
    };
    if !exact_role_exists(constitution, input.tick.role)
        || input
            .events
            .iter()
            .any(|binding| !exact_role_exists(constitution, binding.role))
    {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    Ok(())
}

fn validate_projection_roles(
    constitution: &ResolvedProgramConstitutionV2,
    program: &ExecutableProgramV1,
) -> Result<(), ExecutableErrorV1> {
    let Some(projection) = &program.projection else {
        return Ok(());
    };
    for binding in &projection.bindings {
        let schema = constitution
            .preimage()
            .schemas
            .iter()
            .find(|schema| schema.id == binding.role.schema)
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        let role = schema
            .roles
            .iter()
            .find(|role| role.id == binding.role.role)
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        if !role.cardinality.is_exactly_one() {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
    Ok(())
}

impl ExecutableProcessRuntimeV1 {
    /// Start the unique stateful or effectful Mode constituted for this
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
                || !record.contract.effect_intents.is_empty()
            {
                if selected.replace(mode).is_some() {
                    return Err(ExecutableCarrierErrorV1::AmbiguousStatefulMode);
                }
            }
        }
        let mode = selected.ok_or(ExecutableCarrierErrorV1::MissingStatefulMode)?;
        if mode != self.physical_mode {
            return Err(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::PhysicalModeMismatch,
            ));
        }
        let mode_record = constitution
            .mode_by_id(mode)
            .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
        let executable = constitution
            .executable_contract(self.application, mode)
            .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
        if !executable.authorization_requirements.is_empty()
            || !mode_record.contract.scheduling_requirements.is_empty()
            || !mode_record.contract.resource_requirements.is_empty()
        {
            return Err(ExecutableCarrierErrorV1::UnsupportedSurface);
        }
        let mut checker_mode = None;
        if let Some(target) = mode_record.contract.state_delta_domain.as_ref() {
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
            if checker_mode.is_none() {
                return Err(ExecutableCarrierErrorV1::MissingCheckerMode);
            }
        }
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

    /// Emit the one Mode-declared external-effect intent at a semantic Step.
    /// Action, resource, and payload are resolved from exact Application role
    /// bindings; the host supplies no semantic selector or payload value.
    pub fn emit_carrier_effect_intent(
        &mut self,
    ) -> Result<EffectIntentOccurrenceV1, ExecutableCarrierErrorV1> {
        if self.pending_effect_intent.is_some() {
            return Err(ExecutableCarrierErrorV1::EffectLifecycleAlreadyActive);
        }
        if self.suspended_continuation.is_some() {
            return Err(ExecutableCarrierErrorV1::AlreadySuspended);
        }
        let (facts, mode, remaining_budget, prior_step) = {
            let execution = self
                .carrier_execution
                .as_ref()
                .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
            (
                execution.facts,
                execution.mode,
                execution.remaining_budget,
                execution
                    .prior_step
                    .ok_or(ExecutableCarrierErrorV1::NotStarted)?,
            )
        };
        if remaining_budget == 0 {
            return Err(ExecutableCarrierErrorV1::BudgetExhausted);
        }
        let contract = {
            let mode_record = self
                .carrier
                .carrier()
                .constitution()
                .mode_by_id(mode)
                .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
            let [contract] = mode_record.contract.effect_intents.as_slice() else {
                return Err(ExecutableCarrierErrorV1::UnsupportedEffectContract);
            };
            contract.clone()
        };
        let action = self.application_role_term(contract.action_role)?;
        let resource = self.application_role_term(contract.resource_role)?;
        let payload = self.application_role_term(contract.payload_role)?;
        let (step_ordinal, next_step_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_step)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (configuration_ordinal, next_configuration_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_configuration)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (intent_ordinal, next_intent_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_effect_intent)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let step = StepId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::Step,
                step_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let after = ConfigurationId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::Configuration,
                configuration_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let intent_id = EffectIntentId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::EffectIntent,
                intent_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let configuration = executable_configuration_term_v1(scope, &self.configuration)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        let after_budget = remaining_budget - 1;
        let reference = StepRef {
            run: self.run,
            activation: self.activation,
            step,
        };
        let step_record = StepProposalV2 {
            id: step,
            run: self.run,
            activation: self.activation,
            before: self.configuration_id,
            after: ConfigurationProposal {
                id: after,
                value: configuration,
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
            causes: vec![StepCause::PriorStep(prior_step)],
            observation_outcomes: vec![],
            candidate_delta: None,
            outcome: StepOutcomeProposalV2::Progress,
        };
        let intent = EffectIntentOccurrenceV1 {
            id: intent_id,
            emitted_by: reference,
            contract_index: 0,
            required_capability: CapabilityRef {
                snapshot: self.carrier.carrier().constitution().snapshot(),
                local: contract.required_capability,
            },
            scope: EffectScopeV1 {
                application: self.application,
                mode,
                program_revision: facts.program_revision,
                world: facts.initial_state,
                session: facts.session,
                budget: Budget {
                    remaining_units: after_budget,
                },
            },
            action,
            resource,
            payload,
        };
        self.carrier
            .apply_ingress(&[
                ProcessRecordV2::Steps(vec![step_record]),
                ProcessRecordV2::EffectIntent(intent.clone()),
            ])
            .map_err(ExecutableCarrierErrorV1::Ingress)?;
        self.configuration_id = after;
        self.identity_ordinals.next_step = next_step_ordinal;
        self.identity_ordinals.next_configuration = next_configuration_ordinal;
        self.identity_ordinals.next_effect_intent = next_intent_ordinal;
        self.pending_effect_intent = Some(intent_id);
        let execution = self
            .carrier_execution
            .as_mut()
            .expect("effect intent retains its execution");
        execution.remaining_budget = after_budget;
        execution.prior_step = Some(reference);
        Ok(intent)
    }

    #[must_use]
    pub const fn pending_carrier_effect_intent(&self) -> Option<EffectIntentId> {
        self.pending_effect_intent
    }

    pub fn issue_carrier_effect_authorization(
        &mut self,
        intent: EffectIntentId,
    ) -> Result<IssuedEffectAuthorizationV1, ExecutableCarrierErrorV1> {
        if self.pending_effect_intent != Some(intent) {
            return Err(ExecutableCarrierErrorV1::UnknownPendingEffectIntent);
        }
        let intent_record = self
            .carrier
            .carrier()
            .effect_intent(intent)
            .cloned()
            .ok_or(ExecutableCarrierErrorV1::UnknownPendingEffectIntent)?;
        let (ordinal, next_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_effect_authorization)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let authorization = IssuedEffectAuthorizationV1 {
            id: IssuedEffectAuthorizationOccurrenceId::from_bytes(
                runtime_identity_bytes(
                    self.allocation.root,
                    RuntimeIdentityDomainV1::EffectAuthorization,
                    ordinal,
                )
                .map_err(ExecutableCarrierErrorV1::Executable)?,
            ),
            intent,
            capability: intent_record.required_capability,
            scope: intent_record.scope,
            action: intent_record.action,
            resource: intent_record.resource,
            payload: intent_record.payload,
        };
        self.carrier
            .apply_ingress(&[ProcessRecordV2::IssuedEffectAuthorization(
                authorization.clone(),
            )])
            .map_err(ExecutableCarrierErrorV1::Ingress)?;
        self.identity_ordinals.next_effect_authorization = next_ordinal;
        Ok(authorization)
    }

    pub fn begin_carrier_effect_attempt(
        &mut self,
        authorization: IssuedEffectAuthorizationOccurrenceId,
    ) -> Result<EffectAttemptOccurrenceV1, ExecutableCarrierErrorV1> {
        let issued = self
            .carrier
            .carrier()
            .issued_effect_authorization(authorization)
            .cloned()
            .ok_or(ExecutableCarrierErrorV1::UnknownEffectAuthorization)?;
        if self.pending_effect_intent != Some(issued.intent) {
            return Err(ExecutableCarrierErrorV1::UnknownPendingEffectIntent);
        }
        let (ordinal, next_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_effect_attempt)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let attempt = EffectAttemptOccurrenceV1 {
            id: EffectAttemptId::from_bytes(
                runtime_identity_bytes(
                    self.allocation.root,
                    RuntimeIdentityDomainV1::EffectAttempt,
                    ordinal,
                )
                .map_err(ExecutableCarrierErrorV1::Executable)?,
            ),
            intent: issued.intent,
            authorization,
            scope: issued.scope,
            action: issued.action,
            resource: issued.resource,
            payload: issued.payload,
        };
        self.carrier
            .apply_ingress(&[ProcessRecordV2::EffectAttempt(attempt.clone())])
            .map_err(ExecutableCarrierErrorV1::Ingress)?;
        self.identity_ordinals.next_effect_attempt = next_ordinal;
        self.active_effect_attempt = Some(attempt.id);
        Ok(attempt)
    }

    pub fn settle_carrier_effect_attempt(
        &mut self,
        attempt: EffectAttemptId,
        receipt: Option<(u32, Vec<u8>)>,
    ) -> Result<ExecutableEffectSettlementV1, ExecutableCarrierErrorV1> {
        if self.active_effect_attempt != Some(attempt) {
            return Err(ExecutableCarrierErrorV1::UnknownActiveEffectAttempt);
        }
        let attempt_record = self
            .carrier
            .carrier()
            .effect_attempt(attempt)
            .cloned()
            .ok_or(ExecutableCarrierErrorV1::UnknownActiveEffectAttempt)?;
        let (judgment_ordinal, next_judgment_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_effect_judgment)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let judgment_id = EffectJudgmentOccurrenceId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::EffectJudgment,
                judgment_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let mut records = Vec::new();
        let (receipt_id, observation_id, disposition, next_receipt, next_observation) =
            if let Some((status, exact_bytes)) = receipt {
                let (receipt_ordinal, next_receipt) =
                    stage_runtime_ordinal(self.identity_ordinals.next_effect_receipt)
                        .map_err(ExecutableCarrierErrorV1::Executable)?;
                let (observation_ordinal, next_observation) =
                    stage_runtime_ordinal(self.identity_ordinals.next_effect_observation)
                        .map_err(ExecutableCarrierErrorV1::Executable)?;
                let receipt_id = EffectReceiptId::from_bytes(
                    runtime_identity_bytes(
                        self.allocation.root,
                        RuntimeIdentityDomainV1::EffectReceipt,
                        receipt_ordinal,
                    )
                    .map_err(ExecutableCarrierErrorV1::Executable)?,
                );
                let observation_id = ObservationId::from_bytes(
                    runtime_identity_bytes(
                        self.allocation.root,
                        RuntimeIdentityDomainV1::EffectObservation,
                        observation_ordinal,
                    )
                    .map_err(ExecutableCarrierErrorV1::Executable)?,
                );
                let scope = TermScope {
                    universe: self.carrier.carrier().constitution().universe(),
                    semantics: self.carrier.carrier().constitution().semantics(),
                };
                let mut payload = Vec::with_capacity(8 + exact_bytes.len());
                payload.extend_from_slice(&status.to_le_bytes());
                payload.extend_from_slice(
                    &u32::try_from(exact_bytes.len())
                        .map_err(|_| ExecutableCarrierErrorV1::UnsupportedSurface)?
                        .to_le_bytes(),
                );
                payload.extend_from_slice(&exact_bytes);
                let value = Term::atom(
                    scope,
                    b"clause/effect-observation-v1".to_vec(),
                    payload,
                    EqualityContract::ExactOctetsV1,
                )
                .map_err(|_| ExecutableCarrierErrorV1::UnsupportedSurface)?;
                records.extend([
                    ProcessRecordV2::EffectReceipt(EffectReceiptOccurrenceV1 {
                        id: receipt_id,
                        attempt,
                        status,
                        exact_bytes,
                    }),
                    ProcessRecordV2::EffectObservation(EffectObservationV1 {
                        receipt: receipt_id,
                        observation: ObservationProposalV2::Value {
                            id: observation_id,
                            value,
                            supports: vec![],
                        },
                    }),
                ]);
                (
                    Some(receipt_id),
                    Some(observation_id),
                    EffectJudgmentDispositionV1::ReceiptObserved,
                    Some(next_receipt),
                    Some(next_observation),
                )
            } else {
                (
                    None,
                    None,
                    EffectJudgmentDispositionV1::NoReceipt,
                    None,
                    None,
                )
            };
        records.push(ProcessRecordV2::EffectJudgment(
            EffectJudgmentOccurrenceV1 {
                id: judgment_id,
                intent: attempt_record.intent,
                attempt,
                receipt: receipt_id,
                observation: observation_id,
                disposition,
            },
        ));
        self.carrier
            .apply_ingress(&records)
            .map_err(ExecutableCarrierErrorV1::Ingress)?;
        self.identity_ordinals.next_effect_judgment = next_judgment_ordinal;
        if let Some(next) = next_receipt {
            self.identity_ordinals.next_effect_receipt = next;
        }
        if let Some(next) = next_observation {
            self.identity_ordinals.next_effect_observation = next;
        }
        self.active_effect_attempt = None;
        self.pending_effect_intent = None;
        Ok(ExecutableEffectSettlementV1 {
            intent: attempt_record.intent,
            attempt,
            receipt: receipt_id,
            observation: observation_id,
            judgment: judgment_id,
            disposition,
        })
    }

    fn application_role_term(&self, role: RoleLocalId) -> Result<Term, ExecutableCarrierErrorV1> {
        let constitution = self.carrier.carrier().constitution();
        let application = constitution
            .application_by_id(self.application)
            .ok_or(ExecutableCarrierErrorV1::UnsupportedEffectContract)?;
        let binding = application
            .form
            .bindings
            .iter()
            .find(|binding| binding.role == role && binding.occurrence == 0)
            .ok_or(ExecutableCarrierErrorV1::UnsupportedEffectContract)?;
        let RoleBindingValuePreimageV2::Known(formation) = binding.value else {
            return Err(ExecutableCarrierErrorV1::UnsupportedEffectContract);
        };
        constitution
            .preimage()
            .formations
            .iter()
            .find(|candidate| candidate.id == formation)
            .map(|candidate| candidate.term.clone())
            .ok_or(ExecutableCarrierErrorV1::UnsupportedEffectContract)
    }

    /// Suspend the live Activation at one exact semantic Step and retain a
    /// linear Continuation pinned to the complete execution context. The
    /// physical host receives custody of the identifier only; it does not
    /// manufacture the continuation or its pins.
    pub fn suspend_carrier_process(
        &mut self,
    ) -> Result<ExecutableSuspensionV1, ExecutableCarrierErrorV1> {
        if self.suspended_continuation.is_some() {
            return Err(ExecutableCarrierErrorV1::AlreadySuspended);
        }
        if self.candidate.is_some() {
            return Err(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::CandidateAlreadyEmitted,
            ));
        }
        let (facts, remaining_budget, prior_step) = {
            let execution = self
                .carrier_execution
                .as_ref()
                .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
            (
                execution.facts,
                execution.remaining_budget,
                execution
                    .prior_step
                    .ok_or(ExecutableCarrierErrorV1::NotStarted)?,
            )
        };
        if remaining_budget <= 1 {
            return Err(ExecutableCarrierErrorV1::BudgetExhausted);
        }
        let activation = self
            .carrier
            .carrier()
            .activation(self.activation)
            .cloned()
            .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
        let (step_ordinal, next_step_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_step)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (configuration_ordinal, next_configuration_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_configuration)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (continuation_ordinal, next_continuation_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_continuation)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let step = StepId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::Step,
                step_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let after = ConfigurationId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::Configuration,
                configuration_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let continuation = ContinuationId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::Continuation,
                continuation_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let remainder = executable_configuration_term_v1(scope, &self.configuration)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        let after_budget = remaining_budget - 1;
        let reference = StepRef {
            run: self.run,
            activation: self.activation,
            step,
        };
        let record = StepProposalV2 {
            id: step,
            run: self.run,
            activation: self.activation,
            before: self.configuration_id,
            after: ConfigurationProposal {
                id: after,
                value: remainder.clone(),
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
            causes: vec![StepCause::PriorStep(prior_step)],
            observation_outcomes: vec![],
            candidate_delta: None,
            outcome: StepOutcomeProposalV2::Suspend(ContinuationProposalV2 {
                id: continuation,
                emitted_by: step,
                pins: ContinuationPins {
                    run: self.run,
                    activation: self.activation,
                    application: self.application,
                    mode: activation.mode(),
                    activation_pins: activation.pins().clone(),
                    remaining_budget: Budget {
                        remaining_units: after_budget,
                    },
                },
                remainder,
            }),
        };
        self.carrier
            .apply_ingress(&[ProcessRecordV2::Steps(vec![record])])
            .map_err(ExecutableCarrierErrorV1::Ingress)?;
        let before = self.configuration_id;
        self.configuration_id = after;
        self.identity_ordinals.next_step = next_step_ordinal;
        self.identity_ordinals.next_configuration = next_configuration_ordinal;
        self.identity_ordinals.next_continuation = next_continuation_ordinal;
        self.suspended_continuation = Some(continuation);
        let execution = self
            .carrier_execution
            .as_mut()
            .expect("suspension retains its execution");
        execution.remaining_budget = after_budget;
        execution.prior_step = Some(reference);
        Ok(ExecutableSuspensionV1 {
            step,
            continuation,
            run: self.run,
            activation: self.activation,
            before,
            after,
            remaining_budget: after_budget,
        })
    }

    /// Consume the live linear Continuation through one fresh Resumption
    /// occurrence and one semantic Step. Resumption keeps the same Run and
    /// Activation; subsequent execution remains causally downstream of the
    /// continuation takeup rather than host command order.
    pub fn resume_carrier_process(
        &mut self,
    ) -> Result<ExecutableResumptionV1, ExecutableCarrierErrorV1> {
        let continuation = self
            .suspended_continuation
            .ok_or(ExecutableCarrierErrorV1::NotSuspended)?;
        let (facts, remaining_budget, emitter) = {
            let execution = self
                .carrier_execution
                .as_ref()
                .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
            (
                execution.facts,
                execution.remaining_budget,
                execution
                    .prior_step
                    .ok_or(ExecutableCarrierErrorV1::NotStarted)?,
            )
        };
        if remaining_budget <= 1 {
            return Err(ExecutableCarrierErrorV1::BudgetExhausted);
        }
        let continuation_record = self
            .carrier
            .carrier()
            .continuation(continuation)
            .cloned()
            .ok_or(ExecutableCarrierErrorV1::NotSuspended)?;
        let (resumption_ordinal, next_resumption_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_resumption)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (step_ordinal, next_step_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_step)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (configuration_ordinal, next_configuration_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_configuration)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let occurrence = ResumptionOccurrenceId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::Resumption,
                resumption_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let step = StepId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::Step,
                step_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let after = ConfigurationId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::Configuration,
                configuration_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let remainder = executable_configuration_term_v1(scope, &self.configuration)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        let after_budget = remaining_budget - 1;
        let resumption = ResumptionOccurrenceV2 {
            body: ResumptionOccurrenceBodyV2 {
                id: occurrence,
                continuation,
                run: self.run,
                activation: self.activation,
                pins: continuation_record.proposal().pins.clone(),
            },
            provenance: OccurrenceProvenance::EnteredThrough(EnteredThrough {
                boundary: facts.resumption_ingress.boundary,
                evidence: facts.resumption_ingress.evidence,
                permission: facts.resumption_ingress.permission,
                payload: runtime_role_term(scope, b"clause/process-resumption-v1")?,
                supports: vec![],
                causes: vec![CausalRef::Step(emitter)],
            }),
        };
        let record = StepProposalV2 {
            id: step,
            run: self.run,
            activation: self.activation,
            before: self.configuration_id,
            after: ConfigurationProposal {
                id: after,
                value: remainder,
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
            causes: vec![StepCause::ContinuationTakeup {
                continuation,
                occurrence: ContinuationTakeupOccurrence::Resumption(occurrence),
            }],
            observation_outcomes: vec![],
            candidate_delta: None,
            outcome: StepOutcomeProposalV2::Progress,
        };
        self.carrier
            .apply_ingress(&[
                ProcessRecordV2::Resumption(resumption),
                ProcessRecordV2::Steps(vec![record]),
            ])
            .map_err(ExecutableCarrierErrorV1::Ingress)?;
        let before = self.configuration_id;
        self.configuration_id = after;
        self.identity_ordinals.next_resumption = next_resumption_ordinal;
        self.identity_ordinals.next_step = next_step_ordinal;
        self.identity_ordinals.next_configuration = next_configuration_ordinal;
        self.suspended_continuation = None;
        let reference = StepRef {
            run: self.run,
            activation: self.activation,
            step,
        };
        let execution = self
            .carrier_execution
            .as_mut()
            .expect("resumption retains its execution");
        execution.remaining_budget = after_budget;
        execution.prior_step = Some(reference);
        Ok(ExecutableResumptionV1 {
            occurrence,
            step,
            continuation,
            run: self.run,
            activation: self.activation,
            before,
            after,
            remaining_budget: after_budget,
        })
    }

    /// Lower one construct-blind physical observation through the exact
    /// package-Role-indexed plan, then enter the resulting occurrence.
    pub fn advance_carrier_input(
        &mut self,
        source: &ExecutableInputSourceV1,
        scalar_value: Option<f64>,
    ) -> Result<&ExecutableStepV1, ExecutableCarrierErrorV1> {
        let mut occurrence = self
            .input
            .as_ref()
            .and_then(|input| {
                input
                    .events
                    .iter()
                    .find(|binding| &binding.source == source)
            })
            .map(|binding| binding.occurrence.clone())
            .ok_or(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::UnknownPhysicalInput,
            ))?;
        match source {
            ExecutableInputSourceV1::Keyboard { .. } if scalar_value.is_none() => {}
            ExecutableInputSourceV1::Scalar { .. } => {
                let value = scalar_value.ok_or(ExecutableCarrierErrorV1::Executable(
                    ExecutableErrorV1::MalformedInputConfiguration,
                ))?;
                occurrence.arguments = vec![
                    ExecutableValueV1::number(value)
                        .map_err(ExecutableCarrierErrorV1::Executable)?,
                ];
            }
            _ => {
                return Err(ExecutableCarrierErrorV1::Executable(
                    ExecutableErrorV1::MalformedInputConfiguration,
                ));
            }
        }
        self.advance_carrier_occurrence(occurrence)
    }

    /// Lower one fixed tick through the plan's exact tick Role and emit the
    /// candidate rooted in that Step. Milliseconds are physical timing data;
    /// only the package rule gives the resulting occurrence game meaning.
    pub fn advance_carrier_tick_and_emit_candidate(
        &mut self,
        fixed_tick_milliseconds: u32,
    ) -> Result<&ExecutableStepV1, ExecutableCarrierErrorV1> {
        if fixed_tick_milliseconds == 0 {
            return Err(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::MalformedInputConfiguration,
            ));
        }
        let entries = self
            .input
            .as_ref()
            .map(|input| input.tick.entries.clone())
            .ok_or(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::MissingInputPlan,
            ))?;
        let seconds = f64::from(fixed_tick_milliseconds) / 1_000.0;
        let argument =
            ExecutableValueV1::number(seconds).map_err(ExecutableCarrierErrorV1::Executable)?;
        let Some((last, preceding)) = entries.split_last() else {
            return Err(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::MalformedInputConfiguration,
            ));
        };
        for entry in preceding {
            self.advance_carrier_occurrence(ExecutableOccurrenceV1 {
                entry: *entry,
                arguments: vec![argument.clone()],
            })?;
        }
        self.advance_carrier_occurrence_and_emit_candidate(ExecutableOccurrenceV1 {
            entry: *last,
            arguments: vec![argument],
        })
    }

    /// Issue one exact, single-use Admission authorization occurrence under
    /// the pre-established root-governed issuer capability.
    pub fn issue_candidate_admission_authorization(
        &mut self,
    ) -> Result<IssuedAdmissionAuthorizationOccurrenceId, ExecutableCarrierErrorV1> {
        if self.issued_admission_authorization.is_some() {
            return Err(ExecutableCarrierErrorV1::AdmissionAuthorizationAlreadyIssued);
        }
        let candidate = self
            .candidate
            .as_ref()
            .ok_or(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::NoCandidate,
            ))?;
        let execution = self
            .carrier_execution
            .as_ref()
            .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
        let (ordinal, next_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_admission_authorization)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let occurrence = IssuedAdmissionAuthorizationOccurrenceId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::IssuedAdmissionAuthorization,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let issuance = IssuedStateAdmissionAuthorizationV2 {
            occurrence,
            issuer: execution.facts.admission_authorization_issuer,
            revision: execution.facts.program_revision,
            package: self.package,
            session: execution.facts.session,
            policy: execution.facts.policy,
            base: candidate.base,
            delta: candidate.id,
            provenance: EnteredThrough {
                boundary: execution.facts.admission_issuance_ingress.boundary,
                evidence: execution.facts.admission_issuance_ingress.evidence,
                permission: execution.facts.admission_issuance_ingress.permission,
                payload: runtime_role_term(
                    scope,
                    b"clause/process-issued-admission-authorization-v1",
                )?,
                supports: vec![],
                causes: vec![CausalRef::CandidateDelta(candidate.id)],
            },
        };
        self.carrier
            .apply_ingress(&[ProcessRecordV2::IssuedAdmissionAuthorization(issuance)])
            .map_err(ExecutableCarrierErrorV1::Ingress)?;
        self.identity_ordinals.next_admission_authorization = next_ordinal;
        self.issued_admission_authorization = Some(occurrence);
        Ok(occurrence)
    }

    fn advance_carrier_occurrence_inner(
        &mut self,
        occurrence: ExecutableOccurrenceV1,
        emit_candidate: bool,
    ) -> Result<&ExecutableStepV1, ExecutableCarrierErrorV1> {
        if self.suspended_continuation.is_some() {
            return Err(ExecutableCarrierErrorV1::AlreadySuspended);
        }
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
                self.allocation.root,
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
        let checker_mode = execution.checker_mode;
        let remaining_budget = execution.remaining_budget;
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let occurrence_payload = executable_occurrence_term_v1(scope, &bridge_step.occurrence)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        let mut entered_occurrence = EnteredObservationV2 {
            observation: ObservationProposalV2::Value {
                id: occurrence_id,
                value: occurrence_payload.clone(),
                supports: vec![],
            },
            provenance: EnteredThrough {
                boundary: facts.occurrence_ingress.boundary,
                evidence: facts.occurrence_ingress.evidence,
                permission: facts.occurrence_ingress.permission,
                payload: occurrence_payload,
                supports: vec![],
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
            if matches!(execution.epoch_origin, CausalRef::SessionStart(_)) {
                let trigger = ExternalTriggerOccurrenceId::from_bytes(
                    runtime_identity_bytes(
                        self.allocation.root,
                        RuntimeIdentityDomainV1::ExternalTrigger,
                        0,
                    )
                    .map_err(ExecutableCarrierErrorV1::Executable)?,
                );
                ingress.push(ProcessRecordV2::ExternalTrigger(
                    ExternalTriggerOccurrenceV2 {
                        id: trigger,
                        provenance: EnteredThrough {
                            boundary: facts.trigger_ingress.boundary,
                            evidence: facts.trigger_ingress.evidence,
                            permission: facts.trigger_ingress.permission,
                            payload: runtime_role_term(
                                scope,
                                b"clause/process-external-trigger-v1",
                            )?,
                            supports: vec![],
                            causes: vec![],
                        },
                    },
                ));
                entered_occurrence.provenance.causes = vec![CausalRef::ExternalTrigger(trigger)];
            }
            ingress.push(ProcessRecordV2::EnteredObservation(
                entered_occurrence.clone(),
            ));
            if !emit_candidate && let Some(checker_mode) = checker_mode {
                let (checker_ordinal, after_checker_ordinal) =
                    stage_runtime_ordinal(next_checker_ordinal)
                        .map_err(ExecutableCarrierErrorV1::Executable)?;
                next_checker_ordinal = after_checker_ordinal;
                let (checker_records, formation, _) = self.prepare_formation_checker(
                    checker_mode,
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
            let checker_mode = checker_mode.ok_or(ExecutableCarrierErrorV1::MissingStatefulMode)?;
            let (candidate_ordinal, next_candidate_ordinal) =
                stage_runtime_ordinal(self.identity_ordinals.next_candidate)
                    .map_err(ExecutableCarrierErrorV1::Executable)?;
            let id = CandidateDeltaId::from_bytes(
                runtime_identity_bytes(
                    self.allocation.root,
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
                checker_mode,
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
            let (prerequisite_bindings, prerequisite_occurrences) = activation_prerequisite
                .map_or_else(
                    || (Vec::new(), Vec::new()),
                    |prerequisite| (vec![prerequisite.binding], prerequisite.causes),
                );
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
                prerequisite_bindings,
                causes: ActivationCauseFrontierV2 {
                    origin: ActivationOrigin::RootedBy(runtime_root_trigger(
                        execution.epoch_origin,
                    )?),
                    prerequisite_occurrences,
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
            observation_outcomes: vec![],
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
    ) -> Result<(Vec<ExecutableSlotV1>, ExecutableStepV1), ExecutableErrorV1> {
        let mut selected = None;
        for rule in self
            .program
            .rules
            .iter()
            .filter(|rule| rule.entry == occurrence.entry)
        {
            let structural_match = rule.required_present.iter().all(|slot| {
                self.configuration
                    .get(usize::from(*slot))
                    .is_some_and(|slot| slot.value().is_some())
            }) && rule.required_absent.iter().all(|slot| {
                self.configuration
                    .get(usize::from(*slot))
                    .is_some_and(|slot| slot.value().is_none())
            });
            if !structural_match {
                continue;
            }
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
                *target = value.into();
            }
            for slot in &rule.removals {
                let target = next
                    .get_mut(usize::from(*slot))
                    .ok_or(ExecutableErrorV1::UnknownSlot(*slot))?;
                *target = ExecutableSlotV1::Absent(target.kind());
            }
        }
        let before = self.configuration_id;
        let after = ConfigurationId::from_bytes(runtime_identity_bytes(
            self.allocation.root,
            RuntimeIdentityDomainV1::Configuration,
            configuration_ordinal,
        )?);
        let step = ExecutableStepV1 {
            id: StepId::from_bytes(runtime_identity_bytes(
                self.allocation.root,
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
        before: &[ExecutableSlotV1],
        subject: &[ExecutableSlotV1],
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
                self.allocation.root,
                RuntimeIdentityDomainV1::CheckerActivation,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let checker_run = RunId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::CheckerRun,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let checker_before = ConfigurationId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::CheckerConfigurationBefore,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let checker_after = ConfigurationId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::CheckerConfigurationAfter,
                ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let formation = ObservationId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
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
                    self.allocation.root,
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
                    prerequisite_bindings: vec![],
                    causes: ActivationCauseFrontierV2 {
                        origin: activation_origin,
                        prerequisite_occurrences: vec![],
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
                    observation_outcomes: vec![StepObservationOutcomeV2::Observed(
                        ObservationProposalV2::Formation {
                            id: formation,
                            subject,
                            target,
                            supports: vec![SupportUse {
                                slot: SupportSlotId::new(0),
                                role: runtime_role_term(
                                    scope,
                                    b"clause/process-formation-support-v1",
                                )?,
                                source: support,
                            }],
                        },
                    )],
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
                self.allocation.root,
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
                permission: facts.judgment_ingress.permission,
                payload: runtime_role_term(scope, b"clause/process-admission-judgment-v1")?,
                supports: vec![],
                causes: vec![CausalRef::CandidateDelta(candidate.id)],
            }),
        };
        let admission_id = AdmissionOccurrenceId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
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
                    self.allocation.root,
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
                permission: facts.admission_ingress.permission,
                payload: successor.payload.clone(),
                supports: vec![],
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

    pub(super) fn settle_carrier_process_project_and_start_epoch(
        &mut self,
        authorization: AdmissionAuthorizationEvidence,
    ) -> Result<
        (
            ExecutableStateRevisionV1,
            Option<ExecutableProjectedObservationV1>,
        ),
        ExecutableCarrierErrorV1,
    > {
        if self.state.is_some() {
            return Err(ExecutableCarrierErrorV1::Executable(
                ExecutableErrorV1::AlreadyAdmitted,
            ));
        }
        let prepared = self.prepare_carrier_settlement(authorization)?;
        let (facts, remaining_budget) = {
            let execution = self
                .carrier_execution
                .as_ref()
                .ok_or(ExecutableCarrierErrorV1::NotStarted)?;
            (execution.facts, execution.remaining_budget)
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
                self.allocation.root,
                RuntimeIdentityDomainV1::Run,
                run_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let next_activation = ActivationId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
                RuntimeIdentityDomainV1::Activation,
                activation_ordinal,
            )
            .map_err(ExecutableCarrierErrorV1::Executable)?,
        );
        let next_configuration = ConfigurationId::from_bytes(
            runtime_identity_bytes(
                self.allocation.root,
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
        let projected =
            self.prepare_projected_observation(&prepared.executable_state, facts, scope)?;
        let mut ingress = vec![
            ProcessRecordV2::Judgment(prepared.judgment),
            ProcessRecordV2::AdmissionDecision(prepared.decision),
        ];
        if let Some((record, _, _)) = &projected {
            ingress.push(record.clone());
        }
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
        self.issued_admission_authorization = None;
        self.suspended_continuation = None;
        self.identity_ordinals.next_run = next_run_ordinal;
        self.identity_ordinals.next_activation = next_activation_ordinal;
        self.identity_ordinals.next_configuration = next_configuration_ordinal;
        if let Some((_, _, next_observation_ordinal)) = &projected {
            self.identity_ordinals.next_state_observation = *next_observation_ordinal;
        }
        let execution = self
            .carrier_execution
            .as_mut()
            .expect("persistent settlement retains its execution");
        execution.facts = next_facts;
        execution.remaining_budget = remaining_budget;
        execution.prior_step = None;
        execution.state_started = false;
        execution.epoch_origin = CausalRef::Admission(admission);
        execution.state_base_support = SupportSource::Admission(admission);
        Ok((admitted, projected.map(|(_, observation, _)| observation)))
    }

    fn prepare_projected_observation(
        &self,
        state: &ExecutableStateRevisionV1,
        facts: ExecutableAuthorityFactsV1,
        scope: TermScope,
    ) -> Result<
        Option<(ProcessRecordV2, ExecutableProjectedObservationV1, u64)>,
        ExecutableCarrierErrorV1,
    > {
        let Some(projection) = &self.program.projection else {
            return Ok(None);
        };
        let bindings = projection
            .bindings
            .iter()
            .copied()
            .map(|binding| (binding.role, binding))
            .collect::<BTreeMap<_, _>>();
        let term = realize_projection_term(&projection.template, &bindings, &state.configuration)
            .map_err(ExecutableCarrierErrorV1::Executable)?;
        let (ordinal, next_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_state_observation)
                .map_err(ExecutableCarrierErrorV1::Executable)?;
        let observation = ExecutableProjectedObservationV1 {
            id: ObservationId::from_bytes(
                runtime_identity_bytes(
                    self.allocation.root,
                    RuntimeIdentityDomainV1::StateObservation,
                    ordinal,
                )
                .map_err(ExecutableCarrierErrorV1::Executable)?,
            ),
            state: state.id,
            term,
        };
        let state_role = Term::atom(
            scope,
            b"clause/process-observed-state-v1".to_vec(),
            observation.state.as_bytes().to_vec(),
            EqualityContract::ExactOctetsV1,
        )
        .map_err(|_| ExecutableCarrierErrorV1::UnsupportedSurface)?;
        let record = ProcessRecordV2::EnteredObservation(EnteredObservationV2 {
            observation: ObservationProposalV2::Value {
                id: observation.id,
                value: observation.term.clone(),
                supports: vec![SupportUse {
                    slot: SupportSlotId::new(0),
                    role: state_role,
                    source: SupportSource::Admission(state.admission),
                }],
            },
            provenance: EnteredThrough {
                boundary: facts.occurrence_ingress.boundary,
                evidence: facts.occurrence_ingress.evidence,
                permission: facts.occurrence_ingress.permission,
                payload: observation.term.clone(),
                supports: vec![],
                causes: vec![CausalRef::Admission(state.admission)],
            },
        });
        Ok(Some((record, observation, next_ordinal)))
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
        let value = executable_configuration_term_v1(scope, &observation.value)
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
                    value: value.clone(),
                    supports: vec![SupportUse {
                        slot: SupportSlotId::new(0),
                        role: state_role,
                        source: SupportSource::Admission(admission.id),
                    }],
                },
                provenance: EnteredThrough {
                    boundary: facts.occurrence_ingress.boundary,
                    evidence: facts.occurrence_ingress.evidence,
                    permission: facts.occurrence_ingress.permission,
                    payload: value.clone(),
                    supports: vec![],
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
                self.allocation.root,
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
                self.allocation.root,
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
            self.allocation.root,
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
                self.allocation.root,
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
                    .cloned()
                    .ok_or(ExecutableErrorV1::UnknownSlot(*slot))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (ordinal, next_ordinal) =
            stage_runtime_ordinal(self.identity_ordinals.next_state_observation)?;
        let observation = ExecutableObservationV1 {
            id: ObservationId::from_bytes(runtime_identity_bytes(
                self.allocation.root,
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
    pub const fn physical_plan(&self) -> ExecutablePhysicalPlanIdV1 {
        self.physical_plan
    }

    #[must_use]
    pub const fn allocation(&self) -> RuntimeAllocationEpochV1 {
        self.allocation
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
    pub fn configuration(&self) -> &[ExecutableSlotV1] {
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

    #[must_use]
    pub fn authority_facts(&self) -> Option<ExecutableAuthorityFactsV1> {
        self.carrier_execution
            .as_ref()
            .map(|execution| execution.facts)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutableErrorV1 {
    MalformedPhysicalPlan,
    UnsupportedPhysicalTarget,
    UnsupportedPhysicalRefinement,
    PhysicalShapeMismatch,
    PhysicalModeMismatch,
    AllocationUnavailable,
    AllocationBindingMismatch,
    MalformedAllocationEpoch,
    MalformedProgram,
    MalformedOccurrence,
    MissingInputPlan,
    UnknownPhysicalInput,
    MalformedInputConfiguration,
    UnknownApplication,
    CarrierRejected,
    ResourceLimit,
    UnknownSlot(u16),
    MissingState,
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
    AdmissionAuthorizationAlreadyIssued,
    AlreadySuspended,
    NotSuspended,
    MissingCheckerPrerequisite,
    MissingFormationEvidence,
    ConstitutiveAdmissionAuthorityUnavailable,
    BudgetExhausted,
    EffectLifecycleAlreadyActive,
    UnsupportedEffectContract,
    UnknownPendingEffectIntent,
    UnknownEffectAuthorization,
    UnknownActiveEffectAttempt,
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

struct PreparedActivationPrerequisiteV1 {
    binding: DynamicPrerequisiteBindingV2,
    causes: Vec<ActivationOccurrenceCauseV2>,
}

fn activation_formation_prerequisite(
    constitution: &ResolvedProgramConstitutionV2,
    application: ApplicationId,
    mode: ModeId,
    observation: ObservationId,
) -> Result<PreparedActivationPrerequisiteV1, ExecutableCarrierErrorV1> {
    let state_contract = constitution
        .executable_contract(application, mode)
        .ok_or(ExecutableCarrierErrorV1::UnsupportedSurface)?;
    let prerequisite = state_contract
        .dynamic_prerequisites
        .iter()
        .find(|requirement| {
            requirement.requirement == ActivationPrerequisiteKind::Observation
                && requirement.cardinality.contains(1)
                && matches!(
                    requirement.scope,
                    PrerequisiteScope::SameSemantics | PrerequisiteScope::SameProgramRevision
                )
        })
        .ok_or(ExecutableCarrierErrorV1::MissingCheckerPrerequisite)?;
    let value = ActivationPrerequisite::Observation(observation);
    Ok(PreparedActivationPrerequisiteV1 {
        binding: DynamicPrerequisiteBindingV2 {
            slot: prerequisite.slot,
            ordinal: 0,
            value,
        },
        causes: prerequisite
            .cause_projection
            .iter()
            .map(|projection| ActivationOccurrenceCauseV2 {
                slot: prerequisite.slot,
                ordinal: 0,
                component: projection.component,
                occurrence: value,
            })
            .collect(),
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
        constitution: CheckedConstitutionBinding::Admitted {
            revision: facts.program_revision,
        },
        runtime_session: stateful.then_some(facts.session),
        observed_state: stateful.then_some(facts.initial_state),
        runtime_policy: stateful.then_some(facts.policy),
        context_requirements,
        constitutive_dependencies,
        capabilities: mode_record
            .contract
            .capability_requirements
            .iter()
            .map(|local| CapabilityRef {
                snapshot: constitution.snapshot(),
                local: *local,
            })
            .collect(),
        scheduling_requirements: vec![],
        resource_requirements: vec![],
        cancellation_scope: CancellationScope::Activation,
        budget: Budget {
            remaining_units: facts.budget_units,
        },
    })
}

fn validate_program(program: &ExecutableProgramV1) -> Result<(), ExecutableErrorV1> {
    let initial_configuration = materialize_initial_configuration(program)?;
    if initial_configuration.len() > MAX_PROGRAM_ITEMS || program.rules.len() > MAX_PROGRAM_ITEMS {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    for rule in &program.rules {
        if [
            rule.predicates.len(),
            rule.required_present.len(),
            rule.required_absent.len(),
            rule.assignments.len(),
            rule.removals.len(),
        ]
        .into_iter()
        .any(|count| count > MAX_PROGRAM_ITEMS)
        {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        let assignment_slots = rule
            .assignments
            .iter()
            .map(|(slot, _)| *slot)
            .collect::<Vec<_>>();
        for mut slots in [
            rule.required_present.clone(),
            rule.required_absent.clone(),
            assignment_slots.clone(),
            rule.removals.clone(),
        ] {
            slots.sort_unstable();
            if slots.windows(2).any(|pair| pair[0] == pair[1])
                || slots
                    .iter()
                    .any(|slot| usize::from(*slot) >= initial_configuration.len())
            {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
        }
        if rule
            .required_present
            .iter()
            .any(|slot| rule.required_absent.contains(slot))
            || assignment_slots
                .iter()
                .any(|slot| rule.removals.contains(slot))
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
    if let Some(projection) = &program.projection {
        if projection.bindings.is_empty() || projection.bindings.len() > MAX_PROGRAM_ITEMS {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let mut roles = BTreeSet::new();
        let mut slots = BTreeSet::new();
        let mut binding_map = BTreeMap::new();
        for binding in &projection.bindings {
            let slot = initial_configuration
                .get(usize::from(binding.slot))
                .ok_or(ExecutableErrorV1::MalformedProgram)?;
            if slot.kind() != binding.value_kind
                || !roles.insert(binding.role)
                || !slots.insert(binding.slot)
            {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            binding_map.insert(binding.role, binding.value_kind);
        }
        let mut used = BTreeSet::new();
        validate_projection_template(&projection.template, &binding_map, &mut used)?;
        if used.len() != projection.bindings.len() {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
    Ok(())
}

fn materialize_initial_configuration(
    program: &ExecutableProgramV1,
) -> Result<Vec<ExecutableSlotV1>, ExecutableErrorV1> {
    let mut configuration = program
        .initial_configuration
        .iter()
        .cloned()
        .map(ExecutableSlotV1::Present)
        .collect::<Vec<_>>();
    let Some(projection) = &program.projection else {
        return Ok(configuration);
    };
    let present = configuration.len();
    let mut absent = BTreeMap::new();
    for binding in projection
        .bindings
        .iter()
        .filter(|binding| usize::from(binding.slot) >= present)
    {
        if absent.insert(binding.slot, binding.value_kind).is_some() {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
    let Some(last) = absent.keys().next_back().copied() else {
        return Ok(configuration);
    };
    let total = usize::from(last)
        .checked_add(1)
        .ok_or(ExecutableErrorV1::ResourceLimit)?;
    if total > MAX_PROGRAM_ITEMS {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    for index in present..total {
        let slot = u16::try_from(index).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
        let kind = absent
            .get(&slot)
            .copied()
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        configuration.push(ExecutableSlotV1::Absent(kind));
    }
    Ok(configuration)
}

fn validate_input_plan_shape(
    input: Option<&ExecutableInputPlanV1>,
    program: &ExecutableProgramV1,
) -> Result<(), ExecutableErrorV1> {
    let Some(input) = input else {
        return Ok(());
    };
    if input.events.is_empty() || input.events.len() > MAX_PROGRAM_ITEMS {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let mut roles = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for binding in &input.events {
        let (designation, arguments_valid) = match &binding.source {
            ExecutableInputSourceV1::Keyboard { code, .. } => (code, true),
            ExecutableInputSourceV1::Scalar { channel } => (
                channel,
                matches!(
                    binding.occurrence.arguments.as_slice(),
                    [ExecutableValueV1::Number(_)]
                ),
            ),
        };
        if designation.is_empty()
            || designation.len() > MAX_INPUT_CODE_BYTES
            || !designation.iter().all(u8::is_ascii_graphic)
            || !arguments_valid
            || !roles.insert(binding.role)
            || !sources.insert(binding.source.clone())
            || !program
                .rules
                .iter()
                .any(|rule| rule.entry == binding.occurrence.entry)
        {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
    let entries = input.tick.entries.iter().copied().collect::<BTreeSet<_>>();
    if !roles.insert(input.tick.role)
        || input.tick.entries.is_empty()
        || input.tick.entries.len() > MAX_PROGRAM_ITEMS
        || entries.len() != input.tick.entries.len()
        || input
            .tick
            .entries
            .iter()
            .any(|entry| !program.rules.iter().any(|rule| rule.entry == *entry))
    {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    Ok(())
}

fn projection_role(
    atom: &Atom,
) -> Result<Option<(LocalRoleRefV2, ExecutableValueKindV1)>, ExecutableErrorV1> {
    if atom.kind() != PROJECTION_ROLE_KIND {
        return Ok(None);
    }
    if atom.equality_contract() != EqualityContract::ExactOctetsV1
        || atom.canonical_payload().len() != 9
    {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    let payload = atom.canonical_payload();
    let schema = RelationSchemaLocalId::new(u32::from_le_bytes(
        payload[0..4].try_into().expect("four bytes"),
    ));
    let role = RoleLocalId::new(u32::from_le_bytes(
        payload[4..8].try_into().expect("four bytes"),
    ));
    let value_kind = match payload[8] {
        0 => ExecutableValueKindV1::Number,
        1 => ExecutableValueKindV1::Boolean,
        2 => ExecutableValueKindV1::Symbol,
        3 => ExecutableValueKindV1::NumberSet,
        4 => ExecutableValueKindV1::BooleanSet,
        5 => ExecutableValueKindV1::SymbolSet,
        _ => return Err(ExecutableErrorV1::MalformedProgram),
    };
    Ok(Some((LocalRoleRefV2 { schema, role }, value_kind)))
}

fn validate_projection_template(
    term: &Term,
    bindings: &BTreeMap<LocalRoleRefV2, ExecutableValueKindV1>,
    used: &mut BTreeSet<LocalRoleRefV2>,
) -> Result<(), ExecutableErrorV1> {
    if let Some(atom) = term.as_atom() {
        if let Some((role, kind)) = projection_role(atom)? {
            if bindings.get(&role) != Some(&kind) {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            used.insert(role);
        }
        return Ok(());
    }
    let triple = term
        .as_triple()
        .ok_or(ExecutableErrorV1::MalformedProgram)?;
    for slot in triple.slots() {
        validate_projection_template(slot, bindings, used)?;
    }
    Ok(())
}

fn projected_value_term(
    scope: TermScope,
    value: ExecutableValueV1,
) -> Result<Term, ExecutableErrorV1> {
    if let ExecutableValueV1::Set(set) = &value {
        let header = projection_literal(scope, PROJECTED_SET_KIND, &[set.element_kind as u8])?;
        let end = projection_literal(scope, PROJECTED_SET_END_KIND, &[])?;
        let values = set.values.iter().collect::<Vec<_>>();
        return Term::triple([header, projected_set_tree(scope, &values)?, end])
            .map_err(|_| ExecutableErrorV1::MalformedProgram);
    }
    projected_scalar_value_term(scope, &value)
}

fn projected_scalar_value_term(
    scope: TermScope,
    value: &ExecutableValueV1,
) -> Result<Term, ExecutableErrorV1> {
    let (kind, payload) = match value {
        ExecutableValueV1::Number(bits) => (PROJECTED_NUMBER_KIND, bits.to_le_bytes().to_vec()),
        ExecutableValueV1::Boolean(value) => (PROJECTED_BOOLEAN_KIND, vec![u8::from(*value)]),
        ExecutableValueV1::Symbol(value) => (PROJECTED_SYMBOL_KIND, value.as_bytes().to_vec()),
        ExecutableValueV1::Set(_) => return Err(ExecutableErrorV1::MalformedProgram),
    };
    Term::atom(
        scope,
        kind.to_vec(),
        payload,
        EqualityContract::ExactOctetsV1,
    )
    .map_err(|_| ExecutableErrorV1::MalformedProgram)
}

fn projected_set_tree(
    scope: TermScope,
    values: &[&ExecutableValueV1],
) -> Result<Term, ExecutableErrorV1> {
    let Some((middle, left, right)) = values.get(values.len() / 2).map(|middle| {
        (
            middle,
            &values[..values.len() / 2],
            &values[values.len() / 2 + 1..],
        )
    }) else {
        return projection_literal(scope, PROJECTED_SET_END_KIND, &[]);
    };
    Term::triple([
        projected_set_tree(scope, left)?,
        projected_scalar_value_term(scope, middle)?,
        projected_set_tree(scope, right)?,
    ])
    .map_err(|_| ExecutableErrorV1::MalformedProgram)
}

fn realize_projection_term(
    template: &Term,
    bindings: &BTreeMap<LocalRoleRefV2, ExecutableProjectionBindingV1>,
    configuration: &[ExecutableSlotV1],
) -> Result<Term, ExecutableErrorV1> {
    if let Some(atom) = template.as_atom() {
        let Some((role, kind)) = projection_role(atom)? else {
            return Ok(template.clone());
        };
        let binding = bindings
            .get(&role)
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        let slot = configuration
            .get(usize::from(binding.slot))
            .ok_or(ExecutableErrorV1::UnknownSlot(binding.slot))?;
        let value = slot.value().ok_or(ExecutableErrorV1::MissingState)?;
        if binding.value_kind != kind || value.kind() != kind {
            return Err(ExecutableErrorV1::TypeMismatch);
        }
        return projected_value_term(template.scope(), value.clone());
    }
    let triple = template
        .as_triple()
        .ok_or(ExecutableErrorV1::MalformedProgram)?;
    let [left, operator, right] = triple.slots();
    if left
        .as_atom()
        .is_some_and(|atom| atom.kind() == b"clause/js-field-v1")
        && projection_subtree_has_role(operator)?
        && !projection_subtree_has_present_role(operator, bindings, configuration)?
    {
        return realize_projection_term(right, bindings, configuration);
    }
    Term::triple([
        realize_projection_term(left, bindings, configuration)?,
        realize_projection_term(operator, bindings, configuration)?,
        realize_projection_term(right, bindings, configuration)?,
    ])
    .map_err(|_| ExecutableErrorV1::MalformedProgram)
}

fn projection_subtree_has_role(term: &Term) -> Result<bool, ExecutableErrorV1> {
    if let Some(atom) = term.as_atom() {
        return Ok(projection_role(atom)?.is_some());
    }
    let triple = term
        .as_triple()
        .ok_or(ExecutableErrorV1::MalformedProgram)?;
    for child in triple.slots() {
        if projection_subtree_has_role(child)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn projection_subtree_has_present_role(
    term: &Term,
    bindings: &BTreeMap<LocalRoleRefV2, ExecutableProjectionBindingV1>,
    configuration: &[ExecutableSlotV1],
) -> Result<bool, ExecutableErrorV1> {
    if let Some(atom) = term.as_atom() {
        let Some((role, _)) = projection_role(atom)? else {
            return Ok(false);
        };
        let binding = bindings
            .get(&role)
            .ok_or(ExecutableErrorV1::MalformedProgram)?;
        return Ok(configuration
            .get(usize::from(binding.slot))
            .is_some_and(|slot| slot.value().is_some()));
    }
    let triple = term
        .as_triple()
        .ok_or(ExecutableErrorV1::MalformedProgram)?;
    for child in triple.slots() {
        if projection_subtree_has_present_role(child, bindings, configuration)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn evaluate(
    expression: &ExecutableExpressionV1,
    slots: &[ExecutableSlotV1],
    arguments: &[ExecutableValueV1],
) -> Result<ExecutableValueV1, ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    match expression {
        E::Constant(value) => Ok(value.clone()),
        E::Slot(slot) => slots
            .get(usize::from(*slot))
            .ok_or(ExecutableErrorV1::UnknownSlot(*slot))?
            .value()
            .cloned()
            .ok_or(ExecutableErrorV1::MissingState),
        E::Argument(argument) => arguments
            .get(usize::from(*argument))
            .cloned()
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
        E::SetInsert(set, value) => {
            let set = evaluate(set, slots, arguments)?;
            let value = evaluate(value, slots, arguments)?;
            let ExecutableValueV1::Set(set) = set else {
                return Err(ExecutableErrorV1::TypeMismatch);
            };
            Ok(ExecutableValueV1::Set(set.inserted(value)?))
        }
        E::SetContains(set, value) => {
            let set = evaluate(set, slots, arguments)?;
            let value = evaluate(value, slots, arguments)?;
            let ExecutableValueV1::Set(set) = set else {
                return Err(ExecutableErrorV1::TypeMismatch);
            };
            Ok(ExecutableValueV1::Boolean(set.contains(&value)?))
        }
        E::SetRemove(set, value) => {
            let set = evaluate(set, slots, arguments)?;
            let value = evaluate(value, slots, arguments)?;
            let ExecutableValueV1::Set(set) = set else {
                return Err(ExecutableErrorV1::TypeMismatch);
            };
            Ok(ExecutableValueV1::Set(set.removed(&value)?))
        }
    }
}

fn numeric2(
    left: &ExecutableExpressionV1,
    right: &ExecutableExpressionV1,
    slots: &[ExecutableSlotV1],
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
    slots: &[ExecutableSlotV1],
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
    allocation_root: [u8; IDENTITY_BYTES],
    domain: RuntimeIdentityDomainV1,
    ordinal: u64,
) -> Result<[u8; clause_package::IDENTITY_BYTES], ExecutableErrorV1> {
    let domain = (domain as u64).to_be_bytes();
    let ordinal = ordinal.to_be_bytes();
    Ok(runtime_domain_hash(
        "clause/runtime-identity/v1",
        &[&allocation_root, &domain, &ordinal],
    ))
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
        encode_value(bytes, value)?;
    }
    Ok(())
}

fn encode_slots(bytes: &mut Vec<u8>, slots: &[ExecutableSlotV1]) -> Result<(), ExecutableErrorV1> {
    encode_count(bytes, slots.len())?;
    for slot in slots {
        match slot {
            ExecutableSlotV1::Absent(kind) => {
                bytes.extend_from_slice(&[0, *kind as u8]);
            }
            ExecutableSlotV1::Present(value) => {
                bytes.push(1);
                encode_value(bytes, value)?;
            }
        }
    }
    Ok(())
}

fn encode_projection(
    bytes: &mut Vec<u8>,
    projection: Option<&ExecutableProjectionV1>,
) -> Result<(), ExecutableErrorV1> {
    let Some(projection) = projection else {
        bytes.push(0);
        return Ok(());
    };
    bytes.push(1);
    encode_count(bytes, projection.bindings.len())?;
    for binding in &projection.bindings {
        bytes.extend_from_slice(&binding.role.schema.get().to_le_bytes());
        bytes.extend_from_slice(&binding.role.role.get().to_le_bytes());
        bytes.extend_from_slice(&binding.slot.to_le_bytes());
        bytes.push(binding.value_kind as u8);
    }
    let template = canonical_term_bytes(&projection.template)
        .map_err(|_| ExecutableErrorV1::MalformedProgram)?;
    encode_count(bytes, template.len())?;
    bytes.extend_from_slice(&template);
    Ok(())
}

fn decode_projection(
    decoder: &mut Decoder<'_>,
) -> Result<Option<ExecutableProjectionV1>, ExecutableErrorV1> {
    match decoder.byte()? {
        0 => Ok(None),
        1 => {
            let count = decoder.count()?;
            let mut bindings = Vec::with_capacity(count);
            for _ in 0..count {
                let role = LocalRoleRefV2 {
                    schema: RelationSchemaLocalId::new(decoder.u32()?),
                    role: RoleLocalId::new(decoder.u32()?),
                };
                let slot = decoder.u16()?;
                let value_kind = match decoder.byte()? {
                    0 => ExecutableValueKindV1::Number,
                    1 => ExecutableValueKindV1::Boolean,
                    2 => ExecutableValueKindV1::Symbol,
                    3 => ExecutableValueKindV1::NumberSet,
                    4 => ExecutableValueKindV1::BooleanSet,
                    5 => ExecutableValueKindV1::SymbolSet,
                    _ => return Err(ExecutableErrorV1::MalformedProgram),
                };
                bindings.push(ExecutableProjectionBindingV1 {
                    role,
                    slot,
                    value_kind,
                });
            }
            let length = decoder.count()?;
            let template = decode_canonical_term_bytes(decoder.take(length)?)
                .map_err(|_| ExecutableErrorV1::MalformedProgram)?;
            Ok(Some(ExecutableProjectionV1 { bindings, template }))
        }
        _ => Err(ExecutableErrorV1::MalformedProgram),
    }
}

fn encode_value(bytes: &mut Vec<u8>, value: &ExecutableValueV1) -> Result<(), ExecutableErrorV1> {
    match value {
        ExecutableValueV1::Number(bits) => {
            bytes.push(0);
            bytes.extend_from_slice(&bits.to_le_bytes());
        }
        ExecutableValueV1::Boolean(value) => {
            bytes.extend_from_slice(&[1, u8::from(*value)]);
        }
        ExecutableValueV1::Symbol(value) => {
            bytes.push(2);
            bytes.push(value.length);
            bytes.extend_from_slice(value.as_bytes());
        }
        ExecutableValueV1::Set(set) => {
            bytes.push(3);
            bytes.push(set.element_kind as u8);
            encode_count(bytes, set.values.len())?;
            for value in &set.values {
                encode_value(bytes, value)?;
            }
        }
    }
    Ok(())
}

fn encode_expression(
    bytes: &mut Vec<u8>,
    expression: &ExecutableExpressionV1,
) -> Result<(), ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    match expression {
        E::Constant(value) => {
            bytes.push(0);
            encode_value(bytes, value)?;
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
        E::SetInsert(set, value) => encode_binary(bytes, 13, set, value)?,
        E::SetContains(set, value) => encode_binary(bytes, 14, set, value)?,
        E::SetRemove(set, value) => encode_binary(bytes, 15, set, value)?,
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
    fn u32(&mut self) -> Result<u32, ExecutableErrorV1> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }
    fn u64(&mut self) -> Result<u64, ExecutableErrorV1> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight bytes"),
        ))
    }
    fn identity(&mut self) -> Result<[u8; IDENTITY_BYTES], ExecutableErrorV1> {
        self.take(IDENTITY_BYTES)?
            .try_into()
            .map_err(|_| ExecutableErrorV1::MalformedPhysicalPlan)
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
            2 => {
                let length = usize::from(self.byte()?);
                ExecutableValueV1::symbol(self.take(length)?)
            }
            3 => {
                let element_kind = match self.byte()? {
                    0 => ExecutableValueKindV1::Number,
                    1 => ExecutableValueKindV1::Boolean,
                    2 => ExecutableValueKindV1::Symbol,
                    _ => return Err(ExecutableErrorV1::MalformedProgram),
                };
                let count = self.count()?;
                let mut set = ExecutableSetV1::empty(element_kind)?;
                for _ in 0..count {
                    set = set.inserted(self.value()?)?;
                }
                Ok(ExecutableValueV1::Set(set))
            }
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
            13 => E::SetInsert(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            14 => E::SetContains(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            15 => E::SetRemove(
                Box::new(self.expression(next)?),
                Box::new(self.expression(next)?),
            ),
            _ => return Err(ExecutableErrorV1::MalformedProgram),
        })
    }
    fn is_complete(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
