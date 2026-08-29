//! Construct-blind evaluator and deterministic certificate producer for the
//! twelve fixed CLCP-v2 `KExpr` forms.

use std::collections::BTreeMap;
use std::fmt;

use crate::compiler_package_v2::{
    Definition, EvalCertificate, EvalJudgment, EvalNode, EvalOutcome, EvalStatement, Hash32, Id32,
    KExpr, KSort, KValue, MAX_NESTING_DEPTH, Term, sha256_operation_id,
};
use crate::physical::{ObservationLog, PhysicalError, SealedPhysical};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticError {
    DefinitionsNotStrictlySorted { index: usize },
    DuplicateDefinition(Id32),
    VariableOutOfBounds(u32),
    SortMismatch { expected: KSort, actual: KSort },
    BranchSortMismatch { left: KSort, right: KSort },
    UnknownDefinition(Id32),
    ArgumentCount { expected: usize, actual: usize },
    OperationOutsideSealedProfile(Id32),
    RecursionLimit,
}

impl fmt::Display for StaticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DefinitionsNotStrictlySorted { index } => {
                write!(
                    formatter,
                    "definitions are not strictly sorted at index {index}"
                )
            }
            Self::DuplicateDefinition(_) => formatter.write_str("duplicate definition ID"),
            Self::VariableOutOfBounds(index) => {
                write!(formatter, "Var({index}) is outside the environment")
            }
            Self::SortMismatch { expected, actual } => {
                write!(formatter, "expected {expected:?}, found {actual:?}")
            }
            Self::BranchSortMismatch { left, right } => {
                write!(formatter, "branch sorts differ: {left:?} and {right:?}")
            }
            Self::UnknownDefinition(_) => formatter.write_str("definition ID is unresolved"),
            Self::ArgumentCount { expected, actual } => {
                write!(formatter, "expected {expected} arguments, found {actual}")
            }
            Self::OperationOutsideSealedProfile(_) => {
                formatter.write_str("physical operation is outside the sealed profile")
            }
            Self::RecursionLimit => {
                formatter.write_str("static checking exhausted its depth budget")
            }
        }
    }
}

impl std::error::Error for StaticError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalError {
    Static(StaticError),
    OutOfFuel,
    VariableOutOfBounds(u32),
    ValueSort { expected: KSort, actual: KSort },
    UnknownDefinition(Id32),
    ArgumentCount { expected: usize, actual: usize },
    Physical(PhysicalError),
    ByteLengthOverflow,
    CertificateNodeOverflow,
    RecursionLimit,
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(error) => write!(formatter, "static check failed: {error}"),
            Self::OutOfFuel => {
                formatter.write_str("evaluation has no successful out-of-fuel judgment")
            }
            Self::VariableOutOfBounds(index) => {
                write!(formatter, "Var({index}) is outside the runtime environment")
            }
            Self::ValueSort { expected, actual } => {
                write!(formatter, "expected {expected:?} value, found {actual:?}")
            }
            Self::UnknownDefinition(_) => formatter.write_str("definition ID is unresolved"),
            Self::ArgumentCount { expected, actual } => {
                write!(formatter, "expected {expected} arguments, found {actual}")
            }
            Self::Physical(error) => write!(formatter, "physical request failed: {error}"),
            Self::ByteLengthOverflow => formatter.write_str("byte concatenation length overflow"),
            Self::CertificateNodeOverflow => {
                formatter.write_str("certificate node index exceeds U32")
            }
            Self::RecursionLimit => formatter.write_str("evaluation exhausted its depth budget"),
        }
    }
}

impl std::error::Error for EvalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Static(error) => Some(error),
            Self::Physical(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StaticError> for EvalError {
    fn from(value: StaticError) -> Self {
        Self::Static(value)
    }
}

impl From<PhysicalError> for EvalError {
    fn from(value: PhysicalError) -> Self {
        Self::Physical(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub value: KValue,
    pub remaining_fuel: u64,
    pub observations: ObservationLog,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateContext {
    pub exact_accepted_predecessor: Vec<u8>,
    pub core_contract_id: Hash32,
    pub physical_profile_id: Hash32,
    pub entrypoint: Id32,
    pub arguments: Vec<KValue>,
    pub fuel_limit: u64,
}

struct DefinitionTable<'a> {
    ordered: &'a [Definition],
    by_id: BTreeMap<Id32, &'a Definition>,
}

impl<'a> DefinitionTable<'a> {
    fn new(definitions: &'a [Definition]) -> Result<Self, StaticError> {
        for (index, pair) in definitions.windows(2).enumerate() {
            if pair[0].id == pair[1].id {
                return Err(StaticError::DuplicateDefinition(pair[0].id));
            }
            if pair[0].id > pair[1].id {
                return Err(StaticError::DefinitionsNotStrictlySorted { index: index + 1 });
            }
        }
        let by_id = definitions
            .iter()
            .map(|definition| (definition.id, definition))
            .collect();
        Ok(Self {
            ordered: definitions,
            by_id,
        })
    }

    fn resolve(&self, id: Id32) -> Option<&'a Definition> {
        self.by_id.get(&id).copied()
    }
}

pub struct Evaluator<'a> {
    definitions: DefinitionTable<'a>,
    physical: SealedPhysical,
}

impl<'a> Evaluator<'a> {
    pub fn new(definitions: &'a [Definition]) -> Result<Self, StaticError> {
        let definitions = DefinitionTable::new(definitions)?;
        let evaluator = Self {
            definitions,
            physical: SealedPhysical::new(),
        };
        evaluator.check_definitions()?;
        Ok(evaluator)
    }

    pub fn check_definitions(&self) -> Result<(), StaticError> {
        for definition in self.definitions.ordered {
            let actual = self.infer(&definition.body, &definition.arguments, 0)?;
            require_sort(definition.result, actual)?;
        }
        Ok(())
    }

    pub fn infer_sort(
        &self,
        expression: &KExpr,
        environment: &[KSort],
    ) -> Result<KSort, StaticError> {
        self.infer(expression, environment, 0)
    }

    pub fn evaluate(
        &self,
        expression: &KExpr,
        environment: &[KValue],
        fuel: u64,
    ) -> Result<Evaluation, EvalError> {
        let sorts: Vec<KSort> = environment.iter().map(KValue::sort).collect();
        self.infer_sort(expression, &sorts)?;
        let mut nodes = Vec::new();
        let state = State {
            fuel,
            observations: ObservationLog::default(),
        };
        let result = self.step(expression, environment, state, &mut nodes, 0)?;
        Ok(Evaluation {
            value: result.value,
            remaining_fuel: result.state.fuel,
            observations: result.state.observations,
        })
    }

    /// Evaluate one generic entrypoint invocation and produce its canonical
    /// postorder rule DAG. This constructs evidence only; it grants no package
    /// or predecessor authority.
    pub fn build_certificate(
        &self,
        context: CertificateContext,
    ) -> Result<EvalCertificate, EvalError> {
        let expression = KExpr::Call {
            definition_id: context.entrypoint,
            arguments: context.arguments.iter().map(value_literal).collect(),
        };
        let mut nodes = Vec::new();
        let state = State {
            fuel: context.fuel_limit,
            observations: ObservationLog::default(),
        };
        let result = self.step(&expression, &[], state, &mut nodes, 0)?;
        let observations = result.state.observations.to_term();
        let statement = EvalStatement {
            exact_accepted_predecessor: context.exact_accepted_predecessor,
            core_contract_id: context.core_contract_id,
            physical_profile_id: context.physical_profile_id,
            entrypoint: context.entrypoint,
            arguments: context.arguments,
            fuel_limit: context.fuel_limit,
            expected: EvalOutcome::Returned {
                value: result.value,
                remaining_fuel: result.state.fuel,
                observations,
            },
        };
        Ok(EvalCertificate {
            format_version: 0x00,
            statement,
            nodes,
        })
    }

    fn infer(
        &self,
        expression: &KExpr,
        environment: &[KSort],
        current_depth: usize,
    ) -> Result<KSort, StaticError> {
        if current_depth >= MAX_NESTING_DEPTH {
            return Err(StaticError::RecursionLimit);
        }
        let next = current_depth + 1;
        match expression {
            KExpr::BytesLiteral(_) => Ok(KSort::Bytes),
            KExpr::TermLiteral(_) => Ok(KSort::Term),
            KExpr::Var(index) => environment
                .get(
                    usize::try_from(*index)
                        .map_err(|_| StaticError::VariableOutOfBounds(*index))?,
                )
                .copied()
                .ok_or(StaticError::VariableOutOfBounds(*index)),
            KExpr::MakeAtom {
                kind,
                payload,
                equality,
            } => {
                require_sort(KSort::Bytes, self.infer(kind, environment, next)?)?;
                require_sort(KSort::Bytes, self.infer(payload, environment, next)?)?;
                require_sort(KSort::Bytes, self.infer(equality, environment, next)?)?;
                Ok(KSort::Term)
            }
            KExpr::MakeTriple {
                first,
                second,
                third,
            } => {
                require_sort(KSort::Term, self.infer(first, environment, next)?)?;
                require_sort(KSort::Term, self.infer(second, environment, next)?)?;
                require_sort(KSort::Term, self.infer(third, environment, next)?)?;
                Ok(KSort::Term)
            }
            KExpr::Let { value, body } => {
                let value_sort = self.infer(value, environment, next)?;
                let body_environment = prepend(&[value_sort], environment);
                self.infer(body, &body_environment, next)
            }
            KExpr::CaseTerm {
                scrutinee,
                atom_body,
                triple_body,
            } => {
                require_sort(KSort::Term, self.infer(scrutinee, environment, next)?)?;
                let atom_environment =
                    prepend(&[KSort::Bytes, KSort::Bytes, KSort::Bytes], environment);
                let triple_environment =
                    prepend(&[KSort::Term, KSort::Term, KSort::Term], environment);
                common_sort(
                    self.infer(atom_body, &atom_environment, next)?,
                    self.infer(triple_body, &triple_environment, next)?,
                )
            }
            KExpr::CaseBytes {
                scrutinee,
                empty_body,
                cons_body,
            } => {
                require_sort(KSort::Bytes, self.infer(scrutinee, environment, next)?)?;
                let cons_environment = prepend(&[KSort::Bytes, KSort::Bytes], environment);
                common_sort(
                    self.infer(empty_body, environment, next)?,
                    self.infer(cons_body, &cons_environment, next)?,
                )
            }
            KExpr::ConcatBytes(parts) => {
                for part in parts {
                    require_sort(KSort::Bytes, self.infer(part, environment, next)?)?;
                }
                Ok(KSort::Bytes)
            }
            KExpr::CaseBytesEqual {
                left,
                right,
                equal_body,
                unequal_body,
            } => {
                require_sort(KSort::Bytes, self.infer(left, environment, next)?)?;
                require_sort(KSort::Bytes, self.infer(right, environment, next)?)?;
                common_sort(
                    self.infer(equal_body, environment, next)?,
                    self.infer(unequal_body, environment, next)?,
                )
            }
            KExpr::Call {
                definition_id,
                arguments,
            } => {
                let definition = self
                    .definitions
                    .resolve(*definition_id)
                    .ok_or(StaticError::UnknownDefinition(*definition_id))?;
                if definition.arguments.len() != arguments.len() {
                    return Err(StaticError::ArgumentCount {
                        expected: definition.arguments.len(),
                        actual: arguments.len(),
                    });
                }
                for (argument, expected) in arguments.iter().zip(&definition.arguments) {
                    require_sort(*expected, self.infer(argument, environment, next)?)?;
                }
                Ok(definition.result)
            }
            KExpr::Request {
                physical_operation_id,
                arguments,
            } => {
                if *physical_operation_id != sha256_operation_id() {
                    return Err(StaticError::OperationOutsideSealedProfile(
                        *physical_operation_id,
                    ));
                }
                if arguments.len() != 1 {
                    return Err(StaticError::ArgumentCount {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                require_sort(KSort::Bytes, self.infer(&arguments[0], environment, next)?)?;
                Ok(KSort::Bytes)
            }
        }
    }

    fn step(
        &self,
        expression: &KExpr,
        environment: &[KValue],
        mut state: State,
        nodes: &mut Vec<EvalNode>,
        current_depth: usize,
    ) -> Result<StepResult, EvalError> {
        if current_depth >= MAX_NESTING_DEPTH {
            return Err(EvalError::RecursionLimit);
        }
        let next = current_depth + 1;
        let fuel_before = state.fuel;
        let observations_before = state.observations.to_term();
        state.fuel = state.fuel.checked_sub(1).ok_or(EvalError::OutOfFuel)?;
        let mut premises = Vec::new();

        let (rule_tag, value, final_state) = match expression {
            KExpr::BytesLiteral(bytes) => (0x30, KValue::Bytes(bytes.clone()), state),
            KExpr::TermLiteral(term) => (0x31, KValue::Term(term.clone()), state),
            KExpr::Var(index) => {
                let value = environment
                    .get(
                        usize::try_from(*index)
                            .map_err(|_| EvalError::VariableOutOfBounds(*index))?,
                    )
                    .cloned()
                    .ok_or(EvalError::VariableOutOfBounds(*index))?;
                (0x32, value, state)
            }
            KExpr::MakeAtom {
                kind,
                payload,
                equality,
            } => {
                let kind_result =
                    self.child(kind, environment, state, nodes, next, &mut premises)?;
                let kind = expect_bytes(kind_result.value)?;
                let payload_result = self.child(
                    payload,
                    environment,
                    kind_result.state,
                    nodes,
                    next,
                    &mut premises,
                )?;
                let payload = expect_bytes(payload_result.value)?;
                let equality_result = self.child(
                    equality,
                    environment,
                    payload_result.state,
                    nodes,
                    next,
                    &mut premises,
                )?;
                let equality = expect_bytes(equality_result.value)?;
                (
                    0x33,
                    KValue::Term(Term::Atom {
                        kind,
                        canonical_payload: payload,
                        equality_contract: equality,
                    }),
                    equality_result.state,
                )
            }
            KExpr::MakeTriple {
                first,
                second,
                third,
            } => {
                let first_result =
                    self.child(first, environment, state, nodes, next, &mut premises)?;
                let first = expect_term(first_result.value)?;
                let second_result = self.child(
                    second,
                    environment,
                    first_result.state,
                    nodes,
                    next,
                    &mut premises,
                )?;
                let second = expect_term(second_result.value)?;
                let third_result = self.child(
                    third,
                    environment,
                    second_result.state,
                    nodes,
                    next,
                    &mut premises,
                )?;
                let third = expect_term(third_result.value)?;
                (
                    0x34,
                    KValue::Term(Term::Triple(
                        Box::new(first),
                        Box::new(second),
                        Box::new(third),
                    )),
                    third_result.state,
                )
            }
            KExpr::Let { value, body } => {
                let value_result =
                    self.child(value, environment, state, nodes, next, &mut premises)?;
                let body_environment = prepend(&[value_result.value], environment);
                let body_result = self.child(
                    body,
                    &body_environment,
                    value_result.state,
                    nodes,
                    next,
                    &mut premises,
                )?;
                (0x35, body_result.value, body_result.state)
            }
            KExpr::CaseTerm {
                scrutinee,
                atom_body,
                triple_body,
            } => {
                let scrutinee_result =
                    self.child(scrutinee, environment, state, nodes, next, &mut premises)?;
                let term = expect_term(scrutinee_result.value)?;
                match term {
                    Term::Atom {
                        kind,
                        canonical_payload,
                        equality_contract,
                    } => {
                        let branch_environment = prepend(
                            &[
                                KValue::Bytes(kind),
                                KValue::Bytes(canonical_payload),
                                KValue::Bytes(equality_contract),
                            ],
                            environment,
                        );
                        let branch = self.child(
                            atom_body,
                            &branch_environment,
                            scrutinee_result.state,
                            nodes,
                            next,
                            &mut premises,
                        )?;
                        (0x36, branch.value, branch.state)
                    }
                    Term::Triple(first, second, third) => {
                        let branch_environment = prepend(
                            &[
                                KValue::Term(*first),
                                KValue::Term(*second),
                                KValue::Term(*third),
                            ],
                            environment,
                        );
                        let branch = self.child(
                            triple_body,
                            &branch_environment,
                            scrutinee_result.state,
                            nodes,
                            next,
                            &mut premises,
                        )?;
                        (0x37, branch.value, branch.state)
                    }
                }
            }
            KExpr::CaseBytes {
                scrutinee,
                empty_body,
                cons_body,
            } => {
                let scrutinee_result =
                    self.child(scrutinee, environment, state, nodes, next, &mut premises)?;
                let bytes = expect_bytes(scrutinee_result.value)?;
                if let Some((&head, tail)) = bytes.split_first() {
                    let branch_environment = prepend(
                        &[KValue::Bytes(vec![head]), KValue::Bytes(tail.to_vec())],
                        environment,
                    );
                    let branch = self.child(
                        cons_body,
                        &branch_environment,
                        scrutinee_result.state,
                        nodes,
                        next,
                        &mut premises,
                    )?;
                    (0x39, branch.value, branch.state)
                } else {
                    let branch = self.child(
                        empty_body,
                        environment,
                        scrutinee_result.state,
                        nodes,
                        next,
                        &mut premises,
                    )?;
                    (0x38, branch.value, branch.state)
                }
            }
            KExpr::ConcatBytes(parts) => {
                let mut bytes = Vec::new();
                let mut next_state = state;
                for part in parts {
                    let part_result =
                        self.child(part, environment, next_state, nodes, next, &mut premises)?;
                    let part = expect_bytes(part_result.value)?;
                    bytes
                        .len()
                        .checked_add(part.len())
                        .ok_or(EvalError::ByteLengthOverflow)?;
                    bytes.extend_from_slice(&part);
                    next_state = part_result.state;
                }
                (0x3a, KValue::Bytes(bytes), next_state)
            }
            KExpr::CaseBytesEqual {
                left,
                right,
                equal_body,
                unequal_body,
            } => {
                let left_result =
                    self.child(left, environment, state, nodes, next, &mut premises)?;
                let left = expect_bytes(left_result.value)?;
                let right_result = self.child(
                    right,
                    environment,
                    left_result.state,
                    nodes,
                    next,
                    &mut premises,
                )?;
                let right = expect_bytes(right_result.value)?;
                let (tag, selected) = if left == right {
                    (0x3b, equal_body.as_ref())
                } else {
                    (0x3c, unequal_body.as_ref())
                };
                let branch = self.child(
                    selected,
                    environment,
                    right_result.state,
                    nodes,
                    next,
                    &mut premises,
                )?;
                (tag, branch.value, branch.state)
            }
            KExpr::Call {
                definition_id,
                arguments,
            } => {
                let definition = self
                    .definitions
                    .resolve(*definition_id)
                    .ok_or(EvalError::UnknownDefinition(*definition_id))?;
                if definition.arguments.len() != arguments.len() {
                    return Err(EvalError::ArgumentCount {
                        expected: definition.arguments.len(),
                        actual: arguments.len(),
                    });
                }
                let mut values = Vec::with_capacity(arguments.len());
                let mut next_state = state;
                for (argument, expected) in arguments.iter().zip(&definition.arguments) {
                    let argument_result = self.child(
                        argument,
                        environment,
                        next_state,
                        nodes,
                        next,
                        &mut premises,
                    )?;
                    require_value_sort(*expected, &argument_result.value)?;
                    values.push(argument_result.value);
                    next_state = argument_result.state;
                }
                let body = self.child(
                    &definition.body,
                    &values,
                    next_state,
                    nodes,
                    next,
                    &mut premises,
                )?;
                require_value_sort(definition.result, &body.value)?;
                (0x3d, body.value, body.state)
            }
            KExpr::Request {
                physical_operation_id,
                arguments,
            } => {
                if arguments.len() != 1 {
                    return Err(EvalError::ArgumentCount {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                let argument = self.child(
                    &arguments[0],
                    environment,
                    state,
                    nodes,
                    next,
                    &mut premises,
                )?;
                require_value_sort(KSort::Bytes, &argument.value)?;
                let mut final_state = argument.state;
                let value = self.physical.request(
                    *physical_operation_id,
                    &[argument.value],
                    &mut final_state.observations,
                )?;
                (0x3e, value, final_state)
            }
        };

        let conclusion = EvalJudgment {
            expression: expression.clone(),
            environment: environment.to_vec(),
            fuel_before,
            observations_before,
            value: value.clone(),
            fuel_after: final_state.fuel,
            observations_after: final_state.observations.to_term(),
        };
        nodes
            .try_reserve(1)
            .map_err(|_| EvalError::CertificateNodeOverflow)?;
        nodes.push(EvalNode {
            rule_tag,
            premises,
            conclusion,
        });
        Ok(StepResult {
            value,
            state: final_state,
        })
    }

    fn child(
        &self,
        expression: &KExpr,
        environment: &[KValue],
        state: State,
        nodes: &mut Vec<EvalNode>,
        depth: usize,
        premises: &mut Vec<u32>,
    ) -> Result<StepResult, EvalError> {
        let result = self.step(expression, environment, state, nodes, depth)?;
        let index =
            u32::try_from(nodes.len() - 1).map_err(|_| EvalError::CertificateNodeOverflow)?;
        premises.push(index);
        Ok(result)
    }
}

#[derive(Clone)]
struct State {
    fuel: u64,
    observations: ObservationLog,
}

struct StepResult {
    value: KValue,
    state: State,
}

fn require_sort(expected: KSort, actual: KSort) -> Result<(), StaticError> {
    if expected == actual {
        Ok(())
    } else {
        Err(StaticError::SortMismatch { expected, actual })
    }
}

fn common_sort(left: KSort, right: KSort) -> Result<KSort, StaticError> {
    if left == right {
        Ok(left)
    } else {
        Err(StaticError::BranchSortMismatch { left, right })
    }
}

fn require_value_sort(expected: KSort, value: &KValue) -> Result<(), EvalError> {
    let actual = value.sort();
    if actual == expected {
        Ok(())
    } else {
        Err(EvalError::ValueSort { expected, actual })
    }
}

fn expect_bytes(value: KValue) -> Result<Vec<u8>, EvalError> {
    match value {
        KValue::Bytes(value) => Ok(value),
        KValue::Term(_) => Err(EvalError::ValueSort {
            expected: KSort::Bytes,
            actual: KSort::Term,
        }),
    }
}

fn expect_term(value: KValue) -> Result<Term, EvalError> {
    match value {
        KValue::Term(value) => Ok(value),
        KValue::Bytes(_) => Err(EvalError::ValueSort {
            expected: KSort::Term,
            actual: KSort::Bytes,
        }),
    }
}

fn prepend<T: Clone>(prefix: &[T], tail: &[T]) -> Vec<T> {
    let mut values = Vec::with_capacity(prefix.len() + tail.len());
    values.extend_from_slice(prefix);
    values.extend_from_slice(tail);
    values
}

fn value_literal(value: &KValue) -> KExpr {
    match value {
        KValue::Bytes(bytes) => KExpr::BytesLiteral(bytes.clone()),
        KValue::Term(term) => KExpr::TermLiteral(term.clone()),
    }
}
