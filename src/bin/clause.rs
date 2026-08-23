//! Native Clause command line interface.

use clause::{elaborate, frontend, generated, request, wire};
use std::{
    env,
    ffi::OsString,
    fmt, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const USAGE: &str = "usage: clause seal SOURCE REVISION_NAME REVISION_FILE | clause run SOURCE | clause emit-rust SOURCE OUTPUT.rs";

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
    Seal {
        source: PathBuf,
        name: frontend::Name,
        revision: PathBuf,
    },
    Run {
        source: PathBuf,
    },
    EmitRust {
        source: PathBuf,
        output: PathBuf,
    },
}

fn text_argument(argument: OsString, label: &str) -> Result<String, CliError> {
    argument
        .into_string()
        .map_err(|_| CliError::usage(format!("{label} must be valid UTF-8\n{USAGE}")))
}

fn parse_command(args: impl IntoIterator<Item = OsString>) -> Result<Command, CliError> {
    let mut args = args.into_iter();
    let command = args.next().ok_or_else(|| CliError::usage(USAGE))?;
    match text_argument(command, "COMMAND")?.as_str() {
        "seal" => {
            let source = args.next().ok_or_else(|| CliError::usage(USAGE))?;
            let name = args.next().ok_or_else(|| CliError::usage(USAGE))?;
            let revision = args.next().ok_or_else(|| CliError::usage(USAGE))?;
            if args.next().is_some() {
                return Err(CliError::usage(USAGE));
            }
            Ok(Command::Seal {
                source: source.into(),
                name: frontend::Name(text_argument(name, "REVISION_NAME")?),
                revision: revision.into(),
            })
        }
        "run" => {
            let source = args.next().ok_or_else(|| CliError::usage(USAGE))?;
            if args.next().is_some() {
                return Err(CliError::usage(USAGE));
            }
            Ok(Command::Run {
                source: source.into(),
            })
        }
        "emit-rust" => {
            let source = args.next().ok_or_else(|| CliError::usage(USAGE))?;
            let output = args.next().ok_or_else(|| CliError::usage(USAGE))?;
            if args.next().is_some() {
                return Err(CliError::usage(USAGE));
            }
            Ok(Command::EmitRust {
                source: source.into(),
                output: output.into(),
            })
        }
        _ => Err(CliError::usage(USAGE)),
    }
}

fn compile(source: &Path) -> Result<elaborate::CompiledProgram, CliError> {
    let text = fs::read_to_string(source).map_err(|error| {
        CliError::failure(format!("read source '{}': {error}", source.display()))
    })?;
    let parsed = frontend::parse(&text).map_err(|error| {
        CliError::failure(format!("parse source '{}': {error}", source.display()))
    })?;
    elaborate::compile(parsed).map_err(|error| {
        CliError::failure(format!("compile source '{}': {error}", source.display()))
    })
}

fn seal(source: &Path, name: &frontend::Name, revision: &Path) -> Result<(), CliError> {
    let program = compile(source)?;
    let revision_value = program.revision(name).map_err(|error| {
        CliError::failure(format!("resolve Revision '{}': {error}", name.as_str()))
    })?;
    fs::write(revision, wire::serialize(revision_value)).map_err(|error| {
        CliError::failure(format!("write revision '{}': {error}", revision.display()))
    })
}

fn run(source: &Path) -> Result<(), CliError> {
    let compiled = compile(source)?;
    let program = request::resolve(&compiled)
        .map_err(|error| CliError::failure(format!("resolve requests: {error}")))?;
    let output = request::run(&program, request::RunLimits::default())
        .map_err(|error| CliError::failure(format!("run requests: {error}")))?;
    println!("{}", output.canonical_bytes());
    Ok(())
}

fn emit_rust(source: &Path, output: &Path) -> Result<(), CliError> {
    let compiled = compile(source)?;
    let resolved = request::resolve(&compiled)
        .map_err(|error| CliError::failure(format!("resolve requests: {error}")))?;
    let emitted = generated::emit_rust(&resolved)
        .map_err(|error| CliError::failure(format!("emit Rust: {error}")))?;
    fs::write(output, emitted)
        .map_err(|error| CliError::failure(format!("write Rust '{}': {error}", output.display())))
}

fn main() -> ExitCode {
    let result = match parse_command(env::args_os().skip(1)) {
        Ok(Command::Seal {
            source,
            name,
            revision,
        }) => seal(&source, &name, &revision),
        Ok(Command::Run { source }) => run(&source),
        Ok(Command::EmitRust { source, output }) => emit_rust(&source, &output),
        Err(error) => Err(error),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(error.status)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_command};

    #[test]
    fn accepts_only_native_commands() {
        assert!(matches!(
            parse_command(["run".into(), "input.clause".into()]),
            Ok(Command::Run { .. })
        ));
        assert!(matches!(
            parse_command([
                "seal".into(),
                "input.clause".into(),
                "graph".into(),
                "graph.rev".into()
            ]),
            Ok(Command::Seal { .. })
        ));
        assert!(matches!(
            parse_command([
                "emit-rust".into(),
                "input.clause".into(),
                "output.rs".into()
            ]),
            Ok(Command::EmitRust { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn preserves_non_utf8_filesystem_paths() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

        let source = OsString::from_vec(b"input-\xff.clause".to_vec());
        let output = OsString::from_vec(b"output-\xff.rs".to_vec());
        let Ok(Command::EmitRust {
            source: parsed_source,
            output: parsed_output,
        }) = parse_command([OsString::from("emit-rust"), source.clone(), output.clone()])
        else {
            panic!("non-UTF-8 filesystem paths should parse");
        };

        assert_eq!(parsed_source, PathBuf::from(source));
        assert_eq!(parsed_output, PathBuf::from(output));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_utf8_revision_names_as_usage_errors() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let Err(error) = parse_command([
            OsString::from("seal"),
            OsString::from("input.clause"),
            OsString::from_vec(b"revision-\xff".to_vec()),
            OsString::from("output.revision"),
        ]) else {
            panic!("non-UTF-8 Revision names should be rejected");
        };

        assert_eq!(error.status, 2);
        assert!(
            error
                .message
                .starts_with("REVISION_NAME must be valid UTF-8")
        );
    }
}
