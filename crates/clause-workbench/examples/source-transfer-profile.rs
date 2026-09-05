//! Exact consumer/source-edit fixture, not an alternate evaluator or benchmark
//! of unchecked compilation. Run only in the admitted serialized heavy slot.
use clause_runtime::{
    ExecutableInputSourceV1, ExecutableKeyPhaseV1, ExecutableValueV1, WasmSessionPhysicalInputV1,
    WasmSessionTickV1, begin_executable_source_profile_v1, finish_executable_source_profile_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{error::Error, path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let directory = PathBuf::from(
        args.next()
            .ok_or("expected lane-local artifact directory")?,
    );
    let compiler = args.next().ok_or("expected exact compiler commit label")?;
    if compiler.len() != 40
        || !compiler
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("compiler label must be exact lower-case Git commit".into());
    }
    let samples: usize = args.next().unwrap_or_else(|| "3".into()).parse()?;
    if !(1..=5).contains(&samples) || args.next().is_some() {
        return Err("expected 1..5 samples".into());
    }
    std::fs::create_dir_all(&directory)?;
    let encounter = include_bytes!("../../../test-vectors/authoring/live-encounter.clause");
    let collection = [
        encounter.as_slice(),
        b"\n",
        include_bytes!("../../../test-vectors/authoring/created-burn-extension.clause").as_slice(),
    ]
    .concat();
    let mut all = Vec::new();
    for (name, source, created) in [
        ("encounter", encounter.as_slice(), false),
        ("collections", collection.as_slice(), true),
    ] {
        let path = directory.join(name);
        std::fs::create_dir_all(&path)?;
        std::fs::write(path.join("source.clause"), source)?;
        let mut observations = Vec::new();
        for index in 0..=samples * 2 {
            // One warm-up, then alternating profiling off/on. Opening, inputs,
            // Admission, file I/O and diagnostic reads are outside the timer.
            let enabled = index > 0 && index % 2 == 0;
            let mut w = ResidentSourceWorkbenchV1::open(source)?;
            let source_package = w
                .generation()
                .source_package
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            if index == 0 {
                std::fs::write(path.join("initial.cwr1"), &w.generation().cwr1)?;
            }
            let key = |code: &[u8]| ExecutableInputSourceV1::Keyboard {
                code: code.to_vec(),
                phase: ExecutableKeyPhaseV1::Down,
            };
            let mut sequence = 0;
            for (input, value) in [(key(b"BeginEncounter"), None), (key(b"Attack"), None)] {
                sequence += 1;
                w.apply_physical_input(
                    w.generation().handle,
                    WasmSessionPhysicalInputV1 {
                        input_sequence: sequence,
                        source: input,
                        value,
                    },
                )?;
            }
            if created {
                for duration in [1.0, 3.0] {
                    sequence += 1;
                    w.apply_physical_input(
                        w.generation().handle,
                        WasmSessionPhysicalInputV1 {
                            input_sequence: sequence,
                            source: ExecutableInputSourceV1::Scalar {
                                channel: b"IgniteDuration".to_vec(),
                            },
                            value: Some(ExecutableValueV1::number(duration)?),
                        },
                    )?;
                }
            }
            w.tick_to_candidate(WasmSessionTickV1 {
                configuration_revision: 1,
                fixed_tick_milliseconds: 16,
            })?;
            w.admit()?;
            let effect = w
                .scalar_effects()?
                .into_iter()
                .find(|effect| effect.expression == b"0.0 - ?damage")
                .ok_or("missing exact offered attack effect")?;
            let old = w.generation().handle;
            if enabled && !begin_executable_source_profile_v1() {
                return Err("profile already active".into());
            }
            let started = Instant::now();
            let result = w.edit_scalar_effect(old, &effect, b"0.0 - (?damage * 2.0)");
            let wall = started.elapsed().as_secs_f64() * 1000.0;
            let profile = if enabled {
                finish_executable_source_profile_v1()
                    .ok_or("unsettled profile scopes")?
                    .to_json()
            } else {
                "null".into()
            };
            result?;
            assert!(w.rejects_stale_handle(old)?);
            if index == 0 {
                std::fs::write(path.join("edited.cwr1"), &w.generation().cwr1)?;
                std::fs::write(
                    path.join("edit.cet1"),
                    w.last_source_edit().ok_or("missing checked witness")?,
                )?;
            }
            observations.push(format!("{{\"index\":{index},\"warmup\":{},\"profiled\":{enabled},\"sourcePackage\":\"{source_package}\",\"wallMs\":{wall},\"profile\":{profile}}}", index == 0));
        }
        all.push(format!("\"{name}\":[{}]", observations.join(",")));
    }
    let report = format!(
        "{{\"compiler\":\"{compiler}\",\"measurement\":\"native resident edit includes preparation and checked transfer; no renderer\",\"samplesPerMode\":{samples},\"variants\":{{{}}}}}\n",
        all.join(",")
    );
    std::fs::write(directory.join("native.json"), &report)?;
    println!("{report}");
    Ok(())
}
