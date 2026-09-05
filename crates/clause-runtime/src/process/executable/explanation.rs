//! Bounded execution evidence and isolated finite intervention over the very
//! same normalized evaluator that commits actual runtime Steps.
use super::*;

const MAX_RECORDED_ENTRIES: usize = 64;
const MAX_TRACE_RULES: usize = 4096;
const MAX_INTERVENTION_CHOICES: usize = 20;
const MAX_INTERVENTION_EVALUATIONS: u32 = 4096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutableReadV1 {
    State(u16, ExecutableValueV1),
    Argument(u16, ExecutableValueV1),
    Binding(u16, ExecutableValueV1),
    RelationRow(u16, ExecutableReferentV1, ExecutableValueV1),
    RelationSearch(u16, Option<ExecutableReferentV1>, usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableEvaluatedExpressionV1 {
    pub expression: ExecutableExpressionV1,
    pub value: ExecutableValueV1,
    pub reads: Vec<ExecutableReadV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableEffectEvaluationV1 {
    pub slot: u16,
    pub additive: bool,
    pub subject: Option<ExecutableReferentV1>,
    pub evaluated: Option<ExecutableEvaluatedExpressionV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableRuleEvaluationV1 {
    pub rule: u16,
    pub bindings: BTreeMap<u16, ExecutableValueV1>,
    pub required_present: Vec<(u16, bool)>,
    pub required_absent: Vec<(u16, bool)>,
    /// Evaluated prefix only. Short-circuited predicates are never claimed read.
    pub predicates: Vec<ExecutableEvaluatedExpressionV1>,
    pub selected: bool,
    pub effects: Vec<ExecutableEffectEvaluationV1>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExecutableEvaluationTraceV1 {
    pub rules: Vec<ExecutableRuleEvaluationV1>,
    pub truncated: bool,
}

impl ExecutableEvaluationTraceV1 {
    pub(super) fn push(&mut self, rule: ExecutableRuleEvaluationV1) {
        if self.rules.len() < MAX_TRACE_RULES {
            self.rules.push(rule);
        } else {
            self.truncated = true;
        }
    }
    pub(super) fn effect(
        &mut self,
        rule: usize,
        slot: u16,
        additive: bool,
        subject: Option<ExecutableReferentV1>,
        evaluated: Option<ExecutableEvaluatedExpressionV1>,
    ) {
        if let Some(rule) = self.rules.get_mut(rule) {
            rule.effects.push(ExecutableEffectEvaluationV1 {
                slot,
                additive,
                subject,
                evaluated,
            });
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableRecordedEventV1 {
    pub step: ExecutableStepV1,
    pub physical_plan: ExecutablePhysicalPlanIdV1,
    pub before: Vec<ExecutableSlotV1>,
    pub after: Vec<ExecutableSlotV1>,
    pub trace: ExecutableEvaluationTraceV1,
    step_ordinal: u64,
    configuration_ordinal: u64,
}

/// Allowed changes are an explicit finite set of typed alternatives to exact
/// pre-state coordinates. Cost is changed coordinate count. Equal-cost sets
/// are ordered by canonical (slot, subject, typed value); one value per coordinate.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutableInterventionChangeV1 {
    pub slot: u16,
    pub subject: Option<ExecutableReferentV1>,
    pub value: ExecutableValueV1,
}

impl ExecutableInterventionChangeV1 {
    fn coordinate_key(&self) -> String {
        match &self.subject {
            None => self.slot.to_string(),
            Some(subject) => {
                let mut bytes = Vec::new();
                encode_referent(&mut bytes, subject);
                format!("{}:{}", self.slot, hex_identity(&bytes))
            }
        }
    }

    fn before_value(
        &self,
        configuration: &[ExecutableSlotV1],
    ) -> Result<Option<ExecutableValueV1>, ExecutableErrorV1> {
        let state = configuration
            .get(usize::from(self.slot))
            .ok_or(ExecutableErrorV1::UnknownSlot(self.slot))?;
        if let Some(subject) = &self.subject {
            let Some(ExecutableValueV1::RelationTable(table)) = state.value() else {
                return Err(ExecutableErrorV1::TypeMismatch);
            };
            if table.cardinality == ExecutableRelationCardinalityV1::Many
                || !table.value_matches(&self.value)
            {
                return Err(ExecutableErrorV1::TypeMismatch);
            }
            return table
                .read(&ExecutableValueV1::Referent(subject.clone()))
                .map(Some);
        }
        if state.kind() != self.value.kind() {
            return Err(ExecutableErrorV1::TypeMismatch);
        }
        if let (Some(ExecutableValueV1::Referent(old)), ExecutableValueV1::Referent(new)) =
            (state.value(), &self.value)
        {
            if old.domain != new.domain {
                return Err(ExecutableErrorV1::TypeMismatch);
            }
        }
        Ok(state.value().cloned())
    }

    fn apply(&self, configuration: &mut [ExecutableSlotV1]) -> Result<(), ExecutableErrorV1> {
        let state = &mut configuration[usize::from(self.slot)];
        *state = if let Some(subject) = &self.subject {
            let Some(ExecutableValueV1::RelationTable(table)) = state.value() else {
                return Err(ExecutableErrorV1::TypeMismatch);
            };
            ExecutableValueV1::RelationTable(table.put(
                &ExecutableValueV1::Referent(subject.clone()),
                self.value.clone(),
            )?)
            .into()
        } else {
            self.value.clone().into()
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableInterventionQueryV1 {
    pub event: StepId,
    pub allowed: Vec<ExecutableInterventionChangeV1>,
    pub desired: ExecutableExpressionV1,
    pub maximum_evaluations: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableInterventionResultV1 {
    pub event: StepId,
    pub evaluations: u32,
    /// False means only a searched prefix, never proof of impossibility.
    pub completed: bool,
    pub exhausted: bool,
    /// First satisfying set is minimal under the declared enumeration order.
    pub solution: Option<Vec<ExecutableInterventionChangeV1>>,
    pub predicted: Option<Vec<ExecutableSlotV1>>,
}

pub fn encode_executable_intervention_query_v1(
    query: &ExecutableInterventionQueryV1,
) -> Result<Vec<u8>, ExecutableErrorV1> {
    let rows = query.allowed.iter().any(|change| change.subject.is_some());
    let mut bytes = if rows { b"CIQ2" } else { b"CIQ1" }.to_vec();
    bytes.extend_from_slice(query.event.as_bytes());
    bytes.extend_from_slice(&query.maximum_evaluations.to_le_bytes());
    encode_count(&mut bytes, query.allowed.len())?;
    for change in &query.allowed {
        bytes.extend_from_slice(&change.slot.to_le_bytes());
        if rows {
            bytes.push(u8::from(change.subject.is_some()));
            if let Some(subject) = &change.subject {
                encode_referent(&mut bytes, subject);
            }
        }
        encode_value(&mut bytes, &change.value)?;
    }
    validate_value_expression(&query.desired, 0)?;
    encode_expression(&mut bytes, &query.desired)?;
    if bytes.len() > 64 * 1024 {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    Ok(bytes)
}

pub fn decode_executable_intervention_query_v1(
    bytes: &[u8],
) -> Result<ExecutableInterventionQueryV1, ExecutableErrorV1> {
    if bytes.len() > 64 * 1024 {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    let mut d = Decoder::new(bytes);
    let rows = match d.take(4)? {
        b"CIQ1" => false,
        b"CIQ2" => true,
        _ => return Err(ExecutableErrorV1::MalformedProgram),
    };
    let event = StepId::from_bytes(d.identity()?);
    let maximum_evaluations = d.u32()?;
    let count = d.count()?;
    if count > MAX_INTERVENTION_CHOICES || maximum_evaluations > MAX_INTERVENTION_EVALUATIONS {
        return Err(ExecutableErrorV1::ResourceLimit);
    }
    let allowed = (0..count)
        .map(|_| {
            Ok(ExecutableInterventionChangeV1 {
                slot: d.u16()?,
                subject: if rows {
                    match d.byte()? {
                        0 => None,
                        1 => Some(d.referent()?),
                        _ => return Err(ExecutableErrorV1::MalformedProgram),
                    }
                } else {
                    None
                },
                value: d.value()?,
            })
        })
        .collect::<Result<_, ExecutableErrorV1>>()?;
    let desired = d.expression(0)?;
    if !d.is_complete() {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
    Ok(ExecutableInterventionQueryV1 {
        event,
        maximum_evaluations,
        allowed,
        desired,
    })
}

pub fn executable_intervention_result_term_v1(
    scope: TermScope,
    result: &ExecutableInterventionResultV1,
) -> Result<Term, ExecutableErrorV1> {
    let boolean = |value| projected_scalar_value_term(scope, &ExecutableValueV1::Boolean(value));
    let mut fields = vec![
        (
            b"event".to_vec(),
            diagnostic_text(scope, &hex_identity(result.event.as_bytes()))?,
        ),
        (
            b"evaluations".to_vec(),
            diagnostic_number(scope, result.evaluations as f64)?,
        ),
        (b"completed".to_vec(), boolean(result.completed)?),
        (b"exhausted".to_vec(), boolean(result.exhausted)?),
        (b"found".to_vec(), boolean(result.solution.is_some())?),
        (
            b"cost-order".to_vec(),
            diagnostic_text(
                scope,
                "changed-coordinate-count; then (slot, typed-subject, canonical-typed-value) lexicographic",
            )?,
        ),
    ];
    if let Some(solution) = &result.solution {
        fields.push((
            b"cost".to_vec(),
            diagnostic_number(scope, solution.len() as f64)?,
        ));
        fields.push((
            b"solution".to_vec(),
            projection_object(
                scope,
                solution
                    .iter()
                    .map(|change| {
                        Ok((
                            change.coordinate_key().into_bytes(),
                            projected_value_term(scope, change.value.clone())?,
                        ))
                    })
                    .collect::<Result<_, ExecutableErrorV1>>()?,
            )?,
        ));
    }
    if let Some(predicted) = &result.predicted {
        fields.push((
            b"predicted".to_vec(),
            diagnostic_index(
                scope,
                predicted
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, value)| {
                        value.value().map(|value| {
                            Ok((
                                slot.to_string().into_bytes(),
                                projected_value_term(scope, value.clone())?,
                            ))
                        })
                    })
                    .collect::<Result<_, ExecutableErrorV1>>()?,
            )?,
        ));
    }
    projection_object(scope, fields)
}

pub(super) fn evaluate_explained(
    expression: &ExecutableExpressionV1,
    configuration: &[ExecutableSlotV1],
    arguments: &[ExecutableValueV1],
    context: EvaluationContextV1,
) -> Result<ExecutableEvaluatedExpressionV1, ExecutableErrorV1> {
    let reads = std::cell::RefCell::new(Vec::new());
    let value = evaluate(
        expression,
        configuration,
        arguments,
        EvaluationContextV1 {
            reads: Some(&reads),
            ..context
        },
    )?;
    Ok(ExecutableEvaluatedExpressionV1 {
        expression: expression.clone(),
        value,
        reads: reads.into_inner(),
    })
}

impl ExecutableProcessRuntimeV1 {
    pub fn explanation_term(&self, entry: u16) -> Result<Term, ExecutableErrorV1> {
        let recorded = self
            .recorded_event(entry)
            .ok_or(ExecutableErrorV1::NoStep)?;
        let scope = TermScope {
            universe: self.carrier.carrier().constitution().universe(),
            semantics: self.carrier.carrier().constitution().semantics(),
        };
        let boolean =
            |value| projected_scalar_value_term(scope, &ExecutableValueV1::Boolean(value));
        let metadata = self.source_metadata.as_ref();
        let mut fields = vec![
            (
                b"step".to_vec(),
                diagnostic_text(scope, &hex_identity(recorded.step.id.as_bytes()))?,
            ),
            (
                b"physical-plan".to_vec(),
                diagnostic_text(scope, &hex_identity(recorded.physical_plan.as_bytes()))?,
            ),
            (b"entry".to_vec(), diagnostic_number(scope, entry as f64)?),
            (b"truncated".to_vec(), boolean(recorded.trace.truncated)?),
            (
                b"rule-applied".to_vec(),
                boolean(recorded.step.rule_applied)?,
            ),
        ];
        if let Some(metadata) = metadata {
            for name in [b"artifact".as_slice(), b"snapshot".as_slice()] {
                if let Some(value) = diagnostic_field(metadata, name) {
                    fields.push((name.to_vec(), value.clone()));
                }
            }
        }
        let mut used_slots = BTreeSet::new();
        let expression = |evaluated: &ExecutableEvaluatedExpressionV1,
                          used: &mut BTreeSet<u16>|
         -> Result<Term, ExecutableErrorV1> {
            let reads = evaluated
                .reads
                .iter()
                .enumerate()
                .map(|(index, read)| {
                    if let ExecutableReadV1::RelationSearch(slot, subject, visits) = read {
                        used.insert(*slot);
                        let mut fields = vec![
                            (
                                b"kind".to_vec(),
                                diagnostic_text(scope, "complete-relation-search")?,
                            ),
                            (
                                b"coordinate".to_vec(),
                                diagnostic_number(scope, *slot as f64)?,
                            ),
                            (
                                b"visited".to_vec(),
                                diagnostic_number(scope, *visits as f64)?,
                            ),
                        ];
                        if let Some(subject) = subject {
                            fields.push((
                                b"subject".to_vec(),
                                projected_value_term(
                                    scope,
                                    ExecutableValueV1::Referent(subject.clone()),
                                )?,
                            ));
                        }
                        return Ok((
                            index.to_string().into_bytes(),
                            projection_object(scope, fields)?,
                        ));
                    }
                    let (kind, coordinate, value) = match read {
                        ExecutableReadV1::State(slot, value) => {
                            used.insert(*slot);
                            ("state", *slot, value)
                        }
                        ExecutableReadV1::Argument(argument, value) => {
                            ("argument", *argument, value)
                        }
                        ExecutableReadV1::Binding(binding, value) => ("binding", *binding, value),
                        ExecutableReadV1::RelationRow(slot, _, value) => {
                            used.insert(*slot);
                            ("relation-row", *slot, value)
                        }
                        ExecutableReadV1::RelationSearch(..) => unreachable!(),
                    };
                    let mut fields = vec![
                        (b"kind".to_vec(), diagnostic_text(scope, kind)?),
                        (
                            b"coordinate".to_vec(),
                            diagnostic_number(scope, coordinate as f64)?,
                        ),
                        (
                            b"value".to_vec(),
                            projected_value_term(scope, value.clone())?,
                        ),
                    ];
                    if let ExecutableReadV1::RelationRow(_, subject, _) = read {
                        fields.push((
                            b"subject".to_vec(),
                            projected_value_term(
                                scope,
                                ExecutableValueV1::Referent(subject.clone()),
                            )?,
                        ));
                    }
                    Ok((
                        index.to_string().into_bytes(),
                        projection_object(scope, fields)?,
                    ))
                })
                .collect::<Result<_, ExecutableErrorV1>>()?;
            projection_object(
                scope,
                vec![
                    (
                        b"expression".to_vec(),
                        diagnostic_text(scope, &format!("{:?}", evaluated.expression))?,
                    ),
                    (
                        b"value".to_vec(),
                        projected_value_term(scope, evaluated.value.clone())?,
                    ),
                    (b"reads".to_vec(), projection_object(scope, reads)?),
                ],
            )
        };
        let rules = recorded
            .trace
            .rules
            .iter()
            .enumerate()
            .map(|(index, rule)| {
                let mut rule_fields = vec![(b"selected".to_vec(), boolean(rule.selected)?)];
                rule_fields.push((
                    b"bindings".to_vec(),
                    projection_object(
                        scope,
                        rule.bindings
                            .iter()
                            .map(|(binding, value)| {
                                Ok((
                                    binding.to_string().into_bytes(),
                                    projected_value_term(scope, value.clone())?,
                                ))
                            })
                            .collect::<Result<_, ExecutableErrorV1>>()?,
                    )?,
                ));
                if let Some(origin) = metadata
                    .and_then(|metadata| diagnostic_field(metadata, b"rules"))
                    .and_then(|rules| diagnostic_index_field(rules, rule.rule))
                {
                    rule_fields.push((b"source".to_vec(), origin.clone()));
                }
                // Retain all evaluated premises on selected rules; blocked rules
                // report their evaluated terminal premise and structural guards.
                // The full native record remains available through recorded_event.
                let premises = rule
                    .predicates
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| rule.selected || *index + 1 == rule.predicates.len())
                    .map(|(index, value)| {
                        Ok((
                            index.to_string().into_bytes(),
                            expression(value, &mut used_slots)?,
                        ))
                    })
                    .collect::<Result<_, ExecutableErrorV1>>()?;
                rule_fields.push((b"premises".to_vec(), projection_object(scope, premises)?));
                rule_fields.push((
                    b"premises-elided".to_vec(),
                    boolean(!rule.selected && rule.predicates.len() > 1)?,
                ));
                let effects = rule
                    .effects
                    .iter()
                    .enumerate()
                    .map(|(index, effect)| {
                        used_slots.insert(effect.slot);
                        let mut effect_fields = vec![
                            (
                                b"slot".to_vec(),
                                diagnostic_number(scope, effect.slot as f64)?,
                            ),
                            (b"additive".to_vec(), boolean(effect.additive)?),
                        ];
                        if let Some(subject) = &effect.subject {
                            effect_fields.push((
                                b"subject".to_vec(),
                                projected_value_term(
                                    scope,
                                    ExecutableValueV1::Referent(subject.clone()),
                                )?,
                            ));
                        }
                        if let Some(evaluated) = &effect.evaluated {
                            effect_fields.push((
                                b"evaluated".to_vec(),
                                expression(evaluated, &mut used_slots)?,
                            ));
                        }
                        Ok((
                            index.to_string().into_bytes(),
                            projection_object(scope, effect_fields)?,
                        ))
                    })
                    .collect::<Result<_, ExecutableErrorV1>>()?;
                rule_fields.push((b"effects".to_vec(), projection_object(scope, effects)?));
                for (name, guards) in [
                    (b"required-present".as_slice(), &rule.required_present),
                    (b"required-absent".as_slice(), &rule.required_absent),
                ] {
                    rule_fields.push((
                        name.to_vec(),
                        projection_object(
                            scope,
                            guards
                                .iter()
                                .map(|(slot, matched)| {
                                    used_slots.insert(*slot);
                                    Ok((slot.to_string().into_bytes(), boolean(*matched)?))
                                })
                                .collect::<Result<_, ExecutableErrorV1>>()?,
                        )?,
                    ));
                }
                Ok((
                    index.to_string().into_bytes(),
                    projection_object(scope, rule_fields)?,
                ))
            })
            .collect::<Result<_, ExecutableErrorV1>>()?;
        fields.push((b"rules".to_vec(), projection_object(scope, rules)?));
        let states = used_slots
            .into_iter()
            .map(|slot| {
                let mut state = vec![];
                if let Some(source) = metadata
                    .and_then(|metadata| diagnostic_field(metadata, b"states"))
                    .and_then(|states| diagnostic_index_field(states, slot))
                {
                    state.push((b"source".to_vec(), source.clone()));
                }
                for (name, config) in [
                    (b"before".as_slice(), &recorded.before),
                    (b"after".as_slice(), &recorded.after),
                ] {
                    if let Some(value) = config[usize::from(slot)].value() {
                        state.push((name.to_vec(), projected_value_term(scope, value.clone())?));
                    }
                }
                let subjects = [&recorded.before, &recorded.after]
                    .into_iter()
                    .filter_map(|config| match config[usize::from(slot)].value() {
                        Some(ExecutableValueV1::RelationTable(table)) => Some(table.rows.keys()),
                        _ => None,
                    })
                    .flatten()
                    .cloned()
                    .collect::<BTreeSet<_>>();
                if !subjects.is_empty() {
                    let rows = subjects
                        .into_iter()
                        .enumerate()
                        .map(|(index, subject)| {
                            let reference = ExecutableValueV1::Referent(subject);
                            let mut row = vec![(
                                b"subject".to_vec(),
                                projected_value_term(scope, reference.clone())?,
                            )];
                            for (name, config) in [
                                (b"before".as_slice(), &recorded.before),
                                (b"after".as_slice(), &recorded.after),
                            ] {
                                if let Some(ExecutableValueV1::RelationTable(table)) =
                                    config[usize::from(slot)].value()
                                {
                                    if table.present(&reference)? {
                                        row.push((
                                            name.to_vec(),
                                            projected_value_term(scope, table.read(&reference)?)?,
                                        ));
                                    }
                                }
                            }
                            Ok((
                                index.to_string().into_bytes(),
                                projection_object(scope, row)?,
                            ))
                        })
                        .collect::<Result<_, ExecutableErrorV1>>()?;
                    state.push((b"rows".to_vec(), diagnostic_index(scope, rows)?));
                }
                Ok((
                    slot.to_string().into_bytes(),
                    projection_object(scope, state)?,
                ))
            })
            .collect::<Result<_, ExecutableErrorV1>>()?;
        fields.push((b"states".to_vec(), projection_object(scope, states)?));
        projection_object(scope, fields)
    }

    pub(super) fn retain_executed_event(
        &mut self,
        step: &ExecutableStepV1,
        step_ordinal: u64,
        configuration_ordinal: u64,
        after: &[ExecutableSlotV1],
        trace: ExecutableEvaluationTraceV1,
    ) {
        // Latest per physical entry: high-frequency tick entries cannot erase
        // an actual input attack/heal. Total retained entries are bounded.
        if self.recorded_events.len() == MAX_RECORDED_ENTRIES
            && !self.recorded_events.contains_key(&step.occurrence.entry)
        {
            if let Some(entry) = self
                .recorded_events
                .iter()
                .min_by_key(|(_, event)| event.step_ordinal)
                .map(|(entry, _)| *entry)
            {
                self.recorded_events.remove(&entry);
            }
        }
        self.recorded_events.insert(
            step.occurrence.entry,
            ExecutableRecordedEventV1 {
                step: step.clone(),
                physical_plan: self.physical_plan,
                before: self.configuration.clone(),
                after: after.to_vec(),
                trace,
                step_ordinal,
                configuration_ordinal,
            },
        );
    }

    pub fn recorded_event(&self, entry: u16) -> Option<&ExecutableRecordedEventV1> {
        self.recorded_events.get(&entry)
    }

    pub fn intervene(
        &self,
        query: &ExecutableInterventionQueryV1,
    ) -> Result<ExecutableInterventionResultV1, ExecutableErrorV1> {
        validate_value_expression(&query.desired, 0)?;
        let recorded = self
            .recorded_events
            .values()
            .find(|event| event.step.id == query.event)
            .ok_or(ExecutableErrorV1::NoStep)?;
        if recorded.physical_plan != self.physical_plan
            || query.allowed.len() > MAX_INTERVENTION_CHOICES
            || query.maximum_evaluations > MAX_INTERVENTION_EVALUATIONS
        {
            return Err(ExecutableErrorV1::ResourceLimit);
        }
        let mut allowed = query.allowed.clone();
        allowed.sort();
        allowed.dedup();
        if allowed.iter().any(|change| {
            change.subject.is_none()
                && allowed
                    .iter()
                    .any(|other| other.slot == change.slot && other.subject.is_some())
        }) {
            return Err(ExecutableErrorV1::MalformedProgram);
        }
        let mut effective = Vec::new();
        for change in &allowed {
            let before = change.before_value(&recorded.before)?;
            // Finite values must be canonical, even for direct native callers.
            let mut bytes = Vec::new();
            encode_value(&mut bytes, &change.value)?;
            Decoder::new(&bytes).value()?;
            if before.as_ref() != Some(&change.value) {
                effective.push(change.clone());
            }
        }
        let allowed = effective;
        let mut result = ExecutableInterventionResultV1 {
            event: query.event,
            evaluations: 0,
            completed: false,
            exhausted: false,
            solution: None,
            predicted: None,
        };
        let context = EvaluationContextV1 {
            allocation_root: self.allocation.root,
            step_ordinal: recorded.step_ordinal,
            reads: None,
            bindings: None,
            relational_occurrence: None,
        };
        // Streaming combinations: no powerset is materialized and the bound
        // counts actual evaluator runs, including the empty intervention.
        for count in 0..=allowed.len() {
            let mut indices = (0..count).collect::<Vec<_>>();
            loop {
                let changes = indices
                    .iter()
                    .map(|index| allowed[*index].clone())
                    .collect::<Vec<_>>();
                if !changes
                    .windows(2)
                    .any(|pair| pair[0].slot == pair[1].slot && pair[0].subject == pair[1].subject)
                {
                    if result.evaluations == query.maximum_evaluations {
                        result.exhausted = true;
                        return Ok(result);
                    }
                    let mut pre = recorded.before.clone();
                    for change in &changes {
                        change.apply(&mut pre)?;
                    }
                    result.evaluations += 1;
                    // A failed evaluator run is not a false hypothesis. This
                    // query contract rejects the query on any domain/conflict/
                    // resource error; it cannot later report full enumeration.
                    let (next, _) = self.prepare_step_traced(
                        recorded.step.occurrence.clone(),
                        recorded.step_ordinal,
                        recorded.configuration_ordinal,
                        &pre,
                        None,
                    )?;
                    let desired = evaluate(
                        &query.desired,
                        &next,
                        &recorded.step.occurrence.arguments,
                        context,
                    )?;
                    if boolean(desired)? {
                        result.solution = Some(changes);
                        result.predicted = Some(next);
                        // Enumeration is intentionally stopped at the first
                        // minimal solution, not falsely called complete.
                        return Ok(result);
                    }
                }
                let Some(position) = (0..count)
                    .rev()
                    .find(|index| indices[*index] < allowed.len() - count + *index)
                else {
                    break;
                };
                indices[position] += 1;
                for index in position + 1..count {
                    indices[index] = indices[index - 1] + 1;
                }
            }
        }
        result.completed = true;
        Ok(result)
    }
}
