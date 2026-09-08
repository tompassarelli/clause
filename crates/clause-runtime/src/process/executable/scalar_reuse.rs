//! Lazy reuse of equal scalar subexpressions within one fixed substitution.
use super::*;
mod compiled_numeric;

pub(super) struct ScalarPlan {
    pub expression: Box<ExecutableExpressionV1>,
    nodes: Vec<(Node, bool, bool)>,
    root: usize,
    instructions: Vec<Instruction>,
    numeric_query: std::sync::OnceLock<Option<Arc<compiled_numeric::NumericQuery>>>,
}

enum Node {
    Value(Box<ExecutableExpressionV1>),
    Sum { expression: Box<ExecutableExpressionV1>, inputs: Vec<usize>, shape: Arc<[u8]> },
    Number(u64), Boolean(bool), Slot(u16), Argument(u16), Binding(u16),
    Add(usize, usize), Subtract(usize, usize), Multiply(usize, usize), Divide(usize, usize),
    GreaterThan(usize, usize), LessThanOrEqual(usize, usize), Equal(usize, usize), And(usize, usize),
    Not(usize), SquareRoot(usize), Conditional(usize, usize, usize), Clamp(usize, usize, usize),
}

enum Instruction {
    Cached { node: usize, end: usize },
    Evaluate(usize),
    Number(usize),
    Boolean(usize),
    Nonzero(usize),
    BranchFalse { condition: usize, target: usize },
    BranchTrue { condition: usize, target: usize },
    Jump(usize),
    Copy { from: usize, to: usize },
}

fn patch_branches(code: &mut [Instruction], positions: &[usize], target: usize) {
    for position in positions {
        match &mut code[*position] {
            Instruction::BranchFalse { target: to, .. }
            | Instruction::BranchTrue { target: to, .. }
            | Instruction::Jump(to) => *to = target,
            _ => unreachable!(),
        }
    }
}

fn boolean_node(index: usize, nodes: &[(Node, bool, bool)]) -> bool {
    match nodes[index].0 {
        Node::Boolean(_) | Node::GreaterThan(..) | Node::LessThanOrEqual(..)
        | Node::Equal(..) | Node::And(..) | Node::Not(_) => true,
        Node::Conditional(_, yes, no) => boolean_node(yes, nodes) && boolean_node(no, nodes),
        _ => false,
    }
}

fn emit_scalar_branch(index: usize, when: bool, nodes: &[(Node, bool, bool)],
    code: &mut Vec<Instruction>, exits: &mut Vec<usize>) {
    let constant = match nodes[index].0 {
        Node::Boolean(value) => Some(value),
        Node::GreaterThan(a, b) | Node::LessThanOrEqual(a, b) => {
            match (&nodes[a].0, &nodes[b].0) {
                (Node::Number(a), Node::Number(b)) => Some(if matches!(nodes[index].0, Node::GreaterThan(..)) {
                    f64::from_bits(*a) > f64::from_bits(*b)
                } else { f64::from_bits(*a) <= f64::from_bits(*b) }),
                _ => None,
            }
        },
        _ => None,
    };
    if let Some(value) = constant {
        if value == when { exits.push(code.len()); code.push(Instruction::Jump(0)); }
        return;
    }
    match nodes[index].0 {
        Node::Conditional(condition, yes, no) if !nodes[index].1 => {
            let mut otherwise = Vec::new();
            emit_scalar_branch(condition, false, nodes, code, &mut otherwise);
            emit_scalar_branch(yes, when, nodes, code, exits);
            let end = code.len(); code.push(Instruction::Jump(0));
            let target = code.len(); patch_branches(code, &otherwise, target);
            emit_scalar_branch(no, when, nodes, code, exits);
            code[end] = Instruction::Jump(code.len());
        },
        Node::Not(value) if !nodes[index].1 => emit_scalar_branch(value, !when, nodes, code, exits),
        Node::Equal(a, b) if !nodes[index].1 && matches!(nodes[b].0, Node::Boolean(_)) && boolean_node(a, nodes) => {
            let Node::Boolean(value) = nodes[b].0 else { unreachable!() };
            emit_scalar_branch(a, when == value, nodes, code, exits);
        },
        Node::Equal(a, b) if !nodes[index].1 && matches!(nodes[a].0, Node::Boolean(_)) && boolean_node(b, nodes) => {
            let Node::Boolean(value) = nodes[a].0 else { unreachable!() };
            emit_scalar_branch(b, when == value, nodes, code, exits);
        },
        _ => {
            emit_scalar_instructions(index, nodes, code);
            exits.push(code.len());
            code.push(if when { Instruction::BranchTrue { condition: index, target: 0 } }
                else { Instruction::BranchFalse { condition: index, target: 0 } });
        },
    }
}

fn emit_scalar_instructions(index: usize, nodes: &[(Node, bool, bool)], code: &mut Vec<Instruction>) {
    fn number(index: usize, nodes: &[(Node, bool, bool)], code: &mut Vec<Instruction>) {
        emit_scalar_instructions(index, nodes, code);
        if !matches!(nodes[index].0, Node::Number(_) | Node::Add(..) | Node::Subtract(..)
            | Node::Multiply(..) | Node::Divide(..) | Node::SquareRoot(_) | Node::Clamp(..)) {
            code.push(Instruction::Number(index));
        }
    }
    fn boolean(index: usize, nodes: &[(Node, bool, bool)], code: &mut Vec<Instruction>) {
        emit_scalar_instructions(index, nodes, code);
        if !matches!(nodes[index].0, Node::Boolean(_) | Node::GreaterThan(..)
            | Node::LessThanOrEqual(..) | Node::Equal(..) | Node::And(..) | Node::Not(_)) {
            code.push(Instruction::Boolean(index));
        }
    }
    let cached = nodes[index].1.then(|| {
        let position = code.len();
        code.push(Instruction::Cached { node: index, end: 0 });
        position
    });
    match nodes[index].0 {
        Node::Add(a, b) | Node::Subtract(a, b) | Node::Multiply(a, b)
        | Node::GreaterThan(a, b) | Node::LessThanOrEqual(a, b) => {
            number(a, nodes, code); number(b, nodes, code);
            code.push(Instruction::Evaluate(index));
        }
        Node::Divide(a, b) => {
            number(b, nodes, code); code.push(Instruction::Nonzero(b));
            number(a, nodes, code); code.push(Instruction::Evaluate(index));
        }
        Node::Equal(a, b) => {
            emit_scalar_instructions(a, nodes, code); emit_scalar_instructions(b, nodes, code);
            code.push(Instruction::Evaluate(index));
        }
        Node::Not(a) => { boolean(a, nodes, code); code.push(Instruction::Evaluate(index)); }
        Node::SquareRoot(a) => { number(a, nodes, code); code.push(Instruction::Evaluate(index)); }
        Node::Clamp(a, b, c) => {
            number(a, nodes, code); number(b, nodes, code); number(c, nodes, code);
            code.push(Instruction::Evaluate(index));
        }
        Node::Conditional(a, b, c) => {
            let mut otherwise = Vec::new();
            emit_scalar_branch(a, false, nodes, code, &mut otherwise);
            emit_scalar_instructions(b, nodes, code); code.push(Instruction::Copy { from: b, to: index });
            let jump = code.len(); code.push(Instruction::Jump(0));
            let target = code.len(); patch_branches(code, &otherwise, target);
            emit_scalar_instructions(c, nodes, code); code.push(Instruction::Copy { from: c, to: index });
            code[jump] = Instruction::Jump(code.len());
        }
        Node::And(a, b) => {
            emit_scalar_instructions(a, nodes, code);
            let branch = code.len(); code.push(Instruction::BranchFalse { condition: a, target: 0 });
            boolean(b, nodes, code); code.push(Instruction::Copy { from: b, to: index });
            let jump = code.len(); code.push(Instruction::Jump(0));
            code[branch] = Instruction::BranchFalse { condition: a, target: code.len() };
            code.push(Instruction::Copy { from: a, to: index });
            code[jump] = Instruction::Jump(code.len());
        }
        Node::Sum { ref inputs, .. } => {
            for input in inputs { emit_scalar_instructions(*input, nodes, code); }
            code.push(Instruction::Evaluate(index));
        }
        _ => code.push(Instruction::Evaluate(index)),
    }
    if let Some(position) = cached { code[position] = Instruction::Cached { node: index, end: code.len() }; }
}

impl ScalarPlan {
    pub fn new(expression: &ExecutableExpressionV1) -> Result<Self, ExecutableErrorV1> {
        let mut nodes = Vec::new();
        let mut interned = BTreeMap::<Vec<u8>, usize>::new();
        fn collect(expression: &ExecutableExpressionV1, nodes: &mut Vec<(Node, bool, bool)>,
            interned: &mut BTreeMap<Vec<u8>, usize>) -> Result<usize, ExecutableErrorV1> {
            use ExecutableExpressionV1 as E;
            let mut key = Vec::new();
            encode_expression(&mut key, expression)?;
            if let Some(index) = interned.get(&key) { return Ok(*index); }
            let mut reusable = true;
            let mut input_only = true;
            let mut child = |expression: &E| -> Result<usize, ExecutableErrorV1> {
                let index = collect(expression, nodes, interned)?;
                reusable &= nodes[index].1;
                input_only &= nodes[index].2;
                Ok(index)
            };
            let node = match expression {
                E::Add(a, b) => Node::Add(child(a)?, child(b)?),
                E::Subtract(a, b) => Node::Subtract(child(a)?, child(b)?),
                E::Multiply(a, b) => Node::Multiply(child(a)?, child(b)?),
                E::Divide(a, b) => Node::Divide(child(a)?, child(b)?),
                E::GreaterThan(a, b) => Node::GreaterThan(child(a)?, child(b)?),
                E::LessThanOrEqual(a, b) => Node::LessThanOrEqual(child(a)?, child(b)?),
                E::Equal(a, b) => Node::Equal(child(a)?, child(b)?),
                E::And(a, b) => Node::And(child(a)?, child(b)?),
                E::Not(a) => Node::Not(child(a)?),
                E::SquareRoot(a) => Node::SquareRoot(child(a)?),
                E::Conditional(a, b, c) => Node::Conditional(child(a)?, child(b)?, child(c)?),
                E::Clamp(a, b, c) => Node::Clamp(child(a)?, child(b)?, child(c)?),
                E::Constant(ExecutableValueV1::Number(bits)) => Node::Number(*bits),
                E::Constant(ExecutableValueV1::Boolean(value)) => Node::Boolean(*value),
                E::Constant(_) => Node::Value(Box::new(expression.clone())),
                E::Slot(index) => Node::Slot(*index),
                E::Argument(index) => Node::Argument(*index),
                E::Binding(index) => { input_only = false; Node::Binding(*index) },
                E::Sum { inputs, predicates, value } => {
                    let inputs = inputs.iter().map(&mut child).collect::<Result<Vec<_>, _>>()?;
                    input_only = false;
                    let mut shape = Vec::new();
                    encode_expression(&mut shape, &E::Sum {
                        inputs: Vec::new(), predicates: predicates.clone(), value: value.clone(),
                    })?;
                    Node::Sum { expression: Box::new(expression.clone()), inputs, shape: shape.into() }
                },
                _ => { reusable = false; input_only = false; Node::Value(Box::new(expression.clone())) },
            };
            let index = nodes.len();
            nodes.push((node, reusable, input_only));
            if reusable { interned.insert(key, index); }
            Ok(index)
        }
        let root = collect(expression, &mut nodes, &mut interned)?;
        let mut uses = vec![0_usize; nodes.len()];
        for (node, _, _) in &nodes {
            match *node {
                Node::Add(a, b) | Node::Subtract(a, b) | Node::Multiply(a, b) | Node::Divide(a, b)
                | Node::GreaterThan(a, b) | Node::LessThanOrEqual(a, b) | Node::Equal(a, b) | Node::And(a, b) => {
                    uses[a] += 1; uses[b] += 1;
                }
                Node::Not(a) | Node::SquareRoot(a) => uses[a] += 1,
                Node::Conditional(a, b, c) | Node::Clamp(a, b, c) => {
                    uses[a] += 1; uses[b] += 1; uses[c] += 1;
                }
                Node::Sum { ref inputs, .. } => { for input in inputs { uses[*input] += 1; } }
                _ => {}
            }
        }
        for (index, (_, reusable, input_only)) in nodes.iter_mut().enumerate() {
            *reusable &= *input_only || uses[index] > 1;
        }
        let mut instructions = Vec::new();
        emit_scalar_instructions(root, &nodes, &mut instructions);
        Ok(Self { expression: Box::new(expression.clone()), nodes, root, instructions, numeric_query: std::sync::OnceLock::new() })
    }

    pub(super) fn compiled_sum(&self, configuration: &[ExecutableSlotV1], arguments: &[ExecutableValueV1],
        matches: &[(relational::Matched, bool)]) -> Option<Result<f64, ExecutableErrorV1>> {
        let query = self.numeric_query.get_or_init(|| compiled_numeric::NumericQuery::compile(self)).as_ref()?;
        let _profile = source_profile_scope_v1(SourceProfilePhaseV1::ScalarEvaluation);
        Some(query.sum(configuration, arguments, matches))
    }

    pub fn memo(&self) -> ScalarMemo<'_> {
        ScalarMemo { plan: self, values: std::cell::RefCell::new(vec![None; self.nodes.len()]),
            other_values: std::cell::RefCell::new(Vec::new()),
            row_values: std::cell::RefCell::new(Vec::new()) }
    }
}

pub(super) struct ScalarMemo<'a> {
    plan: &'a ScalarPlan,
    values: std::cell::RefCell<Vec<Option<ScalarValue>>>,
    other_values: std::cell::RefCell<Vec<ExecutableValueV1>>,
    row_values: std::cell::RefCell<Vec<usize>>,
}

struct ScalarEvaluation<'a> {
    configuration: &'a [ExecutableSlotV1],
    arguments: &'a [ExecutableValueV1],
    context: EvaluationContextV1<'a>,
}

#[derive(Clone, Copy, PartialEq)]
enum ScalarValue {
    Number(u64),
    Boolean(bool),
    Other(usize),
}

impl ScalarValue {
    fn number(value: f64) -> Result<Self, ExecutableErrorV1> {
        if !value.is_finite() { return Err(ExecutableErrorV1::NumericDomain); }
        Ok(Self::Number(canonical_number_bits(value)))
    }

    fn as_number(self) -> Result<f64, ExecutableErrorV1> {
        match self {
            Self::Number(bits) => Ok(f64::from_bits(bits)),
            _ => Err(ExecutableErrorV1::TypeMismatch),
        }
    }

    fn as_boolean(self) -> Result<bool, ExecutableErrorV1> {
        match self {
            Self::Boolean(value) => Ok(value),
            _ => Err(ExecutableErrorV1::TypeMismatch),
        }
    }
}

impl ScalarMemo<'_> {
    fn retain(&self, value: &ExecutableValueV1) -> ScalarValue {
        match value {
            ExecutableValueV1::Number(bits) => ScalarValue::Number(*bits),
            ExecutableValueV1::Boolean(value) => ScalarValue::Boolean(*value),
            value => {
                let mut other = self.other_values.borrow_mut();
                let index = other.len();
                other.push(value.clone());
                ScalarValue::Other(index)
            },
        }
    }

    fn expand(&self, value: ScalarValue) -> ExecutableValueV1 {
        match value {
            ScalarValue::Number(bits) => ExecutableValueV1::Number(bits),
            ScalarValue::Boolean(value) => ExecutableValueV1::Boolean(value),
            ScalarValue::Other(index) => self.other_values.borrow()[index].clone(),
        }
    }

    fn equal(&self, left: ScalarValue, right: ScalarValue) -> bool {
        match (left, right) {
            (ScalarValue::Other(a), ScalarValue::Other(b)) => {
                let other = self.other_values.borrow();
                other[a] == other[b]
            },
            _ => left == right,
        }
    }

    pub fn evaluate(&self, expression: &ExecutableExpressionV1,
        configuration: &[ExecutableSlotV1], arguments: &[ExecutableValueV1], context: EvaluationContextV1)
        -> Result<ExecutableValueV1, ExecutableErrorV1> {
        debug_assert!(std::ptr::eq(expression, self.plan.expression.as_ref()));
        let mut values = self.values.borrow_mut();
        let mut row_values = self.row_values.borrow_mut();
        for index in row_values.drain(..) { values[index] = None; }
        let evaluation = ScalarEvaluation { configuration, arguments, context: EvaluationContextV1 { scalar_memo: None, ..context } };
        let mut pc = 0;
        while pc < self.plan.instructions.len() {
            let stored = match self.plan.instructions[pc] {
                Instruction::Cached { node, end } => {
                    if values[node].is_some() { pc = end; continue; }
                    None
                }
                Instruction::Evaluate(node) => Some((node, self.value(node, &evaluation, &values)?)),
                Instruction::Copy { from, to } => Some((to, values[from].ok_or(ExecutableErrorV1::MalformedProgram)?)),
                Instruction::Number(node) => { values[node].ok_or(ExecutableErrorV1::MalformedProgram)?.as_number()?; None }
                Instruction::Boolean(node) => { values[node].ok_or(ExecutableErrorV1::MalformedProgram)?.as_boolean()?; None }
                Instruction::Nonzero(node) => {
                    if values[node].ok_or(ExecutableErrorV1::MalformedProgram)?.as_number()? == 0.0 {
                        return Err(ExecutableErrorV1::NumericDomain);
                    }
                    None
                }
                Instruction::BranchFalse { condition, target } => {
                    if !values[condition].ok_or(ExecutableErrorV1::MalformedProgram)?.as_boolean()? {
                        pc = target; continue;
                    }
                    None
                }
                Instruction::BranchTrue { condition, target } => {
                    if values[condition].ok_or(ExecutableErrorV1::MalformedProgram)?.as_boolean()? {
                        pc = target; continue;
                    }
                    None
                }
                Instruction::Jump(target) => { pc = target; continue; }
            };
            if let Some((node, value)) = stored {
                values[node] = Some(value);
                if self.plan.nodes[node].1 && !self.plan.nodes[node].2 { row_values.push(node); }
            }
            pc += 1;
        }
        Ok(self.expand(values[self.plan.root].ok_or(ExecutableErrorV1::MalformedProgram)?))
    }

    fn value(&self, index: usize, evaluation: &ScalarEvaluation,
        values: &[Option<ScalarValue>])
        -> Result<ScalarValue, ExecutableErrorV1> {
        macro_rules! eval { ($index:expr) => { values[$index].ok_or(ExecutableErrorV1::MalformedProgram) }; }
        macro_rules! numeric { ($index:expr) => { eval!($index)?.as_number() }; }
        macro_rules! boolean_value { ($index:expr) => { eval!($index)?.as_boolean() }; }
        let value = match self.plan.nodes[index].0 {
            Node::Value(ref expression) => self.retain(&evaluate_uncached(expression, evaluation.configuration, evaluation.arguments, evaluation.context)?),
            Node::Sum { ref expression, ref inputs, ref shape } => {
                let ExecutableExpressionV1::Sum { predicates, value, .. } = expression.as_ref() else {
                    return Err(ExecutableErrorV1::MalformedProgram);
                };
                let _profile = source_profile_scope_v1(SourceProfilePhaseV1::SumEvaluation);
                let inputs = inputs.iter().map(|input| eval!(*input).map(|value| self.expand(value)))
                    .collect::<Result<Vec<_>, _>>()?;
                self.retain(&relational::sum_with_values(inputs, predicates, value,
                    evaluation.configuration, evaluation.context, Some(shape))?)
            },
            Node::Number(bits) => ScalarValue::Number(bits),
            Node::Boolean(value) => ScalarValue::Boolean(value),
            Node::Slot(slot) => self.retain(evaluation.configuration.get(usize::from(slot))
                .ok_or(ExecutableErrorV1::UnknownSlot(slot))?.value().ok_or(ExecutableErrorV1::MissingState)?),
            Node::Argument(argument) => self.retain(evaluation.arguments.get(usize::from(argument))
                .ok_or(ExecutableErrorV1::UnknownArgument(argument))?),
            Node::Binding(binding) => self.retain(evaluation.context.bindings.and_then(|bindings| bindings.get(&binding))
                .ok_or(ExecutableErrorV1::MalformedProgram)?),
            Node::Add(a, b) => ScalarValue::number(numeric!(a)? + numeric!(b)?)?,
            Node::Subtract(a, b) => ScalarValue::number(numeric!(a)? - numeric!(b)?)?,
            Node::Multiply(a, b) => ScalarValue::number(numeric!(a)? * numeric!(b)?)?,
            Node::Divide(a, b) => {
                let denominator = numeric!(b)?;
                if denominator == 0.0 { return Err(ExecutableErrorV1::NumericDomain); }
                ScalarValue::number(numeric!(a)? / denominator)?
            },
            Node::GreaterThan(a, b) => ScalarValue::Boolean(numeric!(a)? > numeric!(b)?),
            Node::LessThanOrEqual(a, b) => ScalarValue::Boolean(numeric!(a)? <= numeric!(b)?),
            Node::Equal(a, b) => ScalarValue::Boolean(self.equal(eval!(a)?, eval!(b)?)),
            Node::And(a, b) => ScalarValue::Boolean(boolean_value!(a)? && boolean_value!(b)?),
            Node::Not(a) => ScalarValue::Boolean(!boolean_value!(a)?),
            Node::SquareRoot(a) => {
                let value = numeric!(a)?;
                if value < 0.0 { return Err(ExecutableErrorV1::NumericDomain); }
                ScalarValue::number(value.sqrt())?
            },
            Node::Conditional(a, b, c) => eval!(if boolean_value!(a)? { b } else { c })?,
            Node::Clamp(a, b, c) => {
                let value = numeric!(a)?;
                let lower = numeric!(b)?;
                let upper = numeric!(c)?;
                if lower > upper { return Err(ExecutableErrorV1::NumericDomain); }
                ScalarValue::number(value.clamp(lower, upper))?
            },
        };
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ExecutableExpressionV1 as E;

    #[test]
    #[ignore = "owning-loop timing with an explicitly supplied consumer fixture"]
    fn measure_consumer_sum_inputs() {
        use clause_package::*;
        let source = std::fs::read(std::env::var("CLAUSE_SUM_INPUT_SOURCE").unwrap()).unwrap();
        let frontend = CanonicalDeclaredFrontendV1::read(DECLARED_FOCUSED_FRONTEND_SOURCE_V1).unwrap();
        let cst = read_canonical_source_with_declared_frontend_v1(&source, &frontend).unwrap();
        let allocation = plan_independent_canonical_source_allocations_v1(&cst,
            ProgramChangeOccurrenceId::from_bytes([17; 32])).unwrap();
        let scope = TermScope { universe: UniverseId::from_bytes([1;32]), semantics: ClauseSemanticsId::from_bytes([2;32]) };
        let compiled = elaborate_canonical_source_package_v1(&cst,
            CanonicalSourceContextV1 { universe: scope.universe, semantics: scope.semantics }, &allocation).unwrap();
        let roles = (0..compiled.state_cells.len()).map(|id| LocalRoleRefV2 {
            schema: RelationSchemaLocalId::new(2), role: RoleLocalId::new(id as u32),
        }).collect::<Vec<_>>();
        let lowered = lower_canonical_executable_program_v1(scope, &compiled.state_cells, &compiled.executable_handlers, &roles).unwrap();
        let slots = lowered.program.initial_configuration.iter().cloned().map(Into::into).collect::<Vec<ExecutableSlotV1>>();
        let configuration = slots.as_slice();
        let arguments = [ExecutableValueV1::number(0.016).unwrap()];
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES], step_ordinal: 0,
            reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        let mut cases = Vec::new();
        let mut seen = BTreeSet::new();
        for rule in &lowered.program.rules {
            let joins = rule.predicates.iter().filter(|p| matches!(p, E::RelationMatch(..))).cloned().collect::<Vec<_>>();
            let Ok(matches) = relational::match_rule(&joins, configuration, &arguments, context, &mut 0, false) else { continue; };
            for (_, expression) in &rule.assignments {
                let expressions: Vec<_> = match expression {
                    E::RelationEffects(effects) => effects.iter().map(|effect| effect.parts().2).collect(),
                    _ => vec![expression],
                };
                for expression in expressions {
                    let plan = ScalarPlan::new(expression).unwrap();
                    for (node, _, _) in &plan.nodes {
                        let Node::Sum { expression, .. } = node else { continue; };
                        let E::Sum { inputs, .. } = expression.as_ref() else { unreachable!() };
                        // Isolate the measured invocation-input loop, retaining the
                        // consumer's exact lowered arithmetic and substitutions.
                        let expression = E::Sum { inputs: inputs.clone(), predicates: vec![],
                            value: Box::new(E::Constant(ExecutableValueV1::number(0.0).unwrap())) };
                        let mut key = Vec::new(); encode_expression(&mut key, &expression).unwrap();
                        if !seen.insert(key) { continue; }
                        let rows = matches.iter().filter(|(matched, accepted)| *accepted &&
                            inputs.iter().all(|input| evaluate(input, configuration, &arguments,
                                EvaluationContextV1 { bindings: Some(&matched.bindings), ..context }).is_ok()))
                            .map(|(matched, _)| matched.bindings.clone()).collect::<Vec<_>>();
                        if !rows.is_empty() { cases.push((ScalarPlan::new(&expression).unwrap(), rows)); }
                    }
                }
            }
        }
        assert!(!cases.is_empty());
        let mut samples = Vec::new();
        for _ in 0..7 {
            let started = std::time::Instant::now();
            for _ in 0..100 {
                for (plan, rows) in &cases {
                    let memo = plan.memo();
                    for bindings in rows {
                        std::hint::black_box(evaluate(&plan.expression, configuration, &arguments,
                            EvaluationContextV1 { bindings: Some(bindings), scalar_memo: Some(&memo), ..context }).unwrap());
                    }
                }
            }
            samples.push(started.elapsed().as_secs_f64() * 1000.0 / 100.0);
        }
        eprintln!("sum-input-loop cases={} rows={} ms_per_sweep={samples:?}", cases.len(), cases.iter().map(|(_,rows)|rows.len()).sum::<usize>());
    }

    #[test]
    fn compiled_sum_inputs_preserve_binding_changes_and_selected_errors() {
        let number = |value| ExecutableValueV1::number(value).unwrap();
        let input = E::Divide(Box::new(E::Binding(0)), Box::new(E::Argument(1)));
        let sum = E::Sum { inputs: vec![input.clone(), input.clone()], predicates: vec![],
            value: Box::new(E::Add(Box::new(E::Argument(0)), Box::new(E::Argument(1)))) };
        let expression = E::Conditional(Box::new(E::Argument(0)),
            Box::new(E::Add(Box::new(sum), Box::new(input))), Box::new(E::Constant(number(7.0))));
        let plan = ScalarPlan::new(&expression).unwrap();
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES], step_ordinal: 0,
            reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        for arguments in [vec![ExecutableValueV1::Boolean(true), number(2.0)],
            vec![ExecutableValueV1::Boolean(true), number(0.0)],
            vec![ExecutableValueV1::Boolean(true)], vec![ExecutableValueV1::Boolean(false)]] {
            let memo = plan.memo();
            for value in [number(2.0), number(9.0), ExecutableValueV1::Boolean(false)] {
                let bindings = relational::Bindings::from([(0, value)]);
                let context = EvaluationContextV1 { bindings: Some(&bindings), ..context };
                assert_eq!(evaluate(&plan.expression, &[], &arguments,
                    EvaluationContextV1 { scalar_memo: Some(&memo), ..context }),
                    evaluate(&expression, &[], &arguments, context));
            }
        }
    }

    #[test]
    fn compiled_boolean_branches_preserve_selected_types_and_errors() {
        let boolean = |value| E::Constant(ExecutableValueV1::Boolean(value));
        let number = |value| E::Constant(ExecutableValueV1::number(value).unwrap());
        let invalid = E::Divide(Box::new(E::Argument(9)), Box::new(number(0.0)));
        let nested = E::Conditional(Box::new(E::Argument(0)),
            Box::new(boolean(true)), Box::new(boolean(false)));
        let predicates = [
            nested.clone(),
            E::Equal(Box::new(nested.clone()), Box::new(boolean(false))),
            E::Equal(Box::new(boolean(false)), Box::new(nested.clone())),
            E::Not(Box::new(nested)),
            E::Conditional(Box::new(E::Argument(0)), Box::new(boolean(false)), Box::new(invalid.clone())),
            E::Equal(Box::new(boolean(false)), Box::new(E::Conditional(Box::new(E::Argument(0)),
                Box::new(number(0.0)), Box::new(boolean(false))))),
            E::Conditional(Box::new(E::LessThanOrEqual(Box::new(number(0.0)), Box::new(number(1.0)))),
                Box::new(E::Argument(0)), Box::new(invalid.clone())),
        ];
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES], step_ordinal: 0,
            reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        for predicate in predicates {
            let expression = E::Conditional(Box::new(predicate), Box::new(number(7.0)), Box::new(invalid.clone()));
            let plan = ScalarPlan::new(&expression).unwrap();
            for argument in [ExecutableValueV1::Boolean(false), ExecutableValueV1::Boolean(true),
                ExecutableValueV1::number(2.0).unwrap()] {
                let memo = plan.memo();
                let arguments = [argument];
                assert_eq!(memo.evaluate(&plan.expression, &[], &arguments, context),
                    evaluate_uncached(&expression, &[], &arguments, context));
            }
        }
    }

    #[test]
    fn scalar_instructions_preserve_operand_errors_and_lazy_branches() {
        let n = |value| E::Constant(ExecutableValueV1::number(value).unwrap());
        let boolean = E::Constant(ExecutableValueV1::Boolean(false));
        let expressions = [
            E::Add(Box::new(boolean.clone()), Box::new(E::Argument(9))),
            E::Divide(Box::new(E::Argument(9)), Box::new(boolean.clone())),
            E::Divide(Box::new(E::Argument(9)), Box::new(n(0.0))),
            E::Clamp(Box::new(boolean.clone()), Box::new(E::Argument(9)), Box::new(n(3.0))),
            E::Conditional(Box::new(boolean.clone()), Box::new(E::Slot(9)), Box::new(n(3.0))),
            E::And(Box::new(boolean.clone()), Box::new(E::Slot(9))),
            E::And(Box::new(E::Constant(ExecutableValueV1::Boolean(true))), Box::new(n(3.0))),
            E::SquareRoot(Box::new(n(-1.0))),
            E::Clamp(Box::new(n(2.0)), Box::new(n(3.0)), Box::new(n(1.0))),
        ];
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES], step_ordinal: 0,
            reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        for expression in expressions {
            let plan = ScalarPlan::new(&expression).unwrap();
            let memo = plan.memo();
            assert_eq!(evaluate(&plan.expression, &[], &[], EvaluationContextV1 { scalar_memo: Some(&memo), ..context }),
                evaluate(&expression, &[], &[], context));
        }
    }

    #[test]
    fn retained_plans_use_fresh_values_across_preparations() {
        let number = |value| ExecutableValueV1::number(value).unwrap();
        let plans = Arc::new(relational::ScalarPlans::default());
        let expression = E::Add(Box::new(E::Slot(0)), Box::new(E::Argument(0)));
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES], step_ordinal: 0,
            reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        let mut retained = None;
        for (state, input) in [(2.0, 3.0), (7.0, -4.0), (2.0, 9.0)] {
            let mut queries = relational::SumQueries::default();
            queries.scalar_plans = plans.clone();
            let queries = std::cell::RefCell::new(queries);
            let shared = EvaluationContextV1 { sum_queries: Some(&queries), ..context };
            let plan = relational::scalar_plan(&expression, shared).unwrap().unwrap();
            if let Some(previous) = &retained { assert!(Arc::ptr_eq(previous, &plan)); }
            let memo = plan.memo();
            let configuration = [number(state).into()];
            let arguments = [number(input)];
            let actual = evaluate(&plan.expression, &configuration, &arguments,
                EvaluationContextV1 { scalar_memo: Some(&memo), ..shared }).unwrap();
            assert_eq!(actual, evaluate_with_reads(&expression, &configuration, &arguments, context).unwrap().value);
            assert_eq!(actual, number(state + input));
            drop(memo);
            retained = Some(plan);
        }
    }

    #[test]
    fn sum_contributions_reuse_structure_with_fresh_values_for_each_match_and_query() {
        let number = |value| ExecutableValueV1::number(value).unwrap();
        let configuration = [ExecutableValueV1::RelationTable(ExecutableRelationTableV1 {
            subject_domain: 7, value_kind: ExecutableRelationValueKindV1::Number,
            value_domain: None, cardinality: ExecutableRelationCardinalityV1::One, total: false,
            rows: Arc::new([1.0, 3.0].into_iter().enumerate().map(|(id, value)|
                (ExecutableReferentV1::declared(7, id as u32), [number(value)].into()))
                .collect::<BTreeMap<_, _>>().into()),
        }).into()];
        let repeated = E::Multiply(Box::new(E::Binding(1)), Box::new(E::Argument(0)));
        let query = E::Sum { inputs: vec![E::Argument(0)],
            predicates: vec![E::RelationMatch(0, Box::new(E::Binding(0)), Box::new(E::Binding(1)))],
            value: Box::new(E::Add(Box::new(repeated.clone()), Box::new(repeated))) };
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES], step_ordinal: 0,
            reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        let queries = std::cell::RefCell::new(relational::SumQueries::default());
        let shared = EvaluationContextV1 { sum_queries: Some(&queries), ..context };
        let effect = E::Add(Box::new(query.clone()), Box::new(query.clone()));
        let first = ScalarPlan::new(&effect).unwrap();
        let first_memo = first.memo();
        assert_eq!(evaluate(&first.expression, &configuration, &[number(2.0)],
            EvaluationContextV1 { scalar_memo: Some(&first_memo), ..shared }).unwrap(), number(32.0));
        let E::Sum { inputs, predicates, .. } = &query else { unreachable!() };
        let distinct = ScalarPlan::new(&E::Sum { inputs: inputs.clone(), predicates: predicates.clone(),
            value: Box::new(E::Constant(number(3.0))) }).unwrap();
        let distinct_memo = distinct.memo();
        assert_eq!(evaluate(&distinct.expression, &configuration, &[number(2.0)],
            EvaluationContextV1 { scalar_memo: Some(&distinct_memo), ..shared }).unwrap(), number(6.0));
        for input in [2.0, 5.0, 2.0] {
            let expected = evaluate_with_reads(&query, &configuration, &[number(input)], context).unwrap();
            assert_eq!(evaluate(&query, &configuration, &[number(input)], shared).unwrap(), number(input * 8.0));
            let actual = evaluate_with_reads(&query, &configuration, &[number(input)], shared).unwrap();
            assert_eq!(actual.value, expected.value);
            assert_eq!(actual.reads, expected.reads);
            let expected = evaluate_with_reads(&effect, &configuration, &[number(input)], context).unwrap();
            let plan = ScalarPlan::new(&effect).unwrap();
            let memo = plan.memo();
            assert_eq!(evaluate_for_trace(&plan.expression, &configuration, &[number(input)],
                EvaluationContextV1 { scalar_memo: Some(&memo), ..shared }, false).unwrap().value, expected.value);
            let actual = evaluate_for_trace(&effect, &configuration, &[number(input)], shared, true).unwrap();
            assert_eq!(actual.value, expected.value);
            assert_eq!(actual.reads, expected.reads);
        }
    }

    #[test]
    fn lazy_scalar_reuse_preserves_changed_bindings_branch_errors_and_reads() {
        let number = |value| E::Constant(ExecutableValueV1::number(value).unwrap());
        let repeated = E::Multiply(Box::new(E::Binding(0)), Box::new(E::Argument(0)));
        let invalid = E::Divide(Box::new(E::Argument(9)), Box::new(number(0.0)));
        let expression = E::Conditional(Box::new(E::Argument(1)),
            Box::new(E::Add(Box::new(repeated.clone()), Box::new(repeated))), Box::new(invalid.clone()));
        let plan = ScalarPlan::new(&expression).unwrap();
        assert!(!plan.nodes.is_empty());
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES], step_ordinal: 0,
            reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        let memo = plan.memo();
        for value in [3.0, 7.0] {
            let bindings = relational::Bindings::from([(0, ExecutableValueV1::number(value).unwrap())]);
            let arguments = [ExecutableValueV1::number(2.0).unwrap(), ExecutableValueV1::Boolean(true)];
            let context = EvaluationContextV1 { bindings: Some(&bindings), ..context };
            let reused = EvaluationContextV1 { scalar_memo: Some(&memo), ..context };
            let expected = evaluate(&expression, &[], &arguments, context).unwrap();
            assert_eq!(evaluate(&plan.expression, &[], &arguments, reused).unwrap(), expected);
            assert_eq!(expected, ExecutableValueV1::number(value * 4.0).unwrap());
            assert_eq!(memo.values.borrow().iter().filter(|value| value.is_some()).count(), 6);
            let expected_reads = evaluate_with_reads(&expression, &[], &arguments, context).unwrap();
            let actual_reads = evaluate_with_reads(&plan.expression, &[], &arguments, reused).unwrap();
            assert_eq!(expected_reads.value, actual_reads.value);
            assert_eq!(expected_reads.reads, actual_reads.reads);
        }
        let memo = plan.memo();
        let arguments = [ExecutableValueV1::number(2.0).unwrap(), ExecutableValueV1::Boolean(false)];
        assert!(matches!(evaluate(&plan.expression, &[], &arguments,
            EvaluationContextV1 { scalar_memo: Some(&memo), ..context }), Err(ExecutableErrorV1::NumericDomain)));
        let short = E::And(Box::new(E::Constant(ExecutableValueV1::Boolean(false))), Box::new(invalid));
        let plan = ScalarPlan::new(&short).unwrap();
        let memo = plan.memo();
        assert_eq!(evaluate(&plan.expression, &[], &[],
            EvaluationContextV1 { scalar_memo: Some(&memo), ..context }).unwrap(), ExecutableValueV1::Boolean(false));
    }
}
