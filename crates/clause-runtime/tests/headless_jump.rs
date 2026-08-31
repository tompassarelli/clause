use clause_package::*;
use clause_runtime::*;

const BROWSER_ADMISSION_COUNT: usize = 8;
const BROWSER_RECORDED_ALLOCATION_ROOT_TAG: u8 = 210;
const BROWSER_COLLECT_ALLOCATION_ROOT_TAG: u8 = 234;
const BROWSER_COLLECT_CHANGED_ALLOCATION_ROOT_TAG: u8 = 235;
const BROWSER_SYMBOLIC_COLLECT_ALLOCATION_ROOT_TAG: u8 = 240;
const BROWSER_SYMBOLIC_COLLECT_CHANGED_ALLOCATION_ROOT_TAG: u8 = 241;
const BROWSER_GAMEPLAY_ALLOCATION_ROOT_TAG: u8 = 242;
const BROWSER_GAMEPLAY_CHANGED_ALLOCATION_ROOT_TAG: u8 = 243;
const BROWSER_GAMEPLAY_DASH_ALLOCATION_ROOT_TAG: u8 = 245;
const SOURCE_ALLOCATION_ROOT_TAG: u8 = 211;
const WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world.clause"
));
const DASH_WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world-dash-jump.clause"
));
const COLLECT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/collect.clause"
));
const COLLECT_STATE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/collect-state.clause"
));
const COLLECT_CONTACT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/collect-contact.clause"
));
const SPRING_PAD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/spring-pad.clause"
));
const LEDGER: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/ledger/ledger.clause"
));

fn gameplay_source() -> Vec<u8> {
    gameplay_source_with_world(WORLD)
}

fn gameplay_source_with_world(world: &[u8]) -> Vec<u8> {
    let mut source = Vec::with_capacity(world.len() + COLLECT_CONTACT.len() + SPRING_PAD.len() + 2);
    source.extend_from_slice(world);
    source.push(b'\n');
    source.extend_from_slice(COLLECT_CONTACT);
    source.push(b'\n');
    source.extend_from_slice(SPRING_PAD);
    source
}

fn dash_gameplay_source() -> Vec<u8> {
    gameplay_source_with_world(DASH_WORLD)
}

macro_rules! id {
    ($kind:ident, $tag:expr) => {
        $kind::from_bytes(raw_id($tag))
    };
}
fn decode_hex(source: &str) -> Vec<u8> {
    let digits = source
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    digits
        .chunks_exact(2)
        .map(|pair| (nibble(pair[0]) << 4) | nibble(pair[1]))
        .collect()
}

fn nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => panic!("fixture contains a non-hex byte"),
    }
}

fn number(value: f64) -> ExecutableValueV1 {
    ExecutableValueV1::number(value).expect("scenario numbers are finite")
}

fn projection_atom(scope: TermScope, kind: &[u8], payload: &[u8]) -> Term {
    Term::atom(
        scope,
        kind.to_vec(),
        payload.to_vec(),
        EqualityContract::ExactOctetsV1,
    )
    .expect("projection literal Atom is valid")
}

fn projection_role(scope: TermScope, role: LocalRoleRefV2, kind: ExecutableValueKindV1) -> Term {
    executable_projection_role_term_v1(scope, role, kind)
        .expect("projection role placeholder is valid")
}

fn projection_object(scope: TermScope, fields: Vec<(&'static [u8], Term)>) -> Term {
    fields.into_iter().rev().fold(
        projection_atom(scope, b"clause/js-object-end-v1", &[]),
        |rest, (field, value)| {
            Term::triple([
                projection_atom(scope, b"clause/js-field-v1", field),
                value,
                rest,
            ])
            .expect("projection object entry has one scope")
        },
    )
}

fn projection_array(scope: TermScope, values: Vec<Term>) -> Term {
    values.into_iter().rev().fold(
        projection_atom(scope, b"clause/js-array-end-v1", &[]),
        |rest, value| {
            Term::triple([
                projection_atom(scope, b"clause/js-item-v1", &[]),
                value,
                rest,
            ])
            .expect("projection array entry has one scope")
        },
    )
}

fn projected_vec3(scope: TermScope, roles: [LocalRoleRefV2; 3]) -> Term {
    projection_object(
        scope,
        vec![
            (
                b"x",
                projection_role(scope, roles[0], ExecutableValueKindV1::Number),
            ),
            (
                b"y",
                projection_role(scope, roles[1], ExecutableValueKindV1::Number),
            ),
            (
                b"z",
                projection_role(scope, roles[2], ExecutableValueKindV1::Number),
            ),
        ],
    )
}

fn source_handlers(
    source: &[u8],
    scope: TermScope,
) -> (
    CanonicalInputHandlerV1,
    CanonicalJumpHandlerV1,
    CanonicalTickProgramV1,
    Vec<CanonicalScalarHandlerV1>,
) {
    let cst = read_canonical_source_v1(source).expect("canonical arena source reads");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(SOURCE_ALLOCATION_ROOT_TAG)),
    )
    .expect("canonical arena source receives rooted allocations");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: scope.universe,
            semantics: scope.semantics,
        },
        &plan,
    )
    .expect("canonical arena source reaches the checked package boundary");
    let input_handler = compiled
        .input_handler
        .expect("the bounded source profile owns one on-input handler");
    let jump_handler = compiled
        .jump_handler
        .expect("the bounded source profile owns one on-jump handler");
    let tick_program = compiled
        .tick_program
        .expect("the bounded source profile owns the three on-tick branches");
    (
        input_handler,
        jump_handler,
        tick_program,
        compiled.scalar_handlers,
    )
}

fn source_scalar_handler(source: &[u8], scope: TermScope) -> CanonicalScalarHandlerV1 {
    let cst = read_canonical_source_v1(source).expect("canonical scalar source reads");
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes(raw_id(SOURCE_ALLOCATION_ROOT_TAG)),
    )
    .expect("canonical scalar source receives rooted allocations");
    elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: scope.universe,
            semantics: scope.semantics,
        },
        &plan,
    )
    .expect("canonical scalar source reaches the checked package boundary")
    .scalar_handlers
    .into_iter()
    .next()
    .expect("the bounded source profile owns one scalar transition")
}

fn scalar_program(
    scope: TermScope,
    source: &CanonicalScalarHandlerV1,
    object_field: &'static [u8],
    value_field: &'static [u8],
) -> ExecutableProgramV1 {
    let role = LocalRoleRefV2 {
        schema: RelationSchemaLocalId::new(2),
        role: RoleLocalId::new(1),
    };
    let value_kind = match &source.initial_value {
        CanonicalScalarValueV1::Number(_) => ExecutableValueKindV1::Number,
        CanonicalScalarValueV1::Boolean(_) => ExecutableValueKindV1::Boolean,
        CanonicalScalarValueV1::Symbol(_) => ExecutableValueKindV1::Symbol,
    };
    let mut program = ExecutableProgramV1 {
        initial_configuration: vec![number(0.0)],
        rules: vec![],
        projection: Some(ExecutableProjectionV1 {
            bindings: vec![ExecutableProjectionBindingV1 {
                role,
                slot: 0,
                value_kind,
            }],
            template: projection_object(
                scope,
                vec![(
                    object_field,
                    projection_object(
                        scope,
                        vec![(value_field, projection_role(scope, role, value_kind))],
                    ),
                )],
            ),
        }),
    };
    lower_canonical_scalar_handler_v1(
        &mut program,
        source,
        ExecutableCanonicalScalarBindingV1 {
            entry: 0,
            state_slot: 0,
            parameters: vec![],
        },
    )
    .expect("source-owned scalar transition lowers to one physical state slot");
    program
}

fn headless_program(
    scope: TermScope,
    input: &CanonicalInputHandlerV1,
    jump: &CanonicalJumpHandlerV1,
    tick: &CanonicalTickProgramV1,
    scalar_handlers: &[CanonicalScalarHandlerV1],
) -> ExecutableProgramV1 {
    let collectible = scalar_handlers.first();
    let role = |id| LocalRoleRefV2 {
        schema: RelationSchemaLocalId::new(2),
        role: RoleLocalId::new(id),
    };
    let player = projection_object(
        scope,
        vec![
            (
                b"position",
                projected_vec3(scope, [role(1), role(2), role(3)]),
            ),
            (
                b"velocity",
                projected_vec3(scope, [role(4), role(5), role(6)]),
            ),
            (
                b"yaw",
                projection_role(scope, role(7), ExecutableValueKindV1::Number),
            ),
            (
                b"grounded",
                projection_role(scope, role(8), ExecutableValueKindV1::Boolean),
            ),
        ],
    );
    let platform = projection_object(
        scope,
        vec![
            (
                b"position",
                projected_vec3(scope, [role(9), role(10), role(11)]),
            ),
            (
                b"size",
                projected_vec3(scope, [role(12), role(13), role(14)]),
            ),
        ],
    );
    let mut world_fields = vec![(
        b"platforms".as_slice(),
        projection_array(scope, vec![platform]),
    )];
    if collectible.is_some() {
        let collectible = projection_object(
            scope,
            vec![
                (
                    b"position",
                    projected_vec3(scope, [role(21), role(22), role(23)]),
                ),
                (
                    b"state",
                    projection_role(scope, role(24), ExecutableValueKindV1::Symbol),
                ),
            ],
        );
        world_fields.push((
            b"collectibles".as_slice(),
            projection_array(scope, vec![collectible]),
        ));
    }
    let template = projection_object(
        scope,
        vec![
            (b"player", player),
            (b"world", projection_object(scope, world_fields)),
        ],
    );

    let mut initial_configuration = vec![
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
        ExecutableValueV1::Boolean(false),
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
        number(-0.25),
        number(0.0),
        number(12.0),
        number(0.5),
        number(12.0),
        number(0.0),
        number(0.0),
        number(0.0),
        number(0.0),
    ];
    if let Some(collectible) = collectible {
        initial_configuration.extend([
            number(0.08),
            number(0.9),
            number(0.0),
            match &collectible.initial_value {
                CanonicalScalarValueV1::Number(bits) => ExecutableValueV1::Number(*bits),
                CanonicalScalarValueV1::Boolean(value) => ExecutableValueV1::Boolean(*value),
                CanonicalScalarValueV1::Symbol(value) => ExecutableValueV1::symbol(value)
                    .expect("source-owned collectible state is bounded"),
            },
        ]);
    }
    let mut projection_bindings = [
        (1, 0, ExecutableValueKindV1::Number),
        (2, 1, ExecutableValueKindV1::Number),
        (3, 12, ExecutableValueKindV1::Number),
        (4, 2, ExecutableValueKindV1::Number),
        (5, 3, ExecutableValueKindV1::Number),
        (6, 13, ExecutableValueKindV1::Number),
        (7, 14, ExecutableValueKindV1::Number),
        (8, 5, ExecutableValueKindV1::Boolean),
        (9, 15, ExecutableValueKindV1::Number),
        (10, 16, ExecutableValueKindV1::Number),
        (11, 17, ExecutableValueKindV1::Number),
        (12, 18, ExecutableValueKindV1::Number),
        (13, 19, ExecutableValueKindV1::Number),
        (14, 20, ExecutableValueKindV1::Number),
    ]
    .into_iter()
    .map(
        |(role_id, slot, value_kind)| ExecutableProjectionBindingV1 {
            role: role(role_id),
            slot,
            value_kind,
        },
    )
    .collect::<Vec<_>>();
    if collectible.is_some() {
        projection_bindings.extend(
            [
                (21, 25, ExecutableValueKindV1::Number),
                (22, 26, ExecutableValueKindV1::Number),
                (23, 27, ExecutableValueKindV1::Number),
                (24, 28, ExecutableValueKindV1::Symbol),
            ]
            .into_iter()
            .map(
                |(role_id, slot, value_kind)| ExecutableProjectionBindingV1 {
                    role: role(role_id),
                    slot,
                    value_kind,
                },
            ),
        );
    }
    let mut program = ExecutableProgramV1 {
        // Gameplay coordinates are placeholders populated only by source-owned
        // input, jump, and tick lowering. The remaining values are passive
        // renderer-only platform coordinates absent from canonical game source.
        initial_configuration,
        rules: vec![],
        projection: Some(ExecutableProjectionV1 {
            bindings: projection_bindings,
            template,
        }),
    };
    lower_canonical_input_handler_v1(
        &mut program,
        input,
        ExecutableCanonicalInputBindingV1 {
            entry: 0,
            x_slot: 4,
            z_slot: 21,
        },
    )
    .expect("source-owned input handler lowers to its physical slots");
    lower_canonical_jump_handler_v1(
        &mut program,
        jump,
        ExecutableCanonicalJumpBindingV1 {
            entry: 1,
            velocity_slots: [2, 3, 13],
            grounded_slot: 5,
            jump_speed_slot: 7,
        },
    )
    .expect("source-owned jump handler lowers to its physical slots");
    lower_canonical_tick_program_v1(
        &mut program,
        tick,
        ExecutableCanonicalTickBindingV1 {
            entry: 2,
            delta_time_argument: 0,
            position_slots: [0, 1, 12],
            velocity_slots: [2, 3, 13],
            intent_slots: [4, 22, 21],
            grounded_slot: 5,
            gravity_slot: 6,
            move_speed_slot: 8,
            floor_height_slot: 9,
            minimum_x_slot: 10,
            maximum_x_slot: 11,
            minimum_z_slot: 23,
            maximum_z_slot: 24,
        },
    )
    .expect("source-owned tick program lowers to physical slots");
    let scalar_state_slots = [28, 3, 5];
    for (index, handler) in scalar_handlers.iter().enumerate() {
        let state_slot = *scalar_state_slots
            .get(index)
            .expect("bounded gameplay profile has collect plus two spring cells");
        assert_eq!(handler.parameters.len(), 2);
        lower_canonical_scalar_handler_v1(
            &mut program,
            handler,
            ExecutableCanonicalScalarBindingV1 {
                entry: 3 + u16::try_from(index).expect("bounded handler index fits u16"),
                state_slot,
                parameters: handler
                    .parameters
                    .iter()
                    .zip([0, 12])
                    .map(
                        |(parameter, slot)| ExecutableCanonicalScalarParameterBindingV1 {
                            slot,
                            parameter: parameter.clone(),
                        },
                    )
                    .collect(),
            },
        )
        .expect("source-owned automatic handler lowers beside arena state");
    }
    program
}

fn checked_program_package_with_scopes(
    checker_count: usize,
    state_admission_scopes: Vec<StateAdmissionScope>,
) -> CheckedProcessPackage {
    checked_program_package_with_scopes_and_roles(checker_count, state_admission_scopes, 20)
}

fn checked_program_package_with_scopes_and_roles(
    checker_count: usize,
    state_admission_scopes: Vec<StateAdmissionScope>,
    projection_role_count: u32,
) -> CheckedProcessPackage {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-vectors/process-v2/positive/process-v2-core.hex"
    ));
    let decoded = decode_process_package(&decode_hex(source)).expect("base package decodes");
    let mut candidate = decoded.candidate().clone();
    candidate.records.clear();

    let mut projection_schema = candidate.snapshot.constitution.schemas[0].clone();
    projection_schema.id = RelationSchemaLocalId::new(2);
    let base_role = projection_schema.roles[0].clone();
    projection_schema.roles = (1..=projection_role_count)
        .map(|role_id| {
            let mut role = base_role.clone();
            role.id = RoleLocalId::new(role_id);
            role
        })
        .collect();
    projection_schema.roles.sort_by_key(|role| role.id);
    candidate
        .snapshot
        .constitution
        .schemas
        .push(projection_schema);
    candidate
        .snapshot
        .constitution
        .schemas
        .sort_by_key(|schema| schema.id);

    match checker_count {
        0 => candidate.snapshot.constitution.operators[0].modes[0]
            .contract
            .formation_checks
            .clear(),
        1 => {}
        2 => {
            let mut duplicate = candidate.snapshot.constitution.operators[0].modes[0].clone();
            duplicate.id = ModeLocalId::new(3);
            candidate.snapshot.constitution.operators[0]
                .modes
                .push(duplicate);
            let dependency = LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
                operator: OperatorLocalId::new(1),
                mode: ModeLocalId::new(3),
            });
            candidate.snapshot.constitution.formations[0]
                .direct_dependencies
                .push(dependency.clone());
            candidate.snapshot.constitution.formations[0]
                .direct_dependencies
                .sort();
            for application in &mut candidate.snapshot.constitution.applications {
                application.form.eligible_modes.push(ModeLocalId::new(3));
                application.form.eligible_modes.sort();
                application.form.dependency_closure.push(dependency.clone());
                application.form.dependency_closure.sort();
            }
        }
        _ => panic!("fixture supports zero, one, or two eligible checker Modes"),
    }
    candidate.snapshot.state_admission_grants = state_admission_scopes
        .into_iter()
        .enumerate()
        .map(|(ordinal, scope)| RevisionStateAdmissionGrantPreimageV2 {
            authorization: AdmissionAuthorizationLocalId::new(
                u32::try_from(ordinal).expect("fixture grant count fits u32"),
            ),
            scope,
        })
        .collect();
    candidate.claimed_snapshot =
        derive_program_snapshot_id(&candidate.snapshot).expect("program snapshot is canonical");
    let bytes = encode_process_package(&candidate).expect("program package encodes");
    check_process_package(decode_process_package(&bytes).expect("program package decodes"))
        .expect("program package checks")
}

fn physical_plan(package: &CheckedProcessPackage) -> ExecutablePhysicalPlanV1 {
    physical_plan_with_source(package, WORLD)
}

fn physical_plan_with_source(
    package: &CheckedProcessPackage,
    source: &[u8],
) -> ExecutablePhysicalPlanV1 {
    let constitution = package.constitution();
    let snapshot = constitution.snapshot();
    let application = ApplicationLocalId::new(1);
    let role = |id| LocalRoleRefV2 {
        schema: RelationSchemaLocalId::new(2),
        role: RoleLocalId::new(id),
    };
    let scope = TermScope {
        universe: constitution.universe(),
        semantics: constitution.semantics(),
    };
    let (input_handler, jump_handler, tick_program, scalar_handlers) =
        source_handlers(source, scope);
    let input_events = vec![
        ExecutableInputBindingV1 {
            role: role(15),
            source: ExecutableInputSourceV1::Keyboard {
                code: b"KeyA".to_vec(),
                phase: ExecutableKeyPhaseV1::Down,
            },
            occurrence: occurrence(0, &[-1.0, 0.0]),
        },
        ExecutableInputBindingV1 {
            role: role(16),
            source: ExecutableInputSourceV1::Keyboard {
                code: b"KeyA".to_vec(),
                phase: ExecutableKeyPhaseV1::Up,
            },
            occurrence: occurrence(0, &[0.0, 0.0]),
        },
        ExecutableInputBindingV1 {
            role: role(17),
            source: ExecutableInputSourceV1::Keyboard {
                code: b"KeyD".to_vec(),
                phase: ExecutableKeyPhaseV1::Down,
            },
            occurrence: occurrence(0, &[1.0, 0.0]),
        },
        ExecutableInputBindingV1 {
            role: role(18),
            source: ExecutableInputSourceV1::Keyboard {
                code: b"KeyD".to_vec(),
                phase: ExecutableKeyPhaseV1::Up,
            },
            occurrence: occurrence(0, &[0.0, 0.0]),
        },
        ExecutableInputBindingV1 {
            role: role(19),
            source: ExecutableInputSourceV1::Keyboard {
                code: b"Space".to_vec(),
                phase: ExecutableKeyPhaseV1::Down,
            },
            occurrence: occurrence(1, &[]),
        },
    ];
    ExecutablePhysicalPlanV1 {
        application_shape: constitution
            .application_shape(application)
            .expect("fixture Application has one exact semantic shape"),
        mode: ModeId {
            operator: OperatorRef {
                snapshot,
                local: OperatorLocalId::new(1),
            },
            local: ModeLocalId::new(2),
        },
        refinement: ExecutableRefinementV1::ClosedApplicationRuleMachineV1,
        target: ExecutablePhysicalTargetV1::PortableScalarInterpreterV1,
        input: Some(ExecutableInputPlanV1 {
            events: input_events,
            tick: ExecutableTickBindingV1 {
                role: role(20),
                entries: std::iter::once(2)
                    .chain((0..scalar_handlers.len()).map(|index| {
                        3 + u16::try_from(index).expect("bounded handler index fits u16")
                    }))
                    .collect(),
            },
        }),
        program: headless_program(
            scope,
            &input_handler,
            &jump_handler,
            &tick_program,
            &scalar_handlers,
        ),
    }
}

fn scalar_plan_with_source_and_projection(
    package: &CheckedProcessPackage,
    source: &[u8],
    object_field: &'static [u8],
    value_field: &'static [u8],
) -> ExecutablePhysicalPlanV1 {
    let constitution = package.constitution();
    let snapshot = constitution.snapshot();
    let application = ApplicationLocalId::new(1);
    let scope = TermScope {
        universe: constitution.universe(),
        semantics: constitution.semantics(),
    };
    let handler = source_scalar_handler(source, scope);
    ExecutablePhysicalPlanV1 {
        application_shape: constitution
            .application_shape(application)
            .expect("fixture Application has one exact semantic shape"),
        mode: ModeId {
            operator: OperatorRef {
                snapshot,
                local: OperatorLocalId::new(1),
            },
            local: ModeLocalId::new(2),
        },
        refinement: ExecutableRefinementV1::ClosedApplicationRuleMachineV1,
        target: ExecutablePhysicalTargetV1::PortableScalarInterpreterV1,
        input: None,
        program: scalar_program(scope, &handler, object_field, value_field),
    }
}

fn browser_state_admission_scopes(checker_count: usize) -> Vec<StateAdmissionScope> {
    if checker_count != 1 {
        return Vec::new();
    }
    let package = checked_program_package_with_scopes(checker_count, Vec::new());
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let physical_plan = physical_plan(&package);
    let (authority, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_RECORDED_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &physical_plan,
        facts.executable(),
    )
    .expect("fixture allocation evidence binds the provisional package");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        physical_plan,
        facts.executable(),
        allocation,
    )
    .expect("fixture scope derivation opens one exact session");
    let mut scopes = Vec::with_capacity(BROWSER_ADMISSION_COUNT);
    for ordinal in 0..BROWSER_ADMISSION_COUNT {
        session
            .apply_opaque_input(
                &encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0])).unwrap(),
            )
            .expect("fixture horizontal input advances");
        session
            .apply_opaque_input_and_emit_candidate(
                &encode_executable_occurrence_v1(&occurrence(2, &[0.25])).unwrap(),
            )
            .expect("fixture tick emits one candidate");
        let candidate = session.candidate().unwrap().unwrap().clone();
        scopes.push(StateAdmissionScope {
            session: facts.session,
            base: candidate.base,
            delta: candidate.id,
        });
        let (policy, authorization) = exact_root_admission_policy(
            package_id,
            facts.session,
            candidate.base,
            candidate.id,
            130 + u8::try_from(ordinal).expect("fixture ordinal fits u8"),
        );
        session
            .establish_root_policy(policy)
            .expect("scope derivation receives fresh external authority");
        session
            .admit_candidate(authorization)
            .expect("scope derivation reaches the next exact base");
    }
    scopes
}

fn checked_program_package(checker_count: usize) -> CheckedProcessPackage {
    checked_program_package_with_scopes(
        checker_count,
        browser_state_admission_scopes(checker_count),
    )
}

fn browser_gameplay_state_admission_scopes(source: &[u8]) -> Vec<StateAdmissionScope> {
    let package = checked_program_package_with_scopes_and_roles(1, Vec::new(), 25);
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let physical_plan = physical_plan_with_source(&package, source);
    let (authority, facts) = carrier_authority_for_plan(
        &package,
        &physical_plan,
        BROWSER_GAMEPLAY_ALLOCATION_ROOT_TAG,
    );
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_GAMEPLAY_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &physical_plan,
        facts.executable(),
    )
    .expect("gameplay allocation evidence binds the provisional package");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        physical_plan,
        facts.executable(),
        allocation,
    )
    .expect("gameplay scope derivation opens one exact session");

    session
        .apply_opaque_input(&encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0])).unwrap())
        .expect("gameplay horizontal input advances");
    session
        .apply_fixed_tick_and_emit_candidate(16)
        .expect("gameplay tick executes movement and contact as one candidate chain");
    let collect = session
        .candidate()
        .expect("automatic-contact candidate lookup succeeds")
        .expect("automatic-contact candidate exists")
        .clone();
    let scopes = vec![StateAdmissionScope {
        session: facts.session,
        base: collect.base,
        delta: collect.id,
    }];
    let (collect_policy, collect_authorization) =
        exact_root_admission_policy(package_id, facts.session, collect.base, collect.id, 243);
    session
        .establish_root_policy(collect_policy)
        .expect("automatic contact receives separate external authority");
    session
        .admit_candidate(collect_authorization)
        .expect("automatic contact establishes the collected configuration");
    scopes
}

fn checked_gameplay_program_package(source: &[u8]) -> CheckedProcessPackage {
    checked_program_package_with_scopes_and_roles(
        1,
        browser_gameplay_state_admission_scopes(source),
        25,
    )
}

fn browser_dash_gameplay_state_admission_scope(source: &[u8]) -> StateAdmissionScope {
    let package = checked_program_package_with_scopes_and_roles(1, Vec::new(), 25);
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let physical_plan = physical_plan_with_source(&package, source);
    let (authority, facts) = carrier_authority_for_plan(
        &package,
        &physical_plan,
        BROWSER_GAMEPLAY_DASH_ALLOCATION_ROOT_TAG,
    );
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_GAMEPLAY_DASH_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &physical_plan,
        facts.executable(),
    )
    .expect("dash-jump allocation evidence binds the provisional package");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        physical_plan,
        facts.executable(),
        allocation,
    )
    .expect("dash-jump scope derivation opens one exact session");

    session
        .apply_opaque_input(&encode_executable_occurrence_v1(&occurrence(1, &[])).unwrap())
        .expect("dash-jump input advances locally");
    session
        .apply_opaque_input_and_emit_candidate(
            &encode_executable_occurrence_v1(&occurrence(2, &[0.016])).unwrap(),
        )
        .expect("dash-jump tick emits one candidate");
    let candidate = session
        .candidate()
        .expect("dash-jump candidate lookup succeeds")
        .expect("dash-jump candidate exists");
    StateAdmissionScope {
        session: facts.session,
        base: candidate.base,
        delta: candidate.id,
    }
}

fn checked_dash_gameplay_program_package(source: &[u8]) -> CheckedProcessPackage {
    checked_program_package_with_scopes_and_roles(
        1,
        vec![browser_dash_gameplay_state_admission_scope(source)],
        25,
    )
}

fn browser_scalar_state_admission_scope_with_projection(
    source: &[u8],
    allocation_tag: u8,
    object_field: &'static [u8],
    value_field: &'static [u8],
) -> StateAdmissionScope {
    let package = checked_program_package_with_scopes(1, Vec::new());
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let physical_plan =
        scalar_plan_with_source_and_projection(&package, source, object_field, value_field);
    let (authority, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(allocation_tag),
        &package,
        application,
        &physical_plan,
        facts.executable(),
    )
    .expect("scalar browser allocation evidence binds the provisional package");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        physical_plan,
        facts.executable(),
        allocation,
    )
    .expect("scalar browser scope derivation opens one exact session");
    session
        .apply_opaque_input_and_emit_candidate(
            &encode_executable_occurrence_v1(&occurrence(0, &[]))
                .expect("scalar browser occurrence encodes"),
        )
        .expect("scalar browser occurrence emits one candidate");
    let candidate = session
        .candidate()
        .expect("scalar browser candidate lookup succeeds")
        .expect("scalar browser Step retains one candidate");
    StateAdmissionScope {
        session: facts.session,
        base: candidate.base,
        delta: candidate.id,
    }
}

fn checked_scalar_program_package_with_projection(
    source: &[u8],
    allocation_tag: u8,
    object_field: &'static [u8],
    value_field: &'static [u8],
) -> CheckedProcessPackage {
    checked_program_package_with_scopes(
        1,
        vec![browser_scalar_state_admission_scope_with_projection(
            source,
            allocation_tag,
            object_field,
            value_field,
        )],
    )
}

fn exact_root_admission_policy(
    package: ProcessPackageId,
    session: RuntimeSessionId,
    base: StateRevisionId,
    candidate: CandidateDeltaId,
    tag: u8,
) -> (RootPolicyAnchor, AdmissionAuthorizationEvidence) {
    let policy = id!(RootPolicyId, tag);
    let authorization = RootAdmissionAuthorizationRef {
        policy,
        local: AdmissionAuthorizationLocalId::new(0),
    };
    (
        RootPolicyAnchor::establish_with_governance(
            policy,
            vec![],
            vec![],
            vec![RootStateAdmissionGrant {
                authorization,
                scope: CheckedStateAdmissionScope {
                    package,
                    session,
                    base,
                    delta: candidate,
                },
            }],
            vec![],
            vec![],
        )
        .expect("scope derivation root policy is coherent"),
        AdmissionAuthorizationEvidence::IrreducibleRoot {
            policy,
            authorization,
        },
    )
}

#[derive(Clone, Copy)]
struct CarrierFacts {
    revision: ProgramRevisionId,
    initial_state: StateRevisionId,
    session: RuntimeSessionId,
    policy: RuntimePolicyId,
    session_start: SessionStartOccurrenceId,
    root_policy: RootPolicyId,
    pure_boundary: BoundaryRef,
    state_boundary: BoundaryRef,
}

impl CarrierFacts {
    fn executable(self) -> ExecutableAuthorityFactsV1 {
        ExecutableAuthorityFactsV1 {
            program_revision: self.revision,
            session: self.session,
            initial_state: self.initial_state,
            policy: self.policy,
            session_start: self.session_start,
            root_policy: self.root_policy,
            judgment_authority: RootJudgmentAuthorityRef {
                policy: self.root_policy,
                local: JudgmentAuthorityLocalId::new(0),
            },
            admission_authorization_issuer: RootAdmissionAuthorizationIssuerRef {
                policy: self.root_policy,
                local: AdmissionAuthorizationIssuerLocalId::new(0),
            },
            trigger_ingress: ExecutableBoundaryFactV1 {
                boundary: self.pure_boundary,
                evidence: id!(ExternalEvidenceRef, 181),
                permission: EXECUTABLE_TRIGGER_PERMISSION_V1,
            },
            occurrence_ingress: ExecutableBoundaryFactV1 {
                boundary: self.pure_boundary,
                evidence: id!(ExternalEvidenceRef, 181),
                permission: EXECUTABLE_OBSERVATION_PERMISSION_V1,
            },
            judgment_ingress: ExecutableBoundaryFactV1 {
                boundary: self.state_boundary,
                evidence: id!(ExternalEvidenceRef, 186),
                permission: EXECUTABLE_JUDGMENT_PERMISSION_V1,
            },
            admission_issuance_ingress: ExecutableBoundaryFactV1 {
                boundary: self.state_boundary,
                evidence: id!(ExternalEvidenceRef, 190),
                permission: EXECUTABLE_ADMISSION_ISSUANCE_PERMISSION_V1,
            },
            admission_ingress: ExecutableBoundaryFactV1 {
                boundary: self.state_boundary,
                evidence: id!(ExternalEvidenceRef, 190),
                permission: EXECUTABLE_ADMISSION_PERMISSION_V1,
            },
            budget_units: 100,
        }
    }

    fn admission_authorization(self) -> AdmissionAuthorizationEvidence {
        AdmissionAuthorizationEvidence::IrreducibleRoot {
            policy: self.root_policy,
            authorization: RootAdmissionAuthorizationRef {
                policy: self.root_policy,
                local: AdmissionAuthorizationLocalId::new(1),
            },
        }
    }
}

fn carrier_authority(checked: &CheckedProcessPackage) -> (AuthorityStore, CarrierFacts) {
    let physical_plan = physical_plan(checked);
    carrier_authority_for_plan(
        checked,
        &physical_plan,
        BROWSER_RECORDED_ALLOCATION_ROOT_TAG,
    )
}

fn carrier_authority_for_plan(
    checked: &CheckedProcessPackage,
    physical_plan: &ExecutablePhysicalPlanV1,
    allocation_tag: u8,
) -> (AuthorityStore, CarrierFacts) {
    let semantics = checked.constitution().semantics();
    let snapshot = checked.constitution().snapshot();
    let session = id!(RuntimeSessionId, 120);
    let policy = id!(RuntimePolicyId, 121);
    let session_start = id!(SessionStartOccurrenceId, 122);
    let revision = ProgramRevisionPreimage {
        semantics,
        program: id!(ProgramId, 123),
        predecessor: None,
        snapshot,
        change: id!(ProgramChangeOccurrenceId, 124),
    }
    .derived_claim();
    let initial_view = checked
        .initial_state_views()
        .first()
        .expect("program package retains its initial State view");
    let session_anchor = RuntimeSessionAnchor::establish(
        session,
        revision.id,
        semantics,
        policy,
        session_start,
        initial_view.canonical_state_snapshot.to_vec(),
    );
    let initial_state = session_anchor.initial_state_id();
    let root_policy = id!(RootPolicyId, 125);
    let pure_boundary = id!(BoundaryRef, 126);
    let state_boundary = id!(BoundaryRef, 127);
    let facts = CarrierFacts {
        revision: revision.id,
        initial_state,
        session,
        policy,
        session_start,
        root_policy,
        pure_boundary,
        state_boundary,
    };
    let application = ApplicationId {
        snapshot,
        local: ApplicationLocalId::new(1),
    };
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(allocation_tag),
        checked,
        application,
        physical_plan,
        facts.executable(),
    )
    .expect("fixture authority scopes one recorded allocation epoch");
    let root_genesis = RootAdmissionAuthorizationRef {
        policy: root_policy,
        local: AdmissionAuthorizationLocalId::new(0),
    };
    let mut authority = AuthorityStore::new();
    authority
        .establish_root_policy(
            RootPolicyAnchor::establish_with_governance(
                root_policy,
                vec![RootGenesisGrant {
                    authorization: root_genesis,
                    scope: RootGenesisScope {
                        semantics,
                        program: revision.preimage.program,
                        snapshot,
                        change: revision.preimage.change,
                    },
                }],
                vec![],
                vec![RootStateAdmissionGrant {
                    authorization: RootAdmissionAuthorizationRef {
                        policy: root_policy,
                        local: AdmissionAuthorizationLocalId::new(1),
                    },
                    scope: CheckedStateAdmissionScope {
                        package: checked.id(),
                        session,
                        base: initial_state,
                        delta: allocation.candidate_id(0),
                    },
                }],
                vec![RootJudgmentAuthorityGrant {
                    authority: RootJudgmentAuthorityRef {
                        policy: root_policy,
                        local: JudgmentAuthorityLocalId::new(0),
                    },
                    scope: JudgmentAuthorityScope {
                        semantics,
                        session,
                        policy,
                    },
                }],
                vec![],
            )
            .expect("root policy is coherent"),
        )
        .expect("root policy is established once");
    authority
        .admit_genesis(
            revision,
            checked.authority_input(),
            root_policy,
            root_genesis,
        )
        .expect("root authority admits the executable snapshot");
    authority
        .establish_runtime_session(session_anchor)
        .expect("runtime session is established once");
    let boundary_target = checked.constitution().preimage().formations[0]
        .target
        .clone();
    let admitted = CheckedConstitutionBinding::Admitted {
        revision: revision.id,
    };
    authority
        .establish_boundary(executable_occurrence_boundary_anchor_v1(
            pure_boundary,
            boundary_target.clone(),
            BoundaryPins {
                semantics,
                snapshot,
                constitution: admitted,
                runtime_session: None,
                observed_state: None,
                runtime_policy: None,
            },
        ))
        .expect("pure boundary is established once");
    authority
        .establish_boundary(executable_state_boundary_anchor_v1(
            state_boundary,
            boundary_target,
            BoundaryPins {
                semantics,
                snapshot,
                constitution: admitted,
                runtime_session: Some(session),
                observed_state: None,
                runtime_policy: Some(policy),
            },
        ))
        .expect("state boundary is established once");
    for (tag, boundary, permissions) in [
        (
            181,
            pure_boundary,
            vec![
                EXECUTABLE_TRIGGER_PERMISSION_V1,
                EXECUTABLE_OBSERVATION_PERMISSION_V1,
            ],
        ),
        (
            183,
            pure_boundary,
            vec![EXECUTABLE_OBSERVATION_PERMISSION_V1],
        ),
        (186, state_boundary, vec![EXECUTABLE_JUDGMENT_PERMISSION_V1]),
        (187, state_boundary, vec![EXECUTABLE_JUDGMENT_PERMISSION_V1]),
        (
            190,
            state_boundary,
            vec![EXECUTABLE_ADMISSION_PERMISSION_V1],
        ),
    ] {
        authority
            .establish_evidence(EvidenceAnchor {
                evidence: id!(ExternalEvidenceRef, tag),
                boundary,
                permissions,
                exact_evidence: vec![tag].into_boxed_slice(),
            })
            .expect("external evidence is established once");
    }
    (authority, facts)
}

fn occurrence(entry: u16, arguments: &[f64]) -> ExecutableOccurrenceV1 {
    ExecutableOccurrenceV1 {
        entry,
        arguments: arguments.iter().copied().map(number).collect(),
    }
}

fn value(configuration: &[ExecutableValueV1], slot: usize) -> f64 {
    configuration[slot]
        .as_number()
        .expect("selected slot is numeric")
}

fn projected_object_field<'a>(term: &'a Term, expected: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        if let Some(end) = current.as_atom() {
            assert_eq!(end.kind(), b"clause/js-object-end-v1");
            panic!("projected object lacks field {:?}", expected);
        }
        let [field, value, rest] = current
            .as_triple()
            .expect("projected object is an entry chain")
            .slots();
        let field = field.as_atom().expect("projected object field is an Atom");
        assert_eq!(field.kind(), b"clause/js-field-v1");
        if field.canonical_payload() == expected {
            return value;
        }
        current = rest;
    }
}

fn projected_array_first(term: &Term) -> &Term {
    let [item, value, _rest] = term
        .as_triple()
        .expect("projected array has at least one item")
        .slots();
    assert_eq!(
        item.as_atom().expect("projected item is an Atom").kind(),
        b"clause/js-item-v1"
    );
    value
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

fn assert_arena_projection(term: &Term, expected_x: f64, expected_velocity_x: f64) {
    let player = projected_object_field(term, b"player");
    let position = projected_object_field(player, b"position");
    assert_eq!(
        projected_number(projected_object_field(position, b"x")),
        expected_x
    );
    assert_eq!(
        [b"y".as_slice(), b"z".as_slice()]
            .map(|field| projected_number(projected_object_field(position, field))),
        [0.0, 0.0]
    );
    let velocity = projected_object_field(player, b"velocity");
    assert_eq!(
        projected_number(projected_object_field(velocity, b"x")),
        expected_velocity_x
    );
    assert_eq!(
        [b"y".as_slice(), b"z".as_slice()]
            .map(|field| projected_number(projected_object_field(velocity, field))),
        [0.0, 0.0]
    );
    assert_eq!(
        projected_number(projected_object_field(player, b"yaw")),
        0.0
    );
    assert_eq!(
        projected_object_field(player, b"grounded")
            .as_atom()
            .expect("projected Boolean Atom")
            .canonical_payload(),
        [1]
    );
    let world = projected_object_field(term, b"world");
    let platform = projected_array_first(projected_object_field(world, b"platforms"));
    let platform_position = projected_object_field(platform, b"position");
    assert_eq!(
        [b"x".as_slice(), b"y".as_slice(), b"z".as_slice()]
            .map(|field| { projected_number(projected_object_field(platform_position, field)) }),
        [0.0, -0.25, 0.0]
    );
    let platform_size = projected_object_field(platform, b"size");
    assert_eq!(
        [b"x".as_slice(), b"y".as_slice(), b"z".as_slice()]
            .map(|field| projected_number(projected_object_field(platform_size, field))),
        [12.0, 0.5, 12.0]
    );
}

fn assert_jump_projection(term: &Term, expected_velocity_y: f64) {
    let player = projected_object_field(term, b"player");
    let velocity = projected_object_field(player, b"velocity");
    assert_eq!(
        [b"x".as_slice(), b"y".as_slice(), b"z".as_slice()]
            .map(|field| projected_number(projected_object_field(velocity, field))),
        [0.0, expected_velocity_y, 0.0]
    );
    assert_eq!(
        projected_object_field(player, b"grounded")
            .as_atom()
            .expect("projected Boolean Atom")
            .canonical_payload(),
        [0]
    );
}

fn admit_source_jump(source: &[u8], allocation_tag: u8, policy_tag: u8) -> (Vec<u8>, Term) {
    let package = checked_program_package_with_scopes(1, vec![]);
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let plan = physical_plan_with_source(&package, source);
    let plan_bytes = encode_executable_physical_plan_v1(&plan)
        .expect("source-owned jump produces one exact CPP1 plan");
    let (authority, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(allocation_tag),
        &package,
        application,
        &plan,
        facts.executable(),
    )
    .expect("source jump session receives one recorded allocation root");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        plan,
        facts.executable(),
        allocation,
    )
    .expect("source jump session starts through the persistent runtime");
    session
        .apply_opaque_input_and_emit_candidate(
            &encode_executable_occurrence_v1(&occurrence(1, &[]))
                .expect("source jump occurrence encodes"),
        )
        .expect("source-owned jump meaning produces one hidden candidate");
    let candidate = session
        .candidate()
        .expect("candidate lookup succeeds")
        .expect("jump Step retains one candidate")
        .clone();
    assert!(session.last_admitted().is_none());
    let (policy, authorization) = exact_root_admission_policy(
        package_id,
        facts.session,
        candidate.base,
        candidate.id,
        policy_tag,
    );
    session
        .establish_root_policy(policy)
        .expect("separate external jump authority is established");
    let (successor, projection) = session
        .admit_candidate_with_projection(authorization)
        .expect("separate jump Admission creates the successor and projection");
    assert_eq!(successor.predecessor, facts.initial_state);
    let projection = projection.expect("admitted jump emits the renderer projection");
    assert_eq!(projection.state, successor.id);
    (plan_bytes, projection.term)
}

fn admit_source_scalar(source: &[u8], allocation_tag: u8, policy_tag: u8) -> (Vec<u8>, Term) {
    admit_source_scalar_with_projection(source, allocation_tag, policy_tag, b"player", b"score")
}

fn admit_source_scalar_with_projection(
    source: &[u8],
    allocation_tag: u8,
    policy_tag: u8,
    object_field: &'static [u8],
    value_field: &'static [u8],
) -> (Vec<u8>, Term) {
    let package = checked_program_package_with_scopes(1, vec![]);
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let plan = scalar_plan_with_source_and_projection(&package, source, object_field, value_field);
    let plan_bytes = encode_executable_physical_plan_v1(&plan)
        .expect("source-owned scalar transition produces one exact CPP1 plan");
    let (authority, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(allocation_tag),
        &package,
        application,
        &plan,
        facts.executable(),
    )
    .expect("scalar transition session receives one recorded allocation root");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        plan,
        facts.executable(),
        allocation,
    )
    .expect("scalar transition session starts through the persistent runtime");
    session
        .apply_opaque_input_and_emit_candidate(
            &encode_executable_occurrence_v1(&occurrence(0, &[]))
                .expect("scalar occurrence encodes"),
        )
        .expect("source-owned scalar transition produces one hidden candidate");
    let candidate = session
        .candidate()
        .expect("candidate lookup succeeds")
        .expect("scalar Step retains one candidate")
        .clone();
    assert!(session.last_admitted().is_none());
    assert_eq!(session.world_base(), facts.initial_state);
    let (policy, authorization) = exact_root_admission_policy(
        package_id,
        facts.session,
        candidate.base,
        candidate.id,
        policy_tag,
    );
    session
        .establish_root_policy(policy)
        .expect("separate external scalar authority is established");
    let (successor, projection) = session
        .admit_candidate_with_projection(authorization)
        .expect("separate scalar Admission creates the successor and projection");
    assert_eq!(successor.predecessor, facts.initial_state);
    assert_eq!(session.world_base(), successor.id);
    let projection = projection.expect("admitted scalar transition emits renderer projection");
    assert_eq!(projection.state, successor.id);
    (plan_bytes, projection.term)
}

fn admit_source_tick(source: &[u8], allocation_tag: u8, policy_tag: u8) -> (Vec<u8>, Term) {
    let package = checked_program_package_with_scopes(1, vec![]);
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let plan = physical_plan_with_source(&package, source);
    let plan_bytes = encode_executable_physical_plan_v1(&plan)
        .expect("source-owned tick produces one exact CPP1 plan");
    let (authority, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(allocation_tag),
        &package,
        application,
        &plan,
        facts.executable(),
    )
    .expect("source tick session receives one recorded allocation root");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        plan,
        facts.executable(),
        allocation,
    )
    .expect("source tick session starts through the persistent runtime");
    session
        .apply_opaque_input(
            &encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0]))
                .expect("source input occurrence encodes"),
        )
        .expect("source-owned horizontal intent enters before tick");
    session
        .apply_opaque_input_and_emit_candidate(
            &encode_executable_occurrence_v1(&occurrence(2, &[3.0]))
                .expect("source tick occurrence encodes"),
        )
        .expect("source-owned tick meaning produces one hidden candidate");
    let candidate = session
        .candidate()
        .expect("candidate lookup succeeds")
        .expect("tick Step retains one candidate")
        .clone();
    assert!(session.last_admitted().is_none());
    let (policy, authorization) = exact_root_admission_policy(
        package_id,
        facts.session,
        candidate.base,
        candidate.id,
        policy_tag,
    );
    session
        .establish_root_policy(policy)
        .expect("separate external tick authority is established");
    let (successor, projection) = session
        .admit_candidate_with_projection(authorization)
        .expect("separate tick Admission creates the successor and projection");
    assert_eq!(successor.predecessor, facts.initial_state);
    let projection = projection.expect("admitted tick emits the renderer projection");
    assert_eq!(projection.state, successor.id);
    (plan_bytes, projection.term)
}

fn admit_source_jump_then_tick(
    source: &[u8],
    allocation_tag: u8,
    policy_tag: u8,
) -> (Vec<u8>, Term) {
    let package = checked_program_package_with_scopes(1, vec![]);
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let plan = physical_plan_with_source(&package, source);
    let plan_bytes = encode_executable_physical_plan_v1(&plan)
        .expect("source-owned air momentum produces one exact CPP1 plan");
    let (authority, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(allocation_tag),
        &package,
        application,
        &plan,
        facts.executable(),
    )
    .expect("air-momentum session receives one recorded allocation root");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        plan,
        facts.executable(),
        allocation,
    )
    .expect("air-momentum session starts through the persistent runtime");
    session
        .apply_opaque_input(
            &encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0]))
                .expect("horizontal intent occurrence encodes"),
        )
        .expect("horizontal intent enters before jumping");

    session
        .apply_opaque_input_and_emit_candidate(
            &encode_executable_occurrence_v1(&occurrence(1, &[])).expect("jump occurrence encodes"),
        )
        .expect("jump emits one hidden candidate");
    let jump_candidate = session
        .candidate()
        .expect("jump candidate lookup succeeds")
        .expect("jump candidate exists")
        .clone();
    let (jump_policy, jump_authorization) = exact_root_admission_policy(
        package_id,
        facts.session,
        jump_candidate.base,
        jump_candidate.id,
        policy_tag,
    );
    session
        .establish_root_policy(jump_policy)
        .expect("jump authority is established separately");
    let (jump_successor, _) = session
        .admit_candidate_with_projection(jump_authorization)
        .expect("jump Admission creates the airborne state");
    let airborne = session
        .configuration()
        .expect("jump successor installs one live configuration");
    assert_eq!(value(airborne, 2), 0.0);
    assert_eq!(value(airborne, 3), 8.0);
    assert_eq!(value(airborne, 4), 1.0);
    assert_eq!(airborne[5], ExecutableValueV1::Boolean(false));

    session
        .apply_opaque_input_and_emit_candidate(
            &encode_executable_occurrence_v1(&occurrence(2, &[0.25]))
                .expect("airborne tick occurrence encodes"),
        )
        .expect("airborne tick emits one hidden candidate");
    let tick_candidate = session
        .candidate()
        .expect("tick candidate lookup succeeds")
        .expect("tick candidate exists")
        .clone();
    assert_eq!(tick_candidate.base, jump_successor.id);
    let (tick_policy, tick_authorization) = exact_root_admission_policy(
        package_id,
        facts.session,
        tick_candidate.base,
        tick_candidate.id,
        policy_tag + 1,
    );
    session
        .establish_root_policy(tick_policy)
        .expect("tick authority is established separately");
    let (tick_successor, projection) = session
        .admit_candidate_with_projection(tick_authorization)
        .expect("airborne tick Admission creates the successor");
    assert_eq!(tick_successor.predecessor, jump_successor.id);
    let projection = projection.expect("admitted airborne tick emits the renderer projection");
    assert_eq!(projection.state, tick_successor.id);
    (plan_bytes, projection.term)
}

fn assert_tick_projection(term: &Term, expected_x: f64) {
    let player = projected_object_field(term, b"player");
    let position = projected_object_field(player, b"position");
    assert_eq!(
        projected_number(projected_object_field(position, b"x")),
        expected_x
    );
    assert_eq!(
        projected_number(projected_object_field(position, b"z")),
        0.0
    );
    assert_eq!(
        projected_object_field(player, b"grounded")
            .as_atom()
            .expect("projected Boolean Atom")
            .canonical_payload(),
        [1]
    );
}

fn browser_fixture_request() -> WasmProcessRequestV1 {
    let package = checked_program_package(1);
    let physical_plan = physical_plan(&package);
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let (_, allocation_facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_RECORDED_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &physical_plan,
        allocation_facts.executable(),
    )
    .expect("fixture allocation evidence binds the final package and plan");
    WasmProcessRequestV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: ApplicationLocalId::new(1),
        physical_plan_bytes: encode_executable_physical_plan_v1(&physical_plan)
            .expect("fixture physical plan encodes beside the package"),
        allocation,
        authority: WasmAuthorityInputV1 {
            program: id!(ProgramId, 123),
            change: id!(ProgramChangeOccurrenceId, 124),
            session: id!(RuntimeSessionId, 120),
            policy: id!(RuntimePolicyId, 121),
            session_start: id!(SessionStartOccurrenceId, 122),
            root_policy: id!(RootPolicyId, 125),
            occurrence_boundary: id!(BoundaryRef, 126),
            state_boundary: id!(BoundaryRef, 127),
            occurrence_evidence: id!(ExternalEvidenceRef, 181),
            occurrence_evidence_bytes: vec![181],
            judgment_evidence: id!(ExternalEvidenceRef, 186),
            judgment_evidence_bytes: vec![186],
            admission_evidence: id!(ExternalEvidenceRef, 190),
            admission_evidence_bytes: vec![190],
            budget_units: 100,
        },
        occurrences: vec![
            encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0]))
                .expect("fixture input occurrence encodes"),
            encode_executable_occurrence_v1(&occurrence(2, &[0.25]))
                .expect("fixture tick occurrence encodes"),
        ],
        render_slots: vec![],
    }
}

fn browser_gameplay_fixture_request(
    source: &[u8],
    allocation_root_tag: u8,
) -> WasmProcessRequestV1 {
    let package = checked_gameplay_program_package(source);
    browser_gameplay_fixture_request_for_package(package, source, allocation_root_tag, false)
}

fn browser_dash_gameplay_fixture_request(source: &[u8]) -> WasmProcessRequestV1 {
    let package = checked_dash_gameplay_program_package(source);
    browser_gameplay_fixture_request_for_package(
        package,
        source,
        BROWSER_GAMEPLAY_DASH_ALLOCATION_ROOT_TAG,
        true,
    )
}

fn browser_gameplay_fixture_request_for_package(
    package: CheckedProcessPackage,
    source: &[u8],
    allocation_root_tag: u8,
    dash_jump: bool,
) -> WasmProcessRequestV1 {
    let physical_plan = physical_plan_with_source(&package, source);
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let (_, allocation_facts) =
        carrier_authority_for_plan(&package, &physical_plan, allocation_root_tag);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(allocation_root_tag),
        &package,
        application,
        &physical_plan,
        allocation_facts.executable(),
    )
    .expect("gameplay browser allocation evidence binds the final package and plan");
    WasmProcessRequestV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: ApplicationLocalId::new(1),
        physical_plan_bytes: encode_executable_physical_plan_v1(&physical_plan)
            .expect("gameplay browser physical plan encodes beside the package"),
        allocation,
        authority: WasmAuthorityInputV1 {
            program: id!(ProgramId, 123),
            change: id!(ProgramChangeOccurrenceId, 124),
            session: id!(RuntimeSessionId, 120),
            policy: id!(RuntimePolicyId, 121),
            session_start: id!(SessionStartOccurrenceId, 122),
            root_policy: id!(RootPolicyId, 125),
            occurrence_boundary: id!(BoundaryRef, 126),
            state_boundary: id!(BoundaryRef, 127),
            occurrence_evidence: id!(ExternalEvidenceRef, 181),
            occurrence_evidence_bytes: vec![181],
            judgment_evidence: id!(ExternalEvidenceRef, 186),
            judgment_evidence_bytes: vec![186],
            admission_evidence: id!(ExternalEvidenceRef, 190),
            admission_evidence_bytes: vec![190],
            budget_units: 100,
        },
        occurrences: if dash_jump {
            vec![
                encode_executable_occurrence_v1(&occurrence(1, &[]))
                    .expect("dash-jump occurrence encodes"),
                encode_executable_occurrence_v1(&occurrence(2, &[0.016]))
                    .expect("dash-jump tick occurrence encodes"),
            ]
        } else {
            vec![
                encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0]))
                    .expect("gameplay input occurrence encodes"),
                encode_executable_occurrence_v1(&occurrence(2, &[0.016]))
                    .expect("gameplay tick occurrence encodes"),
            ]
        },
        render_slots: vec![],
    }
}

fn browser_scalar_fixture_request(source: &[u8], allocation_tag: u8) -> WasmProcessRequestV1 {
    browser_scalar_fixture_request_with_projection(source, allocation_tag, b"player", b"score")
}

fn browser_scalar_fixture_request_with_projection(
    source: &[u8],
    allocation_tag: u8,
    object_field: &'static [u8],
    value_field: &'static [u8],
) -> WasmProcessRequestV1 {
    let package = checked_scalar_program_package_with_projection(
        source,
        allocation_tag,
        object_field,
        value_field,
    );
    let physical_plan =
        scalar_plan_with_source_and_projection(&package, source, object_field, value_field);
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let (_, allocation_facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(allocation_tag),
        &package,
        application,
        &physical_plan,
        allocation_facts.executable(),
    )
    .expect("scalar browser allocation evidence binds the final package and plan");
    WasmProcessRequestV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: ApplicationLocalId::new(1),
        physical_plan_bytes: encode_executable_physical_plan_v1(&physical_plan)
            .expect("scalar browser physical plan encodes beside the package"),
        allocation,
        authority: WasmAuthorityInputV1 {
            program: id!(ProgramId, 123),
            change: id!(ProgramChangeOccurrenceId, 124),
            session: id!(RuntimeSessionId, 120),
            policy: id!(RuntimePolicyId, 121),
            session_start: id!(SessionStartOccurrenceId, 122),
            root_policy: id!(RootPolicyId, 125),
            occurrence_boundary: id!(BoundaryRef, 126),
            state_boundary: id!(BoundaryRef, 127),
            occurrence_evidence: id!(ExternalEvidenceRef, 181),
            occurrence_evidence_bytes: vec![181],
            judgment_evidence: id!(ExternalEvidenceRef, 186),
            judgment_evidence_bytes: vec![186],
            admission_evidence: id!(ExternalEvidenceRef, 190),
            admission_evidence_bytes: vec![190],
            budget_units: 100,
        },
        occurrences: vec![
            encode_executable_occurrence_v1(&occurrence(0, &[]))
                .expect("scalar browser occurrence encodes"),
        ],
        render_slots: vec![],
    }
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

fn raw_id(tag: u8) -> [u8; IDENTITY_BYTES] {
    let mut bytes = [0; IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[IDENTITY_BYTES - 1] = tag;
    bytes
}

fn admit_spring_journey(source: &[u8], allocation_tag: u8, policy_tag: u8) -> (Vec<u8>, Term) {
    let package = checked_program_package_with_scopes_and_roles(1, Vec::new(), 25);
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let plan = physical_plan_with_source(&package, source);
    let plan_bytes = encode_executable_physical_plan_v1(&plan)
        .expect("spring CPP1 encodes from canonical source");
    let (authority, facts) = carrier_authority_for_plan(&package, &plan, allocation_tag);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(allocation_tag),
        &package,
        application,
        &plan,
        facts.executable(),
    )
    .expect("spring allocation binds the source-owned CPP1");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        plan,
        facts.executable(),
        allocation,
    )
    .expect("spring journey opens one persistent native session");
    session
        .apply_physical_input(&ExecutableInputSourceV1::Keyboard {
            code: b"KeyD".to_vec(),
            phase: ExecutableKeyPhaseV1::Down,
        })
        .expect("ordinary movement input is the only external trigger");

    session
        .apply_fixed_tick_and_emit_candidate(16)
        .expect("first tick moves through collection contact");
    session
        .admit_candidate(facts.admission_authorization())
        .expect("first separate Admission installs the collected position");

    let pre_launch_world = session.world_base();
    let pre_launch_run = session.run().expect("launch run is live");
    let pre_launch_activation = session.activation().expect("launch Activation is live");
    let launch_step = session
        .apply_fixed_tick_and_emit_candidate(16)
        .expect("second tick reaches the spring contact");
    let launch = session
        .candidate()
        .expect("launch candidate lookup succeeds")
        .expect("spring contact retains one hidden candidate")
        .clone();
    assert_eq!(launch.produced_by, launch_step.id);
    assert!(launch.configuration[3].as_number().unwrap() > 0.0);
    assert_eq!(launch.configuration[5].as_boolean(), Some(false));
    assert_eq!(
        session
            .configuration()
            .expect("launch configuration is live"),
        launch.configuration
    );
    assert_eq!(session.world_base(), pre_launch_world);
    assert_eq!(
        session.run().expect("launch Run remains live"),
        pre_launch_run
    );
    assert_eq!(
        session
            .activation()
            .expect("launch Activation remains live"),
        pre_launch_activation
    );
    let (launch_policy, launch_authorization) = exact_root_admission_policy(
        package_id,
        facts.session,
        launch.base,
        launch.id,
        policy_tag,
    );
    session
        .establish_root_policy(launch_policy)
        .expect("launch receives a separate exact Admission policy");
    session
        .admit_candidate(launch_authorization)
        .expect("separate Admission installs launch velocity");

    let launched_world = session.world_base();
    let airborne_step = session
        .apply_fixed_tick_and_emit_candidate(16)
        .expect("third tick advances the airborne position");
    let airborne = session
        .candidate()
        .expect("airborne candidate lookup succeeds")
        .expect("airborne position remains hidden before Admission")
        .clone();
    assert_eq!(airborne.produced_by, airborne_step.id);
    assert_eq!(session.world_base(), launched_world);
    let (airborne_policy, airborne_authorization) = exact_root_admission_policy(
        package_id,
        facts.session,
        airborne.base,
        airborne.id,
        policy_tag
            .checked_add(1)
            .expect("policy tag remains bounded"),
    );
    session
        .establish_root_policy(airborne_policy)
        .expect("airborne position receives separate exact authority");
    let (_, projection) = session
        .admit_candidate_with_projection(airborne_authorization)
        .expect("Admission alone exposes the launched frame");
    (
        plan_bytes,
        projection
            .expect("admitted spring journey projects one passive frame")
            .term,
    )
}

#[test]
fn canonical_source_input_reaches_persistent_admission_and_projection() {
    let package = checked_program_package_with_scopes(1, vec![]);
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let plan = physical_plan(&package);
    let (authority, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_RECORDED_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &plan,
        facts.executable(),
    )
    .expect("source-input session receives one recorded allocation root");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        plan,
        facts.executable(),
        allocation,
    )
    .expect("source-input session starts through the persistent runtime");

    session
        .apply_opaque_input_and_emit_candidate(
            &encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0]))
                .expect("source input occurrence encodes"),
        )
        .expect("source-owned input meaning produces one hidden candidate");
    let candidate = session
        .candidate()
        .expect("candidate lookup succeeds")
        .expect("input Step retains one candidate")
        .clone();
    assert!(session.last_admitted().is_none());
    let (policy, authorization) =
        exact_root_admission_policy(package_id, facts.session, candidate.base, candidate.id, 212);
    session
        .establish_root_policy(policy)
        .expect("separate external authority is established");
    let (successor, projection) = session
        .admit_candidate_with_projection(authorization)
        .expect("separate Admission creates the successor and projection");
    assert_eq!(successor.predecessor, facts.initial_state);
    assert_eq!(successor.configuration[4].as_number(), Some(1.0));
    assert_eq!(successor.configuration[21].as_number(), Some(0.0));
    let projection = projection.expect("admitted source input emits the renderer projection");
    assert_eq!(projection.state, successor.id);
    assert_arena_projection(&projection.term, 0.0, 0.0);
}

#[test]
fn canonical_source_jump_reaches_admission_and_source_only_changes_behavior() {
    let (base_plan, base_projection) = admit_source_jump(WORLD, 213, 216);
    assert_jump_projection(&base_projection, 8.0);

    let changed_speed = std::str::from_utf8(WORLD)
        .expect("arena source is UTF-8")
        .replacen("jump-arena jump speed 8.0", "jump-arena jump speed 9.25", 1);
    let (speed_plan, speed_projection) = admit_source_jump(changed_speed.as_bytes(), 214, 217);
    assert_ne!(speed_plan, base_plan);
    assert_jump_projection(&speed_projection, 9.25);

    let changed_include = std::str::from_utf8(WORLD)
        .expect("arena source is UTF-8")
        .replacen(
            "?player velocity Vec3 { x: ?velocity-x, y: ?jump-speed, z: ?velocity-z }",
            "?player velocity Vec3 { x: ?velocity-x, y: 6.5, z: ?velocity-z }",
            1,
        );
    let (include_plan, include_projection) =
        admit_source_jump(changed_include.as_bytes(), 215, 218);
    assert_ne!(include_plan, base_plan);
    assert_ne!(include_plan, speed_plan);
    assert_jump_projection(&include_projection, 6.5);
}

#[test]
fn canonical_source_tick_reaches_admission_and_source_only_clamp_change_alters_behavior() {
    let (base_plan, base_projection) = admit_source_tick(WORLD, 219, 222);
    assert_tick_projection(&base_projection, 10.0);

    let changed_maximum = std::str::from_utf8(WORLD)
        .expect("arena source is UTF-8")
        .replacen("jump-arena maximum x 10.0", "jump-arena maximum x 2.0", 1);
    let (changed_plan, changed_projection) =
        admit_source_tick(changed_maximum.as_bytes(), 220, 223);
    assert_ne!(changed_plan, base_plan);
    assert_tick_projection(&changed_projection, 2.0);
}

#[test]
fn canonical_source_collect_reaches_admission_without_a_host_semantic_switch() {
    let (base_plan, base_projection) = admit_source_scalar(COLLECT, 230, 232);
    let base_player = projected_object_field(&base_projection, b"player");
    assert_eq!(
        projected_number(projected_object_field(base_player, b"score")),
        1.0
    );

    let changed_source = std::str::from_utf8(COLLECT)
        .expect("collect source is UTF-8")
        .replacen(
            "?player score ?score + 1.0",
            "?player score ?score + 4.0",
            1,
        );
    let (changed_plan, changed_projection) =
        admit_source_scalar(changed_source.as_bytes(), 231, 233);
    assert_ne!(changed_plan, base_plan);
    let changed_player = projected_object_field(&changed_projection, b"player");
    assert_eq!(
        projected_number(projected_object_field(changed_player, b"score")),
        4.0
    );
}

#[test]
fn canonical_source_collect_changes_symbolic_state_only_after_admission() {
    let (base_plan, base_projection) =
        admit_source_scalar_with_projection(COLLECT_STATE, 236, 238, b"collectible", b"state");
    let base_state = projected_object_field(
        projected_object_field(&base_projection, b"collectible"),
        b"state",
    )
    .as_atom()
    .expect("projected symbolic state is an Atom");
    assert_eq!(base_state.kind(), b"clause/process-projected-symbol-v1");
    assert_eq!(base_state.canonical_payload(), b"collected");

    let changed_source = std::str::from_utf8(COLLECT_STATE)
        .expect("collect-state source is UTF-8")
        .replacen(
            "?collectible state collected",
            "?collectible state spent",
            1,
        );
    let (changed_plan, changed_projection) = admit_source_scalar_with_projection(
        changed_source.as_bytes(),
        237,
        239,
        b"collectible",
        b"state",
    );
    assert_ne!(changed_plan, base_plan);
    let changed_state = projected_object_field(
        projected_object_field(&changed_projection, b"collectible"),
        b"state",
    )
    .as_atom()
    .expect("changed projected symbolic state is an Atom");
    assert_eq!(changed_state.kind(), b"clause/process-projected-symbol-v1");
    assert_eq!(changed_state.canonical_payload(), b"spent");
}

#[test]
fn automatic_contact_tick_keeps_collection_hidden_until_admission_and_inactive_away() {
    let source = gameplay_source();
    let package = checked_program_package_with_scopes_and_roles(1, Vec::new(), 25);
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let plan = physical_plan_with_source(&package, &source);
    let input = plan
        .input
        .as_ref()
        .expect("gameplay CPP1 carries one physical input plan");
    assert_eq!(input.tick.entries, [2, 3, 4, 5]);
    assert!(input.events.iter().all(|binding| {
        !matches!(
            &binding.source,
            ExecutableInputSourceV1::Keyboard { code, .. } if code == b"KeyE"
        )
    }));
    let contact_rule = plan
        .program
        .rules
        .iter()
        .find(|rule| rule.entry == 3)
        .expect("Clause contact meaning lowers to the final chained entry");
    assert_eq!(
        contact_rule.predicates,
        [
            ExecutableExpressionV1::Equal(
                Box::new(ExecutableExpressionV1::Slot(28)),
                Box::new(ExecutableExpressionV1::Constant(
                    ExecutableValueV1::symbol(b"active").expect("active is a bounded symbol"),
                )),
            ),
            ExecutableExpressionV1::Equal(
                Box::new(ExecutableExpressionV1::Slot(0)),
                Box::new(ExecutableExpressionV1::Constant(number(0.08))),
            ),
            ExecutableExpressionV1::Equal(
                Box::new(ExecutableExpressionV1::Slot(12)),
                Box::new(ExecutableExpressionV1::Constant(number(0.0))),
            ),
        ]
    );

    let (authority, facts) =
        carrier_authority_for_plan(&package, &plan, BROWSER_GAMEPLAY_ALLOCATION_ROOT_TAG);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_GAMEPLAY_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &plan,
        facts.executable(),
    )
    .expect("automatic-contact allocation binds the exact package and CPP1 plan");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        plan,
        facts.executable(),
        allocation,
    )
    .expect("automatic-contact native session opens");
    let initial_world = session.world_base();
    session
        .apply_physical_input(&ExecutableInputSourceV1::Keyboard {
            code: b"KeyD".to_vec(),
            phase: ExecutableKeyPhaseV1::Down,
        })
        .expect("ordinary movement input advances without a contact trigger");
    let contact_step = session
        .apply_fixed_tick_and_emit_candidate(16)
        .expect("one fixed tick executes movement then Clause-owned contact");
    assert_eq!(contact_step.occurrence.entry, 5);
    assert!(!contact_step.rule_applied);
    let candidate = session
        .candidate()
        .expect("contact candidate lookup succeeds")
        .expect("the final chained entry emits one hidden candidate")
        .clone();
    assert_eq!(
        candidate.configuration[28],
        ExecutableValueV1::symbol(b"collected").expect("collected is a bounded symbol")
    );
    assert_eq!(candidate.configuration[0], number(0.08));
    assert_eq!(session.carrier().unwrap().candidate_delta_count(), 1);
    assert_eq!(session.world_base(), initial_world);
    assert!(session.last_admitted().is_none());
    assert_eq!(session.carrier().unwrap().decision_count(), 0);
    assert_eq!(session.carrier().unwrap().state_revision_count(), 1);

    let (successor, projection) = session
        .admit_candidate_with_projection(facts.admission_authorization())
        .expect("separate Admission alone installs the collected world state");
    assert_eq!(successor.predecessor, initial_world);
    assert_eq!(successor.configuration, candidate.configuration);
    assert_ne!(session.world_base(), initial_world);
    assert_eq!(session.carrier().unwrap().decision_count(), 1);
    assert_eq!(session.carrier().unwrap().state_revision_count(), 2);
    let projection = projection.expect("Admission emits the passive renderer projection");
    assert_eq!(projection.state, successor.id);
    let player = projected_object_field(&projection.term, b"player");
    let player_position = projected_object_field(player, b"position");
    assert_eq!(
        projected_number(projected_object_field(player_position, b"x")),
        0.08
    );
    let world = projected_object_field(&projection.term, b"world");
    let collectible = projected_array_first(projected_object_field(world, b"collectibles"));
    assert_eq!(
        projected_object_field(collectible, b"state")
            .as_atom()
            .expect("projected collectible state is a symbol Atom")
            .canonical_payload(),
        b"collected"
    );

    let away_package = checked_program_package_with_scopes_and_roles(1, Vec::new(), 25);
    let away_application = ApplicationId {
        snapshot: away_package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let away_plan = physical_plan_with_source(&away_package, &source);
    let (away_authority, away_facts) = carrier_authority_for_plan(
        &away_package,
        &away_plan,
        BROWSER_GAMEPLAY_CHANGED_ALLOCATION_ROOT_TAG,
    );
    let away_allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_GAMEPLAY_CHANGED_ALLOCATION_ROOT_TAG),
        &away_package,
        away_application,
        &away_plan,
        away_facts.executable(),
    )
    .expect("away-from-contact allocation binds the exact package and CPP1 plan");
    let mut away_session = PersistentProcessSessionV1::rematerialize(
        away_package,
        away_authority,
        away_application,
        away_plan,
        away_facts.executable(),
        away_allocation,
    )
    .expect("away-from-contact native session opens");
    let away_step = away_session
        .apply_fixed_tick_and_emit_candidate(16)
        .expect("away tick still completes the ordered movement/contact chain");
    assert_eq!(away_step.occurrence.entry, 5);
    assert!(!away_step.rule_applied);
    let away_candidate = away_session
        .candidate()
        .expect("away candidate lookup succeeds")
        .expect("the movement tick emits its sole final candidate");
    assert_eq!(
        away_candidate.configuration[28],
        ExecutableValueV1::symbol(b"active").expect("active is a bounded symbol")
    );
    assert_eq!(away_candidate.configuration[3].as_number(), Some(0.0));
    assert_eq!(away_candidate.configuration[5].as_boolean(), Some(true));
    assert_eq!(away_session.carrier().unwrap().candidate_delta_count(), 1);
    assert_eq!(away_session.carrier().unwrap().decision_count(), 0);
    assert_eq!(away_session.carrier().unwrap().state_revision_count(), 1);
    assert_eq!(package_id, session.package().unwrap());
}

#[test]
fn automatic_spring_contact_launches_and_source_only_strength_changes_the_visible_frame() {
    let source = gameplay_source();
    let package = checked_program_package_with_scopes_and_roles(1, Vec::new(), 25);
    let plan = physical_plan_with_source(&package, &source);
    assert_eq!(
        plan.input
            .as_ref()
            .expect("spring CPP1 has input")
            .tick
            .entries,
        [2, 3, 4, 5]
    );
    let launch = plan
        .program
        .rules
        .iter()
        .find(|rule| rule.entry == 4)
        .expect("source-owned launch velocity is the third ordered tick stage");
    assert_eq!(
        launch.assignments,
        [(3, ExecutableExpressionV1::Constant(number(12.0)))]
    );
    let airborne = plan
        .program
        .rules
        .iter()
        .find(|rule| rule.entry == 5)
        .expect("source-owned airborne state is the final ordered tick stage");
    assert_eq!(
        airborne.assignments,
        [(
            5,
            ExecutableExpressionV1::Constant(ExecutableValueV1::Boolean(false))
        )]
    );

    let (base_cpp1, base_projection) = admit_spring_journey(&source, 247, 248);
    let base_player = projected_object_field(&base_projection, b"player");
    let base_y = projected_number(projected_object_field(
        projected_object_field(base_player, b"position"),
        b"y",
    ));
    assert!(base_y > 0.0);

    let changed_source = std::str::from_utf8(&source)
        .expect("spring source is UTF-8")
        .replacen(
            "?player launch strength 12.0",
            "?player launch strength 16.0",
            1,
        );
    let (changed_cpp1, changed_projection) =
        admit_spring_journey(changed_source.as_bytes(), 250, 251);
    let changed_player = projected_object_field(&changed_projection, b"player");
    let changed_y = projected_number(projected_object_field(
        projected_object_field(changed_player, b"position"),
        b"y",
    ));
    assert_ne!(base_cpp1, changed_cpp1);
    assert!(changed_y > base_y);

    let base_cwr1 = encode_wasm_process_request_v1(&browser_gameplay_fixture_request(
        &source,
        BROWSER_GAMEPLAY_CHANGED_ALLOCATION_ROOT_TAG,
    ))
    .expect("base spring CWR1 encodes");
    let changed_cwr1 = encode_wasm_process_request_v1(&browser_gameplay_fixture_request(
        changed_source.as_bytes(),
        BROWSER_GAMEPLAY_CHANGED_ALLOCATION_ROOT_TAG,
    ))
    .expect("changed spring CWR1 encodes");
    assert_ne!(base_cwr1, changed_cwr1);
}

#[test]
fn canonical_source_ledger_deposit_reaches_persistent_admission_and_changes_balance() {
    let (base_plan, base_projection) =
        admit_source_scalar_with_projection(LEDGER, 244, 246, b"account", b"balance");
    let base_account = projected_object_field(&base_projection, b"account");
    assert_eq!(
        projected_number(projected_object_field(base_account, b"balance")),
        125.0
    );

    let changed_source = std::str::from_utf8(LEDGER)
        .expect("ledger source is UTF-8")
        .replacen(
            "?account balance ?balance + 25.0",
            "?account balance ?balance + 40.0",
            1,
        );
    let (changed_plan, changed_projection) = admit_source_scalar_with_projection(
        changed_source.as_bytes(),
        245,
        247,
        b"account",
        b"balance",
    );
    assert_ne!(changed_plan, base_plan);
    let changed_account = projected_object_field(&changed_projection, b"account");
    assert_eq!(
        projected_number(projected_object_field(changed_account, b"balance")),
        140.0
    );
}

#[test]
fn clause_only_air_momentum_reaches_admission_without_host_semantic_changes() {
    let source = std::str::from_utf8(WORLD).expect("arena source is UTF-8");
    let legacy = source
        .replace(
            "(?position-x + (?velocity-x + ?intent-x * ?move-speed * ?dt) * ?dt)",
            "(?position-x + ?intent-x * ?move-speed * ?dt)",
        )
        .replace(
            "(?position-z + (?velocity-z + ?intent-z * ?move-speed * ?dt) * ?dt)",
            "(?position-z + ?intent-z * ?move-speed * ?dt)",
        );
    let (legacy_plan, _) = admit_source_jump_then_tick(legacy.as_bytes(), 208, 209);
    let (momentum_plan, projection) = admit_source_jump_then_tick(WORLD, 211, 212);

    assert_ne!(legacy_plan, momentum_plan);
    let player = projected_object_field(&projection, b"player");
    let position = projected_object_field(player, b"position");
    let velocity = projected_object_field(player, b"velocity");
    assert_eq!(
        projected_number(projected_object_field(position, b"x")),
        0.3125
    );
    assert_eq!(
        projected_number(projected_object_field(position, b"y")),
        1.5
    );
    assert_eq!(
        projected_number(projected_object_field(velocity, b"x")),
        1.25
    );
    assert_eq!(
        projected_number(projected_object_field(velocity, b"y")),
        6.0
    );
    assert_eq!(
        projected_object_field(player, b"grounded")
            .as_atom()
            .expect("projected Boolean Atom")
            .canonical_payload(),
        [0]
    );
}

#[test]
fn clause_authored_dash_keeps_local_configuration_custody_until_admission() {
    let source = dash_gameplay_source();
    let package = checked_dash_gameplay_program_package(&source);
    let package_id = package.id();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let physical_plan = physical_plan_with_source(&package, &source);
    let (authority, facts) = carrier_authority_for_plan(
        &package,
        &physical_plan,
        BROWSER_GAMEPLAY_DASH_ALLOCATION_ROOT_TAG,
    );
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_GAMEPLAY_DASH_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &physical_plan,
        facts.executable(),
    )
    .expect("Clause dash allocation is bound to the checked package and CPP1 plan");
    let mut session = PersistentProcessSessionV1::rematerialize(
        package,
        authority,
        application,
        physical_plan,
        facts.executable(),
        allocation,
    )
    .expect("one persistent Clause-authored dash session opens");

    let initial_world = session.world_base();
    let initial_run = session.run().expect("the session owns one Run");
    let initial_activation = session
        .activation()
        .expect("the session owns one Activation");
    let initial_configuration = session
        .configuration_id()
        .expect("the Activation owns one initial Configuration");
    assert_eq!(
        session
            .carrier()
            .expect("the runtime carrier remains live")
            .state_revision_count(),
        1
    );

    let dash_step = session
        .apply_opaque_input(
            &encode_executable_occurrence_v1(&occurrence(1, &[])).expect("dash occurrence encodes"),
        )
        .expect("Clause-authored dash advances local configuration");
    assert_eq!(dash_step.before, initial_configuration);
    assert_ne!(dash_step.after, dash_step.before);
    assert_eq!(session.run().expect("Run remains live"), initial_run);
    assert_eq!(
        session.activation().expect("Activation remains live"),
        initial_activation
    );
    assert_eq!(session.world_base(), initial_world);
    assert_eq!(
        session.configuration().expect("dash configuration is live")[2].as_number(),
        Some(8.0)
    );
    assert_eq!(
        session.configuration().expect("dash configuration is live")[3].as_number(),
        Some(8.0)
    );
    {
        let carrier = session.carrier().expect("the runtime carrier remains live");
        let start = carrier
            .configuration(dash_step.before)
            .expect("the Activation start configuration is constituted");
        assert_eq!(start.activation, initial_activation);
        assert_eq!(
            start.predecessor,
            ConfigurationPredecessorV2::ActivationStart(initial_activation)
        );
        let after = carrier
            .configuration(dash_step.after)
            .expect("the dash successor configuration is constituted");
        assert_eq!(after.activation, initial_activation);
        assert_eq!(
            after.predecessor,
            ConfigurationPredecessorV2::ConfigurationAfter(StepRef {
                run: initial_run,
                activation: initial_activation,
                step: dash_step.id,
            })
        );
        assert_eq!(
            carrier
                .activation(initial_activation)
                .expect("the dash Activation is constituted")
                .latest_configuration(),
            dash_step.after
        );
        assert_eq!(carrier.state_revision_count(), 1);
        assert_eq!(carrier.decision_count(), 0);
    }

    let candidate_step = session
        .apply_opaque_input_and_emit_candidate(
            &encode_executable_occurrence_v1(&occurrence(2, &[0.016]))
                .expect("fixed dash tick encodes"),
        )
        .expect("the local dash tick emits one hidden candidate");
    let candidate = session
        .candidate()
        .expect("candidate lookup succeeds")
        .expect("the dash candidate remains retained")
        .clone();
    let candidate_configuration = session
        .configuration_id()
        .expect("the candidate retains its exact local Configuration");
    assert_eq!(candidate.produced_by, candidate_step.id);
    assert_eq!(candidate.base, initial_world);
    assert_eq!(candidate.configuration, session.configuration().unwrap());
    assert_eq!(session.run().unwrap(), initial_run);
    assert_eq!(session.activation().unwrap(), initial_activation);
    assert_eq!(session.world_base(), initial_world);
    {
        let carrier = session.carrier().expect("the runtime carrier remains live");
        let configuration = carrier
            .configuration(candidate_configuration)
            .expect("candidate configuration is constituted");
        assert_eq!(configuration.activation, initial_activation);
        assert_eq!(carrier.state_revision_count(), 1);
        assert_eq!(carrier.decision_count(), 0);
    }

    let held_configuration = session.configuration().unwrap().to_vec();
    let held_step_count = session.carrier().unwrap().step_count();
    let blocked = session
        .apply_opaque_input(
            &encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0]))
                .expect("post-candidate input encodes"),
        )
        .expect_err("candidate custody blocks further local mutation");
    assert!(matches!(
        blocked,
        PersistentProcessSessionErrorV1::Carrier(ExecutableCarrierErrorV1::Executable(
            ExecutableErrorV1::CandidateAlreadyEmitted
        ))
    ));
    assert_eq!(session.configuration_id().unwrap(), candidate_configuration);
    assert_eq!(session.configuration().unwrap(), held_configuration);
    assert_eq!(session.carrier().unwrap().step_count(), held_step_count);
    assert_eq!(session.world_base(), initial_world);

    assert_eq!(session.carrier().unwrap().state_revision_count(), 1);
    assert_eq!(session.carrier().unwrap().decision_count(), 0);
    let (successor, projection) = session
        .admit_constituted_candidate_with_projection()
        .expect("the admitted package's exact grant alone installs the dash successor");
    assert_eq!(successor.predecessor, initial_world);
    assert_eq!(successor.configuration, held_configuration);
    assert_eq!(session.world_base(), successor.id);
    assert_eq!(session.carrier().unwrap().state_revision_count(), 2);
    assert_eq!(session.carrier().unwrap().decision_count(), 1);
    let projection = projection.expect("Admission exposes the passive renderer projection");
    assert_eq!(projection.state, successor.id);
    let projected_player = projected_object_field(&projection.term, b"player");
    let projected_position = projected_object_field(projected_player, b"position");
    assert!(projected_number(projected_object_field(projected_position, b"x")) > 0.0);
    assert!(projected_number(projected_object_field(projected_position, b"y")) > 0.0);

    let successor_run = session.run().expect("Admission installs a successor Run");
    let successor_activation = session
        .activation()
        .expect("Admission installs a successor Activation identity");
    let successor_start_configuration = session
        .configuration_id()
        .expect("the successor Activation owns an initial Configuration identity");
    assert_ne!(successor_run, initial_run);
    assert_ne!(successor_activation, initial_activation);
    assert_ne!(successor_start_configuration, candidate_configuration);

    let local_input = session
        .apply_opaque_input(
            &encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0]))
                .expect("successor input encodes"),
        )
        .expect("successor Activation advances local intent");
    assert_eq!(local_input.before, successor_start_configuration);
    assert_eq!(session.run().unwrap(), successor_run);
    assert_eq!(session.activation().unwrap(), successor_activation);
    assert_eq!(session.world_base(), successor.id);
    assert_eq!(session.configuration().unwrap()[4].as_number(), Some(1.0));
    assert_eq!(successor.configuration[4].as_number(), Some(0.0));
    assert_eq!(session.carrier().unwrap().state_revision_count(), 2);
    assert!(session.candidate().unwrap().is_none());
    {
        let carrier = session.carrier().expect("the runtime carrier remains live");
        let start = carrier
            .configuration(local_input.before)
            .expect("the successor start Configuration is constituted on first use");
        assert_eq!(start.activation, successor_activation);
        assert_eq!(
            start.predecessor,
            ConfigurationPredecessorV2::ActivationStart(successor_activation)
        );
        let after = carrier
            .configuration(local_input.after)
            .expect("the successor local Configuration is constituted");
        assert_eq!(after.activation, successor_activation);
        assert_eq!(
            after.predecessor,
            ConfigurationPredecessorV2::ConfigurationAfter(StepRef {
                run: successor_run,
                activation: successor_activation,
                step: local_input.id,
            })
        );
    }
    assert_eq!(package_id, session.package().unwrap());
}

#[test]
fn package_owned_headless_api_reaches_one_admitted_render_state() {
    for checker_count in [0, 2] {
        let rejected_package = checked_program_package(checker_count);
        let rejected_application = ApplicationId {
            snapshot: rejected_package.constitution().snapshot(),
            local: ApplicationLocalId::new(1),
        };
        let rejected_plan = physical_plan(&rejected_package);
        let (rejected_authority, rejected_facts) = carrier_authority(&rejected_package);
        let mut rejected_runtime = ExecutableProcessRuntimeV1::instantiate_new(
            rejected_package,
            rejected_authority,
            rejected_application,
            rejected_plan,
            rejected_facts.executable(),
        )
        .expect("negative package still has a checked executable program");
        let error = rejected_runtime
            .start_carrier_process(rejected_facts.executable())
            .expect_err("missing or duplicate checker Mode fails closed");
        assert!(match checker_count {
            0 => matches!(error, ExecutableCarrierErrorV1::MissingCheckerMode),
            2 => matches!(error, ExecutableCarrierErrorV1::AmbiguousCheckerMode),
            _ => unreachable!(),
        });
        assert_eq!(rejected_runtime.carrier().carrier().activation_count(), 0);
        assert_eq!(rejected_runtime.carrier().carrier().observation_count(), 0);
        assert_eq!(
            rejected_runtime.carrier().carrier().candidate_delta_count(),
            0
        );
    }

    let package = checked_program_package(1);
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let physical_plan = physical_plan(&package);
    let (authority, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_RECORDED_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &physical_plan,
        facts.executable(),
    )
    .expect("fixture native run rematerializes one recorded occurrence");
    let mut runtime = ExecutableProcessRuntimeV1::instantiate_rematerialized(
        package,
        authority,
        application,
        physical_plan,
        facts.executable(),
        allocation,
    )
    .expect("checked package executable instantiates");
    runtime
        .start_carrier_process(facts.executable())
        .expect("production bridge synthesizes the Activation");

    runtime
        .advance_carrier_occurrence(occurrence(0, &[1.0, 0.0]))
        .expect("opaque input enters with its computed Step");
    runtime
        .advance_carrier_occurrence(occurrence(2, &[0.25]))
        .expect("ground Step enters");
    assert_eq!(value(runtime.configuration(), 0), 1.25);
    assert_eq!(value(runtime.configuration(), 2), 5.0);

    runtime
        .advance_carrier_occurrence(occurrence(1, &[]))
        .expect("grounded impulse enters");
    assert_eq!(value(runtime.configuration(), 3), 8.0);
    assert_eq!(runtime.configuration()[5].as_boolean(), Some(false));
    let before_rejected = runtime.configuration().to_vec();
    let rejected = runtime
        .advance_carrier_occurrence(occurrence(1, &[]))
        .expect("unmatched occurrence still advances Configuration custody");
    assert!(!rejected.rule_applied);
    assert_eq!(runtime.configuration(), before_rejected);

    for _ in 0..6 {
        runtime
            .advance_carrier_occurrence(occurrence(2, &[0.25]))
            .expect("airborne Step enters");
    }
    runtime
        .advance_carrier_occurrence_and_emit_candidate(occurrence(2, &[0.25]))
        .expect("landing Step emits the one candidate");
    assert_eq!(value(runtime.configuration(), 1), 0.0);
    assert_eq!(value(runtime.configuration(), 3), 0.0);
    assert_eq!(runtime.configuration()[5].as_boolean(), Some(true));
    let candidate = runtime.candidate().expect("candidate is retained").clone();
    assert_eq!(candidate.base, facts.initial_state);
    assert!(runtime.judgment().is_none());
    assert!(runtime.admission().is_none());
    assert!(
        runtime
            .carrier()
            .carrier()
            .candidate_delta(candidate.id)
            .is_some()
    );
    assert_eq!(runtime.carrier().carrier().state_revision_count(), 1);

    let successor = runtime
        .settle_carrier_process(facts.admission_authorization())
        .expect("production bridge synthesizes Judgment and Admission")
        .clone();
    assert_eq!(successor.predecessor, facts.initial_state);
    assert_ne!(successor.id, facts.initial_state);
    let observation = runtime
        .observe_carrier_state(&[0, 1, 3, 5])
        .expect("production bridge synthesizes the admitted render Observation");
    assert_eq!(observation.state, successor.id);
    assert_eq!(observation.value[0].as_number(), Some(10.0));
    assert_eq!(observation.value[1].as_number(), Some(0.0));
    assert_eq!(observation.value[2].as_number(), Some(0.0));
    assert_eq!(observation.value[3].as_boolean(), Some(true));
    let repeated_observation = runtime
        .observe_carrier_state(&[0, 1, 3, 5])
        .expect("repeated State projection receives a fresh occurrence identity");
    assert_ne!(repeated_observation.id, observation.id);
    assert_eq!(repeated_observation.state, observation.state);
    assert_eq!(repeated_observation.value, observation.value);

    assert_eq!(runtime.carrier().carrier().candidate_delta_count(), 1);
    assert_eq!(runtime.carrier().carrier().decision_count(), 1);
    assert_eq!(runtime.carrier().carrier().state_revision_count(), 2);
    assert!(
        runtime
            .carrier()
            .carrier()
            .observation(observation.id)
            .is_some()
    );
    assert!(
        runtime
            .carrier()
            .carrier()
            .observation(repeated_observation.id)
            .is_some()
    );
}

#[test]
fn bounded_wasm_bytes_return_only_the_admitted_observation() {
    let package = checked_program_package(1);
    let physical_plan = physical_plan(&package);
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let (_, allocation_facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_RECORDED_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &physical_plan,
        allocation_facts.executable(),
    )
    .expect("bounded request records one exact allocation epoch");
    let request = WasmProcessRequestV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: ApplicationLocalId::new(1),
        physical_plan_bytes: encode_executable_physical_plan_v1(&physical_plan)
            .expect("physical plan encodes outside the semantic package"),
        allocation,
        authority: WasmAuthorityInputV1 {
            program: id!(ProgramId, 123),
            change: id!(ProgramChangeOccurrenceId, 124),
            session: id!(RuntimeSessionId, 120),
            policy: id!(RuntimePolicyId, 121),
            session_start: id!(SessionStartOccurrenceId, 122),
            root_policy: id!(RootPolicyId, 125),
            occurrence_boundary: id!(BoundaryRef, 126),
            state_boundary: id!(BoundaryRef, 127),
            occurrence_evidence: id!(ExternalEvidenceRef, 181),
            occurrence_evidence_bytes: vec![181],
            judgment_evidence: id!(ExternalEvidenceRef, 186),
            judgment_evidence_bytes: vec![186],
            admission_evidence: id!(ExternalEvidenceRef, 190),
            admission_evidence_bytes: vec![190],
            budget_units: 100,
        },
        occurrences: vec![
            encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0]))
                .expect("opaque occurrence encodes"),
        ],
        render_slots: vec![4],
    };
    let exact_request = encode_wasm_process_request_v1(&request).expect("bounded request encodes");
    assert_eq!(
        decode_wasm_process_request_v1(&exact_request).expect("exact request decodes"),
        request
    );

    let mut boundary = WasmProcessBuffersV1::new();
    for byte in exact_request {
        boundary
            .push_request_byte(byte)
            .expect("request remains in the fixed bound");
    }
    boundary
        .dispatch()
        .expect("production ProcessCarrier admits the run");
    assert_eq!(boundary.status(), WasmProcessStatusV1::Ready);
    let output = decode_wasm_process_observation_v1(boundary.response())
        .expect("boundary returns one exact admitted Observation");
    assert!(output.observation.as_bytes().iter().any(|byte| *byte != 0));
    assert_ne!(output.state, id!(StateRevisionId, 120));
    assert_eq!(
        output.exact_value_bytes,
        [1_u8, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63]
    );

    let oversized = vec![0; WASM_PROCESS_REQUEST_LIMIT_V1 + 1];
    assert_eq!(
        decode_wasm_process_request_v1(&oversized),
        Err(WasmProcessStatusV1::RequestOutOfBounds)
    );
    boundary.reset();
    for byte in b"bad!" {
        boundary.push_request_byte(*byte).unwrap();
    }
    assert_eq!(
        boundary.dispatch(),
        Err(WasmProcessStatusV1::MalformedRequest)
    );
    assert_eq!(boundary.response(), &[]);
}

#[test]
fn persistent_wasm_session_keeps_generation_sequence_and_admission_custody() {
    let package = checked_program_package(1);
    let package_id = package.id();
    let physical_plan = physical_plan(&package);
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let (_, allocation_facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_RECORDED_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &physical_plan,
        allocation_facts.executable(),
    )
    .expect("tracked fixture allocation binds the final package and plan");
    let session = id!(RuntimeSessionId, 120);
    let open = WasmSessionOpenV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: ApplicationLocalId::new(1),
        physical_plan_bytes: encode_executable_physical_plan_v1(&physical_plan)
            .expect("physical plan encodes beside the package"),
        authority: WasmAuthorityInputV1 {
            program: id!(ProgramId, 123),
            change: id!(ProgramChangeOccurrenceId, 124),
            session,
            policy: id!(RuntimePolicyId, 121),
            session_start: id!(SessionStartOccurrenceId, 122),
            root_policy: id!(RootPolicyId, 125),
            occurrence_boundary: id!(BoundaryRef, 126),
            state_boundary: id!(BoundaryRef, 127),
            occurrence_evidence: id!(ExternalEvidenceRef, 181),
            occurrence_evidence_bytes: vec![181],
            judgment_evidence: id!(ExternalEvidenceRef, 186),
            judgment_evidence_bytes: vec![186],
            admission_evidence: id!(ExternalEvidenceRef, 190),
            admission_evidence_bytes: vec![190],
            budget_units: 100,
        },
        allocation: WasmSessionAllocationV1::Rematerialize(allocation),
        limits: WasmSessionLimitsV1 {
            max_commands: 16,
            command_bytes: 4096,
            event_bytes: WASM_SESSION_EVENT_LIMIT_V1 as u32,
        },
    };
    let exact_open = encode_wasm_session_open_v1(&open).expect("bounded CWS1 open encodes");
    assert_eq!(
        decode_wasm_session_open_v1(&exact_open).expect("exact CWS1 open decodes"),
        open
    );
    let mut boundary = WasmPersistentSessionBoundaryV1::new();
    let opened = boundary
        .open(&exact_open)
        .expect("one persistent slot opens");
    let handle = opened.handle;
    let (initial_world, initial_run, initial_activation) = match opened.kind {
        WasmSessionEventKindV1::Opened {
            package: opened_package,
            session: actual_session,
            world,
            run,
            activation,
            allocation: opened_allocation,
            state_revision_count,
        } => {
            assert_eq!(opened_package, package_id);
            assert_eq!(actual_session, session);
            assert_eq!(opened_allocation, allocation);
            assert_eq!(state_revision_count, 1);
            (world, run, activation)
        }
        other => panic!("unexpected open event: {other:?}"),
    };

    let command = |expected_sequence, operation| WasmSessionCommandV1 {
        handle,
        expected_sequence,
        operation,
    };
    let apply = |boundary: &mut WasmPersistentSessionBoundaryV1, command: WasmSessionCommandV1| {
        let bytes = encode_wasm_session_command_v1(&command).expect("bounded CWI1 command encodes");
        let decoded = decode_wasm_session_command_v1(&bytes).expect("exact CWI1 command decodes");
        assert_eq!(decoded, command);
        let event = boundary.command(&bytes).expect("valid command transports");
        let event_bytes = encode_wasm_session_event_v1(&event);
        assert_eq!(
            decode_wasm_session_event_v1(&event_bytes).expect("exact CSE1 event decodes"),
            event
        );
        event
    };

    let input_one = apply(
        &mut boundary,
        command(
            0,
            WasmSessionOperationV1::Input(
                encode_executable_occurrence_v1(&occurrence(0, &[1.0, 0.0])).unwrap(),
            ),
        ),
    );
    assert_eq!(input_one.accepted_sequence, 1);
    let input_two = apply(
        &mut boundary,
        command(
            1,
            WasmSessionOperationV1::Input(
                encode_executable_occurrence_v1(&occurrence(2, &[0.25])).unwrap(),
            ),
        ),
    );
    assert_eq!(input_two.accepted_sequence, 2);
    let candidate_event = apply(
        &mut boundary,
        command(
            2,
            WasmSessionOperationV1::Candidate(
                encode_executable_occurrence_v1(&occurrence(2, &[0.25])).unwrap(),
            ),
        ),
    );
    let candidate = match candidate_event.kind {
        WasmSessionEventKindV1::CandidateAccepted {
            candidate,
            base,
            run,
            activation,
            state_revision_count,
            ..
        } => {
            assert_eq!(base, initial_world);
            assert_eq!(run, initial_run);
            assert_eq!(activation, initial_activation);
            assert_eq!(state_revision_count, 1);
            candidate
        }
        other => panic!("unexpected candidate event: {other:?}"),
    };

    let duplicate = encode_wasm_session_command_v1(&command(
        2,
        WasmSessionOperationV1::Input(
            encode_executable_occurrence_v1(&occurrence(2, &[0.25])).unwrap(),
        ),
    ))
    .unwrap();
    assert_eq!(
        boundary.command(&duplicate),
        Err(WasmProcessStatusV1::SequenceRejected)
    );
    let stale = encode_wasm_session_command_v1(&WasmSessionCommandV1 {
        handle: WasmSessionHandleV1 {
            slot: handle.slot,
            generation: handle.generation + 1,
        },
        expected_sequence: 3,
        operation: WasmSessionOperationV1::Dispose,
    })
    .unwrap();
    assert_eq!(
        boundary.command(&stale),
        Err(WasmProcessStatusV1::StaleSessionHandle)
    );

    let missing_issuance = apply(
        &mut boundary,
        command(
            3,
            WasmSessionOperationV1::Admit(WasmSessionAdmissionV1 {
                package: package_id,
                session,
                base: initial_world,
                candidate,
                authorization: id!(IssuedAdmissionAuthorizationOccurrenceId, 240),
            }),
        ),
    );
    assert!(matches!(
        missing_issuance.kind,
        WasmSessionEventKindV1::Rejected(WasmSessionRejectionV1::AdmissionRejected)
    ));
    for (expected_sequence, scope) in [
        (
            4,
            WasmSessionAdmissionScopeV1 {
                package: id!(ProcessPackageId, 241),
                session,
                base: initial_world,
                candidate,
            },
        ),
        (
            5,
            WasmSessionAdmissionScopeV1 {
                package: package_id,
                session: id!(RuntimeSessionId, 242),
                base: initial_world,
                candidate,
            },
        ),
        (
            6,
            WasmSessionAdmissionScopeV1 {
                package: package_id,
                session,
                base: id!(StateRevisionId, 243),
                candidate,
            },
        ),
        (
            7,
            WasmSessionAdmissionScopeV1 {
                package: package_id,
                session,
                base: initial_world,
                candidate: id!(CandidateDeltaId, 244),
            },
        ),
    ] {
        let rejected = apply(
            &mut boundary,
            command(
                expected_sequence,
                WasmSessionOperationV1::IssueAdmission(scope),
            ),
        );
        assert!(matches!(
            rejected.kind,
            WasmSessionEventKindV1::Rejected(WasmSessionRejectionV1::AdmissionScopeRejected)
        ));
    }

    let issued = apply(
        &mut boundary,
        command(
            8,
            WasmSessionOperationV1::IssueAdmission(WasmSessionAdmissionScopeV1 {
                package: package_id,
                session,
                base: initial_world,
                candidate,
            }),
        ),
    );
    let authorization = match issued.kind {
        WasmSessionEventKindV1::AdmissionAuthorizationIssued {
            occurrence,
            package,
            session: issued_session,
            base,
            candidate: issued_candidate,
            state_revision_count,
        } => {
            assert_eq!(package, package_id);
            assert_eq!(issued_session, session);
            assert_eq!(base, initial_world);
            assert_eq!(issued_candidate, candidate);
            assert_eq!(state_revision_count, 1);
            occurrence
        }
        other => panic!("unexpected Admission authorization event: {other:?}"),
    };
    let duplicate_issuance = apply(
        &mut boundary,
        command(
            9,
            WasmSessionOperationV1::IssueAdmission(WasmSessionAdmissionScopeV1 {
                package: package_id,
                session,
                base: initial_world,
                candidate,
            }),
        ),
    );
    assert!(matches!(
        duplicate_issuance.kind,
        WasmSessionEventKindV1::Rejected(WasmSessionRejectionV1::AuthorityRejected)
    ));
    let admitted = apply(
        &mut boundary,
        command(
            10,
            WasmSessionOperationV1::Admit(WasmSessionAdmissionV1 {
                package: package_id,
                session,
                base: initial_world,
                candidate,
                authorization,
            }),
        ),
    );
    match admitted.kind {
        WasmSessionEventKindV1::AdmissionAccepted {
            predecessor,
            successor,
            run,
            activation,
            session: admitted_session,
            state_revision_count,
            projection,
        } => {
            assert_eq!(predecessor, initial_world);
            assert_ne!(successor, initial_world);
            assert_ne!(run, initial_run);
            assert_ne!(activation, initial_activation);
            assert_eq!(admitted_session, session);
            assert_eq!(state_revision_count, 2);
            let projection = projection.expect("package projection is transported only now");
            let term = decode_canonical_term_bytes(&projection.exact_term_bytes)
                .expect("projected Term remains exact and canonical");
            assert_arena_projection(&term, 2.5, 5.0);
        }
        other => panic!("unexpected Admission event: {other:?}"),
    }

    let disposed = apply(&mut boundary, command(11, WasmSessionOperationV1::Dispose));
    assert!(matches!(disposed.kind, WasmSessionEventKindV1::Disposed));
    let post_dispose =
        encode_wasm_session_command_v1(&command(12, WasmSessionOperationV1::Dispose)).unwrap();
    assert_eq!(
        boundary.command(&post_dispose),
        Err(WasmProcessStatusV1::StaleSessionHandle)
    );

    let reopened = boundary
        .open(&exact_open)
        .expect("slot reuse publishes a fresh generation");
    assert_eq!(reopened.handle.slot, handle.slot);
    assert_eq!(reopened.handle.generation, handle.generation + 1);
    assert_eq!(
        boundary.command(&post_dispose),
        Err(WasmProcessStatusV1::StaleSessionHandle)
    );
}

#[test]
fn persistent_wasm_open_replaces_only_after_the_replacement_is_checked() {
    let request = browser_fixture_request();
    let open = |request: &WasmProcessRequestV1| WasmSessionOpenV1 {
        package_bytes: request.package_bytes.clone(),
        application: request.application,
        physical_plan_bytes: request.physical_plan_bytes.clone(),
        authority: request.authority.clone(),
        allocation: WasmSessionAllocationV1::Rematerialize(request.allocation),
        limits: WasmSessionLimitsV1 {
            max_commands: 16,
            command_bytes: 4096,
            event_bytes: WASM_SESSION_EVENT_LIMIT_V1 as u32,
        },
    };
    let mut boundary = WasmPersistentSessionBoundaryV1::new();
    let exact_open = encode_wasm_session_open_v1(&open(&request)).expect("CWS1 open encodes");
    let first = boundary.open(&exact_open).expect("first session opens");

    let mut rejected_request = request.clone();
    rejected_request.physical_plan_bytes = vec![0];
    let rejected_open =
        encode_wasm_session_open_v1(&open(&rejected_request)).expect("rejected CWS1 open encodes");
    assert_eq!(
        boundary.open(&rejected_open),
        Err(WasmProcessStatusV1::ProcessRejected)
    );
    let retained_command = encode_wasm_session_command_v1(&WasmSessionCommandV1 {
        handle: first.handle,
        expected_sequence: 1,
        operation: WasmSessionOperationV1::Dispose,
    })
    .expect("retained-session probe encodes");
    assert_eq!(
        boundary.command(&retained_command),
        Err(WasmProcessStatusV1::SequenceRejected)
    );

    let second = boundary
        .open(&exact_open)
        .expect("checked replacement opens atomically");
    assert_eq!(second.handle.slot, first.handle.slot);
    assert_eq!(second.handle.generation, first.handle.generation + 1);
    let stale_command = encode_wasm_session_command_v1(&WasmSessionCommandV1 {
        handle: first.handle,
        expected_sequence: 0,
        operation: WasmSessionOperationV1::Dispose,
    })
    .expect("stale-session probe encodes");
    assert_eq!(
        boundary.command(&stale_command),
        Err(WasmProcessStatusV1::StaleSessionHandle)
    );
}

#[test]
fn shipped_cwr1_has_external_physical_plan_and_successive_issued_admission() {
    let request = browser_fixture_request();
    let checked = check_process_package(
        decode_process_package(&request.package_bytes).expect("fixture package decodes"),
    )
    .expect("fixture package checks");
    let changed_source = std::str::from_utf8(WORLD)
        .expect("arena source is UTF-8")
        .replacen(
            "include\n    ?player horizontal intent Vec3 { x: ?intent-x, y: 0.0, z: ?intent-z }",
            "include\n    ?player horizontal intent Vec3 { x: 0.5, y: 0.0, z: ?intent-z }",
            1,
        );
    let changed_plan = physical_plan_with_source(&checked, changed_source.as_bytes());
    assert_ne!(
        encode_executable_physical_plan_v1(&changed_plan)
            .expect("changed source plan remains physical"),
        request.physical_plan_bytes,
        "a source-only handler change changes CPP1 without a Rust semantic edit"
    );
    assert_eq!(
        changed_plan.program.rules[0].assignments[0],
        (4, ExecutableExpressionV1::Constant(number(0.5)),)
    );
    let exact = encode_wasm_process_request_v1(&request).expect("browser CWR1 fixture encodes");
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../browser/jump-arena-shell/fixtures/wasm-jump-v1/jump-v1.cwr1.hex"
    );
    if std::env::var_os("CLAUSE_UPDATE_BROWSER_CWR1").is_some() {
        std::fs::write(fixture_path, lowercase_hex_lines(&exact))
            .expect("browser CWR1 fixture update succeeds");
        return;
    }
    let tracked = decode_hex(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../browser/jump-arena-shell/fixtures/wasm-jump-v1/jump-v1.cwr1.hex"
    )));
    assert_eq!(tracked, exact);
    assert_eq!(
        decode_wasm_process_request_v1(&tracked).expect("tracked CWR1 decodes"),
        request
    );

    let package = checked;
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: request.application,
    };
    let physical_plan = decode_executable_physical_plan_v1(&request.physical_plan_bytes)
        .expect("fixture physical plan decodes independently");
    let (_, allocation_facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_RECORDED_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &physical_plan,
        allocation_facts.executable(),
    )
    .expect("tracked fixture allocation binds the final package and plan");
    let open = WasmSessionOpenV1 {
        package_bytes: request.package_bytes.clone(),
        application: request.application,
        physical_plan_bytes: request.physical_plan_bytes.clone(),
        authority: request.authority.clone(),
        allocation: WasmSessionAllocationV1::Rematerialize(allocation),
        limits: WasmSessionLimitsV1 {
            max_commands: 16,
            command_bytes: 4096,
            event_bytes: WASM_SESSION_EVENT_LIMIT_V1 as u32,
        },
    };
    let mut boundary = WasmPersistentSessionBoundaryV1::new();
    let opened = boundary
        .open(&encode_wasm_session_open_v1(&open).expect("fixture CWS1 encodes"))
        .expect("tracked fixture opens");
    let handle = opened.handle;
    let package = match opened.kind {
        WasmSessionEventKindV1::Opened { package, .. } => package,
        other => panic!("unexpected fixture open: {other:?}"),
    };
    let mut sequence = 0;
    for ordinal in 0..2 {
        let mut candidate_scope = None;
        for (occurrence_index, occurrence) in request.occurrences.iter().enumerate() {
            let candidate_command = occurrence_index + 1 == request.occurrences.len();
            let operation = if candidate_command {
                WasmSessionOperationV1::Candidate(occurrence.clone())
            } else {
                WasmSessionOperationV1::Input(occurrence.clone())
            };
            let event = boundary
                .command(
                    &encode_wasm_session_command_v1(&WasmSessionCommandV1 {
                        handle,
                        expected_sequence: sequence,
                        operation,
                    })
                    .expect("fixture occurrence command encodes"),
                )
                .expect("fixture occurrence command transports");
            sequence += 1;
            assert_eq!(event.accepted_sequence, sequence);
            match event.kind {
                WasmSessionEventKindV1::InputAccepted { .. } if !candidate_command => {}
                WasmSessionEventKindV1::CandidateAccepted {
                    candidate, base, ..
                } if candidate_command => candidate_scope = Some((candidate, base)),
                other => panic!("unexpected fixture occurrence event: {other:?}"),
            }
        }
        let (candidate, base) = candidate_scope.expect("fixture tick emits one candidate");
        let issued = boundary
            .command(
                &encode_wasm_session_command_v1(&WasmSessionCommandV1 {
                    handle,
                    expected_sequence: sequence,
                    operation: WasmSessionOperationV1::IssueAdmission(
                        WasmSessionAdmissionScopeV1 {
                            package,
                            session: request.authority.session,
                            base,
                            candidate,
                        },
                    ),
                })
                .expect("fixture Admission issuance command encodes"),
            )
            .expect("fixture Admission issuance command transports");
        sequence += 1;
        assert_eq!(issued.accepted_sequence, sequence);
        let authorization = match issued.kind {
            WasmSessionEventKindV1::AdmissionAuthorizationIssued {
                occurrence,
                package: issued_package,
                session,
                base: issued_base,
                candidate: issued_candidate,
                ..
            } => {
                assert_eq!(issued_package, package);
                assert_eq!(session, request.authority.session);
                assert_eq!(issued_base, base);
                assert_eq!(issued_candidate, candidate);
                occurrence
            }
            other => panic!("unexpected fixture Admission issuance event: {other:?}"),
        };
        let event = boundary
            .command(
                &encode_wasm_session_command_v1(&WasmSessionCommandV1 {
                    handle,
                    expected_sequence: sequence,
                    operation: WasmSessionOperationV1::Admit(WasmSessionAdmissionV1 {
                        package,
                        session: request.authority.session,
                        base,
                        candidate,
                        authorization,
                    }),
                })
                .expect("fixture Admission command encodes"),
            )
            .expect("fixture Admission command transports");
        let exact_event = encode_wasm_session_event_v1(&event);
        assert!(exact_event.len() <= WASM_SESSION_EVENT_LIMIT_V1);
        assert_eq!(
            decode_wasm_session_event_v1(&exact_event).expect("fixture Admission CSE1 decodes"),
            event
        );
        sequence += 1;
        assert_eq!(event.accepted_sequence, sequence);
        let (successor, projection) = match event.kind {
            WasmSessionEventKindV1::AdmissionAccepted {
                predecessor,
                successor,
                state_revision_count,
                projection: Some(projection),
                ..
            } => {
                assert_eq!(predecessor, base);
                assert_eq!(state_revision_count, ordinal + 2);
                (successor, projection)
            }
            other => panic!("unexpected fixture Admission event: {other:?}"),
        };
        assert_ne!(successor, base);
        let term = decode_canonical_term_bytes(&projection.exact_term_bytes)
            .expect("fixture projection remains canonical");
        assert_arena_projection(&term, 1.25 * (ordinal as f64 + 1.0), 5.0);
    }
}

#[test]
fn shipped_collect_cwr1_preserves_clause_owned_scalar_behavior() {
    let changed_source = std::str::from_utf8(COLLECT)
        .expect("collect source is UTF-8")
        .replacen(
            "?player score ?score + 1.0",
            "?player score ?score + 4.0",
            1,
        );
    let base = browser_scalar_fixture_request(COLLECT, BROWSER_COLLECT_ALLOCATION_ROOT_TAG);
    let changed = browser_scalar_fixture_request(
        changed_source.as_bytes(),
        BROWSER_COLLECT_CHANGED_ALLOCATION_ROOT_TAG,
    );
    let base_plan = decode_executable_physical_plan_v1(&base.physical_plan_bytes)
        .expect("base collect CPP1 decodes");
    let changed_plan = decode_executable_physical_plan_v1(&changed.physical_plan_bytes)
        .expect("changed collect CPP1 decodes");
    assert_eq!(
        base_plan.program.rules[0].assignments[0].1,
        ExecutableExpressionV1::Add(
            Box::new(ExecutableExpressionV1::Slot(0)),
            Box::new(ExecutableExpressionV1::Constant(number(1.0))),
        )
    );
    assert_eq!(
        changed_plan.program.rules[0].assignments[0].1,
        ExecutableExpressionV1::Add(
            Box::new(ExecutableExpressionV1::Slot(0)),
            Box::new(ExecutableExpressionV1::Constant(number(4.0))),
        )
    );
    assert_ne!(base_plan.program, changed_plan.program);

    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../browser/jump-arena-shell/fixtures/wasm-collect-v1");
    let fixtures = [
        ("collect-plus-1.cwr1.hex", base),
        ("collect-plus-4.cwr1.hex", changed),
    ];
    if std::env::var_os("CLAUSE_UPDATE_BROWSER_COLLECT_CWR1").is_some() {
        std::fs::create_dir_all(&fixture_root)
            .expect("collect browser fixture directory is created");
        for (name, request) in fixtures {
            let exact = encode_wasm_process_request_v1(&request)
                .expect("collect browser CWR1 fixture encodes");
            std::fs::write(fixture_root.join(name), lowercase_hex_lines(&exact))
                .expect("collect browser CWR1 fixture update succeeds");
        }
        return;
    }
    for (name, request) in fixtures {
        let exact =
            encode_wasm_process_request_v1(&request).expect("collect browser CWR1 fixture encodes");
        let tracked = std::fs::read_to_string(fixture_root.join(name))
            .expect("tracked collect browser CWR1 fixture exists");
        assert_eq!(decode_hex(&tracked), exact);
        assert_eq!(
            decode_wasm_process_request_v1(&exact).expect("tracked collect CWR1 decodes"),
            request
        );
    }
}

#[test]
fn shipped_symbolic_collect_cwr1_preserves_clause_owned_state() {
    let changed_source = std::str::from_utf8(COLLECT_STATE)
        .expect("collect-state source is UTF-8")
        .replacen(
            "?collectible state collected",
            "?collectible state spent",
            1,
        );
    let base = browser_scalar_fixture_request_with_projection(
        COLLECT_STATE,
        BROWSER_SYMBOLIC_COLLECT_ALLOCATION_ROOT_TAG,
        b"collectible",
        b"state",
    );
    let changed = browser_scalar_fixture_request_with_projection(
        changed_source.as_bytes(),
        BROWSER_SYMBOLIC_COLLECT_CHANGED_ALLOCATION_ROOT_TAG,
        b"collectible",
        b"state",
    );
    let base_plan = decode_executable_physical_plan_v1(&base.physical_plan_bytes)
        .expect("base symbolic collect CPP1 decodes");
    let changed_plan = decode_executable_physical_plan_v1(&changed.physical_plan_bytes)
        .expect("changed symbolic collect CPP1 decodes");
    assert_eq!(
        base_plan.program.rules[0].assignments[0].1,
        ExecutableExpressionV1::Constant(
            ExecutableValueV1::symbol(b"collected").expect("symbol is bounded")
        )
    );
    assert_eq!(
        changed_plan.program.rules[0].assignments[0].1,
        ExecutableExpressionV1::Constant(
            ExecutableValueV1::symbol(b"spent").expect("symbol is bounded")
        )
    );
    assert_ne!(base_plan.program, changed_plan.program);

    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../browser/jump-arena-shell/fixtures/wasm-collect-state-v1");
    let fixtures = [("collected.cwr1.hex", base), ("spent.cwr1.hex", changed)];
    if std::env::var_os("CLAUSE_UPDATE_BROWSER_SYMBOLIC_COLLECT_CWR1").is_some() {
        std::fs::create_dir_all(&fixture_root)
            .expect("symbolic collect browser fixture directory is created");
        for (name, request) in fixtures {
            let exact = encode_wasm_process_request_v1(&request)
                .expect("symbolic collect browser CWR1 fixture encodes");
            std::fs::write(fixture_root.join(name), lowercase_hex_lines(&exact))
                .expect("symbolic collect browser CWR1 fixture update succeeds");
        }
        return;
    }
    for (name, request) in fixtures {
        let exact = encode_wasm_process_request_v1(&request)
            .expect("symbolic collect browser CWR1 fixture encodes");
        let tracked = std::fs::read_to_string(fixture_root.join(name))
            .expect("tracked symbolic collect browser CWR1 fixture exists");
        assert_eq!(decode_hex(&tracked), exact);
        assert_eq!(
            decode_wasm_process_request_v1(&exact).expect("tracked symbolic collect CWR1 decodes"),
            request
        );
    }
}

#[test]
fn shipped_unified_gameplay_cwr1_carries_arena_and_symbolic_collect() {
    let source = gameplay_source();
    let dash_source = dash_gameplay_source();
    let changed_source = std::str::from_utf8(&source)
        .expect("gameplay source is UTF-8")
        .replacen(
            "?collectible state collected",
            "?collectible state spent",
            1,
        )
        .into_bytes();
    assert_ne!(source, changed_source);
    let request = browser_gameplay_fixture_request(&source, BROWSER_GAMEPLAY_ALLOCATION_ROOT_TAG);
    let changed_request = browser_gameplay_fixture_request(
        &changed_source,
        BROWSER_GAMEPLAY_CHANGED_ALLOCATION_ROOT_TAG,
    );
    let dash_request = browser_dash_gameplay_fixture_request(&dash_source);
    let plan = decode_executable_physical_plan_v1(&request.physical_plan_bytes)
        .expect("unified gameplay CPP1 decodes");
    let changed_plan = decode_executable_physical_plan_v1(&changed_request.physical_plan_bytes)
        .expect("changed unified gameplay CPP1 decodes");
    let dash_plan = decode_executable_physical_plan_v1(&dash_request.physical_plan_bytes)
        .expect("dash-jump unified gameplay CPP1 decodes");
    assert!(plan.program.rules.iter().any(|rule| rule.entry == 0));
    assert!(plan.program.rules.iter().any(|rule| rule.entry == 1));
    assert!(plan.program.rules.iter().any(|rule| rule.entry == 2));
    assert_eq!(
        plan.program
            .rules
            .iter()
            .find(|rule| rule.entry == 3)
            .expect("unified gameplay carries the collect transition")
            .assignments[0]
            .1,
        ExecutableExpressionV1::Constant(
            ExecutableValueV1::symbol(b"collected").expect("symbol is bounded")
        )
    );
    assert_eq!(
        changed_plan
            .program
            .rules
            .iter()
            .find(|rule| rule.entry == 3)
            .expect("changed unified gameplay carries the collect transition")
            .assignments[0]
            .1,
        ExecutableExpressionV1::Constant(
            ExecutableValueV1::symbol(b"spent").expect("symbol is bounded")
        )
    );
    assert_ne!(plan.program, changed_plan.program);
    let dash_rule = dash_plan
        .program
        .rules
        .iter()
        .find(|rule| rule.entry == 1)
        .expect("dash-jump gameplay carries the source-owned jump transition");
    assert_eq!(dash_rule.assignments[0].1, ExecutableExpressionV1::Slot(7));
    assert_eq!(dash_rule.assignments[1].1, ExecutableExpressionV1::Slot(7));
    assert_ne!(plan.program, dash_plan.program);
    assert_eq!(
        plan.program.initial_configuration[28],
        ExecutableValueV1::symbol(b"active").expect("symbol is bounded")
    );
    assert_eq!(
        changed_plan.program.initial_configuration[28],
        ExecutableValueV1::symbol(b"active").expect("symbol is bounded")
    );

    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../browser/jump-arena-shell/fixtures/wasm-gameplay-v1");
    let fixtures = [
        ("gameplay-v1.cwr1.hex", request),
        ("gameplay-spent-v1.cwr1.hex", changed_request),
        ("gameplay-dash-jump-v1.cwr1.hex", dash_request),
    ];
    if std::env::var_os("CLAUSE_UPDATE_BROWSER_GAMEPLAY_CWR1").is_some() {
        std::fs::create_dir_all(&fixture_root)
            .expect("unified gameplay browser fixture directory is created");
        for (name, request) in fixtures {
            let exact = encode_wasm_process_request_v1(&request)
                .expect("unified gameplay browser CWR1 fixture encodes");
            std::fs::write(fixture_root.join(name), lowercase_hex_lines(&exact))
                .expect("unified gameplay browser CWR1 fixture update succeeds");
        }
        return;
    }
    for (name, request) in fixtures {
        let exact = encode_wasm_process_request_v1(&request)
            .expect("unified gameplay browser CWR1 fixture encodes");
        let tracked = std::fs::read_to_string(fixture_root.join(name))
            .expect("tracked unified gameplay browser CWR1 fixture exists");
        assert_eq!(decode_hex(&tracked), exact);
        assert_eq!(
            decode_wasm_process_request_v1(&exact).expect("tracked gameplay CWR1 decodes"),
            request
        );
    }
}
