//! Bounded finite relation matching and simultaneous exact-row effects.
use super::*;

const MAX_JOIN_VISITS: usize = 65_536;
const MAX_MATCHES: usize = 4_096;
const MAX_BINDINGS: usize = 128;

pub(super) fn validate_bindings(rule: &ExecutableRuleV1) -> Result<(), ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    fn check(
        value: &E,
        bound: &mut BTreeSet<u16>,
        pattern: bool,
        depth: usize,
        closed: bool,
    ) -> Result<(), ExecutableErrorV1> {
        if depth > MAX_EXPRESSION_DEPTH {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        match value {
            E::Sum { predicates, value } => {
                if closed || pattern {
                    return Err(ExecutableErrorV1::MalformedProgram);
                }
                let mut local = BTreeSet::new();
                for predicate in predicates {
                    check(predicate, &mut local, false, depth + 1, true)?;
                }
                check(value, &mut local, false, depth + 1, true)?;
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
            E::ReferentFacet { value, .. } => check(value, bound, pattern, depth + 1, closed)?,
            E::RelationMatch(_, a, b) => {
                check(a, bound, true, depth + 1, closed)?;
                check(b, bound, true, depth + 1, closed)?;
            }
            E::RelationEffects(effects) => {
                for effect in effects {
                    let (_, a, b) = effect.parts();
                    check(a, bound, false, depth + 1, closed)?;
                    check(b, bound, false, depth + 1, closed)?;
                }
            }
            E::Not(a) | E::Accumulate(a) | E::SquareRoot(a) => check(a, bound, false, depth + 1, closed)?,
            E::RelationRead(a, b)
            | E::RelationPresent(a, b)
            | E::RelationRemoveRow(a, b)
            | E::Concatenate(a, b)
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
                check(a, bound, false, depth + 1, closed)?;
                check(b, bound, false, depth + 1, closed)?;
            }
            E::RelationPut(a, b, c)
            | E::RelationInsert(a, b, c)
            | E::RelationRemoveValue(a, b, c)
            | E::Clamp(a, b, c) => {
                check(a, bound, false, depth + 1, closed)?;
                check(b, bound, false, depth + 1, closed)?;
                check(c, bound, false, depth + 1, closed)?;
            }
            E::Argument(_) | E::FreshReferent { .. } if closed => {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            E::Constant(_) | E::Slot(_) | E::Argument(_) | E::FreshReferent { .. } => {}
        }
        Ok(())
    }
    let mut bound = BTreeSet::new();
    for predicate in &rule.predicates {
        check(predicate, &mut bound, false, 0, false)?;
    }
    for (_, value) in &rule.assignments {
        check(value, &mut bound, false, 0, false)?;
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
    pub predicates: Vec<ExecutableEvaluatedExpressionV1>,
}

pub(super) fn sum(
    predicates: &[ExecutableExpressionV1],
    value: &ExecutableExpressionV1,
    configuration: &[ExecutableSlotV1],
    context: EvaluationContextV1,
) -> Result<ExecutableValueV1, ExecutableErrorV1> {
    let query = ExecutableRuleV1 {
        entry: 0,
        predicates: predicates.to_vec(),
        required_present: vec![], required_absent: vec![],
        assignments: vec![], removals: vec![],
    };
    let mut visits = 0;
    let mut total = 0.0;
    for (matched, accepted) in match_rule(&query, configuration, &[],
        EvaluationContextV1 { bindings: None, ..context }, &mut visits)? {
        if let Some(reads) = context.reads {
            for predicate in &matched.predicates {
                reads.borrow_mut().extend(predicate.reads.iter().cloned());
            }
        }
        if accepted {
            let contribution = evaluate(value, configuration, &[],
                EvaluationContextV1 { bindings: Some(&matched.bindings), ..context })?;
            total += contribution.as_number().ok_or(ExecutableErrorV1::TypeMismatch)?;
            if !total.is_finite() {
                return Err(ExecutableErrorV1::NumericDomain);
            }
        }
    }
    ExecutableValueV1::number(total)
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
    rule: &ExecutableRuleV1,
    configuration: &[ExecutableSlotV1],
    arguments: &[ExecutableValueV1],
    context: EvaluationContextV1,
    visits: &mut usize,
) -> Result<Vec<(Matched, bool)>, ExecutableErrorV1> {
    let mut active = vec![Matched::default()];
    let mut rejected = Vec::new();
    for predicate in &rule.predicates {
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
                let rows: Box<
                    dyn Iterator<Item = (&ExecutableReferentV1, &BTreeSet<ExecutableValueV1>)> + '_,
                > = if !unbound && bound_subject.is_none() {
                    Box::new(std::iter::empty())
                } else if let Some(subject) = bound_subject.as_ref() {
                    let subject = table.subject(subject)?;
                    Box::new(table.rows.get_key_value(subject).into_iter())
                } else {
                    Box::new(table.rows.iter())
                };
                for (subject, values) in rows {
                    for value in values {
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
                        matched.predicates.push(ExecutableEvaluatedExpressionV1 {
                            expression: predicate.clone(),
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
                }
                if !found {
                    let mut incoming = incoming;
                    incoming.predicates.push(ExecutableEvaluatedExpressionV1 {
                        expression: predicate.clone(),
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
                let evaluated = evaluate_explained(
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
