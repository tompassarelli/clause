use clause_runtime::{
    SourceProfilePhaseV1 as Phase, begin_executable_source_profile_v1 as begin,
    finish_executable_source_profile_v1 as finish, source_profile_scope_v1 as scope,
};
use clause_workbench::ResidentSourceWorkbenchV1;

#[test]
fn profile_is_opt_in_nested_bounded_and_never_resets_a_live_observation() {
    {
        let _off = scope(Phase::Lowering);
    }
    assert!(finish().is_none());
    assert!(begin());
    assert!(!begin());
    {
        let _parent = scope(Phase::WitnessCheck);
        let _child = scope(Phase::Lowering);
        assert!(finish().is_none());
    }
    let report = finish().unwrap();
    assert!(!report.truncated);
    assert_eq!(report.phases[Phase::WitnessCheck as usize].calls, 1);
    assert_eq!(report.phases[Phase::Lowering as usize].calls, 1);
    assert!(
        report.phases[Phase::WitnessCheck as usize].inclusive_milliseconds
            >= report.phases[Phase::Lowering as usize].inclusive_milliseconds
    );
    assert!(finish().is_none());
}

#[test]
fn profiled_real_source_edit_still_checks_noop_rejection_stale_and_live_continuity() {
    let source = include_bytes!("../../../test-vectors/authoring/live-encounter.clause");
    let mut w = ResidentSourceWorkbenchV1::open(source).unwrap();
    let effect = w
        .scalar_effects()
        .unwrap()
        .into_iter()
        .find(|effect| effect.expression == b"0.0 - ?damage")
        .unwrap();
    let old = w.generation().clone();
    assert!(begin());
    assert_eq!(
        w.edit_scalar_effect(old.handle, &effect, &effect.expression)
            .unwrap(),
        old
    );
    assert!(w.edit_scalar_effect(old.handle, &effect, b"true").is_err());
    let nochange = finish().unwrap();
    assert_eq!(nochange.phases[Phase::Transfer as usize].calls, 0);
    assert_eq!(w.exact_source(), source);
    assert!(begin());
    w.edit_scalar_effect(old.handle, &effect, b"0.0 - (?damage * 2.0)")
        .unwrap();
    let report = finish().unwrap();
    assert!(!report.truncated);
    assert_eq!(report.phases[Phase::Transfer as usize].calls, 1);
    // Native preparation performs a checked preflight; the independently
    // owned live boundary replays the witness again before replacement.
    assert_eq!(report.phases[Phase::WitnessCheck as usize].calls, 2);
    assert_eq!(report.phases[Phase::OldElaboration as usize].calls, 2);
    assert_eq!(report.phases[Phase::NewElaboration as usize].calls, 2);
    assert_eq!(report.phases[Phase::Migration as usize].calls, 1);
    assert!(report.phases[Phase::Lowering as usize].calls >= 2);
    assert!(w.rejects_stale_handle(old.handle).unwrap());
    assert!(w.source_continuity().is_ok());
    assert!(w.edit_scalar_effect(old.handle, &effect, b"0.0").is_err());
}
