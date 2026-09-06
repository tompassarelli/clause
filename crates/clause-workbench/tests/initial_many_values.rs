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
    let source = SOURCE.replace("yields known: many", "yields known: one");
    assert_ne!(source, SOURCE);
    assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err());
}

#[test]
fn static_catalog_projects_all_rows_without_a_reader_handler() {
    let source = include_str!("../../../test-vectors/authoring/static-catalog.clause");
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let occurrence = workbench.handler_occurrence(b"tick", &[]).unwrap();
    workbench.run_occurrences_to_candidate(&[occurrence]).unwrap();
    let frame = decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes).unwrap();
    let relations = field(&frame, b"relations");
    let known = clause_runtime::projected_relation_table_v1(field(relations, b"known")).unwrap().unwrap();
    let members = known.rows().values().flatten().collect::<Vec<_>>();
    assert_eq!(members.len(), 2);
    for (name, label) in [(b"first".as_slice(), "First"), (b"second", "Second")] {
        let item = field(&frame, name);
        let identity = clause_runtime::projected_referent_value_v1(field(item, b"$referent")).unwrap().unwrap();
        assert!(members.iter().any(|value| value.as_referent() == Some(&identity)));
        assert_eq!(clause_runtime::projected_text_value_v1(field(item, b"label")).unwrap(), Some(label));
    }
}
