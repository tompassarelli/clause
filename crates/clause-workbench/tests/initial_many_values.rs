use clause_package::{Term, decode_canonical_term_bytes};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/initial-many-values.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key {
            return value;
        }
        current = rest;
    }
}

#[test]
fn initial_many_values_share_storage_and_retain_distinct_members() {
    for source in [
        SOURCE.to_owned(),
        SOURCE.replace("root known second", "root known second\nroot known first"),
    ] {
        let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        let occurrence = workbench.handler_occurrence(b"inspect", &[]).unwrap();
        workbench.run_occurrences_to_candidate(&[occurrence]).unwrap();
        let frame = decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes).unwrap();
        let count = field(field(&frame, b"root"), b"count").as_atom().unwrap().canonical_payload();
        assert_eq!(f64::from_bits(u64::from_le_bytes(count.try_into().unwrap())), 2.0);
    }
}

#[test]
fn conflicting_initial_single_values_remain_invalid() {
    let source = SOURCE.replace("yields value: many", "yields value: one");
    assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err());
}
