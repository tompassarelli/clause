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
    pub evaluated: Option<ExecutableEvaluatedExpressionV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableRuleEvaluationV1 {
    pub rule: u16,
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
        rule: u16,
        slot: u16,
        additive: bool,
        evaluated: Option<ExecutableEvaluatedExpressionV1>,
    ) {
        if let Some(rule) = self.rules.iter_mut().find(|value| value.rule == rule) {
            rule.effects.push(ExecutableEffectEvaluationV1 {
                slot,
                additive,
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
/// are ordered by canonical (slot, typed value); one value per slot.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutableInterventionChangeV1 {
    pub slot: u16,
    pub value: ExecutableValueV1,
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
    let mut bytes = b"CIQ1".to_vec();
    bytes.extend_from_slice(query.event.as_bytes());
    bytes.extend_from_slice(&query.maximum_evaluations.to_le_bytes());
    encode_count(&mut bytes, query.allowed.len())?;
    for change in &query.allowed {
        bytes.extend_from_slice(&change.slot.to_le_bytes());
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
    if d.take(4)? != b"CIQ1" {
        return Err(ExecutableErrorV1::MalformedProgram);
    }
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
                "changed-coordinate-count; then (slot, canonical-typed-value) lexicographic",
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
                            change.slot.to_string().into_bytes(),
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
                    let (kind, coordinate, value) = match read {
                        ExecutableReadV1::State(slot, value) => {
                            used.insert(*slot);
                            ("state", *slot, value)
                        }
                        ExecutableReadV1::Argument(argument, value) => {
                            ("argument", *argument, value)
                        }
                    };
                    Ok((
                        index.to_string().into_bytes(),
                        projection_object(
                            scope,
                            vec![
                                (b"kind".to_vec(), diagnostic_text(scope, kind)?),
                                (
                                    b"coordinate".to_vec(),
                                    diagnostic_number(scope, coordinate as f64)?,
                                ),
                                (
                                    b"value".to_vec(),
                                    projected_value_term(scope, value.clone())?,
                                ),
                            ],
                        )?,
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
            .map(|rule| {
                let mut rule_fields = vec![(b"selected".to_vec(), boolean(rule.selected)?)];
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
                    rule.rule.to_string().into_bytes(),
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
        for change in &allowed {
            let state = recorded
                .before
                .get(usize::from(change.slot))
                .ok_or(ExecutableErrorV1::UnknownSlot(change.slot))?;
            if state.kind() != change.value.kind() {
                return Err(ExecutableErrorV1::TypeMismatch);
            }
            // Finite values must be canonical, even for direct native callers.
            let mut bytes = Vec::new();
            encode_value(&mut bytes, &change.value)?;
            Decoder::new(&bytes).value()?;
            if let Some(ExecutableValueV1::Referent(old)) = state.value() {
                let ExecutableValueV1::Referent(new) = &change.value else {
                    return Err(ExecutableErrorV1::TypeMismatch);
                };
                if old.domain != new.domain {
                    return Err(ExecutableErrorV1::TypeMismatch);
                }
            }
        }
        allowed.retain(|change| {
            recorded.before[usize::from(change.slot)].value() != Some(&change.value)
        });
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
                if !changes.windows(2).any(|pair| pair[0].slot == pair[1].slot) {
                    if result.evaluations == query.maximum_evaluations {
                        result.exhausted = true;
                        return Ok(result);
                    }
                    let mut pre = recorded.before.clone();
                    for change in &changes {
                        pre[usize::from(change.slot)] = change.value.clone().into();
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
