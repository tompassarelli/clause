//! Exact reuse of allocation-independent effects between closed configurations.
use super::*;

#[derive(Default)]
pub(super) struct EvaluationCache {
    program: Option<Arc<ExecutableProgramV1>>,
    entries: BTreeMap<u16, Entry>,
}

struct Entry {
    reads: Vec<u16>,
    writes: Vec<u16>,
    reusable: bool,
    saved: Option<SavedEffects>,
}

struct SavedEffects {
    arguments: Vec<ExecutableValueV1>,
    before: Vec<ExecutableSlotV1>,
    after: Vec<ExecutableSlotV1>,
    rule_applied: bool,
}

impl EvaluationCache {
    pub(super) fn evaluate(
        &mut self,
        program: &Arc<ExecutableProgramV1>,
        entry: u16,
        configuration: &[ExecutableSlotV1],
        arguments: &[ExecutableValueV1],
        compute: impl FnOnce() -> Result<(Vec<ExecutableSlotV1>, bool), ExecutableErrorV1>,
    ) -> Result<(Vec<ExecutableSlotV1>, bool), ExecutableErrorV1> {
        if self.program.as_ref().is_none_or(|prior| !Arc::ptr_eq(prior, program)) {
            self.entries.clear();
            self.program = Some(program.clone());
        }
        let entry = self.entries.entry(entry).or_insert_with(|| Entry::new(program, entry));
        if !entry.reusable { return compute(); }
        if let Some(saved) = &entry.saved
            && saved.arguments == arguments
            && entry.reads.iter().zip(&saved.before)
                .all(|(slot, before)| configuration.get(usize::from(*slot)) == Some(before))
        {
            let mut next = configuration.to_vec();
            for (slot, value) in entry.writes.iter().zip(&saved.after) {
                next[usize::from(*slot)] = value.clone();
            }
            return Ok((next, saved.rule_applied));
        }
        let (next, rule_applied) = compute()?;
        // Keep one exact input/output per entry. Values retain immutable rows;
        // replacing a program or an entry result releases its previous inputs.
        entry.saved = Some(SavedEffects {
            arguments: arguments.to_vec(),
            before: entry.reads.iter().map(|slot| configuration[usize::from(*slot)].clone()).collect(),
            after: entry.writes.iter().map(|slot| next[usize::from(*slot)].clone()).collect(),
            rule_applied,
        });
        Ok((next, rule_applied))
    }
}

impl Entry {
    fn new(program: &ExecutableProgramV1, entry: u16) -> Self {
        let mut reads = BTreeSet::new();
        let mut writes = BTreeSet::new();
        let mut reusable = true;
        for rule in program.rules.iter().filter(|rule| rule.entry == entry && !closure::is_derivation(rule)) {
            reads.extend(rule.required_present.iter().chain(&rule.required_absent).copied());
            for predicate in &rule.predicates {
                reusable &= dependencies(predicate, &mut reads);
            }
            for (slot, expression) in &rule.assignments {
                writes.insert(*slot);
                reusable &= dependencies(expression, &mut reads);
            }
            writes.extend(&rule.removals);
        }
        // Row effects and accumulation consume their destination's pre-state.
        // Unselected assignments must also preserve that pre-state exactly.
        reads.extend(&writes);
        Self { reads: reads.into_iter().collect(), writes: writes.into_iter().collect(), reusable, saved: None }
    }
}

fn dependencies(expression: &ExecutableExpressionV1, reads: &mut BTreeSet<u16>) -> bool {
    use ExecutableExpressionV1 as E;
    match expression {
        E::Let { value, body, .. } => dependencies(value, reads) & dependencies(body, reads),
        E::Constant(_) | E::Binding(_) | E::Argument(_) => true,
        E::FreshReferent { .. } => false,
        E::Slot(slot) => { reads.insert(*slot); true }
        E::RelationMatch(slot, subject, value) => {
            reads.insert(*slot);
            dependencies(subject, reads) & dependencies(value, reads)
        }
        E::Sum { inputs, predicates, value } => inputs.iter().chain(predicates).chain(std::iter::once(value.as_ref()))
            .fold(true, |pure, expression| dependencies(expression, reads) & pure),
        E::RelationEffects(effects) | E::DerivedRelation(effects) => effects.iter().fold(true, |pure, effect| {
            let (_, subject, value) = effect.parts();
            dependencies(subject, reads) & dependencies(value, reads) & pure
        }),
        E::Not(value) | E::Accumulate(value) | E::SquareRoot(value) | E::TextTransform(_, value)
        | E::ReferentFacet { value, .. } => dependencies(value, reads),
        E::RelationRead(a, b) | E::RelationPresent(a, b) | E::RelationRemoveRow(a, b)
        | E::Concatenate(a, b) | E::StartsWith(a, b) | E::ContainsText(a, b)
        | E::Add(a, b) | E::Subtract(a, b) | E::Multiply(a, b) | E::Divide(a, b)
        | E::GreaterThan(a, b) | E::LessThanOrEqual(a, b) | E::Equal(a, b) | E::And(a, b)
        | E::SetInsert(a, b) | E::SetContains(a, b) | E::SetRemove(a, b) =>
            dependencies(a, reads) & dependencies(b, reads),
        E::RelationPut(a, b, c) | E::RelationInsert(a, b, c) | E::RelationRemoveValue(a, b, c)
        | E::Conditional(a, b, c) | E::Clamp(a, b, c) =>
            dependencies(a, reads) & dependencies(b, reads) & dependencies(c, reads),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ExecutableExpressionV1 as E;
    use ExecutableValueV1 as V;

    fn number(value: f64) -> V { V::number(value).unwrap() }
    fn table(values: &[(u32, f64)]) -> V {
        V::RelationTable(ExecutableRelationTableV1 {
            subject_domain: 7, value_kind: ExecutableRelationValueKindV1::Number,
            value_domain: None, cardinality: ExecutableRelationCardinalityV1::Many, total: false,
            rows: Arc::new(values.iter().map(|(id, value)|
                (ExecutableReferentV1::declared(7, *id), [number(*value)].into())).collect::<BTreeMap<_, _>>().into()),
        })
    }
    fn evaluator<'a>(program: &'a Arc<ExecutableProgramV1>, cache: Option<&'a Mutex<EvaluationCache>>) -> StepEvaluator<'a> {
        StepEvaluator { program, cache, allocation_root: [11; IDENTITY_BYTES],
            configuration_id: ConfigurationId::from_bytes([13; IDENTITY_BYTES]) }
    }
    fn rule(entry: u16, assignments: Vec<(u16, E)>) -> ExecutableRuleV1 {
        ExecutableRuleV1 { entry, predicates: vec![], required_present: vec![],
            required_absent: vec![], assignments, removals: vec![] }
    }
    fn run(program: &Arc<ExecutableProgramV1>, cache: Option<&Mutex<EvaluationCache>>,
        configuration: &[ExecutableSlotV1], arguments: Vec<V>, ordinal: u64)
        -> Result<(Vec<ExecutableSlotV1>, ExecutableStepV1), ExecutableErrorV1> {
        evaluator(program, cache).prepare_step_traced(ExecutableOccurrenceV1 { entry: 2, arguments },
            ordinal, ordinal, configuration, None)
    }

    #[test]
    fn exact_dependencies_reuse_effects_and_preserve_changed_inputs() {
        let mut matched = rule(2, vec![(1, E::RelationEffects(vec![ExecutableRelationEffectV1::Insert(
            E::Binding(0), E::Add(Box::new(E::Binding(1)), Box::new(E::Slot(2))))]))]);
        matched.predicates = vec![E::RelationMatch(0, Box::new(E::Binding(0)), Box::new(E::Binding(1))),
            E::Equal(Box::new(E::Binding(1)), Box::new(E::Argument(0)))];
        matched.required_present = vec![2];
        matched.required_absent = vec![4];
        let program = Arc::new(ExecutableProgramV1 { initial_configuration: vec![], projection: None,
            rules: vec![matched, rule(2, vec![(5, E::Accumulate(Box::new(E::Argument(1))))])] });
        let cache = Mutex::new(EvaluationCache::default());
        let mut configuration: Vec<ExecutableSlotV1> = vec![table(&[(1, 3.0), (2, 3.0)]),
            table(&[]), number(4.0), number(10.0), V::Boolean(false), number(0.0)].into_iter().map(Into::into).collect();
        configuration[4] = ExecutableSlotV1::Absent(ExecutableValueKindV1::Boolean);
        let arguments = vec![number(3.0), number(2.0)];
        let original = run(&program, Some(&cache), &configuration, arguments.clone(), 1).unwrap();
        assert_eq!(original, run(&program, None, &configuration, arguments.clone(), 1).unwrap());
        let hit = cache.lock().unwrap().evaluate(&program, 2, &configuration, &arguments,
            || panic!("unchanged exact dependencies must reuse their evaluated effects")).unwrap();
        assert_eq!(hit, (original.0.clone(), original.1.rule_applied));
        let mut unrelated = configuration.clone();
        unrelated[3] = number(99.0).into();
        let hit = cache.lock().unwrap().evaluate(&program, 2, &unrelated, &arguments,
            || panic!("unrelated state is preserved without re-evaluating effects")).unwrap();
        assert_eq!(hit.0[3], number(99.0));
        for (index, changed) in [(0, table(&[(1, 9.0)]).into()), (1, table(&[(8, 11.0)]).into()),
            (2, number(7.0).into()), (2, ExecutableSlotV1::Absent(ExecutableValueKindV1::Number)),
            (4, V::Boolean(true).into()), (5, number(12.0).into())] {
            let mut changed_configuration = configuration.clone();
            changed_configuration[index] = changed;
            assert_eq!(run(&program, Some(&cache), &changed_configuration, arguments.clone(), 2),
                run(&program, None, &changed_configuration, arguments.clone(), 2));
        }
        for arguments in [vec![number(9.0), number(2.0)], vec![number(3.0), V::Boolean(true)]] {
            assert_eq!(run(&program, Some(&cache), &configuration, arguments.clone(), 3),
                run(&program, None, &configuration, arguments, 3));
        }
        assert_eq!(run(&program, Some(&cache), &original.0, vec![number(3.0), number(2.0)], 4),
            run(&program, None, &original.0, vec![number(3.0), number(2.0)], 4));
    }

    #[test]
    fn fresh_results_and_traces_are_evaluated_at_their_exact_occurrence() {
        let cache = Mutex::new(EvaluationCache::default());
        let program = Arc::new(ExecutableProgramV1 { initial_configuration: vec![], projection: None,
            rules: vec![rule(2, vec![(0, E::FreshReferent { domain: 7, binder: 1 })])] });
        let configuration = [V::Referent(ExecutableReferentV1::declared(7, 1)).into()];
        let first = run(&program, Some(&cache), &configuration, vec![], 1).unwrap();
        let second = run(&program, Some(&cache), &configuration, vec![], 2).unwrap();
        assert_ne!(first.0, second.0);
        assert_eq!(second, run(&program, None, &configuration, vec![], 2).unwrap());
        assert!(cache.lock().unwrap().entries[&2].saved.is_none());
        let program = Arc::new(ExecutableProgramV1 { initial_configuration: vec![], projection: None,
            rules: vec![rule(2, vec![(0, E::Argument(0))])] });
        let arguments = vec![number(3.0)];
        let configuration = [number(0.0).into()];
        run(&program, Some(&cache), &configuration, arguments.clone(), 1).unwrap();
        let mut observed = ExecutableEvaluationTraceV1::default();
        let mut expected = ExecutableEvaluationTraceV1::default();
        let occurrence = ExecutableOccurrenceV1 { entry: 2, arguments };
        let result = evaluator(&program, Some(&cache)).prepare_step_traced(occurrence.clone(), 2, 2,
            &configuration, Some(&mut observed)).unwrap();
        assert_eq!(result, evaluator(&program, None).prepare_step_traced(occurrence, 2, 2,
            &configuration, Some(&mut expected)).unwrap());
        assert_eq!(observed, expected);
        assert!(!observed.rules.is_empty());
        let changed_program = Arc::new(ExecutableProgramV1 { rules: vec![rule(2, vec![(0, E::Constant(number(9.0)))])],
            ..program.as_ref().clone() });
        let result = run(&changed_program, Some(&cache), &configuration, vec![number(3.0)], 3).unwrap();
        assert_eq!(result.0[0], number(9.0));
        assert!(Arc::ptr_eq(cache.lock().unwrap().program.as_ref().unwrap(), &changed_program));
    }

    #[test]
    fn changed_closure_and_nested_query_dependencies_match_clean_execution() {
        let mut derived = rule(9, vec![(1, E::DerivedRelation(vec![ExecutableRelationEffectV1::Insert(
            E::Binding(0), E::Binding(1))]))]);
        derived.predicates = vec![E::RelationMatch(0, Box::new(E::Binding(0)), Box::new(E::Binding(1)))];
        let query = E::Sum { inputs: vec![E::Slot(3)],
            predicates: vec![E::RelationMatch(1, Box::new(E::Binding(0)), Box::new(E::Binding(1)))],
            value: Box::new(E::Add(Box::new(E::Binding(1)), Box::new(E::Argument(0)))) };
        let program = Arc::new(ExecutableProgramV1 { initial_configuration: vec![], projection: None,
            rules: vec![derived, rule(2, vec![(2, query)])] });
        let cache = Mutex::new(EvaluationCache::default());
        let mut configuration: Vec<ExecutableSlotV1> = vec![table(&[(1, 3.0)]), table(&[]), number(0.0), number(4.0)]
            .into_iter().map(Into::into).collect();
        for ordinal in 1..=4 {
            if ordinal == 3 { configuration[0] = table(&[(1, 9.0), (2, 9.0)]).into(); }
            if ordinal == 4 { configuration[3] = number(8.0).into(); }
            let cached = run(&program, Some(&cache), &configuration, vec![], ordinal).unwrap();
            assert_eq!(cached, run(&program, None, &configuration, vec![], ordinal).unwrap());
            configuration = cached.0;
        }
        assert_eq!(configuration[2], number(34.0));
    }
}
