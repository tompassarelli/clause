use std::error::Error;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use clause_package::Term;
use clause_runtime::{
    ExecutableValueV1, decode_executable_occurrence_v1, projected_referent_value_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let Some(source_path) = arguments.next() else {
        eprintln!("usage: scheduling_source_server SOURCE.clause");
        return ExitCode::FAILURE;
    };
    if arguments.next().is_some() {
        eprintln!("usage: scheduling_source_server SOURCE.clause");
        return ExitCode::FAILURE;
    }
    serve(Path::new(&source_path))
}

fn serve(source_path: &Path) -> ExitCode {
    let source = match std::fs::read(source_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("resident schedule read failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    let mut workbench = match ResidentSourceWorkbenchV1::open(&source) {
        Ok(workbench) => workbench,
        Err(error) => {
            eprintln!("resident schedule failed to open: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut output = std::io::stdout().lock();
    if let Err(error) = write_generation(&mut output, &workbench, started, false, true) {
        eprintln!("resident schedule generation failed: {error}");
        return ExitCode::FAILURE;
    }

    for line in std::io::stdin().lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("resident schedule command read failed: {error}");
                return ExitCode::FAILURE;
            }
        };
        let command = line.trim();
        match command {
            "quit" => return ExitCode::SUCCESS,
            "prepare" => {
                let result = workbench.source_preparation().map_err(|error| error.to_string())
                    .and_then(|bytes| {
                        writeln!(output, "preparation\t{}\t{}", workbench.generation().handle.generation, hex(&bytes))
                            .and_then(|()| output.flush()).map_err(|error| error.to_string())
                    });
                if let Err(error) = result && write_error(&mut output, &error).is_err() { return ExitCode::FAILURE; }
            },
            _ if command.starts_with("edit\t") => {
                let started = Instant::now();
                let result =
                    edit_scalar_effect(&mut workbench, source_path, command).and_then(|changed| {
                        write_generation(&mut output, &workbench, started, changed, false)
                            .map_err(|error| error.to_string())
                    });
                if let Err(error) = result
                    && write_error(&mut output, &error).is_err()
                {
                    return ExitCode::FAILURE;
                }
            }
            command => {
                if write_error(&mut output, &format!("unknown resident command: {command}"))
                    .is_err()
                {
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    ExitCode::SUCCESS
}

fn write_generation(
    output: &mut impl Write,
    workbench: &ResidentSourceWorkbenchV1,
    started: Instant,
    edited: bool,
    include_preparation: bool,
) -> Result<(), Box<dyn Error>> {
    let task = referent(workbench, b"prototype")?;
    let root = referent(workbench, b"approval")?;
    let entries = [
        (b"resolve".as_slice(), root),
        (b"complete".as_slice(), task.clone()),
        (b"extend".as_slice(), task),
    ]
    .into_iter()
    .map(|(designation, argument)| {
        Ok((
            designation,
            decode_executable_occurrence_v1(
                &workbench.handler_occurrence(designation, &[argument])?,
            )?
            .entry,
        ))
    })
    .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    writeln!(
        output,
        "generation\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        workbench.generation().handle.generation,
        started.elapsed().as_micros(),
        hex(&workbench.generation().cwr1),
        if edited {
            hex(workbench
                .last_source_edit()
                .ok_or("edited generation omitted checked transaction")?)
        } else {
            String::new()
        },
        scalar_catalog(workbench, &entries)?,
        entries
            .iter()
            .find(|(designation, _)| *designation == b"complete")
            .ok_or("schedule omitted completion handler")?
            .1,
        if include_preparation { hex(&workbench.source_preparation()?) } else { String::new() },
    )?;
    output.flush()?;
    Ok(())
}

fn referent(
    workbench: &ResidentSourceWorkbenchV1,
    designation: &[u8],
) -> Result<ExecutableValueV1, Box<dyn Error>> {
    let frame = workbench.project_current_world()?;
    let projected = field(field(&frame, designation)?, b"$referent")?;
    Ok(ExecutableValueV1::Referent(
        projected_referent_value_v1(projected)?.ok_or("schedule referent was not projected")?,
    ))
}

fn field<'a>(term: &'a Term, key: &[u8]) -> Result<&'a Term, Box<dyn Error>> {
    let mut node = term;
    loop {
        let triple = node
            .as_triple()
            .ok_or("schedule projection field is absent")?;
        let [name, value, rest] = triple.slots();
        if name
            .as_atom()
            .ok_or("schedule projection key is not an atom")?
            .canonical_payload()
            == key
        {
            return Ok(value);
        }
        node = rest;
    }
}

fn scalar_catalog(
    workbench: &ResidentSourceWorkbenchV1,
    entries: &[(&[u8], u16)],
) -> Result<String, Box<dyn Error>> {
    Ok(workbench
        .scalar_effects()?
        .iter()
        .enumerate()
        .map(|(index, effect)| {
            let entry = workbench.diagnostic_handler_entry(effect.handler)?;
            let designation = entries
                .iter()
                .find_map(|(designation, candidate)| (*candidate == entry).then_some(*designation))
                .unwrap_or(b"");
            Ok(format!(
                "{index},{entry},{},{},{},{},{}",
                effect.expression_origin.start,
                effect.expression_origin.end,
                hex(effect.artifact.as_bytes()),
                hex(&effect.expression),
                hex(designation),
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?
        .join(";"))
}

fn edit_scalar_effect(
    workbench: &mut ResidentSourceWorkbenchV1,
    source_path: &Path,
    command: &str,
) -> Result<bool, String> {
    let mut fields = command.split('\t');
    if fields.next() != Some("edit") {
        return Err("invalid source edit command".into());
    }
    let generation = fields
        .next()
        .ok_or("source edit omitted generation")?
        .parse::<u32>()
        .map_err(|_| "source edit generation is invalid")?;
    let index = fields
        .next()
        .ok_or("source edit omitted catalog index")?
        .parse::<usize>()
        .map_err(|_| "source edit catalog index is invalid")?;
    let expression = unhex(fields.next().ok_or("source edit omitted expression")?)?;
    if fields.next().is_some() {
        return Err("source edit command has trailing fields".into());
    }
    let captured = workbench.generation().handle;
    if generation != captured.generation {
        return Err("stale structured source operation".into());
    }
    let selected = workbench
        .scalar_effects()
        .map_err(|error| error.to_string())?
        .get(index)
        .cloned()
        .ok_or("source edit catalog index is absent")?;
    let next = workbench
        .edit_scalar_effect(captured, &selected, &expression)
        .map_err(|error| format!("structured edit: {error}"))?;
    let changed = next.handle != captured;
    if changed {
        persist_exact_source(
            source_path,
            next.handle.generation,
            workbench.exact_source(),
        )?;
    }
    Ok(changed)
}

fn persist_exact_source(
    source_path: &Path,
    generation: u32,
    exact_source: &[u8],
) -> Result<(), String> {
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("source path has no UTF-8 file name")?;
    let temporary = source_path.with_file_name(format!(
        ".{file_name}.scheduling-edit-{}-{generation}",
        std::process::id(),
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("source persistence create failed: {error}"))?;
    if let Err(error) = file.write_all(exact_source).and_then(|()| file.sync_all()) {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!("source persistence write failed: {error}"));
    }
    std::fs::rename(&temporary, source_path)
        .map_err(|error| format!("source persistence install failed: {error}"))?;
    Ok(())
}

fn write_error(output: &mut impl Write, error: &str) -> std::io::Result<()> {
    writeln!(output, "error\t{}", hex(error.as_bytes()))?;
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

fn unhex(value: &str) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err("hex source edit expression has odd length".into());
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => Ok(byte - b'0'),
                b'a'..=b'f' => Ok(byte - b'a' + 10),
                _ => Err("hex source edit expression is invalid".to_string()),
            };
            Ok((digit(pair[0])? << 4) | digit(pair[1])?)
        })
        .collect()
}
