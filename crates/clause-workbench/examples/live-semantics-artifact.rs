use clause_runtime::decode_executable_occurrence_v1;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("expected lane-local artifact directory")?,
    );
    std::fs::create_dir_all(&directory)?;
    let mut workbench = ResidentSourceWorkbenchV1::open(include_bytes!(
        "../../../test-vectors/authoring/live-encounter.clause"
    ))?;
    std::fs::write(directory.join("initial.cwr1"), &workbench.generation().cwr1)?;
    let attack =
        decode_executable_occurrence_v1(&workbench.handler_occurrence(b"party-attack", &[])?)?
            .entry;
    let heal =
        decode_executable_occurrence_v1(&workbench.handler_occurrence(b"party-heal", &[])?)?.entry;
    let effect = workbench
        .scalar_effects()?
        .into_iter()
        .find(|effect| effect.expression == b"0.0 - ?damage")
        .ok_or("expected source expression")?;
    workbench.edit_scalar_effect(
        workbench.generation().handle,
        &effect,
        b"0.0 - (?damage * 2.0)",
    )?;
    std::fs::write(directory.join("edited.cwr1"), &workbench.generation().cwr1)?;
    std::fs::write(
        directory.join("edit.cet1"),
        workbench
            .last_source_edit()
            .ok_or("missing compiler witness")?,
    )?;
    let edited_attack =
        decode_executable_occurrence_v1(&workbench.handler_occurrence(b"party-attack", &[])?)?
            .entry;
    let edited_heal =
        decode_executable_occurrence_v1(&workbench.handler_occurrence(b"party-heal", &[])?)?.entry;
    std::fs::write(
        directory.join("entries.json"),
        format!(
            "{{\"attack\":{attack},\"heal\":{heal},\"editedAttack\":{edited_attack},\"editedHeal\":{edited_heal}}}\n"
        ),
    )?;
    Ok(())
}
