use clause_runtime::{begin_executable_source_profile_v1, finish_executable_source_profile_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if !(3..=4).contains(&args.len()) { return Err("expected source path, selected expression, replacement expression, optional artifact directory".into()); }
    let source = std::fs::read(&args[0])?;
    let mut workbench = ResidentSourceWorkbenchV1::open_continuous(&source)?;
    let artifacts = args.get(3).map(std::path::Path::new);
    if let Some(directory) = artifacts {
        std::fs::create_dir_all(directory)?;
        std::fs::write(directory.join("initial.cwr1"), &workbench.generation().cwr1)?;
        std::fs::write(directory.join("initial.cps1"), workbench.source_preparation()?)?;
        std::fs::write(directory.join("initial.cws1"), session_open(&workbench.generation().cwr1)?)?;
    }
    for (index, (selected, replacement)) in [(&args[1], &args[2]), (&args[2], &args[1]), (&args[1], &args[2])].into_iter().enumerate() {
        let effect = workbench.scalar_effects()?.into_iter().find(|effect| effect.expression == selected.as_bytes()).ok_or("selected expression absent")?;
        let old = workbench.generation().handle;
        assert!(begin_executable_source_profile_v1());
        let result = workbench.edit_scalar_effect(old, &effect, replacement.as_bytes());
        let report = finish_executable_source_profile_v1().ok_or("profile still active")?;
        result?;
        assert!(workbench.rejects_stale_handle(old)?);
        workbench.source_continuity()?;
        if let Some(directory) = artifacts {
            std::fs::write(directory.join(format!("{}.cwr1", index + 1)), &workbench.generation().cwr1)?;
            std::fs::write(directory.join(format!("{}.cws1", index + 1)), session_open(&workbench.generation().cwr1)?)?;
            std::fs::write(directory.join(format!("{}.cet1", index + 1)), workbench.last_source_edit().ok_or("checked edit omitted witness")?)?;
            std::fs::write(directory.join(format!("{}.csc1", index + 1)), workbench.source_continuity_bytes()?)?;
            std::fs::write(directory.join(format!("{}.continuity-term", index + 1)), clause_package::canonical_term_bytes(&workbench.source_continuity()?)?)?;
        }
        println!("{}", report.to_json());
    }
    Ok(())
}

fn session_open(cwr1: &[u8]) -> Result<Vec<u8>, clause_runtime::WasmProcessStatusV1> {
    use clause_runtime::*;
    let request = decode_wasm_process_request_v1(cwr1)?;
    encode_wasm_session_open_v1(&WasmSessionOpenV1 {
        package_bytes: request.package_bytes, application: request.application,
        physical_plan_bytes: request.physical_plan_bytes, authority: request.authority,
        allocation: WasmSessionAllocationV1::New,
        limits: WasmSessionLimitsV1 { max_commands: 4096,
            command_bytes: WASM_SESSION_COMMAND_LIMIT_V1 as u32,
            event_bytes: WASM_SESSION_EVENT_LIMIT_V1 as u32,
            trace_retention: WasmSessionTraceRetentionV1::CurrentAdmission },
    })
}
