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
