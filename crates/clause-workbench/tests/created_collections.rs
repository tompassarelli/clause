use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{
    ExecutableReferentV1, ExecutableRelationTableV1, ExecutableValueV1 as V,
    projected_relation_table_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] =
    include_bytes!("../../../test-vectors/authoring/created-timed-contributions.clause");
fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current
            .as_triple()
            .unwrap_or_else(|| panic!("missing projected field {:?}", String::from_utf8_lossy(key)))
            .slots();
        if name.as_atom().unwrap().canonical_payload() == key {
            return value;
        }
        current = rest;
    }
}
fn table(term: &Term, name: &[u8]) -> ExecutableRelationTableV1 {
    projected_relation_table_v1(field(field(term, b"relations"), name))
        .unwrap()
        .unwrap()
}
fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8], arguments: &[V]) -> Term {
    let occurrence = w.handler_occurrence(name, arguments).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
}
fn n(value: f64) -> V {
    V::number(value).unwrap()
}
fn entries(term: &Term) -> Vec<&Term> {
    let mut values = Vec::new();
    let mut current = term;
    while let Some(triple) = current.as_triple() {
        let [_, value, rest] = triple.slots();
        values.push(value);
        current = rest;
    }
    values
}
fn known(frame: &Term) -> Vec<ExecutableReferentV1> {
    table(frame, b"known-goal")
        .rows()
        .values()
        .flatten()
        .map(|value| value.as_referent().unwrap().clone())
        .collect()
}
fn balance(frame: &Term) -> f64 {
    table(frame, b"balance")
        .rows()
        .values()
        .flatten()
        .next()
        .unwrap()
        .as_number()
        .unwrap()
}

#[test]
fn created_equal_values_are_distinct_timed_contributions_and_expire_independently() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    let first_frame = run(&mut w, b"create-goal", &[n(7.0), n(1.0)]);
    let first = known(&first_frame)[0].clone();
    let second_frame = run(&mut w, b"create-goal", &[n(7.0), n(3.0)]);
    let goals = known(&second_frame);
    assert_eq!(goals.len(), 2);
    let second = goals.into_iter().find(|goal| goal != &first).unwrap();
    assert_ne!(first, second);
    let after = run(&mut w, b"tick", &[n(1.0)]);
    assert_eq!(balance(&after), 114.0);
    let event = w.recorded_event(b"tick").unwrap().unwrap();
    let contributions = event
        .trace
        .rules
        .iter()
        .filter(|rule| rule.selected)
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
    assert_eq!(contributions, vec![7.0, 7.0]);
    let after = run(&mut w, b"expire", &[]);
    assert_eq!(known(&after), vec![second.clone()]);
    assert!(!table(&after, b"remaining").rows().contains_key(&first));
    assert_eq!(
        table(&after, b"remaining").rows()[&second]
            .first()
            .unwrap()
            .as_number(),
        Some(2.0)
    );
    let after = run(&mut w, b"tick", &[n(1.0)]);
    assert_eq!(balance(&after), 121.0);
    let after = run(&mut w, b"cancel-goal", &[V::Referent(second)]);
    assert!(known(&after).is_empty());
    assert_eq!(balance(&run(&mut w, b"tick", &[n(1.0)])), 121.0);
    let trace = &w.recorded_event(b"tick").unwrap().unwrap().trace;
    assert!(trace.rules.iter().all(|rule| !rule.selected));
    assert!(
        trace
            .rules
            .iter()
            .flat_map(|rule| &rule.predicates)
            .flat_map(|predicate| &predicate.reads)
            .any(|read| matches!(read, clause_runtime::ExecutableReadV1::RelationSearch(..)))
    );
}

#[test]
fn collection_contributions_read_one_prestate_and_ordinary_overlap_rejects_atomically() {
    let source = std::str::from_utf8(SOURCE).unwrap().replace(
        "    ?remaining > 0.0\n",
        "    ?remaining > 0.0\n    ?account balance ?before\n    ?before = 100.0\n",
    );
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    run(&mut w, b"create-goal", &[n(7.0), n(3.0)]);
    run(&mut w, b"create-goal", &[n(7.0), n(3.0)]);
    assert_eq!(balance(&run(&mut w, b"tick", &[n(1.0)])), 114.0);
    let conflict = source
        .replace(
            "  accumulate\n    ?account balance ?amount * ?dt",
            "    ?account balance ?before + ?amount * ?dt",
        )
        .replace(
            "  withdraw\n    ?goal remaining ?remaining\n  include",
            "  withdraw\n    ?goal remaining ?remaining\n    ?account balance ?before\n  include",
        );
    let mut w = ResidentSourceWorkbenchV1::open(conflict.as_bytes()).unwrap();
    run(&mut w, b"create-goal", &[n(7.0), n(3.0)]);
    let before = run(&mut w, b"create-goal", &[n(7.0), n(3.0)]);
    let occurrence = w.handler_occurrence(b"tick", &[n(1.0)]).unwrap();
    let error = w
        .run_occurrences_to_candidate(&[occurrence])
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("ConflictingStateEffects"),
        "wrong failure: {error}"
    );
    assert!(w.recorded_event(b"tick").unwrap().is_none());
    let after = run(&mut w, b"expire", &[]);
    assert_eq!(balance(&after), 100.0);
    assert_eq!(table(&after, b"remaining"), table(&before, b"remaining"));
}

#[test]
fn exhausted_finite_join_is_an_error_not_absence() {
    let source = format!(
        "{}\non impossible-search ?account\n  when\n    ?account known goal ?first\n    ?account known goal ?second\n    ?account known goal ?third\n    ?first remaining ?first-duration\n    ?second remaining ?second-duration\n    ?third remaining ?third-duration\n    ?account balance ?balance\n    ?first-duration + ?second-duration + ?third-duration < 0.0\n  accumulate\n    ?account balance 1.0\n\non cheap-rejection ?account\n  when\n    ?account known goal ?first\n    ?account known goal ?second\n    ?account known goal ?third\n    ?account balance ?balance\n    ?balance < 0.0\n  accumulate\n    ?account balance 1.0\n",
        std::str::from_utf8(SOURCE).unwrap()
    );
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    for _ in 0..17 {
        run(&mut w, b"create-goal", &[n(7.0), n(3.0)]);
    }
    // An independent false guard can reject before expansion. Exhaustion still
    // rejects atomically when the condition requires the complete broad join.
    let unchanged = run(&mut w, b"cheap-rejection", &[]);
    assert_eq!(balance(&unchanged), 100.0);
    assert_eq!(known(&unchanged).len(), 17);
    let trace = &w.recorded_event(b"cheap-rejection").unwrap().unwrap().trace;
    assert!(trace.rules.iter().all(|rule| !rule.selected));
    assert!(!trace.rules.iter().flat_map(|rule| &rule.predicates)
        .flat_map(|predicate| &predicate.reads)
        .any(|read| matches!(read, clause_runtime::ExecutableReadV1::RelationRow(_, _, V::Referent(_)))));
    let occurrence = w.handler_occurrence(b"impossible-search", &[]).unwrap();
    let error = w
        .run_occurrences_to_candidate(&[occurrence])
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("ResourceLimit"),
        "exhausted join returned wrong result: {error}"
    );
    assert!(w.recorded_event(b"impossible-search").unwrap().is_none());
    let frame = run(&mut w, b"expire", &[]);
    assert_eq!(known(&frame).len(), 17);
    assert_eq!(balance(&frame), 100.0);
}

#[test]
fn real_encounter_accepts_independent_runtime_created_burns() {
    let source = [
        include_bytes!("../../../test-vectors/authoring/live-encounter.clause").as_slice(),
        b"\n",
        include_bytes!("../../../test-vectors/authoring/created-burn-extension.clause").as_slice(),
    ]
    .concat();
    let mut w = ResidentSourceWorkbenchV1::open(&source).unwrap();
    let first = run(&mut w, b"ignite-target", &[n(1.0)]);
    let first_effect = table(&first, b"effect-remaining")
        .rows()
        .keys()
        .next()
        .unwrap()
        .clone();
    let second = run(&mut w, b"ignite-target", &[n(3.0)]);
    assert_eq!(table(&second, b"burn-target").rows().len(), 2);
    let target = table(&second, b"burn-target")
        .rows()
        .values()
        .flatten()
        .next()
        .unwrap()
        .as_referent()
        .unwrap()
        .clone();
    let facets = field(field(&second, b"cinder-1"), b"$referents");
    assert_eq!(
        clause_runtime::projected_referent_value_v1(field(
            facets,
            target.domain().to_string().as_bytes()
        ))
        .unwrap()
        .unwrap(),
        target
    );
    let value = |term: &Term| {
        f64::from_bits(u64::from_le_bytes(
            term.as_atom()
                .unwrap()
                .canonical_payload()
                .try_into()
                .unwrap(),
        ))
    };
    assert_eq!(
        value(field(field(&second, b"cinder-1"), b"vitality")),
        100.0
    );
    assert_eq!(
        value(field(
            field(field(&second, b"warrior-1"), b"actor-position"),
            b"x"
        )),
        -2.0
    );
    let tick = |w: &mut ResidentSourceWorkbenchV1, revision| {
        w.tick_to_candidate(clause_runtime::WasmSessionTickV1 {
            configuration_revision: revision,
            fixed_tick_milliseconds: 1000,
        })
        .unwrap();
        decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
    };
    let after = tick(&mut w, 1);
    assert_eq!(value(field(field(&after, b"cinder-1"), b"vitality")), 86.0);
    let after = tick(&mut w, 2);
    assert_eq!(value(field(field(&after, b"cinder-1"), b"vitality")), 79.0);
    assert!(
        !table(&after, b"effect-remaining")
            .rows()
            .contains_key(&first_effect)
    );
    assert_eq!(table(&after, b"burn-target").rows().len(), 1);
    let after = tick(&mut w, 3);
    assert_eq!(value(field(field(&after, b"cinder-1"), b"vitality")), 72.0);
    let after = tick(&mut w, 4);
    assert_eq!(value(field(field(&after, b"cinder-1"), b"vitality")), 72.0);
    assert!(table(&after, b"burn-target").rows().is_empty());
}

#[test]
fn collection_types_reject_nonnumeric_contributions() {
    let source = std::str::from_utf8(SOURCE).unwrap();
    for invalid in ["true", "?goal", "?goal * ?dt", "\"seven\""] {
        let bad = source.replace("?amount * ?dt", invalid);
        assert!(
            ResidentSourceWorkbenchV1::open(bad.as_bytes()).is_err(),
            "accepted {invalid}"
        );
    }
}

#[test]
fn physical_collection_program_rejects_unbound_bindings_and_unordered_facets() {
    use clause_runtime::{
        ExecutableErrorV1, ExecutableExpressionV1 as E, decode_executable_physical_plan_v1,
        encode_executable_physical_plan_v1,
    };
    let w = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    let original = decode_executable_physical_plan_v1(&w.generation().cpp1).unwrap();
    for bad in [
        E::Binding(100),
        E::ReferentFacet {
            value: Box::new(E::Argument(0)),
            domain: 1,
            members: vec![2, 1],
        },
    ] {
        let mut plan = original.clone();
        plan.program.rules[0].predicates.insert(0, bad);
        assert_eq!(
            encode_executable_physical_plan_v1(&plan).unwrap_err(),
            ExecutableErrorV1::MalformedProgram
        );
    }
}

#[test]
fn one_creation_step_allocates_distinct_identity_per_matching_account() {
    let source = format!(
        "{}\nsecond-account\n  shape: Account\nsecond-account balance 100.0\n",
        std::str::from_utf8(SOURCE).unwrap()
    );
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let frame = run(&mut w, b"create-goal", &[n(7.0), n(3.0)]);
    let identities = known(&frame);
    assert_eq!(identities.len(), 2);
    assert_ne!(identities[0], identities[1]);
    assert_eq!(table(&frame, b"known-goal").rows().len(), 2);
    let frame = run(&mut w, b"tick", &[n(1.0)]);
    assert!(
        table(&frame, b"balance")
            .rows()
            .values()
            .flatten()
            .all(|value| value.as_number() == Some(107.0))
    );
}

#[test]
fn fractional_burn_lifetimes_and_live_edit_preserve_created_identity() {
    let source = [
        include_bytes!("../../../test-vectors/authoring/live-encounter.clause").as_slice(),
        b"\n",
        include_bytes!("../../../test-vectors/authoring/created-burn-extension.clause").as_slice(),
    ]
    .concat();
    let mut w = ResidentSourceWorkbenchV1::open(&source).unwrap();
    run(&mut w, b"ignite-target", &[n(0.5)]);
    let before = run(&mut w, b"ignite-target", &[n(1.5)]);
    let identities = table(&before, b"burn-target");
    let effect = w
        .scalar_effects()
        .unwrap()
        .into_iter()
        .find(|effect| effect.expression == b"0.0 - ?damage * ?elapsed")
        .unwrap();
    let old = w.generation().clone();
    assert_eq!(
        w.edit_scalar_effect(old.handle, &effect, &effect.expression)
            .unwrap(),
        old
    );
    assert!(w.edit_scalar_effect(old.handle, &effect, b"true").is_err());
    w.edit_scalar_effect(old.handle, &effect, b"0.0 - ?damage * ?elapsed * 2.0")
        .unwrap();
    assert!(w.edit_scalar_effect(old.handle, &effect, b"0.0").is_err());
    w.tick_to_candidate(clause_runtime::WasmSessionTickV1 {
        configuration_revision: 1,
        fixed_tick_milliseconds: 1000,
    })
    .unwrap();
    let after =
        decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap();
    assert_eq!(
        table(&after, b"burn-target")
            .rows()
            .keys()
            .map(|referent| referent.identity().clone())
            .collect::<Vec<_>>(),
        identities
            .rows()
            .keys()
            .map(|referent| referent.identity().clone())
            .collect::<Vec<_>>()
    );
    let number = |term: &Term| {
        f64::from_bits(u64::from_le_bytes(
            term.as_atom()
                .unwrap()
                .canonical_payload()
                .try_into()
                .unwrap(),
        ))
    };
    let continuity = w.source_continuity().unwrap();
    let domain = entries(field(&continuity, b"formations"))
        .into_iter()
        .flat_map(entries)
        .find(|mapping| number(field(mapping, b"old")) == identities.subject_domain() as f64)
        .unwrap();
    assert_eq!(
        number(field(domain, b"new")),
        table(&after, b"burn-target").subject_domain() as f64
    );
    assert_eq!(number(field(field(&after, b"cinder-1"), b"vitality")), 79.0);
    let current_effect = w
        .scalar_effects()
        .unwrap()
        .into_iter()
        .find(|effect| effect.expression == b"0.0 - ?damage * ?elapsed * 2.0")
        .unwrap();
    let trace = &w
        .recorded_handler_event(current_effect.handler)
        .unwrap()
        .unwrap()
        .trace;
    let additive = trace
        .rules
        .iter()
        .filter(|rule| rule.selected)
        .flat_map(|rule| &rule.effects)
        .filter(|effect| effect.additive)
        .collect::<Vec<_>>();
    assert_eq!(additive.len(), 2);
    assert!(additive.iter().all(|effect| effect.subject.is_some()));
    let explanation = w.handler_explanation(current_effect.handler).unwrap();
    assert!(field(&explanation, b"rules").as_triple().is_some());
    w.tick_to_candidate(clause_runtime::WasmSessionTickV1 {
        configuration_revision: 2,
        fixed_tick_milliseconds: 1000,
    })
    .unwrap();
    let after =
        decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap();
    assert_eq!(number(field(field(&after, b"cinder-1"), b"vitality")), 72.0);
    assert_eq!(table(&after, b"burn-target").rows().len(), 1);
}

#[test]
fn collection_encounter_retains_real_battle_movement_targeting_and_selection_guards() {
    let source = [
        include_bytes!("../../../test-vectors/authoring/live-encounter.clause").as_slice(),
        b"\n",
        include_bytes!("../../../test-vectors/authoring/created-burn-extension.clause").as_slice(),
    ]
    .concat();
    let mut w = ResidentSourceWorkbenchV1::open(&source).unwrap();
    run(&mut w, b"begin-encounter", &[]);
    run(&mut w, b"observe-order-x", &[n(2.0)]);
    run(&mut w, b"observe-order-z", &[n(4.0)]);
    run(&mut w, b"issue-move", &[]);
    w.tick_to_candidate(clause_runtime::WasmSessionTickV1 {
        configuration_revision: 1,
        fixed_tick_milliseconds: 16,
    })
    .unwrap();
    let frame =
        decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap();
    let number = |term: &Term| {
        f64::from_bits(u64::from_le_bytes(
            term.as_atom()
                .unwrap()
                .canonical_payload()
                .try_into()
                .unwrap(),
        ))
    };
    assert!(
        number(field(
            field(field(&frame, b"warrior-1"), b"actor-position"),
            b"x"
        )) > -2.0
    );
    run(&mut w, b"clear-selection", &[]);
    let after = run(&mut w, b"party-attack", &[]);
    assert_eq!(
        number(field(field(&after, b"cinder-1"), b"vitality")),
        100.0,
        "unselected units attacked"
    );
    run(&mut w, b"select-all", &[]);
    let after = run(&mut w, b"party-attack", &[]);
    assert_eq!(number(field(field(&after, b"cinder-1"), b"vitality")), 9.0);
    let facets = field(field(&after, b"cinder-2"), b"$referents");
    let domain = number(field(field(&after, b"$referent-inputs"), b"Target")) as u32;
    let target =
        clause_runtime::projected_referent_value_v1(field(facets, domain.to_string().as_bytes()))
            .unwrap()
            .unwrap();
    run(&mut w, b"choose-target", &[V::Referent(target)]);
    run(&mut w, b"ignite-target", &[n(1.0)]);
    run(&mut w, b"ignite-target", &[n(3.0)]);
    w.tick_to_candidate(clause_runtime::WasmSessionTickV1 {
        configuration_revision: 2,
        fixed_tick_milliseconds: 1000,
    })
    .unwrap();
    let after =
        decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap();
    assert_eq!(number(field(field(&after, b"cinder-2"), b"vitality")), 86.0);
}
