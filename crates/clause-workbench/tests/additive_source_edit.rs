use clause_package::{CanonicalAllocatedIdentityV1, FormationLocalId, Term};
use clause_runtime::*;
use clause_workbench::ResidentSourceWorkbenchV1;
const BEFORE: &[u8] = include_bytes!("../../../test-vectors/authoring/additive-world-before.clause");
const ITEMS: &[u8] = include_bytes!("../../../test-vectors/authoring/additive-world-items.clause");
fn field<'a>(mut term: &'a Term, name: &[u8]) -> &'a Term {
    while let Some(triple) = term.as_triple() {
        let [key, value, rest] = triple.slots();
        if key.as_atom().unwrap().canonical_payload() == name { return value; }
        term = rest;
    }
    panic!("missing {}", String::from_utf8_lossy(name));
}
fn value<'a>(world: &'a Term, subject: &[u8], role: &[u8]) -> &'a Term { field(field(world, subject), role) }
fn number(world: &Term, role: &[u8]) -> f64 {
    f64::from_le_bytes(value(world, b"player", role).as_atom().unwrap().canonical_payload().try_into().unwrap())
}
fn input(w: &mut ResidentSourceWorkbenchV1, code: &[u8], sequence: u64) {
    w.apply_physical_input(w.generation().handle, WasmSessionPhysicalInputV1 { input_sequence: sequence,
        source: ExecutableInputSourceV1::Keyboard { code: code.to_vec(), phase: ExecutableKeyPhaseV1::Down }, value: None }).unwrap();
}
fn candidate(w: &mut ResidentSourceWorkbenchV1, revision: u64) {
    w.tick_to_candidate(WasmSessionTickV1 { configuration_revision: revision, fixed_tick_milliseconds: 16 }).unwrap();
}
#[test]
fn additive_source_preserves_progress_and_identity_then_inputs_admit_and_reopen() {
    let mut w = ResidentSourceWorkbenchV1::open(BEFORE).unwrap();
    input(&mut w, b"Travel", 1); candidate(&mut w, 1); w.admit().unwrap();
    let before = w.project_current_world().unwrap();
    assert_eq!(number(&before, b"health"), 92.875);
    assert_eq!(number(&before, b"stock"), 21.0);
    assert_eq!(number(&before, b"position"), 0.3);
    let generation = w.generation().clone();
    let checkpoint = w.checkpoint_admitted().unwrap();
    let mut w = ResidentSourceWorkbenchV1::reopen(BEFORE, &checkpoint).unwrap();
    let handle = w.generation().handle;
    candidate(&mut w, 2);
    assert!(w.append_source_items(handle, ITEMS).is_err());
    assert_eq!(w.exact_source(), BEFORE);
    assert_eq!(w.project_current_world().unwrap(), before);
    w.admit().unwrap();
    for invalid in [b"player\n  health: 1.0\n".as_slice(), b"health\n  domain: Player\n  range: F64\n  cardinality: one\n", b"player\n  health: \"invalid\"\n"] {
        assert!(w.append_source_items(handle, invalid).is_err(), "accepted {}", String::from_utf8_lossy(invalid));
        assert_eq!(w.exact_source(), BEFORE);
        assert_eq!(w.project_current_world().unwrap(), before);
        assert_eq!(w.generation().cpp1, generation.cpp1);
    }
    w.append_source_items(handle, ITEMS).unwrap();
    assert!(w.append_source_items(handle, ITEMS).is_err());
    let after = w.project_current_world().unwrap();
    for role in [b"health".as_slice(), b"stock", b"position"] {
        assert_eq!(value(&before,b"player",role), value(&after,b"player",role));
    }
    assert_eq!(number(&after,b"potions"),2.0);
    assert!(value(&after,b"apothecary",b"place-name").as_atom().is_some());
    let witness = decode_executable_source_edit_v1(w.last_source_edit().unwrap()).unwrap();
    assert_eq!(encode_executable_source_edit_v1(&witness).unwrap(),w.last_source_edit().unwrap());
    let checked = check_executable_source_edit_v1(&witness,after.scope()).unwrap();
    let old = projected_referent_value_v1(value(&before,b"player",b"home")).unwrap().unwrap();
    let new = projected_referent_value_v1(value(&after,b"player",b"home")).unwrap().unwrap();
    let ExecutableReferentIdentityV1::Declared(old_id) = old.identity() else { panic!() };
    let ExecutableReferentIdentityV1::Declared(new_id) = new.identity() else { panic!() };
    for (old,new) in [(old.domain(),new.domain()),(*old_id,*new_id)] {
        assert_eq!(checked.continuity().identities.get(&CanonicalAllocatedIdentityV1::Formation(FormationLocalId::new(old))),
            Some(&CanonicalAllocatedIdentityV1::Formation(FormationLocalId::new(new))));
    }
    input(&mut w,b"Drink",1);
    assert_eq!(w.project_current_world().unwrap(),after);
    candidate(&mut w,1); w.admit().unwrap();
    let admitted=w.project_current_world().unwrap();
    assert_eq!(number(&admitted,b"health"),102.875);
    assert_eq!(number(&admitted,b"potions"),1.0);
    let saved=w.checkpoint_admitted().unwrap();
    let mut reopened=ResidentSourceWorkbenchV1::reopen(w.exact_source(),&saved).unwrap();
    assert_eq!(reopened.project_current_world().unwrap(),admitted);
    let effect=reopened.scalar_effects().unwrap().into_iter().find(|effect|effect.expression==b"?health + 10.0").unwrap();
    reopened.edit_scalar_effect(reopened.generation().handle,&effect,b"?health + 20.0").unwrap();
    assert_eq!(number(&reopened.project_current_world().unwrap(),b"health"),102.875);
}

// Executable counterexample using only the previously available surfaces:
// item replacement cannot add this state; unstructured reload resets progress.
#[test]
fn prior_source_operations_cannot_extend_an_admitted_world() {
    let mut w=ResidentSourceWorkbenchV1::open(BEFORE).unwrap();
    input(&mut w,b"Travel",1); candidate(&mut w,1); w.admit().unwrap();
    let before=w.project_current_world().unwrap();
    let selected=w.source_items().unwrap().into_iter().find(|item|item.source.starts_with(b"on travel")).unwrap();
    let mut replacement=selected.source.clone();
    replacement.extend_from_slice(b"\n\n"); replacement.extend_from_slice(ITEMS);
    assert!(w.replace_source_items(w.generation().handle,&[clause_package::CanonicalSourceItemReplacementV1 { selected,replacement }]).is_err());
    assert_eq!(w.project_current_world().unwrap(),before);
    let mut fresh=BEFORE.to_vec(); fresh.extend_from_slice(b"\n\n"); fresh.extend_from_slice(ITEMS);
    w.hot_reload(&fresh).unwrap();
    assert_eq!(number(&w.project_current_world().unwrap(),b"health"),100.0);
    assert_eq!(number(&before,b"health"),92.875);
}
