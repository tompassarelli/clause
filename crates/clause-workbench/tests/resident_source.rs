use std::{fmt::Write as _, time::Instant};

use clause_package::{
    ApplicationId, ProgramRevisionPreimage, Term, canonical_term_bytes, check_process_package,
    decode_canonical_term_bytes, decode_process_package, read_canonical_source_v1,
};
use clause_runtime::{
    ExecutableInputSourceV1, ExecutableKeyPhaseV1, ExecutableValueKindV1, ExecutableValueV1,
    ForkedProcessBranchV1, WASM_PROCESS_REQUEST_LIMIT_V1, decode_executable_occurrence_v1,
    decode_executable_physical_plan_v1, decode_wasm_process_request_v1,
    encode_executable_physical_plan_v1, open_fresh_persistent_process_session_v1,
    projected_relation_table_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world.clause"
));

const REFERENT_INPUT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/authoring/referent-input-transition.clause"
));

const CROSS_SUBJECT_TARGET: &[u8] =
    include_bytes!("../../../test-vectors/authoring/cross-subject-referent-target.clause");
const ACCOUNT_CONTRIBUTIONS: &[u8] =
    include_bytes!("../../../test-vectors/authoring/selected-account-contributions.clause");

fn wide_source_projection_fixture() -> String {
    let mut source = String::from(concat!(
        "F64\n",
        "Meter\n",
        "\n",
        "Vec3:\n",
        "  x: F64\n",
        "  y: F64\n",
        "  z: F64\n",
        "\n",
        "reading-with-a-source-owned-designation-that-makes-the-canonical-projection-wide-enough-for-real-programs:\n",
        "  ?meter shape Meter\n",
        "  ?reading shape Vec3\n",
        "  ?meter:\n",
        "    reading: ?reading\n",
        "mode reading-with-a-source-owned-designation-that-makes-the-canonical-projection-wide-enough-for-real-programs given meter yields reading: one\n",
        "charge-with-a-source-owned-designation-that-makes-the-canonical-projection-wide-enough-for-real-programs:\n",
        "  ?meter shape Meter\n",
        "  ?charge shape F64\n",
        "  ?meter:\n",
        "    charge: ?charge\n",
        "mode charge-with-a-source-owned-designation-that-makes-the-canonical-projection-wide-enough-for-real-programs given meter yields charge: one\n",
    ));
    for index in 0..90 {
        write!(
            source,
            concat!(
                "\nmeter-with-a-source-owned-identity-that-makes-the-canonical-projection-wide-enough-for-real-programs-{}\n",
                "  member of: Meter\n",
                "meter-with-a-source-owned-identity-that-makes-the-canonical-projection-wide-enough-for-real-programs-{} reading Vec3 {{ x: {}.0, y: 0.0, z: 0.0 }}\n",
                "meter-with-a-source-owned-identity-that-makes-the-canonical-projection-wide-enough-for-real-programs-{} charge 0.0\n",
            ),
            index,
            index,
            index,
            index,
        )
        .unwrap();
    }
    source.push_str(concat!(
        "\n",
        "on measure ?meter\n",
        "  when\n",
        "    ?meter charge ?value\n",
        "  withdraw\n",
        "    ?meter charge ?value\n",
        "  include\n",
        "    ?meter charge ?value + 1.0\n",
    ));
    source
}

fn source_artifact_hex(source: &[u8]) -> String {
    read_canonical_source_v1(source)
        .unwrap()
        .artifact()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn physical_plan_source_artifact(cpp1: &[u8]) -> String {
    let plan = decode_executable_physical_plan_v1(cpp1).unwrap();
    projected_text(projected_object_field(
        plan.source_metadata.as_ref().unwrap(),
        b"artifact",
    ))
    .to_owned()
}

#[test]
fn wide_source_projection_roundtrips_and_retains_checked_edit_identity() {
    let source = wide_source_projection_fixture();
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes())
        .expect("a checked source whose projection exceeds the narrow encoding opens");
    let initial = workbench.generation().clone();
    let plan = decode_executable_physical_plan_v1(&initial.cpp1).unwrap();
    let template =
        canonical_term_bytes(&plan.program.projection.as_ref().unwrap().template).unwrap();
    assert!(template.len() > usize::from(u16::MAX), "{}", template.len());
    assert!(initial.cpp1.len() < WASM_PROCESS_REQUEST_LIMIT_V1);
    assert!(initial.cwr1.len() < WASM_PROCESS_REQUEST_LIMIT_V1);
    assert_eq!(
        encode_executable_physical_plan_v1(&plan).unwrap(),
        initial.cpp1
    );
    assert_eq!(
        physical_plan_source_artifact(&initial.cpp1),
        source_artifact_hex(source.as_bytes())
    );
    let request = decode_wasm_process_request_v1(&initial.cwr1).unwrap();
    assert_eq!(request.physical_plan_bytes, initial.cpp1);

    let effect = workbench
        .scalar_effects()
        .unwrap()
        .into_iter()
        .find(|effect| effect.expression == b"?value + 1.0")
        .unwrap();
    let edited = workbench
        .edit_scalar_effect(initial.handle, &effect, b"?value + 2.0")
        .expect("the checked large-source edit transfers the live generation");
    assert_eq!(edited.handle.generation, initial.handle.generation + 1);
    assert_ne!(edited.source_package, initial.source_package);
    assert_ne!(
        physical_plan_source_artifact(&edited.cpp1),
        physical_plan_source_artifact(&initial.cpp1)
    );
    assert_eq!(
        physical_plan_source_artifact(&edited.cpp1),
        source_artifact_hex(workbench.exact_source())
    );
    assert!(workbench.source_continuity().is_ok());
    assert!(workbench.last_source_edit().is_some());
    assert!(workbench.rejects_stale_handle(initial.handle).unwrap());
    let edited_plan = decode_executable_physical_plan_v1(&edited.cpp1).unwrap();
    assert_eq!(
        encode_executable_physical_plan_v1(&edited_plan).unwrap(),
        edited.cpp1
    );
}

#[test]
fn cross_subject_input_replaces_typed_declared_target() {
    use clause_runtime::{
        WasmSessionPhysicalInputV1, WasmSessionTickV1, projected_referent_value_v1,
    };
    let mut workbench = ResidentSourceWorkbenchV1::open(CROSS_SUBJECT_TARGET).unwrap();
    let handle = workbench.generation().handle;
    workbench
        .tick_to_candidate(WasmSessionTickV1 {
            configuration_revision: 1,
            fixed_tick_milliseconds: 100,
        })
        .unwrap();
    let initial = workbench.admit().unwrap();
    let term = decode_canonical_term_bytes(&initial.projection.exact_term_bytes()).unwrap();
    let second = projected_referent_value_v1(projected_object_field(
        projected_object_field(&term, b"second"),
        b"$referent",
    ))
    .unwrap()
    .unwrap();
    workbench
        .apply_physical_input(
            handle,
            WasmSessionPhysicalInputV1 {
                input_sequence: 1,
                source: ExecutableInputSourceV1::Referent {
                    channel: b"Target".to_vec(),
                },
                value: Some(ExecutableValueV1::Referent(second.clone())),
            },
        )
        .unwrap();
    assert_eq!(workbench.last_projection().unwrap(), &initial.projection);
    workbench
        .tick_to_candidate(WasmSessionTickV1 {
            configuration_revision: 2,
            fixed_tick_milliseconds: 100,
        })
        .unwrap();
    assert_eq!(workbench.last_projection().unwrap(), &initial.projection);
    let admitted = workbench.admit().unwrap();
    let term = decode_canonical_term_bytes(&admitted.projection.exact_term_bytes()).unwrap();
    assert_eq!(
        projected_referent_value_v1(projected_object_field(
            projected_object_field(&term, b"player"),
            b"chosen-target"
        ))
        .unwrap(),
        Some(second)
    );
}

#[test]
fn explicit_contributions_reach_runtime_selected_account_once_per_contributor() {
    use clause_runtime::{
        WasmSessionPhysicalInputV1, WasmSessionTickV1, projected_referent_value_v1,
    };
    let mut workbench = ResidentSourceWorkbenchV1::open(ACCOUNT_CONTRIBUTIONS).unwrap();
    let handle = workbench.generation().handle;
    let tick = |workbench: &mut ResidentSourceWorkbenchV1, revision, milliseconds| {
        workbench
            .tick_to_candidate(WasmSessionTickV1 {
                configuration_revision: revision,
                fixed_tick_milliseconds: milliseconds,
            })
            .unwrap();
        let admitted = workbench.admit().unwrap();
        decode_canonical_term_bytes(&admitted.projection.exact_term_bytes()).unwrap()
    };
    let term = tick(&mut workbench, 1, 100);
    let second = projected_referent_value_v1(projected_object_field(
        projected_object_field(&term, b"second"),
        b"$referent",
    ))
    .unwrap()
    .unwrap();
    let apply = |workbench: &mut ResidentSourceWorkbenchV1, sequence| {
        workbench
            .apply_physical_input(
                handle,
                WasmSessionPhysicalInputV1 {
                    input_sequence: sequence,
                    source: ExecutableInputSourceV1::Keyboard {
                        code: b"Apply".to_vec(),
                        phase: ExecutableKeyPhaseV1::Down,
                    },
                    value: None,
                },
            )
            .unwrap();
    };
    apply(&mut workbench, 1);
    let term = tick(&mut workbench, 2, 100);
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"first"),
            b"balance"
        )),
        118.0
    );
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"second"),
            b"balance"
        )),
        200.0
    );
    apply(&mut workbench, 2);
    let term = tick(&mut workbench, 3, 1000);
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"first"),
            b"balance"
        )),
        118.0
    );
    workbench
        .apply_physical_input(
            handle,
            WasmSessionPhysicalInputV1 {
                input_sequence: 3,
                source: ExecutableInputSourceV1::Referent {
                    channel: b"Choose".to_vec(),
                },
                value: Some(ExecutableValueV1::Referent(second)),
            },
        )
        .unwrap();
    apply(&mut workbench, 4);
    let term = tick(&mut workbench, 4, 100);
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"first"),
            b"balance"
        )),
        118.0
    );
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"second"),
            b"balance"
        )),
        218.0
    );
}

#[test]
fn overlapping_ordinary_replacements_reject_the_whole_step() {
    use clause_runtime::{WasmSessionPhysicalInputV1, WasmSessionTickV1};
    let source = std::str::from_utf8(ACCOUNT_CONTRIBUTIONS).unwrap()
        .replace("  withdraw\n    ?contributor cooldown ?cooldown\n  include\n    ?contributor cooldown 1.0\n  accumulate\n    ?account balance ?amount",
                 "  withdraw\n    ?contributor cooldown ?cooldown\n    ?account balance ?balance\n  include\n    ?contributor cooldown 1.0\n    ?account balance ?balance + ?amount");
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let mut native =
        open_fresh_persistent_process_session_v1(&workbench.generation().cwr1).unwrap();
    let error = native
        .apply_typed_physical_input(
            native.runtime_session(),
            &ExecutableInputSourceV1::Keyboard {
                code: b"Apply".to_vec(),
                phase: ExecutableKeyPhaseV1::Down,
            },
            None,
        )
        .unwrap_err();
    assert!(
        error.to_string().contains("ConflictingStateEffects"),
        "{error}"
    );
    workbench
        .tick_to_candidate(WasmSessionTickV1 {
            configuration_revision: 1,
            fixed_tick_milliseconds: 100,
        })
        .unwrap();
    let initial = workbench.admit().unwrap();
    let error = workbench
        .apply_physical_input(
            workbench.generation().handle,
            WasmSessionPhysicalInputV1 {
                input_sequence: 1,
                source: ExecutableInputSourceV1::Keyboard {
                    code: b"Apply".to_vec(),
                    phase: ExecutableKeyPhaseV1::Down,
                },
                value: None,
            },
        )
        .unwrap_err();
    assert!(
        error.to_string().contains("InputConfigurationRejected"),
        "{error}"
    );
    assert_eq!(workbench.last_projection().unwrap(), &initial.projection);
    workbench
        .tick_to_candidate(WasmSessionTickV1 {
            configuration_revision: 2,
            fixed_tick_milliseconds: 100,
        })
        .unwrap();
    let term = decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes())
        .unwrap();
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"first"),
            b"balance"
        )),
        100.0
    );
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"alpha"),
            b"cooldown"
        )),
        0.0
    );
}

#[test]
fn actual_party_source_checks_target_range_selection_and_cooldown_before_aggregating() {
    use clause_runtime::{
        WasmSessionPhysicalInputV1, WasmSessionTickV1, projected_referent_value_v1,
    };
    let source =
        include_str!("../../../test-vectors/authoring/targeted-party-contributions.clause");
    for (source, expected) in [
        (source.to_owned(), 80.0),
        (
            source.replace("second selected true", "second selected false"),
            90.0,
        ),
        (
            source.replace("cinder hostile true", "cinder hostile false"),
            100.0,
        ),
        (
            source.replace("attack range 5.0", "attack range 0.5"),
            100.0,
        ),
        (
            source.replace("second action cooldown 0.0", "second action cooldown 1.0"),
            90.0,
        ),
    ] {
        let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        let handle = workbench.generation().handle;
        workbench
            .tick_to_candidate(WasmSessionTickV1 {
                configuration_revision: 1,
                fixed_tick_milliseconds: 100,
            })
            .unwrap();
        let initial = workbench.admit().unwrap();
        let term = decode_canonical_term_bytes(&initial.projection.exact_term_bytes()).unwrap();
        let cinder = projected_referent_value_v1(projected_object_field(
            projected_object_field(&term, b"cinder"),
            b"$referent",
        ))
        .unwrap()
        .unwrap();
        workbench
            .apply_physical_input(
                handle,
                WasmSessionPhysicalInputV1 {
                    input_sequence: 1,
                    source: ExecutableInputSourceV1::Referent {
                        channel: b"Target".to_vec(),
                    },
                    value: Some(ExecutableValueV1::Referent(cinder)),
                },
            )
            .unwrap();
        workbench
            .apply_physical_input(
                handle,
                WasmSessionPhysicalInputV1 {
                    input_sequence: 2,
                    source: ExecutableInputSourceV1::Keyboard {
                        code: b"Attack".to_vec(),
                        phase: ExecutableKeyPhaseV1::Down,
                    },
                    value: None,
                },
            )
            .unwrap();
        assert_eq!(workbench.last_projection().unwrap(), &initial.projection);
        workbench
            .tick_to_candidate(WasmSessionTickV1 {
                configuration_revision: 2,
                fixed_tick_milliseconds: 100,
            })
            .unwrap();
        assert_eq!(workbench.last_projection().unwrap(), &initial.projection);
        let term =
            decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes())
                .unwrap();
        assert_eq!(
            projected_number(projected_object_field(
                projected_object_field(&term, b"cinder"),
                b"vitality"
            )),
            expected
        );
        assert_eq!(
            projected_number(projected_object_field(
                projected_object_field(&term, b"first"),
                b"vitality"
            )),
            100.0
        );
    }
    let mut workbench = ResidentSourceWorkbenchV1::open(include_bytes!(
        "../../../test-vectors/authoring/targeted-party-attack-conflict.clause"
    ))
    .unwrap();
    workbench
        .tick_to_candidate(WasmSessionTickV1 {
            configuration_revision: 1,
            fixed_tick_milliseconds: 100,
        })
        .unwrap();
    let initial = workbench.admit().unwrap();
    let term = decode_canonical_term_bytes(&initial.projection.exact_term_bytes()).unwrap();
    let target = projected_referent_value_v1(projected_object_field(
        projected_object_field(&term, b"cinder"),
        b"$referent",
    ))
    .unwrap()
    .unwrap();
    workbench
        .apply_physical_input(
            workbench.generation().handle,
            WasmSessionPhysicalInputV1 {
                input_sequence: 1,
                source: ExecutableInputSourceV1::Referent {
                    channel: b"Target".to_vec(),
                },
                value: Some(ExecutableValueV1::Referent(target)),
            },
        )
        .unwrap();
    assert!(
        workbench
            .apply_physical_input(
                workbench.generation().handle,
                WasmSessionPhysicalInputV1 {
                    input_sequence: 2,
                    source: ExecutableInputSourceV1::Keyboard {
                        code: b"Attack".to_vec(),
                        phase: ExecutableKeyPhaseV1::Down
                    },
                    value: None
                }
            )
            .is_err()
    );
    workbench
        .tick_to_candidate(WasmSessionTickV1 {
            configuration_revision: 2,
            fixed_tick_milliseconds: 100,
        })
        .unwrap();
    let term = decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes())
        .unwrap();
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"cinder"),
            b"vitality"
        )),
        100.0
    );
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"first"),
            b"action-cooldown"
        )),
        0.0
    );
}

#[test]
fn additive_effects_require_numeric_present_targets_and_cannot_be_nested() {
    use clause_runtime::ExecutableExpressionV1 as E;
    let workbench = ResidentSourceWorkbenchV1::open(ACCOUNT_CONTRIBUTIONS).unwrap();
    let plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1).unwrap();
    assert_eq!(
        decode_executable_physical_plan_v1(&encode_executable_physical_plan_v1(&plan).unwrap())
            .unwrap(),
        plan
    );
    let (rule_index, assignment_index) = plan
        .program
        .rules
        .iter()
        .enumerate()
        .find_map(|(ri, rule)| {
            rule.assignments
                .iter()
                .position(|(_, e)| matches!(e, E::Accumulate(_)))
                .map(|ai| (ri, ai))
        })
        .unwrap();
    let mut nested = plan.clone();
    let (_, expression) = &mut nested.program.rules[rule_index].assignments[assignment_index];
    *expression = E::Accumulate(Box::new(expression.clone()));
    assert!(encode_executable_physical_plan_v1(&nested).is_err());
    let mut predicate = plan.clone();
    predicate.program.rules[rule_index]
        .predicates
        .push(E::Accumulate(Box::new(E::Constant(
            ExecutableValueV1::number(1.0).unwrap(),
        ))));
    assert!(encode_executable_physical_plan_v1(&predicate).is_err());
    let mut absent = plan.clone();
    let target = absent.program.rules[rule_index].assignments[assignment_index].0;
    absent.program.rules[rule_index]
        .required_present
        .retain(|slot| *slot != target);
    assert!(encode_executable_physical_plan_v1(&absent).is_err());
    let mut wrong_kind = plan;
    wrong_kind.program.initial_configuration[usize::from(target)] =
        ExecutableValueV1::Boolean(true);
    assert!(encode_executable_physical_plan_v1(&wrong_kind).is_err());
    let bool_target = std::str::from_utf8(ACCOUNT_CONTRIBUTIONS).unwrap().replace(
        "  accumulate\n    ?account balance ?amount",
        "  accumulate\n    ?account enabled ?amount",
    );
    assert!(ResidentSourceWorkbenchV1::open(bool_target.as_bytes()).is_err());
    let bool_delta = std::str::from_utf8(ACCOUNT_CONTRIBUTIONS).unwrap().replace(
        "  accumulate\n    ?account balance ?amount",
        "  accumulate\n    ?account balance true",
    );
    assert!(ResidentSourceWorkbenchV1::open(bool_delta.as_bytes()).is_err());
    let mixed = std::str::from_utf8(ACCOUNT_CONTRIBUTIONS).unwrap()
        .replace("  withdraw\n    ?contributor cooldown ?cooldown\n  include\n    ?contributor cooldown 1.0", "  withdraw\n    ?contributor cooldown ?cooldown\n    ?account balance ?balance\n  include\n    ?contributor cooldown 1.0\n    ?account balance ?balance");
    assert!(ResidentSourceWorkbenchV1::open(mixed.as_bytes()).is_err());
}

#[test]
fn typed_physical_input_selects_exact_occurrence_and_preserves_admission_and_generation() {
    use clause_runtime::{
        ExecutableReferentV1, WasmSessionPhysicalInputV1, WasmSessionTickV1,
        projected_referent_value_v1,
    };
    let mut workbench = ResidentSourceWorkbenchV1::open(REFERENT_INPUT).unwrap();
    let old_handle = workbench.generation().handle;
    let tick = |workbench: &mut ResidentSourceWorkbenchV1, revision| {
        workbench
            .tick_to_candidate(WasmSessionTickV1 {
                configuration_revision: revision,
                fixed_tick_milliseconds: 100,
            })
            .unwrap();
    };
    tick(&mut workbench, 1);
    let initial = workbench.admit().unwrap();
    let term = decode_canonical_term_bytes(&initial.projection.exact_term_bytes()).unwrap();
    let first = projected_referent_value_v1(projected_object_field(
        projected_object_field(&term, b"first"),
        b"$referent",
    ))
    .unwrap()
    .unwrap();
    let second = projected_referent_value_v1(projected_object_field(
        projected_object_field(&term, b"second"),
        b"$referent",
    ))
    .unwrap()
    .unwrap();
    assert_eq!(first.domain(), second.domain());
    assert_ne!(first.identity(), second.identity());
    let input = |sequence, value| WasmSessionPhysicalInputV1 {
        input_sequence: sequence,
        source: ExecutableInputSourceV1::Referent {
            channel: b"Pick".to_vec(),
        },
        value: Some(ExecutableValueV1::Referent(value)),
    };
    for invalid in [
        ExecutableReferentV1::declared(first.domain() + 1, 1),
        ExecutableReferentV1::declared(first.domain(), u32::MAX),
    ] {
        assert!(
            workbench
                .apply_physical_input(old_handle, input(1, invalid))
                .is_err()
        );
        assert_eq!(workbench.last_projection().unwrap(), &initial.projection);
    }
    workbench
        .apply_physical_input(old_handle, input(2, first.clone()))
        .unwrap();
    assert_eq!(
        workbench.last_projection().unwrap(),
        &initial.projection,
        "input cannot publish a projection"
    );
    tick(&mut workbench, 2);
    assert_eq!(
        workbench.last_projection().unwrap(),
        &initial.projection,
        "candidate is hidden until separate Admission"
    );
    let changed = workbench.admit().unwrap();
    let term = decode_canonical_term_bytes(&changed.projection.exact_term_bytes()).unwrap();
    let item = |name| projected_object_field(&term, name);
    assert!(projected_boolean(projected_object_field(
        item(b"first"),
        b"selected"
    )));
    assert!(!projected_boolean(projected_object_field(
        item(b"second"),
        b"selected"
    )));
    assert_eq!(
        projected_number(projected_object_field(item(b"first"), b"progress")),
        0.1
    );
    assert_eq!(
        projected_number(projected_object_field(item(b"second"), b"progress")),
        0.0
    );
    workbench
        .apply_physical_input(old_handle, input(3, second))
        .unwrap();
    tick(&mut workbench, 3);
    let both = workbench.admit().unwrap();
    let term = decode_canonical_term_bytes(&both.projection.exact_term_bytes()).unwrap();
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"second"),
            b"progress"
        )),
        0.1
    );
    workbench.hot_reload(REFERENT_INPUT).unwrap();
    assert_eq!(workbench.generation().handle, old_handle);
    let changed_source = String::from_utf8(REFERENT_INPUT.to_vec())
        .unwrap()
        .replace("first progress 0.0", "first progress 2.0");
    workbench.hot_reload(changed_source.as_bytes()).unwrap();
    assert!(
        workbench
            .apply_physical_input(old_handle, input(4, first.clone()))
            .unwrap_err()
            .to_string()
            .contains("stale")
    );
    // Native callers carry the runtime-session token with the same exact referent.
    let mut native =
        open_fresh_persistent_process_session_v1(&workbench.generation().cwr1).unwrap();
    let token = native.runtime_session();
    let mut other = open_fresh_persistent_process_session_v1(&workbench.generation().cwr1).unwrap();
    assert!(
        other
            .apply_typed_physical_input(
                token,
                &ExecutableInputSourceV1::Referent {
                    channel: b"Pick".to_vec()
                },
                Some(ExecutableValueV1::Referent(first))
            )
            .is_err()
    );
    assert!(
        native
            .apply_typed_physical_input(
                token,
                &ExecutableInputSourceV1::Referent {
                    channel: b"Pick".to_vec()
                },
                Some(ExecutableValueV1::number(1.0).unwrap())
            )
            .is_err()
    );
}

#[test]
fn referent_input_binding_checks_source_domain_and_renames() {
    let source = std::str::from_utf8(REFERENT_INPUT).unwrap();
    assert!(
        ResidentSourceWorkbenchV1::open(
            source
                .replace("Pick as Item", "Pick as ItemClass")
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        ResidentSourceWorkbenchV1::open(
            source
                .replace("to select-item", "to absent-handler")
                .as_bytes()
        )
        .is_err()
    );
    let renamed = source
        .replace("ItemClass", "DocumentKind")
        .replace("Item", "Document")
        .replace("item", "document")
        .replace("Pick", "Focus");
    ResidentSourceWorkbenchV1::open(renamed.as_bytes()).unwrap();
    let no_tick = source.split("on tick").next().unwrap();
    let workbench = ResidentSourceWorkbenchV1::open(no_tick.as_bytes()).unwrap();
    let plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1).unwrap();
    assert_eq!(
        plan.input.unwrap().events.len(),
        1,
        "physical input exists independently of timers"
    );
}

fn referent_input_snapshot(workbench: &mut ResidentSourceWorkbenchV1, revision: u64) -> Term {
    workbench.tick_to_candidate(clause_runtime::WasmSessionTickV1 {
        configuration_revision: revision,
        fixed_tick_milliseconds: 100,
    }).unwrap();
    let admitted = workbench.admit().unwrap();
    decode_canonical_term_bytes(&admitted.projection.exact_term_bytes()).unwrap()
}

fn referent_input_pick(workbench: &mut ResidentSourceWorkbenchV1, snapshot: &Term,
    channel: &[u8], name: &[u8], sequence: u64) {
    let referent = clause_runtime::projected_referent_value_v1(projected_object_field(
        projected_object_field(snapshot, name), b"$referent",
    )).unwrap().unwrap();
    workbench.apply_physical_input(workbench.generation().handle,
        clause_runtime::WasmSessionPhysicalInputV1 {
            input_sequence: sequence,
            source: ExecutableInputSourceV1::Referent { channel: channel.to_vec() },
            value: Some(ExecutableValueV1::Referent(referent)),
        }).unwrap();
}

#[test]
fn repeated_referent_input_clauses_share_one_atomic_event() {
    let source = include_str!("../../../test-vectors/authoring/repeated-referent-input.clause");
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1).unwrap();
    assert_eq!(plan.input.unwrap().events.len(), 1);
    let initial = referent_input_snapshot(&mut workbench, 1);
    referent_input_pick(&mut workbench, &initial, b"Pick", b"first", 1);
    let after = referent_input_snapshot(&mut workbench, 2);
    assert_eq!(projected_number(projected_object_field(
        projected_object_field(&after, b"first"), b"charge")), 1.0);

    // Both clauses must observe the prior selection, not another clause's write.
    let source = source.replace("selected: first", "selected: second")
        .replace("bind referent-input", "second\n  charge: 5.0\n\nbind referent-input")
        .replace("    ?target charge ?charge\n  withdraw", "    ?target charge ?charge\n    ?prior = second\n  withdraw");
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let initial = referent_input_snapshot(&mut workbench, 1);
    referent_input_pick(&mut workbench, &initial, b"Pick", b"first", 1);
    let after = referent_input_snapshot(&mut workbench, 2);
    assert_eq!(projected_number(projected_object_field(
        projected_object_field(&after, b"first"), b"charge")), 1.0);
    let selected = clause_runtime::projected_referent_value_v1(projected_object_field(
        projected_object_field(&after, b"controller"), b"selected")).unwrap().unwrap();
    let first = clause_runtime::projected_referent_value_v1(projected_object_field(
        projected_object_field(&after, b"first"), b"$referent")).unwrap().unwrap();
    assert_eq!(selected, first);
}

#[test]
fn repeated_referent_input_full_workshop_opens_and_dispatches() {
    let source = include_bytes!("../../../test-vectors/greywrought/workshop-expedition.clause");
    let mut workbench = ResidentSourceWorkbenchV1::open(source).unwrap();
    let initial = referent_input_snapshot(&mut workbench, 1);
    referent_input_pick(&mut workbench, &initial, b"PickComponent", b"armor", 1);
    let selected = referent_input_snapshot(&mut workbench, 2);
    referent_input_pick(&mut workbench, &selected, b"FitComponent", b"torso", 2);
    let fitted = referent_input_snapshot(&mut workbench, 3);
    assert_eq!(projected_text(projected_object_field(
        projected_object_field(&fitted, b"workshop"), b"fit-report")),
        "Equipment fitted to the selected body part.");
    let attachment = clause_runtime::projected_referent_value_v1(projected_object_field(
        projected_object_field(&fitted, b"armor"), b"attached-to")).unwrap().unwrap();
    let torso = clause_runtime::projected_referent_value_v1(projected_object_field(
        projected_object_field(&fitted, b"torso"), b"$referent")).unwrap().unwrap();
    assert_eq!(attachment, torso);
    referent_input_pick(&mut workbench, &fitted, b"FitComponent", b"back", 3);
    let refused = referent_input_snapshot(&mut workbench, 4);
    assert_eq!(projected_text(projected_object_field(
        projected_object_field(&refused, b"workshop"), b"fit-report")),
        "That equipment does not fit this body part.");
    assert_eq!(clause_runtime::projected_referent_value_v1(projected_object_field(
        projected_object_field(&refused, b"armor"), b"attached-to")).unwrap().unwrap(), torso);
    referent_input_pick(&mut workbench, &refused, b"PickComponent", b"lance", 4);
    let selected = referent_input_snapshot(&mut workbench, 5);
    referent_input_pick(&mut workbench, &selected, b"FitComponent", b"left-arm", 5);
    let replaced = referent_input_snapshot(&mut workbench, 6);
    assert_eq!(projected_text(projected_object_field(
        projected_object_field(&replaced, b"workshop"), b"fit-report")),
        "Equipment fitted to the selected body part.");
    let left_arm = clause_runtime::projected_referent_value_v1(projected_object_field(
        projected_object_field(&replaced, b"left-arm"), b"$referent")).unwrap().unwrap();
    assert_eq!(clause_runtime::projected_referent_value_v1(projected_object_field(
        projected_object_field(&replaced, b"lance"), b"attached-to")).unwrap().unwrap(), left_arm);
}
const DASH_WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world-dash-jump.clause"
));
const COLLECT_CONTACT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/collect-contact.clause"
));
const SPRING_PAD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/spring-pad.clause"
));
const OBJECTIVE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/objective.clause"
));
const LEDGER: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/ledger/ledger.clause"
));
const NORTH_REPEATED_TURN: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/north/repeated-turn.clause"
));
const TEXT_STATE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/authoring/text-state-transition.clause"
));
const DYNAMIC_TEXT_GOALS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/authoring/dynamic-text-goals.clause"
));
const MULTI_REFERENT_SCALAR_HANDLER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/authoring/multi-referent-scalar-handler.clause"
));
const RUNTIME_SELECTED_POLICY: &str = r#"F64
Root
Policy
policy-a
policy-b

PolicyParameters:
  adjustment: F64
  floor: F64

balance:
  ?root shape Root
  ?balance shape F64
  ?root:
    balance: ?balance

mode balance given root yields balance: one

selected-policy:
  ?root shape Root
  ?policy shape Policy
  ?root:
    selected policy: ?policy

mode selected-policy given root yields policy: one

policy-parameters:
  ?policy shape Policy
  ?policy-parameters shape PolicyParameters
  ?policy:
    policy parameters: ?policy-parameters

mode policy-parameters given policy yields policy-parameters: one

root-1
  member of: Root
policy-a
  member of: Policy
policy-b
  member of: Policy
root-1 balance 10.0
root-1 selected policy policy-a
policy-a policy parameters PolicyParameters { adjustment: 2.0, floor: 0.0 }
policy-b policy parameters PolicyParameters { adjustment: 4.0, floor: 0.0 }

on choose-policy-b ?root
  when
    ?root selected policy ?prior
    ?prior = policy-a
  withdraw
    ?root selected policy ?prior
  include
    ?root selected policy policy-b

on apply-selected-policy ?root
  when
    ?root balance ?prior
    ?root selected policy ?policy
    ?policy policy parameters PolicyParameters { adjustment: ?adjustment, floor: ?floor }
    ?floor < ?prior
  withdraw
    ?root balance ?prior
  include
    ?root balance ?prior - ?adjustment
"#;
const SOURCE_ONLY_AUTOMATIC_EXTENSION: &[u8] = br#"
pulse-count:
  ?objective shape Objective
  ?pulse-count shape F64
  ?objective:
    pulse count: ?pulse-count

mode pulse-count given objective yields pulse-count: one

pulse-radius:
  ?objective shape Objective
  ?pulse-radius shape F64
  ?objective:
    pulse radius: ?pulse-radius

mode pulse-radius given objective yields pulse-radius: one

pulse-echo:
  ?player shape Player
  ?pulse-echo shape F64
  ?player:
    pulse echo: ?pulse-echo

mode pulse-echo given player yields pulse-echo: one

pulse-contact:
  ?objective shape Objective
  ?player shape Player
  ?pulse-contact shape Bool
  ?objective:
    has pulse contact with ?player as: ?pulse-contact

mode pulse-contact given objective player yields pulse-contact: one

game-objective pulse count 0.0
game-objective pulse radius 0.6
player-1 pulse echo 0.0

law pulse-contact-within-radius
  if
    ?player position Vec3 { x: ?player-x, y: ?player-y, z: ?player-z }
    game-objective pulse radius ?radius
    ((?player-x - 0.5) * (?player-x - 0.5)) + ((?player-z - 0.0) * (?player-z - 0.0)) <= ?radius * ?radius
  then
    game-objective has pulse contact with ?player as true

derive pulse-contact-within-radius

on count-pulse ?objective
  when
    ?objective pulse count ?count
    game-objective has pulse contact with player-1 as true
    ?count >= 0.0
  withdraw
    ?objective pulse count ?count
  include
    ?objective pulse count ?count + 1.0

on echo-pulse ?player
  when
    ?player pulse echo ?echo
    game-objective has pulse contact with ?player as true
  withdraw
    ?player pulse echo ?echo
  include
    ?player pulse echo ?echo + 1.0
"#;
const SOURCE_KEYBOARD_BURST_EXTENSION: &[u8] = br#"
bind keyboard KeyQ down to planar-burst

on planar-burst ?player
  when
    ?player velocity Vec3 { x: ?velocity-x, y: ?velocity-y, z: ?velocity-z }
  withdraw
    ?player velocity Vec3 { x: ?velocity-x, y: ?velocity-y, z: ?velocity-z }
  include
    ?player velocity Vec3 { x: ?velocity-x + 3.0, y: ?velocity-y, z: ?velocity-z - 2.0 }
"#;
const SOURCE_SCALAR_CAMERA_EXTENSION: &[u8] = br#"
camera-heading:
  ?player shape Player
  ?camera-heading shape F64
  ?player:
    camera heading: ?camera-heading

mode camera-heading given player yields camera-heading: one

player-1 camera heading 0.0

bind scalar-input CameraHeading to observe-camera-heading

on observe-camera-heading ?player ?heading
  when
    ?player camera heading ?prior
  withdraw
    ?player camera heading ?prior
  include
    ?player camera heading ?heading
"#;
const ACTOR_NEUTRAL_HIT: &str = r#"F64
Actor
Move

vitality:
  ?actor shape Actor
  ?vitality shape F64
  ?actor:
    vitality: ?vitality

mode vitality given actor yields vitality: one

destabilization:
  ?actor shape Actor
  ?destabilization shape F64
  ?actor:
    destabilization: ?destabilization

mode destabilization given actor yields destabilization: one

damage:
  ?move shape Move
  ?damage shape F64
  ?move:
    damage: ?damage

mode damage given move yields damage: one

move-destabilization:
  ?move shape Move
  ?move-destabilization shape F64
  ?move:
    move destabilization: ?move-destabilization

mode move-destabilization given move yields move-destabilization: one

magitek-boar
  member of: Actor
blade-one
  member of: Move

magitek-boar vitality 100.0
magitek-boar destabilization 0.0
blade-one damage 8.0
blade-one move destabilization 25.0

on probe ?defender
  when
    ?defender vitality ?vitality
  withdraw
    ?defender vitality ?vitality
  include
    ?defender vitality ?vitality + 0.0

on admitted-hit ?defender
  when
    ?defender vitality ?vitality
    ?defender destabilization ?destabilization
    blade-one damage ?damage
    blade-one move destabilization ?gain
  withdraw
    ?defender vitality ?vitality
    ?defender destabilization ?destabilization
  include
    ?defender vitality ?vitality - ?damage
    ?defender destabilization ?destabilization + ?gain

on finish-reaction ?defender
  when
    ?defender vitality ?vitality
    ?defender destabilization ?destabilization
    blade-one damage ?damage
  withdraw
    ?defender vitality ?vitality
    ?defender destabilization ?destabilization
  include
    ?defender vitality 0.0
    ?defender destabilization 0.0
"#;
const SCALAR_LAW_BOUND_HIT: &str = r#"F64
Actor
Move
CombatRules

clamped-between:
  (shape: F64):
    ?value ?lower ?upper ?result
  ?value clamped between ?lower and ?upper as ?result

mode clamped-between given value lower upper yields result: maybe

vitality:
  ?actor shape Actor
  ?vitality shape F64
  ?actor:
    vitality: ?vitality

mode vitality given actor yields vitality: one

destabilization:
  ?actor shape Actor
  ?destabilization shape F64
  ?actor:
    destabilization: ?destabilization

mode destabilization given actor yields destabilization: one

mass:
  ?actor shape Actor
  ?mass shape F64
  ?actor:
    mass: ?mass

mode mass given actor yields mass: one

launch-velocity:
  ?actor shape Actor
  ?launch-velocity shape F64
  ?actor:
    launch velocity: ?launch-velocity

mode launch-velocity given actor yields launch-velocity: one

damage:
  ?move shape Move
  ?damage shape F64
  ?move:
    damage: ?damage

mode damage given move yields damage: one

destabilization-gain:
  ?move shape Move
  ?destabilization-gain shape F64
  ?move:
    destabilization gain: ?destabilization-gain

mode destabilization-gain given move yields destabilization-gain: one

base-impulse:
  ?move shape Move
  ?base-impulse shape F64
  ?move:
    base impulse: ?base-impulse

mode base-impulse given move yields base-impulse: one

launch-growth:
  ?move shape Move
  ?launch-growth shape F64
  ?move:
    launch growth: ?launch-growth

mode launch-growth given move yields launch-growth: one

destabilization-threshold:
  ?rules shape CombatRules
  ?destabilization-threshold shape F64
  ?rules:
    destabilization threshold: ?destabilization-threshold

mode destabilization-threshold given rules yields destabilization-threshold: one

law clamp-lower
  if
    ?lower <= ?upper
    ?value < ?lower
  then
    ?value clamped between ?lower and ?upper as ?lower

law clamp-interior
  if
    ?lower <= ?value
    ?value <= ?upper
  then
    ?value clamped between ?lower and ?upper as ?value

law clamp-upper
  if
    ?lower <= ?upper
    ?value > ?upper
  then
    ?value clamped between ?lower and ?upper as ?upper

derive clamp-lower
derive clamp-interior
derive clamp-upper

magitek-boar
  member of: Actor
blade-two
  member of: Move
combat-rules
  member of: CombatRules

magitek-boar vitality 100.0
magitek-boar destabilization 100.0
magitek-boar mass 1.25
magitek-boar launch velocity 0.0
blade-two damage 14.0
blade-two destabilization gain 35.0
blade-two base impulse 5.0
blade-two launch growth 8.0
combat-rules destabilization threshold 100.0

on probe ?defender
  when
    ?defender vitality ?vitality
  withdraw
    ?defender vitality ?vitality
  include
    ?defender vitality ?vitality + 0.0

on blade-two-hit ?defender
  when
    ?defender vitality ?vitality
    ?defender destabilization ?destabilization
    ?defender mass ?mass
    ?defender launch velocity ?launch
    blade-two damage ?damage
    blade-two destabilization gain ?gain
    blade-two base impulse ?impulse
    blade-two launch growth ?growth
    combat-rules destabilization threshold ?threshold
    (?destabilization + ?gain) clamped between 0.0 and ?threshold as ?next-destabilization
  withdraw
    ?defender vitality ?vitality
    ?defender destabilization ?destabilization
    ?defender launch velocity ?launch
  include
    ?defender vitality ?vitality - ?damage
    ?defender destabilization ?next-destabilization
    ?defender launch velocity (?impulse + ?growth * ?next-destabilization / ?threshold) / ?mass
"#;
const OPTIONAL_RELATION_TRANSITION: &str = r#"F64
Actor
Phase
ready
committed

Vec3:
  x: F64
  y: F64
  z: F64

phase:
  ?actor shape Actor
  ?phase shape Phase
  ?actor:
    phase: ?phase

mode phase given actor yields phase: one

position:
  ?actor shape Actor
  ?position shape Vec3
  ?actor:
    position: ?position

mode position given actor yields position: one

anchor:
  ?actor shape Actor
  ?position shape Vec3
  ?actor:
    anchor: ?position

mode anchor given actor yields position: maybe

test-actor
  member of: Actor
test-actor phase ready
test-actor position Vec3 { x: 2.0, y: 3.0, z: 4.0 }

on probe ?actor
  when
    ?actor phase ?phase
  withdraw
    ?actor phase ?phase
  include
    ?actor phase ?phase

on replace-position-from-binding ?actor
  when
    ?actor phase ?phase
    ?actor position ?prior-position
  withdraw
    ?actor phase ?phase
    ?actor position ?prior-position
  include
    ?actor phase committed
    ?actor position Vec3 { x: 5.0, y: 6.0, z: 7.0 }

on materialize-anchor ?actor
  when
    ?actor phase ?phase
    ?actor position Vec3 { x: ?x, y: ?y, z: ?z }
  withdraw
    ?actor phase ?phase
  include
    ?actor phase committed
    ?actor anchor Vec3 { x: ?x, y: ?y, z: ?z }

on clear-anchor ?actor
  when
    ?actor phase ?phase
    ?actor anchor Vec3 { x: ?x, y: ?y, z: ?z }
  withdraw
    ?actor phase ?phase
    ?actor anchor Vec3 { x: ?x, y: ?y, z: ?z }
  include
    ?actor phase ready
"#;
const MANY_RELATION_MEMBERSHIP: &str = r#"Root
Item
idle
alpha
beta

phase:
  ?root shape Root
  ?phase shape Item
  ?root:
    phase: ?phase

mode phase given root yields phase: one

known:
  ?root shape Root
  ?known shape Item
  ?root:
    known: ?known

mode known given root yields known: many

root phase idle

on discover ?root ?item
  when
    ?root phase ?phase
  withdraw
    ?root phase ?phase
  include
    ?root phase ?phase
    ?root known ?item

on probe ?root
  when
    ?root phase ?phase
  withdraw
    ?root phase ?phase
  include
    ?root phase ?phase

on select ?root ?item
  when
    ?root phase ?prior
    ?root known ?item
  withdraw
    ?root phase ?prior
  include
    ?root phase ?item

on forget ?root ?item
  when
    ?root phase ?phase
    ?root known ?item
  withdraw
    ?root phase ?phase
    ?root known ?item
  include
    ?root phase ?phase
"#;

fn coherent_source(objective: &[u8]) -> Vec<u8> {
    let mut source = Vec::with_capacity(
        DASH_WORLD.len() + COLLECT_CONTACT.len() + SPRING_PAD.len() + objective.len() + 3,
    );
    for part in [DASH_WORLD, COLLECT_CONTACT, SPRING_PAD, objective] {
        if !source.is_empty() {
            source.push(b'\n');
        }
        source.extend_from_slice(part);
    }
    source
}

fn coherent_source_with_automatic_extension() -> Vec<u8> {
    let mut source = coherent_source(OBJECTIVE);
    source.extend_from_slice(SOURCE_ONLY_AUTOMATIC_EXTENSION);
    source
}

fn arguments(values: &[f64]) -> Vec<ExecutableValueV1> {
    values
        .iter()
        .copied()
        .map(|value| ExecutableValueV1::number(value).expect("finite test number"))
        .collect()
}

fn decode_hex(source: &str) -> Vec<u8> {
    let digits = source
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    assert_eq!(digits.len() % 2, 0, "fixture hex is complete");
    digits
        .chunks_exact(2)
        .map(|pair| {
            let digit = |value: u8| match value {
                b'0'..=b'9' => value - b'0',
                b'a'..=b'f' => value - b'a' + 10,
                _ => panic!("fixture hex is lowercase"),
            };
            (digit(pair[0]) << 4) | digit(pair[1])
        })
        .collect()
}

fn lowercase_hex_lines(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2 + bytes.len() / 64 + 1);
    for line in bytes.chunks(64) {
        for byte in line {
            text.push(char::from(DIGITS[usize::from(byte >> 4)]));
            text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
        }
        text.push('\n');
    }
    text
}

fn tick_chain(workbench: &ResidentSourceWorkbenchV1, mut prefix: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    prefix.extend(
        workbench
            .fixed_tick_occurrences(0.016)
            .expect("checked source owns the exact tick chain"),
    );
    prefix
}

fn projected_object_field<'a>(term: &'a Term, expected: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        if current.as_atom().is_some() {
            panic!("projected object lacks field {expected:?}");
        }
        let [field, value, rest] = current
            .as_triple()
            .expect("projected object is an entry chain")
            .slots();
        let field = field.as_atom().expect("projected field is an Atom");
        if field.canonical_payload() == expected {
            return value;
        }
        current = rest;
    }
}

fn projected_object_has_field(term: &Term, expected: &[u8]) -> bool {
    let mut current = term;
    while let Some(triple) = current.as_triple() {
        let [field, _, rest] = triple.slots();
        if field
            .as_atom()
            .is_some_and(|field| field.canonical_payload() == expected)
        {
            return true;
        }
        current = rest;
    }
    false
}

fn projected_symbol(term: &Term) -> &[u8] {
    let atom = term.as_atom().expect("projected symbol is an Atom");
    assert_eq!(atom.kind(), b"clause/process-projected-symbol-v1");
    atom.canonical_payload()
}

fn projected_text(term: &Term) -> &str {
    let atom = term.as_atom().expect("projected text is an Atom");
    assert_eq!(atom.kind(), b"clause/process-projected-text-v1");
    std::str::from_utf8(atom.canonical_payload()).expect("projected Text is canonical UTF-8")
}

fn projected_symbol_set(term: &Term) -> Vec<&[u8]> {
    let [header, tree, end] = term
        .as_triple()
        .expect("projected set has a typed wrapper")
        .slots();
    let header = header.as_atom().expect("projected set header is an Atom");
    assert_eq!(header.kind(), b"clause/process-projected-set-v1");
    assert_eq!(header.canonical_payload(), &[2]);
    assert_eq!(
        end.as_atom()
            .expect("projected set wrapper has an end Atom")
            .kind(),
        b"clause/process-projected-set-end-v1"
    );

    fn collect<'a>(term: &'a Term, values: &mut Vec<&'a [u8]>) {
        if let Some(end) = term.as_atom() {
            assert_eq!(end.kind(), b"clause/process-projected-set-end-v1");
            return;
        }
        let [left, value, right] = term
            .as_triple()
            .expect("projected set uses a balanced tree")
            .slots();
        collect(left, values);
        values.push(projected_symbol(value));
        collect(right, values);
    }

    let mut values = Vec::new();
    collect(tree, &mut values);
    values
}

fn projected_text_set(term: &Term) -> Vec<&str> {
    let [header, tree, end] = term
        .as_triple()
        .expect("projected Text set has a typed wrapper")
        .slots();
    let header = header
        .as_atom()
        .expect("projected Text set header is an Atom");
    assert_eq!(header.kind(), b"clause/process-projected-set-v1");
    assert_eq!(header.canonical_payload(), &[6]);
    assert_eq!(
        end.as_atom()
            .expect("projected Text set wrapper has an end Atom")
            .kind(),
        b"clause/process-projected-set-end-v1"
    );

    fn collect<'a>(term: &'a Term, values: &mut Vec<&'a str>) {
        if let Some(end) = term.as_atom() {
            assert_eq!(end.kind(), b"clause/process-projected-set-end-v1");
            return;
        }
        let [left, value, right] = term
            .as_triple()
            .expect("projected Text set uses a balanced tree")
            .slots();
        collect(left, values);
        values.push(projected_text(value));
        collect(right, values);
    }

    let mut values = Vec::new();
    collect(tree, &mut values);
    values
}

fn projected_number(term: &Term) -> f64 {
    let atom = term.as_atom().expect("projected number is an Atom");
    assert_eq!(atom.kind(), b"clause/process-projected-f64-v1");
    f64::from_bits(u64::from_le_bytes(
        atom.canonical_payload()
            .try_into()
            .expect("projected F64 is exact"),
    ))
}

fn projected_boolean(term: &Term) -> bool {
    let atom = term.as_atom().expect("projected Boolean is an Atom");
    assert_eq!(atom.kind(), b"clause/process-projected-bool-v1");
    match atom.canonical_payload() {
        [0] => false,
        [1] => true,
        _ => panic!("projected Boolean payload is canonical"),
    }
}

#[test]
fn one_handler_occurrence_updates_every_independent_matching_referent() {
    let source = MULTI_REFERENT_SCALAR_HANDLER.replacen(
        "cephorium-cache loot state hidden",
        "cephorium-cache loot state acquired",
        1,
    );
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes())
        .expect("the multi-referent scalar source opens");
    let carry = workbench
        .handler_occurrence(b"carry-loot", &[])
        .expect("one semantic handler owns every matching loot referent");
    workbench
        .run_occurrences_to_candidate(&[carry])
        .expect("one occurrence produces one hidden multi-referent candidate");
    let admitted = workbench
        .admit()
        .expect("Admission exposes the atomic multi-referent update");
    let projection = decode_canonical_term_bytes(&admitted.projection.exact_term_bytes())
        .expect("the admitted projection decodes");
    let ashen_key = projected_object_field(&projection, b"ashen-key");
    let cephorium_cache = projected_object_field(&projection, b"cephorium-cache");
    assert_eq!(
        projected_number(projected_object_field(ashen_key, b"carried-distance")),
        2.0,
    );
    assert_eq!(
        projected_number(projected_object_field(cephorium_cache, b"carried-distance",)),
        3.0,
    );
}

fn objective_state(exact_term_bytes: &[u8]) -> Vec<u8> {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let objective = projected_object_field(&term, b"game-objective");
    projected_symbol(projected_object_field(objective, b"objective-state")).to_vec()
}

fn player_launch_state(exact_term_bytes: &[u8]) -> (f64, bool) {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let player = projected_object_field(&term, b"player-1");
    let velocity = projected_object_field(player, b"velocity");
    (
        projected_number(projected_object_field(velocity, b"y")),
        projected_boolean(projected_object_field(player, b"grounded")),
    )
}

fn ledger_balance(exact_term_bytes: &[u8]) -> f64 {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let account = projected_object_field(&term, b"operating-account");
    projected_number(projected_object_field(account, b"balance"))
}

fn pulse_count(exact_term_bytes: &[u8]) -> f64 {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let objective = projected_object_field(&term, b"game-objective");
    projected_number(projected_object_field(objective, b"pulse-count"))
}

fn pulse_echo(exact_term_bytes: &[u8]) -> f64 {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let player = projected_object_field(&term, b"player-1");
    projected_number(projected_object_field(player, b"pulse-echo"))
}

fn player_planar_velocity(exact_term_bytes: &[u8]) -> (f64, f64) {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let player = projected_object_field(&term, b"player-1");
    let velocity = projected_object_field(player, b"velocity");
    (
        projected_number(projected_object_field(velocity, b"x")),
        projected_number(projected_object_field(velocity, b"z")),
    )
}

fn boar_combat_state(exact_term_bytes: &[u8]) -> (f64, f64) {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let boar = projected_object_field(&term, b"magitek-boar");
    (
        projected_number(projected_object_field(boar, b"vitality")),
        projected_number(projected_object_field(boar, b"destabilization")),
    )
}

fn boar_blade_two_state(exact_term_bytes: &[u8]) -> (f64, f64, f64) {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let boar = projected_object_field(&term, b"magitek-boar");
    (
        projected_number(projected_object_field(boar, b"vitality")),
        projected_number(projected_object_field(boar, b"destabilization")),
        projected_number(projected_object_field(boar, b"launch-velocity")),
    )
}

#[test]
fn unchanged_and_rejected_edits_preserve_the_live_world_and_pending_candidate() {
    let mut workbench = ResidentSourceWorkbenchV1::open(LEDGER).unwrap();
    workbench.run_to_candidate().unwrap();
    let first = workbench.admit().unwrap();
    let generation = workbench.generation().clone();
    let candidate = workbench.run_to_candidate().unwrap();
    let deposit = workbench.handler_occurrence(b"deposit", &[]).unwrap();

    assert_eq!(workbench.hot_reload(LEDGER).unwrap(), generation);
    assert_eq!(workbench.pending_candidate(), Some(candidate));
    assert_eq!(workbench.last_projection(), Some(&first.projection));

    let source = std::str::from_utf8(LEDGER).unwrap();
    let without_handlers = source.split("on deposit").next().unwrap();
    let rejected = workbench.hot_reload(without_handlers.as_bytes());
    assert!(
        rejected.is_err(),
        "state without an executable handler cannot open"
    );
    assert_eq!(workbench.generation(), &generation);
    assert_eq!(workbench.pending_candidate(), Some(candidate));
    assert_eq!(workbench.last_projection(), Some(&first.projection));
    assert_eq!(
        workbench.handler_occurrence(b"deposit", &[]).unwrap(),
        deposit
    );
    let second = workbench.admit().unwrap();
    assert_eq!(second.predecessor, first.successor);
    assert_ne!(second.projection, first.projection);
    workbench.run_to_candidate().unwrap();
    workbench.admit().unwrap();
}

#[test]
fn source_edit_hot_reloads_in_one_workbench_without_admission_custody_leak() {
    let mut workbench =
        ResidentSourceWorkbenchV1::open(WORLD).expect("base source opens in one workbench");
    let base_generation = workbench.generation().clone();
    let base_candidate = workbench
        .run_to_candidate()
        .expect("base source produces one hidden candidate");
    assert_eq!(base_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let base_admission = workbench
        .admit()
        .expect("separate base Admission returns the rendered frame");
    assert_eq!(base_admission.state_revision_count, 2);
    assert_ne!(base_admission.predecessor, base_admission.successor);

    let changed = std::str::from_utf8(WORLD)
        .expect("world source is UTF-8")
        .replacen("jump-arena move speed 5.0", "jump-arena move speed 7.0", 1);
    assert_ne!(changed.as_bytes(), WORLD);
    let changed_generation = workbench
        .hot_reload(changed.as_bytes())
        .expect("source-only edit hot reloads in the resident process");
    assert_eq!(
        changed_generation.handle.generation,
        base_generation.handle.generation + 1
    );
    assert_ne!(
        changed_generation.source_package,
        base_generation.source_package
    );
    assert_ne!(changed_generation.cpp1, base_generation.cpp1);
    assert_ne!(changed_generation.cwr1, base_generation.cwr1);
    assert!(
        workbench
            .rejects_stale_handle(base_generation.handle)
            .expect("stale-handle probe reaches the live boundary")
    );

    let changed_candidate = workbench
        .run_to_candidate()
        .expect("changed source reruns without restarting Rust");
    assert_eq!(changed_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let changed_admission = workbench
        .admit()
        .expect("separate changed-source Admission returns its frame");
    assert_eq!(changed_admission.state_revision_count, 2);
    assert_ne!(
        changed_admission.projection.exact_term_bytes(), base_admission.projection.exact_term_bytes(),
        "the source edit changes the admitted rendered frame"
    );

    let changed_again =
        changed.replacen("jump-arena move speed 7.0", "jump-arena move speed 9.0", 1);
    let changed_again_generation = workbench
        .hot_reload(changed_again.as_bytes())
        .expect("a second source-only edit reclaims the prior resident generation");
    assert_eq!(
        changed_again_generation.handle.generation,
        changed_generation.handle.generation + 1
    );
    assert!(
        workbench
            .rejects_stale_handle(changed_generation.handle)
            .expect("the second source edit keeps the prior handle stale")
    );
}

#[test]
fn resident_generation_opens_exact_fresh_branch_sessions() {
    let workbench =
        ResidentSourceWorkbenchV1::open(WORLD).expect("source opens one resident generation");
    let generation = workbench.generation();
    let request = decode_wasm_process_request_v1(&generation.cwr1)
        .expect("resident generation retains one exact CWR1");
    let package = check_process_package(
        decode_process_package(&request.package_bytes).expect("CWR1 package decodes"),
    )
    .expect("CWR1 package remains checked");
    let expected_application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: request.application,
    };
    let expected_revision = ProgramRevisionPreimage {
        semantics: package.constitution().semantics(),
        program: request.authority.program,
        predecessor: None,
        snapshot: package.constitution().snapshot(),
        change: request.authority.change,
    }
    .derived_claim()
    .id;

    let authoritative = open_fresh_persistent_process_session_v1(&generation.cwr1)
        .expect("resident CWR1 opens one fresh authoritative session");
    let branch_session = open_fresh_persistent_process_session_v1(&generation.cwr1)
        .expect("the same resident CWR1 opens one distinct fresh branch session");
    assert_eq!(authoritative.package().unwrap(), package.id());
    assert_eq!(authoritative.application().unwrap(), expected_application);
    assert_eq!(authoritative.program_revision(), expected_revision);
    assert_eq!(branch_session.package().unwrap(), package.id());
    assert_eq!(branch_session.application().unwrap(), expected_application);
    assert_eq!(branch_session.program_revision(), expected_revision);
    assert_eq!(
        branch_session.runtime_session(),
        authoritative.runtime_session()
    );
    assert_ne!(
        branch_session.allocation().root(),
        authoritative.allocation().root(),
        "each fresh session owns a distinct runtime allocation root"
    );

    let disconnect = request
        .occurrences
        .first()
        .expect("resident CWR1 retains one construct-blind occurrence");
    let branch = ForkedProcessBranchV1::fork(&authoritative, branch_session, 1, disconnect)
        .expect("fresh CWR1 sessions enter the exact branch path");
    assert_eq!(branch.pins().package, package.id());
    assert_eq!(branch.pins().application, expected_application);
    assert_eq!(branch.pins().program_revision, expected_revision);
}

#[test]
fn coherent_source_fails_resets_completes_and_hot_reloads_in_one_workbench() {
    let source = coherent_source(OBJECTIVE);
    let mut workbench =
        ResidentSourceWorkbenchV1::open(&source).expect("coherent source opens resident workbench");
    let base_generation = workbench.generation().clone();

    let failure_input = workbench
        .handler_occurrence(b"input", &arguments(&[0.0, -1.0]))
        .expect("checked input handler accepts southward intent");
    let failure = tick_chain(&workbench, vec![failure_input]);
    let failed_candidate = workbench
        .run_occurrences_to_candidate(&failure)
        .expect("hazard produces one hidden candidate");
    assert_eq!(failed_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let failed = workbench
        .admit()
        .expect("separate Admission exposes the failed frame");
    assert_eq!(
        objective_state(&failed.projection.exact_term_bytes()),
        b"failed"
    );

    let north = workbench
        .handler_occurrence(b"input", &arguments(&[0.0, 1.0]))
        .expect("checked input handler accepts northward intent");
    let reset_handler = workbench
        .handler_occurrence(b"reset-objective", &[])
        .expect("checked reset handler accepts its external occurrence");
    let reset = tick_chain(&workbench, vec![north, reset_handler]);
    let reset_candidate = workbench
        .run_occurrences_to_candidate(&reset)
        .expect("reset produces one hidden candidate");
    assert_eq!(reset_candidate.state_revision_count, 2);
    assert_eq!(
        objective_state(&workbench.last_projection().unwrap().exact_term_bytes()),
        b"failed",
        "the admitted renderer remains on failure before reset Admission"
    );
    let reset = workbench
        .admit()
        .expect("separate Admission exposes the reset frame");
    assert_eq!(
        objective_state(&reset.projection.exact_term_bytes()),
        b"playing"
    );

    let east = workbench
        .handler_occurrence(b"input", &arguments(&[1.0, 0.0]))
        .expect("checked input handler accepts eastward intent");
    let completion = tick_chain(&workbench, vec![east]);
    workbench
        .run_occurrences_to_candidate(&completion)
        .expect("movement and collection produce hidden completion");
    assert_eq!(
        objective_state(&workbench.last_projection().unwrap().exact_term_bytes()),
        b"playing",
        "completion is invisible before Admission"
    );
    let completed = workbench
        .admit()
        .expect("separate Admission exposes completion");
    assert_eq!(
        objective_state(&completed.projection.exact_term_bytes()),
        b"completed"
    );

    let spring_input = workbench
        .handler_occurrence(b"input", &arguments(&[1.0, 0.0]))
        .expect("checked input handler advances onto the spring");
    let launch = tick_chain(&workbench, vec![spring_input]);
    workbench
        .run_occurrences_to_candidate(&launch)
        .expect("source-owned spring transition remains hidden");
    assert_eq!(
        player_launch_state(&workbench.last_projection().unwrap().exact_term_bytes()),
        (0.0, true),
        "spring velocity and airborne state remain invisible before Admission"
    );
    let launched = workbench
        .admit()
        .expect("separate Admission exposes the source-owned spring transition");
    assert_eq!(
        player_launch_state(&launched.projection.exact_term_bytes()),
        (12.0, false)
    );

    let changed_objective = std::str::from_utf8(OBJECTIVE)
        .expect("objective source is UTF-8")
        .replacen("?player-x = 0.08", "?player-x = 0.16", 1);
    let changed_source = coherent_source(changed_objective.as_bytes());
    let reload_started = Instant::now();
    let changed_generation = workbench
        .hot_reload(&changed_source)
        .expect("Clause-only objective threshold hot reloads in-process");
    let reload_elapsed = reload_started.elapsed();
    eprintln!(
        "resident coherent source hot reload: {:.3} ms",
        reload_elapsed.as_secs_f64() * 1000.0
    );
    assert_ne!(
        changed_generation.source_package,
        base_generation.source_package
    );
    assert_ne!(changed_generation.cpp1, base_generation.cpp1);
    workbench
        .run_to_candidate()
        .expect("changed source reruns without rebuilding Rust");
    let changed = workbench
        .admit()
        .expect("changed source reaches separate Admission");
    assert_eq!(
        objective_state(&changed.projection.exact_term_bytes()),
        b"playing",
        "the edited completion threshold defers the objective by one tick"
    );
}

#[test]
fn ledger_uses_the_same_checked_resident_binding_path() {
    let mut workbench =
        ResidentSourceWorkbenchV1::open(LEDGER).expect("ledger opens in the generic workbench");
    workbench
        .run_to_candidate()
        .expect("source-owned deposit produces one hidden candidate");
    assert!(workbench.last_projection().is_none());
    let deposited = workbench
        .admit()
        .expect("separate Admission exposes the deposited balance");
    assert_eq!(
        ledger_balance(&deposited.projection.exact_term_bytes()),
        125.0
    );

    let changed = std::str::from_utf8(LEDGER)
        .expect("ledger source is UTF-8")
        .replacen(
            "?account balance ?balance + 25.0",
            "?account balance ?balance + 40.0",
            1,
        );
    workbench
        .hot_reload(changed.as_bytes())
        .expect("Clause-only deposit edit hot reloads without a Rust binding change");
    workbench
        .run_to_candidate()
        .expect("edited deposit produces one hidden candidate");
    let changed = workbench
        .admit()
        .expect("separate Admission exposes the edited balance");
    assert_eq!(ledger_balance(&changed.projection.exact_term_bytes()), 140.0);
}

#[test]
fn source_only_state_and_bounded_automatic_handler_need_no_host_binding_edit() {
    let source = coherent_source_with_automatic_extension();
    let mut workbench = ResidentSourceWorkbenchV1::open(&source)
        .expect("source-only state and automatic handler allocate generically");
    workbench
        .run_to_candidate()
        .expect("the new automatic handler participates in the checked tick chain");
    assert!(workbench.last_projection().is_none());
    let admitted = workbench
        .admit()
        .expect("separate Admission exposes the source-only state");
    assert_eq!(pulse_count(&admitted.projection.exact_term_bytes()), 1.0);
    assert_eq!(pulse_echo(&admitted.projection.exact_term_bytes()), 1.0);
}

#[test]
fn runtime_selected_referent_uses_the_newly_admitted_binding() {
    let mut workbench = ResidentSourceWorkbenchV1::open(RUNTIME_SELECTED_POLICY.as_bytes())
        .expect("the typed runtime-selected policy source opens");
    let choose = workbench
        .handler_occurrence(b"choose-policy-b", &[])
        .expect("the policy selection transition has one occurrence");
    let apply = workbench
        .handler_occurrence(b"apply-selected-policy", &[])
        .expect("all policy alternatives remain one physical handler occurrence");
    workbench
        .run_occurrences_to_candidate(&[choose, apply])
        .expect("selection and its consequence produce one hidden CandidateDelta");
    assert!(workbench.last_projection().is_none());
    let admitted = workbench
        .admit()
        .expect("separate Admission exposes the selected policy consequence");
    let projection = decode_canonical_term_bytes(&admitted.projection.exact_term_bytes())
        .expect("selected policy projection decodes");
    let root = projected_object_field(&projection, b"root-1");
    assert_eq!(
        clause_runtime::projected_referent_value_v1(projected_object_field(
            root,
            b"selected-policy"
        ))
        .unwrap(),
        clause_runtime::projected_referent_value_v1(projected_object_field(
            projected_object_field(&projection, b"policy-b"),
            b"$referent"
        ))
        .unwrap(),
    );
    assert_eq!(
        projected_number(projected_object_field(root, b"balance")),
        6.0,
        "the newly selected policy-b adjustment, not policy-a's initial adjustment, executes"
    );
}

#[test]
fn shaped_state_rejects_a_value_outside_its_declared_field_domain() {
    let malformed = RUNTIME_SELECTED_POLICY.replacen(
        "PolicyParameters { adjustment: 4.0, floor: 0.0 }",
        "PolicyParameters { adjustment: policy-a, floor: 0.0 }",
        1,
    );
    assert!(
        ResidentSourceWorkbenchV1::open(malformed.as_bytes()).is_err(),
        "an F64 field must reject a symbolic value before execution"
    );
}

#[test]
fn source_keyboard_binding_reaches_one_atomic_multi_assignment_candidate() {
    let mut source = WORLD.to_vec();
    source.extend_from_slice(SOURCE_KEYBOARD_BURST_EXTENSION);
    let mut workbench = ResidentSourceWorkbenchV1::open(&source)
        .expect("source keyboard binding and general handler open");
    let burst = workbench
        .handler_occurrence(b"planar-burst", &[])
        .expect("checked burst handler has one occurrence");
    let burst_occurrence =
        decode_executable_occurrence_v1(&burst).expect("burst occurrence decodes");
    let plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1)
        .expect("generated physical plan decodes");
    let input = plan
        .input
        .expect("source keyboard binding creates input plan");
    let key = input
        .events
        .iter()
        .find(|event| {
            event.source
                == ExecutableInputSourceV1::Keyboard {
                    code: b"KeyQ".to_vec(),
                    phase: ExecutableKeyPhaseV1::Down,
                }
        })
        .expect("KeyQ down is present in the physical plan");
    assert_eq!(key.occurrence, burst_occurrence);

    workbench
        .run_occurrences_to_candidate(&[burst])
        .expect("burst produces one hidden candidate");
    assert!(workbench.last_projection().is_none());
    let admitted = workbench
        .admit()
        .expect("separate Admission exposes both burst assignments");
    assert_eq!(
        player_planar_velocity(&admitted.projection.exact_term_bytes()),
        (3.0, -2.0)
    );
}

#[test]
fn source_scalar_input_binding_carries_one_finite_runtime_value() {
    let mut source = WORLD.to_vec();
    source.extend_from_slice(SOURCE_SCALAR_CAMERA_EXTENSION);
    let workbench = ResidentSourceWorkbenchV1::open(&source)
        .expect("source scalar input binding and one-argument handler open");
    let plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1)
        .expect("generated scalar input plan decodes");
    let input = plan.input.expect("scalar input creates one physical plan");
    let binding = input
        .events
        .iter()
        .find(|event| {
            event.source
                == ExecutableInputSourceV1::Scalar {
                    channel: b"CameraHeading".to_vec(),
                }
        })
        .expect("camera heading channel is present in the plan");
    assert_eq!(
        binding.occurrence.arguments,
        vec![ExecutableValueV1::number(0.0).unwrap()]
    );

    let mut session = open_fresh_persistent_process_session_v1(&workbench.generation().cwr1)
        .expect("scalar-input CWR1 opens natively");
    session
        .apply_physical_input(
            &ExecutableInputSourceV1::Scalar {
                channel: b"CameraHeading".to_vec(),
            },
            Some(0.625),
        )
        .expect("the scalar observation enters the checked handler");
    assert!(
        session
            .configuration()
            .expect("scalar input retains local configuration")
            .iter()
            .any(|slot| *slot == ExecutableValueV1::number(0.625).unwrap()),
        "the observed scalar replaces the declared state value"
    );
}

#[test]
fn actor_neutral_hit_updates_two_state_cells_in_one_admitted_candidate() {
    let mut workbench = ResidentSourceWorkbenchV1::open(ACTOR_NEUTRAL_HIT.as_bytes())
        .expect("the actor-neutral combat source opens");
    let probe = workbench
        .handler_occurrence(b"probe", &[])
        .expect("the supported scalar probe has one occurrence");
    workbench
        .run_occurrences_to_candidate(&[probe])
        .expect("the scalar probe produces an initial hidden candidate");
    let initial = workbench
        .admit()
        .expect("separate Admission exposes the initial combat state");
    assert_eq!(
        boar_combat_state(&initial.projection.exact_term_bytes()),
        (100.0, 0.0)
    );

    let admitted_hit = workbench
        .handler_occurrence(b"admitted-hit", &[])
        .expect("one source handler owns vitality and Destabilization");
    workbench
        .run_occurrences_to_candidate(&[admitted_hit])
        .expect("the actor-neutral hit produces one hidden candidate");
    assert_eq!(
        boar_combat_state(&workbench.last_projection().unwrap().exact_term_bytes()),
        (100.0, 0.0),
        "neither combat state cell is visible before Admission"
    );
    let admitted = workbench
        .admit()
        .expect("one Admission exposes both combat state changes");
    assert_eq!(
        boar_combat_state(&admitted.projection.exact_term_bytes()),
        (92.0, 25.0)
    );

    let finish_reaction = workbench
        .handler_occurrence(b"finish-reaction", &[])
        .expect("the three-condition general handler falls through jump classification");
    workbench
        .run_occurrences_to_candidate(&[finish_reaction])
        .expect("the general handler produces one hidden candidate");
    assert_eq!(
        boar_combat_state(&workbench.last_projection().unwrap().exact_term_bytes()),
        (92.0, 25.0),
        "the general-handler result is invisible before Admission"
    );
    let finished = workbench
        .admit()
        .expect("one Admission exposes both general-handler assignments");
    assert_eq!(
        boar_combat_state(&finished.projection.exact_term_bytes()),
        (0.0, 0.0)
    );
}

#[test]
fn many_relation_retains_every_discovered_value_for_membership() {
    let mut workbench = ResidentSourceWorkbenchV1::open(MANY_RELATION_MEMBERSHIP.as_bytes())
        .expect("the neutral many-relation source opens");
    let symbol = |value: &[u8]| {
        ExecutableValueV1::symbol(value).expect("fixture symbols are executable values")
    };

    let probe = workbench
        .handler_occurrence(b"probe", &[])
        .expect("probe accepts the root identity");
    workbench
        .run_occurrences_to_candidate(&[probe])
        .expect("probe produces one hidden candidate");
    let initial = workbench
        .admit()
        .expect("probe reaches the initial admitted world");
    let initial_term = decode_canonical_term_bytes(&initial.projection.exact_term_bytes())
        .expect("the initial projection decodes");
    let initial_root = projected_object_field(&initial_term, b"root");
    assert!(projected_object_has_field(initial_root, b"known"));
    assert!(projected_symbol_set(projected_object_field(initial_root, b"known")).is_empty());

    for value in [b"alpha".as_slice(), b"beta".as_slice()] {
        let discover = workbench
            .handler_occurrence(b"discover", &[symbol(value)])
            .expect("discover accepts one item identity");
        workbench
            .run_occurrences_to_candidate(&[discover])
            .expect("discover produces one hidden candidate");
        workbench
            .admit()
            .expect("discover reaches one admitted successor");
    }
    let before_duplicate = workbench
        .last_projection()
        .expect("the two-value set projects")
        .exact_term_bytes()
        .clone();
    let duplicate = workbench
        .handler_occurrence(b"discover", &[symbol(b"alpha")])
        .expect("duplicate discover accepts one item identity");
    workbench
        .run_occurrences_to_candidate(&[duplicate])
        .expect("duplicate discover produces one hidden candidate");
    let after_duplicate = workbench
        .admit()
        .expect("duplicate discover reaches one admitted successor");
    assert_eq!(
        after_duplicate.projection.exact_term_bytes(), before_duplicate,
        "set insertion is idempotent in the exact projection"
    );

    let select_alpha = workbench
        .handler_occurrence(b"select", &[symbol(b"alpha")])
        .expect("select accepts one item identity");
    workbench
        .run_occurrences_to_candidate(&[select_alpha])
        .expect("membership in the retained set admits selection");
    let selected = workbench
        .admit()
        .expect("selection reaches one admitted successor");
    let term = decode_canonical_term_bytes(&selected.projection.exact_term_bytes())
        .expect("the selected projection decodes");
    let root = projected_object_field(&term, b"root");
    let known = projected_symbol_set(projected_object_field(root, b"known"));
    assert_eq!(known.len(), 2);
    assert!(known.contains(&b"alpha".as_slice()));
    assert!(known.contains(&b"beta".as_slice()));
    assert_eq!(
        projected_symbol(projected_object_field(root, b"phase")),
        b"alpha"
    );

    let mut reverse = ResidentSourceWorkbenchV1::open(MANY_RELATION_MEMBERSHIP.as_bytes())
        .expect("the neutral many-relation source reopens");
    for value in [b"beta".as_slice(), b"alpha".as_slice(), b"alpha".as_slice()] {
        let discover = reverse
            .handler_occurrence(b"discover", &[symbol(value)])
            .expect("reverse discover accepts one item identity");
        reverse
            .run_occurrences_to_candidate(&[discover])
            .expect("reverse discover produces one hidden candidate");
        reverse
            .admit()
            .expect("reverse discover reaches one admitted successor");
    }
    let reverse_term = decode_canonical_term_bytes(
        &reverse
            .last_projection()
            .expect("reverse insertion projects")
            .exact_term_bytes(),
    )
    .expect("the reverse projection decodes");
    let reverse_root = projected_object_field(&reverse_term, b"root");
    assert_eq!(
        projected_symbol_set(projected_object_field(reverse_root, b"known")),
        known,
        "the projected order is independent of insertion order"
    );

    let forget_alpha = workbench
        .handler_occurrence(b"forget", &[symbol(b"alpha")])
        .expect("forget accepts one item identity");
    workbench
        .run_occurrences_to_candidate(&[forget_alpha])
        .expect("a retained member can be withdrawn");
    let forgotten = workbench
        .admit()
        .expect("member withdrawal reaches one admitted successor");
    let forgotten_term = decode_canonical_term_bytes(&forgotten.projection.exact_term_bytes())
        .expect("the removal projection decodes");
    let forgotten_root = projected_object_field(&forgotten_term, b"root");
    assert_eq!(
        projected_symbol_set(projected_object_field(forgotten_root, b"known")),
        vec![b"beta".as_slice()]
    );
    let select_retained = workbench
        .handler_occurrence(b"select", &[symbol(b"beta")])
        .expect("select accepts the retained item identity");
    workbench
        .run_occurrences_to_candidate(&[select_retained])
        .expect("the retained member still satisfies membership");
    workbench
        .admit()
        .expect("selecting the retained member reaches one successor");

    let select_forgotten = workbench
        .handler_occurrence(b"select", &[symbol(b"alpha")])
        .expect("select still accepts the item identity shape");
    workbench
        .run_occurrences_to_candidate(&[select_forgotten])
        .expect("a non-matching occurrence still produces a no-op candidate");
    let unchanged = workbench
        .admit()
        .expect("the no-op candidate remains admissible");
    let unchanged_term = decode_canonical_term_bytes(&unchanged.projection.exact_term_bytes())
        .expect("the no-op projection decodes");
    let unchanged_root = projected_object_field(&unchanged_term, b"root");
    assert_eq!(
        projected_symbol(projected_object_field(unchanged_root, b"phase")),
        b"beta",
        "a withdrawn member no longer satisfies membership"
    );
}

#[test]
fn optional_relation_inserts_and_removes_with_atomic_state_replacement() {
    let mut workbench = ResidentSourceWorkbenchV1::open(OPTIONAL_RELATION_TRANSITION.as_bytes())
        .expect("the neutral optional-relation source opens");
    let probe = workbench
        .handler_occurrence(b"probe", &[])
        .expect("the no-op probe has one occurrence");
    workbench
        .run_occurrences_to_candidate(&[probe])
        .expect("the probe produces one initial Candidate");
    let initial = workbench
        .admit()
        .expect("Admission establishes the prior world");
    let initial_term = decode_canonical_term_bytes(&initial.projection.exact_term_bytes())
        .expect("the initial projection decodes");
    let initial_actor = projected_object_field(&initial_term, b"test-actor");
    assert_eq!(
        projected_symbol(projected_object_field(initial_actor, b"phase")),
        b"ready"
    );
    assert!(!projected_object_has_field(initial_actor, b"anchor"));
    let exact_prior = initial.projection.exact_term_bytes();

    let materialize = workbench
        .handler_occurrence(b"materialize-anchor", &[])
        .expect("the optional insertion handler has one occurrence");
    workbench
        .run_occurrences_to_candidate(&[materialize])
        .expect("replacement and insertion produce one Candidate");
    assert_eq!(
        workbench.last_projection().unwrap().exact_term_bytes(),
        exact_prior,
        "neither replacement nor insertion is visible before Admission"
    );
    let inserted = workbench
        .admit()
        .expect("one Admission exposes replacement and insertion");
    let inserted_term = decode_canonical_term_bytes(&inserted.projection.exact_term_bytes())
        .expect("the inserted projection decodes");
    let inserted_actor = projected_object_field(&inserted_term, b"test-actor");
    assert_eq!(
        projected_symbol(projected_object_field(inserted_actor, b"phase")),
        b"committed"
    );
    let anchor = projected_object_field(inserted_actor, b"anchor");
    assert_eq!(projected_number(projected_object_field(anchor, b"x")), 2.0);
    assert_eq!(projected_number(projected_object_field(anchor, b"y")), 3.0);
    assert_eq!(projected_number(projected_object_field(anchor, b"z")), 4.0);
    let exact_inserted = inserted.projection.exact_term_bytes();

    let clear = workbench
        .handler_occurrence(b"clear-anchor", &[])
        .expect("the optional removal handler has one occurrence");
    workbench
        .run_occurrences_to_candidate(&[clear])
        .expect("replacement and removal produce one Candidate");
    assert_eq!(
        workbench.last_projection().unwrap().exact_term_bytes(),
        exact_inserted,
        "neither replacement nor removal is visible before Admission"
    );
    let removed = workbench
        .admit()
        .expect("one Admission exposes replacement and removal");
    let removed_term = decode_canonical_term_bytes(&removed.projection.exact_term_bytes())
        .expect("the removed projection decodes");
    let removed_actor = projected_object_field(&removed_term, b"test-actor");
    assert_eq!(
        projected_symbol(projected_object_field(removed_actor, b"phase")),
        b"ready"
    );
    assert!(!projected_object_has_field(removed_actor, b"anchor"));
}

#[test]
fn aggregate_binding_replaces_vec3_in_one_atomic_candidate() {
    let mut workbench = ResidentSourceWorkbenchV1::open(OPTIONAL_RELATION_TRANSITION.as_bytes())
        .expect("the neutral aggregate-replacement source opens");
    let probe = workbench
        .handler_occurrence(b"probe", &[])
        .expect("the no-op probe has one occurrence");
    workbench
        .run_occurrences_to_candidate(&[probe])
        .expect("the probe produces one initial Candidate");
    let initial = workbench
        .admit()
        .expect("Admission establishes the prior world");
    let exact_prior = initial.projection.exact_term_bytes();

    let replace = workbench
        .handler_occurrence(b"replace-position-from-binding", &[])
        .expect("the aggregate replacement handler has one occurrence");
    workbench
        .run_occurrences_to_candidate(&[replace])
        .expect("the Vec3 replacement produces one Candidate");
    assert_eq!(
        workbench.last_projection().unwrap().exact_term_bytes(),
        exact_prior,
        "the aggregate replacement remains hidden before Admission"
    );
    let replaced = workbench
        .admit()
        .expect("one Admission exposes all Vec3 components");
    let replaced_term = decode_canonical_term_bytes(&replaced.projection.exact_term_bytes())
        .expect("the replaced projection decodes");
    let actor = projected_object_field(&replaced_term, b"test-actor");
    assert_eq!(
        projected_symbol(projected_object_field(actor, b"phase")),
        b"committed"
    );
    let position = projected_object_field(actor, b"position");
    assert_eq!(
        projected_number(projected_object_field(position, b"x")),
        5.0
    );
    assert_eq!(
        projected_number(projected_object_field(position, b"y")),
        6.0
    );
    assert_eq!(
        projected_number(projected_object_field(position, b"z")),
        7.0
    );
}

#[test]
fn declared_scalar_laws_compose_without_formula_or_binder_spelling_dispatch() {
    let specimen = include_str!("../../../test-vectors/authoring/composed-scalar-laws.clause");
    for (input, expected) in [(-4.0, 6.0), (0.0, 10.0), (12.0, 2.0)] {
        let source = specimen.replace("reading -4.0", &format!("reading {input}"));
        let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        let rectify = workbench.handler_occurrence(b"rectify", &[]).unwrap();
        workbench.run_occurrences_to_candidate(&[rectify]).unwrap();
        let admitted = workbench.admit().unwrap();
        let term = decode_canonical_term_bytes(&admitted.projection.exact_term_bytes()).unwrap();
        let meter = projected_object_field(&term, b"meter-1");
        assert_eq!(
            projected_number(projected_object_field(meter, b"reading")),
            expected
        );
    }
    let renamed = SCALAR_LAW_BOUND_HIT
        .replace("clamp-", "boundary-")
        .replace("clamped between", "restricted by")
        .replace(" and ", " through ")
        .replace(" as ", " giving ")
        .replace("?lower", "?low")
        .replace("?upper", "?high")
        .replace("?value", "?sample")
        .replace("given value lower upper", "given sample low high");
    let mut workbench = ResidentSourceWorkbenchV1::open(renamed.as_bytes()).unwrap();
    let hit = workbench.handler_occurrence(b"blade-two-hit", &[]).unwrap();
    workbench.run_occurrences_to_candidate(&[hit]).unwrap();
    let admitted = workbench.admit().unwrap();
    assert_eq!(
        boar_blade_two_state(&admitted.projection.exact_term_bytes()),
        (86.0, 100.0, 10.4)
    );
}

#[test]
fn guarded_law_results_are_not_evaluated_outside_their_domain() {
    let source = br#"F64
Meter
reciprocal:
  (shape: F64):
    ?input ?output
  reciprocal ?input is ?output

mode reciprocal given input yields output: maybe
law positive-reciprocal
  if
    ?x > 0.0
  then
    reciprocal ?x is (1.0 / ?x)
derive positive-reciprocal
reading:
  ?meter shape Meter
  ?reading shape F64
  ?meter:
    reading: ?reading

mode reading given meter yields reading: one
meter-1 reading 0.0
on invert ?meter
  when
    ?meter reading ?x
    reciprocal ?x is ?next
    ?next > 0.0
  withdraw
    ?meter reading ?x
  include
    ?meter reading ?next
"#;
    let mut workbench = ResidentSourceWorkbenchV1::open(source).unwrap();
    let plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1).unwrap();
    let input = plan.input.as_ref().unwrap();
    assert!(
        input.events.is_empty(),
        "a timer-only world requires no external input binding"
    );
    assert!(!input.tick.entries.is_empty());
    let invert = workbench.handler_occurrence(b"invert", &[]).unwrap();
    workbench.run_occurrences_to_candidate(&[invert]).unwrap();
    let admitted = workbench.admit().unwrap();
    let term = decode_canonical_term_bytes(&admitted.projection.exact_term_bytes()).unwrap();
    assert_eq!(
        projected_number(projected_object_field(
            projected_object_field(&term, b"meter-1"),
            b"reading"
        )),
        0.0
    );
}

#[test]
fn scalar_laws_reject_unbound_results_and_unproved_unique_outputs() {
    let source = include_str!("../../../test-vectors/authoring/composed-scalar-laws.clause");
    for invalid in [
        source.replace("(0.0 - ?x)", "?unbound"),
        source.replace("?x >= 0.0", "?x <= 0.0"),
        source.replace(
            "    ?input ?output",
            "    ?input\n  ?output shape Text",
        ),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
    }
}

#[test]
fn scalar_law_result_feeds_one_atomic_multi_state_candidate() {
    let mut workbench = ResidentSourceWorkbenchV1::open(SCALAR_LAW_BOUND_HIT.as_bytes())
        .expect("the scalar-law combat source opens");
    let probe = workbench
        .handler_occurrence(b"probe", &[])
        .expect("the no-op probe has one occurrence");
    workbench
        .run_occurrences_to_candidate(&[probe])
        .expect("the no-op probe produces an initial hidden candidate");
    let initial = workbench
        .admit()
        .expect("Admission establishes the exact prior projection");
    assert_eq!(
        boar_blade_two_state(&initial.projection.exact_term_bytes()),
        (100.0, 100.0, 0.0)
    );
    let exact_prior_projection = initial.projection.exact_term_bytes();

    let blade_two_hit = workbench
        .handler_occurrence(b"blade-two-hit", &[])
        .expect("the scalar-law handler has one occurrence");
    workbench
        .run_occurrences_to_candidate(&[blade_two_hit])
        .expect("blade two produces one pending Candidate");
    assert_eq!(
        workbench
            .last_projection()
            .expect("the prior admitted projection remains visible")
            .exact_term_bytes(),
        exact_prior_projection,
        "the pending Candidate cannot expose any assignment"
    );

    let admitted = workbench
        .admit()
        .expect("one Admission atomically exposes the dependent results");
    assert_eq!(
        boar_blade_two_state(&admitted.projection.exact_term_bytes()),
        (86.0, 100.0, 10.4)
    );
}

#[test]
fn tracked_browser_carrier_uses_the_generic_source_plan() {
    let source = std::str::from_utf8(WORLD)
        .expect("world source is UTF-8")
        .replace("player-1", "player");
    let workbench = ResidentSourceWorkbenchV1::open(source.as_bytes())
        .expect("browser fixture compiles through generic source bindings");
    let current = decode_wasm_process_request_v1(&workbench.generation().cwr1)
        .expect("generated generic CWR1 decodes");
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../browser/jump-arena-shell/fixtures/wasm-generic-source-v1");
    let fixture_path = fixture_root.join("generic-source-v1.cwr1.hex");
    if std::env::var_os("CLAUSE_UPDATE_BROWSER_GENERIC_SOURCE_CWR1").is_some() {
        std::fs::create_dir_all(&fixture_root).expect("generic browser fixture directory exists");
        std::fs::write(
            &fixture_path,
            lowercase_hex_lines(&workbench.generation().cwr1),
        )
        .expect("generic browser fixture updates");
    }
    let tracked = std::fs::read_to_string(&fixture_path)
        .expect("tracked generic browser CWR1 fixture exists");
    let tracked = decode_wasm_process_request_v1(&decode_hex(&tracked))
        .expect("tracked generic browser CWR1 decodes");
    assert_eq!(tracked.package_bytes, current.package_bytes);
    assert_eq!(tracked.application, current.application);
    assert_eq!(tracked.physical_plan_bytes, current.physical_plan_bytes);
    assert_eq!(tracked.authority, current.authority);
    assert_eq!(tracked.occurrences, current.occurrences);
    assert_eq!(tracked.render_slots, current.render_slots);
}

#[test]
fn resident_source_opens_the_north_repeated_turn_machine() {
    let mut workbench = ResidentSourceWorkbenchV1::open(NORTH_REPEATED_TURN)
        .expect("North's repeated-turn machine opens in the generic workbench");
    let submit = workbench
        .handler_occurrence(b"submit", &[])
        .expect("the host can select North's checked submit handler");
    workbench
        .run_occurrences_to_candidate(&[submit])
        .expect("submit reaches one hidden candidate");
    workbench
        .admit()
        .expect("submit reaches one admitted successor");
}

#[test]
fn text_handler_state_projection_and_persistent_redirect_are_one_value_path() {
    let mut workbench = ResidentSourceWorkbenchV1::open(TEXT_STATE)
        .expect("the text-state source opens in the resident workbench");
    let restored_plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1)
        .expect("the Text physical plan restores from CPP1");
    assert!(
        restored_plan
            .program
            .initial_configuration
            .iter()
            .any(|value| value.kind() == ExecutableValueKindV1::Text)
    );
    assert!(
        restored_plan
            .program
            .initial_configuration
            .iter()
            .any(|value| value.kind() == ExecutableValueKindV1::TextSet)
    );
    assert_eq!(
        encode_executable_physical_plan_v1(&restored_plan)
            .expect("the restored Text physical plan re-encodes"),
        workbench.generation().cpp1
    );
    let create = workbench
        .handler_occurrence(
            b"create-goal",
            &[
                ExecutableValueV1::text("Launch North").expect("title is bounded Text"),
                ExecutableValueV1::text("North handles goals elegantly 🚀")
                    .expect("Unicode objective is bounded Text"),
            ],
        )
        .expect("the checked handler accepts Text arguments");
    workbench
        .run_occurrences_to_candidate(&[create])
        .expect("Text creation reaches one hidden candidate");
    let created = workbench
        .admit()
        .expect("Text creation reaches one admitted successor");
    let created = decode_canonical_term_bytes(&created.projection.exact_term_bytes())
        .expect("the Text projection decodes");
    let north = projected_object_field(&created, b"north-main");
    assert_eq!(
        projected_symbol(projected_object_field(north, b"goal-state")),
        b"active"
    );
    assert_eq!(
        projected_text(projected_object_field(north, b"goal-title")),
        "Launch North"
    );
    assert_eq!(
        projected_text(projected_object_field(north, b"goal-objective")),
        "North handles goals elegantly 🚀"
    );
    assert_eq!(
        projected_text(projected_object_field(north, b"banner")),
        "North says:\n\"ready\" 🚀"
    );
    assert_eq!(
        projected_text_set(projected_object_field(north, b"goal-tags")),
        vec!["Launch North"]
    );

    let tag = workbench
        .handler_occurrence(
            b"tag-goal",
            &[ExecutableValueV1::text("🚀 durable").expect("tag is bounded Text")],
        )
        .expect("the checked handler accepts a Text set member");
    workbench
        .run_occurrences_to_candidate(&[tag])
        .expect("Text set insertion reaches one hidden candidate");
    workbench
        .admit()
        .expect("Text set insertion reaches one admitted successor");

    let redirect = workbench
        .handler_occurrence(
            b"redirect-goal",
            &[ExecutableValueV1::text("subsumes its Rust semantics")
                .expect("revised objective is bounded Text")],
        )
        .expect("the redirect handler accepts Text");
    workbench
        .run_occurrences_to_candidate(&[redirect])
        .expect("Text replacement reaches one hidden candidate");
    let redirected = workbench
        .admit()
        .expect("Text replacement reaches one admitted successor");
    let redirected = decode_canonical_term_bytes(&redirected.projection.exact_term_bytes())
        .expect("the redirected projection decodes");
    let north = projected_object_field(&redirected, b"north-main");
    assert_eq!(
        projected_text(projected_object_field(north, b"goal-title")),
        "Launch North"
    );
    assert_eq!(
        projected_text(projected_object_field(north, b"goal-objective")),
        "North subsumes its Rust semantics"
    );
    assert_eq!(
        projected_text_set(projected_object_field(north, b"goal-tags")),
        vec!["Launch North", "🚀 durable"]
    );
}

#[test]
fn runtime_created_referent_keys_goal_rows_and_retains_redirect_history() {
    let mut workbench = ResidentSourceWorkbenchV1::open(DYNAMIC_TEXT_GOALS)
        .expect("the dynamic relational goal source opens in the resident workbench");
    let create = workbench
        .handler_occurrence(
            b"create-goal",
            &[
                ExecutableValueV1::text("Ship North").expect("title is bounded Text"),
                ExecutableValueV1::text("Clause owns the goal").expect("objective is bounded Text"),
            ],
        )
        .expect("the create handler takes only its two Text inputs");
    workbench
        .run_occurrences_to_candidate(&[create])
        .expect("goal creation reaches one hidden candidate");
    let created = workbench
        .admit()
        .expect("goal creation reaches one admitted successor");
    let created = decode_canonical_term_bytes(&created.projection.exact_term_bytes())
        .expect("the keyed goal projection decodes");
    let relations = projected_object_field(&created, b"relations");
    let known = projected_relation_table_v1(projected_object_field(relations, b"known-goal"))
        .expect("the known-goal table projection is canonical")
        .expect("known-goal projects one relation table");
    let goal = known
        .rows()
        .values()
        .flat_map(|values| values.iter())
        .find_map(|value| value.as_referent().cloned())
        .expect("Clause inserted one runtime-created Goal referent");
    let title = projected_relation_table_v1(projected_object_field(relations, b"goal-title"))
        .expect("the title table projection is canonical")
        .expect("goal-title projects one relation table");
    assert_eq!(
        title
            .rows()
            .get(&goal)
            .and_then(|values| values.first())
            .and_then(ExecutableValueV1::as_text),
        Some("Ship North")
    );
    let objective =
        projected_relation_table_v1(projected_object_field(relations, b"goal-objective"))
            .expect("the objective table projection is canonical")
            .expect("goal-objective projects one relation table");
    assert_eq!(
        objective
            .rows()
            .get(&goal)
            .and_then(|values| values.first())
            .and_then(ExecutableValueV1::as_text),
        Some("Clause owns the goal")
    );

    let redirect = workbench
        .handler_occurrence(
            b"redirect-goal",
            &[
                ExecutableValueV1::Referent(goal.clone()),
                ExecutableValueV1::text("Clause owns the history")
                    .expect("redirected objective is bounded Text"),
            ],
        )
        .expect("the redirect handler accepts the created Goal identity");
    workbench
        .run_occurrences_to_candidate(&[redirect])
        .expect("goal redirect reaches one hidden candidate");
    let redirected = workbench
        .admit()
        .expect("goal redirect reaches one admitted successor");
    let redirected = decode_canonical_term_bytes(&redirected.projection.exact_term_bytes())
        .expect("the redirected keyed goal projection decodes");
    let relations = projected_object_field(&redirected, b"relations");
    let objective =
        projected_relation_table_v1(projected_object_field(relations, b"goal-objective"))
            .expect("the redirected objective table is canonical")
            .expect("goal-objective remains a relation table");
    assert_eq!(
        objective
            .rows()
            .get(&goal)
            .and_then(|values| values.first())
            .and_then(ExecutableValueV1::as_text),
        Some("Clause owns the history")
    );
    let history =
        projected_relation_table_v1(projected_object_field(relations, b"prior-goal-objective"))
            .expect("the prior-objective table is canonical")
            .expect("prior-goal-objective projects one relation table");
    assert!(history.rows().get(&goal).is_some_and(|values| {
        values
            .iter()
            .any(|value| value.as_text() == Some("Clause owns the goal"))
    }));
}

#[test]
fn created_referent_physical_input_preserves_full_identity_bytes() {
    let source = format!(
        "{}\nbind referent-input Finish as Goal to finish-goal\n\non finish-goal ?north ?goal\n  when\n    ?north known goal ?goal\n    ?goal status ?status\n  withdraw\n    ?goal status ?status\n  include\n    ?goal status ready\n",
        std::str::from_utf8(DYNAMIC_TEXT_GOALS).unwrap()
    );
    let workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let create = workbench
        .handler_occurrence(
            b"create-goal",
            &[
                ExecutableValueV1::text("one").unwrap(),
                ExecutableValueV1::text("objective").unwrap(),
            ],
        )
        .unwrap();
    let mut session =
        open_fresh_persistent_process_session_v1(&workbench.generation().cwr1).unwrap();
    session.apply_opaque_input(&create).unwrap();
    let goal = session
        .configuration()
        .unwrap()
        .iter()
        .find_map(|slot| {
            let clause_runtime::ExecutableSlotV1::Present(ExecutableValueV1::RelationTable(table)) =
                slot
            else {
                return None;
            };
            table
                .rows()
                .keys()
                .find(|referent| {
                    matches!(
                        referent.identity(),
                        clause_runtime::ExecutableReferentIdentityV1::Created(_)
                    )
                })
                .cloned()
        })
        .unwrap();
    let command = clause_runtime::WasmSessionCommandV1 {
        handle: workbench.generation().handle,
        expected_sequence: 0,
        operation: clause_runtime::WasmSessionOperationV1::PhysicalInput(
            clause_runtime::WasmSessionPhysicalInputV1 {
                input_sequence: 1,
                source: ExecutableInputSourceV1::Referent {
                    channel: b"Finish".to_vec(),
                },
                value: Some(ExecutableValueV1::Referent(goal.clone())),
            },
        ),
    };
    let bytes = clause_runtime::encode_wasm_session_command_v1(&command).unwrap();
    assert_eq!(
        clause_runtime::decode_wasm_session_command_v1(&bytes).unwrap(),
        command
    );
    assert!(clause_runtime::decode_wasm_session_command_v1(&bytes[..bytes.len() - 1]).is_err());
    let clause_runtime::WasmSessionOperationV1::PhysicalInput(input) = command.operation else {
        unreachable!()
    };
    session
        .apply_typed_physical_input(session.runtime_session(), &input.source, input.value)
        .unwrap();
    let unknown = clause_runtime::ExecutableReferentV1::created(goal.domain(), [0xff; 32]);
    assert!(
        session
            .apply_typed_physical_input(
                session.runtime_session(),
                &input.source,
                Some(ExecutableValueV1::Referent(unknown))
            )
            .is_err()
    );
}

#[test]
fn source_keyboard_arguments_preserve_movement_across_reload() {
    use clause_runtime::{WasmSessionPhysicalInputV1, WasmSessionTickV1};
    let mut workbench = ResidentSourceWorkbenchV1::open(WORLD).unwrap();
    for (iteration, speed) in [5.0, 7.0].into_iter().enumerate() {
        if iteration == 1 {
            let changed = std::str::from_utf8(WORLD).unwrap()
                .replacen("jump-arena move speed 5.0", "jump-arena move speed 7.0", 1);
            workbench.hot_reload(changed.as_bytes()).unwrap();
        }
        let plan = decode_executable_physical_plan_v1(&workbench.generation().cpp1).unwrap();
        assert_eq!(plan.input.as_ref().unwrap().tick.entries.len(), 1,
            "the root event evaluates all three branches against one pre-state");
        let key = plan.input.unwrap().events.into_iter().find(|event| matches!(&event.source,
            ExecutableInputSourceV1::Keyboard { code, phase: ExecutableKeyPhaseV1::Down } if code == b"KeyD")).unwrap();
        assert_eq!(key.occurrence.arguments, [ExecutableValueV1::number(1.0).unwrap(), ExecutableValueV1::number(0.0).unwrap()]);
        workbench.apply_physical_input(workbench.generation().handle, WasmSessionPhysicalInputV1 {
            input_sequence: 1, source: key.source, value: None,
        }).unwrap();
        workbench.tick_to_candidate(WasmSessionTickV1 {
            configuration_revision: 1, fixed_tick_milliseconds: 16,
        }).unwrap();
        let admission = workbench.admit().unwrap();
        assert_eq!(player_planar_velocity(&admission.projection.exact_term_bytes()), (speed, 0.0));
    }
    for invalid in ["with 1.0", "with 1.0 0.0 2.0", "with NaN 0.0", "with inf 0.0"] {
        let rejected = std::str::from_utf8(WORLD).unwrap()
            .replacen("to input with 1.0 0.0", &format!("to input {invalid}"), 1);
        assert!(ResidentSourceWorkbenchV1::open(rejected.as_bytes()).is_err());
    }
}
