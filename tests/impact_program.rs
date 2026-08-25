//! Executable source-to-request journey for the typed impact program.

use clause::{elaborate, frontend, generated, request, wire};
use std::{env, fs, path::PathBuf, process::Command};

const SOURCE: &str = include_str!("../examples/impact.clause");

fn temporary(extension: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "clause-impact-program-{}.{}",
        std::process::id(),
        extension
    ))
}

#[test]
fn impact_source_seals_runs_and_generated_requests_survive_source_removal() {
    let parsed = frontend::parse(SOURCE).expect("impact source parses");
    let compiled = elaborate::compile(parsed).expect("impact source compiles");
    let resolved = request::resolve(&compiled).expect("requests resolve in authored order");
    assert_eq!(resolved.requests().len(), 5);
    let expected = request::run(&resolved, request::RunLimits::default())
        .expect("typed requests execute")
        .canonical_bytes();
    assert!(expected.starts_with("[\"clause-run-v2\",[[\"revision\","));
    assert!(expected.contains("[\"find\","));
    for tag in [
        "\"why-all\"",
        "\"prevent-all\"",
        "\"achieve-one\"",
        "\"diff\"",
    ] {
        assert!(expected.contains(tag), "missing request result {tag}");
    }

    let source = temporary("clause");
    let revision = temporary("revision");
    let generated_source = temporary("rs");
    let generated_binary = temporary("bin");
    fs::write(&source, SOURCE).expect("source writes");

    let seal = Command::new(env!("CARGO_BIN_EXE_clause"))
        .arg("seal")
        .arg(&source)
        .arg("impact")
        .arg(&revision)
        .output()
        .expect("seal command starts");
    assert!(
        seal.status.success(),
        "{}",
        String::from_utf8_lossy(&seal.stderr)
    );
    let sealed = wire::reload(&fs::read_to_string(&revision).expect("revision reads"))
        .expect("v3 revision reloads");
    assert_eq!(
        sealed.identity(),
        compiled
            .revision(&frontend::Name("impact".to_owned()))
            .expect("base revision exists")
            .identity()
    );

    let run = Command::new(env!("CARGO_BIN_EXE_clause"))
        .arg("run")
        .arg(&source)
        .output()
        .expect("run command starts");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8(run.stdout).expect("run output is UTF-8"),
        format!("{expected}\n")
    );

    fs::remove_file(&source).expect("authoring source removes before generation");
    fs::write(
        &generated_source,
        generated::emit_rust(&resolved, request::RunLimits::default())
            .expect("resolved requests emit Rust"),
    )
    .expect("generated source writes");
    let compile = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&generated_source)
        .arg("-o")
        .arg(&generated_binary)
        .output()
        .expect("generated Rust compiler starts");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let generated = Command::new(&generated_binary)
        .output()
        .expect("source-deleted generated program starts");
    assert!(generated.status.success());
    assert_eq!(generated.stdout, expected.as_bytes());

    fs::remove_file(&generated_source).expect("generated source cleans up");
    fs::remove_file(&generated_binary).expect("generated binary cleans up");
    fs::remove_file(&revision).expect("revision cleans up");
}
