//! Bounded finite relation matching and simultaneous exact-row effects.
use super::*;

const MAX_JOIN_VISITS: usize = 65_536;
const MAX_MATCHES: usize = 4_096;
const MAX_BINDINGS: usize = 128;

/// Relation-level totality is checked on the complete candidate, not between
/// individual row effects. A Mode's one-result guarantee does not imply it.
pub(super) fn validate_contracts(configuration: &[ExecutableSlotV1]) -> Result<(), ExecutableErrorV1> {
    let tables = configuration.iter().filter_map(|slot| match slot.value() {
        Some(ExecutableValueV1::RelationTable(table)) => Some(table),
        _ => None,
    }).collect::<Vec<_>>();
    if !tables.iter().any(|table| table.total) { return Ok(()); }
    let mut participants = BTreeMap::<u32, BTreeSet<ExecutableReferentV1>>::new();
    for table in &tables {
        for (subject, values) in &table.rows {
            participants.entry(subject.domain).or_default().insert(subject.clone());
            for value in values {
                if let ExecutableValueV1::Referent(value) = value {
                    participants.entry(value.domain).or_default().insert(value.clone());
                }
            }
        }
    }
    for table in tables.into_iter().filter(|table| table.total) {
        if table.cardinality != ExecutableRelationCardinalityV1::One {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        if participants.get(&table.subject_domain).into_iter().flatten().any(|subject|
            table.rows.get(subject).is_none_or(|values| values.len() != 1)) {
            return Err(ExecutableErrorV1::MissingState);
        }
    }
    Ok(())
}

pub(super) fn validate_bindings(rule: &ExecutableRuleV1) -> Result<(), ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    fn check(
        value: &E,
        bound: &mut BTreeSet<u16>,
        pattern: bool,
        depth: usize,
        query_inputs: Option<usize>,
    ) -> Result<(), ExecutableErrorV1> {
        if depth > MAX_EXPRESSION_DEPTH {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        match value {
            E::Sum { inputs, predicates, value } => {
                if query_inputs.is_some() || pattern {
                    return Err(ExecutableErrorV1::MalformedProgram);
                }
                for input in inputs {
                    check(input, bound, false, depth + 1, None)?;
                }
                let mut local = BTreeSet::new();
                for predicate in predicates {
                    check(predicate, &mut local, false, depth + 1, Some(inputs.len()))?;
                }
                check(value, &mut local, false, depth + 1, Some(inputs.len()))?;
            }
            E::Binding(binding) => {
                if usize::from(*binding) >= MAX_BINDINGS {
                    return Err(ExecutableErrorV1::ResourceLimit);
                }
                if pattern {
                    bound.insert(*binding);
                } else if !bound.contains(binding) {
                    return Err(ExecutableErrorV1::MalformedProgram);
                }
            }
            E::ReferentFacet { value, .. } => check(value, bound, pattern, depth + 1, query_inputs)?,
            E::RelationMatch(_, a, b) => {
                check(a, bound, true, depth + 1, query_inputs)?;
                check(b, bound, true, depth + 1, query_inputs)?;
            }
            E::RelationEffects(effects) | E::DerivedRelation(effects) => {
                for effect in effects {
                    let (_, a, b) = effect.parts();
                    check(a, bound, false, depth + 1, query_inputs)?;
                    check(b, bound, false, depth + 1, query_inputs)?;
                }
            }
            E::Not(a) | E::Accumulate(a) | E::SquareRoot(a) | E::TextTransform(_, a) => check(a, bound, false, depth + 1, query_inputs)?,
            E::RelationRead(a, b)
            | E::RelationPresent(a, b)
            | E::RelationRemoveRow(a, b)
            | E::Concatenate(a, b)
            | E::StartsWith(a, b)
            | E::ContainsText(a, b)
            | E::Add(a, b)
            | E::Subtract(a, b)
            | E::Multiply(a, b)
            | E::Divide(a, b)
            | E::GreaterThan(a, b)
            | E::LessThanOrEqual(a, b)
            | E::Equal(a, b)
            | E::And(a, b)
            | E::SetInsert(a, b)
            | E::SetContains(a, b)
            | E::SetRemove(a, b) => {
                check(a, bound, false, depth + 1, query_inputs)?;
                check(b, bound, false, depth + 1, query_inputs)?;
            }
            E::RelationPut(a, b, c)
            | E::RelationInsert(a, b, c)
            | E::RelationRemoveValue(a, b, c)
            | E::Conditional(a, b, c)
            | E::Clamp(a, b, c) => {
                check(a, bound, false, depth + 1, query_inputs)?;
                check(b, bound, false, depth + 1, query_inputs)?;
                check(c, bound, false, depth + 1, query_inputs)?;
            }
            E::Argument(ordinal) if query_inputs.is_some_and(|count| usize::from(*ordinal) >= count) => {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            E::FreshReferent { .. } if query_inputs.is_some() => {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            E::Constant(_) | E::Slot(_) | E::Argument(_) | E::FreshReferent { .. } => {}
        }
        Ok(())
    }
    let mut bound = BTreeSet::new();
    for predicate in &rule.predicates {
        check(predicate, &mut bound, false, 0, None)?;
    }
    for (_, value) in &rule.assignments {
        check(value, &mut bound, false, 0, None)?;
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExecutableRelationEffectV1 {
    Put(ExecutableExpressionV1, ExecutableExpressionV1),
    Insert(ExecutableExpressionV1, ExecutableExpressionV1),
    Remove(ExecutableExpressionV1, ExecutableExpressionV1),
    Accumulate(ExecutableExpressionV1, ExecutableExpressionV1),
}

impl ExecutableRelationEffectV1 {
    pub(super) fn parts(&self) -> (u8, &ExecutableExpressionV1, &ExecutableExpressionV1) {
        match self {
            Self::Put(s, v) => (0, s, v),
            Self::Insert(s, v) => (1, s, v),
            Self::Remove(s, v) => (2, s, v),
            Self::Accumulate(s, v) => (3, s, v),
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct Matched {
    pub bindings: BTreeMap<u16, ExecutableValueV1>,
    pub predicates: Vec<EvaluatedValue>,
}

// Successful queries belong to one immutable pre-state evaluation scope; no
// result survives another preparation, state, or step.
#[derive(Default)]
pub(super) struct SumQueries {
    entries: Vec<SumQuery>,
}

struct SumQuery {
    predicates: Vec<ExecutableExpressionV1>,
    contribution: ExecutableExpressionV1,
    inputs: Vec<ExecutableValueV1>,
    result: ExecutableValueV1,
    reads: Vec<ExecutableReadV1>,
}

pub(super) fn sum(
    inputs: &[ExecutableExpressionV1],
    predicates: &[ExecutableExpressionV1],
    value: &ExecutableExpressionV1,
    configuration: &[ExecutableSlotV1],
    arguments: &[ExecutableValueV1],
    context: EvaluationContextV1,
) -> Result<ExecutableValueV1, ExecutableErrorV1> {
    let inputs = inputs.iter().map(|input| evaluate(input, configuration, arguments, context))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(queries) = context.sum_queries {
        if let Some(previous) = queries.borrow().entries.iter().find(|previous|
            previous.predicates == predicates && previous.contribution == *value && previous.inputs == inputs) {
            if let Some(reads) = context.reads {
                reads.borrow_mut().extend(previous.reads.iter().cloned());
            }
            return Ok(previous.result.clone());
        }
    }
    let _profile = source_profile_scope_v1(SourceProfilePhaseV1::SumQuery);
    let query_reads = std::cell::RefCell::new(Vec::new());
    let query_context = EvaluationContextV1 { reads: Some(&query_reads), ..context };
    let mut visits = 0;
    let mut total = 0.0;
    for (matched, accepted) in match_rule(predicates, configuration, &inputs,
        EvaluationContextV1 { bindings: None, ..query_context }, &mut visits)? {
        if let Some(reads) = query_context.reads {
            for predicate in &matched.predicates {
                reads.borrow_mut().extend(predicate.reads.iter().cloned());
            }
        }
        if accepted {
            let contribution = evaluate(value, configuration, &inputs,
                EvaluationContextV1 { bindings: Some(&matched.bindings), ..query_context })?;
            total += contribution.as_number().ok_or(ExecutableErrorV1::TypeMismatch)?;
            if !total.is_finite() {
                return Err(ExecutableErrorV1::NumericDomain);
            }
        }
    }
    let result = ExecutableValueV1::number(total)?;
    let query_reads = query_reads.into_inner();
    if let Some(reads) = context.reads {
        reads.borrow_mut().extend(query_reads.iter().cloned());
    }
    if let Some(queries) = context.sum_queries {
        queries.borrow_mut().entries.push(SumQuery {
            predicates: predicates.to_vec(), contribution: value.clone(), inputs,
            result: result.clone(), reads: query_reads,
        });
    }
    Ok(result)
}

fn unify(
    expression: &ExecutableExpressionV1,
    value: &ExecutableValueV1,
    bindings: &mut BTreeMap<u16, ExecutableValueV1>,
    configuration: &[ExecutableSlotV1],
    arguments: &[ExecutableValueV1],
    context: EvaluationContextV1,
) -> Result<bool, ExecutableErrorV1> {
    if let ExecutableExpressionV1::ReferentFacet {
        value: inner,
        domain,
        members,
    } = expression
    {
        if let ExecutableExpressionV1::Binding(binding) = inner.as_ref() {
            if let Some(previous) = bindings.get(binding) {
                return Ok(facet_value(previous.clone(), *domain, members).as_ref() == Some(value));
            }
            if facet_value(value.clone(), *domain, members).as_ref() != Some(value) {
                return Ok(false);
            }
            return unify(inner, value, bindings, configuration, arguments, context);
        }
        return Ok(facet_value(
            evaluate(
                inner,
                configuration,
                arguments,
                EvaluationContextV1 {
                    bindings: Some(bindings),
                    ..context
                },
            )?,
            *domain,
            members,
        )
        .as_ref()
            == Some(value));
    }
    if let ExecutableExpressionV1::Binding(binding) = expression {
        if let Some(previous) = bindings.get(binding) {
            return Ok(previous == value);
        }
        if bindings.len() == MAX_BINDINGS {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        bindings.insert(*binding, value.clone());
        return Ok(true);
    }
    Ok(evaluate(
        expression,
        configuration,
        arguments,
        EvaluationContextV1 {
            bindings: Some(bindings),
            ..context
        },
    )? == *value)
}

pub(super) fn facet_value(
    value: ExecutableValueV1,
    domain: u32,
    members: &[u32],
) -> Option<ExecutableValueV1> {
    let ExecutableValueV1::Referent(mut referent) = value else {
        return None;
    };
    if referent.domain == domain {
        return Some(ExecutableValueV1::Referent(referent));
    }
    let ExecutableReferentIdentityV1::Declared(id) = referent.identity else {
        return None;
    };
    if members.binary_search(&id).is_err() {
        return None;
    }
    referent.domain = domain;
    Some(ExecutableValueV1::Referent(referent))
}

fn bound_pattern(
    expression: &ExecutableExpressionV1,
    bindings: &BTreeMap<u16, ExecutableValueV1>,
) -> Option<bool> {
    match expression {
        ExecutableExpressionV1::Binding(binding) => Some(bindings.contains_key(binding)),
        ExecutableExpressionV1::ReferentFacet { value, .. } => bound_pattern(value, bindings),
        _ => None,
    }
}

/// Complete finite positive matching or an explicit error. Never interpret a
/// bound-exhausted prefix as no match. Duplicate derivations of the same exact
/// substitution are one match; equal-valued distinct referents are not equal.
pub(super) fn match_rule(
    predicates: &[ExecutableExpressionV1],
    configuration: &[ExecutableSlotV1],
    arguments: &[ExecutableValueV1],
    context: EvaluationContextV1,
    visits: &mut usize,
) -> Result<Vec<(Matched, bool)>, ExecutableErrorV1> {
    let mut active = vec![Matched::default()];
    let mut rejected = Vec::new();
    for predicate in predicates {
        if let ExecutableExpressionV1::RelationMatch(slot, subject_pattern, value_pattern) =
            predicate
        {
            let Some(ExecutableValueV1::RelationTable(table)) = configuration
                .get(usize::from(*slot))
                .and_then(ExecutableSlotV1::value)
            else {
                return Err(ExecutableErrorV1::TypeMismatch);
            };
            let mut next = BTreeMap::new();
            let mut by_value = None::<BTreeMap<
                &ExecutableValueV1,
                Vec<(&ExecutableReferentV1, &ExecutableValueV1)>,
            >>;
            for incoming in active {
                let start_visits = *visits;
                let mut found = false;
                let unbound = bound_pattern(subject_pattern, &incoming.bindings) == Some(false);
                let bound_subject = if unbound {
                    None
                } else {
                    let evaluation = EvaluationContextV1 {
                        bindings: Some(&incoming.bindings),
                        ..context
                    };
                    if let ExecutableExpressionV1::ReferentFacet {
                        value,
                        domain,
                        members,
                    } = subject_pattern.as_ref()
                    {
                        facet_value(
                            evaluate(value, configuration, arguments, evaluation)?,
                            *domain,
                            members,
                        )
                    } else {
                        Some(evaluate(
                            subject_pattern,
                            configuration,
                            arguments,
                            evaluation,
                        )?)
                    }
                };
                let mut rows: Box<
                    dyn Iterator<Item = (&ExecutableReferentV1, &BTreeSet<ExecutableValueV1>)> + '_,
                > = if !unbound && bound_subject.is_none() {
                    Box::new(std::iter::empty())
                } else if let Some(subject) = bound_subject.as_ref() {
                    let subject = table.subject(subject)?;
                    Box::new(table.rows.get_key_value(subject).into_iter())
                } else {
                    Box::new(table.rows.iter())
                };
                // A bound value is an exact lookup in the same pre-state.
                // Keep partial expressions inside unification so an empty
                // subject match never evaluates an otherwise unused value.
                let value_bound = matches!(value_pattern.as_ref(),
                    ExecutableExpressionV1::Constant(_) | ExecutableExpressionV1::Argument(_))
                    || bound_pattern(value_pattern, &incoming.bindings) == Some(true);
                let first_row = rows.next();
                let bound_value = if value_bound && first_row.is_some() {
                    let evaluation = EvaluationContextV1 {
                        bindings: Some(&incoming.bindings),
                        ..context
                    };
                    if let ExecutableExpressionV1::ReferentFacet { value, domain, members } = value_pattern.as_ref() {
                        facet_value(evaluate(value, configuration, arguments, evaluation)?, *domain, members)
                    } else {
                        Some(evaluate(value_pattern, configuration, arguments, evaluation)?)
                    }
                } else { None };
                let candidates: Box<dyn Iterator<Item = (&ExecutableReferentV1, &ExecutableValueV1)> + '_> =
                    if value_bound && bound_value.is_none() {
                        Box::new(std::iter::empty())
                    } else if let Some(value) = bound_value.as_ref() {
                        if unbound {
                            if by_value.is_none() {
                                let mut index = BTreeMap::<_, Vec<_>>::new();
                                for (subject, values) in &table.rows {
                                    for value in values {
                                        *visits = visits.checked_add(1).ok_or(ExecutableErrorV1::ResourceLimit)?;
                                        if *visits > MAX_JOIN_VISITS { return Err(ExecutableErrorV1::ResourceLimit); }
                                        index.entry(value).or_default().push((subject, value));
                                    }
                                }
                                by_value = Some(index);
                            }
                            Box::new(by_value.as_ref().unwrap().get(value).into_iter().flatten().copied())
                        } else {
                            Box::new(first_row.into_iter().filter_map(|(subject, values)| values.get(value).map(|value| (subject, value))))
                        }
                    } else {
                        Box::new(first_row.into_iter().chain(rows).flat_map(|(subject, values)| values.iter().map(move |value| (subject, value))))
                    };
                for (subject, value) in candidates {
                        *visits = visits
                            .checked_add(1)
                            .ok_or(ExecutableErrorV1::ResourceLimit)?;
                        if *visits > MAX_JOIN_VISITS {
                            return Err(ExecutableErrorV1::ResourceLimit);
                        }
                        let mut matched = incoming.clone();
                        if !unify(
                            subject_pattern,
                            &ExecutableValueV1::Referent(subject.clone()),
                            &mut matched.bindings,
                            configuration,
                            arguments,
                            context,
                        )? || !unify(
                            value_pattern,
                            value,
                            &mut matched.bindings,
                            configuration,
                            arguments,
                            context,
                        )? {
                            continue;
                        }
                        found = true;
                        matched.predicates.push(EvaluatedValue {
                            value: ExecutableValueV1::Boolean(true),
                            reads: vec![ExecutableReadV1::RelationRow(
                                *slot,
                                subject.clone(),
                                value.clone(),
                            )],
                        });
                        next.entry(matched.bindings.clone()).or_insert(matched);
                        if next.len() > MAX_MATCHES {
                            return Err(ExecutableErrorV1::ResourceLimit);
                        }
                }
                if !found {
                    let mut incoming = incoming;
                    incoming.predicates.push(EvaluatedValue {
                        value: ExecutableValueV1::Boolean(false),
                        reads: vec![ExecutableReadV1::RelationSearch(
                            *slot,
                            bound_subject
                                .as_ref()
                                .and_then(ExecutableValueV1::as_referent)
                                .cloned(),
                            *visits - start_visits,
                        )],
                    });
                    rejected.push((incoming, false));
                    if rejected.len() > MAX_MATCHES {
                        return Err(ExecutableErrorV1::ResourceLimit);
                    }
                }
            }
            active = next.into_values().collect();
        } else {
            let mut next = Vec::new();
            for mut matched in active {
                let evaluated = evaluate_with_reads(
                    predicate,
                    configuration,
                    arguments,
                    EvaluationContextV1 {
                        bindings: Some(&matched.bindings),
                        ..context
                    },
                )?;
                let accepted = boolean(evaluated.value.clone())?;
                matched.predicates.push(evaluated);
                if accepted {
                    next.push(matched);
                } else {
                    rejected.push((matched, false));
                    if rejected.len() > MAX_MATCHES {
                        return Err(ExecutableErrorV1::ResourceLimit);
                    }
                }
            }
            active = next;
        }
        if active.is_empty() {
            break;
        }
    }
    rejected.extend(active.into_iter().map(|matched| (matched, true)));
    Ok(rejected)
}

pub(super) fn occurrence_identity(
    context: EvaluationContextV1,
    rule: usize,
    bindings: &BTreeMap<u16, ExecutableValueV1>,
) -> Result<[u8; IDENTITY_BYTES], ExecutableErrorV1> {
    let mut bytes = Vec::new();
    for (binding, value) in bindings {
        bytes.extend_from_slice(&binding.to_le_bytes());
        encode_value(&mut bytes, value)?;
    }
    Ok(runtime_domain_hash(
        "clause/relational-match/v1",
        &[
            &context.allocation_root,
            &context.step_ordinal.to_be_bytes(),
            &(rule as u64).to_be_bytes(),
            &bytes,
        ],
    ))
}

#[derive(Default)]
pub(super) struct RowEffects {
    rows: BTreeMap<(u16, ExecutableReferentV1), Vec<(u8, ExecutableValueV1)>>,
}

impl RowEffects {
    pub fn push(
        &mut self,
        slot: u16,
        mode: u8,
        subject: ExecutableValueV1,
        value: ExecutableValueV1,
        configuration: &[ExecutableSlotV1],
    ) -> Result<(), ExecutableErrorV1> {
        let Some(ExecutableValueV1::RelationTable(table)) = configuration
            .get(usize::from(slot))
            .and_then(ExecutableSlotV1::value)
        else {
            return Err(ExecutableErrorV1::TypeMismatch);
        };
        let subject = table.subject(&subject)?.clone();
        if !table.value_matches(&value) {
            return Err(ExecutableErrorV1::TypeMismatch);
        }
        let prior = self.rows.entry((slot, subject.clone())).or_default();
        if table.cardinality == ExecutableRelationCardinalityV1::Many {
            if !matches!(mode, 1 | 2) {
                return Err(ExecutableErrorV1::TypeMismatch);
            }
            if prior.iter().any(|(_, previous)| previous == &value) {
                return Err(ExecutableErrorV1::ConflictingStateEffects(slot));
            }
        } else {
            if mode == 1 {
                return Err(ExecutableErrorV1::TypeMismatch);
            }
            if mode == 3
                && (table.value_kind != ExecutableRelationValueKindV1::Number
                    || !table.rows.contains_key(&subject))
            {
                return Err(ExecutableErrorV1::TypeMismatch);
            }
            if !prior.is_empty() && (mode != 3 || prior.iter().any(|(mode, _)| *mode != 3)) {
                return Err(ExecutableErrorV1::ConflictingStateEffects(slot));
            }
        }
        prior.push((mode, value));
        Ok(())
    }

    pub fn apply(self, next: &mut [ExecutableSlotV1]) -> Result<(), ExecutableErrorV1> {
        for ((slot, subject), effects) in self.rows {
            let Some(ExecutableValueV1::RelationTable(current)) = next[usize::from(slot)].value()
            else {
                return Err(ExecutableErrorV1::TypeMismatch);
            };
            let mut table = current.clone();
            let subject = ExecutableValueV1::Referent(subject);
            if effects[0].0 == 3 {
                let mut value = number(table.read(&subject)?)?;
                let mut deltas = effects
                    .into_iter()
                    .map(|(_, value)| number(value))
                    .collect::<Result<Vec<_>, _>>()?;
                deltas.sort_by(f64::total_cmp);
                for delta in deltas {
                    value += delta;
                    if !value.is_finite() {
                        return Err(ExecutableErrorV1::NumericDomain);
                    }
                }
                table = table.put(&subject, ExecutableValueV1::number(value)?)?;
            } else {
                for (mode, value) in effects {
                    table = match mode {
                        0 => table.put(&subject, value)?,
                        1 => table.insert(&subject, value)?,
                        2 if table.cardinality == ExecutableRelationCardinalityV1::Many => {
                            table.remove_value(&subject, &value)?
                        }
                        2 => {
                            if table.read(&subject)? != value {
                                return Err(ExecutableErrorV1::MissingState);
                            }
                            table.remove_row(&subject)?
                        }
                        _ => return Err(ExecutableErrorV1::MalformedProgram),
                    };
                }
            }
            next[usize::from(slot)] = ExecutableValueV1::RelationTable(table).into();
        }
        Ok(())
    }
}
#[cfg(test)]
mod sum_reuse_tests {
    use super::*;

    #[test]
    fn preparation_effects_share_queries_with_exact_reads_and_independent_errors() {
        use ExecutableExpressionV1 as E;
        let number = |n| ExecutableValueV1::number(n).unwrap();
        let configuration = vec![ExecutableValueV1::RelationTable(ExecutableRelationTableV1 {
            subject_domain: 7, value_kind: ExecutableRelationValueKindV1::Number,
            value_domain: None, cardinality: ExecutableRelationCardinalityV1::One,
            total: false,
            rows: (0..3).map(|id| (ExecutableReferentV1::declared(7, id),
                BTreeSet::from([number(1.0)]))).collect(),
        }).into()];
        let query = E::Sum {
            inputs: vec![E::Argument(0)],
            predicates: vec![E::RelationMatch(0, Box::new(E::Binding(0)), Box::new(E::Argument(0)))],
            value: Box::new(E::Constant(number(1.0))),
        };
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES],
            step_ordinal: 0, reads: None, sum_queries: None, bindings: None, relational_occurrence: None };
        let expected = [1.0, 2.0].map(|input|
            evaluate_with_reads(&query, &configuration, &[number(input)], context).unwrap());
        assert_eq!(expected[0].value, number(3.0));
        assert_eq!(expected[1].value, number(0.0));
        assert!(expected[1].reads.iter().any(|read| matches!(read, ExecutableReadV1::RelationSearch(..))));
        for _preparation in 0..2 {
            let queries = std::cell::RefCell::new(SumQueries::default());
            let shared = EvaluationContextV1 { sum_queries: Some(&queries), ..context };
            assert!(begin_executable_source_profile_v1());
            for _effect in 0..3 {
                for (input, expected) in [1.0, 2.0].into_iter().zip(&expected) {
                    let actual = evaluate_with_reads(&query, &configuration, &[number(input)], shared).unwrap();
                    assert_eq!(actual.value, expected.value);
                    assert_eq!(actual.reads, expected.reads);
                }
            }
            let report = finish_executable_source_profile_v1().unwrap();
            assert_eq!(report.phases[SourceProfilePhaseV1::SumEvaluation as usize].calls, 6);
            assert_eq!(report.phases[SourceProfilePhaseV1::SumQuery as usize].calls, 2);
            let invalid = E::Sum { inputs: vec![], predicates: vec![],
                value: Box::new(E::Constant(ExecutableValueV1::Boolean(true))) };
            for _ in 0..2 {
                assert!(matches!(evaluate_with_reads(&invalid, &configuration, &[], shared),
                    Err(ExecutableErrorV1::TypeMismatch)));
                assert_eq!(queries.borrow().entries.len(), 2);
            }
        }
    }

    #[test]
    fn repeated_sum_preserves_value_and_ordered_reads() {
        let number = |n| ExecutableValueV1::number(n).unwrap();
        let table = ExecutableRelationTableV1 {
            subject_domain: 7,
            value_kind: ExecutableRelationValueKindV1::Number,
            value_domain: None,
            cardinality: ExecutableRelationCardinalityV1::One,
            total: false,
            rows: (0..100).map(|id| (ExecutableReferentV1::declared(7, id),
                BTreeSet::from([number(1.0)]))).collect(),
        };
        let configuration = vec![ExecutableValueV1::RelationTable(table).into()];
        let sum = ExecutableExpressionV1::Sum {
            inputs: vec![],
            predicates: vec![ExecutableExpressionV1::RelationMatch(0,
                Box::new(ExecutableExpressionV1::Binding(0)),
                Box::new(ExecutableExpressionV1::Binding(1)))],
            value: Box::new(ExecutableExpressionV1::Binding(1)),
        };
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES],
            step_ordinal: 0, reads: None, sum_queries: None, bindings: None, relational_occurrence: None };
        let expected = evaluate_with_reads(&sum, &configuration, &[], context).unwrap();
        let mut repeated = sum.clone();
        for _ in 1..8 {
            repeated = ExecutableExpressionV1::Add(Box::new(repeated), Box::new(sum.clone()));
        }
        assert!(begin_executable_source_profile_v1());
        let started = std::time::Instant::now();
        for _ in 0..84 {
            let actual = evaluate_with_reads(&repeated, &configuration, &[], context).unwrap();
            assert_eq!(actual.value, number(800.0));
            assert_eq!(actual.reads, (0..8).flat_map(|_| expected.reads.iter().cloned()).collect::<Vec<_>>());
        }
        let report = finish_executable_source_profile_v1().unwrap();
        eprintln!("repeated sum: {:?}; {}", started.elapsed(), report.to_json());
        assert_eq!(report.phases[SourceProfilePhaseV1::SumEvaluation as usize].calls, 672);
        assert_eq!(report.phases[SourceProfilePhaseV1::SumQuery as usize].calls, 84);

        // Equal query shapes with different evaluated inputs cannot share a result.
        let query = |input| ExecutableExpressionV1::Sum {
            inputs: vec![ExecutableExpressionV1::Constant(number(input))],
            predicates: vec![],
            value: Box::new(ExecutableExpressionV1::Argument(0)),
        };
        let different = ExecutableExpressionV1::Add(Box::new(query(1.0)), Box::new(query(2.0)));
        assert_eq!(evaluate_with_reads(&different, &[], &[], context).unwrap().value, number(3.0));
        let changed = vec![ExecutableValueV1::RelationTable(ExecutableRelationTableV1 {
            rows: BTreeMap::new(),
            ..match configuration[0].value().unwrap() {
                ExecutableValueV1::RelationTable(table) => table.clone(),
                _ => unreachable!(),
            }
        }).into()];
        assert_eq!(evaluate_with_reads(&repeated, &changed, &[], context).unwrap().value, number(0.0));
        let invalid = ExecutableExpressionV1::Sum { inputs: vec![], predicates: vec![],
            value: Box::new(ExecutableExpressionV1::Constant(ExecutableValueV1::Boolean(true))) };
        assert!(matches!(evaluate_with_reads(&invalid, &[], &[], context), Err(ExecutableErrorV1::TypeMismatch)));
    }
}
