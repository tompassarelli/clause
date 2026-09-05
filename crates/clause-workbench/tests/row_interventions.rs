use clause_package::Term;
use clause_runtime::{
    ExecutableExpressionV1 as E, ExecutableInterventionChangeV1 as Change,
    ExecutableInterventionQueryV1 as Query, ExecutableReferentV1 as Referent,
    ExecutableRelationTableV1, ExecutableValueV1 as V, decode_executable_intervention_query_v1,
    encode_executable_intervention_query_v1, projected_referent_value_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] =
    include_bytes!("../../../test-vectors/authoring/created-relation-live-intervention.clause");
fn n(value: f64) -> V {
    V::number(value).unwrap()
}
fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8], arguments: &[V]) {
    let occurrence = w.handler_occurrence(name, arguments).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    w.admit().unwrap();
}
fn slot(w: &ResidentSourceWorkbenchV1, name: &[u8]) -> u16 {
    w.state_bindings()
        .unwrap()
        .iter()
        .find(|b| b.state.relation_designation == name)
        .unwrap()
        .slot
}
fn table(state: &clause_runtime::ExecutableSlotV1) -> &ExecutableRelationTableV1 {
    let Some(V::RelationTable(table)) = state.value() else {
        panic!("expected table")
    };
    table
}
fn read(slot: u16, subject: &Referent) -> E {
    E::RelationRead(
        Box::new(E::Slot(slot)),
        Box::new(E::Constant(V::Referent(subject.clone()))),
    )
}
fn above(slot: u16, subject: &Referent, threshold: f64) -> E {
    E::GreaterThan(
        Box::new(read(slot, subject)),
        Box::new(E::Constant(n(threshold))),
    )
}
fn entries(term: &Term) -> Vec<(&[u8], &Term)> {
    let mut result = Vec::new();
    let mut current = term;
    while let Some(triple) = current.as_triple() {
        let [key, value, rest] = triple.slots();
        result.push((key.as_atom().unwrap().canonical_payload(), value));
        current = rest;
    }
    result
}
fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    entries(term)
        .into_iter()
        .find(|(name, _)| *name == key)
        .unwrap()
        .1
}

#[test]
fn independent_rows_have_independent_costs_and_numeric_explanations() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    run(&mut w, b"attack", &[]);
    let event = w.recorded_event(b"attack").unwrap().unwrap().clone();
    let selected = slot(&w, b"selected");
    let health = slot(&w, b"vitality");
    let actors = table(&event.before[usize::from(selected)])
        .rows()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(actors.len(), 2);
    let mut query = Query {
        event: event.step.id,
        allowed: actors
            .iter()
            .map(|subject| Change {
                slot: selected,
                subject: Some(subject.clone()),
                value: V::Boolean(false),
            })
            .collect(),
        desired: E::And(
            Box::new(above(health, &actors[0], 95.0)),
            Box::new(above(health, &actors[1], 95.0)),
        ),
        maximum_evaluations: 4,
    };
    let bytes = encode_executable_intervention_query_v1(&query).unwrap();
    assert_eq!(&bytes[..4], b"CIQ2");
    assert_eq!(
        decode_executable_intervention_query_v1(&bytes).unwrap(),
        query
    );
    let answer = w.intervene(&query).unwrap();
    assert_eq!(answer.solution.as_ref().unwrap().len(), 2);
    assert_eq!(answer.evaluations, 4);
    query.allowed.reverse();
    query.allowed.push(query.allowed[0].clone());
    assert_eq!(w.intervene(&query).unwrap(), answer);
    for actor in &actors {
        assert_eq!(
            table(&answer.predicted.as_ref().unwrap()[usize::from(health)]).rows()[actor].first(),
            Some(&n(100.0))
        );
    }
    query.maximum_evaluations = 1;
    let bounded = w.intervene(&query).unwrap();
    assert!(bounded.exhausted && !bounded.completed && bounded.solution.is_none());
    query.maximum_evaluations = 4;
    query.desired = E::Constant(V::Boolean(false));
    let complete = w.intervene(&query).unwrap();
    assert!(complete.completed && !complete.exhausted && complete.solution.is_none());
    assert_eq!(complete.evaluations, 4);
    let explanation = w.explanation(b"attack").unwrap();
    let state = field(
        field(&explanation, b"states"),
        health.to_string().as_bytes(),
    );
    let rows = entries(field(state, b"rows"))
        .into_iter()
        .flat_map(|(_, page)| entries(page))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    for (_, row) in rows {
        let actor = projected_referent_value_v1(field(row, b"subject"))
            .unwrap()
            .unwrap();
        assert!(actors.contains(&actor));
        assert_eq!(
            field(row, b"before").as_atom().unwrap().canonical_payload(),
            100f64.to_bits().to_le_bytes()
        );
        assert_eq!(
            field(row, b"after").as_atom().unwrap().canonical_payload(),
            90f64.to_bits().to_le_bytes()
        );
    }
    assert_eq!(w.recorded_event(b"attack").unwrap().unwrap(), &event);
    run(&mut w, b"attack", &[]);
    let next = w.recorded_event(b"attack").unwrap().unwrap();
    for values in table(&next.after[usize::from(health)]).rows().values() {
        assert_eq!(values.first(), Some(&n(80.0)));
    }
    assert!(
        w.intervene(&query).is_err(),
        "replaced historical event was accepted"
    );
}

#[test]
fn created_subjects_remain_exact_and_invalid_coordinates_reject() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    run(&mut w, b"ignite", &[n(2.0)]);
    run(&mut w, b"attack", &[]);
    let event = w.recorded_event(b"attack").unwrap().unwrap().clone();
    let remaining = slot(&w, b"effect-remaining");
    let effects = table(&event.before[usize::from(remaining)])
        .rows()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(effects.len(), 2);
    let mut query = Query {
        event: event.step.id,
        allowed: vec![Change {
            slot: remaining,
            subject: Some(effects[0].clone()),
            value: n(10.0),
        }],
        desired: above(remaining, &effects[0], 9.0),
        maximum_evaluations: 2,
    };
    assert_eq!(
        decode_executable_intervention_query_v1(
            &encode_executable_intervention_query_v1(&query).unwrap()
        )
        .unwrap(),
        query
    );
    let answer = w.intervene(&query).unwrap();
    assert_eq!(answer.solution.as_ref().unwrap().len(), 1);
    let prediction = table(&answer.predicted.as_ref().unwrap()[usize::from(remaining)]);
    assert_eq!(prediction.rows()[&effects[0]].first(), Some(&n(10.0)));
    assert_eq!(prediction.rows()[&effects[1]].first(), Some(&n(2.0)));
    let valid = query.allowed[0].clone();
    for invalid in [
        Change {
            subject: Some(Referent::created(effects[0].domain(), [0; 32])),
            ..valid.clone()
        },
        Change {
            subject: Some(Referent::created(effects[0].domain() + 1, [0; 32])),
            ..valid.clone()
        },
        Change {
            slot: u16::MAX,
            ..valid.clone()
        },
        Change {
            value: V::Boolean(false),
            ..valid.clone()
        },
        Change {
            value: V::Number(f64::NAN.to_bits()),
            ..valid.clone()
        },
    ] {
        query.allowed = vec![invalid];
        assert!(w.intervene(&query).is_err());
    }
    query.allowed = vec![
        valid,
        Change {
            slot: remaining,
            subject: None,
            value: event.before[usize::from(remaining)]
                .value()
                .unwrap()
                .clone(),
        },
    ];
    assert!(
        w.intervene(&query).is_err(),
        "overlapping table/row coordinates accepted"
    );
    assert_eq!(w.recorded_event(b"attack").unwrap().unwrap(), &event);
    let effect = w
        .scalar_effects()
        .unwrap()
        .into_iter()
        .find(|effect| effect.expression == b"0.0 - 10.0")
        .unwrap();
    w.edit_scalar_effect(w.generation().handle, &effect, b"0.0 - 11.0")
        .unwrap();
    query.allowed.clear();
    assert!(
        w.intervene(&query).is_err(),
        "old-generation event was accepted"
    );
}

#[test]
fn row_changes_do_not_reinterpret_many_valued_rows_as_scalars() {
    let source = std::str::from_utf8(SOURCE).unwrap().replace(
        "mode given actor yields value: one\nrelation burn-target",
        "mode given actor yields value: many\nrelation burn-target",
    );
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    run(&mut w, b"attack", &[]);
    let event = w.recorded_event(b"attack").unwrap().unwrap();
    let selected = slot(&w, b"selected");
    let subject = table(&event.before[usize::from(selected)])
        .rows()
        .keys()
        .next()
        .unwrap()
        .clone();
    let query = Query {
        event: event.step.id,
        allowed: vec![Change {
            slot: selected,
            subject: Some(subject),
            value: V::Boolean(false),
        }],
        desired: E::Constant(V::Boolean(true)),
        maximum_evaluations: 2,
    };
    assert!(w.intervene(&query).is_err());
}
