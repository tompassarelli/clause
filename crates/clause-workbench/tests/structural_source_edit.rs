use clause_package::{CanonicalSourceItemReplacementV1, Term};
use clause_runtime::{
    ExecutableInputSourceV1, ExecutableKeyPhaseV1, WasmSessionPhysicalInputV1, WasmSessionTickV1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const BEFORE: &str = include_str!("../../../test-vectors/greywrought/encounter-before.clause");
const AFTER: &str = include_str!("../../../test-vectors/greywrought/encounter-after.clause");

fn field<'a>(mut term: &'a Term, name: &[u8]) -> &'a Term {
    while let Some(triple) = term.as_triple() {
        let [key, value, rest] = triple.slots();
        if key.as_atom().unwrap().canonical_payload() == name {
            return value;
        }
        term = rest;
    }
    panic!("missing field {}", String::from_utf8_lossy(name));
}

fn value<'a>(world: &'a Term, subject: &str, role: &str) -> &'a Term {
    field(field(world, subject.as_bytes()), role.as_bytes())
}

fn number(world: &Term, subject: &str, role: &str) -> f64 {
    f64::from_le_bytes(
        value(world, subject, role)
            .as_atom()
            .unwrap()
            .canonical_payload()
            .try_into()
            .unwrap(),
    )
}

fn tick(w: &mut ResidentSourceWorkbenchV1, revision: &mut u64) -> Term {
    *revision += 1;
    w.tick_to_candidate(WasmSessionTickV1 {
        configuration_revision: *revision,
        fixed_tick_milliseconds: 16,
    })
    .unwrap();
    w.admit().unwrap();
    w.project_current_world().unwrap()
}

fn repair(w: &ResidentSourceWorkbenchV1) -> Vec<CanonicalSourceItemReplacementV1> {
    let next = ResidentSourceWorkbenchV1::open(AFTER.as_bytes()).unwrap();
    let offered = w.source_items().unwrap();
    let new = next.source_items().unwrap();
    let selects: [fn(&[u8]) -> bool; 4] = [
        |text| text.starts_with(b"pilot:"),
        |text| text.starts_with(b"mode pilot "),
        |text| text.starts_with(b"law pilot-priorities"),
        |text| {
            text.starts_with(b"on tick ?workshop")
                && text
                    .windows(b"pilot at".len())
                    .any(|part| part == b"pilot at")
        },
    ];
    selects
        .into_iter()
        .map(|select| {
            let matches = offered
                .iter()
                .filter(|item| select(&item.source))
                .collect::<Vec<_>>();
            let [old] = matches.as_slice() else {
                panic!("ambiguous selected old occurrence")
            };
            let matches = new
                .iter()
                .filter(|item| select(&item.source))
                .collect::<Vec<_>>();
            let [new] = matches.as_slice() else {
                panic!("ambiguous selected replacement")
            };
            CanonicalSourceItemReplacementV1 {
                selected: (*old).clone(),
                replacement: new.source.clone(),
            }
        })
        .collect()
}

#[test]
fn checked_structural_edit_preserves_world_rejects_atomically_and_reopens() {
    let mut w = ResidentSourceWorkbenchV1::open(BEFORE.as_bytes()).unwrap();
    let mut revision = 0;
    tick(&mut w, &mut revision);
    let handle = w.generation().handle;
    w.apply_physical_input(
        handle,
        WasmSessionPhysicalInputV1 {
            input_sequence: 1,
            source: ExecutableInputSourceV1::Keyboard {
                code: b"LaunchExpedition".to_vec(),
                phase: ExecutableKeyPhaseV1::Down,
            },
            value: None,
        },
    )
    .unwrap();
    for _ in 0..12 {
        tick(&mut w, &mut revision);
    }
    let before = w.project_current_world().unwrap();
    assert!(number(&before, "workshop", "position") > 0.0);
    let generation = w.generation().clone();
    let edits = repair(&w);
    let noop = edits
        .iter()
        .map(|op| CanonicalSourceItemReplacementV1 {
            selected: op.selected.clone(),
            replacement: op.selected.source.clone(),
        })
        .collect::<Vec<_>>();
    assert_eq!(w.replace_source_items(handle, &noop).unwrap(), generation);
    revision += 1;
    w.tick_to_candidate(WasmSessionTickV1 {
        configuration_revision: revision,
        fixed_tick_milliseconds: 16,
    })
    .unwrap();
    assert_eq!(w.replace_source_items(handle, &noop).unwrap(), generation);
    assert!(w.replace_source_items(handle, &edits).is_err());
    assert_eq!(w.project_current_world().unwrap(), before);
    w.admit().unwrap();
    let before = w.project_current_world().unwrap();
    let mut invalid = edits.clone();
    invalid[3].replacement =
        b"on tick ?workshop ?dt\n  include\n    ?workshop nonexistent 7.0".to_vec();
    assert!(w.replace_source_items(handle, &invalid).is_err());
    assert_eq!(w.generation(), &generation);
    assert_eq!(w.project_current_world().unwrap(), before);
    assert_eq!(w.exact_source(), BEFORE.as_bytes());
    w.replace_source_items(handle, &edits).unwrap();
    assert_eq!(w.exact_source(), AFTER.as_bytes());
    let after = w.project_current_world().unwrap();
    for (subject, role) in [
        ("workshop", "stock"),
        ("workshop", "cargo"),
        ("workshop", "position"),
        ("workshop", "reserve"),
        ("lance", "health"),
        ("drive", "health"),
        ("legs", "part-health"),
        ("torso", "part-health"),
    ] {
        assert_eq!(
            value(&before, subject, role),
            value(&after, subject, role),
            "{subject}.{role}"
        );
    }
    assert!(w.replace_source_items(handle, &edits).is_err());
    let witness =
        clause_runtime::decode_executable_source_edit_v1(w.last_source_edit().unwrap()).unwrap();
    assert_eq!(
        clause_runtime::encode_executable_source_edit_v1(&witness).unwrap(),
        w.last_source_edit().unwrap()
    );
    let cpp = clause_runtime::decode_executable_physical_plan_v1(&w.generation().cpp1).unwrap();
    let checked = clause_runtime::check_executable_source_edit_v1(
        &witness,
        cpp.program.projection.as_ref().unwrap().template.scope(),
    )
    .unwrap();
    let old = clause_runtime::projected_referent_value_v1(value(&before, "lance", "attached-to"))
        .unwrap()
        .unwrap();
    let new = clause_runtime::projected_referent_value_v1(value(&after, "lance", "attached-to"))
        .unwrap()
        .unwrap();
    let mapped = checked
        .continuity()
        .identities
        .get(&clause_package::CanonicalAllocatedIdentityV1::Formation(
            clause_package::FormationLocalId::new(old.domain()),
        ))
        .unwrap();
    assert_eq!(
        *mapped,
        clause_package::CanonicalAllocatedIdentityV1::Formation(
            clause_package::FormationLocalId::new(new.domain())
        )
    );
    let clause_runtime::ExecutableReferentIdentityV1::Declared(old_id) = old.identity() else {
        panic!("declared attachment")
    };
    let clause_runtime::ExecutableReferentIdentityV1::Declared(new_id) = new.identity() else {
        panic!("declared attachment")
    };
    assert_eq!(
        checked.continuity().identities.get(
            &clause_package::CanonicalAllocatedIdentityV1::Formation(
                clause_package::FormationLocalId::new(*old_id)
            )
        ),
        Some(&clause_package::CanonicalAllocatedIdentityV1::Formation(
            clause_package::FormationLocalId::new(*new_id)
        ))
    );
    let checkpoint = w.checkpoint_admitted().unwrap();
    let mut reopened = ResidentSourceWorkbenchV1::reopen(w.exact_source(), &checkpoint).unwrap();
    assert_eq!(reopened.project_current_world().unwrap(), after);
    assert!(
        number(&tick(&mut reopened, &mut revision), "workshop", "position")
            > number(&after, "workshop", "position")
    );
}

#[test]
fn checked_structural_edit_stops_existing_cooling_and_retreat_oscillation() {
    for retreat in [false, true] {
        let source = BEFORE
            .replace("  phase: \"Workshop\"\n", "  phase: \"Expedition\"\n")
            .replace("  position: 0.0\n", "  position: 8.0\n")
            .replace("  heat: 0.0\n", "  heat: 35.0\n")
            .replace("  action: 0.0\n", "  action: 3.0\n")
            .replace("?stock - ?repair / 4.0", "?stock - ?repair / 8.0");
        let source = if retreat {
            source.replace("  firepower: 9.0\n", "  firepower: 0.0\n")
        } else {
            source
        };
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        let mut revision = 0;
        // Two old ticks expose the saved world's actual bad alternating rule.
        let first = tick(&mut w, &mut revision);
        let second = tick(&mut w, &mut revision);
        assert_ne!(
            number(&first, "workshop", "action"),
            number(&second, "workshop", "action")
        );
        let edits = repair(&w);
        w.replace_source_items(w.generation().handle, &edits)
            .unwrap();
        assert!(
            w.scalar_effects()
                .unwrap()
                .iter()
                .any(|effect| effect.expression == b"?stock - ?repair / 8.0")
        );
        let mut previous = w.project_current_world().unwrap();
        let mut cooling = false;
        let mut withdrawing = false;
        for _ in 0..20 {
            let current = tick(&mut w, &mut revision);
            let action = number(&current, "workshop", "action");
            if retreat {
                if action == 0.0 {
                    withdrawing = true;
                }
                if withdrawing {
                    assert_eq!(action, 0.0);
                    assert!(
                        number(&current, "workshop", "position")
                            < number(&previous, "workshop", "position")
                    );
                }
            } else {
                if cooling && number(&previous, "workshop", "heat") > 25.0 {
                    assert_eq!(action, 3.0);
                }
                cooling |= action == 3.0;
            }
            previous = current;
        }
        assert!(if retreat { withdrawing } else { cooling });
    }
}
