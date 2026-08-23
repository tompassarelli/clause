//! Standalone generated Rust for canonical semantic-journey parity.

#![allow(unexpected_cfgs)]

use crate::{
    intervention::{AchieveConfig, PreventLimits},
    kernel::{Clause, Result, Revision},
    semantic_output::SemanticJourney,
};
use std::fmt::Write;

/// Emit a source-agnostic standalone Rust program that reloads a sealed
/// revision and prints the same complete bounded semantic-journey bytes.
#[cfg(not(clause_generated))]
pub fn emit_rust(revision: &Revision, journey: &SemanticJourney) -> Result<String> {
    let mut source = String::new();
    writeln!(source, "mod kernel {{\n{}\n}}", include_str!("kernel.rs")).unwrap();
    writeln!(source, "mod wire {{\n{}\n}}", include_str!("wire.rs")).unwrap();
    writeln!(source, "mod derive {{\n{}\n}}", include_str!("derive.rs")).unwrap();
    writeln!(source, "mod delta {{\n{}\n}}", include_str!("delta.rs")).unwrap();
    writeln!(
        source,
        "mod execution {{\n{}\n}}",
        include_str!("execution.rs")
    )
    .unwrap();
    writeln!(
        source,
        "mod intervention {{\n{}\n}}",
        include_str!("intervention.rs")
    )
    .unwrap();
    writeln!(
        source,
        "mod semantic_diff {{\n{}\n}}",
        include_str!("semantic_diff.rs")
    )
    .unwrap();
    writeln!(
        source,
        "mod semantic_output {{\n{}\n}}",
        include_str!("semantic_output.rs")
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
        "const SUPPORT_LOSS_WIRE: &str = {:?};",
        crate::wire::serialize(journey.support_loss())
    )
    .unwrap();
    let query_limits = journey.query_limits();
    let prevent = prevent_source(journey.prevent_limits());
    let achieve = achieve_source(journey.achieve_config());
    writeln!(
        source,
        "fn main() {{ let revision = wire::reload(REVISION_WIRE).expect(\"sealed revision reloads\"); let support_loss = wire::reload(SUPPORT_LOSS_WIRE).expect(\"support-loss revision reloads\"); let journey = semantic_output::SemanticJourney::new(support_loss, {}, {prevent}, {}, {achieve}, derive::Limits::new({}, {}, {})); let output = semantic_output::canonical_output(&revision, &journey).expect(\"bounded semantic journey executes\"); print!(\"{{}}\", output); }}",
        clause_source(journey.support_target()),
        clause_source(journey.achievement_goal()),
        query_limits.max_facts,
        query_limits.max_rounds,
        query_limits.max_join_attempts,
    )
    .unwrap();
    Ok(source)
}

#[cfg(not(clause_generated))]
fn limits_source(limits: crate::derive::Limits) -> String {
    format!(
        "derive::Limits::new({}, {}, {})",
        limits.max_facts, limits.max_rounds, limits.max_join_attempts
    )
}

#[cfg(not(clause_generated))]
fn support_limits_source(limits: crate::derive::SupportLimits) -> String {
    format!(
        "derive::SupportLimits::new({}, {}, {})",
        limits_source(limits.closure),
        limits.max_expansions,
        limits.max_supports_per_clause,
    )
}

#[cfg(not(clause_generated))]
fn prevent_source(limits: &PreventLimits) -> String {
    let mut source = format!(
        "intervention::PreventLimits::new({}, {}, {}).with_support_limits({})",
        limits.max_candidates(),
        limits.max_solutions(),
        limits_source(limits.closure_limits()),
        support_limits_source(limits.support_limits()),
    );
    if let Some(relations) = limits.withdrawal_relations() {
        write!(
            source,
            ".using_relations(vec![{}])",
            strings_source(relations)
        )
        .unwrap();
    }
    source
}

#[cfg(not(clause_generated))]
fn achieve_source(config: &AchieveConfig) -> String {
    format!(
        "intervention::AchieveConfig::new(vec![{}], vec![{}], {}, {}, {})",
        strings_source(config.allowed_relations()),
        strings_source(config.active_domain()),
        config.max_candidates(),
        config.max_solutions(),
        limits_source(config.closure_limits()),
    )
}

#[cfg(not(clause_generated))]
fn strings_source(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("String::from({value:?})"))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(not(clause_generated))]
fn clause_source(clause: &Clause) -> String {
    let roles = clause
        .roles()
        .iter()
        .map(|(role, term)| {
            format!(
                "(String::from({role:?}), kernel::Term::literal({:?}).expect(\"generated ground term is valid\"))",
                term.text()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "kernel::Clause::new({:?}, vec![{roles}]).expect(\"generated ground clause is valid\")",
        clause.relation()
    )
}

#[cfg(test)]
mod tests {
    use super::emit_rust;
    use crate::{
        derive::Limits,
        elaborate, frontend, kernel,
        semantic_output::{self, SemanticJourney},
    };
    use std::fs;
    use std::process::Command;

    #[test]
    fn impact_semantic_journey_generated_rust_matches_in_process_bytes() {
        let model = elaborate::program(
            frontend::parse(include_str!("../examples/impact.clause"))
                .expect("impact source parses"),
        )
        .expect("impact source elaborates");
        let revision = kernel::Revision::admit(model);
        let limits = Limits::new(100, 10, 10_000);
        let intent = &revision.model().intents()[0];
        let branch = kernel::Branch::new("impact", revision.clone()).expect("branch admits");
        let claimed = kernel::claim(&branch, intent.desired().clone()).expect("intent claims");
        let successor = claimed.successor().expect("claim has successor").revision();
        let journey = SemanticJourney::from_successor(&revision, successor, limits)
            .expect("semantic journey prepares");
        let expected = semantic_output::canonical_output(&revision, &journey)
            .expect("semantic journey executes");
        assert!(expected.starts_with("[\"clause-semantic-journey-v1\","));
        assert!(expected.contains("impact/recursive-dependency"));
        assert!(expected.contains("impact/impact"));

        let root = std::env::temp_dir().join(format!(
            "clause-impact-generated-parity-{}",
            std::process::id()
        ));
        let source = root.with_extension("rs");
        let binary = root.with_extension("bin");
        fs::write(
            &source,
            emit_rust(&revision, &journey).expect("generated source emits"),
        )
        .expect("generated source writes");
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
