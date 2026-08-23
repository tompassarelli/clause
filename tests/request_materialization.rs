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
