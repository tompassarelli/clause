use clause_package::{FormationLocalId, Term};
use clause_runtime::{
    check_executable_source_edit_v1, decode_executable_scalar_edit_transaction_v1,
    encode_executable_source_edit_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] =
    include_bytes!("../../../test-vectors/authoring/structured-field-edit.clause");

fn field<'a>(term: &'a Term, name: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [key, value, rest] = current.as_triple().unwrap().slots();
        if key.as_atom().unwrap().canonical_payload() == name {
            return value;
        }
        current = rest;
    }
}

fn entries(term: &Term) -> Vec<&Term> {
    let mut current = term;
    let mut result = vec![];
    while let Some(triple) = current.as_triple() {
        let [_, value, rest] = triple.slots();
        result.push(value);
        current = rest;
    }
    result
}

fn number(term: &Term) -> f64 {
    f64::from_bits(u64::from_le_bytes(
        term.as_atom()
            .unwrap()
            .canonical_payload()
            .try_into()
            .unwrap(),
    ))
}

fn reading(w: &ResidentSourceWorkbenchV1) -> [f64; 2] {
    let frame = w.project_current_world().unwrap();
    let reading = field(field(&frame, b"counter"), b"reading");
    [
        number(field(reading, b"first")),
        number(field(reading, b"second")),
    ]
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8]) {
    let occurrence = w.handler_occurrence(name, &[]).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    w.admit().unwrap();
}

fn mapping(continuity: &Term, old: FormationLocalId) -> &Term {
    entries(field(continuity, b"formations"))
        .into_iter()
        .flat_map(entries)
        .find(|value| number(field(value, b"old")) == f64::from(old.get()))
        .unwrap()
}

#[test]
fn checked_field_edit_preserves_equal_siblings_occurrences_and_live_world() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE).unwrap();
    let fields = w.scalar_effects().unwrap();
    assert_eq!(fields.len(), 4);
    assert!(
        fields
            .iter()
            .all(|field| field.expression == b"1.0" && field.field_path.len() == 1)
    );
    let selected = fields[0].clone();
    let sibling = fields[1].clone();
    let peer = fields[2].clone();
    assert_eq!(selected.effect, sibling.effect);
    assert_ne!(selected.field_path, sibling.field_path);
    assert_eq!(selected.field_path, peer.field_path);
    assert_ne!(selected.effect, peer.effect);
    assert_eq!(
        &SOURCE[selected.expression_origin.start as usize..selected.expression_origin.end as usize],
        &SOURCE[peer.expression_origin.start as usize..peer.expression_origin.end as usize]
    );
    run(&mut w, b"set");
    assert_eq!(reading(&w), [1.0, 1.0]);

    let old = w.generation().clone();
    let preparation = w.source_preparation().unwrap();
    let world = w.project_current_world().unwrap();
    assert_eq!(
        w.edit_scalar_effect(old.handle, &selected, b"1.0").unwrap(),
        old
    );
    for replacement in [b"true".as_slice(), b"1.0\non injected", b"?missing"] {
        assert!(
            w.edit_scalar_effect(old.handle, &selected, replacement)
                .is_err()
        );
        assert_eq!(w.generation(), &old);
        assert_eq!(w.exact_source(), SOURCE);
        assert_eq!(w.project_current_world().unwrap(), world);
    }
    let mut invalid = selected.clone();
    invalid.field_path = vec![selected.handler];
    assert!(w.edit_scalar_effect(old.handle, &invalid, b"7.0").is_err());
    assert_eq!(w.project_current_world().unwrap(), world);

    w.edit_scalar_effect(old.handle, &selected, b"7.0").unwrap();
    assert_eq!(
        reading(&w),
        [1.0, 1.0],
        "editing a future effect must not reset or run the world"
    );
    let transaction = decode_executable_scalar_edit_transaction_v1(w.last_source_edit().unwrap()).unwrap();
    assert_eq!(clause_runtime::decode_executable_scalar_edit_transaction_v1(&clause_runtime::encode_executable_scalar_edit_transaction_v1(&transaction).unwrap()).unwrap(), transaction);
    let witness = clause_runtime::ExecutableSourceEditV1 {
        old_source: SOURCE.to_vec(),
        declared_frontend: clause_package::DECLARED_FOCUSED_FRONTEND_SOURCE_V1.to_vec(),
        old_root: clause_package::ProgramChangeOccurrenceId::from_bytes(preparation[4..36].try_into().unwrap()),
        new_root: transaction.new_root, operation: transaction.operation,
        old_cpp1: old.cpp1.clone(), new_cpp1: w.generation().cpp1.clone(),
    };
    let clause_runtime::ExecutableSourceOperationV1::ScalarEffect { field_path, .. } =
        &witness.operation
    else {
        panic!("expected scalar operation")
    };
    assert_eq!(*field_path, selected.field_path);
    assert_eq!(
        clause_runtime::decode_executable_source_edit_v1(&encode_executable_source_edit_v1(&witness).unwrap())
            .unwrap(),
        witness
    );
    let scope = world.scope();
    check_executable_source_edit_v1(&witness, scope).unwrap();
    let mut forged = witness.clone();
    let clause_runtime::ExecutableSourceOperationV1::ScalarEffect {
        field_path, effect, ..
    } = &mut forged.operation
    else {
        panic!("expected scalar operation")
    };
    *field_path = peer.field_path.clone();
    *effect = peer.effect;
    assert!(
        check_executable_source_edit_v1(&forged, scope).is_err(),
        "equal source text does not select another occurrence"
    );
    forged = witness;
    let clause_runtime::ExecutableSourceOperationV1::ScalarEffect { field_path, .. } =
        &mut forged.operation
    else {
        panic!("expected scalar operation")
    };
    *field_path = sibling.field_path.clone();
    assert!(
        check_executable_source_edit_v1(&forged, scope).is_err(),
        "equal sibling value does not select another field"
    );

    let continuity = w.source_continuity().unwrap();
    let offered = w.scalar_effects().unwrap();
    let migrate = |old: FormationLocalId| {
        FormationLocalId::new(number(field(mapping(&continuity, old), b"new")) as u32)
    };
    for old_field in [&sibling, &peer] {
        let continuing = offered
            .iter()
            .find(|new| {
                new.effect == migrate(old_field.effect)
                    && new.field_path
                        == old_field
                            .field_path
                            .iter()
                            .copied()
                            .map(migrate)
                            .collect::<Vec<_>>()
            })
            .unwrap();
        assert_eq!(continuing.expression, b"1.0");
    }
    let accepted = w.generation().clone();
    let accepted_world = w.project_current_world().unwrap();
    assert!(w.rejects_stale_handle(old.handle).unwrap());
    assert!(w.edit_scalar_effect(old.handle, &selected, b"9.0").is_err());
    assert!(
        w.edit_scalar_effect(accepted.handle, &selected, b"9.0")
            .is_err()
    );
    assert_eq!(w.generation(), &accepted);
    assert_eq!(w.project_current_world().unwrap(), accepted_world);
    run(&mut w, b"set");
    assert_eq!(reading(&w), [7.0, 1.0]);

    let next_selected = offered
        .iter()
        .find(|field| field.expression == b"7.0")
        .unwrap();
    w.edit_scalar_effect(w.generation().handle, next_selected, b"8.0")
        .unwrap();
    let next_continuity = w.source_continuity().unwrap();
    for old_id in [sibling.effect, sibling.field_path[0], peer.effect] {
        let first = mapping(&continuity, old_id);
        let next = mapping(&next_continuity, migrate(old_id));
        assert_eq!(field(first, b"occurrence"), field(next, b"occurrence"));
        assert_eq!(
            field(first, b"occurrence-snapshot"),
            field(next, b"occurrence-snapshot")
        );
        assert_eq!(
            field(first, b"occurrence-coordinate"),
            field(next, b"occurrence-coordinate")
        );
    }
    run(&mut w, b"peer");
    assert_eq!(
        reading(&w),
        [1.0, 1.0],
        "unaffected equal-text handler remains usable"
    );
}
