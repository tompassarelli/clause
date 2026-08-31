use std::process::ExitCode;

use clause_workbench::WorkbenchService;

fn main() -> ExitCode {
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
