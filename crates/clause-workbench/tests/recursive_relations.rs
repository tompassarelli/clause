use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableValueV1 as V, projected_relation_table_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/recursive-dependencies.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().expect("projected object").slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8], arguments: &[V]) -> Term {
    let occurrence = w.handler_occurrence(name, arguments).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes()).unwrap()
}

fn blocked(frame: &Term) -> clause_runtime::ExecutableRelationTableV1 {
    projected_relation_table_v1(field(field(frame, b"relations"), b"blocker")).unwrap().unwrap()
}

#[test]
fn recursive_dependencies_withdraw_last_root_without_self_support() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    let first = run(&mut w, b"inspect", &[]);
    let first = blocked(&first);
    assert_eq!(first.rows().len(), 3);
    assert!(first.rows().values().all(|values| values.len() == 2));
    let roots = first.rows().values().next().unwrap().iter().cloned().collect::<Vec<_>>();
    let second = blocked(&run(&mut w, b"resolve", &[roots[0].clone()]));
    assert_eq!(second.rows().len(), 3);
    assert!(second.rows().values().all(|values| values.len() == 1 && values.contains(&roots[1])));
    let empty = blocked(&run(&mut w, b"resolve", &[roots[1].clone()]));
    assert!(empty.rows().is_empty(), "the cycle cannot support itself");
    let restored = blocked(&run(&mut w, b"obstruct", &[roots[0].clone()]));
    assert_eq!(restored.rows().len(), 3);
    assert!(restored.rows().values().all(|values| values.len() == 1));
}

#[test]
fn equal_conclusions_from_independent_laws_are_one_value() {
    let source = format!("{SOURCE}\nlaw another-obstruction\n  if\n    ?item obstruction ?cause\n  then\n    ?item blocker ?cause\nderive another-obstruction\n");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let first = blocked(&run(&mut w, b"inspect", &[]));
    assert_eq!(first.rows().len(), 3);
    assert!(first.rows().values().all(|values| values.len() == 2));
}

#[test]
fn derived_relations_reject_external_writes_and_unbound_outputs() {
    for source in [
        format!("{SOURCE}\non forge ?task\n  when\n    ?task obstruction ?root\n  include\n    ?task blocker ?root\n"),
        SOURCE.replace("then\n    ?task blocker ?root", "then\n    ?task blocker ?unknown"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err());
    }
}
