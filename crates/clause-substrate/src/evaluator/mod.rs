//! Construct-blind evaluator and compact receipt producer for the twelve fixed
//! CLCP-v3 `KExpr` forms.

use std::fmt;
use std::ops::Range;

use crate::compiler_package_v3::{
    Definition, EvalReceipt, Id32, KExpr, KSort, KValue, MAX_EVALUATION_FRAMES,
    MAX_EXPRESSION_DEPTH, MAX_EXPRESSION_NODES, MAX_RUNTIME_ENVIRONMENTS, MAX_WIRE_BYTES,
    MAX_WIRE_ITEMS, Term, eval_receipt_observations_hash, eval_receipt_value_hash,
    sha256_operation_id, try_copy_bytes,
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
    ResourceExhausted,
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
                formatter.write_str("static checking exhausted its expression-depth budget")
            }
            Self::ResourceExhausted => {
                formatter.write_str("static checking exhausted physical resources")
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
    RecursionLimit,
    ResourceExhausted,
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
            Self::RecursionLimit => formatter.write_str("evaluation machine stack was exhausted"),
            Self::ResourceExhausted => {
                formatter.write_str("evaluation exhausted physical resources")
            }
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

#[derive(Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub value: KValue,
    pub remaining_fuel: u64,
    pub observations: ObservationLog,
}

struct DefinitionTable<'a> {
    ordered: &'a [Definition],
}

impl<'a> DefinitionTable<'a> {
    fn new(definitions: &'a [Definition]) -> Result<Self, StaticError> {
        if definitions.len() > MAX_WIRE_ITEMS {
            return Err(StaticError::ResourceExhausted);
        }
        for (index, pair) in definitions.windows(2).enumerate() {
            let [previous, current] = pair else {
                return Err(StaticError::ResourceExhausted);
            };
            if previous.id == current.id {
                return Err(StaticError::DuplicateDefinition(previous.id));
            }
            if previous.id > current.id {
                let index = index.checked_add(1).ok_or(StaticError::ResourceExhausted)?;
                return Err(StaticError::DefinitionsNotStrictlySorted { index });
            }
        }
        Ok(Self {
            ordered: definitions,
        })
    }

    fn resolve(&self, id: Id32) -> Option<&'a Definition> {
        self.ordered
            .binary_search_by_key(&id, |definition| definition.id)
            .ok()
            .and_then(|index| self.ordered.get(index))
    }
}

pub struct Evaluator<'a> {
    definitions: DefinitionTable<'a>,
    physical: SealedPhysical,
}

impl<'a> Evaluator<'a> {
    pub fn new(definitions: &'a [Definition]) -> Result<Self, StaticError> {
        let evaluator = Self::new_unprofiled(definitions)?;
        evaluator.check_physical_profile()?;
        Ok(evaluator)
    }

    pub(crate) fn new_unprofiled(definitions: &'a [Definition]) -> Result<Self, StaticError> {
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
            if definition.arguments.len() > MAX_WIRE_ITEMS {
                return Err(StaticError::ResourceExhausted);
            }
            definition
                .body
                .validate_resource_bounds()
                .map_err(|_| StaticError::ResourceExhausted)?;
            let actual = self.infer(&definition.body, &definition.arguments, false)?;
            require_sort(definition.result, actual)?;
        }
        Ok(())
    }

    pub(crate) fn check_physical_profile(&self) -> Result<(), StaticError> {
        for definition in self.definitions.ordered {
            let actual = self.infer(&definition.body, &definition.arguments, true)?;
            require_sort(definition.result, actual)?;
        }
        Ok(())
    }

    pub fn infer_sort(
        &self,
        expression: &KExpr,
        environment: &[KSort],
    ) -> Result<KSort, StaticError> {
        expression
            .validate_resource_bounds()
            .map_err(|_| StaticError::ResourceExhausted)?;
        self.infer(expression, environment, true)
    }

    pub fn evaluate(
        &self,
        expression: &KExpr,
        environment: &[KValue],
        fuel: u64,
    ) -> Result<Evaluation, EvalError> {
        let sorts = value_sorts(environment)?;
        self.infer_sort(expression, &sorts)?;
        for value in environment {
            value
                .validate_resource_bounds()
                .map_err(|_| EvalError::ResourceExhausted)?;
        }
        let result = EvaluationMachine::new(self, environment, fuel)?.run(expression)?;
        Ok(Evaluation {
            value: result.value,
            remaining_fuel: result.fuel,
            observations: result.observations,
        })
    }

    /// Evaluate one generic entrypoint invocation and commit to its actual
    /// value, remaining fuel, and canonical observations. A receipt is only a
    /// producer claim; it grants no package or predecessor authority.
    pub fn build_receipt(
        &self,
        entrypoint: Id32,
        arguments: &[KValue],
        fuel_limit: u64,
    ) -> Result<EvalReceipt, EvalError> {
        let evaluation = self.replay_entrypoint(entrypoint, arguments, fuel_limit)?;
        let observations = evaluation.observations.try_to_term()?;
        let expected_value_hash =
            eval_receipt_value_hash(&evaluation.value).map_err(|_| EvalError::ResourceExhausted)?;
        let expected_observations_hash = eval_receipt_observations_hash(&observations)
            .map_err(|_| EvalError::ResourceExhausted)?;
        Ok(EvalReceipt {
            format_version: 0x00,
            expected_value_hash,
            expected_remaining_fuel: evaluation.remaining_fuel,
            expected_observations_hash,
        })
    }

    pub(crate) fn replay_entrypoint(
        &self,
        entrypoint: Id32,
        arguments: &[KValue],
        fuel_limit: u64,
    ) -> Result<Evaluation, EvalError> {
        if arguments.len() > MAX_WIRE_ITEMS {
            return Err(EvalError::ResourceExhausted);
        }
        let mut argument_expressions = Vec::new();
        argument_expressions
            .try_reserve_exact(arguments.len())
            .map_err(|_| EvalError::ResourceExhausted)?;
        for value in arguments {
            value
                .validate_resource_bounds()
                .map_err(|_| EvalError::ResourceExhausted)?;
            argument_expressions.push(value_literal(value)?);
        }
        let expression = KExpr::Call {
            definition_id: entrypoint,
            arguments: argument_expressions,
        };
        self.evaluate(&expression, &[], fuel_limit)
    }

    fn infer(
        &self,
        expression: &KExpr,
        environment: &[KSort],
        enforce_physical_profile: bool,
    ) -> Result<KSort, StaticError> {
        let mut environments = SortEnvironments::new(environment)?;
        let mut tasks = Vec::new();
        push_infer_task(
            &mut tasks,
            InferTask::Expression {
                expression,
                environment: 0,
                depth: 1,
            },
        )?;
        let mut results = Vec::new();

        while let Some(task) = tasks.pop() {
            match task {
                InferTask::Expression {
                    expression,
                    environment,
                    depth,
                } => {
                    if depth > MAX_EXPRESSION_DEPTH {
                        return Err(StaticError::RecursionLimit);
                    }
                    let next = depth.checked_add(1).ok_or(StaticError::RecursionLimit)?;
                    match expression {
                        KExpr::BytesLiteral(_) => push_sort(&mut results, KSort::Bytes)?,
                        KExpr::TermLiteral(_) => push_sort(&mut results, KSort::Term)?,
                        KExpr::Var(index) => {
                            let wire_index = *index;
                            let index = usize::try_from(wire_index)
                                .map_err(|_| StaticError::VariableOutOfBounds(wire_index))?;
                            let sort = environments
                                .get(environment, index)
                                .ok_or(StaticError::VariableOutOfBounds(wire_index))?;
                            push_sort(&mut results, sort)?;
                        }
                        KExpr::MakeAtom {
                            kind,
                            payload,
                            equality,
                        } => {
                            reserve_infer_tasks(&mut tasks, 7)?;
                            tasks.push(InferTask::Return(KSort::Term));
                            tasks.push(InferTask::Require(KSort::Bytes));
                            tasks.push(InferTask::Expression {
                                expression: equality,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Require(KSort::Bytes));
                            tasks.push(InferTask::Expression {
                                expression: payload,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Require(KSort::Bytes));
                            tasks.push(InferTask::Expression {
                                expression: kind,
                                environment,
                                depth: next,
                            });
                        }
                        KExpr::MakeTriple {
                            first,
                            second,
                            third,
                        } => {
                            reserve_infer_tasks(&mut tasks, 7)?;
                            tasks.push(InferTask::Return(KSort::Term));
                            tasks.push(InferTask::Require(KSort::Term));
                            tasks.push(InferTask::Expression {
                                expression: third,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Require(KSort::Term));
                            tasks.push(InferTask::Expression {
                                expression: second,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Require(KSort::Term));
                            tasks.push(InferTask::Expression {
                                expression: first,
                                environment,
                                depth: next,
                            });
                        }
                        KExpr::Let { value, body } => {
                            reserve_infer_tasks(&mut tasks, 2)?;
                            tasks.push(InferTask::LetBody {
                                body,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Expression {
                                expression: value,
                                environment,
                                depth: next,
                            });
                        }
                        KExpr::CaseTerm {
                            scrutinee,
                            atom_body,
                            triple_body,
                        } => {
                            reserve_infer_tasks(&mut tasks, 3)?;
                            tasks.push(InferTask::CaseTermBodies {
                                atom_body,
                                triple_body,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Require(KSort::Term));
                            tasks.push(InferTask::Expression {
                                expression: scrutinee,
                                environment,
                                depth: next,
                            });
                        }
                        KExpr::CaseBytes {
                            scrutinee,
                            empty_body,
                            cons_body,
                        } => {
                            reserve_infer_tasks(&mut tasks, 3)?;
                            tasks.push(InferTask::CaseBytesBodies {
                                empty_body,
                                cons_body,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Require(KSort::Bytes));
                            tasks.push(InferTask::Expression {
                                expression: scrutinee,
                                environment,
                                depth: next,
                            });
                        }
                        KExpr::ConcatBytes(parts) => {
                            let additional = parts
                                .len()
                                .checked_mul(2)
                                .and_then(|count| count.checked_add(1))
                                .ok_or(StaticError::ResourceExhausted)?;
                            reserve_infer_tasks(&mut tasks, additional)?;
                            tasks.push(InferTask::Return(KSort::Bytes));
                            for part in parts.iter().rev() {
                                tasks.push(InferTask::Require(KSort::Bytes));
                                tasks.push(InferTask::Expression {
                                    expression: part,
                                    environment,
                                    depth: next,
                                });
                            }
                        }
                        KExpr::CaseBytesEqual {
                            left,
                            right,
                            equal_body,
                            unequal_body,
                        } => {
                            reserve_infer_tasks(&mut tasks, 7)?;
                            tasks.push(InferTask::Common);
                            tasks.push(InferTask::Expression {
                                expression: unequal_body,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Expression {
                                expression: equal_body,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Require(KSort::Bytes));
                            tasks.push(InferTask::Expression {
                                expression: right,
                                environment,
                                depth: next,
                            });
                            tasks.push(InferTask::Require(KSort::Bytes));
                            tasks.push(InferTask::Expression {
                                expression: left,
                                environment,
                                depth: next,
                            });
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
                            let additional = arguments
                                .len()
                                .checked_mul(2)
                                .and_then(|count| count.checked_add(1))
                                .ok_or(StaticError::ResourceExhausted)?;
                            reserve_infer_tasks(&mut tasks, additional)?;
                            tasks.push(InferTask::Return(definition.result));
                            for (argument, expected) in
                                arguments.iter().zip(&definition.arguments).rev()
                            {
                                tasks.push(InferTask::Require(*expected));
                                tasks.push(InferTask::Expression {
                                    expression: argument,
                                    environment,
                                    depth: next,
                                });
                            }
                        }
                        KExpr::Request {
                            physical_operation_id,
                            arguments,
                        } => {
                            if enforce_physical_profile {
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
                                let argument =
                                    arguments.first().ok_or(StaticError::ResourceExhausted)?;
                                reserve_infer_tasks(&mut tasks, 3)?;
                                tasks.push(InferTask::Return(KSort::Bytes));
                                tasks.push(InferTask::Require(KSort::Bytes));
                                tasks.push(InferTask::Expression {
                                    expression: argument,
                                    environment,
                                    depth: next,
                                });
                            } else {
                                let additional = arguments
                                    .len()
                                    .checked_add(1)
                                    .ok_or(StaticError::ResourceExhausted)?;
                                reserve_infer_tasks(&mut tasks, additional)?;
                                tasks.push(InferTask::DiscardAndReturn {
                                    count: arguments.len(),
                                    sort: KSort::Bytes,
                                });
                                for argument in arguments.iter().rev() {
                                    tasks.push(InferTask::Expression {
                                        expression: argument,
                                        environment,
                                        depth: next,
                                    });
                                }
                            }
                        }
                    }
                }
                InferTask::Require(expected) => {
                    let actual = results.pop().ok_or(StaticError::ResourceExhausted)?;
                    require_sort(expected, actual)?;
                }
                InferTask::Return(sort) => push_sort(&mut results, sort)?,
                InferTask::DiscardAndReturn { count, sort } => {
                    let retained = results
                        .len()
                        .checked_sub(count)
                        .ok_or(StaticError::ResourceExhausted)?;
                    results.truncate(retained);
                    push_sort(&mut results, sort)?;
                }
                InferTask::Common => {
                    let right = results.pop().ok_or(StaticError::ResourceExhausted)?;
                    let left = results.pop().ok_or(StaticError::ResourceExhausted)?;
                    push_sort(&mut results, common_sort(left, right)?)?;
                }
                InferTask::LetBody {
                    body,
                    environment,
                    depth,
                } => {
                    let value_sort = results.pop().ok_or(StaticError::ResourceExhausted)?;
                    let environment = environments.extend_one(value_sort, environment)?;
                    push_infer_task(
                        &mut tasks,
                        InferTask::Expression {
                            expression: body,
                            environment,
                            depth,
                        },
                    )?;
                }
                InferTask::CaseTermBodies {
                    atom_body,
                    triple_body,
                    environment,
                    depth,
                } => {
                    let atom_environment = environments.extend_three(
                        KSort::Bytes,
                        KSort::Bytes,
                        KSort::Bytes,
                        environment,
                    )?;
                    let triple_environment = environments.extend_three(
                        KSort::Term,
                        KSort::Term,
                        KSort::Term,
                        environment,
                    )?;
                    reserve_infer_tasks(&mut tasks, 3)?;
                    tasks.push(InferTask::Common);
                    tasks.push(InferTask::Expression {
                        expression: triple_body,
                        environment: triple_environment,
                        depth,
                    });
                    tasks.push(InferTask::Expression {
                        expression: atom_body,
                        environment: atom_environment,
                        depth,
                    });
                }
                InferTask::CaseBytesBodies {
                    empty_body,
                    cons_body,
                    environment,
                    depth,
                } => {
                    let cons_environment =
                        environments.extend_two(KSort::Bytes, KSort::Bytes, environment)?;
                    reserve_infer_tasks(&mut tasks, 3)?;
                    tasks.push(InferTask::Common);
                    tasks.push(InferTask::Expression {
                        expression: cons_body,
                        environment: cons_environment,
                        depth,
                    });
                    tasks.push(InferTask::Expression {
                        expression: empty_body,
                        environment,
                        depth,
                    });
                }
            }
        }

        if results.len() != 1 {
            return Err(StaticError::ResourceExhausted);
        }
        results.pop().ok_or(StaticError::ResourceExhausted)
    }
}

enum SortValues<'a> {
    Borrowed(&'a [KSort]),
    One(KSort),
    Two(KSort, KSort),
    Three(KSort, KSort, KSort),
}

impl SortValues<'_> {
    const fn len(&self) -> usize {
        match self {
            Self::Borrowed(values) => values.len(),
            Self::One(_) => 1,
            Self::Two(_, _) => 2,
            Self::Three(_, _, _) => 3,
        }
    }

    fn get(&self, index: usize) -> Option<KSort> {
        match self {
            Self::Borrowed(values) => values.get(index).copied(),
            Self::One(first) => (index == 0).then_some(*first),
            Self::Two(first, second) => [*first, *second].get(index).copied(),
            Self::Three(first, second, third) => [*first, *second, *third].get(index).copied(),
        }
    }
}

struct SortEnvironment<'a> {
    values: SortValues<'a>,
    parent: Option<usize>,
    total_len: usize,
}

struct SortEnvironments<'a> {
    entries: Vec<SortEnvironment<'a>>,
}

impl<'a> SortEnvironments<'a> {
    fn new(values: &'a [KSort]) -> Result<Self, StaticError> {
        if values.len() > MAX_WIRE_ITEMS {
            return Err(StaticError::ResourceExhausted);
        }
        let mut entries = Vec::new();
        entries
            .try_reserve(1)
            .map_err(|_| StaticError::ResourceExhausted)?;
        entries.push(SortEnvironment {
            values: SortValues::Borrowed(values),
            parent: None,
            total_len: values.len(),
        });
        Ok(Self { entries })
    }

    fn extend_one(&mut self, value: KSort, parent: usize) -> Result<usize, StaticError> {
        self.extend(SortValues::One(value), parent)
    }

    fn extend_two(
        &mut self,
        first: KSort,
        second: KSort,
        parent: usize,
    ) -> Result<usize, StaticError> {
        self.extend(SortValues::Two(first, second), parent)
    }

    fn extend_three(
        &mut self,
        first: KSort,
        second: KSort,
        third: KSort,
        parent: usize,
    ) -> Result<usize, StaticError> {
        self.extend(SortValues::Three(first, second, third), parent)
    }

    fn extend(&mut self, values: SortValues<'a>, parent: usize) -> Result<usize, StaticError> {
        let parent_len = self
            .entries
            .get(parent)
            .ok_or(StaticError::ResourceExhausted)?
            .total_len;
        let total_len = parent_len
            .checked_add(values.len())
            .ok_or(StaticError::ResourceExhausted)?;
        if total_len > MAX_WIRE_ITEMS {
            return Err(StaticError::ResourceExhausted);
        }
        self.entries
            .try_reserve(1)
            .map_err(|_| StaticError::ResourceExhausted)?;
        let index = self.entries.len();
        self.entries.push(SortEnvironment {
            values,
            parent: Some(parent),
            total_len,
        });
        Ok(index)
    }

    fn get(&self, mut environment: usize, mut index: usize) -> Option<KSort> {
        loop {
            let entry = self.entries.get(environment)?;
            if index < entry.values.len() {
                return entry.values.get(index);
            }
            index = index.checked_sub(entry.values.len())?;
            environment = entry.parent?;
        }
    }
}

enum InferTask<'a> {
    Expression {
        expression: &'a KExpr,
        environment: usize,
        depth: usize,
    },
    Require(KSort),
    Return(KSort),
    DiscardAndReturn {
        count: usize,
        sort: KSort,
    },
    Common,
    LetBody {
        body: &'a KExpr,
        environment: usize,
        depth: usize,
    },
    CaseTermBodies {
        atom_body: &'a KExpr,
        triple_body: &'a KExpr,
        environment: usize,
        depth: usize,
    },
    CaseBytesBodies {
        empty_body: &'a KExpr,
        cons_body: &'a KExpr,
        environment: usize,
        depth: usize,
    },
}

fn reserve_infer_tasks(
    tasks: &mut Vec<InferTask<'_>>,
    additional: usize,
) -> Result<(), StaticError> {
    let final_len = tasks
        .len()
        .checked_add(additional)
        .ok_or(StaticError::ResourceExhausted)?;
    if final_len > MAX_EVALUATION_FRAMES {
        return Err(StaticError::ResourceExhausted);
    }
    tasks
        .try_reserve(additional)
        .map_err(|_| StaticError::ResourceExhausted)
}

fn push_infer_task<'a>(
    tasks: &mut Vec<InferTask<'a>>,
    task: InferTask<'a>,
) -> Result<(), StaticError> {
    reserve_infer_tasks(tasks, 1)?;
    tasks.push(task);
    Ok(())
}

fn push_sort(results: &mut Vec<KSort>, sort: KSort) -> Result<(), StaticError> {
    results
        .try_reserve(1)
        .map_err(|_| StaticError::ResourceExhausted)?;
    results.push(sort);
    Ok(())
}

#[derive(Debug)]
struct RuntimeBytes {
    storage: usize,
    range: Range<usize>,
}

enum RuntimeByteStorage<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl RuntimeByteStorage<'_> {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Borrowed(bytes) => bytes,
            Self::Owned(bytes) => bytes,
        }
    }

    fn owned_len(&self) -> usize {
        match self {
            Self::Borrowed(_) => 0,
            Self::Owned(bytes) => bytes.len(),
        }
    }
}

struct RuntimeByteEntry<'a> {
    storage: RuntimeByteStorage<'a>,
    references: usize,
}

enum RuntimeByteSlot<'a> {
    Occupied(RuntimeByteEntry<'a>),
    Vacant { next: Option<usize> },
}

struct RuntimeByteStore<'a> {
    entries: Vec<RuntimeByteSlot<'a>>,
    first_vacant: Option<usize>,
    retained_owned_bytes: usize,
    owned_byte_limit: usize,
    #[cfg(test)]
    retained_backing_bytes: usize,
    #[cfg(test)]
    peak_retained_backing_bytes: usize,
}

impl<'a> RuntimeByteStore<'a> {
    const fn new() -> Self {
        Self {
            entries: Vec::new(),
            first_vacant: None,
            retained_owned_bytes: 0,
            owned_byte_limit: MAX_WIRE_BYTES,
            #[cfg(test)]
            retained_backing_bytes: 0,
            #[cfg(test)]
            peak_retained_backing_bytes: 0,
        }
    }

    #[cfg(test)]
    const fn with_owned_byte_limit(owned_byte_limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            first_vacant: None,
            retained_owned_bytes: 0,
            owned_byte_limit,
            retained_backing_bytes: 0,
            peak_retained_backing_bytes: 0,
        }
    }

    fn borrowed(&mut self, bytes: &'a [u8]) -> Result<RuntimeBytes, EvalError> {
        if bytes.len() > MAX_WIRE_BYTES {
            return Err(EvalError::ResourceExhausted);
        }
        let len = bytes.len();
        let storage = self.insert(RuntimeByteStorage::Borrowed(bytes))?;
        Ok(RuntimeBytes {
            storage,
            range: 0..len,
        })
    }

    fn owned(&mut self, bytes: Vec<u8>) -> Result<RuntimeBytes, EvalError> {
        if bytes.len() > MAX_WIRE_BYTES {
            return Err(EvalError::ResourceExhausted);
        }
        let retained_owned_bytes = self
            .retained_owned_bytes
            .checked_add(bytes.len())
            .ok_or(EvalError::ResourceExhausted)?;
        if retained_owned_bytes > self.owned_byte_limit {
            return Err(EvalError::ResourceExhausted);
        }
        let len = bytes.len();
        let storage = self.insert(RuntimeByteStorage::Owned(bytes))?;
        self.retained_owned_bytes = retained_owned_bytes;
        Ok(RuntimeBytes {
            storage,
            range: 0..len,
        })
    }

    fn insert(&mut self, storage: RuntimeByteStorage<'a>) -> Result<usize, EvalError> {
        #[cfg(test)]
        let backing_len = storage.as_slice().len();
        let entry = RuntimeByteEntry {
            storage,
            references: 1,
        };
        let index = if let Some(index) = self.first_vacant {
            let slot = self
                .entries
                .get_mut(index)
                .ok_or(EvalError::ResourceExhausted)?;
            let RuntimeByteSlot::Vacant { next } = slot else {
                return Err(EvalError::ResourceExhausted);
            };
            self.first_vacant = *next;
            *slot = RuntimeByteSlot::Occupied(entry);
            index
        } else {
            if self.entries.len() >= MAX_WIRE_ITEMS {
                return Err(EvalError::ResourceExhausted);
            }
            self.entries
                .try_reserve(1)
                .map_err(|_| EvalError::ResourceExhausted)?;
            let index = self.entries.len();
            self.entries.push(RuntimeByteSlot::Occupied(entry));
            index
        };
        #[cfg(test)]
        {
            self.retained_backing_bytes = self
                .retained_backing_bytes
                .checked_add(backing_len)
                .ok_or(EvalError::ResourceExhausted)?;
            self.peak_retained_backing_bytes = self
                .peak_retained_backing_bytes
                .max(self.retained_backing_bytes);
        }
        Ok(index)
    }

    fn get(&self, bytes: &RuntimeBytes) -> Result<&[u8], EvalError> {
        let RuntimeByteSlot::Occupied(entry) = self
            .entries
            .get(bytes.storage)
            .ok_or(EvalError::ResourceExhausted)?
        else {
            return Err(EvalError::ResourceExhausted);
        };
        entry
            .storage
            .as_slice()
            .get(bytes.range.start..bytes.range.end)
            .ok_or(EvalError::ResourceExhausted)
    }

    fn retain(&mut self, bytes: &RuntimeBytes) -> Result<RuntimeBytes, EvalError> {
        let RuntimeByteSlot::Occupied(entry) = self
            .entries
            .get_mut(bytes.storage)
            .ok_or(EvalError::ResourceExhausted)?
        else {
            return Err(EvalError::ResourceExhausted);
        };
        entry.references = entry
            .references
            .checked_add(1)
            .ok_or(EvalError::ResourceExhausted)?;
        Ok(RuntimeBytes {
            storage: bytes.storage,
            range: bytes.range.start..bytes.range.end,
        })
    }

    fn split_first(
        &mut self,
        mut bytes: RuntimeBytes,
    ) -> Result<(RuntimeBytes, RuntimeBytes), EvalError> {
        let start = bytes.range.start;
        let next = start.checked_add(1).ok_or(EvalError::ResourceExhausted)?;
        self.get(&bytes)?;
        if next > bytes.range.end {
            return Err(EvalError::ResourceExhausted);
        }
        let head = self.retain(&RuntimeBytes {
            storage: bytes.storage,
            range: start..next,
        })?;
        bytes.range.start = next;
        Ok((head, bytes))
    }

    fn try_copy(&self, bytes: &RuntimeBytes) -> Result<Vec<u8>, EvalError> {
        try_copy_bytes(self.get(bytes)?).map_err(|_| EvalError::ResourceExhausted)
    }

    fn materialize(&mut self, bytes: RuntimeBytes) -> Result<Vec<u8>, EvalError> {
        let can_take = match self.entries.get(bytes.storage) {
            Some(RuntimeByteSlot::Occupied(entry)) => {
                entry.references == 1
                    && bytes.range.start == 0
                    && bytes.range.end == entry.storage.as_slice().len()
            }
            _ => return Err(EvalError::ResourceExhausted),
        };
        if can_take {
            let slot = self
                .entries
                .get_mut(bytes.storage)
                .ok_or(EvalError::ResourceExhausted)?;
            let RuntimeByteSlot::Occupied(entry) = std::mem::replace(
                slot,
                RuntimeByteSlot::Vacant {
                    next: self.first_vacant,
                },
            ) else {
                return Err(EvalError::ResourceExhausted);
            };
            self.first_vacant = Some(bytes.storage);
            #[cfg(test)]
            let backing_len = entry.storage.as_slice().len();
            let owned_len = entry.storage.owned_len();
            self.retained_owned_bytes = self
                .retained_owned_bytes
                .checked_sub(owned_len)
                .ok_or(EvalError::ResourceExhausted)?;
            #[cfg(test)]
            {
                self.retained_backing_bytes = self
                    .retained_backing_bytes
                    .checked_sub(backing_len)
                    .ok_or(EvalError::ResourceExhausted)?;
            }
            return match entry.storage {
                RuntimeByteStorage::Borrowed(borrowed) => {
                    try_copy_bytes(borrowed).map_err(|_| EvalError::ResourceExhausted)
                }
                RuntimeByteStorage::Owned(owned) => Ok(owned),
            };
        }
        let copied = self.try_copy(&bytes)?;
        self.release(bytes)?;
        Ok(copied)
    }

    fn release(&mut self, bytes: RuntimeBytes) -> Result<(), EvalError> {
        let entry = match self.entries.get_mut(bytes.storage) {
            Some(RuntimeByteSlot::Occupied(entry)) => entry,
            _ => return Err(EvalError::ResourceExhausted),
        };
        entry.references = entry
            .references
            .checked_sub(1)
            .ok_or(EvalError::ResourceExhausted)?;
        if entry.references != 0 {
            return Ok(());
        }
        let slot = self
            .entries
            .get_mut(bytes.storage)
            .ok_or(EvalError::ResourceExhausted)?;
        let RuntimeByteSlot::Occupied(entry) = std::mem::replace(
            slot,
            RuntimeByteSlot::Vacant {
                next: self.first_vacant,
            },
        ) else {
            return Err(EvalError::ResourceExhausted);
        };
        self.first_vacant = Some(bytes.storage);
        self.retained_owned_bytes = self
            .retained_owned_bytes
            .checked_sub(entry.storage.owned_len())
            .ok_or(EvalError::ResourceExhausted)?;
        #[cfg(test)]
        {
            self.retained_backing_bytes = self
                .retained_backing_bytes
                .checked_sub(entry.storage.as_slice().len())
                .ok_or(EvalError::ResourceExhausted)?;
        }
        Ok(())
    }
}

enum RuntimeTerm<'a> {
    Borrowed(&'a Term),
    Owned(Term),
}

impl RuntimeTerm<'_> {
    fn into_term(self) -> Result<Term, EvalError> {
        match self {
            Self::Borrowed(value) => value
                .try_clone_resource()
                .map_err(|_| EvalError::ResourceExhausted),
            Self::Owned(value) => Ok(value),
        }
    }

    fn validate_resource_bounds(&self) -> Result<(), EvalError> {
        let value = match self {
            Self::Borrowed(value) => *value,
            Self::Owned(value) => value,
        };
        value
            .validate_resource_bounds()
            .map_err(|_| EvalError::ResourceExhausted)
    }
}

enum RuntimeValue<'a> {
    Bytes(RuntimeBytes),
    Term(RuntimeTerm<'a>),
}

impl<'a> RuntimeValue<'a> {
    const fn sort(&self) -> KSort {
        match self {
            Self::Bytes(_) => KSort::Bytes,
            Self::Term(_) => KSort::Term,
        }
    }

    fn borrowed(value: &'a KValue, bytes: &mut RuntimeByteStore<'a>) -> Result<Self, EvalError> {
        match value {
            KValue::Bytes(value) => Ok(Self::Bytes(bytes.borrowed(value)?)),
            KValue::Term(value) => Ok(Self::Term(RuntimeTerm::Borrowed(value))),
        }
    }

    fn owned(value: KValue, bytes: &mut RuntimeByteStore<'a>) -> Result<Self, EvalError> {
        match value {
            KValue::Bytes(value) => Ok(Self::Bytes(bytes.owned(value)?)),
            KValue::Term(value) => Ok(Self::Term(RuntimeTerm::Owned(value))),
        }
    }

    fn validate_resource_bounds(&self, bytes: &RuntimeByteStore<'a>) -> Result<(), EvalError> {
        match self {
            Self::Bytes(value) if bytes.get(value)?.len() <= MAX_WIRE_BYTES => Ok(()),
            Self::Bytes(_) => Err(EvalError::ResourceExhausted),
            Self::Term(value) => value.validate_resource_bounds(),
        }
    }

    fn try_clone_resource(&self, bytes: &mut RuntimeByteStore<'a>) -> Result<Self, EvalError> {
        match self {
            Self::Bytes(value) => Ok(Self::Bytes(bytes.retain(value)?)),
            Self::Term(RuntimeTerm::Borrowed(value)) => {
                Ok(Self::Term(RuntimeTerm::Borrowed(value)))
            }
            Self::Term(RuntimeTerm::Owned(value)) => Ok(Self::Term(RuntimeTerm::Owned(
                value
                    .try_clone_resource()
                    .map_err(|_| EvalError::ResourceExhausted)?,
            ))),
        }
    }

    fn into_kvalue(self, bytes: &mut RuntimeByteStore<'a>) -> Result<KValue, EvalError> {
        match self {
            Self::Bytes(value) => Ok(KValue::Bytes(bytes.materialize(value)?)),
            Self::Term(RuntimeTerm::Borrowed(value)) => Ok(KValue::Term(
                value
                    .try_clone_resource()
                    .map_err(|_| EvalError::ResourceExhausted)?,
            )),
            Self::Term(RuntimeTerm::Owned(value)) => Ok(KValue::Term(value)),
        }
    }

    fn release(self, bytes: &mut RuntimeByteStore<'a>) -> Result<(), EvalError> {
        if let Self::Bytes(value) = self {
            bytes.release(value)?;
        }
        Ok(())
    }
}

enum RuntimeValues<'a> {
    Borrowed(&'a [KValue]),
    Owned(Vec<RuntimeOwnedValue<'a>>),
}

struct RuntimeOwnedValue<'a> {
    value: Option<RuntimeValue<'a>>,
    live_epoch: u64,
}

impl RuntimeValues<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Borrowed(values) => values.len(),
            Self::Owned(values) => values.len(),
        }
    }
}

struct RuntimeEnvironment<'a> {
    values: RuntimeValues<'a>,
    parent: Option<usize>,
    total_len: usize,
    references: usize,
}

enum RuntimeEnvironmentSlot<'a> {
    Occupied(RuntimeEnvironment<'a>),
    Vacant { next: Option<usize> },
}

enum RuntimeValueReference<'a, 'value> {
    Borrowed(&'a KValue),
    Owned(&'value RuntimeValue<'a>),
}

struct RuntimeEnvironments<'a> {
    entries: Vec<RuntimeEnvironmentSlot<'a>>,
    first_vacant: Option<usize>,
    live_entries: usize,
    live_epoch: u64,
    #[cfg(test)]
    peak_live_entries: usize,
}

impl<'a> RuntimeEnvironments<'a> {
    fn new(values: &'a [KValue]) -> Result<Self, EvalError> {
        if values.len() > MAX_WIRE_ITEMS {
            return Err(EvalError::ResourceExhausted);
        }
        let mut entries = Vec::new();
        entries
            .try_reserve(1)
            .map_err(|_| EvalError::ResourceExhausted)?;
        entries.push(RuntimeEnvironmentSlot::Occupied(RuntimeEnvironment {
            values: RuntimeValues::Borrowed(values),
            parent: None,
            total_len: values.len(),
            references: 0,
        }));
        Ok(Self {
            entries,
            first_vacant: None,
            live_entries: 1,
            live_epoch: 0,
            #[cfg(test)]
            peak_live_entries: 1,
        })
    }

    fn entry(&self, index: usize) -> Result<&RuntimeEnvironment<'a>, EvalError> {
        match self.entries.get(index) {
            Some(RuntimeEnvironmentSlot::Occupied(entry)) => Ok(entry),
            _ => Err(EvalError::ResourceExhausted),
        }
    }

    fn entry_mut(&mut self, index: usize) -> Result<&mut RuntimeEnvironment<'a>, EvalError> {
        match self.entries.get_mut(index) {
            Some(RuntimeEnvironmentSlot::Occupied(entry)) => Ok(entry),
            _ => Err(EvalError::ResourceExhausted),
        }
    }

    fn extend(
        &mut self,
        values: Vec<RuntimeValue<'a>>,
        parent: Option<usize>,
    ) -> Result<usize, EvalError> {
        let parent_len = match parent {
            Some(index) => self.entry(index)?.total_len,
            None => 0,
        };
        let total_len = parent_len
            .checked_add(values.len())
            .ok_or(EvalError::ResourceExhausted)?;
        if total_len > MAX_WIRE_ITEMS || self.live_entries >= MAX_RUNTIME_ENVIRONMENTS {
            return Err(EvalError::ResourceExhausted);
        }
        if self.first_vacant.is_none() {
            self.entries
                .try_reserve(1)
                .map_err(|_| EvalError::ResourceExhausted)?;
        }
        if let Some(parent) = parent {
            self.retain(parent)?;
        }
        let mut owned_values = Vec::new();
        owned_values
            .try_reserve_exact(values.len())
            .map_err(|_| EvalError::ResourceExhausted)?;
        for value in values {
            owned_values.push(RuntimeOwnedValue {
                value: Some(value),
                live_epoch: 0,
            });
        }
        let entry = RuntimeEnvironment {
            values: RuntimeValues::Owned(owned_values),
            parent,
            total_len,
            references: 0,
        };
        let index = if let Some(index) = self.first_vacant {
            let slot = self
                .entries
                .get_mut(index)
                .ok_or(EvalError::ResourceExhausted)?;
            let RuntimeEnvironmentSlot::Vacant { next } = slot else {
                return Err(EvalError::ResourceExhausted);
            };
            self.first_vacant = *next;
            *slot = RuntimeEnvironmentSlot::Occupied(entry);
            index
        } else {
            let index = self.entries.len();
            self.entries.push(RuntimeEnvironmentSlot::Occupied(entry));
            index
        };
        self.live_entries = self
            .live_entries
            .checked_add(1)
            .ok_or(EvalError::ResourceExhausted)?;
        #[cfg(test)]
        {
            self.peak_live_entries = self.peak_live_entries.max(self.live_entries);
        }
        Ok(index)
    }

    fn retain(&mut self, environment: usize) -> Result<(), EvalError> {
        let entry = self.entry_mut(environment)?;
        entry.references = entry
            .references
            .checked_add(1)
            .ok_or(EvalError::ResourceExhausted)?;
        Ok(())
    }

    fn release(
        &mut self,
        mut environment: usize,
        bytes: &mut RuntimeByteStore<'a>,
    ) -> Result<(), EvalError> {
        loop {
            let entry = self.entry_mut(environment)?;
            entry.references = entry
                .references
                .checked_sub(1)
                .ok_or(EvalError::ResourceExhausted)?;
            if entry.references != 0 {
                return Ok(());
            }
            let slot = self
                .entries
                .get_mut(environment)
                .ok_or(EvalError::ResourceExhausted)?;
            let RuntimeEnvironmentSlot::Occupied(entry) = std::mem::replace(
                slot,
                RuntimeEnvironmentSlot::Vacant {
                    next: self.first_vacant,
                },
            ) else {
                return Err(EvalError::ResourceExhausted);
            };
            self.first_vacant = Some(environment);
            self.live_entries = self
                .live_entries
                .checked_sub(1)
                .ok_or(EvalError::ResourceExhausted)?;
            if let RuntimeValues::Owned(values) = entry.values {
                for slot in values {
                    if let Some(value) = slot.value {
                        value.release(bytes)?;
                    }
                }
            }
            let Some(parent) = entry.parent else {
                return Ok(());
            };
            environment = parent;
        }
    }

    fn get(
        &self,
        mut environment: usize,
        mut index: usize,
    ) -> Option<RuntimeValueReference<'a, '_>> {
        loop {
            let entry = self.entry(environment).ok()?;
            if index < entry.values.len() {
                return match &entry.values {
                    RuntimeValues::Borrowed(values) => {
                        values.get(index).map(RuntimeValueReference::Borrowed)
                    }
                    RuntimeValues::Owned(values) => values
                        .get(index)
                        .and_then(|slot| slot.value.as_ref())
                        .map(RuntimeValueReference::Owned),
                };
            }
            index = index.checked_sub(entry.values.len())?;
            environment = entry.parent?;
        }
    }

    fn try_clone_value(
        &self,
        environment: usize,
        index: usize,
        bytes: &mut RuntimeByteStore<'a>,
    ) -> Result<RuntimeValue<'a>, EvalError> {
        match self
            .get(environment, index)
            .ok_or(EvalError::ResourceExhausted)?
        {
            RuntimeValueReference::Borrowed(value) => RuntimeValue::borrowed(value, bytes),
            RuntimeValueReference::Owned(value) => value.try_clone_resource(bytes),
        }
    }

    fn locate(&self, mut environment: usize, mut index: usize) -> Option<(usize, usize)> {
        loop {
            let entry = self.entry(environment).ok()?;
            if index < entry.values.len() {
                return Some((environment, index));
            }
            index = index.checked_sub(entry.values.len())?;
            environment = entry.parent?;
        }
    }

    fn begin_live_epoch(&mut self) -> u64 {
        if self.live_epoch == u64::MAX {
            for slot in &mut self.entries {
                let RuntimeEnvironmentSlot::Occupied(entry) = slot else {
                    continue;
                };
                let RuntimeValues::Owned(values) = &mut entry.values else {
                    continue;
                };
                for value in values {
                    value.live_epoch = 0;
                }
            }
            self.live_epoch = 1;
        } else {
            self.live_epoch = self.live_epoch.wrapping_add(1);
        }
        self.live_epoch
    }

    fn mark_live(&mut self, environment: usize, index: usize, epoch: u64) -> Result<(), EvalError> {
        let reported_index = u32::try_from(index).unwrap_or(u32::MAX);
        let (environment, local_index) = self
            .locate(environment, index)
            .ok_or(EvalError::VariableOutOfBounds(reported_index))?;
        let entry = self.entry_mut(environment)?;
        let RuntimeValues::Owned(values) = &mut entry.values else {
            return Ok(());
        };
        let slot = values
            .get_mut(local_index)
            .ok_or(EvalError::ResourceExhausted)?;
        if slot.value.is_none() {
            return Err(EvalError::ResourceExhausted);
        }
        slot.live_epoch = epoch;
        Ok(())
    }

    fn discard_unmarked(
        &mut self,
        epoch: u64,
        bytes: &mut RuntimeByteStore<'a>,
    ) -> Result<usize, EvalError> {
        let mut discarded = 0_usize;
        for slot in &mut self.entries {
            let RuntimeEnvironmentSlot::Occupied(entry) = slot else {
                continue;
            };
            let RuntimeValues::Owned(values) = &mut entry.values else {
                continue;
            };
            for slot in values {
                if slot.value.is_none() || slot.live_epoch == epoch {
                    continue;
                }
                let value = slot.value.take().ok_or(EvalError::ResourceExhausted)?;
                value.release(bytes)?;
                discarded = discarded
                    .checked_add(1)
                    .ok_or(EvalError::ResourceExhausted)?;
            }
        }
        Ok(discarded)
    }
}

enum EvalTask<'a> {
    Expression {
        expression: &'a KExpr,
        environment: usize,
    },
    MakeAtom,
    MakeTriple,
    Let {
        environment: usize,
        body: &'a KExpr,
    },
    CaseTerm {
        environment: usize,
        atom_body: &'a KExpr,
        triple_body: &'a KExpr,
    },
    CaseBytes {
        environment: usize,
        empty_body: &'a KExpr,
        cons_body: &'a KExpr,
    },
    ConcatBytes {
        count: usize,
    },
    CaseBytesEqual {
        environment: usize,
        equal_body: &'a KExpr,
        unequal_body: &'a KExpr,
    },
    Call {
        definition: &'a Definition,
        argument_count: usize,
    },
    Request {
        operation_id: Id32,
    },
}

impl EvalTask<'_> {
    fn environment(&self) -> Option<usize> {
        match self {
            Self::Expression { environment, .. } => Some(*environment),
            Self::Let { environment, .. }
            | Self::CaseTerm { environment, .. }
            | Self::CaseBytes { environment, .. }
            | Self::CaseBytesEqual { environment, .. } => Some(*environment),
            Self::MakeAtom
            | Self::MakeTriple
            | Self::ConcatBytes { .. }
            | Self::Call { .. }
            | Self::Request { .. } => None,
        }
    }
}

#[derive(Clone, Copy)]
struct LiveExpression<'a> {
    expression: &'a KExpr,
    bound: usize,
    environment: usize,
}

struct LiveScanFrame<'a> {
    expression: &'a KExpr,
    bound: usize,
    next_child: usize,
}

fn push_live_scan_frame<'a>(
    stack: &mut Vec<LiveScanFrame<'a>>,
    expression: &'a KExpr,
    bound: usize,
) -> Result<(), EvalError> {
    if stack.len() >= MAX_EXPRESSION_DEPTH {
        return Err(EvalError::RecursionLimit);
    }
    stack.push(LiveScanFrame {
        expression,
        bound,
        next_child: 0,
    });
    Ok(())
}

fn next_live_child(
    expression: &KExpr,
    child: usize,
    bound: usize,
) -> Result<Option<(&KExpr, usize)>, EvalError> {
    match expression {
        KExpr::BytesLiteral(_) | KExpr::TermLiteral(_) | KExpr::Var(_) => Ok(None),
        KExpr::MakeAtom {
            kind,
            payload,
            equality,
        } => match [&**kind, &**payload, &**equality].get(child).copied() {
            Some(expression) => Ok(Some((expression, bound))),
            None => Ok(None),
        },
        KExpr::MakeTriple {
            first,
            second,
            third,
        } => match [&**first, &**second, &**third].get(child).copied() {
            Some(expression) => Ok(Some((expression, bound))),
            None => Ok(None),
        },
        KExpr::Let { value, body } => match child {
            0 => Ok(Some((value, bound))),
            1 => Ok(Some((
                body,
                bound.checked_add(1).ok_or(EvalError::ResourceExhausted)?,
            ))),
            _ => Ok(None),
        },
        KExpr::CaseTerm {
            scrutinee,
            atom_body,
            triple_body,
        } => match child {
            0 => Ok(Some((scrutinee, bound))),
            1 => Ok(Some((
                atom_body,
                bound.checked_add(3).ok_or(EvalError::ResourceExhausted)?,
            ))),
            2 => Ok(Some((
                triple_body,
                bound.checked_add(3).ok_or(EvalError::ResourceExhausted)?,
            ))),
            _ => Ok(None),
        },
        KExpr::CaseBytes {
            scrutinee,
            empty_body,
            cons_body,
        } => match child {
            0 => Ok(Some((scrutinee, bound))),
            1 => Ok(Some((empty_body, bound))),
            2 => Ok(Some((
                cons_body,
                bound.checked_add(2).ok_or(EvalError::ResourceExhausted)?,
            ))),
            _ => Ok(None),
        },
        KExpr::ConcatBytes(parts) => match parts.get(child) {
            Some(expression) => Ok(Some((expression, bound))),
            None => Ok(None),
        },
        KExpr::CaseBytesEqual {
            left,
            right,
            equal_body,
            unequal_body,
        } => match [&**left, &**right, &**equal_body, &**unequal_body]
            .get(child)
            .copied()
        {
            Some(expression) => Ok(Some((expression, bound))),
            None => Ok(None),
        },
        KExpr::Call { arguments, .. } | KExpr::Request { arguments, .. } => {
            match arguments.get(child) {
                Some(expression) => Ok(Some((expression, bound))),
                None => Ok(None),
            }
        }
    }
}

fn mark_live_expression<'expression>(
    environments: &mut RuntimeEnvironments<'expression>,
    epoch: u64,
    stack: &mut Vec<LiveScanFrame<'expression>>,
    root: LiveExpression<'expression>,
) -> Result<usize, EvalError> {
    stack.clear();
    let result = mark_live_expression_inner(environments, epoch, stack, root);
    stack.clear();
    result
}

fn mark_live_expression_inner<'expression>(
    environments: &mut RuntimeEnvironments<'expression>,
    epoch: u64,
    stack: &mut Vec<LiveScanFrame<'expression>>,
    LiveExpression {
        expression,
        bound,
        environment,
    }: LiveExpression<'expression>,
) -> Result<usize, EvalError> {
    push_live_scan_frame(stack, expression, bound)?;
    let mut nodes = 0_usize;
    let mut peak_depth = stack.len();
    while !stack.is_empty() {
        let first_visit = matches!(stack.last(), Some(frame) if frame.next_child == 0);
        if first_visit {
            nodes = nodes.checked_add(1).ok_or(EvalError::ResourceExhausted)?;
            if nodes > MAX_EXPRESSION_NODES {
                return Err(EvalError::ResourceExhausted);
            }
            let frame = stack.last().ok_or(EvalError::ResourceExhausted)?;
            if let KExpr::Var(index) = frame.expression {
                let index =
                    usize::try_from(*index).map_err(|_| EvalError::VariableOutOfBounds(*index))?;
                if index >= frame.bound {
                    let outer_index = index
                        .checked_sub(frame.bound)
                        .ok_or(EvalError::ResourceExhausted)?;
                    environments.mark_live(environment, outer_index, epoch)?;
                }
            }
        }
        let child = {
            let frame = stack.last_mut().ok_or(EvalError::ResourceExhausted)?;
            let child = next_live_child(frame.expression, frame.next_child, frame.bound)?;
            frame.next_child = frame
                .next_child
                .checked_add(1)
                .ok_or(EvalError::ResourceExhausted)?;
            child
        };
        if let Some((expression, bound)) = child {
            push_live_scan_frame(stack, expression, bound)?;
            peak_depth = peak_depth.max(stack.len());
        } else {
            stack.pop();
        }
    }
    Ok(peak_depth)
}

fn mark_task_live_slots<'expression>(
    environments: &mut RuntimeEnvironments<'expression>,
    epoch: u64,
    stack: &mut Vec<LiveScanFrame<'expression>>,
    task: &EvalTask<'expression>,
) -> Result<usize, EvalError> {
    let mut peak_depth = 0_usize;
    let mut mark = |expression, bound, environment| {
        let depth = mark_live_expression(
            environments,
            epoch,
            stack,
            LiveExpression {
                expression,
                bound,
                environment,
            },
        )?;
        peak_depth = peak_depth.max(depth);
        Ok::<(), EvalError>(())
    };
    match task {
        EvalTask::Expression {
            expression,
            environment,
        } => mark(expression, 0, *environment)?,
        EvalTask::Let { environment, body } => mark(body, 1, *environment)?,
        EvalTask::CaseTerm {
            environment,
            atom_body,
            triple_body,
        } => {
            mark(atom_body, 3, *environment)?;
            mark(triple_body, 3, *environment)?;
        }
        EvalTask::CaseBytes {
            environment,
            empty_body,
            cons_body,
        } => {
            mark(empty_body, 0, *environment)?;
            mark(cons_body, 2, *environment)?;
        }
        EvalTask::CaseBytesEqual {
            environment,
            equal_body,
            unequal_body,
        } => {
            mark(equal_body, 0, *environment)?;
            mark(unequal_body, 0, *environment)?;
        }
        EvalTask::MakeAtom
        | EvalTask::MakeTriple
        | EvalTask::ConcatBytes { .. }
        | EvalTask::Call { .. }
        | EvalTask::Request { .. } => {}
    }
    Ok(peak_depth)
}

struct RuntimeResult<'a> {
    value: RuntimeValue<'a>,
}

struct MachineResult {
    value: KValue,
    fuel: u64,
    observations: ObservationLog,
    #[cfg(test)]
    peak_runtime_byte_backing: usize,
    #[cfg(test)]
    peak_runtime_environments: usize,
    #[cfg(test)]
    retained_runtime_byte_backing: usize,
    #[cfg(test)]
    live_runtime_environments: usize,
    #[cfg(test)]
    liveness_reclamation_runs: usize,
    #[cfg(test)]
    peak_liveness_scan_depth: usize,
}

struct EvaluationMachine<'a> {
    evaluator: &'a Evaluator<'a>,
    environments: RuntimeEnvironments<'a>,
    byte_store: RuntimeByteStore<'a>,
    tasks: Vec<EvalTask<'a>>,
    results: Vec<RuntimeResult<'a>>,
    fuel: u64,
    observations: ObservationLog,
    live_scan_stack: Vec<LiveScanFrame<'a>>,
    #[cfg(test)]
    liveness_reclamation_runs: usize,
    #[cfg(test)]
    peak_liveness_scan_depth: usize,
}

impl<'a> EvaluationMachine<'a> {
    fn new(
        evaluator: &'a Evaluator<'a>,
        environment: &'a [KValue],
        fuel: u64,
    ) -> Result<Self, EvalError> {
        let mut live_scan_stack = Vec::new();
        live_scan_stack
            .try_reserve_exact(MAX_EXPRESSION_DEPTH)
            .map_err(|_| EvalError::ResourceExhausted)?;
        Ok(Self {
            evaluator,
            environments: RuntimeEnvironments::new(environment)?,
            byte_store: RuntimeByteStore::new(),
            tasks: Vec::new(),
            results: Vec::new(),
            fuel,
            observations: ObservationLog::default(),
            live_scan_stack,
            #[cfg(test)]
            liveness_reclamation_runs: 0,
            #[cfg(test)]
            peak_liveness_scan_depth: 0,
        })
    }

    fn run(mut self, expression: &'a KExpr) -> Result<MachineResult, EvalError> {
        self.push_task(EvalTask::Expression {
            expression,
            environment: 0,
        })?;
        while let Some(task) = self.tasks.pop() {
            let environment = task.environment();
            let outcome = match task {
                EvalTask::Expression {
                    expression,
                    environment,
                } => self.enter(expression, environment),
                EvalTask::MakeAtom => self.finish_make_atom(),
                EvalTask::MakeTriple => self.finish_make_triple(),
                EvalTask::Let { environment, body } => self.continue_let(environment, body),
                EvalTask::CaseTerm {
                    environment,
                    atom_body,
                    triple_body,
                } => self.continue_case_term(environment, atom_body, triple_body),
                EvalTask::CaseBytes {
                    environment,
                    empty_body,
                    cons_body,
                } => self.continue_case_bytes(environment, empty_body, cons_body),
                EvalTask::ConcatBytes { count } => self.finish_concat(count),
                EvalTask::CaseBytesEqual {
                    environment,
                    equal_body,
                    unequal_body,
                } => self.continue_case_bytes_equal(environment, equal_body, unequal_body),
                EvalTask::Call {
                    definition,
                    argument_count,
                } => self.continue_call(definition, argument_count),
                EvalTask::Request { operation_id } => self.finish_request(operation_id),
            };
            if let Some(environment) = environment {
                self.environments
                    .release(environment, &mut self.byte_store)?;
            }
            outcome?;
        }
        if self.results.len() != 1 {
            return Err(EvalError::ResourceExhausted);
        }
        let value = self
            .results
            .pop()
            .ok_or(EvalError::ResourceExhausted)?
            .value
            .into_kvalue(&mut self.byte_store)?;
        Ok(MachineResult {
            value,
            fuel: self.fuel,
            observations: self.observations,
            #[cfg(test)]
            peak_runtime_byte_backing: self.byte_store.peak_retained_backing_bytes,
            #[cfg(test)]
            peak_runtime_environments: self.environments.peak_live_entries,
            #[cfg(test)]
            retained_runtime_byte_backing: self.byte_store.retained_backing_bytes,
            #[cfg(test)]
            live_runtime_environments: self.environments.live_entries,
            #[cfg(test)]
            liveness_reclamation_runs: self.liveness_reclamation_runs,
            #[cfg(test)]
            peak_liveness_scan_depth: self.peak_liveness_scan_depth,
        })
    }

    fn prepare_owned_allocation(
        &mut self,
        current: Option<LiveExpression<'a>>,
        incoming: usize,
    ) -> Result<(), EvalError> {
        let projected = self
            .byte_store
            .retained_owned_bytes
            .checked_add(incoming)
            .ok_or(EvalError::ResourceExhausted)?;
        if projected <= self.byte_store.owned_byte_limit {
            return Ok(());
        }
        #[cfg(test)]
        {
            self.liveness_reclamation_runs = self
                .liveness_reclamation_runs
                .checked_add(1)
                .ok_or(EvalError::ResourceExhausted)?;
        }
        let epoch = self.environments.begin_live_epoch();
        let mut peak_depth = 0_usize;
        if let Some(current) = current {
            peak_depth = mark_live_expression(
                &mut self.environments,
                epoch,
                &mut self.live_scan_stack,
                current,
            )?;
        }
        for task in &self.tasks {
            let depth = mark_task_live_slots(
                &mut self.environments,
                epoch,
                &mut self.live_scan_stack,
                task,
            )?;
            peak_depth = peak_depth.max(depth);
        }
        #[cfg(test)]
        {
            self.peak_liveness_scan_depth = self.peak_liveness_scan_depth.max(peak_depth);
        }
        self.environments
            .discard_unmarked(epoch, &mut self.byte_store)?;
        let projected = self
            .byte_store
            .retained_owned_bytes
            .checked_add(incoming)
            .ok_or(EvalError::ResourceExhausted)?;
        if projected <= self.byte_store.owned_byte_limit {
            Ok(())
        } else {
            Err(EvalError::ResourceExhausted)
        }
    }

    fn enter(&mut self, expression: &'a KExpr, environment: usize) -> Result<(), EvalError> {
        self.fuel = self.fuel.checked_sub(1).ok_or(EvalError::OutOfFuel)?;

        match expression {
            KExpr::BytesLiteral(bytes) => {
                let value = RuntimeValue::Bytes(self.byte_store.borrowed(bytes)?);
                self.push_result(value)
            }
            KExpr::TermLiteral(term) => {
                self.push_result(RuntimeValue::Term(RuntimeTerm::Borrowed(term)))
            }
            KExpr::Var(index) => {
                let wire_index = *index;
                let index = usize::try_from(wire_index)
                    .map_err(|_| EvalError::VariableOutOfBounds(wire_index))?;
                if self.environments.get(environment, index).is_none() {
                    return Err(EvalError::VariableOutOfBounds(wire_index));
                }
                let value =
                    self.environments
                        .try_clone_value(environment, index, &mut self.byte_store)?;
                self.push_result(value)
            }
            KExpr::MakeAtom {
                kind,
                payload,
                equality,
            } => {
                self.reserve_tasks(4)?;
                self.push_reserved_task(EvalTask::MakeAtom)?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: equality,
                    environment,
                })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: payload,
                    environment,
                })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: kind,
                    environment,
                })?;
                Ok(())
            }
            KExpr::MakeTriple {
                first,
                second,
                third,
            } => {
                self.reserve_tasks(4)?;
                self.push_reserved_task(EvalTask::MakeTriple)?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: third,
                    environment,
                })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: second,
                    environment,
                })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: first,
                    environment,
                })?;
                Ok(())
            }
            KExpr::Let { value, body } => {
                self.reserve_tasks(2)?;
                self.push_reserved_task(EvalTask::Let { environment, body })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: value,
                    environment,
                })?;
                Ok(())
            }
            KExpr::CaseTerm {
                scrutinee,
                atom_body,
                triple_body,
            } => {
                self.reserve_tasks(2)?;
                self.push_reserved_task(EvalTask::CaseTerm {
                    environment,
                    atom_body,
                    triple_body,
                })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: scrutinee,
                    environment,
                })?;
                Ok(())
            }
            KExpr::CaseBytes {
                scrutinee,
                empty_body,
                cons_body,
            } => {
                self.reserve_tasks(2)?;
                self.push_reserved_task(EvalTask::CaseBytes {
                    environment,
                    empty_body,
                    cons_body,
                })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: scrutinee,
                    environment,
                })?;
                Ok(())
            }
            KExpr::ConcatBytes(parts) => {
                let additional = parts
                    .len()
                    .checked_add(1)
                    .ok_or(EvalError::ResourceExhausted)?;
                self.reserve_tasks(additional)?;
                self.push_reserved_task(EvalTask::ConcatBytes { count: parts.len() })?;
                for part in parts.iter().rev() {
                    self.push_reserved_task(EvalTask::Expression {
                        expression: part,
                        environment,
                    })?;
                }
                Ok(())
            }
            KExpr::CaseBytesEqual {
                left,
                right,
                equal_body,
                unequal_body,
            } => {
                self.reserve_tasks(3)?;
                self.push_reserved_task(EvalTask::CaseBytesEqual {
                    environment,
                    equal_body,
                    unequal_body,
                })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: right,
                    environment,
                })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: left,
                    environment,
                })?;
                Ok(())
            }
            KExpr::Call {
                definition_id,
                arguments,
            } => {
                let definition = self
                    .evaluator
                    .definitions
                    .resolve(*definition_id)
                    .ok_or(EvalError::UnknownDefinition(*definition_id))?;
                if definition.arguments.len() != arguments.len() {
                    return Err(EvalError::ArgumentCount {
                        expected: definition.arguments.len(),
                        actual: arguments.len(),
                    });
                }
                let additional = arguments
                    .len()
                    .checked_add(1)
                    .ok_or(EvalError::ResourceExhausted)?;
                self.reserve_tasks(additional)?;
                self.push_reserved_task(EvalTask::Call {
                    definition,
                    argument_count: arguments.len(),
                })?;
                for argument in arguments.iter().rev() {
                    self.push_reserved_task(EvalTask::Expression {
                        expression: argument,
                        environment,
                    })?;
                }
                Ok(())
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
                let argument = arguments.first().ok_or(EvalError::ResourceExhausted)?;
                self.reserve_tasks(2)?;
                self.push_reserved_task(EvalTask::Request {
                    operation_id: *physical_operation_id,
                })?;
                self.push_reserved_task(EvalTask::Expression {
                    expression: argument,
                    environment,
                })?;
                Ok(())
            }
        }
    }

    fn finish_make_atom(&mut self) -> Result<(), EvalError> {
        let equality = self.pop_result()?;
        let payload = self.pop_result()?;
        let kind = self.pop_result()?;
        let kind = self
            .byte_store
            .materialize(expect_runtime_bytes(kind.value)?)?;
        let canonical_payload = self
            .byte_store
            .materialize(expect_runtime_bytes(payload.value)?)?;
        let equality_contract = self
            .byte_store
            .materialize(expect_runtime_bytes(equality.value)?)?;
        let value = RuntimeValue::Term(RuntimeTerm::Owned(Term::Atom {
            kind,
            canonical_payload,
            equality_contract,
        }));
        self.push_result(value)
    }

    fn finish_make_triple(&mut self) -> Result<(), EvalError> {
        let third = self.pop_result()?;
        let second = self.pop_result()?;
        let first = self.pop_result()?;
        let first = expect_runtime_term(first.value)?.into_term()?;
        let second = expect_runtime_term(second.value)?.into_term()?;
        let third = expect_runtime_term(third.value)?.into_term()?;
        let term =
            Term::try_triple(first, second, third).map_err(|_| EvalError::ResourceExhausted)?;
        self.push_result(RuntimeValue::Term(RuntimeTerm::Owned(term)))
    }

    fn continue_let(&mut self, parent: usize, body: &'a KExpr) -> Result<(), EvalError> {
        let value = self.pop_result()?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(1)
            .map_err(|_| EvalError::ResourceExhausted)?;
        values.push(value.value);
        self.reserve_tasks(1)?;
        let environment = self.environments.extend(values, Some(parent))?;
        self.push_reserved_task(EvalTask::Expression {
            expression: body,
            environment,
        })?;
        Ok(())
    }

    fn continue_case_term(
        &mut self,
        parent: usize,
        atom_body: &'a KExpr,
        triple_body: &'a KExpr,
    ) -> Result<(), EvalError> {
        let scrutinee = self.pop_result()?;
        let (body, values) = match expect_runtime_term(scrutinee.value)? {
            RuntimeTerm::Borrowed(Term::Atom {
                kind,
                canonical_payload,
                equality_contract,
            }) => {
                let mut values = Vec::new();
                values
                    .try_reserve_exact(3)
                    .map_err(|_| EvalError::ResourceExhausted)?;
                values.push(RuntimeValue::Bytes(self.byte_store.borrowed(kind)?));
                values.push(RuntimeValue::Bytes(
                    self.byte_store.borrowed(canonical_payload)?,
                ));
                values.push(RuntimeValue::Bytes(
                    self.byte_store.borrowed(equality_contract)?,
                ));
                (atom_body, values)
            }
            RuntimeTerm::Borrowed(Term::Triple(first, second, third)) => {
                let mut values = Vec::new();
                values
                    .try_reserve_exact(3)
                    .map_err(|_| EvalError::ResourceExhausted)?;
                values.push(RuntimeValue::Term(RuntimeTerm::Borrowed(first)));
                values.push(RuntimeValue::Term(RuntimeTerm::Borrowed(second)));
                values.push(RuntimeValue::Term(RuntimeTerm::Borrowed(third)));
                (triple_body, values)
            }
            RuntimeTerm::Owned(Term::Atom {
                kind,
                canonical_payload,
                equality_contract,
            }) => {
                let incoming = kind
                    .len()
                    .checked_add(canonical_payload.len())
                    .and_then(|bytes| bytes.checked_add(equality_contract.len()))
                    .ok_or(EvalError::ResourceExhausted)?;
                self.prepare_owned_allocation(
                    Some(LiveExpression {
                        expression: atom_body,
                        bound: 3,
                        environment: parent,
                    }),
                    incoming,
                )?;
                let mut values = Vec::new();
                values
                    .try_reserve_exact(3)
                    .map_err(|_| EvalError::ResourceExhausted)?;
                values.push(RuntimeValue::Bytes(self.byte_store.owned(kind)?));
                values.push(RuntimeValue::Bytes(
                    self.byte_store.owned(canonical_payload)?,
                ));
                values.push(RuntimeValue::Bytes(
                    self.byte_store.owned(equality_contract)?,
                ));
                (atom_body, values)
            }
            RuntimeTerm::Owned(Term::Triple(first, second, third)) => {
                let mut values = Vec::new();
                values
                    .try_reserve_exact(3)
                    .map_err(|_| EvalError::ResourceExhausted)?;
                values.push(RuntimeValue::Term(RuntimeTerm::Owned(first.into_inner())));
                values.push(RuntimeValue::Term(RuntimeTerm::Owned(second.into_inner())));
                values.push(RuntimeValue::Term(RuntimeTerm::Owned(third.into_inner())));
                (triple_body, values)
            }
        };
        self.reserve_tasks(1)?;
        let environment = self.environments.extend(values, Some(parent))?;
        self.push_reserved_task(EvalTask::Expression {
            expression: body,
            environment,
        })?;
        Ok(())
    }

    fn continue_case_bytes(
        &mut self,
        parent: usize,
        empty_body: &'a KExpr,
        cons_body: &'a KExpr,
    ) -> Result<(), EvalError> {
        let scrutinee = self.pop_result()?;
        let bytes = expect_runtime_bytes(scrutinee.value)?;
        self.reserve_tasks(1)?;
        self.byte_store.get(&bytes)?;
        let (body, environment) = if bytes.range.start == bytes.range.end {
            self.byte_store.release(bytes)?;
            (empty_body, parent)
        } else {
            let (head, tail) = self.byte_store.split_first(bytes)?;
            let mut values = Vec::new();
            values
                .try_reserve_exact(2)
                .map_err(|_| EvalError::ResourceExhausted)?;
            values.push(RuntimeValue::Bytes(head));
            values.push(RuntimeValue::Bytes(tail));
            let environment = self.environments.extend(values, Some(parent))?;
            (cons_body, environment)
        };
        self.push_reserved_task(EvalTask::Expression {
            expression: body,
            environment,
        })?;
        Ok(())
    }

    fn finish_concat(&mut self, count: usize) -> Result<(), EvalError> {
        let results = self.take_results(count)?;
        let mut total = 0_usize;
        for result in &results {
            let RuntimeValue::Bytes(bytes) = &result.value else {
                return Err(EvalError::ValueSort {
                    expected: KSort::Bytes,
                    actual: KSort::Term,
                });
            };
            total = total
                .checked_add(self.byte_store.get(bytes)?.len())
                .ok_or(EvalError::ByteLengthOverflow)?;
            if total > MAX_WIRE_BYTES {
                return Err(EvalError::ByteLengthOverflow);
            }
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(total)
            .map_err(|_| EvalError::ResourceExhausted)?;
        for result in results {
            let RuntimeValue::Bytes(part) = result.value else {
                return Err(EvalError::ValueSort {
                    expected: KSort::Bytes,
                    actual: KSort::Term,
                });
            };
            bytes.extend_from_slice(self.byte_store.get(&part)?);
            self.byte_store.release(part)?;
        }
        self.prepare_owned_allocation(None, total)?;
        let value = RuntimeValue::Bytes(self.byte_store.owned(bytes)?);
        self.push_result(value)
    }

    fn continue_case_bytes_equal(
        &mut self,
        environment: usize,
        equal_body: &'a KExpr,
        unequal_body: &'a KExpr,
    ) -> Result<(), EvalError> {
        let right = self.pop_result()?;
        let left = self.pop_result()?;
        let left = expect_runtime_bytes(left.value)?;
        let right = expect_runtime_bytes(right.value)?;
        let equal = self.byte_store.get(&left)? == self.byte_store.get(&right)?;
        self.byte_store.release(left)?;
        self.byte_store.release(right)?;
        let body = if equal { equal_body } else { unequal_body };
        self.reserve_tasks(1)?;
        self.push_reserved_task(EvalTask::Expression {
            expression: body,
            environment,
        })?;
        Ok(())
    }

    fn continue_call(
        &mut self,
        definition: &'a Definition,
        argument_count: usize,
    ) -> Result<(), EvalError> {
        let results = self.take_results(argument_count)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(argument_count)
            .map_err(|_| EvalError::ResourceExhausted)?;
        for (result, expected) in results.into_iter().zip(&definition.arguments) {
            require_runtime_value_sort(*expected, &result.value)?;
            values.push(result.value);
        }
        self.reserve_tasks(1)?;
        let environment = self.environments.extend(values, None)?;
        self.push_reserved_task(EvalTask::Expression {
            expression: &definition.body,
            environment,
        })?;
        Ok(())
    }

    fn finish_request(&mut self, operation_id: Id32) -> Result<(), EvalError> {
        let argument = self.pop_result()?;
        require_runtime_value_sort(KSort::Bytes, &argument.value)?;
        let mut arguments = Vec::new();
        arguments
            .try_reserve_exact(1)
            .map_err(|_| EvalError::ResourceExhausted)?;
        arguments.push(argument.value.into_kvalue(&mut self.byte_store)?);
        let value =
            self.evaluator
                .physical
                .request(operation_id, arguments, &mut self.observations)?;
        let incoming = match &value {
            KValue::Bytes(bytes) => bytes.len(),
            KValue::Term(_) => 0,
        };
        self.prepare_owned_allocation(None, incoming)?;
        let value = RuntimeValue::owned(value, &mut self.byte_store)?;
        self.push_result(value)
    }

    fn push_result(&mut self, value: RuntimeValue<'a>) -> Result<(), EvalError> {
        value.validate_resource_bounds(&self.byte_store)?;
        self.results
            .try_reserve(1)
            .map_err(|_| EvalError::ResourceExhausted)?;
        self.results.push(RuntimeResult { value });
        Ok(())
    }

    fn pop_result(&mut self) -> Result<RuntimeResult<'a>, EvalError> {
        self.results.pop().ok_or(EvalError::ResourceExhausted)
    }

    fn take_results(&mut self, count: usize) -> Result<Vec<RuntimeResult<'a>>, EvalError> {
        if count > self.results.len() {
            return Err(EvalError::ResourceExhausted);
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| EvalError::ResourceExhausted)?;
        for _ in 0..count {
            values.push(self.pop_result()?);
        }
        values.reverse();
        Ok(values)
    }

    fn reserve_tasks(&mut self, additional: usize) -> Result<(), EvalError> {
        let final_len = self
            .tasks
            .len()
            .checked_add(additional)
            .ok_or(EvalError::RecursionLimit)?;
        if final_len > MAX_EVALUATION_FRAMES {
            return Err(EvalError::RecursionLimit);
        }
        self.tasks
            .try_reserve(additional)
            .map_err(|_| EvalError::ResourceExhausted)
    }

    fn push_task(&mut self, task: EvalTask<'a>) -> Result<(), EvalError> {
        self.reserve_tasks(1)?;
        self.push_reserved_task(task)?;
        Ok(())
    }

    fn push_reserved_task(&mut self, task: EvalTask<'a>) -> Result<(), EvalError> {
        if let Some(environment) = task.environment() {
            self.environments.retain(environment)?;
        }
        self.tasks.push(task);
        Ok(())
    }
}

fn value_sorts(values: &[KValue]) -> Result<Vec<KSort>, EvalError> {
    if values.len() > MAX_WIRE_ITEMS {
        return Err(EvalError::ResourceExhausted);
    }
    let mut sorts = Vec::new();
    sorts
        .try_reserve_exact(values.len())
        .map_err(|_| EvalError::ResourceExhausted)?;
    for value in values {
        sorts.push(value.sort());
    }
    Ok(sorts)
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

fn require_runtime_value_sort(expected: KSort, value: &RuntimeValue<'_>) -> Result<(), EvalError> {
    let actual = value.sort();
    if actual == expected {
        Ok(())
    } else {
        Err(EvalError::ValueSort { expected, actual })
    }
}

fn expect_runtime_bytes(value: RuntimeValue<'_>) -> Result<RuntimeBytes, EvalError> {
    match value {
        RuntimeValue::Bytes(value) => Ok(value),
        RuntimeValue::Term(_) => Err(EvalError::ValueSort {
            expected: KSort::Bytes,
            actual: KSort::Term,
        }),
    }
}

fn expect_runtime_term(value: RuntimeValue<'_>) -> Result<RuntimeTerm<'_>, EvalError> {
    match value {
        RuntimeValue::Term(value) => Ok(value),
        RuntimeValue::Bytes(_) => Err(EvalError::ValueSort {
            expected: KSort::Term,
            actual: KSort::Bytes,
        }),
    }
}

fn value_literal(value: &KValue) -> Result<KExpr, EvalError> {
    match value {
        KValue::Bytes(bytes) => Ok(KExpr::BytesLiteral(
            try_copy_bytes(bytes).map_err(|_| EvalError::ResourceExhausted)?,
        )),
        KValue::Term(term) => Ok(KExpr::TermLiteral(
            term.try_clone_resource()
                .map_err(|_| EvalError::ResourceExhausted)?,
        )),
    }
}

#[cfg(test)]
mod runtime_retention_tests {
    use super::*;
    use crate::compiler_package_v3::FallibleBox;

    fn boxed(value: KExpr) -> FallibleBox<KExpr> {
        FallibleBox::try_new(value).expect("test expression allocation")
    }

    #[test]
    fn recursive_case_bytes_retains_one_backing_and_reclaims_environments() {
        const INPUT_LEN: usize = 4_096;
        const LOOP_ID: Id32 = Id32([0x5a; 32]);

        let definitions = vec![Definition {
            id: LOOP_ID,
            arguments: vec![KSort::Bytes],
            result: KSort::Bytes,
            body: KExpr::CaseBytes {
                scrutinee: boxed(KExpr::Var(0)),
                empty_body: boxed(KExpr::BytesLiteral(Vec::new())),
                cons_body: boxed(KExpr::Call {
                    definition_id: LOOP_ID,
                    arguments: vec![KExpr::Var(1)],
                }),
            },
        }];
        let evaluator = Evaluator::new(&definitions).expect("recursive definition is well formed");
        let expression = KExpr::Call {
            definition_id: LOOP_ID,
            arguments: vec![KExpr::BytesLiteral(vec![0x41; INPUT_LEN])],
        };
        let result = EvaluationMachine::new(&evaluator, &[], 20_000)
            .expect("runtime machine allocation")
            .run(&expression)
            .expect("recursive byte traversal evaluates");

        assert_eq!(result.value, KValue::Bytes(Vec::new()));
        assert_eq!(result.peak_runtime_byte_backing, INPUT_LEN);
        assert!(result.peak_runtime_environments <= 3);
        assert_eq!(result.retained_runtime_byte_backing, 0);
        assert_eq!(result.live_runtime_environments, 0);
        let formerly_retained_suffix_bytes = INPUT_LEN * (INPUT_LEN + 1) / 2;
        assert!(
            result.peak_runtime_byte_backing * 1_000 < formerly_retained_suffix_bytes,
            "byte ranges must not retain the sum of all traversed suffixes"
        );
    }

    fn owned_literal(byte: u8) -> KExpr {
        owned_literal_with_len(byte, 4)
    }

    fn owned_literal_with_len(byte: u8, len: usize) -> KExpr {
        KExpr::ConcatBytes(vec![KExpr::BytesLiteral(vec![byte; len])])
    }

    fn owned_runtime_bytes<'a>(
        store: &mut RuntimeByteStore<'a>,
        byte: u8,
        len: usize,
    ) -> RuntimeValue<'a> {
        RuntimeValue::Bytes(
            store
                .owned(vec![byte; len])
                .expect("test runtime bytes fit"),
        )
    }

    fn live_scan_stack<'a>() -> Vec<LiveScanFrame<'a>> {
        let mut stack = Vec::new();
        stack
            .try_reserve_exact(MAX_EXPRESSION_DEPTH)
            .expect("test liveness stack allocation");
        stack
    }

    fn nested_owned_values(final_body: KExpr) -> KExpr {
        KExpr::Let {
            value: boxed(owned_literal(0x11)),
            body: boxed(KExpr::Let {
                value: boxed(owned_literal(0x22)),
                body: boxed(KExpr::Let {
                    value: boxed(owned_literal(0x33)),
                    body: boxed(final_body),
                }),
            }),
        }
    }

    #[test]
    fn allocation_preflight_reclaims_dead_lexical_values() {
        let evaluator = Evaluator::new(&[]).expect("empty evaluator is well formed");
        let expression = nested_owned_values(KExpr::BytesLiteral(Vec::new()));
        let mut machine =
            EvaluationMachine::new(&evaluator, &[], 100).expect("runtime machine allocation");
        machine.byte_store.owned_byte_limit = 8;

        let result = machine
            .run(&expression)
            .expect("dead lexical byte values are reclaimable");

        assert_eq!(result.value, KValue::Bytes(Vec::new()));
        assert_eq!(result.retained_runtime_byte_backing, 0);
        assert_eq!(result.live_runtime_environments, 0);
    }

    #[test]
    fn allocation_preflight_preserves_live_lexical_values() {
        let evaluator = Evaluator::new(&[]).expect("empty evaluator is well formed");
        let expression = nested_owned_values(KExpr::ConcatBytes(vec![
            KExpr::Var(2),
            KExpr::Var(1),
            KExpr::Var(0),
        ]));
        let mut machine =
            EvaluationMachine::new(&evaluator, &[], 100).expect("runtime machine allocation");
        machine.byte_store.owned_byte_limit = 8;

        assert!(matches!(
            machine.run(&expression),
            Err(EvalError::ResourceExhausted)
        ));
    }

    #[test]
    fn nested_binder_offsets_mark_the_exact_outer_slot() {
        let base = Vec::<KValue>::new();
        let mut environments = RuntimeEnvironments::new(&base).expect("base environment");
        let mut store = RuntimeByteStore::with_owned_byte_limit(16);
        let first = owned_runtime_bytes(&mut store, 0x11, 2);
        let second = owned_runtime_bytes(&mut store, 0x22, 2);
        let environment = environments
            .extend(vec![first, second], None)
            .expect("owned environment");
        let expression = KExpr::Let {
            value: boxed(KExpr::BytesLiteral(Vec::new())),
            body: boxed(KExpr::Var(2)),
        };
        let epoch = environments.begin_live_epoch();
        let mut stack = live_scan_stack();

        mark_live_expression(
            &mut environments,
            epoch,
            &mut stack,
            LiveExpression {
                expression: &expression,
                bound: 0,
                environment,
            },
        )
        .expect("nested binder scan");
        environments
            .discard_unmarked(epoch, &mut store)
            .expect("dead slot release");

        assert!(environments.get(environment, 0).is_none());
        assert!(environments.get(environment, 1).is_some());
        assert_eq!(store.retained_owned_bytes, 2);
    }

    #[test]
    fn pending_sibling_keeps_outer_value_live_during_preflight() {
        let evaluator = Evaluator::new(&[]).expect("empty evaluator is well formed");
        let expression = KExpr::Let {
            value: boxed(owned_literal(0x11)),
            body: boxed(KExpr::ConcatBytes(vec![
                owned_literal_with_len(0x22, 5),
                KExpr::Var(0),
            ])),
        };
        let mut machine =
            EvaluationMachine::new(&evaluator, &[], 100).expect("runtime machine allocation");
        machine.byte_store.owned_byte_limit = 8;

        assert!(matches!(
            machine.run(&expression),
            Err(EvalError::ResourceExhausted)
        ));
    }

    #[test]
    fn pending_case_term_marks_both_branch_roots_before_selection() {
        let base = Vec::<KValue>::new();
        let mut environments = RuntimeEnvironments::new(&base).expect("base environment");
        let mut store = RuntimeByteStore::with_owned_byte_limit(16);
        let atom_only = owned_runtime_bytes(&mut store, 0x11, 2);
        let triple_only = owned_runtime_bytes(&mut store, 0x22, 2);
        let dead = owned_runtime_bytes(&mut store, 0x33, 2);
        let environment = environments
            .extend(vec![atom_only, triple_only, dead], None)
            .expect("owned environment");
        let atom_body = KExpr::Var(3);
        let triple_body = KExpr::Var(4);
        let task = EvalTask::CaseTerm {
            environment,
            atom_body: &atom_body,
            triple_body: &triple_body,
        };
        let epoch = environments.begin_live_epoch();
        let mut stack = live_scan_stack();

        mark_task_live_slots(&mut environments, epoch, &mut stack, &task)
            .expect("pending CaseTerm roots scan");
        environments
            .discard_unmarked(epoch, &mut store)
            .expect("dead slot release");

        assert!(environments.get(environment, 0).is_some());
        assert!(environments.get(environment, 1).is_some());
        assert!(environments.get(environment, 2).is_none());
        assert_eq!(store.retained_owned_bytes, 4);
    }

    #[test]
    fn pending_case_bytes_cons_body_uses_two_binder_offset() {
        let base = Vec::<KValue>::new();
        let mut environments = RuntimeEnvironments::new(&base).expect("base environment");
        let mut store = RuntimeByteStore::with_owned_byte_limit(16);
        let dead = owned_runtime_bytes(&mut store, 0x11, 2);
        let live = owned_runtime_bytes(&mut store, 0x22, 2);
        let environment = environments
            .extend(vec![dead, live], None)
            .expect("owned environment");
        let empty_body = KExpr::BytesLiteral(Vec::new());
        let cons_body = KExpr::Var(3);
        let task = EvalTask::CaseBytes {
            environment,
            empty_body: &empty_body,
            cons_body: &cons_body,
        };
        let epoch = environments.begin_live_epoch();
        let mut stack = live_scan_stack();

        mark_task_live_slots(&mut environments, epoch, &mut stack, &task)
            .expect("pending CaseBytes roots scan");
        environments
            .discard_unmarked(epoch, &mut store)
            .expect("dead slot release");

        assert!(environments.get(environment, 0).is_none());
        assert!(environments.get(environment, 1).is_some());
        assert_eq!(store.retained_owned_bytes, 2);
    }

    #[test]
    fn selected_case_term_branch_does_not_retain_the_dead_branch_dependency() {
        let evaluator = Evaluator::new(&[]).expect("empty evaluator is well formed");
        let expression = KExpr::Let {
            value: boxed(owned_literal(0x55)),
            body: boxed(KExpr::CaseTerm {
                scrutinee: boxed(KExpr::MakeAtom {
                    kind: boxed(KExpr::BytesLiteral(vec![0x01])),
                    payload: boxed(KExpr::BytesLiteral(vec![0x02; 2])),
                    equality: boxed(KExpr::BytesLiteral(vec![0x03; 2])),
                }),
                atom_body: boxed(KExpr::Var(0)),
                triple_body: boxed(KExpr::Var(3)),
            }),
        };
        let mut machine =
            EvaluationMachine::new(&evaluator, &[], 100).expect("runtime machine allocation");
        machine.byte_store.owned_byte_limit = 5;

        let result = machine
            .run(&expression)
            .expect("selected atom branch releases the triple-only dependency");

        assert_eq!(result.value, KValue::Bytes(vec![0x01]));
        assert_eq!(result.liveness_reclamation_runs, 1);
        assert!(result.peak_liveness_scan_depth <= MAX_EXPRESSION_DEPTH);
    }

    #[test]
    fn low_pressure_reclamation_preserves_results_and_matches_unpressured_evaluation() {
        let evaluator = Evaluator::new(&[]).expect("empty evaluator is well formed");
        let expression = KExpr::Let {
            value: boxed(owned_literal(0x11)),
            body: boxed(KExpr::Let {
                value: boxed(owned_literal(0x22)),
                body: boxed(KExpr::ConcatBytes(vec![
                    KExpr::Var(0),
                    owned_literal_with_len(0x33, 5),
                    KExpr::BytesLiteral(Vec::new()),
                ])),
            }),
        };
        let high = EvaluationMachine::new(&evaluator, &[], 100)
            .expect("unpressured machine allocation")
            .run(&expression)
            .expect("unpressured evaluation");
        let mut low_machine =
            EvaluationMachine::new(&evaluator, &[], 100).expect("pressured machine allocation");
        low_machine.byte_store.owned_byte_limit = 9;
        let low = low_machine
            .run(&expression)
            .expect("result-stack ownership survives environment reclamation");

        assert_eq!(low.value, high.value);
        assert_eq!(low.fuel, high.fuel);
        assert_eq!(low.observations, high.observations);
        assert_eq!(
            low.value,
            KValue::Bytes([vec![0x22; 4], vec![0x33; 5]].concat())
        );
        assert!(low.liveness_reclamation_runs >= 1);
        assert_eq!(high.liveness_reclamation_runs, 0);
    }

    #[test]
    fn owned_atom_components_are_preflighted_as_one_allocation() {
        let evaluator = Evaluator::new(&[]).expect("empty evaluator is well formed");
        let mut machine =
            EvaluationMachine::new(&evaluator, &[], 100).expect("runtime machine allocation");
        machine.byte_store.owned_byte_limit = 8;
        machine
            .push_result(RuntimeValue::Term(RuntimeTerm::Owned(Term::Atom {
                kind: vec![0x01; 3],
                canonical_payload: vec![0x02; 3],
                equality_contract: vec![0x03; 3],
            })))
            .expect("owned atom result");
        let atom_body = KExpr::BytesLiteral(Vec::new());
        let triple_body = KExpr::BytesLiteral(Vec::new());

        assert!(matches!(
            machine.continue_case_term(0, &atom_body, &triple_body),
            Err(EvalError::ResourceExhausted)
        ));
        assert_eq!(machine.byte_store.retained_owned_bytes, 0);
        assert_eq!(machine.liveness_reclamation_runs, 1);
    }

    #[test]
    fn wide_liveness_scan_uses_depth_bounded_reusable_scratch() {
        let base = Vec::<KValue>::new();
        let mut environments = RuntimeEnvironments::new(&base).expect("base environment");
        let expression = KExpr::ConcatBytes(
            (0..4_096)
                .map(|_| KExpr::BytesLiteral(Vec::new()))
                .collect(),
        );
        let epoch = environments.begin_live_epoch();
        let mut stack = live_scan_stack();

        let peak = mark_live_expression(
            &mut environments,
            epoch,
            &mut stack,
            LiveExpression {
                expression: &expression,
                bound: 0,
                environment: 0,
            },
        )
        .expect("wide expression scan");

        assert_eq!(peak, 2);
        assert!(stack.is_empty());
    }

    #[test]
    fn liveness_epoch_wrap_clears_stale_slot_marks() {
        let base = Vec::<KValue>::new();
        let mut environments = RuntimeEnvironments::new(&base).expect("base environment");
        let mut store = RuntimeByteStore::with_owned_byte_limit(8);
        let value = owned_runtime_bytes(&mut store, 0x44, 4);
        let environment = environments
            .extend(vec![value], None)
            .expect("owned environment");
        environments.live_epoch = u64::MAX;
        let RuntimeValues::Owned(values) = &mut environments
            .entry_mut(environment)
            .expect("owned environment entry")
            .values
        else {
            panic!("test environment must own its slot")
        };
        values[0].live_epoch = u64::MAX;

        let epoch = environments.begin_live_epoch();
        assert_eq!(epoch, 1);
        environments
            .discard_unmarked(epoch, &mut store)
            .expect("stale mark release");

        assert!(environments.get(environment, 0).is_none());
        assert_eq!(store.retained_owned_bytes, 0);
    }

    #[test]
    fn liveness_scan_operates_on_a_small_host_stack() {
        std::thread::Builder::new()
            .name("clcp-liveness-small-stack".to_owned())
            .stack_size(256 * 1024)
            .spawn(|| {
                let mut expression = KExpr::BytesLiteral(Vec::new());
                for _ in 0..400 {
                    expression = KExpr::Let {
                        value: boxed(KExpr::BytesLiteral(Vec::new())),
                        body: boxed(expression),
                    };
                }
                expression
                    .validate_resource_bounds()
                    .expect("deep test expression stays within wire bounds");
                let base = Vec::<KValue>::new();
                let mut environments = RuntimeEnvironments::new(&base).expect("base environment");
                let epoch = environments.begin_live_epoch();
                let mut stack = live_scan_stack();
                let peak = mark_live_expression(
                    &mut environments,
                    epoch,
                    &mut stack,
                    LiveExpression {
                        expression: &expression,
                        bound: 0,
                        environment: 0,
                    },
                )
                .expect("iterative liveness scan");
                assert_eq!(peak, 401);
            })
            .expect("small-stack test thread starts")
            .join()
            .expect("small-stack liveness scan completes");
    }

    #[test]
    fn owned_runtime_byte_limit_fails_deterministically_and_recovers() {
        let mut store = RuntimeByteStore::with_owned_byte_limit(3);
        let retained = store
            .owned(vec![0x01, 0x02])
            .expect("first bounded allocation fits");
        assert!(matches!(
            store.owned(vec![0x03, 0x04]),
            Err(EvalError::ResourceExhausted)
        ));
        assert_eq!(store.retained_owned_bytes, 2);

        store.release(retained).expect("retained bytes release");
        assert_eq!(store.retained_owned_bytes, 0);
        let replacement = store
            .owned(vec![0x03, 0x04, 0x05])
            .expect("released capacity is reusable");
        store.release(replacement).expect("replacement releases");
    }
}
