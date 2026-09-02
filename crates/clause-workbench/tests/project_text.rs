use std::process::Command;

#[test]
fn clause_source_projects_the_checked_in_flake_exactly() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../../flake.clause");
    let expected = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../flake.nix"));
    let output = Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
        .args(["project-text", source, "emit-flake"])
        .output()
        .expect("the workbench binary executes");

    assert!(
        output.status.success(),
        "project-text failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected);
}
