//! Public request admission and standalone materialization coverage.

use std::{
    collections::BTreeMap,
    env, fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use clause::{elaborate, frontend, request};

const SOURCE: &str = "Item: Type\nlink: Relation\n    {left: Item} links {right: Item}\n    mode left -> right: many\ngraph: Model\n    A: Item\n    B: Item\n    A links B\ngraph/add: Revision\n    from: graph\n    admit:\n        B links A\nfind all ?right in graph:\n    A links ?right\nwhy all in graph:\n    A links B\ndiff graph -> graph/add\n";

fn temporary(extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    env::temp_dir().join(format!(
        "clause-request-integrity-{}-{nonce}.{extension}",
        std::process::id()
    ))
}

#[test]
fn rejects_a_revision_registry_key_for_a_different_sealed_revision() {
    let compiled = elaborate::compile(frontend::parse(SOURCE).expect("source parses"))
        .expect("source compiles");
    let base = compiled
        .revision(&frontend::Name("graph".into()))
        .expect("base Revision exists");
    let successor = compiled
        .revision(&frontend::Name("graph/add".into()))
        .expect("successor Revision exists");
    assert_ne!(base.identity(), successor.identity());

    let error = request::ResolvedProgram::new(
        BTreeMap::from([(base.identity().clone(), successor.clone())]),
        vec![],
    )
    .expect_err("registry key must authenticate the stored Revision");
    assert!(
        error
            .to_string()
            .contains("Revision registry key must match sealed Revision identity")
    );
}

#[test]
fn emit_rust_cli_compiles_after_source_deletion_with_canonical_output_parity() {
    let source = temporary("clause");
    let emitted = temporary("rs");
    let binary = temporary("bin");
    fs::write(&source, SOURCE).expect("source writes");

    let interpreted = Command::new(env!("CARGO_BIN_EXE_clause"))
        .args(["run", source.to_str().expect("UTF-8 source path")])
        .output()
        .expect("run command starts");
    assert!(
        interpreted.status.success(),
        "{}",
        String::from_utf8_lossy(&interpreted.stderr)
    );
    let expected = String::from_utf8(interpreted.stdout)
        .expect("run output is UTF-8")
        .trim_end_matches('\n')
        .to_owned();

    let materialized = Command::new(env!("CARGO_BIN_EXE_clause"))
        .args([
            "emit-rust",
            source.to_str().expect("UTF-8 source path"),
            emitted.to_str().expect("UTF-8 output path"),
        ])
        .output()
        .expect("emit-rust command starts");
    assert!(
        materialized.status.success(),
        "{}",
        String::from_utf8_lossy(&materialized.stderr)
    );
    let emitted_text = fs::read_to_string(&emitted).expect("emitted Rust reads");
    assert!(!emitted_text.contains(SOURCE));
    assert!(!emitted_text.contains("mod frontend"));
    fs::remove_file(&source).expect("authoring source removes before Rust compilation");

    let compile = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&emitted)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("generated Rust compiler starts");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let generated = Command::new(&binary)
        .output()
        .expect("source-deleted generated program starts");
    assert!(generated.status.success());
    assert_eq!(generated.stdout, expected.as_bytes());

    fs::remove_file(&emitted).expect("generated Rust source cleans up");
    fs::remove_file(&binary).expect("generated binary cleans up");
}
