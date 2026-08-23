//! Generic six-request source-deleted materialization journey.

use clause::{elaborate, frontend, generated, request};
use std::{env, fs, path::PathBuf, process::Command};

const SOURCE: &str = "Node: Type
Gate: Type
State: Type

network/connects: Relation
    {gate: Gate} connects {origin: Node} to {destination: Node}
    mode gate, origin -> destination: many

network/open: Relation
    {gate: Gate} is {state: State}
    mode gate -> state: many

network/reaches: Relation
    {origin: Node} reaches {destination: Node}
    mode origin -> destination: many

scenario: Model
    A: Node
    B: Node
    C: Node
    D: Node
    E: Node
    Active: State
    AB: Gate
    BD: Gate
    AC: Gate
    CD: Gate
    EB: Gate
    EC: Gate
    AB connects A to B
    BD connects B to D
    AC connects A to C
    CD connects C to D
    EB connects E to B
    EC connects E to C
    AB is Active
    BD is Active
    AC is Active
    CD is Active

scenario/direct: Law
    ?origin reaches ?destination
    when:
        ?gate connects ?origin to ?destination
        ?gate is Active

scenario/recursive: Law
    ?origin reaches ?destination
    when:
        ?gate connects ?origin to ?intermediate
        ?gate is Active
        ?intermediate reaches ?destination

scenario/ab-withdrawn: Revision
    from: scenario
    withdraw:
        AB is Active

find all ?destination in scenario:
    A reaches ?destination

why all in scenario:
    A reaches D

prevent all minimal in scenario:
    A reaches D
using:
    network/open

prevent all minimal in scenario/ab-withdrawn:
    A reaches D
using:
    network/open

achieve all minimal in scenario:
    E reaches D
using:
    network/open

diff scenario -> scenario/ab-withdrawn
";

fn temporary(extension: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "clause-request-materialization-{}.{}",
        std::process::id(),
        extension
    ))
}

#[test]
fn six_resolved_requests_preserve_order_and_survive_source_deletion() {
    let compiled = elaborate::compile(frontend::parse(SOURCE).expect("generic source parses"))
        .expect("generic source compiles");
    let resolved = request::resolve(&compiled).expect("requests resolve");
    assert!(matches!(
        resolved.requests(),
        [
            request::Request::Find { .. },
            request::Request::Why { all: true, .. },
            request::Request::Prevent {
                selection: request::Selection::AllMinimal,
                ..
            },
            request::Request::Prevent {
                selection: request::Selection::AllMinimal,
                ..
            },
            request::Request::Achieve {
                selection: request::Selection::AllMinimal,
                ..
            },
            request::Request::Diff { .. },
        ]
    ));
    let expected = request::run(&resolved, request::RunLimits::default())
        .expect("all resolved requests execute")
        .canonical_bytes();

    let emitted = generated::emit_rust(&resolved).expect("sealed requests emit Rust");
    assert!(emitted.contains("wire::reload"));
    assert!(!emitted.contains("find all ?destination in scenario"));
    assert!(!emitted.contains("mod frontend"));
    assert!(!emitted.contains("mod elaborate"));

    let authoring = temporary("clause");
    let rust = temporary("rs");
    let binary = temporary("bin");
    fs::write(&authoring, SOURCE).expect("authoring source writes");
    fs::remove_file(&authoring).expect("authoring source deletes before generated compile");
    fs::write(&rust, emitted).expect("generated Rust writes");
    let compiled = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("generated Rust compiler starts");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let actual = Command::new(&binary)
        .output()
        .expect("source-deleted generated executable starts");
    assert!(actual.status.success());
    assert_eq!(actual.stdout, expected.as_bytes());

    fs::remove_file(&rust).expect("generated Rust cleans up");
    fs::remove_file(&binary).expect("generated executable cleans up");
}

mod emit_rust_cli {
    use std::{
        env, fs,
        path::PathBuf,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

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
    fn emit_rust_cli_compiles_after_source_deletion_with_canonical_output_parity() {
        let source = temporary("clause");
        let emitted = temporary("rs");
        let binary = temporary("bin");
        fs::write(&source, SOURCE).expect("source writes");

        let interpreted = Command::new(env!("CARGO_BIN_EXE_clause"))
            .arg("run")
            .arg(&source)
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
            .arg("emit-rust")
            .arg(&source)
            .arg(&emitted)
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
}
