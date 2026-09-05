use clause_package::{Term, decode_canonical_term_bytes};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/event-branches.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1) -> Term {
    let occurrence = w.handler_occurrence(b"use", &[]).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
}

fn number(frame: &Term, relation: &[u8]) -> f64 {
    f64::from_le_bytes(field(field(frame, b"device"), relation).as_atom().unwrap().canonical_payload().try_into().unwrap())
}

#[test]
fn event_rules_share_the_prestate_and_report_rejected_actions() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    let accepted = run(&mut w);
    assert_eq!(number(&accepted, b"charge"), 0.0);
    assert_eq!(number(&accepted, b"attempts"), 1.0);
    assert_eq!(field(field(&accepted, b"device"), b"accepted").as_atom().unwrap().canonical_payload(), &[1]);
    let rejected = run(&mut w);
    assert_eq!(number(&rejected, b"charge"), 0.0);
    assert_eq!(number(&rejected, b"attempts"), 2.0);
    assert_eq!(field(field(&rejected, b"device"), b"accepted").as_atom().unwrap().canonical_payload(), &[0]);
}

#[test]
fn individual_event_rule_edits_retain_live_state_and_diagnostic_identity() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    run(&mut w);
    let effect = w.scalar_effects().unwrap().into_iter().find(|effect| effect.expression == b"?attempts + 1.0").unwrap();
    assert!(w.recorded_handler_event(effect.handler).unwrap().is_some());
    w.edit_scalar_effect(w.generation().handle, &effect, b"?attempts + 2.0").unwrap();
    let next = run(&mut w);
    assert_eq!(number(&next, b"charge"), 0.0);
    assert_eq!(number(&next, b"attempts"), 3.0);
}

#[test]
fn argument_event_dispatches_all_rules_with_the_same_input() {
    let source = SOURCE.replace("bind keyboard Use down to use\n", "")
        .replace("on use ?device", "on use ?device ?amount")
        .replace("?charge - 1.0", "?charge - ?amount")
        .replace("?attempts + 1.0", "?attempts + ?amount");
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    for expected in [0.5, 1.0, 1.5] {
        let occurrence = workbench.handler_occurrence(b"use", &[clause_runtime::ExecutableValueV1::number(0.5).unwrap()]).unwrap();
        workbench.run_occurrences_to_candidate(&[occurrence]).unwrap();
        let frame = decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes).unwrap();
        assert_eq!(number(&frame, b"attempts"), expected);
        assert_eq!(number(&frame, b"charge"), (1.0 - expected).max(0.0));
    }
}
