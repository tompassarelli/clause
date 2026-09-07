use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableValueV1 as V, projected_relation_table_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/recursive-dependencies.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

fn inspect(workbench: &mut ResidentSourceWorkbenchV1) -> Term {
    let event = workbench.handler_occurrence(b"inspect", &[]).unwrap();
    workbench.run_occurrences_to_candidate(&[event]).unwrap();
    decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes()).unwrap()
}

#[test]
fn role_contracts_check_participation_without_a_nominal_roster() {
    assert!(!SOURCE.contains("member of"));
    let mut workbench = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    let frame = inspect(&mut workbench);
    let blockers = projected_relation_table_v1(field(field(&frame, b"relations"), b"blocker")).unwrap().unwrap();
    assert_eq!(blockers.rows().len(), 3);
    assert!(blockers.rows().values().all(|values| values.len() == 2));
}

#[test]
fn missing_mistyped_and_wrong_target_properties_reject() {
    for invalid in [
        SOURCE.replace("  duration: 3.0\n", ""),
        SOURCE.replace("  duration: 3.0", "  duration: true"),
        SOURCE.replace("  prerequisite: design", "  prerequisite: approval"),
        SOURCE.replace("  reason: \"Build material is unavailable\"", "  reason: 3.0"),
        SOURCE.replace("  duration: 3.0", "  member of: Task"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err(), "{invalid}");
    }
}

#[test]
fn atomic_updates_cannot_remove_a_required_property_of_a_participant() {
    let source = format!("{SOURCE}\non lose-duration ?task\n  when\n    ?task duration ?duration\n  withdraw\n    ?task duration ?duration\n");
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let before = inspect(&mut workbench);
    let event = workbench.handler_occurrence(b"lose-duration", &[]).unwrap();
    assert!(workbench.run_occurrences_to_candidate(&[event]).is_err());
    let after = inspect(&mut workbench);
    assert_eq!(field(&before, b"relations"), field(&after, b"relations"));
}

const ADD_TASK: &str = "\non add-task ?prior ?chosen ?duration\n  when\n    ?prior duration ?old-duration\n    ?prior = ?chosen\n  create\n    ?task\n  include\n    ?task duration ?duration\n    ?task prerequisite ?prior\n";

#[test]
fn creation_infers_the_domain_and_checks_the_complete_new_participant() {
    let mut workbench = ResidentSourceWorkbenchV1::open(format!("{SOURCE}{ADD_TASK}").as_bytes()).unwrap();
    let before = inspect(&mut workbench);
    let durations = projected_relation_table_v1(field(field(&before, b"relations"), b"duration")).unwrap().unwrap();
    let prior = V::Referent(durations.rows().keys().next().unwrap().clone());
    let add = workbench.handler_occurrence(b"add-task", &[prior.clone(), V::number(4.0).unwrap()]).unwrap();
    workbench.run_occurrences_to_candidate(&[add]).unwrap();
    let after = decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes()).unwrap();
    let durations = projected_relation_table_v1(field(field(&after, b"relations"), b"duration")).unwrap().unwrap();
    assert_eq!(durations.rows().len(), 4);
    assert!(durations.rows().values().flatten().any(|value| value.as_number() == Some(4.0)));
    let blockers = projected_relation_table_v1(field(field(&after, b"relations"), b"blocker")).unwrap().unwrap();
    assert_eq!(blockers.rows().len(), 4);
    assert!(blockers.rows().values().all(|values| values.len() == 2));

    let invalid = format!("{SOURCE}{}", ADD_TASK.replace("    ?task duration ?duration\n", ""));
    let mut invalid = ResidentSourceWorkbenchV1::open(invalid.as_bytes()).unwrap();
    let before = inspect(&mut invalid);
    let durations = projected_relation_table_v1(field(field(&before, b"relations"), b"duration")).unwrap().unwrap();
    let prior = V::Referent(durations.rows().keys().next().unwrap().clone());
    let event = invalid.handler_occurrence(b"add-task", &[prior, V::number(4.0).unwrap()]).unwrap();
    assert!(invalid.run_occurrences_to_candidate(&[event]).is_err());
}
