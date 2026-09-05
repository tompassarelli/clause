use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::projected_relation_table_v1;
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/recursive-dependencies.clause");

fn focused_source() -> String {
    SOURCE
        .replace(
            "    ?task obstruction ?root",
            "    ?task:\n      obstruction: ?root",
        )
        .replace(
            "    ?task blocker ?root",
            "    ?task:\n      blocker: ?root",
        )
        .replace(
            "    ?task prerequisite ?prior",
            "    ?task:\n      prerequisite: ?prior",
        )
        .replace(
            "    ?prior blocker ?root",
            "    ?prior:\n      blocker: ?root",
        )
}

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
fn focused_patterns_keep_correlated_bindings_and_withdrawal() {
    let mut workbench = ResidentSourceWorkbenchV1::open(focused_source().as_bytes()).unwrap();
    let inspect = workbench.handler_occurrence(b"inspect", &[]).unwrap();
    workbench.run_occurrences_to_candidate(&[inspect]).unwrap();
    let frame =
        decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes)
            .unwrap();
    let blockers = projected_relation_table_v1(field(field(&frame, b"relations"), b"blocker"))
        .unwrap()
        .unwrap();
    assert_eq!(blockers.rows().len(), 3);
    assert!(blockers.rows().values().all(|values| values.len() == 2));
    let roots = blockers
        .rows()
        .values()
        .next()
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    for (index, root) in roots.iter().enumerate() {
        let resolve = workbench
            .handler_occurrence(b"resolve", &[root.clone()])
            .unwrap();
        workbench.run_occurrences_to_candidate(&[resolve]).unwrap();
        let frame =
            decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes)
                .unwrap();
        let remaining = projected_relation_table_v1(field(field(&frame, b"relations"), b"blocker"))
            .unwrap()
            .unwrap();
        if index == 0 {
            assert_eq!(remaining.rows().len(), 3);
            assert!(
                remaining
                    .rows()
                    .values()
                    .all(|values| values.len() == 1 && values.contains(&roots[1]))
            );
        } else {
            assert!(
                remaining.rows().is_empty(),
                "a focused cycle cannot support itself"
            );
        }
    }
}

#[test]
fn focused_children_do_not_turn_types_or_unbound_names_into_facts() {
    for invalid in [
        focused_source().replace(
            "    ?task:\n      blocker: ?root",
            "    ?task:\n      blocker: ?unknown",
        ),
        focused_source().replace(
            "    ?task:\n      obstruction: ?root",
            "    ?task: Task\n      obstruction: ?root",
        ),
        focused_source().replace("      obstruction: ?root", "        obstruction: ?root"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
    }
}

#[test]
fn nested_focus_keeps_the_same_join() {
    let source = focused_source().replace(
        "    ?task:\n      prerequisite: ?prior\n    ?prior:\n      blocker: ?root",
        "    ?task:\n      prerequisite\n        ?prior\n          blocker: ?root",
    );
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let inspect = workbench.handler_occurrence(b"inspect", &[]).unwrap();
    workbench.run_occurrences_to_candidate(&[inspect]).unwrap();
    let frame =
        decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes)
            .unwrap();
    let blockers = projected_relation_table_v1(field(field(&frame, b"relations"), b"blocker"))
        .unwrap()
        .unwrap();
    assert_eq!(blockers.rows().len(), 3);
    assert!(blockers.rows().values().all(|values| values.len() == 2));
}

#[test]
fn focused_scalar_updates_keep_editable_source_spans_and_live_state() {
    let source = include_str!("../../../test-vectors/authoring/role-contracts.clause").replace(
        "    ?device charge ?prior",
        "    ?device:\n      charge: ?prior",
    );
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let consume = workbench.handler_occurrence(b"consume", &[]).unwrap();
    workbench.run_occurrences_to_candidate(&[consume]).unwrap();
    workbench.admit().unwrap();

    let effects = workbench.scalar_effects().unwrap();
    assert_eq!(
        effects.len(),
        1,
        "focus must not hide an editable expression"
    );
    assert_eq!(effects[0].expression, b"?prior - 1.0");
    let range = &effects[0].expression_origin;
    assert_eq!(
        &source.as_bytes()[range.start as usize..range.end as usize],
        b"?prior - 1.0"
    );
    workbench
        .edit_scalar_effect(workbench.generation().handle, &effects[0], b"?prior - 0.5")
        .unwrap();
    assert_eq!(
        workbench.exact_source(),
        source.replace("?prior - 1.0", "?prior - 0.5").as_bytes()
    );
    let carried = workbench.project_current_world().unwrap();
    let carried_charge =
        projected_relation_table_v1(field(field(&carried, b"relations"), b"charge"))
            .unwrap()
            .unwrap();
    assert_eq!(
        carried_charge
            .rows()
            .values()
            .flatten()
            .next()
            .unwrap()
            .as_number(),
        Some(1.0),
        "source edit must immediately project the preserved accepted world"
    );
    let consume = workbench.handler_occurrence(b"consume", &[]).unwrap();
    workbench.run_occurrences_to_candidate(&[consume]).unwrap();
    let frame =
        decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes)
            .unwrap();
    let charge = projected_relation_table_v1(field(field(&frame, b"relations"), b"charge"))
        .unwrap()
        .unwrap();
    assert_eq!(
        charge.rows().values().flatten().next().unwrap().as_number(),
        Some(0.5),
        "the edit continues from live charge 1.0, not initial charge 2.0"
    );
}
