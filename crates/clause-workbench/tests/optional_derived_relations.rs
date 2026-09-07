use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/optional-derived-formation.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8], arguments: &[V]) {
    let event = w.handler_occurrence(name, arguments).unwrap();
    w.run_occurrences_to_candidate(&[event]).unwrap();
    w.admit().unwrap();
}

fn total(w: &mut ResidentSourceWorkbenchV1) -> f64 {
    let event = w.handler_occurrence(b"measure", &[]).unwrap();
    w.run_occurrences_to_candidate(&[event]).unwrap();
    let frame = decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes()).unwrap();
    f64::from_bits(u64::from_le_bytes(field(field(&frame, b"report"), b"total")
        .as_atom().unwrap().canonical_payload().try_into().unwrap()))
}

#[test]
fn optional_structured_derivations_follow_selection_liveness_and_health() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    assert_eq!(total(&mut w), 4.0, "equal-valued subjects each contribute once");
    run(&mut w, b"select", &[V::Boolean(false)]);
    assert_eq!(total(&mut w), 0.0);
    run(&mut w, b"select", &[V::Boolean(true)]);
    assert_eq!(total(&mut w), 4.0);
    run(&mut w, b"vitality", &[V::number(0.0).unwrap()]);
    assert_eq!(total(&mut w), 0.0);
    run(&mut w, b"vitality", &[V::number(1.0).unwrap()]);
    run(&mut w, b"living", &[V::Boolean(false)]);
    assert_eq!(total(&mut w), 0.0);
    run(&mut w, b"living", &[V::Boolean(true)]);
    assert_eq!(total(&mut w), 4.0);
}

#[test]
fn optional_equal_proofs_share_one_value_and_conflicts_reject() {
    let law = "\nlaw another-formation\n  if\n    ?item selected true\n    ?item offset ?offset\n  then\n    ?item formation ?offset\nderive another-formation\n";
    let mut w = ResidentSourceWorkbenchV1::open(format!("{SOURCE}{law}").as_bytes()).unwrap();
    assert_eq!(total(&mut w), 4.0);
    let conflicting = "\nlaw conflicting-formation\n  if\n    ?item selected true\n  then\n    ?item:\n      formation:\n        x: 8.0\n        z: 9.0\nderive conflicting-formation\n";
    let equal = conflicting.replace("x: 8.0\n        z: 9.0", "x: 2.0\n        z: 3.0");
    let mut w = ResidentSourceWorkbenchV1::open(format!("{SOURCE}{equal}").as_bytes()).unwrap();
    assert_eq!(total(&mut w), 4.0);
    let error = ResidentSourceWorkbenchV1::open(format!("{SOURCE}{conflicting}").as_bytes()).err().unwrap();
    assert!(format!("{error:?}").contains("ProcessRejected"), "{error:?}");
}
