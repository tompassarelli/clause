//! Positive set-valued relation closure. Derived rows never become roots.
use super::*;

const MAX_RULE_CHECKS: usize = 65_536;
const MAX_DERIVED_ROWS: usize = 4_096;

pub(super) fn is_derivation(rule: &ExecutableRuleV1) -> bool {
    rule.assignments.iter().any(|(_, value)| matches!(value,
        ExecutableExpressionV1::DerivedRelation(_)))
}

// Only row matches introduce relational dependencies. All remaining values
// are pure expressions over their bound values, so adding rows cannot revoke
// a conclusion. Absence, aggregates, occurrence arguments and allocation need
// separate semantics and are not admitted by this positive closure.
fn pure(value: &ExecutableExpressionV1, depth: usize) -> Result<(), ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    if depth > MAX_EXPRESSION_DEPTH { return Err(ExecutableErrorV1::ResourceLimit); }
    match value {
        E::Constant(_) | E::Binding(_) => Ok(()),
        E::ReferentFacet { value, .. } | E::Not(value) | E::SquareRoot(value) => pure(value, depth + 1),
        E::Add(a, b) | E::Subtract(a, b) | E::Multiply(a, b) | E::Divide(a, b)
        | E::Equal(a, b) | E::GreaterThan(a, b) | E::LessThanOrEqual(a, b)
        | E::And(a, b) | E::Concatenate(a, b) => {
            pure(a, depth + 1)?;
            pure(b, depth + 1)
        }
        E::Conditional(a, b, c) | E::Clamp(a, b, c) => {
            pure(a, depth + 1)?;
            pure(b, depth + 1)?;
            pure(c, depth + 1)
        }
        _ => Err(ExecutableErrorV1::MalformedProgram),
    }
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
        for predicate in &rule.predicates {
            if let E::RelationMatch(_, subject, value) = predicate {
                pure(subject, 0)?;
                pure(value, 0)?;
            } else { pure(predicate, 0)?; }
        }
        for (slot, expression) in &rule.assignments {
            let E::DerivedRelation(effects) = expression else {
                return Err(ExecutableErrorV1::MalformedProgram);
            };
            let Some(ExecutableValueV1::RelationTable(table)) = initial
                .get(usize::from(*slot)).and_then(ExecutableSlotV1::value) else {
                return Err(ExecutableErrorV1::MalformedProgram);
            };
            if table.cardinality != ExecutableRelationCardinalityV1::Many || !table.rows.is_empty() {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            targets.insert(*slot);
            for effect in effects {
                let ExecutableRelationEffectV1::Insert(subject, value) = effect else {
                    return Err(ExecutableErrorV1::MalformedProgram);
                };
                pure(subject, 0)?;
                pure(value, 0)?;
            }
        }
    }
    for rule in program.rules.iter().filter(|rule| !is_derivation(rule)) {
        if rule.assignments.iter().map(|(slot, _)| slot).chain(&rule.removals)
            .any(|slot| targets.contains(slot)) {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
    }
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
    loop {
        let prior_count = count;
        for (rule_index, rule) in &rules {
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
                        if Arc::make_mut(&mut table.rows).entry(subject.clone()).or_default().insert(value.value.clone()) {
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
        if count == prior_count { return Ok(next); }
    }
}
