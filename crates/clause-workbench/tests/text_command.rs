use std::path::PathBuf;
use std::process::Command;

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
        .arg("run-text")
        .arg(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test-vectors/authoring/command-text.clause"),
        )
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn text_arguments_select_checked_relational_command_output() {
    let result = run(&["describe", "response", "output", "module", "list"]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, b"firn module list: list modules\n");
    let result = run(&["describe", "response", "output", "module", "status"]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, b"firn module status: show module status\n");
}

#[test]
fn mismatched_arguments_and_missing_result_reject_without_output() {
    for arguments in [
        vec!["describe", "response", "output", "module"],
        vec!["describe", "response", "missing", "module", "list"],
    ] {
        let result = run(&arguments);
        assert!(!result.status.success(), "{result:?}");
        assert!(result.stdout.is_empty(), "{result:?}");
    }
}
