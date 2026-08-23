//! Standalone generated Rust for derived query and why-graph parity.

use crate::{
    derive::Limits,
    execution,
    kernel::{Result, Revision},
};
use std::fmt::Write;

/// Execute the revision's admitted query with an explicit finite closure bound
/// and encode the exact public query/why contract.
pub fn canonical_output(revision: &Revision, limits: Limits) -> Result<String> {
    let plan = revision.plan()?;
    let output = execution::execute(revision, &plan, limits)?;
    Ok(execution::canonical_json(&output))
}

/// Emit a source-agnostic standalone Rust program that reloads a sealed
/// revision and prints the same derived query and why-graph bytes.
#[cfg(not(clause_generated))]
pub fn emit_rust(revision: &Revision, limits: Limits) -> Result<String> {
    let mut source = String::new();
    writeln!(source, "mod kernel {{\n{}\n}}", include_str!("kernel.rs")).unwrap();
    writeln!(source, "mod wire {{\n{}\n}}", include_str!("wire.rs")).unwrap();
    writeln!(source, "mod derive {{\n{}\n}}", include_str!("derive.rs")).unwrap();
    writeln!(
        source,
        "mod execution {{\n{}\n}}",
        include_str!("execution.rs")
    )
    .unwrap();
    writeln!(
        source,
        "mod generated {{\n{}\n}}",
        include_str!("generated.rs")
    )
    .unwrap();
    writeln!(
        source,
        "const REVISION_WIRE: &str = {:?};",
        crate::wire::serialize(revision)
    )
    .unwrap();
    writeln!(
        source,
        "fn main() {{ let revision = wire::reload(REVISION_WIRE).expect(\"sealed revision reloads\"); let output = generated::canonical_output(&revision, derive::Limits::new({}, {}, {})).expect(\"bounded query executes\"); print!(\"{{}}\", output); }}",
        limits.max_facts, limits.max_rounds, limits.max_join_attempts
    )
    .unwrap();
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::{Limits, canonical_output, emit_rust};
    use crate::{elaborate, frontend, kernel};
    use std::fs;
    use std::process::Command;

    #[test]
    fn impact_query_generated_rust_matches_the_in_process_why_graph() {
        let model = elaborate::program(
            frontend::parse(include_str!("../examples/impact.clause"))
                .expect("impact source parses"),
        )
        .expect("impact source elaborates");
        let revision = kernel::Revision::admit(model);
        let limits = Limits::new(100, 10, 10_000);
        let expected = canonical_output(&revision, limits).expect("impact query executes");
        assert!(expected.starts_with("[\"clause-query-output-v2\","));
        assert!(expected.contains("impact/recursive-dependency"));
        assert!(expected.contains("impact/impact"));

        let root = std::env::temp_dir().join(format!(
            "clause-impact-generated-parity-{}",
            std::process::id()
        ));
        let source = root.with_extension("rs");
        let binary = root.with_extension("bin");
        let generated = emit_rust(&revision, limits).expect("generated source emits");
        assert!(!generated.contains("relation impact/imports"));
        fs::write(&source, generated).expect("generated source writes");
        let compile = Command::new("rustc")
            .args(["--edition=2024", "--cfg", "clause_generated"])
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .expect("rustc starts");
        assert!(
            compile.status.success(),
            "{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let generated = Command::new(&binary)
            .output()
            .expect("generated program runs");
        assert!(generated.status.success());
        assert_eq!(generated.stdout, expected.as_bytes());
        fs::remove_file(source).expect("generated source cleans up");
        fs::remove_file(binary).expect("generated binary cleans up");
    }
}
