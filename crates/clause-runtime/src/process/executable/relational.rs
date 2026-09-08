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
        for (subject, values) in table.rows.iter() {
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
            E::Sequence(values) => { for value in values { check(value,bound,false,depth+1,query_inputs)?; } }
            E::Record(fields) => { for value in fields.values() { check(value,bound,false,depth+1,query_inputs)?; } }
            E::SequenceSort(value) | E::SequenceCount(value) | E::ScalarText(value) | E::Field(value,_) => check(value,bound,false,depth+1,query_inputs)?,
            E::SequenceAppend(a,b) | E::SequenceJoin(a,b) | E::SequenceDrop(a,b) => { check(a,bound,false,depth+1,query_inputs)?; check(b,bound,false,depth+1,query_inputs)?; }
            E::Require(a,b,c) => { for value in [a,b,c] { check(value,bound,false,depth+1,query_inputs)?; } }
            E::SequenceFold { accumulator, item, source, initial, body } => {
                if pattern { return Err(ExecutableErrorV1::MalformedProgram); }
                check(source,bound,false,depth+1,query_inputs)?;
                check(initial,bound,false,depth+1,query_inputs)?;
                let mut nested = bound.clone();
                if !nested.insert(*accumulator) || !nested.insert(*item) { return Err(ExecutableErrorV1::MalformedProgram); }
                check(body,&mut nested,false,depth+1,query_inputs)?;
            }
            E::Foreign { .. } => return Err(ExecutableErrorV1::MalformedProgram),
            E::Let { binding, value, body } | E::SequenceMap { binding, source: value, body } => {
                if pattern { return Err(ExecutableErrorV1::MalformedProgram); }
                check(value, bound, false, depth + 1, query_inputs)?;
                let mut nested = bound.clone();
                nested.insert(*binding);
                check(body, &mut nested, false, depth + 1, query_inputs)?;
            }
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
                if pattern && usize::from(*binding) >= MAX_BINDINGS {
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Bindings(Vec<(u16, ExecutableValueV1)>);

impl Bindings {
    pub fn get(&self, binding: &u16) -> Option<&ExecutableValueV1> {
        self.0.binary_search_by_key(binding, |(key, _)| *key).ok().map(|index| &self.0[index].1)
    }

    pub fn contains_key(&self, binding: &u16) -> bool { self.get(binding).is_some() }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    pub fn insert(&mut self, binding: u16, value: ExecutableValueV1) -> Option<ExecutableValueV1> {
        match self.0.binary_search_by_key(&binding, |(key, _)| *key) {
            Ok(index) => Some(std::mem::replace(&mut self.0[index].1, value)),
            Err(index) => { self.0.insert(index, (binding, value)); None }
        }
    }

    pub fn remove(&mut self, binding: &u16) -> Option<ExecutableValueV1> {
        self.0.binary_search_by_key(binding, |(key, _)| *key).ok().map(|index| self.0.remove(index).1)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u16, &ExecutableValueV1)> {
        self.0.iter().map(|(binding, value)| (binding, value))
    }
}

impl<const N: usize> From<[(u16, ExecutableValueV1); N]> for Bindings {
    fn from(entries: [(u16, ExecutableValueV1); N]) -> Self {
        let mut bindings = Self::default();
        for (binding, value) in entries { bindings.insert(binding, value); }
        bindings
    }
}

#[derive(Clone, Default)]
pub(super) struct Matched {
    pub bindings: Arc<Bindings>,
    pub predicates: Vec<EvaluatedValue>,
}

// Successful queries belong to one immutable pre-state evaluation scope; no
// result survives another preparation, state, or step.
#[derive(Default)]
pub(super) struct SumQueries {
    entries: Vec<SumQuery>,
    prefixes: Vec<SumPrefix>,
    pub scalar_plans: Arc<ScalarPlans>,
}

#[derive(Default)]
pub(super) struct ScalarPlans(Mutex<Vec<Arc<scalar_reuse::ScalarPlan>>>, Mutex<EffectPlans>);

#[derive(Default)]
struct EffectPlans {
    program: Option<Arc<ExecutableProgramV1>>,
    plans: BTreeMap<(usize, usize, usize), Option<Arc<scalar_reuse::ScalarPlan>>>,
}

struct SumPrefix {
    predicates: Vec<ExecutableExpressionV1>,
    matches: MatchState,
    visits: usize,
    reads: Vec<ExecutableReadV1>,
    captured_reads: bool,
}

struct SumQuery {
    shape: Option<Arc<[u8]>>,
    predicates: Vec<ExecutableExpressionV1>,
    contribution: ExecutableExpressionV1,
    captured_reads: bool,
    results: Vec<SumResult>,
}

struct SumResult {
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
    sum_with_shape(inputs, predicates, value, configuration, arguments, context, None)
}

pub(super) fn sum_with_shape(
    inputs: &[ExecutableExpressionV1],
    predicates: &[ExecutableExpressionV1],
    value: &ExecutableExpressionV1,
    configuration: &[ExecutableSlotV1],
    arguments: &[ExecutableValueV1],
    context: EvaluationContextV1,
    shape: Option<&Arc<[u8]>>,
) -> Result<ExecutableValueV1, ExecutableErrorV1> {
    let same_query = |previous: &SumQuery| {
        previous.captured_reads == context.reads.is_some()
            && match (shape, previous.shape.as_ref()) {
                (Some(left), Some(right)) => left == right,
                _ => previous.contribution == *value && previous.predicates == predicates,
            }
    };
    let inputs = inputs.iter().map(|input| evaluate(input, configuration, arguments, context))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(queries) = context.sum_queries {
        if let Some(previous) = queries.borrow().entries.iter().find(|previous|
            same_query(previous))
            .and_then(|query| query.results.iter().find(|previous| previous.inputs == inputs)) {
            if let Some(reads) = context.reads {
                reads.borrow_mut().extend(previous.reads.iter().cloned());
            }
            return Ok(previous.result.clone());
        }
    }
    let _profile = source_profile_scope_v1(SourceProfilePhaseV1::SumQuery);
    let query_reads = std::cell::RefCell::new(Vec::new());
    let query_context = EvaluationContextV1 { reads: context.reads.map(|_| &query_reads), ..context };
    let plan = scalar_plan(value, query_context)?;
    let mut visits = 0;
    let mut total = 0.0;
    let matches = match_sum(predicates, configuration, &inputs,
        EvaluationContextV1 { bindings: None, ..query_context }, &mut visits)?;
    if let Some(result) = plan.as_ref().and_then(|plan| plan.compiled_sum(configuration, &inputs, &matches)) {
        total = result?;
    } else {
        let memo = plan.as_ref().map(|plan| plan.memo());
        for (matched, accepted) in matches {
              if let Some(reads) = query_context.reads {
                  for predicate in &matched.predicates {
                      reads.borrow_mut().extend(predicate.reads.iter().cloned());
                  }
              }
              if accepted {
                  let _profile = source_profile_scope_v1(SourceProfilePhaseV1::ScalarEvaluation);
                  let contribution = evaluate(plan.as_ref().map_or(value, |plan| plan.expression.as_ref()), configuration, &inputs,
                      EvaluationContextV1 { bindings: Some(&matched.bindings), scalar_memo: memo.as_ref(), ..query_context })?;
                  total += contribution.as_number().ok_or(ExecutableErrorV1::TypeMismatch)?;
                  if !total.is_finite() {
                      return Err(ExecutableErrorV1::NumericDomain);
                  }
              }
        }
    }
    let result = ExecutableValueV1::number(total)?;
    let query_reads = query_reads.into_inner();
    if let Some(reads) = context.reads {
        reads.borrow_mut().extend(query_reads.iter().cloned());
    }
    if let Some(queries) = context.sum_queries {
        let mut queries = queries.borrow_mut();
        let result = SumResult { inputs, result: result.clone(), reads: query_reads };
        if let Some(query) = queries.entries.iter_mut().find(|previous|
            same_query(previous)) {
            query.results.push(result);
        } else {
            queries.entries.push(SumQuery {
                shape: shape.cloned(), predicates: predicates.to_vec(), contribution: value.clone(),
                captured_reads: context.reads.is_some(), results: vec![result],
            });
        }
    }
    Ok(result)
}

pub(super) fn effect_scalar_plan(
    program: &Arc<ExecutableProgramV1>, coordinate: (usize, usize, usize),
    expression: &ExecutableExpressionV1, context: EvaluationContextV1,
) -> Result<Option<Arc<scalar_reuse::ScalarPlan>>, ExecutableErrorV1> {
    let Some(queries) = context.sum_queries else { return Ok(None) };
    let retained = queries.borrow().scalar_plans.clone();
    let mut effects = retained.1.lock().map_err(|_| ExecutableErrorV1::CarrierRejected)?;
    if effects.program.as_ref().is_none_or(|prior| !Arc::ptr_eq(prior, program)) {
        effects.plans.clear();
        effects.program = Some(program.clone());
    }
    if let Some(plan) = effects.plans.get(&coordinate) { return Ok(plan.clone()); }
    let plan = scalar_plan(expression, context)?;
    effects.plans.insert(coordinate, plan.clone());
    Ok(plan)
}

pub(super) fn scalar_plan(expression: &ExecutableExpressionV1, context: EvaluationContextV1)
    -> Result<Option<Arc<scalar_reuse::ScalarPlan>>, ExecutableErrorV1> {
    use ExecutableExpressionV1 as E;
    if context.reads.is_some() || matches!(expression, E::Constant(_) | E::Binding(_) | E::Argument(_) | E::Slot(_)) {
        return Ok(None);
    }
    let Some(queries) = context.sum_queries else { return Ok(None) };
    let _profile = source_profile_scope_v1(SourceProfilePhaseV1::ScalarPlanLookup);
    let queries = queries.borrow();
    let mut plans = queries.scalar_plans.0.lock().map_err(|_| ExecutableErrorV1::CarrierRejected)?;
    if let Some(existing) = plans.iter().find(|plan| plan.expression.as_ref() == expression) {
        return Ok(Some(existing.clone()));
    }
    let _profile = source_profile_scope_v1(SourceProfilePhaseV1::ScalarPlanBuild);
    let plan = Arc::new(scalar_reuse::ScalarPlan::new(expression)?);
    plans.push(plan.clone());
    Ok(Some(plan))
}

fn match_sum(
    predicates: &[ExecutableExpressionV1],
    configuration: &[ExecutableSlotV1],
    arguments: &[ExecutableValueV1],
    context: EvaluationContextV1,
    visits: &mut usize,
) -> Result<Vec<(Matched, bool)>, ExecutableErrorV1> {
    fn independent_pattern(pattern: &ExecutableExpressionV1) -> bool {
        match pattern {
            ExecutableExpressionV1::Constant(_) | ExecutableExpressionV1::Binding(_) => true,
            ExecutableExpressionV1::ReferentFacet { value, .. } => independent_pattern(value),
            _ => false,
        }
    }
    let capture = context.reads.is_some();
    let prefix_len = predicates.iter().take_while(|predicate| matches!(predicate,
        ExecutableExpressionV1::RelationMatch(_, subject, value)
            if independent_pattern(subject) && independent_pattern(value))).count();
    let Some(queries) = context.sum_queries.filter(|_| prefix_len > 0) else {
        return match_rule(predicates, configuration, arguments, context, visits, capture);
    };
    let (prefix, rest) = predicates.split_at(prefix_len);
    let cached = queries.borrow().prefixes.iter().find(|previous|
        previous.captured_reads == capture && previous.predicates == prefix)
        .map(|previous| {
            *visits = previous.visits;
            if let Some(reads) = context.reads {
                reads.borrow_mut().extend(previous.reads.iter().cloned());
            }
            previous.matches.clone()
        });
    let state = if let Some(cached) = cached { cached } else {
        let reads = std::cell::RefCell::new(Vec::new());
        let state = match_rule_from(prefix, configuration, arguments,
            EvaluationContextV1 { reads: context.reads.map(|_| &reads), ..context },
            visits, capture, MatchState::default())?;
        let reads = reads.into_inner();
        if let Some(destination) = context.reads {
            destination.borrow_mut().extend(reads.iter().cloned());
        }
        // Only argument-independent leading joins are shared, inside the same
        // immutable pre-state as sum results. Retain their logical work counts
        // and ordered rejection/read evidence so reuse cannot change limits.
        queries.borrow_mut().prefixes.push(SumPrefix {
            predicates: prefix.to_vec(), matches: state.clone(), visits: *visits,
            reads, captured_reads: capture,
        });
        state
    };
    Ok(match_rule_from(rest, configuration, arguments, context, visits, capture, state)?.finish())
}

fn unify(
    expression: &ExecutableExpressionV1,
    value: &ExecutableValueV1,
    bindings: &mut Arc<Bindings>,
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
        Arc::make_mut(bindings).insert(*binding, value.clone());
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
    bindings: &Bindings,
) -> Option<bool> {
    match expression {
        ExecutableExpressionV1::Binding(binding) => Some(bindings.contains_key(binding)),
        ExecutableExpressionV1::ReferentFacet { value, .. } => bound_pattern(value, bindings),
        _ => None,
    }
}

type ValueIndex<'a> = BTreeMap<
    &'a ExecutableValueV1,
    Vec<(&'a ExecutableReferentV1, &'a ExecutableValueV1)>,
>;

/// A physical specialization of exact value selection, not another relation.
/// The borrowed pre-state cannot change while the checked index is in use.
struct CheckedValueIndex<'a> {
    buckets: ValueIndex<'a>,
}

impl<'a> CheckedValueIndex<'a> {
    fn build(
        table: &'a ExecutableRelationTableV1,
        visits: &mut usize,
    ) -> Result<Self, ExecutableErrorV1> {
        let mut buckets = BTreeMap::<_, Vec<_>>::new();
        for (subject, values) in table.rows.iter() {
            for value in values {
                *visits = visits.checked_add(1).ok_or(ExecutableErrorV1::ResourceLimit)?;
                if *visits > MAX_JOIN_VISITS {
                    return Err(ExecutableErrorV1::ResourceLimit);
                }
                buckets.entry(value).or_default().push((subject, value));
            }
        }
        Self::check(table, buckets)
    }

    fn check(
        table: &'a ExecutableRelationTableV1,
        buckets: ValueIndex<'a>,
    ) -> Result<Self, ExecutableErrorV1> {
        let mut covered = 0usize;
        for (key, rows) in &buckets {
            if rows.is_empty() || rows.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(ExecutableErrorV1::MalformedProgram);
            }
            for (subject, value) in rows {
                if key != value || !table.rows.get(subject).is_some_and(|values| values.contains(value)) {
                    return Err(ExecutableErrorV1::MalformedProgram);
                }
                covered = covered.checked_add(1).ok_or(ExecutableErrorV1::ResourceLimit)?;
                if covered > MAX_JOIN_VISITS {
                    return Err(ExecutableErrorV1::ResourceLimit);
                }
            }
        }
        // Strict bucket ordering forbids duplicates; exact membership forbids
        // inventions; disjoint value keys plus equal cardinality prove coverage.
        if covered != table.rows.values().map(|values| values.len()).sum::<usize>() {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        Ok(Self { buckets })
    }

    fn select(&self, value: &ExecutableValueV1) -> impl Iterator<Item = (&'a ExecutableReferentV1, &'a ExecutableValueV1)> + '_ {
        self.buckets.get(value).into_iter().flatten().copied()
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
    capture: bool,
) -> Result<Vec<(Matched, bool)>, ExecutableErrorV1> {
    Ok(match_rule_from(predicates, configuration, arguments, context, visits,
        capture, MatchState::default())?.finish())
}

#[derive(Clone)]
struct MatchState {
    active: Vec<Matched>,
    rejected: Vec<(Matched, bool)>,
    rejected_count: usize,
}

impl Default for MatchState {
    fn default() -> Self {
        Self { active: vec![Matched::default()], rejected: Vec::new(), rejected_count: 0 }
    }
}

impl MatchState {
    fn finish(mut self) -> Vec<(Matched, bool)> {
        self.rejected.extend(self.active.into_iter().map(|matched| (matched, true)));
        self.rejected
    }
}

fn normalize_matches(matches: &mut Vec<Matched>) {
    // Stable order keeps the first derivation's evidence for equal substitutions.
    matches.sort_by(|a, b| a.bindings.cmp(&b.bindings));
    matches.dedup_by(|a, b| a.bindings == b.bindings);
}

fn match_rule_from(
    predicates: &[ExecutableExpressionV1],
    configuration: &[ExecutableSlotV1],
    arguments: &[ExecutableValueV1],
    context: EvaluationContextV1,
    visits: &mut usize,
    capture: bool,
    state: MatchState,
) -> Result<MatchState, ExecutableErrorV1> {
    let MatchState { mut active, mut rejected, mut rejected_count } = state;
    for predicate in predicates {
        if active.is_empty() { break; }
        if let ExecutableExpressionV1::RelationMatch(slot, subject_pattern, value_pattern) =
            predicate
        {
            let Some(ExecutableValueV1::RelationTable(table)) = configuration
                .get(usize::from(*slot))
                .and_then(ExecutableSlotV1::value)
            else {
                return Err(ExecutableErrorV1::TypeMismatch);
            };
            let mut next = Vec::new();
            let mut by_value = None::<CheckedValueIndex<'_>>;
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
                    dyn Iterator<Item = (&ExecutableReferentV1, &ExecutableValuesV1)> + '_,
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
                                by_value = Some(CheckedValueIndex::build(table, visits)?);
                            }
                            Box::new(by_value.as_ref().unwrap().select(value))
                        } else {
                            Box::new(first_row.into_iter().filter_map(|(subject, values)| values.get(value).map(|value| (subject, value))))
                        }
                    } else {
                        Box::new(first_row.into_iter().chain(rows).flat_map(|(subject, values)| values.iter().map(move |value| (subject, value))))
                    };
                let mut candidates = candidates.peekable();
                let mut incoming = Some(incoming);
                while let Some((subject, value)) = candidates.next() {
                        *visits = visits
                            .checked_add(1)
                            .ok_or(ExecutableErrorV1::ResourceLimit)?;
                        if *visits > MAX_JOIN_VISITS {
                            return Err(ExecutableErrorV1::ResourceLimit);
                        }
                        let last = candidates.peek().is_none();
                        let mut matched = if last { incoming.take().expect("last candidate owns its prefix") }
                            else { incoming.as_ref().expect("more candidates retain their prefix").clone() };
                        let new_bindings = [subject_pattern.as_ref(), value_pattern.as_ref()].map(|pattern| {
                            fn binding(pattern: &ExecutableExpressionV1) -> Option<u16> {
                                match pattern {
                                    ExecutableExpressionV1::Binding(id) => Some(*id),
                                    ExecutableExpressionV1::ReferentFacet { value, .. } => binding(value),
                                    _ => None,
                                }
                            }
                            binding(pattern).filter(|id| !matched.bindings.contains_key(id))
                        });
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
                            if last {
                                for binding in new_bindings.into_iter().flatten() { Arc::make_mut(&mut matched.bindings).remove(&binding); }
                                incoming = Some(matched);
                            }
                            continue;
                        }
                        found = true;
                        if capture { matched.predicates.push(EvaluatedValue {
                            value: ExecutableValueV1::Boolean(true),
                            reads: vec![ExecutableReadV1::RelationRow(
                                *slot,
                                subject.clone(),
                                value.clone(),
                            )],
                        }); }
                        next.push(matched);
                        if next.len() > MAX_MATCHES {
                            normalize_matches(&mut next);
                            if next.len() > MAX_MATCHES {
                                return Err(ExecutableErrorV1::ResourceLimit);
                            }
                        }
                }
                if !found {
                    let mut incoming = incoming.expect("failed candidates retain their original prefix");
                    if capture { incoming.predicates.push(EvaluatedValue {
                        value: ExecutableValueV1::Boolean(false),
                        reads: vec![ExecutableReadV1::RelationSearch(
                            *slot,
                            bound_subject
                                .as_ref()
                                .and_then(ExecutableValueV1::as_referent)
                                .cloned(),
                            *visits - start_visits,
                        )],
                    }); }
                    rejected_count += 1;
                    if capture { rejected.push((incoming, false)); }
                    if rejected_count > MAX_MATCHES {
                        return Err(ExecutableErrorV1::ResourceLimit);
                    }
                }
            }
            normalize_matches(&mut next);
            active = next;
        } else {
            let plan = if capture { None } else { scalar_plan(predicate, context)? };
            let memo = plan.as_ref().map(|plan| plan.memo());
            let _profile = source_profile_scope_v1(SourceProfilePhaseV1::ScalarEvaluation);
            let mut next = Vec::new();
            for mut matched in active {
                let evaluated = evaluate_for_trace(
                    plan.as_ref().map_or(predicate, |plan| plan.expression.as_ref()),
                    configuration,
                    arguments,
                    EvaluationContextV1 {
                        bindings: Some(&matched.bindings),
                        scalar_memo: memo.as_ref(),
                        ..context
                    },
                    capture,
                )?;
                let accepted = boolean(evaluated.value.clone())?;
                if capture { matched.predicates.push(evaluated); }
                if accepted {
                    next.push(matched);
                } else {
                    rejected_count += 1;
                    if capture { rejected.push((matched, false)); }
                    if rejected_count > MAX_MATCHES {
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
    Ok(MatchState { active, rejected, rejected_count })
}

pub(super) struct LazyOccurrenceIdentity<'a> {
    context: EvaluationContextV1<'a>,
    rule: usize,
    bindings: &'a Bindings,
    identity: std::cell::Cell<Option<[u8; IDENTITY_BYTES]>>,
}

impl<'a> LazyOccurrenceIdentity<'a> {
    pub(super) fn new(context: EvaluationContextV1<'a>, rule: usize,
        bindings: &'a Bindings) -> Self {
        Self { context, rule, bindings, identity: std::cell::Cell::new(None) }
    }

    pub(super) fn get(&self) -> Result<[u8; IDENTITY_BYTES], ExecutableErrorV1> {
        if let Some(identity) = self.identity.get() { return Ok(identity); }
        let identity = occurrence_identity(self.context, self.rule, self.bindings)?;
        self.identity.set(Some(identity));
        Ok(identity)
    }
}

pub(super) fn occurrence_identity(
    context: EvaluationContextV1,
    rule: usize,
    bindings: &Bindings,
) -> Result<[u8; IDENTITY_BYTES], ExecutableErrorV1> {
    let _profile = source_profile_scope_v1(SourceProfilePhaseV1::OccurrenceIdentity);
    let mut bytes = Vec::new();
    for (binding, value) in bindings.iter() {
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
            let ExecutableSlotV1::Present(ExecutableValueV1::RelationTable(table)) = &mut next[usize::from(slot)]
            else {
                return Err(ExecutableErrorV1::TypeMismatch);
            };
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
                table.put(&subject, ExecutableValueV1::number(value)?)?;
            } else {
                for (mode, value) in effects {
                    match mode {
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
        }
        Ok(())
    }
}

#[cfg(test)]
mod ordered_specialization_tests {
    use super::*;

    fn table() -> ExecutableRelationTableV1 {
        let n = |v| ExecutableValueV1::number(v).unwrap();
        ExecutableRelationTableV1 {
            subject_domain: 7,
            value_kind: ExecutableRelationValueKindV1::Number,
            value_domain: None,
            cardinality: ExecutableRelationCardinalityV1::Many,
            total: false,
            rows: Arc::new(BTreeMap::from([
                (ExecutableReferentV1::declared(7, 9), BTreeSet::from([n(4.0), n(9.0)]).into()),
                (ExecutableReferentV1::created(7, [1; IDENTITY_BYTES]), BTreeSet::from([n(4.0)]).into()),
                (ExecutableReferentV1::created(7, [2; IDENTITY_BYTES]), BTreeSet::from([n(4.0)]).into()),
            ]).into()),
        }
    }

    #[test]
    fn checked_index_refines_exact_ordered_selection_and_preserves_limits() {
        let table = table();
        let mut visits = 0;
        let index = CheckedValueIndex::build(&table, &mut visits).unwrap();
        assert_eq!(visits, 4);
        for value in [0.0, 4.0, 9.0] {
            let value = ExecutableValueV1::number(value).unwrap();
            let expected = table.rows.iter().flat_map(|(subject, values)| {
                values.iter().filter(|candidate| **candidate == value).map(move |value| (subject, value))
            }).collect::<Vec<_>>();
            assert_eq!(index.select(&value).collect::<Vec<_>>(), expected);
        }
        let mut visits = MAX_JOIN_VISITS - 4;
        assert!(CheckedValueIndex::build(&table, &mut visits).is_ok());
        assert_eq!(visits, MAX_JOIN_VISITS);
        let mut visits = MAX_JOIN_VISITS - 3;
        assert!(matches!(CheckedValueIndex::build(&table, &mut visits), Err(ExecutableErrorV1::ResourceLimit)));
        assert_eq!(visits, MAX_JOIN_VISITS + 1);
        let mut empty = table.clone();
        Arc::make_mut(&mut empty.rows).clear();
        assert!(CheckedValueIndex::build(&empty, &mut 0).unwrap().buckets.is_empty());
    }

    #[test]
    fn specialization_checker_rejects_missing_duplicate_reordered_and_invented_rows() {
        let table = table();
        let index = CheckedValueIndex::build(&table, &mut 0).unwrap();
        let value = ExecutableValueV1::number(4.0).unwrap();
        for mutation in 0..5 {
            let mut buckets = index.buckets.clone();
            let rows = buckets.get_mut(&value).unwrap();
            match mutation {
                0 => { rows.pop(); }
                1 => { rows.push(rows[0]); }
                2 => rows.swap(0, 1),
                3 => { rows[1] = rows[0]; }
                4 => { buckets.insert(&value, vec![]); }
                _ => unreachable!(),
            }
            assert!(CheckedValueIndex::check(&table, buckets).is_err(), "mutation {mutation}");
        }
        let foreign = ExecutableReferentV1::created(7, [3; IDENTITY_BYTES]);
        let mut buckets = index.buckets.clone();
        buckets.get_mut(&value).unwrap()[2].0 = &foreign;
        assert!(CheckedValueIndex::check(&table, buckets).is_err());
        let wrong = ExecutableValueV1::number(16.0).unwrap();
        let mut buckets = index.buckets.clone();
        let rows = buckets.remove(&value).unwrap();
        buckets.insert(&wrong, rows);
        assert!(CheckedValueIndex::check(&table, buckets).is_err());
    }
}

#[cfg(test)]
mod sum_reuse_tests {
    use super::*;

    #[test]
    fn shared_join_prefix_preserves_input_filtering_sum_order_and_reads() {
        use ExecutableExpressionV1 as E;
        let number = |n| ExecutableValueV1::number(n).unwrap();
        let table = |values: &[f64]| ExecutableValueV1::RelationTable(ExecutableRelationTableV1 {
            subject_domain: 7, value_kind: ExecutableRelationValueKindV1::Number,
            value_domain: None, cardinality: ExecutableRelationCardinalityV1::One, total: false,
            rows: Arc::new(values.iter().enumerate().map(|(id, value)|
                (ExecutableReferentV1::declared(7, id as u32), [number(*value)].into()))
                .collect::<BTreeMap<_, _>>().into()),
        });
        let configuration = [table(&[1e16, 1.0, -1e16, 7.0]).into(), table(&[1.0; 3]).into()];
        let query = E::Sum {
            inputs: vec![E::Argument(0)],
            predicates: vec![
                E::RelationMatch(0, Box::new(E::Binding(0)), Box::new(E::Binding(1))),
                E::RelationMatch(1, Box::new(E::Binding(0)), Box::new(E::Constant(number(1.0)))),
                E::LessThanOrEqual(Box::new(E::Binding(1)), Box::new(E::Argument(0))),
            ],
            value: Box::new(E::Binding(1)),
        };
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES],
            step_ordinal: 0, reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        let queries = std::cell::RefCell::new(SumQueries::default());
        let shared = EvaluationContextV1 { sum_queries: Some(&queries), ..context };
        for (input, result) in [(1e16, 0.0), (0.0, -1e16), (1e16, 0.0)] {
            let arguments = [number(input)];
            let expected = evaluate_with_reads(&query, &configuration, &arguments, context).unwrap();
            assert_eq!(expected.value, number(result));
            assert_eq!(evaluate(&query, &configuration, &arguments, shared).unwrap(), expected.value);
            let actual = evaluate_with_reads(&query, &configuration, &arguments, shared).unwrap();
            assert_eq!(actual.value, expected.value);
            assert_eq!(actual.reads, expected.reads);
            assert!(actual.reads.iter().any(|read| matches!(read, ExecutableReadV1::RelationSearch(..))));
        }
        assert_eq!(queries.borrow().prefixes.len(), 2, "one join prefix per trace mode");
        let changed = [table(&[2.0, 3.0]).into(), table(&[1.0; 3]).into()];
        let next_queries = std::cell::RefCell::new(SumQueries::default());
        assert_eq!(evaluate(&query, &changed, &[number(1e16)],
            EvaluationContextV1 { sum_queries: Some(&next_queries), ..context }).unwrap(), number(5.0));
    }

    #[test]
    fn shared_join_prefix_preserves_rejection_limits_and_empty_short_circuit() {
        use ExecutableExpressionV1 as E;
        let table = |count| ExecutableValueV1::RelationTable(ExecutableRelationTableV1 {
            subject_domain: 7, value_kind: ExecutableRelationValueKindV1::Referent,
            value_domain: Some(7), cardinality: ExecutableRelationCardinalityV1::One, total: false,
            rows: Arc::new((0..count).map(|id| {
                let subject = ExecutableReferentV1::declared(7, id as u32);
                (subject.clone(), [ExecutableValueV1::Referent(subject)].into())
            }).collect::<BTreeMap<_, _>>().into()),
        });
        let configuration = [table(MAX_MATCHES).into(), table(1).into()];
        let same = |slot, binding| E::RelationMatch(slot,
            Box::new(E::Binding(binding)), Box::new(E::Binding(binding)));
        let query = E::Sum {
            inputs: vec![E::Argument(0)],
            predicates: vec![same(0, 0), same(1, 0), E::Argument(0), same(0, 1),
                E::Constant(ExecutableValueV1::Boolean(false))],
            value: Box::new(E::Constant(ExecutableValueV1::number(1.0).unwrap())),
        };
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES],
            step_ordinal: 0, reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        for capture in [false, true] {
            let queries = std::cell::RefCell::new(SumQueries::default());
            let shared = EvaluationContextV1 { sum_queries: Some(&queries), ..context };
            for input in [true, false, true] {
                let arguments = [ExecutableValueV1::Boolean(input)];
                let expected = evaluate_for_trace(&query, &configuration, &arguments, context, capture);
                let actual = evaluate_for_trace(&query, &configuration, &arguments, shared, capture);
                if input {
                    assert!(matches!(expected, Err(ExecutableErrorV1::ResourceLimit)));
                    assert!(matches!(actual, Err(ExecutableErrorV1::ResourceLimit)));
                } else {
                    let expected = expected.unwrap();
                    let actual = actual.unwrap();
                    assert_eq!(actual.value, expected.value);
                    assert_eq!(actual.reads, expected.reads);
                }
            }
        }
        let empty = [table(0).into()];
        let queries = std::cell::RefCell::new(SumQueries::default());
        for input in [true, false] {
            assert_eq!(evaluate(&query, &empty, &[ExecutableValueV1::Boolean(input)],
                EvaluationContextV1 { sum_queries: Some(&queries), ..context }).unwrap(),
                ExecutableValueV1::number(0.0).unwrap());
        }
    }

    #[test]
    fn preparation_effects_share_queries_with_exact_reads_and_independent_errors() {
        use ExecutableExpressionV1 as E;
        let number = |n| ExecutableValueV1::number(n).unwrap();
        let configuration = vec![ExecutableValueV1::RelationTable(ExecutableRelationTableV1 {
            subject_domain: 7, value_kind: ExecutableRelationValueKindV1::Number,
            value_domain: None, cardinality: ExecutableRelationCardinalityV1::One,
            total: false,
            rows: Arc::new((0..3).map(|id| (ExecutableReferentV1::declared(7, id),
                BTreeSet::from([number(1.0)]).into())).collect::<BTreeMap<_, _>>().into()),
        }).into()];
        let query = E::Sum {
            inputs: vec![E::Argument(0)],
            predicates: vec![E::RelationMatch(0, Box::new(E::Binding(0)), Box::new(E::Argument(0)))],
            value: Box::new(E::Constant(number(1.0))),
        };
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES],
            step_ordinal: 0, reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
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
                assert_eq!(queries.borrow().entries.iter().map(|query| query.results.len()).sum::<usize>(), 2);
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
            rows: Arc::new((0..100).map(|id| (ExecutableReferentV1::declared(7, id),
                BTreeSet::from([number(1.0)]).into())).collect::<BTreeMap<_, _>>().into()),
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
            step_ordinal: 0, reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        let expected = evaluate_with_reads(&sum, &configuration, &[], context).unwrap();
        let queries = std::cell::RefCell::new(SumQueries::default());
        let shared = EvaluationContextV1 { sum_queries: Some(&queries), ..context };
        for _ in 0..2 {
            assert_eq!(evaluate(&sum, &configuration, &[], shared).unwrap(), expected.value);
            assert_eq!(queries.borrow().entries.len(), 1);
            assert!(queries.borrow().entries[0].results[0].reads.is_empty());
        }
        let traced = evaluate_with_reads(&sum, &configuration, &[], shared).unwrap();
        assert_eq!(traced.value, expected.value);
        assert_eq!(traced.reads, expected.reads);
        assert!(!traced.reads.is_empty());
        assert_eq!(queries.borrow().entries.len(), 2);

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
            rows: Arc::default(),
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

#[cfg(test)]
mod match_ownership_tests {
    use super::*;

    #[test]
    fn contiguous_bindings_preserve_sparse_map_order_replacement_and_removal() {
        let n = |value| ExecutableValueV1::number(value).unwrap();
        let mut cases = Vec::new();
        for entries in [vec![], vec![(7, n(2.0)), (1, n(4.0))],
            vec![(1, n(4.0))], vec![(7, n(2.0)), (1, n(4.0)), (7, n(-1.0))]] {
            let mut actual = Bindings::default();
            let mut expected = BTreeMap::new();
            for (binding, value) in entries {
                assert_eq!(actual.insert(binding, value.clone()), expected.insert(binding, value));
            }
            assert_eq!(actual.iter().collect::<Vec<_>>(), expected.iter().collect::<Vec<_>>());
            for binding in [0, 1, 7, 128] { assert_eq!(actual.get(&binding), expected.get(&binding)); }
            cases.push((actual.clone(), expected.clone()));
            assert_eq!(actual.remove(&7), expected.remove(&7));
            assert_eq!(actual.remove(&7), expected.remove(&7));
            assert_eq!(actual.iter().collect::<Vec<_>>(), expected.iter().collect::<Vec<_>>());
        }
        for (left, left_map) in &cases {
            for (right, right_map) in &cases { assert_eq!(left.cmp(right), left_map.cmp(right_map)); }
        }
    }

    #[test]
    fn relational_identity_is_derived_only_for_evaluated_fresh_referents() {
        use ExecutableExpressionV1 as E;
        let bindings = Bindings::from([(3, ExecutableValueV1::text("bound text").unwrap())]);
        let context = EvaluationContextV1 { allocation_root: [17; IDENTITY_BYTES],
            step_ordinal: 19, reads: None, sum_queries: None, scalar_memo: None, bindings: Some(&bindings), relational_occurrence: None };
        let identity = LazyOccurrenceIdentity::new(context, 23, &bindings);
        let evaluation = EvaluationContextV1 { relational_occurrence: Some(&identity), ..context };
        let fresh = E::FreshReferent { domain: 29, binder: 31 };
        let ordinary = E::Conditional(Box::new(E::Constant(ExecutableValueV1::Boolean(false))),
            Box::new(fresh.clone()), Box::new(E::Binding(3)));
        assert_eq!(&evaluate(&ordinary, &[], &[], evaluation).unwrap(), bindings.get(&3).unwrap());
        assert_eq!(identity.identity.get(), None);
        let mut preimage = 3u16.to_le_bytes().to_vec();
        encode_value(&mut preimage, bindings.get(&3).unwrap()).unwrap();
        let expected_match = runtime_domain_hash("clause/relational-match/v1", &[
            &[17; IDENTITY_BYTES], &19u64.to_be_bytes(), &23u64.to_be_bytes(), &preimage]);
        let expected = ExecutableValueV1::Referent(ExecutableReferentV1::created(29,
            runtime_domain_hash("clause/runtime-referent/v1", &[
                &expected_match, &19u64.to_be_bytes(), &29u32.to_be_bytes(), &31u16.to_be_bytes()])));
        assert_eq!(evaluate(&fresh, &[], &[], evaluation).unwrap(), expected);
        assert_eq!(identity.identity.get(), Some(expected_match));
        assert_eq!(evaluate(&fresh, &[], &[], evaluation).unwrap(), expected);
        let later = LazyOccurrenceIdentity::new(EvaluationContextV1 { step_ordinal: 20, ..context }, 23, &bindings);
        assert_ne!(later.get().unwrap(), expected_match);
        assert_ne!(evaluate(&fresh, &[], &[], context).unwrap(), expected);
    }

    #[test]
    fn untraced_matches_preserve_rejection_and_visit_limits() {
        use ExecutableExpressionV1 as E;
        let table = ExecutableRelationTableV1 {
            subject_domain: 7, value_kind: ExecutableRelationValueKindV1::Referent,
            value_domain: Some(7), cardinality: ExecutableRelationCardinalityV1::Many,
            total: false, rows: Arc::new((0..MAX_MATCHES).map(|index| {
                let subject = ExecutableReferentV1::declared(7, index as u32);
                (subject.clone(), BTreeSet::from([ExecutableValueV1::Referent(subject)]).into())
            }).collect::<BTreeMap<_, _>>().into()),
        };
        let configuration = [ExecutableValueV1::RelationTable(table).into()];
        let predicates = [
            E::RelationMatch(0, Box::new(E::Binding(0)), Box::new(E::Binding(0))),
            E::Equal(Box::new(E::Binding(0)), Box::new(E::Constant(
                ExecutableValueV1::Referent(ExecutableReferentV1::declared(7, 0))))),
            E::RelationMatch(0, Box::new(E::Binding(1)), Box::new(E::Binding(1))),
            E::Constant(ExecutableValueV1::Boolean(false)),
        ];
        let context = EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES],
            step_ordinal: 0, reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None };
        for capture in [false, true] {
            let mut visits = 0;
            assert!(matches!(match_rule(&predicates, &configuration, &[], context,
                &mut visits, capture), Err(ExecutableErrorV1::ResourceLimit)));
            assert_eq!(visits, MAX_MATCHES * 2);
            let mut visits = MAX_JOIN_VISITS;
            assert!(matches!(match_rule(&predicates[..1], &configuration, &[], context,
                &mut visits, capture), Err(ExecutableErrorV1::ResourceLimit)));
        }
    }

    #[test]
    fn a_failed_final_candidate_restores_the_original_bindings() {
        let subject = ExecutableReferentV1::declared(7, 1);
        let value = ExecutableValueV1::Referent(ExecutableReferentV1::declared(7, 2));
        let table = ExecutableRelationTableV1 {
            subject_domain: 7, value_kind: ExecutableRelationValueKindV1::Referent,
            value_domain: Some(7), cardinality: ExecutableRelationCardinalityV1::Many,
            total: false, rows: Arc::new(BTreeMap::from([(subject, BTreeSet::from([value]).into())]).into()),
        };
        let predicate = ExecutableExpressionV1::RelationMatch(0,
            Box::new(ExecutableExpressionV1::Binding(0)),
            Box::new(ExecutableExpressionV1::Binding(0)));
        let results = match_rule(&[predicate], &[ExecutableValueV1::RelationTable(table).into()], &[],
            EvaluationContextV1 { allocation_root: [0; IDENTITY_BYTES], step_ordinal: 0,
                reads: None, sum_queries: None, scalar_memo: None, bindings: None, relational_occurrence: None }, &mut 0, true).unwrap();
        assert_eq!(results.len(), 1);
        let (matched, accepted) = &results[0];
        assert!(!accepted);
        assert!(matched.bindings.is_empty());
        assert_eq!(matched.predicates.len(), 1);
        assert_eq!(matched.predicates[0].value, ExecutableValueV1::Boolean(false));
        assert!(matches!(matched.predicates[0].reads.as_slice(), [ExecutableReadV1::RelationSearch(0, None, 1)]));
    }
}
