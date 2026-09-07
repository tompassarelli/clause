//! Stratified finite relation closure. Derived rows never become roots.
use super::*;

const MAX_RULE_CHECKS: usize = 65_536;
const MAX_DERIVED_ROWS: usize = 4_096;

pub(super) fn is_derivation(rule: &ExecutableRuleV1) -> bool {
    rule.assignments.iter().any(|(_, value)| matches!(value,
        ExecutableExpressionV1::DerivedRelation(_)))
}

// Query row reads require a completed predecessor relation. Ordinary row
// matches may participate in the same positive fixed point.
fn dependencies(
    value: &ExecutableExpressionV1,
    depth: usize,
    query: bool,
    reads: &mut BTreeMap<u16, bool>,
) -> Result<(), ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    if depth > MAX_EXPRESSION_DEPTH { return Err(ExecutableErrorV1::ResourceLimit); }
    match value {
        E::Constant(_) | E::Binding(_) => Ok(()),
        E::Argument(_) if query => Ok(()),
        E::ReferentFacet { value, .. } | E::Not(value) | E::SquareRoot(value) =>
            dependencies(value, depth + 1, query, reads),
        E::Add(a, b) | E::Subtract(a, b) | E::Multiply(a, b) | E::Divide(a, b)
        | E::Equal(a, b) | E::GreaterThan(a, b) | E::LessThanOrEqual(a, b)
        | E::And(a, b) | E::Concatenate(a, b) => {
            dependencies(a, depth + 1, query, reads)?;
            dependencies(b, depth + 1, query, reads)
        }
        E::Conditional(a, b, c) | E::Clamp(a, b, c) => {
            dependencies(a, depth + 1, query, reads)?;
            dependencies(b, depth + 1, query, reads)?;
            dependencies(c, depth + 1, query, reads)
        }
        E::Sum { inputs, predicates, value } if !query => {
            for input in inputs { dependencies(input, depth + 1, false, reads)?; }
            for predicate in predicates { predicate_dependencies(predicate, depth + 1, true, reads)?; }
            dependencies(value, depth + 1, true, reads)
        }
        _ => Err(ExecutableErrorV1::MalformedProgram),
    }
}

fn predicate_dependencies(
    predicate: &ExecutableExpressionV1,
    depth: usize,
    query: bool,
    reads: &mut BTreeMap<u16, bool>,
) -> Result<(), ExecutableErrorV1> {
    if let ExecutableExpressionV1::RelationMatch(slot, subject, value) = predicate {
        reads.entry(*slot).and_modify(|strict| *strict |= query).or_insert(query);
        dependencies(subject, depth + 1, query, reads)?;
        dependencies(value, depth + 1, query, reads)
    } else { dependencies(predicate, depth, query, reads) }
}

fn strata(rules: &[(usize, &ExecutableRuleV1)]) -> Result<Vec<usize>, ExecutableErrorV1> {
    let mut producers = BTreeMap::<u16, Vec<usize>>::new();
    for (index, (_, rule)) in rules.iter().enumerate() {
        for (slot, _) in &rule.assignments { producers.entry(*slot).or_default().push(index); }
    }
    let mut edges = Vec::new();
    for (consumer, (_, rule)) in rules.iter().enumerate() {
        let mut reads = BTreeMap::new();
        for predicate in &rule.predicates { predicate_dependencies(predicate, 0, false, &mut reads)?; }
        for (_, expression) in &rule.assignments {
            let ExecutableExpressionV1::DerivedRelation(effects) = expression else {
                return Err(ExecutableErrorV1::MalformedProgram);
            };
            for effect in effects {
                let ExecutableRelationEffectV1::Insert(subject, value) = effect else {
                    return Err(ExecutableErrorV1::MalformedProgram);
                };
                dependencies(subject, 0, false, &mut reads)?;
                dependencies(value, 0, false, &mut reads)?;
            }
        }
        for (slot, strict) in reads {
            for producer in producers.get(&slot).into_iter().flatten() {
                edges.push((*producer, consumer, usize::from(strict)));
            }
        }
    }
    let mut levels = vec![0; rules.len()];
    for _ in 0..rules.len() {
        let mut changed = false;
        for &(producer, consumer, strict) in &edges {
            let required = levels[producer] + strict;
            if required > levels[consumer] {
                levels[consumer] = required;
                changed = true;
            }
        }
        if !changed { return Ok(levels); }
    }
    if rules.is_empty() { return Ok(levels); }
    // An increasing level after every finite relaxation proves a cycle
    // containing an aggregate dependency, which has no admitted stratum.
    Err(ExecutableErrorV1::UnstratifiedDerivation)
}

pub(super) fn validate(
    program: &ExecutableProgramV1,
    initial: &[ExecutableSlotV1],
) -> Result<(), ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    let mut targets = BTreeSet::new();
    for rule in program.rules.iter().filter(|rule| is_derivation(rule)) {
        if !rule.required_present.is_empty() || !rule.required_absent.is_empty()
            || !rule.removals.is_empty() {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        for (slot, expression) in &rule.assignments {
            let E::DerivedRelation(_) = expression else {
                return Err(ExecutableErrorV1::MalformedProgram);
            };
            let Some(ExecutableValueV1::RelationTable(table)) = initial
                .get(usize::from(*slot)).and_then(ExecutableSlotV1::value) else {
                return Err(ExecutableErrorV1::MalformedProgram);
            };
            if !matches!(table.cardinality, ExecutableRelationCardinalityV1::Many | ExecutableRelationCardinalityV1::Maybe)
                || !table.rows.is_empty() {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            targets.insert(*slot);
        }
    }
    for rule in program.rules.iter().filter(|rule| !is_derivation(rule)) {
        if rule.assignments.iter().map(|(slot, _)| slot).chain(&rule.removals)
            .any(|slot| targets.contains(slot)) {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
    strata(&program.rules.iter().enumerate().filter(|(_, rule)| is_derivation(rule)).collect::<Vec<_>>())?;
    Ok(())
}

/// Return the entire least fixed point, or no result on exhaustion. The caller
/// stages this result before changing any admitted or process-resident state.
/// Retain at most one finite witness for each new row; never enumerate cyclic
/// proof trees or confuse the number of witnesses with value cardinality.
pub(super) fn close(
    program: &ExecutableProgramV1,
    configuration: &[ExecutableSlotV1],
    context: EvaluationContextV1,
    mut trace: Option<(&std::sync::Arc<ExecutableProgramV1>, &mut ExecutableEvaluationTraceV1)>,
) -> Result<Vec<ExecutableSlotV1>, ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    let rules = program.rules.iter().enumerate()
        .filter(|(_, rule)| is_derivation(rule)).collect::<Vec<_>>();
    let levels = strata(&rules)?;
    let mut next = configuration.to_vec();
    for (_, rule) in &rules {
        for (slot, _) in &rule.assignments {
            let Some(ExecutableSlotV1::Present(ExecutableValueV1::RelationTable(table))) =
                next.get_mut(usize::from(*slot)) else {
                return Err(ExecutableErrorV1::MalformedProgram);
            };
            Arc::make_mut(&mut table.rows).clear();
        }
    }
    let mut checks = 0;
    let mut visits = 0;
    let mut count = 0;
    let mut level = 0;
    let last_level = levels.iter().copied().max().unwrap_or(0);
    loop {
        let prior_count = count;
        for ((rule_index, rule), _) in rules.iter().zip(&levels).filter(|(_, stratum)| **stratum == level) {
            checks += 1;
            if checks > MAX_RULE_CHECKS { return Err(ExecutableErrorV1::ResourceLimit); }
            for (matched, accepted) in relational::match_rule(
                &rule.predicates, &next, &[], context, &mut visits,
            )? {
                if !accepted { continue; }
                let evaluation = EvaluationContextV1 { bindings: Some(&matched.bindings), ..context };
                let mut discovered = Vec::new();
                for (assignment, (slot, expression)) in rule.assignments.iter().enumerate() {
                    let E::DerivedRelation(effects) = expression else {
                        return Err(ExecutableErrorV1::MalformedProgram);
                    };
                    for (effect_index, effect) in effects.iter().enumerate() {
                        let ExecutableRelationEffectV1::Insert(subject, value) = effect else {
                            return Err(ExecutableErrorV1::MalformedProgram);
                        };
                        let subject = evaluate_with_reads(subject, &next, &[], evaluation)?;
                        let mut value = evaluate_with_reads(value, &next, &[], evaluation)?;
                        value.reads.extend(subject.reads);
                        let Some(ExecutableSlotV1::Present(ExecutableValueV1::RelationTable(table))) =
                            next.get_mut(usize::from(*slot)) else {
                            return Err(ExecutableErrorV1::MalformedProgram);
                        };
                        let subject = table.subject(&subject.value)?.clone();
                        if !table.value_matches(&value.value) { return Err(ExecutableErrorV1::TypeMismatch); }
                        let values = Arc::make_mut(&mut table.rows).entry(subject.clone()).or_default();
                        // Optional conclusions are still monotone sets: equal proofs
                        // share one value, while competing values reject the closure.
                        if table.cardinality == ExecutableRelationCardinalityV1::Maybe
                            && !values.is_empty() && !values.contains(&value.value) {
                            return Err(ExecutableErrorV1::ConflictingStateEffects(*slot));
                        }
                        if values.insert(value.value.clone()) {
                            count += 1;
                            if count > MAX_DERIVED_ROWS { return Err(ExecutableErrorV1::ResourceLimit); }
                            discovered.push((*slot, subject, assignment, effect_index, value));
                        }
                    }
                }
                if !discovered.is_empty() && let Some((retained_program, trace)) = &mut trace {
                    let index = trace.rules.len();
                    trace.push(ExecutableRuleEvaluationV1 {
                        rule: *rule_index as u16, bindings: matched.bindings,
                        required_present: vec![], required_absent: vec![], selected: true,
                        predicates: matched.predicates.into_iter().enumerate().map(|(index, value)|
                            value.retain(ExecutableExpressionReferenceV1::new(retained_program, *rule_index,
                                ExpressionCoordinate::Predicate(index)))).collect(),
                        effects: vec![],
                    });
                    for (slot, subject, assignment, effect, value) in discovered {
                        trace.effect(index, slot, false, Some(subject), Some(value.retain(
                            ExecutableExpressionReferenceV1::new(retained_program, *rule_index,
                                ExpressionCoordinate::RowEffectValue { assignment, effect }),
                        )));
                    }
                }
            }
        }
        if count == prior_count {
            if level == last_level { return Ok(next); }
            level += 1;
        }
    }
}
