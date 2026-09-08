use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use clause_package::lower_javascript_v1;
use clause_workbench::ResidentSourceWorkbenchV1;

pub fn run(arguments: Vec<OsString>) -> ExitCode {
    match compile(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("compile-js failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn compile(arguments: &[OsString]) -> Result<(), String> {
    let [source, output] = arguments else {
        return Err("usage: clause-workbench compile-js FILE.clause OUTPUT.js".into());
    };
    let output = Path::new(output);
    if output.extension().is_none_or(|extension| extension != "js") {
        return Err("the output filename must end in .js".into());
    }
    let workbench = ResidentSourceWorkbenchV1::open_file(Path::new(source))
        .map_err(|error| format!("source open: {error}"))?;
    let checked = workbench.checked_source_package().map_err(|error| error.to_string())?;
    let artifacts = lower_javascript_v1(&checked).map_err(|error| error.to_string())?;
    std::fs::write(output, artifacts.module)
        .map_err(|error| format!("JavaScript output: {error}"))?;
    std::fs::write(output.with_extension("d.ts"), artifacts.declarations)
        .map_err(|error| format!("declaration output: {error}"))?;
    Ok(())
}
