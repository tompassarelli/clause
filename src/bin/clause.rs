//! External-source Clause command line interface.
//!
//! The authoring file is only needed by `seal`.  `query` reads the persisted
//! revision and therefore remains usable after the source file is removed.

use clause::{
    derive::{Limits, SupportLimits},
    elaborate, execution, frontend, generated,
    intervention::{self, AchieveConfig, AchieveResult, PreventLimits, PreventStatus},
    kernel,
    semantic_diff::SemanticDiff,
    wire,
};
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as Process, ExitCode};

const USAGE: &str =
    "usage: clause seal SOURCE REVISION | clause query REVISION | clause e2e SOURCE REVISION";

fn limits() -> Limits {
    Limits::new(100, 10, 10_000)
}

fn support_limits() -> SupportLimits {
    SupportLimits::new(limits(), 10_000, 100)
}

#[derive(Debug)]
struct CliError {
    status: u8,
    message: String,
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            status: 2,
            message: message.into(),
        }
    }

    fn failure(message: impl Into<String>) -> Self {
        Self {
            status: 1,
            message: message.into(),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "clause: {}", self.message)
    }
}

enum Command {
    Seal { source: PathBuf, revision: PathBuf },
    Query { revision: PathBuf },
    E2e { source: PathBuf, revision: PathBuf },
}

fn parse_command(args: impl IntoIterator<Item = String>) -> Result<Command, CliError> {
    let mut args = args.into_iter();
    let operation = args.next().ok_or_else(|| CliError::usage(USAGE))?;
    match operation.as_str() {
        "seal" => {
            let source = args.next().ok_or_else(|| {
                CliError::usage("seal requires SOURCE\nusage: clause seal SOURCE REVISION")
            })?;
            let revision = args.next().ok_or_else(|| {
                CliError::usage("seal requires REVISION\nusage: clause seal SOURCE REVISION")
            })?;
            if args.next().is_some() {
                return Err(CliError::usage(
                    "seal accepts exactly SOURCE and REVISION\nusage: clause seal SOURCE REVISION",
                ));
            }
            Ok(Command::Seal {
                source: PathBuf::from(source),
                revision: PathBuf::from(revision),
            })
        }
        "query" => {
            let revision = args.next().ok_or_else(|| {
                CliError::usage("query requires REVISION\nusage: clause query REVISION")
            })?;
            if args.next().is_some() {
                return Err(CliError::usage(
                    "query accepts exactly REVISION\nusage: clause query REVISION",
                ));
            }
            Ok(Command::Query {
                revision: PathBuf::from(revision),
            })
        }
        "e2e" => {
            let source = args.next().ok_or_else(|| {
                CliError::usage("e2e requires SOURCE\nusage: clause e2e SOURCE REVISION")
            })?;
            let revision = args.next().ok_or_else(|| {
                CliError::usage("e2e requires REVISION\nusage: clause e2e SOURCE REVISION")
            })?;
            if args.next().is_some() {
                return Err(CliError::usage(
                    "e2e accepts exactly SOURCE and REVISION\nusage: clause e2e SOURCE REVISION",
                ));
            }
            Ok(Command::E2e {
                source: PathBuf::from(source),
                revision: PathBuf::from(revision),
            })
        }
        "--help" | "-h" => Err(CliError::usage(USAGE)),
        _ => Err(CliError::usage(format!(
            "unknown operation '{operation}'\n{USAGE}"
        ))),
    }
}

fn read_utf8(path: &Path, purpose: &str) -> Result<String, CliError> {
    fs::read_to_string(path)
        .map_err(|error| CliError::failure(format!("{purpose} '{}': {error}", path.display())))
}

fn write_revision(path: &Path, bytes: &str) -> Result<(), CliError> {
    fs::write(path, bytes)
        .map_err(|error| CliError::failure(format!("write revision '{}': {error}", path.display())))
}

fn seal(source_path: &Path, revision_path: &Path) -> Result<(), CliError> {
    let source = read_utf8(source_path, "read source")?;
    let parsed = frontend::parse(&source).map_err(|error| {
        CliError::failure(format!("parse source '{}': {error}", source_path.display()))
    })?;
    let model = elaborate::program(parsed).map_err(|error| {
        CliError::failure(format!(
            "elaborate source '{}': {error}",
            source_path.display()
        ))
    })?;
    let revision = kernel::Revision::admit(model);
    let bytes = wire::serialize(&revision);
    write_revision(revision_path, &bytes)?;
    eprintln!(
        "clause: seal ok revision={} source={} revision_file={}",
        revision.identity(),
        source_path.display(),
        revision_path.display()
    );
    Ok(())
}

fn query(revision_path: &Path) -> Result<(), CliError> {
    let bytes = read_utf8(revision_path, "read revision")?;
    let revision = wire::reload(&bytes).map_err(|error| {
        CliError::failure(format!(
            "reload revision '{}': {error}",
            revision_path.display()
        ))
    })?;
    let output = query_json(&revision).map_err(|error| {
        CliError::failure(format!(
            "query revision '{}': {error}",
            revision_path.display()
        ))
    })?;
    println!("{output}");
    eprintln!(
        "clause: query ok revision={} revision_file={}",
        revision.identity(),
        revision_path.display()
    );
    Ok(())
}

fn query_json(revision: &kernel::Revision) -> Result<String, CliError> {
    Ok(execution::canonical_json(&query_output(revision)?))
}

fn query_output(revision: &kernel::Revision) -> Result<execution::QueryOutput, CliError> {
    let plan = revision
        .plan()
        .map_err(|error| CliError::failure(format!("plan revision: {error}")))?;
    execution::execute(revision, &plan, limits())
        .map_err(|error| CliError::failure(format!("query revision: {error}")))
}

fn added_query_fact(
    base: &execution::QueryOutput,
    successor: &execution::QueryOutput,
    revision: &kernel::Revision,
) -> Result<kernel::Clause, CliError> {
    let added = successor
        .results
        .iter()
        .filter(|result| !base.results.contains(result))
        .collect::<Vec<_>>();
    let [added] = added.as_slice() else {
        return Err(CliError::failure(
            "e2e requires exactly one newly entailed query result",
        ));
    };
    let query = revision.model().query();
    let roles = query
        .roles()
        .iter()
        .map(|(role, term)| {
            let term = if term.is_variable() {
                kernel::Term::literal(added.as_str())?
            } else {
                term.clone()
            };
            Ok((role.clone(), term))
        })
        .collect::<kernel::Result<Vec<_>>>()
        .map_err(|error| CliError::failure(format!("bind added query result: {error}")))?;
    kernel::Clause::new(query.relation(), roles)
        .map_err(|error| CliError::failure(format!("instantiate added query result: {error}")))
}

fn verify_generated_rust(revision: &kernel::Revision, output: &str) -> Result<(), CliError> {
    let stem = env::temp_dir().join(format!("clause-e2e-generated-{}", std::process::id()));
    let source = stem.with_extension("rs");
    let binary = stem.with_extension("bin");
    let generated = generated::emit_rust(revision, limits())
        .map_err(|error| CliError::failure(format!("generate Rust: {error}")))?;
    fs::write(&source, generated)
        .map_err(|error| CliError::failure(format!("write generated Rust: {error}")))?;
    let compile = Process::new("rustc")
        .arg("--edition=2024")
        .arg("--cfg")
        .arg("clause_generated")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()
        .map_err(|error| CliError::failure(format!("compile generated Rust: {error}")))?;
    if !compile.status.success() {
        return Err(CliError::failure(format!(
            "generated Rust rejected: {}",
            String::from_utf8_lossy(&compile.stderr)
        )));
    }
    let generated_output = Process::new(&binary)
        .output()
        .map_err(|error| CliError::failure(format!("run generated Rust: {error}")))?;
    let _ = fs::remove_file(source);
    let _ = fs::remove_file(binary);
    if !generated_output.status.success() || generated_output.stdout != output.as_bytes() {
        return Err(CliError::failure(
            "interpreter/generated Rust query-v2 output differs",
        ));
    }
    Ok(())
}

fn json(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value <= '\u{1f}' => escaped.push_str(&format!("\\u{:04x}", value as u32)),
            value => escaped.push(value),
        }
    }
    format!("\"{escaped}\"")
}

fn clauses_json(clauses: &[kernel::Clause]) -> String {
    clauses
        .iter()
        .map(clause_json)
        .collect::<Vec<_>>()
        .join(",")
}

fn clause_json(clause: &kernel::Clause) -> String {
    let roles = clause
        .roles()
        .iter()
        .map(|(name, term)| format!("[{},{}]", json(name), json(term.text())))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"clause\",\"relation\",{},\"roles\",[{roles}]]",
        json(clause.relation())
    )
}

fn diff_json(diff: &SemanticDiff) -> String {
    let proof_changes = diff
        .changed_proofs()
        .iter()
        .map(|change| clause_json(change.fact()))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"clause-semantic-diff-v1\",[\"asserted\",[\"added\",[{}]],[\"removed\",[{}]]],[\"entailed\",[\"added\",[{}]],[\"removed\",[{}]]],[\"proof-changes\",[{proof_changes}]]]",
        clauses_json(diff.authored().added()),
        clauses_json(diff.authored().removed()),
        clauses_json(diff.entailed_added()),
        clauses_json(diff.entailed_removed()),
    )
}

fn prevent_status(status: PreventStatus) -> &'static str {
    match status {
        PreventStatus::Complete => "complete",
        PreventStatus::AlreadyAbsent => "already-absent",
        PreventStatus::CandidateBudgetExhausted => "candidate-budget-exhausted",
        PreventStatus::SolutionBudgetExhausted => "solution-budget-exhausted",
    }
}

fn prevent_json(report: &intervention::PreventReport) -> String {
    let solutions = report
        .solutions()
        .iter()
        .map(|solution| {
            format!(
                "[\"withdrawals\",[{}]]",
                clauses_json(solution.withdrawals())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"clause-prevent-output-v1\",[\"status\",{}],[\"candidates\",{}],[\"solutions\",[{solutions}]]]",
        json(prevent_status(report.status())),
        report.candidates_examined(),
    )
}

fn achieve_json(result: &AchieveResult) -> String {
    let status = match result {
        AchieveResult::Solutions(_) => "solutions",
        AchieveResult::Impossible => "impossible",
        AchieveResult::CandidateLimit(_) => "candidate-limit",
        AchieveResult::SolutionLimit(_) => "solution-limit",
    };
    let interventions = result
        .interventions()
        .iter()
        .map(|intervention| {
            format!(
                "[\"additions\",[{}]]",
                clauses_json(intervention.additions())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"clause-achieve-output-v1\",[\"status\",{}],[\"interventions\",[{interventions}]]]",
        json(status),
    )
}

fn e2e(source_path: &Path, revision_path: &Path) -> Result<(), CliError> {
    seal(source_path, revision_path)?;
    let persisted = read_utf8(revision_path, "read revision")?;
    let base = wire::reload(&persisted)
        .map_err(|error| CliError::failure(format!("reload revision: {error}")))?;
    let intent = match base.model().intents() {
        [intent] => intent,
        _ => {
            return Err(CliError::failure(
                "e2e requires exactly one declared intent",
            ));
        }
    };
    let branch_name = intent
        .name()
        .split_once('/')
        .map(|(namespace, _)| namespace)
        .ok_or_else(|| CliError::failure("intent has no model namespace"))?;
    let branch = kernel::Branch::new(branch_name, base.clone())
        .map_err(|error| CliError::failure(error.to_string()))?;
    let base_query_output = query_output(&base)?;
    let base_query = execution::canonical_json(&base_query_output);
    let intent_name = intent.name().to_owned();
    let proposed = kernel::intent(&branch, &intent_name);
    let desired = proposed
        .intent()
        .ok_or_else(|| CliError::failure("declared intent was not selectable"))?
        .desired()
        .clone();
    let proposed_output = wire::intent_output(&proposed);
    let claimed = kernel::claim(&branch, desired.clone())
        .map_err(|error| CliError::failure(error.to_string()))?;
    let successor = claimed
        .successor()
        .ok_or_else(|| CliError::failure("intent claim did not create successor"))?;
    let canonical_next = wire::serialize(successor.revision());
    write_revision(revision_path, &canonical_next)?;
    let persisted_next = read_utf8(revision_path, "read claimed revision")?;
    if persisted_next != canonical_next {
        return Err(CliError::failure(
            "final revision bytes differ from canonical NEXT envelope",
        ));
    }
    let next = wire::reload(&persisted_next)
        .map_err(|error| CliError::failure(format!("reload claimed revision: {error}")))?;
    if wire::serialize(&next) != canonical_next {
        return Err(CliError::failure(
            "reloaded final revision differs from canonical NEXT envelope",
        ));
    }
    let next_branch = kernel::Branch::new(branch_name, next.clone())
        .map_err(|error| CliError::failure(error.to_string()))?;
    let required = kernel::require(&next, desired.clone())
        .map_err(|error| CliError::failure(error.to_string()))?;
    let next_query_output = query_output(&next)?;
    let next_query = execution::canonical_json(&next_query_output);
    let intervention_target = added_query_fact(&base_query_output, &next_query_output, &next)?;
    let satisfied = kernel::intent(&next_branch, &intent_name);
    let diff = SemanticDiff::between(&base, &next, support_limits())
        .map_err(|error| CliError::failure(format!("diff revisions: {error}")))?;
    let prevention = intervention::prevent(
        &next,
        intervention_target.clone(),
        PreventLimits::new(100, 10, limits()),
    )
    .map_err(|error| CliError::failure(format!("prevent added query result: {error}")))?;
    if prevention.solutions().is_empty() {
        return Err(CliError::failure(
            "prevent added query result produced no withdrawal",
        ));
    }
    let domain = desired
        .roles()
        .values()
        .map(|term| term.text().to_owned())
        .collect();
    let achievement = intervention::achieve(
        &base,
        intervention_target.clone(),
        &AchieveConfig::new(
            vec![desired.relation().to_owned()],
            domain,
            100,
            10,
            limits(),
        ),
    )
    .map_err(|error| CliError::failure(format!("achieve added query result: {error}")))?;
    if achievement.interventions().is_empty() {
        return Err(CliError::failure("achieve intent produced no intervention"));
    }
    let output = format!(
        "[\"clause-demo-output-v1\",[\"base-query\",{base_query}],[\"successor-query\",{next_query}],[\"intervention-target\",{}],[\"intent\",{proposed_output}],[\"claim\",{}],[\"require\",{}],[\"satisfied-intent\",{}],[\"diff\",{}],[\"prevent\",{}],[\"achieve\",{}],[\"generated-parity\",true]]",
        clause_json(&intervention_target),
        wire::claim_output(&claimed),
        wire::require_output(&required),
        wire::intent_output(&satisfied),
        diff_json(&diff),
        prevent_json(&prevention),
        achieve_json(&achievement),
    );
    verify_generated_rust(&base, &base_query)?;
    println!("{output}");
    eprintln!(
        "clause: e2e ok base={} successor={} revision_file={}",
        base.identity(),
        next.identity(),
        revision_path.display()
    );
    Ok(())
}

fn run() -> Result<(), CliError> {
    match parse_command(env::args().skip(1))? {
        Command::Seal { source, revision } => seal(&source, &revision),
        Command::Query { revision } => query(&revision),
        Command::E2e { source, revision } => e2e(&source, &revision),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(error.status)
        }
    }
}
