use clause_package::*;
use clause_runtime::*;

const BROWSER_ADMISSION_COUNT: usize = 8;
const BROWSER_RECORDED_ALLOCATION_ROOT_TAG: u8 = 210;

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

fn n(value: f64) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Constant(number(value))
}

fn b(value: bool) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Constant(ExecutableValueV1::Boolean(value))
}

fn s(slot: u16) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Slot(slot)
}

fn a(argument: u16) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Argument(argument)
}

fn boxed(expression: ExecutableExpressionV1) -> Box<ExecutableExpressionV1> {
    Box::new(expression)
}

fn add(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Add(boxed(left), boxed(right))
}

fn sub(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Subtract(boxed(left), boxed(right))
}

fn mul(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Multiply(boxed(left), boxed(right))
}

fn div(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Divide(boxed(left), boxed(right))
}

fn clamp(
    value: ExecutableExpressionV1,
    lower: ExecutableExpressionV1,
    upper: ExecutableExpressionV1,
) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Clamp(boxed(value), boxed(lower), boxed(upper))
}

fn eq(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::Equal(boxed(left), boxed(right))
}

fn gt(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::GreaterThan(boxed(left), boxed(right))
}

fn le(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::LessThanOrEqual(boxed(left), boxed(right))
}

fn and(left: ExecutableExpressionV1, right: ExecutableExpressionV1) -> ExecutableExpressionV1 {
    ExecutableExpressionV1::And(boxed(left), boxed(right))
}

fn next_x() -> ExecutableExpressionV1 {
    clamp(add(s(0), mul(mul(s(4), s(8)), a(0))), s(10), s(11))
}

fn next_vertical_velocity() -> ExecutableExpressionV1 {
    add(s(3), mul(s(6), a(0)))
}

fn next_y() -> ExecutableExpressionV1 {
    add(s(1), mul(next_vertical_velocity(), a(0)))
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
            Term::raw_triple([
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
            Term::raw_triple([
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

fn headless_program(scope: TermScope) -> ExecutableProgramV1 {
    let horizontal_assignments = || vec![(0, next_x()), (2, div(sub(next_x(), s(0)), a(0)))];
    let mut grounded_tick = horizontal_assignments();
    grounded_tick.extend([(1, s(9)), (3, n(0.0))]);
    let mut airborne_tick = horizontal_assignments();
    airborne_tick.extend([(1, next_y()), (3, next_vertical_velocity())]);
    let mut landing_tick = horizontal_assignments();
    landing_tick.extend([(1, s(9)), (3, n(0.0)), (5, b(true))]);

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
    let template = projection_object(
        scope,
        vec![
            (b"player", player),
            (
                b"world",
                projection_object(
                    scope,
                    vec![(b"platforms", projection_array(scope, vec![platform]))],
                ),
            ),
        ],
    );

    ExecutableProgramV1 {
        // x, y, vx, vy, horizontal intent, grounded, and six package constants.
        initial_configuration: vec![
            number(9.5),
            number(0.0),
            number(0.0),
            number(0.0),
            number(0.0),
            ExecutableValueV1::Boolean(true),
            number(-8.0),
            number(8.0),
            number(5.0),
            number(0.0),
            number(-10.0),
            number(10.0),
            number(0.0),
            number(0.0),
            number(0.0),
            number(0.0),
            number(-0.25),
            number(0.0),
            number(12.0),
            number(0.5),
            number(12.0),
        ],
        rules: vec![
            ExecutableRuleV1 {
                entry: 0,
                predicates: vec![],
                assignments: vec![(4, a(0))],
            },
            ExecutableRuleV1 {
                entry: 1,
                predicates: vec![eq(s(5), b(true))],
                assignments: vec![(3, s(7)), (5, b(false))],
            },
            ExecutableRuleV1 {
                entry: 2,
                predicates: vec![eq(s(5), b(true))],
                assignments: grounded_tick,
            },
            ExecutableRuleV1 {
                entry: 2,
                predicates: vec![and(eq(s(5), b(false)), gt(next_y(), s(9)))],
                assignments: airborne_tick,
            },
            ExecutableRuleV1 {
                entry: 2,
                predicates: vec![and(eq(s(5), b(false)), le(next_y(), s(9)))],
                assignments: landing_tick,
            },
        ],
        projection: Some(ExecutableProjectionV1 {
            bindings: [
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
            .collect(),
            template,
        }),
    }
}

fn checked_program_package_with_scopes(
    checker_count: usize,
    state_admission_scopes: Vec<StateAdmissionScope>,
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
    projection_schema.roles = (1..=14)
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
    let constitution = package.constitution();
    let snapshot = constitution.snapshot();
    let application = ApplicationLocalId::new(1);
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
        program: headless_program(TermScope {
            universe: constitution.universe(),
            semantics: constitution.semantics(),
        }),
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
            .apply_opaque_input(&encode_executable_occurrence_v1(&occurrence(0, &[1.0])).unwrap())
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
            occurrence_ingress: ExecutableBoundaryFactV1 {
                boundary: self.pure_boundary,
                evidence: id!(ExternalEvidenceRef, 181),
            },
            judgment_ingress: ExecutableBoundaryFactV1 {
                boundary: self.state_boundary,
                evidence: id!(ExternalEvidenceRef, 186),
            },
            admission_ingress: ExecutableBoundaryFactV1 {
                boundary: self.state_boundary,
                evidence: id!(ExternalEvidenceRef, 190),
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
    let physical_plan = physical_plan(checked);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_RECORDED_ALLOCATION_ROOT_TAG),
        checked,
        application,
        &physical_plan,
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
    authority
        .establish_boundary(BoundaryAnchor {
            boundary: pure_boundary,
            semantics,
            snapshot,
            program_revision: revision.id,
            runtime_session: None,
            runtime_policy: None,
            permits: vec![
                EnteredOccurrenceKind::ExternalTrigger,
                EnteredOccurrenceKind::Observation,
            ],
        })
        .expect("pure boundary is established once");
    authority
        .establish_boundary(BoundaryAnchor {
            boundary: state_boundary,
            semantics,
            snapshot,
            program_revision: revision.id,
            runtime_session: Some(session),
            runtime_policy: Some(policy),
            permits: vec![
                EnteredOccurrenceKind::Judgment,
                EnteredOccurrenceKind::AdmissionDecision,
            ],
        })
        .expect("state boundary is established once");
    for (tag, boundary) in [
        (181, pure_boundary),
        (183, pure_boundary),
        (186, state_boundary),
        (187, state_boundary),
        (190, state_boundary),
    ] {
        authority
            .establish_evidence(EvidenceAnchor {
                evidence: id!(ExternalEvidenceRef, tag),
                boundary,
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
            .as_raw_triple()
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
        .as_raw_triple()
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
            encode_executable_occurrence_v1(&occurrence(0, &[1.0]))
                .expect("fixture input occurrence encodes"),
            encode_executable_occurrence_v1(&occurrence(2, &[0.25]))
                .expect("fixture tick occurrence encodes"),
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
        .advance_carrier_occurrence(occurrence(0, &[1.0]))
        .expect("opaque input enters with its computed Step");
    runtime
        .advance_carrier_occurrence(occurrence(2, &[0.25]))
        .expect("ground Step enters");
    assert_eq!(value(runtime.configuration(), 0), 10.0);
    assert_eq!(value(runtime.configuration(), 2), 2.0);

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
            encode_executable_occurrence_v1(&occurrence(0, &[1.0]))
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
                encode_executable_occurrence_v1(&occurrence(0, &[1.0])).unwrap(),
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

    let admitted = apply(
        &mut boundary,
        command(
            3,
            WasmSessionOperationV1::Admit(WasmSessionAdmissionV1 {
                package: package_id,
                session,
                base: initial_world,
                candidate,
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
            assert_arena_projection(&term, 10.0, 0.0);
        }
        other => panic!("unexpected Admission event: {other:?}"),
    }

    let disposed = apply(&mut boundary, command(4, WasmSessionOperationV1::Dispose));
    assert!(matches!(disposed.kind, WasmSessionEventKindV1::Disposed));
    let post_dispose =
        encode_wasm_session_command_v1(&command(5, WasmSessionOperationV1::Dispose)).unwrap();
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
fn shipped_cwr1_has_external_physical_plan_and_successive_constitutive_admission() {
    let request = browser_fixture_request();
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

    let package = check_process_package(
        decode_process_package(&request.package_bytes).expect("fixture package decodes"),
    )
    .expect("fixture package checks");
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
                    }),
                })
                .expect("fixture Admission command encodes"),
            )
            .expect("fixture Admission command transports");
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
        assert_arena_projection(&term, 10.0, if ordinal == 0 { 2.0 } else { 0.0 });
    }
}
