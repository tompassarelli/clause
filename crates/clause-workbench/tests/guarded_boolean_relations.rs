use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableValueV1 as V, projected_relation_table_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/guarded-powered-network.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8], arguments: &[V]) -> usize {
    let event = w.handler_occurrence(name, arguments).unwrap();
    w.run_occurrences_to_candidate(&[event]).unwrap();
    let frame = decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes()).unwrap();
    let powered = projected_relation_table_v1(field(field(&frame, b"relations"), b"powered"))
        .unwrap().unwrap();
    assert!(powered.rows().values().all(|values| values.len() == 1 && values.contains(&V::Boolean(true))));
    powered.rows().len()
}

#[test]
fn guarded_boolean_closure_retracts_cycles_when_the_last_live_source_disappears() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    assert_eq!(run(&mut w, b"inspect", &[]), 3);
    for (handler, absent, restored) in [
        (b"set-generation".as_slice(), V::number(0.0).unwrap(), V::number(12.0).unwrap()),
        (b"set-health".as_slice(), V::number(0.0).unwrap(), V::number(100.0).unwrap()),
        (b"set-mounted".as_slice(), V::Boolean(false), V::Boolean(true)),
    ] {
        assert_eq!(run(&mut w, handler, &[absent]), 0, "unsupported cycles retract without false rows");
        assert_eq!(run(&mut w, handler, &[restored]), 3);
    }
    assert_eq!(run(&mut w, b"disconnect", &[]), 1);
}

#[test]
fn many_boolean_conclusions_share_equal_proofs() {
    let source = SOURCE.replace("range: Bool\n  cardinality: maybe", "range: Bool\n  cardinality: many");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    assert_eq!(run(&mut w, b"inspect", &[]), 3);
    assert_eq!(run(&mut w, b"set-mounted", &[V::Boolean(false)]), 0);
}

#[test]
fn sums_match_optional_boolean_conclusions_and_retract_their_contributions() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    for (mounted, expected) in [(true, 12.0), (false, 0.0), (true, 12.0)] {
        run(&mut w, b"set-mounted", &[V::Boolean(mounted)]);
        let event = w.handler_occurrence(b"measure", &[]).unwrap();
        w.run_occurrences_to_candidate(&[event]).unwrap();
        let frame = decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes()).unwrap();
        let total = f64::from_le_bytes(field(field(&frame, b"report"), b"total")
            .as_atom().unwrap().canonical_payload().try_into().unwrap());
        assert_eq!(total, expected);
    }
}
