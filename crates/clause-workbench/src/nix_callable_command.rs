use std::{ffi::OsString, path::Path, process::ExitCode};
use clause_package::render_nix_callable_v1;
use clause_workbench::ResidentSourceWorkbenchV1;

pub fn run(arguments: Vec<OsString>) -> ExitCode {
    match compile(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => { eprintln!("compile-nix failed: {error}"); ExitCode::FAILURE }
    }
}
fn compile(arguments: &[OsString]) -> Result<(), String> {
    let [source, entry, output] = arguments else {
        return Err("usage: clause-workbench compile-nix SOURCE.clause ENTRY OUTPUT.nix".into());
    };
    let workbench = ResidentSourceWorkbenchV1::open_file(Path::new(source)).map_err(|e| format!("source open: {e}"))?;
    let checked = workbench.checked_source_package().map_err(|e| e.to_string())?;
    let entry = entry.to_str().ok_or("entry must be UTF-8")?;
    let callable = checked.callables.iter().find(|c| c.exported && c.designation == entry.as_bytes()).ok_or("unknown exported Nix entry")?;
    let output = Path::new(output);
    if output.extension().is_none_or(|e| e != "nix") { return Err("output must end in .nix".into()); }
    let rendered = render_nix_callable_v1(callable)?;
    std::fs::write(output, rendered).map_err(|e| e.to_string())
}
