use std::ffi::OsStr;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clause_package::CanonicalSourceProductionV1;
use clause_substrate::compiler_package_v3::compiler_package_hash;
use clause_workbench::{
    ResidentSourceGenerationV1, ResidentSourceWorkbenchV1, WorkbenchService,
    render_authoring_card_v1,
};

const USAGE: &str = "usage:\n  clause-workbench\n  clause-workbench source-loop SOURCE.clause\n  clause-workbench authoring-card\n  clause-workbench check-source FILE.clause";

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    match arguments.next().as_deref() {
        None => serve_binary_workbench(),
        Some(command) if command == OsStr::new("source-loop") => {
            let Some(source) = arguments.next() else {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            };
            if arguments.next().is_some() {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            }
            serve_source_loop(Path::new(&source))
        }
        Some(command) if command == OsStr::new("authoring-card") => {
            if arguments.next().is_some() {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            }
            print_authoring_card()
        }
        Some(command) if command == OsStr::new("check-source") => {
            let Some(source) = arguments.next() else {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            };
            if arguments.next().is_some() {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            }
            check_source(Path::new(&source))
        }
        Some(_) => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn serve_binary_workbench() -> ExitCode {
    let mut service = match WorkbenchService::open() {
        Ok(service) => service,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    match service.serve(std::io::stdin().lock(), std::io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print_authoring_card() -> ExitCode {
    let card = render_authoring_card_v1();
    let mut output = std::io::stdout().lock();
    match output
        .write_all(card.as_bytes())
        .and_then(|()| output.flush())
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("authoring card write failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn check_source(source: &Path) -> ExitCode {
    let startup = Instant::now();
    let exact_source = match std::fs::read(source) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("source read failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let workbench = match ResidentSourceWorkbenchV1::open(&exact_source) {
        Ok(workbench) => workbench,
        Err(error) => {
            eprintln!("source check failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let unsupported_handlers = workbench
        .generation()
        .unsupported
        .iter()
        .filter(|unsupported| unsupported.production == CanonicalSourceProductionV1::Handler)
        .collect::<Vec<_>>();
    if !unsupported_handlers.is_empty() {
        for unsupported in unsupported_handlers {
            eprintln!(
                "source check found unsupported {:?} production at bytes {}..{}",
                unsupported.production, unsupported.origin.start, unsupported.origin.end
            );
        }
        return ExitCode::FAILURE;
    }
    let mut output = std::io::stdout().lock();
    match write_generation(
        &mut output,
        "checked",
        workbench.generation(),
        startup.elapsed(),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("source check result write failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn serve_source_loop(source: &Path) -> ExitCode {
    let startup = Instant::now();
    let exact_source = match std::fs::read(source) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("source read failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut workbench = match ResidentSourceWorkbenchV1::open(&exact_source) {
        Ok(workbench) => workbench,
        Err(error) => {
            eprintln!("resident source workbench failed to open: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut output = std::io::stdout().lock();
    if write_generation(
        &mut output,
        "opened",
        workbench.generation(),
        startup.elapsed(),
    )
    .is_err()
    {
        return ExitCode::FAILURE;
    }
    let input = std::io::stdin().lock();
    for line in input.lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("source-loop input failed: {error}");
                return ExitCode::FAILURE;
            }
        };
        let command = line.trim();
        let reload_source = if command == "hotReload" {
            Some(source)
        } else {
            command
                .strip_prefix("hotReload ")
                .map(|path| Path::new(path.trim()))
        };
        let started = Instant::now();
        let result = if let Some(reload_source) = reload_source {
            std::fs::read(reload_source)
                .map_err(|error| format!("source read failed: {error}"))
                .and_then(|source| {
                    workbench
                        .hot_reload(&source)
                        .map_err(|error| error.to_string())
                })
                .and_then(|generation| {
                    write_generation(&mut output, "reloaded", &generation, started.elapsed())
                        .map_err(|error| error.to_string())
                })
        } else {
            match command {
            "run" => workbench
                .run_to_candidate()
                .map_err(|error| error.to_string())
                .and_then(|candidate| {
                    writeln!(
                        output,
                        "candidate generation={} base={} candidate={} stateRevisions={} hidden=true",
                        candidate.handle.generation,
                        hex(candidate.base.as_bytes()),
                        hex(candidate.candidate.as_bytes()),
                        candidate.state_revision_count,
                    )
                    .and_then(|()| output.flush())
                    .map_err(|error| error.to_string())
                }),
            "admit" => workbench
                .admit()
                .map_err(|error| error.to_string())
                .and_then(|admission| {
                    writeln!(
                        output,
                        "admitted generation={} predecessor={} successor={} stateRevisions={} projection={}",
                        admission.handle.generation,
                        hex(admission.predecessor.as_bytes()),
                        hex(admission.successor.as_bytes()),
                        admission.state_revision_count,
                        hex(&admission.projection.exact_term_bytes),
                    )
                    .and_then(|()| output.flush())
                    .map_err(|error| error.to_string())
                }),
            "quit" => return ExitCode::SUCCESS,
            _ => Err("expected one of: hotReload [SOURCE.clause], run, admit, quit".into()),
        }
        };
        if let Err(error) = result {
            eprintln!("source-loop command failed: {error}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

fn write_generation(
    output: &mut impl Write,
    status: &str,
    generation: &ResidentSourceGenerationV1,
    elapsed: Duration,
) -> std::io::Result<()> {
    writeln!(
        output,
        "{status} generation={} sourcePackage={} cpp1={} cwr1={} cpp1Bytes={} cwr1Bytes={} latencyMicros={}",
        generation.handle.generation,
        hex(generation.source_package.as_bytes()),
        hex(&compiler_package_hash(&generation.cpp1).0),
        hex(&compiler_package_hash(&generation.cwr1).0),
        generation.cpp1.len(),
        generation.cwr1.len(),
        elapsed.as_micros(),
    )?;
    output.flush()
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
