use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{
    ExecutableExpressionV1 as E, ExecutableInputSourceV1, ExecutableInterventionChangeV1,
    ExecutableInterventionQueryV1, ExecutableKeyPhaseV1, ExecutableValueV1,
    WasmSessionPhysicalInputV1, WasmSessionTickV1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const ENCOUNTER: &[u8] = include_bytes!("../../../test-vectors/authoring/live-encounter.clause");

fn field<'a>(term: &'a Term, name: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [key, value, rest] = current.as_triple().unwrap().slots();
        if key.as_atom().unwrap().canonical_payload() == name {
            return value;
        }
        current = rest;
    }
}
fn number(term: &Term) -> f64 {
    f64::from_bits(u64::from_le_bytes(
        term.as_atom()
            .unwrap()
            .canonical_payload()
            .try_into()
            .unwrap(),
    ))
}
fn entries(term: &Term) -> Vec<(&[u8], &Term)> {
    let mut entries = Vec::new();
    let mut current = term;
    while let Some(triple) = current.as_triple() {
        let [key, value, rest] = triple.slots();
        entries.push((key.as_atom().unwrap().canonical_payload(), value));
        current = rest;
    }
    entries
}
fn input(
    w: &mut ResidentSourceWorkbenchV1,
    sequence: u64,
    source: ExecutableInputSourceV1,
    value: Option<ExecutableValueV1>,
) {
    w.apply_physical_input(
        w.generation().handle,
        WasmSessionPhysicalInputV1 {
            input_sequence: sequence,
            source,
            value,
        },
    )
    .unwrap();
}
fn key(w: &mut ResidentSourceWorkbenchV1, sequence: u64, code: &[u8]) {
    input(
        w,
        sequence,
        ExecutableInputSourceV1::Keyboard {
            code: code.to_vec(),
            phase: ExecutableKeyPhaseV1::Down,
        },
        None,
    );
}
fn tick(w: &mut ResidentSourceWorkbenchV1, revision: u64) -> Term {
    w.tick_to_candidate(WasmSessionTickV1 {
        configuration_revision: revision,
        fixed_tick_milliseconds: 16,
    })
    .unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
}

#[test]
fn real_open_encounter_changed_rule_preserves_affected_world_state() {
    let mut w = ResidentSourceWorkbenchV1::open(ENCOUNTER).unwrap();
    assert_eq!(w.exact_source(), ENCOUNTER);
    key(&mut w, 1, b"BeginEncounter");
    input(
        &mut w,
        2,
        ExecutableInputSourceV1::Scalar {
            channel: b"PointerWorldX".to_vec(),
        },
        Some(ExecutableValueV1::number(2.0).unwrap()),
    );
    input(
        &mut w,
        3,
        ExecutableInputSourceV1::Scalar {
            channel: b"PointerWorldZ".to_vec(),
        },
        Some(ExecutableValueV1::number(4.0).unwrap()),
    );
    key(&mut w, 4, b"IssueMove");
    tick(&mut w, 1);
    key(&mut w, 5, b"Attack");
    let before = tick(&mut w, 2);
    assert_eq!(number(field(field(&before, b"cinder-1"), b"vitality")), 9.0);
    assert!(number(field(field(&before, b"warrior-1"), b"action-cooldown")) > 0.0);
    assert!(number(field(field(&before, b"moonwell"), b"burn-remaining")) > 0.0);
    key(&mut w, 6, b"Attack");
    let prior_bindings = w.state_bindings().unwrap();
    let old_generation = w.generation().clone();
    let effect = w
        .scalar_effects()
        .unwrap()
        .into_iter()
        .find(|effect| effect.expression == b"0.0 - ?damage")
        .unwrap();
    let pending = w
        .tick_to_candidate(WasmSessionTickV1 {
            configuration_revision: 3,
            fixed_tick_milliseconds: 16,
        })
        .unwrap();
    assert_eq!(
        w.edit_scalar_effect(old_generation.handle, &effect, &effect.expression)
            .unwrap(),
        old_generation
    );
    assert_eq!(w.pending_candidate(), Some(pending));
    assert_eq!(w.exact_source(), ENCOUNTER, "no-op changed exact source");
    assert!(
        w.edit_scalar_effect(old_generation.handle, &effect, b"0.0\non injected")
            .is_err()
    );
    assert!(
        w.edit_scalar_effect(old_generation.handle, &effect, b"0.0 - (?damage * 2.0)")
            .is_err(),
        "changed edit carried an unadmitted candidate"
    );
    assert_eq!(w.generation(), &old_generation);
    assert_eq!(w.pending_candidate(), Some(pending));
    assert_eq!(
        w.exact_source(),
        ENCOUNTER,
        "rejected edit changed exact source"
    );
    w.admit().unwrap();
    key(&mut w, 7, b"Attack");
    let prior_state = w
        .recorded_event(b"party-attack")
        .unwrap()
        .unwrap()
        .before
        .clone();
    w.edit_scalar_effect(w.generation().handle, &effect, b"0.0 - (?damage * 2.0)")
        .unwrap();
    let mut expected_source = ENCOUNTER[..effect.expression_origin.start as usize].to_vec();
    expected_source.extend_from_slice(b"0.0 - (?damage * 2.0)");
    expected_source.extend_from_slice(&ENCOUNTER[effect.expression_origin.end as usize..]);
    assert_eq!(
        w.exact_source(),
        expected_source,
        "accepted edit omitted exact compiler-produced bytes"
    );
    assert!(w.rejects_stale_handle(old_generation.handle).unwrap());
    assert!(
        w.edit_scalar_effect(old_generation.handle, &effect, b"0.0")
            .is_err()
    );
    assert_eq!(
        w.exact_source(),
        expected_source,
        "stale edit changed exact source"
    );
    assert!(w.pending_candidate().is_none());
    assert!(
        w.last_projection().is_none(),
        "source edit does not admit or publish configuration"
    );
    key(&mut w, 1, b"Attack");
    let new_state = &w.recorded_event(b"party-attack").unwrap().unwrap().before;
    let new_bindings = w.state_bindings().unwrap();
    for binding in &prior_bindings {
        // Display labels select assertions to TEST. Compiler continuity never
        // uses these labels; exact typed mapping is checked in runtime.
        let new = new_bindings
            .iter()
            .find(|candidate| {
                candidate.state.subject == binding.state.subject
                    && candidate.state.relation_designation == binding.state.relation_designation
                    && std::mem::discriminant(&candidate.state.path)
                        == std::mem::discriminant(&binding.state.path)
                    && match (&candidate.state.path, &binding.state.path) {
                        (
                            clause_package::CanonicalStatePathV1::Field { designation: a, .. },
                            clause_package::CanonicalStatePathV1::Field { designation: b, .. },
                        ) => a == b,
                        _ => true,
                    }
            })
            .unwrap();
        let old_value = &prior_state[usize::from(binding.slot)];
        let new_value = &new_state[usize::from(new.slot)];
        if let Some(ExecutableValueV1::Referent(old_reference)) = old_value.value() {
            let Some(ExecutableValueV1::Referent(new_reference)) = new_value.value() else {
                panic!("typed target was lost");
            };
            assert_ne!(
                old_reference, new_reference,
                "new snapshot address is not old coordinate"
            );
        } else {
            assert_eq!(
                old_value, new_value,
                "unrelated state changed: {:?}",
                binding.state
            );
        }
    }
    let after = tick(&mut w, 1);
    assert_eq!(
        number(field(field(&after, b"cinder-1"), b"vitality")),
        9.0,
        "changed-source rule editing reset the actual ongoing encounter"
    );
}

#[test]
fn real_attack_explanation_and_minimal_finite_intervention_share_execution() {
    let mut w = ResidentSourceWorkbenchV1::open(ENCOUNTER).unwrap();
    key(&mut w, 1, b"BeginEncounter");
    tick(&mut w, 1);
    key(&mut w, 2, b"Attack");
    tick(&mut w, 2);
    let first = w.recorded_event(b"party-attack").unwrap().unwrap().clone();
    assert_eq!(
        first
            .trace
            .rules
            .iter()
            .filter(|rule| rule.selected)
            .count(),
        5
    );
    let deltas = first
        .trace
        .rules
        .iter()
        .flat_map(|rule| &rule.effects)
        .filter(|effect| effect.additive)
        .map(|effect| {
            effect
                .evaluated
                .as_ref()
                .unwrap()
                .value
                .as_number()
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(deltas.iter().sum::<f64>(), -91.0);
    for revision in 3..=54 {
        tick(&mut w, revision);
    }
    assert_eq!(
        w.recorded_event(b"party-attack").unwrap().unwrap(),
        &first,
        "later ticks erased the actual input event"
    );
    key(&mut w, 3, b"Attack");
    let event = w.recorded_event(b"party-attack").unwrap().unwrap().clone();
    let bindings = w.state_bindings().unwrap();
    let target = bindings
        .iter()
        .find(|binding| {
            binding.state.subject == b"cinder-1"
                && binding.state.relation_designation == b"vitality"
        })
        .unwrap()
        .slot;
    assert_eq!(event.before[usize::from(target)].as_number(), Some(9.0));
    assert_eq!(event.after[usize::from(target)].as_number(), Some(-82.0));
    let allowed = bindings
        .iter()
        .filter(|binding| binding.state.relation_designation == b"selected")
        .map(|binding| ExecutableInterventionChangeV1 {
            slot: binding.slot,
            value: ExecutableValueV1::Boolean(false),
        })
        .collect::<Vec<_>>();
    assert_eq!(allowed.len(), 5);
    let mut query = ExecutableInterventionQueryV1 {
        event: event.step.id,
        allowed,
        desired: E::GreaterThan(
            Box::new(E::Slot(target)),
            Box::new(E::Constant(ExecutableValueV1::number(0.0).unwrap())),
        ),
        maximum_evaluations: 32,
    };
    let answer = w.intervene(&query).unwrap();
    assert_eq!(answer.solution.as_ref().unwrap().len(), 4);
    assert_eq!(
        answer.predicted.as_ref().unwrap()[usize::from(target)].as_number(),
        Some(2.0)
    );
    assert!(!answer.exhausted);
    assert_eq!(
        w.recorded_event(b"party-attack").unwrap().unwrap(),
        &event,
        "query mutated recorded execution"
    );
    // Reproduce the recorded pre-state in an isolated, normally executed
    // encounter, then apply the answer ONLY through declared physical inputs.
    let mut replay = ResidentSourceWorkbenchV1::open(ENCOUNTER).unwrap();
    key(&mut replay, 1, b"BeginEncounter");
    tick(&mut replay, 1);
    key(&mut replay, 2, b"Attack");
    tick(&mut replay, 2);
    for revision in 3..=54 {
        tick(&mut replay, revision);
    }
    key(&mut replay, 3, b"ClearSelection");
    let mut input_sequence = 3;
    for binding in bindings
        .iter()
        .filter(|binding| binding.state.relation_designation == b"selected")
    {
        if answer
            .solution
            .as_ref()
            .unwrap()
            .iter()
            .any(|change| change.slot == binding.slot)
        {
            continue;
        }
        let reference = binding.state.subject_identity.unwrap();
        input_sequence += 1;
        input(
            &mut replay,
            input_sequence,
            ExecutableInputSourceV1::Referent {
                channel: b"Pick".to_vec(),
            },
            Some(ExecutableValueV1::Referent(
                clause_runtime::ExecutableReferentV1::declared(
                    reference.domain.get(),
                    reference.identity.get(),
                ),
            )),
        );
    }
    key(&mut replay, input_sequence + 1, b"Attack");
    let actual = tick(&mut replay, 55);
    assert_eq!(
        number(field(field(&actual, b"cinder-1"), b"vitality")),
        answer.predicted.as_ref().unwrap()[usize::from(target)]
            .as_number()
            .unwrap(),
        "normal input replay disagreed with intervention"
    );
    query.maximum_evaluations = 1;
    let bounded = w.intervene(&query).unwrap();
    assert!(bounded.exhausted && !bounded.completed && bounded.solution.is_none());
    query.maximum_evaluations = 32;
    query.desired = E::Constant(ExecutableValueV1::Boolean(false));
    let all = w.intervene(&query).unwrap();
    assert!(all.completed && !all.exhausted && all.solution.is_none());
    assert_eq!(all.evaluations, 32);
    let explanation = w.explanation(b"party-attack").unwrap();
    assert!(field(&explanation, b"rules").as_triple().is_some());
    eprintln!(
        "attack explanation bytes={}",
        clause_package::canonical_term_bytes(&explanation)
            .unwrap()
            .len()
    );
    query.desired = E::GreaterThan(
        Box::new(E::Divide(
            Box::new(E::Constant(ExecutableValueV1::number(1.0).unwrap())),
            Box::new(E::Constant(ExecutableValueV1::number(0.0).unwrap())),
        )),
        Box::new(E::Constant(ExecutableValueV1::number(0.0).unwrap())),
    );
    assert!(
        w.intervene(&query).is_err(),
        "undefined predicate became complete/no solution"
    );
    assert_eq!(w.recorded_event(b"party-attack").unwrap().unwrap(), &event);
}

#[test]
fn raw_text_import_is_honestly_fresh_and_account_edit_is_general() {
    let source =
        include_bytes!("../../../test-vectors/authoring/selected-account-contributions.clause");
    let mut w = ResidentSourceWorkbenchV1::open(source).unwrap();
    key(&mut w, 1, b"Apply");
    let first = tick(&mut w, 1);
    assert_eq!(number(field(field(&first, b"first"), b"balance")), 118.0);
    let effect = w
        .scalar_effects()
        .unwrap()
        .into_iter()
        .find(|effect| effect.expression == b"?amount")
        .unwrap();
    w.edit_scalar_effect(w.generation().handle, &effect, b"?amount * 2.0")
        .unwrap();
    let continuity = w.source_continuity().unwrap();
    let occurrences = entries(field(&continuity, b"formations"))
        .into_iter()
        .flat_map(|(_, page)| entries(page))
        .map(|(_, mapping)| {
            (
                number(field(mapping, b"new")) as u32,
                field(mapping, b"occurrence").clone(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let preserved = tick(&mut w, 1);
    assert_eq!(
        number(field(field(&preserved, b"first"), b"balance")),
        118.0
    );
    for revision in 2..=64 {
        tick(&mut w, revision);
    }
    key(&mut w, 1, b"Apply");
    let changed = tick(&mut w, 65);
    assert_eq!(number(field(field(&changed, b"first"), b"balance")), 154.0);
    let event = w.recorded_event(b"contribute").unwrap().unwrap();
    assert_eq!(
        event
            .trace
            .rules
            .iter()
            .flat_map(|rule| &rule.effects)
            .filter(|effect| effect.additive)
            .map(|effect| effect
                .evaluated
                .as_ref()
                .unwrap()
                .value
                .as_number()
                .unwrap())
            .sum::<f64>(),
        36.0
    );
    let effect = w
        .scalar_effects()
        .unwrap()
        .into_iter()
        .find(|effect| effect.expression == b"?amount * 2.0")
        .unwrap();
    w.edit_scalar_effect(w.generation().handle, &effect, b"?amount * 3.0")
        .unwrap();
    let next_continuity = w.source_continuity().unwrap();
    for (_, page) in entries(field(&next_continuity, b"formations")) {
        for (_, mapping) in entries(page) {
            let old = number(field(mapping, b"old")) as u32;
            if let Some(occurrence) = occurrences.get(&old) {
                assert_eq!(
                    field(mapping, b"occurrence"),
                    occurrence,
                    "second explicit edit changed continuing occurrence identity"
                );
            }
        }
    }
    w.hot_reload(source).unwrap();
    let imported = tick(&mut w, 1);
    assert_eq!(number(field(field(&imported, b"first"), b"balance")), 100.0);
}

#[test]
fn actual_heal_explains_selected_law_origin_and_evaluated_result() {
    let mut w = ResidentSourceWorkbenchV1::open(ENCOUNTER).unwrap();
    key(&mut w, 1, b"BeginEncounter");
    let frame = tick(&mut w, 1);
    let reference = clause_runtime::projected_referent_value_v1(field(
        field(&frame, b"moonwell"),
        b"$referent",
    ))
    .unwrap()
    .unwrap();
    input(
        &mut w,
        2,
        ExecutableInputSourceV1::Referent {
            channel: b"Target".to_vec(),
        },
        Some(ExecutableValueV1::Referent(reference)),
    );
    key(&mut w, 3, b"Heal");
    let explanation = w.explanation(b"party-heal").unwrap();
    let selected = entries(field(&explanation, b"rules"))
        .into_iter()
        .filter(|(_, rule)| {
            field(rule, b"selected")
                .as_atom()
                .unwrap()
                .canonical_payload()
                == [1]
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    let source = field(selected[0].1, b"source");
    let laws = entries(field(source, b"laws"));
    assert!(!laws.is_empty(), "actual selected law provenance was lost");
    let mut exact_origins = Vec::new();
    for (_, law) in laws {
        let start = number(field(law, b"start")) as usize;
        let end = number(field(law, b"end")) as usize;
        assert_eq!(
            field(law, b"artifact"),
            field(field(source, b"origin"), b"artifact")
        );
        exact_origins.push(std::str::from_utf8(&ENCOUNTER[start..end]).unwrap());
    }
    assert_eq!(
        exact_origins,
        vec![
            "law clamp-interior\n  if\n    ?lower <= ?value\n    ?value <= ?upper\n  then\n    ?value clamped between ?lower and ?upper as ?value",
            "derive clamp-interior",
        ]
    );
    assert!(!entries(field(selected[0].1, b"effects")).is_empty());
    let event = w.recorded_event(b"party-heal").unwrap().unwrap();
    assert_eq!(
        event
            .trace
            .rules
            .iter()
            .flat_map(|rule| &rule.effects)
            .filter(|effect| effect.additive)
            .map(|effect| effect
                .evaluated
                .as_ref()
                .unwrap()
                .value
                .as_number()
                .unwrap())
            .collect::<Vec<_>>(),
        vec![28.0]
    );
    let target = w
        .state_bindings()
        .unwrap()
        .into_iter()
        .find(|binding| {
            binding.state.subject == b"moonwell"
                && binding.state.relation_designation == b"vitality"
        })
        .unwrap()
        .slot;
    assert_eq!(
        event.after[usize::from(target)].as_number().unwrap()
            - event.before[usize::from(target)].as_number().unwrap(),
        28.0
    );
    let actual = tick(&mut w, 2);
    assert!(
        number(field(field(&actual, b"moonwell"), b"vitality"))
            > number(field(field(&frame, b"moonwell"), b"vitality"))
    );
}
