//! Executable flagship journey for the dependency-impact model.

use clause::{derive::Limits, generated, wire};
use std::{env, fs, process::Command};

const SOURCE: &str = include_str!("../examples/impact.clause");

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

    assert!(demo.starts_with("[\"clause-demo-output-v1\","));
    assert!(demo.contains(
        "[\"base-query\",[\"clause-query-output-v2\",[\"results\",[\"North\",\"Store\"]]"
    ));
    assert!(demo.contains("[\"successor-query\",[\"clause-query-output-v2\",[\"results\",[\"North\",\"South\",\"Store\"]]"));
    assert!(demo.contains("\"impact/recursive-dependency\""));
    assert!(demo.contains("\"impact/impact\""));
    assert!(demo.contains("[\"intervention-target\",[\"clause\",\"relation\",\"impact/affected\",\"roles\",[[\"change\",\"compiler-change\"],[\"consumer\",\"South\"]]]]"));
    assert!(demo.contains("[\"asserted\",[\"added\",[[\"clause\",\"relation\",\"impact/imports\",\"roles\",[[\"consumer\",\"South\"],[\"dependency\",\"North\"]]]]]"));
    assert!(
        demo.contains("[\"entailed\",[\"added\",[[\"clause\",\"relation\",\"impact/affected\"")
    );
    assert_eq!(demo.matches("\"impact/depends\"").count(), 12);
    assert!(demo.contains("[\"proof-changes\",[]]"));
    assert!(
        demo.contains(
            "[\"clause-prevent-output-v1\",[\"status\",\"complete\"],[\"candidates\",15]"
        )
    );
    assert!(demo.contains("[\"clause-achieve-output-v1\",[\"status\",\"solutions\"]"));
    assert_eq!(demo.matches("[\"additions\",[[\"clause\"").count(), 1);
    assert!(demo.contains("[\"additions\",[[\"clause\",\"relation\",\"impact/imports\",\"roles\",[[\"consumer\",\"South\"],[\"dependency\",\"North\"]]]]]"));
    assert!(demo.ends_with("[\"generated-parity\",true]]\n"));

    fs::remove_file(&source_path).expect("authoring source removes after seal");
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
    assert!(
        query.starts_with(
            "[\"clause-query-output-v2\",[\"results\",[\"North\",\"South\",\"Store\"]]"
        )
    );
    assert!(demo.contains(&format!("[\"successor-query\",{}]", query.trim())));

    let persisted = fs::read_to_string(&revision_path).expect("sealed revision reads");
    let revision = wire::reload(&persisted).expect("sealed revision reloads");
    let generated_source = temp_path("rs");
    let generated_binary = temp_path("bin");
    fs::write(
        &generated_source,
        generated::emit_rust(&revision, Limits::new(100, 10, 10_000))
            .expect("standalone Rust emits after source removal"),
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
    let generated_query = Command::new(&generated_binary)
        .output()
        .expect("source-deleted generated query starts");
    assert!(generated_query.status.success());
    assert_eq!(generated_query.stdout, query.trim().as_bytes());

    let _ = fs::remove_file(generated_source);
    let _ = fs::remove_file(generated_binary);
    let _ = fs::remove_file(revision_path);
}
