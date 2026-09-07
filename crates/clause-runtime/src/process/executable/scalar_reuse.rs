//! Lazy reuse of equal pure subexpressions within one fixed substitution.
use super::*;
use std::collections::HashMap;

pub(super) struct ScalarPlan {
    pub expression: Box<ExecutableExpressionV1>,
    slots: HashMap<usize, usize>,
    count: usize,
}

impl ScalarPlan {
    pub fn new(expression: &ExecutableExpressionV1) -> Result<Self, ExecutableErrorV1> {
        let expression = Box::new(expression.clone());
        let mut slots = HashMap::new();
        let mut interned = BTreeMap::<Vec<u8>, usize>::new();
        let mut counts = Vec::<usize>::new();
        fn collect(expression: &ExecutableExpressionV1, slots: &mut HashMap<usize, usize>,
            interned: &mut BTreeMap<Vec<u8>, usize>, counts: &mut Vec<usize>) -> Result<bool, ExecutableErrorV1> {
            use ExecutableExpressionV1 as E;
            let children: Vec<&E> = match expression {
                E::Constant(_) | E::Slot(_) | E::Argument(_) | E::Binding(_) => return Ok(true),
                // These introduce another binding, identity or evaluation scope.
                E::Sum { .. } | E::FreshReferent { .. } | E::RelationMatch(..)
                | E::RelationEffects(_) | E::DerivedRelation(_) | E::Accumulate(_) => return Ok(false),
                E::Not(a) | E::SquareRoot(a) | E::TextTransform(_, a) | E::ReferentFacet { value: a, .. } => vec![a],
                E::RelationRead(a, b) | E::RelationPresent(a, b) | E::RelationRemoveRow(a, b)
                | E::Concatenate(a, b) | E::StartsWith(a, b) | E::ContainsText(a, b)
                | E::Add(a, b) | E::Subtract(a, b) | E::Multiply(a, b) | E::Divide(a, b)
                | E::GreaterThan(a, b) | E::LessThanOrEqual(a, b) | E::Equal(a, b) | E::And(a, b)
                | E::SetInsert(a, b) | E::SetContains(a, b) | E::SetRemove(a, b) => vec![a, b],
                E::RelationPut(a, b, c) | E::RelationInsert(a, b, c) | E::RelationRemoveValue(a, b, c)
                | E::Conditional(a, b, c) | E::Clamp(a, b, c) => vec![a, b, c],
            };
            let mut pure = true;
            for child in children { pure &= collect(child, slots, interned, counts)?; }
            if pure {
                let mut key = Vec::new();
                encode_expression(&mut key, expression)?;
                let index = *interned.entry(key).or_insert_with(|| {
                    counts.push(0);
                    counts.len() - 1
                });
                counts[index] += 1;
                slots.insert(std::ptr::from_ref(expression) as usize, index);
            }
            Ok(pure)
        }
        collect(&expression, &mut slots, &mut interned, &mut counts)?;
        slots.retain(|_, index| counts[*index] > 1);
        // Addresses only locate nodes inside this owned, unmoved tree. Exact
        // expression encoding, not addresses, establishes reusable equality.
        Ok(Self { expression, slots, count: counts.len() })
    }

    pub fn memo(&self) -> ScalarMemo<'_> {
        ScalarMemo { plan: self, values: std::cell::RefCell::new(vec![None; self.count]) }
    }
}

pub(super) struct ScalarMemo<'a> {
    plan: &'a ScalarPlan,
    values: std::cell::RefCell<Vec<Option<ExecutableValueV1>>>,
}

impl ScalarMemo<'_> {
    pub fn evaluate(&self, expression: &ExecutableExpressionV1,
        configuration: &[ExecutableSlotV1], arguments: &[ExecutableValueV1], context: EvaluationContextV1)
        -> Result<ExecutableValueV1, ExecutableErrorV1> {
        let Some(index) = self.plan.slots.get(&(std::ptr::from_ref(expression) as usize)).copied() else {
            return evaluate_uncached(expression, configuration, arguments, context);
        };
        if let Some(value) = &self.values.borrow()[index] { return Ok(value.clone()); }
        let value = evaluate_uncached(expression, configuration, arguments, context)?;
        self.values.borrow_mut()[index] = Some(value.clone());
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ExecutableExpressionV1 as E;

    #[test]
    fn lazy_scalar_reuse_preserves_changed_bindings_branch_errors_and_reads() {
        let number = |value| E::Constant(ExecutableValueV1::number(value).unwrap());
        let repeated = E::Multiply(Box::new(E::Binding(0)), Box::new(E::Argument(0)));
        let invalid = E::Divide(Box::new(E::Argument(9)), Box::new(number(0.0)));
        let expression = E::Conditional(Box::new(E::Argument(1)),
            Box::new(E::Add(Box::new(repeated.clone()), Box::new(repeated))), Box::new(invalid.clone()));
        let plan = ScalarPlan::new(&expression).unwrap();
        assert!(!plan.slots.is_empty());
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
            assert_eq!(memo.values.borrow().iter().filter(|value| value.is_some()).count(), 1);
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
