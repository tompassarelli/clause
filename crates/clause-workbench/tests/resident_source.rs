use clause_workbench::ResidentSourceWorkbenchV1;

const WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world.clause"
));

#[test]
fn source_edit_hot_reloads_in_one_workbench_without_admission_custody_leak() {
    let mut workbench =
        ResidentSourceWorkbenchV1::open(WORLD).expect("base source opens in one workbench");
    let base_generation = workbench.generation().clone();
    let base_candidate = workbench
        .run_to_candidate()
        .expect("base source produces one hidden candidate");
    assert_eq!(base_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let base_admission = workbench
        .admit()
        .expect("separate base Admission returns the rendered frame");
    assert_eq!(base_admission.state_revision_count, 2);
    assert_ne!(base_admission.predecessor, base_admission.successor);

    let changed = std::str::from_utf8(WORLD)
        .expect("world source is UTF-8")
        .replacen("jump-arena move speed 5.0", "jump-arena move speed 7.0", 1);
    assert_ne!(changed.as_bytes(), WORLD);
    let changed_generation = workbench
        .hot_reload(changed.as_bytes())
        .expect("source-only edit hot reloads in the resident process");
    assert_eq!(
        changed_generation.handle.generation,
        base_generation.handle.generation + 1
    );
    assert_ne!(
        changed_generation.source_package,
        base_generation.source_package
    );
    assert_ne!(changed_generation.cpp1, base_generation.cpp1);
    assert_ne!(changed_generation.cwr1, base_generation.cwr1);
    assert!(
        workbench
            .rejects_stale_handle(base_generation.handle)
            .expect("stale-handle probe reaches the live boundary")
    );

    let changed_candidate = workbench
        .run_to_candidate()
        .expect("changed source reruns without restarting Rust");
    assert_eq!(changed_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let changed_admission = workbench
        .admit()
        .expect("separate changed-source Admission returns its frame");
    assert_eq!(changed_admission.state_revision_count, 2);
    assert_ne!(
        changed_admission.projection.exact_term_bytes, base_admission.projection.exact_term_bytes,
        "the source edit changes the admitted rendered frame"
    );
}
