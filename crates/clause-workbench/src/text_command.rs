use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableValueV1, projected_text_value_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const USAGE: &str =
    "usage: clause-workbench run-text SOURCE.clause HANDLER SUBJECT ROLE [TEXT ...]";

pub fn run(arguments: Vec<OsString>) -> ExitCode {
    match execute(&arguments) {
        Ok(text) => match std::io::stdout().lock().write_all(text.as_bytes()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("run-text output failed: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("run-text failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(arguments: &[OsString]) -> Result<String, String> {
    let [source, handler, subject, role, values @ ..] = arguments else {
        return Err(USAGE.into());
    };
    let utf8 = |value: &OsString| {
        value
            .clone()
            .into_string()
            .map_err(|_| "handler, projection labels, and Text arguments must be UTF-8".to_string())
    };
    let handler = utf8(handler)?;
    let subject = utf8(subject)?;
    let role = utf8(role)?;
    let values = values
        .iter()
        .map(|value| ExecutableValueV1::text(&utf8(value)?).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let source = std::fs::read(source).map_err(|error| format!("source read: {error}"))?;
    let mut workbench = ResidentSourceWorkbenchV1::open(&source)
        .map_err(|error| format!("source open: {error}"))?;
    let occurrence = workbench
        .handler_occurrence(handler.as_bytes(), &values)
        .map_err(|error| format!("handler selection: {error}"))?;
    workbench
        .run_occurrences_to_candidate(&[occurrence])
        .map_err(|error| format!("execution: {error}"))?;
    let admission = workbench
        .admit()
        .map_err(|error| format!("admission: {error}"))?;
    let projection = decode_canonical_term_bytes(&admission.projection.exact_term_bytes())
        .map_err(|error| format!("projection decode: {error}"))?;
    let row = field(&projection, subject.as_bytes())?;
    let value = field(row, role.as_bytes())?;
    projected_text_value_v1(value)
        .map_err(|error| error.to_string())?
        .map(str::to_owned)
        .ok_or_else(|| "selected projection value is not Text".into())
}

fn field<'a>(object: &'a Term, label: &[u8]) -> Result<&'a Term, String> {
    let mut node = object;
    let mut selected = None;
    while let Some(triple) = node.as_triple() {
        let [key, value, rest] = triple.slots();
        let key = key
            .as_atom()
            .filter(|atom| atom.kind() == b"clause/js-field-v1")
            .ok_or_else(|| "malformed projected object field".to_string())?;
        if key.canonical_payload() == label {
            if selected.replace(value).is_some() {
                return Err("ambiguous projected field".into());
            }
        }
        node = rest;
    }
    if node.as_atom().is_none_or(|atom| {
        atom.kind() != b"clause/js-object-end-v1" || !atom.canonical_payload().is_empty()
    }) {
        return Err("malformed projected object end".into());
    }
    selected.ok_or_else(|| format!("projection has no field {}", String::from_utf8_lossy(label)))
}
