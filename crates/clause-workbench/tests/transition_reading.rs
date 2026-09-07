use clause_package::{Term, read_canonical_source_v1, print_canonical_source_v1};
use clause_runtime::projected_relation_table_v1;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/transition-reading.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut node = term;
    loop {
        let [name, value, rest] = node.as_triple().expect("projected object").slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        node = rest;
    }
}

fn execute(source: &str) -> f64 {
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).expect("checked reading");
    let event = workbench.handler_occurrence(b"add", &[]).unwrap();
    workbench.run_occurrences_to_candidate(&[event]).unwrap();
    workbench.admit().unwrap();
    let after = workbench.project_current_world().unwrap();
    let table = projected_relation_table_v1(field(field(&after, b"relations"), b"amount")).unwrap().unwrap();
    table.rows().values().flatten().next().unwrap().as_number().unwrap()
}

#[test]
fn declared_transition_executes_checked_bindings_and_one_source_rule() {
    assert_eq!(execute(SOURCE), 5.0);
    let edited = SOURCE.replace("?prior + ?increment", "?prior + ?increment * 2.0");
    assert_eq!(execute(&edited), 8.0);
    let wrong_type = SOURCE.replace("gains 3.0", "gains true");
    assert!(ResidentSourceWorkbenchV1::open(wrong_type.as_bytes()).is_err());
    let unbound_effect = SOURCE.replace("?prior + ?increment", "?missing + ?increment");
    assert!(ResidentSourceWorkbenchV1::open(unbound_effect.as_bytes()).is_err());
    let cst = read_canonical_source_v1(SOURCE.as_bytes()).unwrap();
    let printed = print_canonical_source_v1(&cst).unwrap();
    assert_eq!(execute(std::str::from_utf8(&printed).unwrap()), 5.0);
}
