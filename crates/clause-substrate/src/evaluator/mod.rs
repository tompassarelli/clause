//! Construct-blind evaluator and deterministic certificate producer for the
//! twelve fixed CLCP-v2 `KExpr` forms.

use std::fmt;

use crate::compiler_package_v2::{
    Definition, EvalCertificate, EvalJudgment, EvalNode, EvalOutcome, EvalStatement, Hash32, Id32,
    KExpr, KSort, KValue, MAX_CERTIFICATE_NODES, MAX_EVALUATION_FRAMES, MAX_EXPRESSION_DEPTH,
    MAX_WIRE_BYTES, MAX_WIRE_ITEMS, Term, sha256_operation_id, try_copy_bytes,
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
    CertificateNodeOverflow,
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
            Self::CertificateNodeOverflow => {
                formatter.write_str("certificate node budget or U32 index was exceeded")
            }
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
}

impl<'a> DefinitionTable<'a> {
    fn new(definitions: &'a [Definition]) -> Result<Self, StaticError> {
        if definitions.len() > MAX_WIRE_ITEMS {
            return Err(StaticError::ResourceExhausted);
        }
        for (index, pair) in definitions.windows(2).enumerate() {
            if pair[0].id == pair[1].id {
                return Err(StaticError::DuplicateDefinition(pair[0].id));
            }
            if pair[0].id > pair[1].id {
                return Err(StaticError::DefinitionsNotStrictlySorted { index: index + 1 });
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
            .map(|index| &self.ordered[index])
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
            if definition.arguments.len() > MAX_WIRE_ITEMS {
                return Err(StaticError::ResourceExhausted);
            }
            definition
                .body
                .validate_resource_bounds()
                .map_err(|_| StaticError::ResourceExhausted)?;
            let actual = self.infer(&definition.body, &definition.arguments)?;
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
        self.infer(expression, environment)
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
        let result = EvaluationMachine::new(self, environment, fuel, false)?.run(expression)?;
        Ok(Evaluation {
            value: result.value,
            remaining_fuel: result.fuel,
            observations: result.observations,
        })
    }

    /// Evaluate one generic entrypoint invocation and produce its canonical
    /// postorder rule DAG. This constructs evidence only; it grants no package
    /// or predecessor authority.
    pub fn build_certificate(
        &self,
        context: CertificateContext,
    ) -> Result<EvalCertificate, EvalError> {
        if context.exact_accepted_predecessor.len() > MAX_WIRE_BYTES
            || context.arguments.len() > MAX_WIRE_ITEMS
        {
            return Err(EvalError::ResourceExhausted);
        }
        let mut arguments = Vec::new();
        arguments
            .try_reserve_exact(context.arguments.len())
            .map_err(|_| EvalError::ResourceExhausted)?;
        for value in &context.arguments {
            value
                .validate_resource_bounds()
                .map_err(|_| EvalError::ResourceExhausted)?;
            arguments.push(value_literal(value)?);
        }
        let expression = KExpr::Call {
            definition_id: context.entrypoint,
            arguments,
        };
        self.infer_sort(&expression, &[])?;
        let result =
            EvaluationMachine::new(self, &[], context.fuel_limit, true)?.run(&expression)?;
        let observations = result.observations.try_to_term()?;
        let statement = EvalStatement {
            exact_accepted_predecessor: context.exact_accepted_predecessor,
            core_contract_id: context.core_contract_id,
            physical_profile_id: context.physical_profile_id,
            entrypoint: context.entrypoint,
            arguments: context.arguments,
            fuel_limit: context.fuel_limit,
            expected: EvalOutcome::Returned {
                value: result.value,
                remaining_fuel: result.fuel,
                observations,
            },
        };
        Ok(EvalCertificate {
            format_version: 0x00,
            statement,
            nodes: result.nodes,
        })
    }

    fn infer(&self, expression: &KExpr, environment: &[KSort]) -> Result<KSort, StaticError> {
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
                            reserve_infer_tasks(&mut tasks, 3)?;
                            tasks.push(InferTask::Return(KSort::Bytes));
                            tasks.push(InferTask::Require(KSort::Bytes));
                            tasks.push(InferTask::Expression {
                                expression: &arguments[0],
                                environment,
                                depth: next,
                            });
                        }
                    }
                }
                InferTask::Require(expected) => {
                    let actual = results.pop().ok_or(StaticError::ResourceExhausted)?;
                    require_sort(expected, actual)?;
                }
                InferTask::Return(sort) => push_sort(&mut results, sort)?,
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
            index -= entry.values.len();
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

enum RuntimeValues<'a> {
    Borrowed(&'a [KValue]),
    Owned(Vec<KValue>),
}

impl RuntimeValues<'_> {
    fn as_slice(&self) -> &[KValue] {
        match self {
            Self::Borrowed(values) => values,
            Self::Owned(values) => values,
        }
    }
}

struct RuntimeEnvironment<'a> {
    values: RuntimeValues<'a>,
    parent: Option<usize>,
    total_len: usize,
}

struct RuntimeEnvironments<'a> {
    entries: Vec<RuntimeEnvironment<'a>>,
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
        entries.push(RuntimeEnvironment {
            values: RuntimeValues::Borrowed(values),
            parent: None,
            total_len: values.len(),
        });
        Ok(Self { entries })
    }

    fn extend(&mut self, values: Vec<KValue>, parent: Option<usize>) -> Result<usize, EvalError> {
        let parent_len = match parent {
            Some(index) => {
                self.entries
                    .get(index)
                    .ok_or(EvalError::ResourceExhausted)?
                    .total_len
            }
            None => 0,
        };
        let total_len = parent_len
            .checked_add(values.len())
            .ok_or(EvalError::ResourceExhausted)?;
        if total_len > MAX_WIRE_ITEMS || self.entries.len() >= MAX_CERTIFICATE_NODES {
            return Err(EvalError::ResourceExhausted);
        }
        self.entries
            .try_reserve(1)
            .map_err(|_| EvalError::ResourceExhausted)?;
        let index = self.entries.len();
        self.entries.push(RuntimeEnvironment {
            values: RuntimeValues::Owned(values),
            parent,
            total_len,
        });
        Ok(index)
    }

    fn get(&self, mut environment: usize, mut index: usize) -> Option<&KValue> {
        loop {
            let entry = self.entries.get(environment)?;
            let values = entry.values.as_slice();
            if index < values.len() {
                return values.get(index);
            }
            index -= values.len();
            environment = entry.parent?;
        }
    }

    fn try_flatten(&self, mut environment: usize) -> Result<Vec<KValue>, EvalError> {
        let total_len = self
            .entries
            .get(environment)
            .ok_or(EvalError::ResourceExhausted)?
            .total_len;
        let mut values = Vec::new();
        values
            .try_reserve_exact(total_len)
            .map_err(|_| EvalError::ResourceExhausted)?;
        loop {
            let entry = self
                .entries
                .get(environment)
                .ok_or(EvalError::ResourceExhausted)?;
            for value in entry.values.as_slice() {
                values.push(
                    value
                        .try_clone_resource()
                        .map_err(|_| EvalError::ResourceExhausted)?,
                );
            }
            let Some(parent) = entry.parent else {
                break;
            };
            environment = parent;
        }
        Ok(values)
    }
}

struct NodeContext<'a> {
    expression: &'a KExpr,
    environment: usize,
    fuel_before: u64,
    observations_before: Option<Term>,
}

enum EvalTask<'a> {
    Expression {
        expression: &'a KExpr,
        environment: usize,
    },
    MakeAtom(NodeContext<'a>),
    MakeTriple(NodeContext<'a>),
    Let {
        context: NodeContext<'a>,
        body: &'a KExpr,
    },
    CaseTerm {
        context: NodeContext<'a>,
        atom_body: &'a KExpr,
        triple_body: &'a KExpr,
    },
    CaseBytes {
        context: NodeContext<'a>,
        empty_body: &'a KExpr,
        cons_body: &'a KExpr,
    },
    ConcatBytes {
        context: NodeContext<'a>,
        count: usize,
    },
    CaseBytesEqual {
        context: NodeContext<'a>,
        equal_body: &'a KExpr,
        unequal_body: &'a KExpr,
    },
    Call {
        context: NodeContext<'a>,
        definition: &'a Definition,
        argument_count: usize,
    },
    Request {
        context: NodeContext<'a>,
        operation_id: Id32,
    },
    Passthrough {
        context: NodeContext<'a>,
        rule_tag: u8,
        leading_premises: Vec<u32>,
    },
}

struct RuntimeResult {
    value: KValue,
    premise: Option<u32>,
}

struct MachineResult {
    value: KValue,
    fuel: u64,
    observations: ObservationLog,
    nodes: Vec<EvalNode>,
}

struct EvaluationMachine<'a> {
    evaluator: &'a Evaluator<'a>,
    environments: RuntimeEnvironments<'a>,
    tasks: Vec<EvalTask<'a>>,
    results: Vec<RuntimeResult>,
    nodes: Vec<EvalNode>,
    fuel: u64,
    observations: ObservationLog,
    steps: usize,
    certificate: bool,
}

impl<'a> EvaluationMachine<'a> {
    fn new(
        evaluator: &'a Evaluator<'a>,
        environment: &'a [KValue],
        fuel: u64,
        certificate: bool,
    ) -> Result<Self, EvalError> {
        Ok(Self {
            evaluator,
            environments: RuntimeEnvironments::new(environment)?,
            tasks: Vec::new(),
            results: Vec::new(),
            nodes: Vec::new(),
            fuel,
            observations: ObservationLog::default(),
            steps: 0,
            certificate,
        })
    }

    fn run(mut self, expression: &'a KExpr) -> Result<MachineResult, EvalError> {
        self.push_task(EvalTask::Expression {
            expression,
            environment: 0,
        })?;
        while let Some(task) = self.tasks.pop() {
            match task {
                EvalTask::Expression {
                    expression,
                    environment,
                } => self.enter(expression, environment)?,
                EvalTask::MakeAtom(context) => self.finish_make_atom(context)?,
                EvalTask::MakeTriple(context) => self.finish_make_triple(context)?,
                EvalTask::Let { context, body } => self.continue_let(context, body)?,
                EvalTask::CaseTerm {
                    context,
                    atom_body,
                    triple_body,
                } => self.continue_case_term(context, atom_body, triple_body)?,
                EvalTask::CaseBytes {
                    context,
                    empty_body,
                    cons_body,
                } => self.continue_case_bytes(context, empty_body, cons_body)?,
                EvalTask::ConcatBytes { context, count } => {
                    self.finish_concat(context, count)?;
                }
                EvalTask::CaseBytesEqual {
                    context,
                    equal_body,
                    unequal_body,
                } => self.continue_case_bytes_equal(context, equal_body, unequal_body)?,
                EvalTask::Call {
                    context,
                    definition,
                    argument_count,
                } => self.continue_call(context, definition, argument_count)?,
                EvalTask::Request {
                    context,
                    operation_id,
                } => self.finish_request(context, operation_id)?,
                EvalTask::Passthrough {
                    context,
                    rule_tag,
                    leading_premises,
                } => self.finish_passthrough(context, rule_tag, leading_premises)?,
            }
        }
        if self.results.len() != 1 {
            return Err(EvalError::ResourceExhausted);
        }
        let value = self
            .results
            .pop()
            .ok_or(EvalError::ResourceExhausted)?
            .value;
        Ok(MachineResult {
            value,
            fuel: self.fuel,
            observations: self.observations,
            nodes: self.nodes,
        })
    }

    fn enter(&mut self, expression: &'a KExpr, environment: usize) -> Result<(), EvalError> {
        if self.steps >= MAX_CERTIFICATE_NODES {
            return Err(EvalError::CertificateNodeOverflow);
        }
        self.steps += 1;
        let fuel_before = self.fuel;
        let observations_before = if self.certificate {
            Some(self.observations.try_to_term()?)
        } else {
            None
        };
        self.fuel = self.fuel.checked_sub(1).ok_or(EvalError::OutOfFuel)?;
        let context = NodeContext {
            expression,
            environment,
            fuel_before,
            observations_before,
        };

        match expression {
            KExpr::BytesLiteral(bytes) => self.complete(
                context,
                0x30,
                empty_premises(),
                KValue::Bytes(try_copy_bytes(bytes).map_err(|_| EvalError::ResourceExhausted)?),
            ),
            KExpr::TermLiteral(term) => self.complete(
                context,
                0x31,
                empty_premises(),
                KValue::Term(
                    term.try_clone_resource()
                        .map_err(|_| EvalError::ResourceExhausted)?,
                ),
            ),
            KExpr::Var(index) => {
                let wire_index = *index;
                let index = usize::try_from(wire_index)
                    .map_err(|_| EvalError::VariableOutOfBounds(wire_index))?;
                let value = self
                    .environments
                    .get(environment, index)
                    .ok_or(EvalError::VariableOutOfBounds(wire_index))?
                    .try_clone_resource()
                    .map_err(|_| EvalError::ResourceExhausted)?;
                self.complete(context, 0x32, empty_premises(), value)
            }
            KExpr::MakeAtom {
                kind,
                payload,
                equality,
            } => {
                self.reserve_tasks(4)?;
                self.tasks.push(EvalTask::MakeAtom(context));
                self.tasks.push(EvalTask::Expression {
                    expression: equality,
                    environment,
                });
                self.tasks.push(EvalTask::Expression {
                    expression: payload,
                    environment,
                });
                self.tasks.push(EvalTask::Expression {
                    expression: kind,
                    environment,
                });
                Ok(())
            }
            KExpr::MakeTriple {
                first,
                second,
                third,
            } => {
                self.reserve_tasks(4)?;
                self.tasks.push(EvalTask::MakeTriple(context));
                self.tasks.push(EvalTask::Expression {
                    expression: third,
                    environment,
                });
                self.tasks.push(EvalTask::Expression {
                    expression: second,
                    environment,
                });
                self.tasks.push(EvalTask::Expression {
                    expression: first,
                    environment,
                });
                Ok(())
            }
            KExpr::Let { value, body } => {
                self.reserve_tasks(2)?;
                self.tasks.push(EvalTask::Let { context, body });
                self.tasks.push(EvalTask::Expression {
                    expression: value,
                    environment,
                });
                Ok(())
            }
            KExpr::CaseTerm {
                scrutinee,
                atom_body,
                triple_body,
            } => {
                self.reserve_tasks(2)?;
                self.tasks.push(EvalTask::CaseTerm {
                    context,
                    atom_body,
                    triple_body,
                });
                self.tasks.push(EvalTask::Expression {
                    expression: scrutinee,
                    environment,
                });
                Ok(())
            }
            KExpr::CaseBytes {
                scrutinee,
                empty_body,
                cons_body,
            } => {
                self.reserve_tasks(2)?;
                self.tasks.push(EvalTask::CaseBytes {
                    context,
                    empty_body,
                    cons_body,
                });
                self.tasks.push(EvalTask::Expression {
                    expression: scrutinee,
                    environment,
                });
                Ok(())
            }
            KExpr::ConcatBytes(parts) => {
                let additional = parts
                    .len()
                    .checked_add(1)
                    .ok_or(EvalError::ResourceExhausted)?;
                self.reserve_tasks(additional)?;
                self.tasks.push(EvalTask::ConcatBytes {
                    context,
                    count: parts.len(),
                });
                for part in parts.iter().rev() {
                    self.tasks.push(EvalTask::Expression {
                        expression: part,
                        environment,
                    });
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
                self.tasks.push(EvalTask::CaseBytesEqual {
                    context,
                    equal_body,
                    unequal_body,
                });
                self.tasks.push(EvalTask::Expression {
                    expression: right,
                    environment,
                });
                self.tasks.push(EvalTask::Expression {
                    expression: left,
                    environment,
                });
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
                self.tasks.push(EvalTask::Call {
                    context,
                    definition,
                    argument_count: arguments.len(),
                });
                for argument in arguments.iter().rev() {
                    self.tasks.push(EvalTask::Expression {
                        expression: argument,
                        environment,
                    });
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
                self.reserve_tasks(2)?;
                self.tasks.push(EvalTask::Request {
                    context,
                    operation_id: *physical_operation_id,
                });
                self.tasks.push(EvalTask::Expression {
                    expression: &arguments[0],
                    environment,
                });
                Ok(())
            }
        }
    }

    fn finish_make_atom(&mut self, context: NodeContext<'a>) -> Result<(), EvalError> {
        let equality = self.pop_result()?;
        let payload = self.pop_result()?;
        let kind = self.pop_result()?;
        let premises = self.premises(&[&kind, &payload, &equality])?;
        let value = KValue::Term(Term::Atom {
            kind: expect_bytes(kind.value)?,
            canonical_payload: expect_bytes(payload.value)?,
            equality_contract: expect_bytes(equality.value)?,
        });
        self.complete(context, 0x33, premises, value)
    }

    fn finish_make_triple(&mut self, context: NodeContext<'a>) -> Result<(), EvalError> {
        let third = self.pop_result()?;
        let second = self.pop_result()?;
        let first = self.pop_result()?;
        let premises = self.premises(&[&first, &second, &third])?;
        let term = Term::try_triple(
            expect_term(first.value)?,
            expect_term(second.value)?,
            expect_term(third.value)?,
        )
        .map_err(|_| EvalError::ResourceExhausted)?;
        self.complete(context, 0x34, premises, KValue::Term(term))
    }

    fn continue_let(&mut self, context: NodeContext<'a>, body: &'a KExpr) -> Result<(), EvalError> {
        let value = self.pop_result()?;
        let leading_premises = self.premises(&[&value])?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(1)
            .map_err(|_| EvalError::ResourceExhausted)?;
        values.push(value.value);
        let environment = self
            .environments
            .extend(values, Some(context.environment))?;
        self.reserve_tasks(2)?;
        self.tasks.push(EvalTask::Passthrough {
            context,
            rule_tag: 0x35,
            leading_premises,
        });
        self.tasks.push(EvalTask::Expression {
            expression: body,
            environment,
        });
        Ok(())
    }

    fn continue_case_term(
        &mut self,
        context: NodeContext<'a>,
        atom_body: &'a KExpr,
        triple_body: &'a KExpr,
    ) -> Result<(), EvalError> {
        let scrutinee = self.pop_result()?;
        let leading_premises = self.premises(&[&scrutinee])?;
        let (rule_tag, body, values) = match expect_term(scrutinee.value)? {
            Term::Atom {
                kind,
                canonical_payload,
                equality_contract,
            } => {
                let mut values = Vec::new();
                values
                    .try_reserve_exact(3)
                    .map_err(|_| EvalError::ResourceExhausted)?;
                values.push(KValue::Bytes(kind));
                values.push(KValue::Bytes(canonical_payload));
                values.push(KValue::Bytes(equality_contract));
                (0x36, atom_body, values)
            }
            Term::Triple(first, second, third) => {
                let mut values = Vec::new();
                values
                    .try_reserve_exact(3)
                    .map_err(|_| EvalError::ResourceExhausted)?;
                values.push(KValue::Term(*first));
                values.push(KValue::Term(*second));
                values.push(KValue::Term(*third));
                (0x37, triple_body, values)
            }
        };
        let environment = self
            .environments
            .extend(values, Some(context.environment))?;
        self.reserve_tasks(2)?;
        self.tasks.push(EvalTask::Passthrough {
            context,
            rule_tag,
            leading_premises,
        });
        self.tasks.push(EvalTask::Expression {
            expression: body,
            environment,
        });
        Ok(())
    }

    fn continue_case_bytes(
        &mut self,
        context: NodeContext<'a>,
        empty_body: &'a KExpr,
        cons_body: &'a KExpr,
    ) -> Result<(), EvalError> {
        let scrutinee = self.pop_result()?;
        let leading_premises = self.premises(&[&scrutinee])?;
        let mut bytes = expect_bytes(scrutinee.value)?;
        let (rule_tag, body, environment) = if bytes.is_empty() {
            (0x38, empty_body, context.environment)
        } else {
            let head = bytes.remove(0);
            let mut head_bytes = Vec::new();
            head_bytes
                .try_reserve_exact(1)
                .map_err(|_| EvalError::ResourceExhausted)?;
            head_bytes.push(head);
            let mut values = Vec::new();
            values
                .try_reserve_exact(2)
                .map_err(|_| EvalError::ResourceExhausted)?;
            values.push(KValue::Bytes(head_bytes));
            values.push(KValue::Bytes(bytes));
            let environment = self
                .environments
                .extend(values, Some(context.environment))?;
            (0x39, cons_body, environment)
        };
        self.reserve_tasks(2)?;
        self.tasks.push(EvalTask::Passthrough {
            context,
            rule_tag,
            leading_premises,
        });
        self.tasks.push(EvalTask::Expression {
            expression: body,
            environment,
        });
        Ok(())
    }

    fn finish_concat(&mut self, context: NodeContext<'a>, count: usize) -> Result<(), EvalError> {
        let results = self.take_results(count)?;
        let premises = self.premises_from_slice(&results)?;
        let mut total = 0_usize;
        for result in &results {
            let KValue::Bytes(bytes) = &result.value else {
                return Err(EvalError::ValueSort {
                    expected: KSort::Bytes,
                    actual: KSort::Term,
                });
            };
            total = total
                .checked_add(bytes.len())
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
            let KValue::Bytes(part) = result.value else {
                return Err(EvalError::ValueSort {
                    expected: KSort::Bytes,
                    actual: KSort::Term,
                });
            };
            bytes.extend_from_slice(&part);
        }
        self.complete(context, 0x3a, premises, KValue::Bytes(bytes))
    }

    fn continue_case_bytes_equal(
        &mut self,
        context: NodeContext<'a>,
        equal_body: &'a KExpr,
        unequal_body: &'a KExpr,
    ) -> Result<(), EvalError> {
        let right = self.pop_result()?;
        let left = self.pop_result()?;
        let leading_premises = self.premises(&[&left, &right])?;
        let left = expect_bytes(left.value)?;
        let right = expect_bytes(right.value)?;
        let (rule_tag, body) = if left == right {
            (0x3b, equal_body)
        } else {
            (0x3c, unequal_body)
        };
        let environment = context.environment;
        self.reserve_tasks(2)?;
        self.tasks.push(EvalTask::Passthrough {
            context,
            rule_tag,
            leading_premises,
        });
        self.tasks.push(EvalTask::Expression {
            expression: body,
            environment,
        });
        Ok(())
    }

    fn continue_call(
        &mut self,
        context: NodeContext<'a>,
        definition: &'a Definition,
        argument_count: usize,
    ) -> Result<(), EvalError> {
        let results = self.take_results(argument_count)?;
        let leading_premises = self.premises_from_slice(&results)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(argument_count)
            .map_err(|_| EvalError::ResourceExhausted)?;
        for (result, expected) in results.into_iter().zip(&definition.arguments) {
            require_value_sort(*expected, &result.value)?;
            values.push(result.value);
        }
        let environment = self.environments.extend(values, None)?;
        self.reserve_tasks(2)?;
        self.tasks.push(EvalTask::Passthrough {
            context,
            rule_tag: 0x3d,
            leading_premises,
        });
        self.tasks.push(EvalTask::Expression {
            expression: &definition.body,
            environment,
        });
        Ok(())
    }

    fn finish_request(
        &mut self,
        context: NodeContext<'a>,
        operation_id: Id32,
    ) -> Result<(), EvalError> {
        let argument = self.pop_result()?;
        require_value_sort(KSort::Bytes, &argument.value)?;
        let premises = self.premises(&[&argument])?;
        let mut arguments = Vec::new();
        arguments
            .try_reserve_exact(1)
            .map_err(|_| EvalError::ResourceExhausted)?;
        arguments.push(argument.value);
        let value =
            self.evaluator
                .physical
                .request(operation_id, arguments, &mut self.observations)?;
        self.complete(context, 0x3e, premises, value)
    }

    fn finish_passthrough(
        &mut self,
        context: NodeContext<'a>,
        rule_tag: u8,
        mut leading_premises: Vec<u32>,
    ) -> Result<(), EvalError> {
        let body = self.pop_result()?;
        if let Some(premise) = body.premise {
            leading_premises
                .try_reserve(1)
                .map_err(|_| EvalError::ResourceExhausted)?;
            leading_premises.push(premise);
        }
        self.complete(context, rule_tag, leading_premises, body.value)
    }

    fn complete(
        &mut self,
        context: NodeContext<'a>,
        rule_tag: u8,
        premises: Vec<u32>,
        value: KValue,
    ) -> Result<(), EvalError> {
        value
            .validate_resource_bounds()
            .map_err(|_| EvalError::ResourceExhausted)?;
        let premise = if self.certificate {
            if self.nodes.len() >= MAX_CERTIFICATE_NODES {
                return Err(EvalError::CertificateNodeOverflow);
            }
            let index =
                u32::try_from(self.nodes.len()).map_err(|_| EvalError::CertificateNodeOverflow)?;
            self.nodes
                .try_reserve(1)
                .map_err(|_| EvalError::ResourceExhausted)?;
            let expression = context
                .expression
                .try_clone_resource()
                .map_err(|_| EvalError::ResourceExhausted)?;
            let environment = self.environments.try_flatten(context.environment)?;
            let observations_before = context
                .observations_before
                .ok_or(EvalError::ResourceExhausted)?;
            let conclusion_value = value
                .try_clone_resource()
                .map_err(|_| EvalError::ResourceExhausted)?;
            let observations_after = self.observations.try_to_term()?;
            self.nodes.push(EvalNode {
                rule_tag,
                premises,
                conclusion: EvalJudgment {
                    expression,
                    environment,
                    fuel_before: context.fuel_before,
                    observations_before,
                    value: conclusion_value,
                    fuel_after: self.fuel,
                    observations_after,
                },
            });
            Some(index)
        } else {
            None
        };
        self.results
            .try_reserve(1)
            .map_err(|_| EvalError::ResourceExhausted)?;
        self.results.push(RuntimeResult { value, premise });
        Ok(())
    }

    fn pop_result(&mut self) -> Result<RuntimeResult, EvalError> {
        self.results.pop().ok_or(EvalError::ResourceExhausted)
    }

    fn take_results(&mut self, count: usize) -> Result<Vec<RuntimeResult>, EvalError> {
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

    fn premises(&self, results: &[&RuntimeResult]) -> Result<Vec<u32>, EvalError> {
        let mut premises = Vec::new();
        if self.certificate {
            premises
                .try_reserve_exact(results.len())
                .map_err(|_| EvalError::ResourceExhausted)?;
            for result in results {
                premises.push(result.premise.ok_or(EvalError::ResourceExhausted)?);
            }
        }
        Ok(premises)
    }

    fn premises_from_slice(&self, results: &[RuntimeResult]) -> Result<Vec<u32>, EvalError> {
        let mut premises = Vec::new();
        if self.certificate {
            premises
                .try_reserve_exact(results.len())
                .map_err(|_| EvalError::ResourceExhausted)?;
            for result in results {
                premises.push(result.premise.ok_or(EvalError::ResourceExhausted)?);
            }
        }
        Ok(premises)
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
        self.tasks.push(task);
        Ok(())
    }
}

fn empty_premises() -> Vec<u32> {
    Vec::new()
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
