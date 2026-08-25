use super::emit_rust;
use crate::{elaborate, frontend, request};
use std::{fs, process::Command};

#[test]
fn embeds_resolved_requests_not_source() {
    let source = "Item\nlink: RelationShape\n  {left: Item} links {right: Item}\n  mode left -> right: many\ngraph\n  A ∈ Item\n  B ∈ Item\n  A links B\nfind all ?right in graph:\n  A links ?right\n";
    let program =
        request::resolve(&elaborate::compile(frontend::parse(source).unwrap()).unwrap()).unwrap();
    let emitted = emit_rust(&program, request::RunLimits::default()).unwrap();
    assert!(emitted.contains("request::Request::Find"));
    assert!(!emitted.contains("find all ?right"));
}

#[test]
fn generated_program_matches_source_deleted_request_transcript() {
    let source = "Item\nlink: RelationShape\n  {left: Item} links {right: Item}\n  mode left -> right: many\ngraph\n  A ∈ Item\n  B ∈ Item\n  A links B\ngraph/add: Revision\n  from: graph\n  admit:\n    B links A\nfind all ?right in graph:\n  A links ?right\nwhy all in graph:\n  A links B\ndiff graph -> graph/add\n";
    let program =
        request::resolve(&elaborate::compile(frontend::parse(source).unwrap()).unwrap()).unwrap();
    let expected = request::run(&program, request::RunLimits::default())
        .unwrap()
        .canonical_bytes();
    let root =
        std::env::temp_dir().join(format!("clause-request-generated-{}", std::process::id()));
    let rust = root.with_extension("rs");
    let binary = root.with_extension("bin");
    fs::write(
        &rust,
        emit_rust(&program, request::RunLimits::default()).unwrap(),
    )
    .unwrap();
    let compiled = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let actual = Command::new(&binary).output().unwrap();
    assert!(actual.status.success());
    assert_eq!(actual.stdout, expected.as_bytes());
    fs::remove_file(rust).unwrap();
    fs::remove_file(binary).unwrap();
}
