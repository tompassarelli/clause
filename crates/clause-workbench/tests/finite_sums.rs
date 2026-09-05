use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableValueV1 as V, projected_relation_table_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/finite-sums.clause");

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

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8], arguments: &[V]) -> Term {
    let occurrence = w.handler_occurrence(name, arguments).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
}

fn result(frame: &Term, key: &[u8]) -> f64 {
    let table = projected_relation_table_v1(field(field(frame, b"relations"), key))
        .unwrap().unwrap();
    table.rows().values().flatten().next().unwrap().as_number().unwrap()
}

#[test]
fn sums_filter_exact_occurrences_include_created_rows_and_handle_empty_sets() {
    let mut workbench = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    let first = run(&mut workbench, b"measure", &[]);
    assert_eq!(result(&first, b"count"), 1.0);
    assert_eq!(result(&first, b"total"), -2.0);
    run(&mut workbench, b"enable-all", &[]);
    let both = run(&mut workbench, b"measure", &[]);
    assert_eq!(result(&both, b"count"), 2.0);
    assert_eq!(result(&both, b"total"), 0.0);
    run(&mut workbench, b"create-item", &[V::number(2.0).unwrap()]);
    let created = run(&mut workbench, b"measure", &[]);
    assert_eq!(result(&created, b"count"), 3.0);
    assert_eq!(result(&created, b"total"), 2.0);
    run(&mut workbench, b"clear", &[]);
    let empty = run(&mut workbench, b"measure", &[]);
    assert_eq!(result(&empty, b"count"), 0.0);
    assert_eq!(result(&empty, b"total"), 0.0);
}

#[test]
fn sums_reject_nonnumeric_values_free_variables_and_colliding_bindings() {
    let source = std::str::from_utf8(SOURCE).unwrap();
    for invalid in [
        source.replace("sum ?value where", "sum true where"),
        source.replace("sum ?value where", "sum ?prior-total where"),
        source.replace("as ?total", "as ?prior-total"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
    }
}

#[test]
fn exhausted_sum_returns_no_prefix_and_does_not_change_the_world() {
    let mut source = std::str::from_utf8(SOURCE).unwrap().to_owned();
    source.push_str("\non exhaust ?report\n  when\n    ?report total ?prior\n    sum 1.0 where { ?a amount ?av; ?b amount ?bv; ?c amount ?cv } as ?sum\n  withdraw\n    ?report total ?prior\n  include\n    ?report total ?sum\n");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    for _ in 0..17 {
        run(&mut w, b"create-item", &[V::number(2.0).unwrap()]);
    }
    let occurrence = w.handler_occurrence(b"exhaust", &[]).unwrap();
    let error = w.run_occurrences_to_candidate(&[occurrence]).unwrap_err().to_string();
    assert!(error.contains("ResourceLimit"), "{error}");
    assert!(w.recorded_event(b"exhaust").unwrap().is_none());
    let frame = run(&mut w, b"measure", &[]);
    assert_eq!(result(&frame, b"count"), 18.0);
    assert_eq!(result(&frame, b"total"), 32.0);
}

const INPUT_SOURCE: &str = include_str!("../../../test-vectors/authoring/query-inputs.clause");

fn availability(frame: &Term) -> Vec<bool> {
    let table = projected_relation_table_v1(field(field(frame, b"relations"), b"available"))
        .unwrap().unwrap();
    table.rows().values().flatten().map(|value| *value == V::Boolean(true)).collect()
}

#[test]
fn explicit_query_inputs_distinguish_absence_and_created_referents() {
    let mut w = ResidentSourceWorkbenchV1::open(INPUT_SOURCE.as_bytes()).unwrap();
    let first = availability(&run(&mut w, b"inspect", &[]));
    assert_eq!(first.iter().filter(|value| **value).count(), 1);
    assert_eq!(first.len(), 2);
    run(&mut w, b"spawn", &[]);
    let created = availability(&run(&mut w, b"inspect", &[]));
    assert_eq!(created.len(), 3);
    assert_eq!(created.iter().filter(|value| **value).count(), 2);
    run(&mut w, b"deplete", &[]);
    assert!(availability(&run(&mut w, b"inspect", &[])).iter().all(|value| !value));

    let closed = INPUT_SOURCE.replace(" given ?device", "");
    let mut w = ResidentSourceWorkbenchV1::open(closed.as_bytes()).unwrap();
    assert!(availability(&run(&mut w, b"inspect", &[])).iter().all(|value| *value));
}

#[test]
fn query_inputs_carry_outer_scalars_and_handler_arguments() {
    let source = std::str::from_utf8(SOURCE).unwrap()
        .replace("sum ?value where", "sum ?value + ?prior-total given ?prior-total where");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), -2.0);
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), -4.0);

    let source = INPUT_SOURCE.replace("on inspect ?device", "on inspect ?device ?threshold")
        .replace("given ?device where", "given ?device ?threshold where")
        .replace("?charge > 0.0", "?charge > ?threshold");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    assert_eq!(availability(&run(&mut w, b"inspect", &[V::number(1.0).unwrap()])).iter().filter(|value| **value).count(), 1);
    assert!(availability(&run(&mut w, b"inspect", &[V::number(3.0).unwrap()])).iter().all(|value| !value));
}

#[test]
fn query_inputs_reject_unknown_duplicate_and_mistyped_values() {
    for source in [
        INPUT_SOURCE.replace("given ?device", "given ?device ?unknown"),
        INPUT_SOURCE.replace("given ?device", "given ?device ?device"),
        INPUT_SOURCE.replace("given ?device where { ?device charge", "given ?prior where { ?prior charge"),
        INPUT_SOURCE.replace("?charge > 0.0", "?charge > ?prior"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err());
    }
}

#[test]
fn dependent_query_results_follow_bindings_not_binder_spelling() {
    let source = std::str::from_utf8(SOURCE).unwrap()
        .replace("sum ?value where", "sum ?value * ?count given ?count where");
    for source in [source.clone(), source.replace("?total", "?aggregate")] {
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), -2.0);
        run(&mut w, b"create-item", &[V::number(5.0).unwrap()]);
        let frame = run(&mut w, b"measure", &[]);
        assert_eq!(result(&frame, b"count"), 2.0);
        assert_eq!(result(&frame, b"total"), 6.0);
    }
    let cycle = std::str::from_utf8(SOURCE).unwrap()
        .replace("sum 1.0 where", "sum ?total given ?total where")
        .replace("sum ?value where", "sum ?value * ?count given ?count where");
    assert!(ResidentSourceWorkbenchV1::open(cycle.as_bytes()).is_err());
}

#[test]
fn query_inputs_expand_law_results_without_capturing_query_locals() {
    let source = include_str!("../../../test-vectors/authoring/query-law-inputs.clause");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), 12.0);
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), 72.0);

    let chained = source.replace("double ?value as ?weight",
        "double ?value as ?first\n    double ?first as ?weight");
    let mut w = ResidentSourceWorkbenchV1::open(chained.as_bytes()).unwrap();
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), 24.0);

    let query_dependent = source.replace("double ?value as ?weight",
        "sum ?amount where { ?item amount ?amount } as ?count\n    double ?count as ?weight");
    let mut w = ResidentSourceWorkbenchV1::open(query_dependent.as_bytes()).unwrap();
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), 18.0);
}

#[test]
fn queries_compose_checked_laws_and_count_each_row_once() {
    let source = include_str!("../../../test-vectors/authoring/query-laws.clause");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), 7.0);
    let overlap = source.replace("derive positive", "derive positive\nlaw also-positive\n  if\n    ?value >= 0.0\n  then\n    magnitude of ?value as ?value\nderive also-positive");
    let mut w = ResidentSourceWorkbenchV1::open(overlap.as_bytes()).unwrap();
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), 7.0);
    let chain = source.replace("sum ?magnitude", "sum ?twice")
        .replace("as ?magnitude }", "as ?magnitude; magnitude of (0.0 - ?magnitude * 2.0) as ?twice }");
    let mut w = ResidentSourceWorkbenchV1::open(chain.as_bytes()).unwrap();
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), 14.0);
    let partial = source.replace("derive negative", "");
    let mut w = ResidentSourceWorkbenchV1::open(partial.as_bytes()).unwrap();
    assert_eq!(result(&run(&mut w, b"measure", &[]), b"total"), 4.0);
    for invalid in [
        source.replace("magnitude of ?value as ?magnitude }", "magnitude of true as ?magnitude }"),
        source.replace("magnitude of ?value as ?magnitude }", "magnitude of ?missing as ?magnitude }"),
        source.replace("as ?magnitude }", "as ?value }"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
    }
}
