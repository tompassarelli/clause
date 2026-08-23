//! Executable flagship journey for the dependency-impact model.

use clause::{
    derive::Limits,
    generated,
    semantic_output::{self, SemanticJourney},
    wire,
};
use std::{env, fs, process::Command};

const SOURCE: &str = include_str!("../examples/impact.clause");
const EXPECTED: &str = include_str!("impact_semantic_journey_v1.json");

fn temp_path(extension: &str) -> std::path::PathBuf {
    env::temp_dir().join(format!(
        "clause-impact-oracle-{}-{}.{}",
        std::process::id(),
        extension,
        extension
    ))
}

#[test]
fn impact_journey_seals_derives_changes_intervenes_and_survives_source_removal() {
    let source_path = temp_path("clause");
    let revision_path = temp_path("revision");
    fs::write(&source_path, SOURCE).expect("fixture writes");

    let seal = Command::new(env!("CARGO_BIN_EXE_clause"))
        .args([
            "seal",
            source_path.to_str().unwrap(),
            revision_path.to_str().unwrap(),
        ])
        .output()
        .expect("seal command starts");
    assert!(
        seal.status.success(),
        "{}",
        String::from_utf8_lossy(&seal.stderr)
    );
    let base_wire = fs::read_to_string(&revision_path).expect("base revision reads");
    let base = wire::reload(&base_wire).expect("base revision reloads");

    let demo = Command::new(env!("CARGO_BIN_EXE_clause"))
        .args([
            "e2e",
            source_path.to_str().unwrap(),
            revision_path.to_str().unwrap(),
        ])
        .output()
        .expect("demo command starts");
    assert!(
        demo.status.success(),
        "{}",
        String::from_utf8_lossy(&demo.stderr)
    );
    let demo = String::from_utf8(demo.stdout).expect("demo is UTF-8");
    assert_eq!(demo, EXPECTED);

    assert!(demo.starts_with("[\"clause-semantic-journey-v1\","));
    assert!(demo.contains("[\"find\",[\"results\",[\"North\",\"Relay\",\"Store\"]]]"));
    assert!(demo.contains("[\"support-frontier\",[\"target\","));
    assert!(demo.contains("[\"why-all\",[\"target\","));
    assert!(demo.contains("[\"support-diff\",[\"asserted\","));
    assert!(demo.contains("\"impact/recursive-dependency\""));
    assert!(demo.contains("\"impact/impact\""));
    assert!(demo.contains("[\"status\",\"complete\"]"));
    assert!(demo.contains("[\"status\",\"candidate-budget-exhausted\"]"));

    let persisted = fs::read_to_string(&revision_path).expect("successor revision reads");
    let successor = wire::reload(&persisted).expect("successor revision reloads");
    let limits = Limits::new(100, 10, 10_000);
    let journey = SemanticJourney::from_successor(&base, &successor, limits)
        .expect("semantic journey prepares from revisions");
    let interpreted =
        semantic_output::canonical_output(&base, &journey).expect("semantic journey interprets");
    assert_eq!(format!("{interpreted}\n"), EXPECTED);

    fs::remove_file(&source_path).expect("authoring source removes before generation");
    let query = Command::new(env!("CARGO_BIN_EXE_clause"))
        .args(["query", revision_path.to_str().unwrap()])
        .output()
        .expect("source-free query starts");
    assert!(
        query.status.success(),
        "{}",
        String::from_utf8_lossy(&query.stderr)
    );
    let query = String::from_utf8(query.stdout).expect("query is UTF-8");
    assert!(query.starts_with(
        "[\"clause-query-output-v2\",[\"results\",[\"North\",\"Relay\",\"South\",\"Store\"]]"
    ));
    let generated_source = temp_path("rs");
    let generated_binary = temp_path("bin");
    fs::write(
        &generated_source,
        generated::emit_rust(&base, &journey).expect("standalone Rust emits after source removal"),
    )
    .expect("standalone Rust writes");
    let compile = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&generated_source)
        .arg("-o")
        .arg(&generated_binary)
        .output()
        .expect("standalone Rust compiler starts");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let generated_journey = Command::new(&generated_binary)
        .output()
        .expect("source-deleted generated journey starts");
    assert!(generated_journey.status.success());
    assert_eq!(
        generated_journey.stdout,
        EXPECTED.strip_suffix('\n').unwrap().as_bytes()
    );

    let _ = fs::remove_file(generated_source);
    let _ = fs::remove_file(generated_binary);
    let _ = fs::remove_file(revision_path);
}
