use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::projected_relation_table_v1;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/role-contracts.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

#[test]
fn ordinary_contract_and_value_facts_drive_typed_execution() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    for expected in [1.0, 0.0] {
        let occurrence = w.handler_occurrence(b"consume", &[]).unwrap();
        w.run_occurrences_to_candidate(&[occurrence]).unwrap();
        let frame = decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes()).unwrap();
        let table = projected_relation_table_v1(field(field(&frame, b"relations"), b"charge")).unwrap().unwrap();
        assert_eq!(table.rows().len(), 1);
        assert_eq!(table.rows().values().flatten().next().unwrap().as_number(), Some(expected));
    }
}

#[test]
fn contracts_reject_missing_contradictory_and_mistyped_facts() {
    let source = std::str::from_utf8(SOURCE).unwrap();
    for invalid in [
        source.replace("  range: F64\n", ""),
        source.replace("  range: F64", "  range: F64\n  range: Device"),
        source.replace("  charge: 2.0", "  charge: true"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
    }
}

#[test]
fn isolated_contract_vocabulary_does_not_reclassify_domain_data() {
    let source = format!("{}\nsensor\n  range: 10.0\n  range: 20.0\nwebsite\n  domain: \"example.test\"\n", std::str::from_utf8(SOURCE).unwrap());
    ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
}

#[test]
fn repeated_equal_contract_facts_keep_the_same_meaning() {
    let source = std::str::from_utf8(SOURCE).unwrap().replace("  range: F64", "  range: F64\n  range: F64");
    ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
}

#[test]
fn checked_conformance_needs_no_membership_and_claims_do_not_supply_properties() {
    let source = std::str::from_utf8(SOURCE).unwrap();
    assert!(!source.contains("member of"));
    ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let false_claim = source.replace("charge: 2.0", "shape: Device");
    assert!(ResidentSourceWorkbenchV1::open(false_claim.as_bytes()).is_err());
    let membership_only = source.replace("charge: 2.0", "member of: Device");
    ResidentSourceWorkbenchV1::open(membership_only.as_bytes()).unwrap();
    let invalid_use = format!("{membership_only}\ncharger\n  domain: Device\n  range: Device\n  cardinality: maybe\nstation\n  charge: 1.0\n  charger: lamp\n");
    assert!(ResidentSourceWorkbenchV1::open(invalid_use.as_bytes()).is_err());
}

#[test]
fn membership_requires_a_declared_group_not_a_literal_or_record() {
    let source = std::str::from_utf8(SOURCE).unwrap();
    for invalid in [
        source.replace("lamp\n", "lamp\n  member of: Missing\n"),
        source.replace("lamp\n", "lamp\n  member of: 1.0\n"),
        source.replacen("Device\n", "Device:\n  ?example:\n    serial: F64\n", 1),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
    }
}
