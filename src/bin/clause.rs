//! External-source Clause command line interface.
//!
//! The authoring file is only needed by `seal`.  `query` reads the persisted
//! revision and therefore remains usable after the source file is removed.

use clause::{
    derive::Limits,
    elaborate, execution, frontend, generated, kernel,
    semantic_output::{self, SemanticJourney},
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

fn verify_generated_rust(
    revision: &kernel::Revision,
    journey: &SemanticJourney,
    output: &str,
) -> Result<(), CliError> {
    let stem = env::temp_dir().join(format!("clause-e2e-generated-{}", std::process::id()));
    let source = stem.with_extension("rs");
    let binary = stem.with_extension("bin");
    let generated = generated::emit_rust(revision, journey)
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
            "interpreter/generated Rust semantic-journey output differs",
        ));
    }
    Ok(())
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
    let intent_name = intent.name().to_owned();
    let proposed = kernel::intent(&branch, &intent_name);
    let desired = proposed
        .intent()
        .ok_or_else(|| CliError::failure("declared intent was not selectable"))?
        .desired()
        .clone();
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
    let journey = SemanticJourney::from_successor(&base, &next, limits())
        .map_err(|error| CliError::failure(format!("prepare semantic journey: {error}")))?;
    let output = semantic_output::canonical_output(&base, &journey)
        .map_err(|error| CliError::failure(format!("execute semantic journey: {error}")))?;
    verify_generated_rust(&base, &journey, &output)?;
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
