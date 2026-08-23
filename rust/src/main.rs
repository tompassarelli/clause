//! The sealed Clause M4 end-to-end intent journey.

use clause_rust_spike::{elaborate, execution, frontend, kernel, wire};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BASE_ID: &str = "rev-sha256-746240d8119edb45ce1971043d46fa865847efa799b682463d484445aa7b8f77";
const NEXT_ID: &str = "rev-sha256-aa2dc7de2b7489b035a4cd6194f2b436a89c765d462eb055d5a01ffdd2004ceb";

const FIXTURE: &str = r#"relation catalog/contains(set: Text, member: Text):
    sentence: {set} contains {member}
    mode set -> member: many

model catalog:
    "letters" contains "a"
    "letters" contains "b"

intent catalog/restock:
    "letters" contains "c"

query catalog:
    ?member where "letters" contains ?member
"#;

fn fail(message: impl Into<String>) -> ! {
    panic!("Clause Rust M4 e2e canary: {}", message.into())
}

fn check(condition: bool, message: impl Into<String>) {
    if !condition {
        fail(message);
    }
}

fn temp_path(suffix: &str) -> PathBuf {
    env::temp_dir().join(format!("clause-rust-m4-{}-{suffix}", std::process::id()))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| fail(format!("read {}: {error}", path.display())))
}

fn query(revision: &kernel::Revision) -> String {
    let plan = revision
        .plan()
        .unwrap_or_else(|error| fail(error.to_string()));
    execution::canonical_json(
        &execution::execute(revision, &plan).unwrap_or_else(|error| fail(error.to_string())),
    )
}

fn main() {
    let source_path = temp_path("fixture.clause");
    let sealed_path = temp_path("revision.json");
    let generated_path = temp_path("generated.rs");
    let binary_path = temp_path("generated-bin");

    fs::write(&source_path, FIXTURE)
        .unwrap_or_else(|error| fail(format!("write fixture: {error}")));
    let parsed =
        frontend::parse(&read(&source_path)).unwrap_or_else(|error| fail(error.to_string()));
    let revision = kernel::Revision::admit(
        elaborate::program(parsed).unwrap_or_else(|error| fail(error.to_string())),
    );
    check(
        revision.identity() == BASE_ID,
        "base revision identity differs from contract",
    );

    let base_wire = wire::serialize(&revision);
    fs::write(&sealed_path, &base_wire)
        .unwrap_or_else(|error| fail(format!("persist base revision: {error}")));
    fs::remove_file(&source_path).unwrap_or_else(|error| fail(format!("delete source: {error}")));
    check(!source_path.exists(), "authoring source survived deletion");

    check(
        wire::reload(&base_wire.replacen("catalog/restock", "catalog/missing", 1)).is_err(),
        "tampered intent name was admitted",
    );
    check(
        wire::reload(&base_wire.replacen(
            "[\"member\",[\"literal\",\"c\"]]",
            "[\"member\",[\"literal\",\"d\"]]",
            1,
        ))
        .is_err(),
        "tampered desired term was admitted",
    );
    check(
        wire::reload(&base_wire.replacen("[\"facts\",", "[\"intents\",", 1)).is_err(),
        "tampered semantic array order was admitted",
    );
    check(
        wire::reload(&base_wire.replacen("rev-sha256-7", "rev-sha256-8", 1)).is_err(),
        "tampered revision identity was admitted",
    );

    let base = wire::reload(&read(&sealed_path)).unwrap_or_else(|error| fail(error.to_string()));
    let branch = kernel::Branch::new("catalog", base.clone())
        .unwrap_or_else(|error| fail(error.to_string()));
    let base_query = query(&base);
    check(
        base_query.contains("[\"results\",[\"a\",\"b\"]]"),
        "base query is not a,b",
    );

    let proposed = kernel::intent(&branch, "catalog/restock");
    let proposed_wire = wire::intent_output(&proposed);
    check(
        proposed_wire.contains("desired-clause-is-absent"),
        "base intent did not propose a justified claim",
    );
    let unknown = wire::intent_output(&kernel::intent(&branch, "catalog/missing"));
    check(
        unknown
            == format!(
                "[\"clause-intent-output-v1\",\"rejected\",[\"revision\",\"{BASE_ID}\"],[\"intent\",\"catalog/missing\"],[\"diagnostic\",\"intent.unknown\"]]"
            ),
        "unknown intent output differs from contract",
    );

    let desired = proposed
        .intent()
        .unwrap_or_else(|| fail("proposal omitted desired intent"))
        .desired()
        .clone();
    let admitted =
        kernel::claim(&branch, desired.clone()).unwrap_or_else(|error| fail(error.to_string()));
    let claim_wire = wire::claim_output(&admitted);
    let successor = admitted
        .successor()
        .unwrap_or_else(|| fail("claim did not create successor"));
    check(
        successor.revision().identity() == NEXT_ID,
        "successor identity differs from contract",
    );
    check(
        branch.revision() == &base && wire::serialize(branch.revision()) == base_wire,
        "intent or claim mutated the base branch",
    );

    let required = kernel::require(successor.revision(), desired)
        .unwrap_or_else(|error| fail(error.to_string()));
    let require_wire = wire::require_output(&required);
    let next_query = query(successor.revision());
    check(
        next_query.contains("[\"results\",[\"a\",\"b\",\"c\"]]"),
        "successor query is not a,b,c",
    );
    let satisfied = wire::intent_output(&kernel::intent(successor, "catalog/restock"));
    check(
        satisfied.contains("desired-clause-is-claimed")
            && satisfied.contains(&format!(
                "proof/{NEXT_ID}/catalog/contains/member=c,set=letters"
            )),
        "successor intent lacks revision-scoped proof",
    );

    let e2e = format!(
        "[\"clause-e2e-output-v1\",{base_query},{proposed_wire},{claim_wire},{require_wire},{next_query},{satisfied}]"
    );
    let generated = format!("fn main() {{ print!(\"{{}}\", {e2e:?}); }}\n");
    fs::write(&generated_path, generated)
        .unwrap_or_else(|error| fail(format!("write generated Rust: {error}")));
    let compile = Command::new("rustc")
        .arg("--edition=2024")
        .arg(&generated_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .unwrap_or_else(|error| fail(format!("compile generated Rust: {error}")));
    check(
        compile.status.success(),
        format!(
            "generated Rust failed: {}",
            String::from_utf8_lossy(&compile.stderr)
        ),
    );
    let output = Command::new(&binary_path)
        .output()
        .unwrap_or_else(|error| fail(format!("run generated Rust: {error}")));
    check(
        output.status.success(),
        "generated Rust exited unsuccessfully",
    );
    check(
        output.stdout == e2e.as_bytes(),
        "interpreter/generated Rust e2e output differs",
    );

    for path in [&sealed_path, &generated_path, &binary_path] {
        let _ = fs::remove_file(path);
    }
    println!("{e2e}");
    println!("PASS clause-rust-m4-e2e {BASE_ID} {NEXT_ID}");
}
