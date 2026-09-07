//! Lazy reuse of equal scalar subexpressions within one fixed substitution.
use super::*;

pub(super) struct ScalarPlan {
    pub expression: Box<ExecutableExpressionV1>,
    nodes: Vec<(Node, bool)>,
    root: usize,
}

enum Node {
    Value(ExecutableExpressionV1),
    Add(usize, usize), Subtract(usize, usize), Multiply(usize, usize), Divide(usize, usize),
    GreaterThan(usize, usize), LessThanOrEqual(usize, usize), Equal(usize, usize), And(usize, usize),
    Not(usize), SquareRoot(usize), Conditional(usize, usize, usize), Clamp(usize, usize, usize),
}

impl ScalarPlan {
    pub fn new(expression: &ExecutableExpressionV1) -> Result<Self, ExecutableErrorV1> {
        let mut nodes = Vec::new();
        let mut interned = BTreeMap::<Vec<u8>, usize>::new();
        fn collect(expression: &ExecutableExpressionV1, nodes: &mut Vec<(Node, bool)>,
            interned: &mut BTreeMap<Vec<u8>, usize>) -> Result<usize, ExecutableErrorV1> {
            use ExecutableExpressionV1 as E;
            let mut key = Vec::new();
            encode_expression(&mut key, expression)?;
            if let Some(index) = interned.get(&key) { return Ok(*index); }
            let mut reusable = true;
            let mut child = |expression: &E| -> Result<usize, ExecutableErrorV1> {
                let index = collect(expression, nodes, interned)?;
                reusable &= nodes[index].1;
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
                E::Constant(_) | E::Slot(_) | E::Argument(_) | E::Binding(_) => Node::Value(expression.clone()),
                _ => { reusable = false; Node::Value(expression.clone()) },
            };
            let index = nodes.len();
            nodes.push((node, reusable));
            if reusable { interned.insert(key, index); }
            Ok(index)
        }
        let root = collect(expression, &mut nodes, &mut interned)?;
        Ok(Self { expression: Box::new(expression.clone()), nodes, root })
    }

    pub fn memo(&self) -> ScalarMemo<'_> {
        ScalarMemo { plan: self, values: std::cell::RefCell::new(vec![None; self.nodes.len()]),
            touched: std::cell::RefCell::new(Vec::new()) }
    }
}

pub(super) struct ScalarMemo<'a> {
    plan: &'a ScalarPlan,
    values: std::cell::RefCell<Vec<Option<ExecutableValueV1>>>,
    touched: std::cell::RefCell<Vec<usize>>,
}

impl ScalarMemo<'_> {
    pub fn evaluate(&self, expression: &ExecutableExpressionV1,
        configuration: &[ExecutableSlotV1], arguments: &[ExecutableValueV1], context: EvaluationContextV1)
        -> Result<ExecutableValueV1, ExecutableErrorV1> {
        debug_assert!(std::ptr::eq(expression, self.plan.expression.as_ref()));
        {
            let mut values = self.values.borrow_mut();
            for index in self.touched.borrow_mut().drain(..) { values[index] = None; }
        }
        self.node(self.plan.root, configuration, arguments, EvaluationContextV1 { scalar_memo: None, ..context })
    }

    fn node(&self, index: usize, configuration: &[ExecutableSlotV1], arguments: &[ExecutableValueV1], context: EvaluationContextV1)
        -> Result<ExecutableValueV1, ExecutableErrorV1> {
        if let Some(value) = &self.values.borrow()[index] { return Ok(value.clone()); }
        let eval = |index| self.node(index, configuration, arguments, context);
        let numeric = |index| number(eval(index)?);
        let boolean_value = |index| boolean(eval(index)?);
        let value = match self.plan.nodes[index].0 {
            Node::Value(ref expression) => evaluate_uncached(expression, configuration, arguments, context)?,
            Node::Add(a, b) => ExecutableValueV1::number(numeric(a)? + numeric(b)?)?,
            Node::Subtract(a, b) => ExecutableValueV1::number(numeric(a)? - numeric(b)?)?,
            Node::Multiply(a, b) => ExecutableValueV1::number(numeric(a)? * numeric(b)?)?,
            Node::Divide(a, b) => {
                let denominator = numeric(b)?;
                if denominator == 0.0 { return Err(ExecutableErrorV1::NumericDomain); }
                ExecutableValueV1::number(numeric(a)? / denominator)?
            },
            Node::GreaterThan(a, b) => ExecutableValueV1::Boolean(numeric(a)? > numeric(b)?),
            Node::LessThanOrEqual(a, b) => ExecutableValueV1::Boolean(numeric(a)? <= numeric(b)?),
            Node::Equal(a, b) => ExecutableValueV1::Boolean(eval(a)? == eval(b)?),
            Node::And(a, b) => ExecutableValueV1::Boolean(boolean_value(a)? && boolean_value(b)?),
            Node::Not(a) => ExecutableValueV1::Boolean(!boolean_value(a)?),
            Node::SquareRoot(a) => {
                let value = numeric(a)?;
                if value < 0.0 { return Err(ExecutableErrorV1::NumericDomain); }
                ExecutableValueV1::number(value.sqrt())?
            },
            Node::Conditional(a, b, c) => eval(if boolean_value(a)? { b } else { c })?,
            Node::Clamp(a, b, c) => {
                let value = numeric(a)?;
                let lower = numeric(b)?;
                let upper = numeric(c)?;
                if lower > upper { return Err(ExecutableErrorV1::NumericDomain); }
                ExecutableValueV1::number(value.clamp(lower, upper))?
            },
        };
        if self.plan.nodes[index].1 {
            self.values.borrow_mut()[index] = Some(value.clone());
            self.touched.borrow_mut().push(index);
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ExecutableExpressionV1 as E;

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
        for input in [2.0, 5.0, 2.0] {
            let expected = evaluate_with_reads(&query, &configuration, &[number(input)], context).unwrap();
            assert_eq!(evaluate(&query, &configuration, &[number(input)], shared).unwrap(), number(input * 8.0));
            let actual = evaluate_with_reads(&query, &configuration, &[number(input)], shared).unwrap();
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
        for value in [3.0, 7.0] {
            let bindings = BTreeMap::from([(0, ExecutableValueV1::number(value).unwrap())]);
            let arguments = [ExecutableValueV1::number(2.0).unwrap(), ExecutableValueV1::Boolean(true)];
            let context = EvaluationContextV1 { bindings: Some(&bindings), ..context };
            let memo = plan.memo();
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
