use clause_package::Term;
use clause_runtime::{
    ExecutableExpressionV1 as E, ExecutableInterventionQueryV1 as Query,
    ExecutableReferentIdentityV1, ExecutableReferentV1 as Referent, ExecutableRelationTableV1,
    ExecutableValueV1 as V, projected_relation_table_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/ordered-measurements.clause");
fn n(value: f64) -> V {
    V::number(value).unwrap()
}
fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().expect("projected object").slots();
        if name.as_atom().unwrap().canonical_payload() == key {
            return value;
        }
        current = rest;
    }
}
fn table(frame: &Term, name: &[u8]) -> ExecutableRelationTableV1 {
    projected_relation_table_v1(field(field(frame, b"relations"), name))
        .unwrap()
        .unwrap()
}
fn value(table: &ExecutableRelationTableV1, subject: &Referent) -> V {
    table.rows()[subject].first().unwrap().clone()
}
fn ordered(frame: &Term) -> Vec<(f64, Referent, f64)> {
    let positions = table(frame, b"position");
    let readings = table(frame, b"reading");
    let batches = table(frame, b"batch");
    let active = table(frame, b"active");
    let mut result = positions
        .rows()
        .keys()
        .filter(|subject| {
            let batch = value(&batches, subject).as_referent().unwrap().clone();
            value(&active, &batch) == V::Boolean(true)
        })
        .map(|subject| {
            (
                value(&positions, subject).as_number().unwrap(),
                subject.clone(),
                value(&readings, subject).as_number().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    result.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    result
}
fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8], args: &[V]) {
    let occurrence = w.handler_occurrence(name, args).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    w.admit().unwrap();
}

#[test]
fn ordered_occurrences_transform_once_and_explain_the_same_checked_meaning() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    let before = w.project_current_world().unwrap();
    assert_eq!(
        ordered(&before)
            .iter()
            .map(|row| (row.0, row.2))
            .collect::<Vec<_>>(),
        vec![(0.0, 4.0), (1.0, 4.0), (2.0, 9.0)]
    );
    run(&mut w, b"calibrate", &[n(2.0)]);
    let after = w.project_current_world().unwrap();
    let rows = ordered(&after);
    assert_eq!(
        rows.iter().map(|row| (row.0, row.2)).collect::<Vec<_>>(),
        vec![(0.0, 1.0), (1.0, 1.0), (2.0, 1.5)]
    );
    assert_eq!(table(&before, b"position"), table(&after, b"position"));
    assert_eq!(table(&before, b"batch"), table(&after, b"batch"));
    assert_eq!(field(&before, b"saved"), field(&after, b"saved"));
    assert_eq!(
        ordered(&before)
            .iter()
            .map(|row| &row.1)
            .collect::<Vec<_>>(),
        rows.iter().map(|row| &row.1).collect::<Vec<_>>()
    );
    let event = w.recorded_event(b"calibrate").unwrap().unwrap().clone();
    let effects = event
        .trace
        .rules
        .iter()
        .filter(|rule| rule.selected)
        .flat_map(|rule| &rule.effects)
        .collect::<Vec<_>>();
    assert_eq!(effects.len(), 3);
    assert!(effects.iter().all(|effect| effect.evaluated.is_some()));
    let explanation = w.explanation(b"calibrate").unwrap();
    assert_eq!(
        table(field(&explanation, b"before-projection"), b"reading"),
        table(&before, b"reading")
    );
    assert_eq!(
        table(field(&explanation, b"after-projection"), b"reading"),
        table(&after, b"reading")
    );
    let slot = w
        .state_bindings()
        .unwrap()
        .into_iter()
        .find(|binding| binding.state.relation_designation == b"reading")
        .unwrap()
        .slot;
    let query = Query {
        event: event.step.id,
        allowed: vec![],
        desired: E::Equal(
            Box::new(E::RelationRead(
                Box::new(E::Slot(slot)),
                Box::new(E::Constant(V::Referent(rows[0].1.clone()))),
            )),
            Box::new(E::Constant(n(1.0))),
        ),
        maximum_evaluations: 1,
    };
    let result = w.intervene(&query).unwrap();
    assert!(!result.completed && !result.exhausted);
    assert_eq!(result.evaluations, 1);
    assert_eq!(result.solution, Some(vec![]));
    assert_eq!(result.predicted.as_ref(), Some(&event.after));
    assert_eq!(w.project_current_world().unwrap(), after);
}

#[test]
fn created_duplicate_occurrences_keep_identity_and_failures_are_atomic() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    run(&mut w, b"append", &[n(3.0), n(4.0)]);
    run(&mut w, b"append", &[n(4.0), n(4.0)]);
    let before = w.project_current_world().unwrap();
    let identities = ordered(&before)
        .into_iter()
        .map(|row| row.1)
        .collect::<Vec<_>>();
    assert_eq!(identities.len(), 5);
    assert_eq!(
        identities
            .iter()
            .filter(|r| matches!(r.identity(), ExecutableReferentIdentityV1::Created(_)))
            .count(),
        2
    );
    for divisor in [0.0, -0.0] {
        let occurrence = w.handler_occurrence(b"calibrate", &[n(divisor)]).unwrap();
        let error = w
            .run_occurrences_to_candidate(&[occurrence])
            .unwrap_err()
            .to_string();
        assert!(error.contains("NumericDomain"), "{error}");
        assert!(w.pending_candidate().is_none());
        assert!(w.recorded_event(b"calibrate").unwrap().is_none());
        assert_eq!(w.project_current_world().unwrap(), before);
    }
    run(&mut w, b"calibrate", &[n(2.0)]);
    let after = w.project_current_world().unwrap();
    assert_eq!(
        ordered(&after).iter().map(|row| &row.1).collect::<Vec<_>>(),
        identities.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        ordered(&after).iter().map(|row| row.2).collect::<Vec<_>>(),
        vec![1.0, 1.0, 1.5, 1.0, 1.0]
    );
    let negative = SOURCE.replace("reading: 9.0", "reading: -1.0");
    let mut w = ResidentSourceWorkbenchV1::open(negative.as_bytes()).unwrap();
    let before = w.project_current_world().unwrap();
    let occurrence = w.handler_occurrence(b"calibrate", &[n(1.0)]).unwrap();
    assert!(
        w.run_occurrences_to_candidate(&[occurrence])
            .unwrap_err()
            .to_string()
            .contains("NumericDomain")
    );
    assert_eq!(w.project_current_world().unwrap(), before);
    assert!(w.recorded_event(b"calibrate").unwrap().is_none());
}
