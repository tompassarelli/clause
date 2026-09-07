use clause_runtime::{begin_executable_source_profile_v1, finish_executable_source_profile_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 { return Err("expected source path, selected expression, replacement expression".into()); }
    let source = std::fs::read(&args[0])?;
    let mut workbench = ResidentSourceWorkbenchV1::open_continuous(&source)?;
    for (selected, replacement) in [(&args[1], &args[2]), (&args[2], &args[1]), (&args[1], &args[2])] {
        let effect = workbench.scalar_effects()?.into_iter().find(|effect| effect.expression == selected.as_bytes()).ok_or("selected expression absent")?;
        let old = workbench.generation().handle;
        assert!(begin_executable_source_profile_v1());
        let result = workbench.edit_scalar_effect(old, &effect, replacement.as_bytes());
        let report = finish_executable_source_profile_v1().ok_or("profile still active")?;
        result?;
        assert!(workbench.rejects_stale_handle(old)?);
        workbench.source_continuity()?;
        println!("{}", report.to_json());
    }
    Ok(())
}
