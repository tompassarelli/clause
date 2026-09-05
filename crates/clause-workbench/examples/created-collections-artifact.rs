use clause_workbench::ResidentSourceWorkbenchV1;
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("expected lane-local directory")?,
    );
    std::fs::create_dir_all(&directory)?;
    let source = [
        include_bytes!("../../../test-vectors/authoring/live-encounter.clause").as_slice(),
        b"\n",
        include_bytes!("../../../test-vectors/authoring/created-burn-extension.clause").as_slice(),
    ]
    .concat();
    std::fs::write(directory.join("encounter.clause"), &source)?;
    let mut w = ResidentSourceWorkbenchV1::open(&source)?;
    std::fs::write(directory.join("initial.cwr1"), &w.generation().cwr1)?;
    let effect = w
        .scalar_effects()?
        .into_iter()
        .find(|effect| effect.expression == b"0.0 - ?damage * ?elapsed")
        .ok_or("expected burn effect")?;
    let tick = w.diagnostic_handler_entry(effect.handler)?;
    w.edit_scalar_effect(
        w.generation().handle,
        &effect,
        b"0.0 - ?damage * ?elapsed * 2.0",
    )?;
    let edited_effect = w
        .scalar_effects()?
        .into_iter()
        .find(|effect| effect.expression == b"0.0 - ?damage * ?elapsed * 2.0")
        .ok_or("expected edited effect")?;
    let edited_tick = w.diagnostic_handler_entry(edited_effect.handler)?;
    std::fs::write(directory.join("edited.cwr1"), &w.generation().cwr1)?;
    std::fs::write(
        directory.join("edit.cet1"),
        w.last_source_edit().ok_or("missing source witness")?,
    )?;
    std::fs::write(
        directory.join("entries.json"),
        format!("{{\"tick\":{tick},\"editedTick\":{edited_tick}}}\n"),
    )?;
    // Same non-game fixture, one source-owned physical input for browser use.
    let goal_source = format!(
        "{}\nbind scalar-input GoalDuration to timed-goal\n\non timed-goal ?account ?duration\n  when\n    ?account balance ?balance\n  create\n    ?goal\n      shape: Goal\n  include\n    ?account known goal ?goal\n    ?goal contribution 7.0\n    ?goal remaining ?duration\n",
        include_str!("../../../test-vectors/authoring/created-timed-contributions.clause")
    );
    let goal = ResidentSourceWorkbenchV1::open(goal_source.as_bytes())?;
    std::fs::write(directory.join("goals.clause"), goal_source)?;
    std::fs::write(directory.join("goals.cwr1"), &goal.generation().cwr1)?;
    Ok(())
}
