use clause_package::*;
use clause_runtime::*;

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

fn headless_program(scope: TermScope) -> ExecutableProgramV1 {
    let horizontal_assignments = || vec![(0, next_x()), (2, div(sub(next_x(), s(0)), a(0)))];
    let mut grounded_tick = horizontal_assignments();
    grounded_tick.extend([(1, s(9)), (3, n(0.0))]);
    let mut airborne_tick = horizontal_assignments();
    airborne_tick.extend([(1, next_y()), (3, next_vertical_velocity())]);
    let mut landing_tick = horizontal_assignments();
    landing_tick.extend([(1, s(9)), (3, n(0.0)), (5, b(true))]);

    let number_role = LocalRoleRefV2 {
        schema: RelationSchemaLocalId::new(2),
        role: RoleLocalId::new(1),
    };
    let boolean_role = LocalRoleRefV2 {
        schema: RelationSchemaLocalId::new(2),
        role: RoleLocalId::new(2),
    };
    let separator = Term::atom(
        scope,
        b"clause.test/projected-pair".to_vec(),
        Vec::new(),
        EqualityContract::ExactOctetsV1,
    )
    .expect("projection separator Atom is valid");
    let template = Term::raw_triple([
        executable_projection_role_term_v1(scope, number_role, ExecutableValueKindV1::Number)
            .expect("numeric projection role is valid"),
        separator,
        executable_projection_role_term_v1(scope, boolean_role, ExecutableValueKindV1::Boolean)
            .expect("boolean projection role is valid"),
    ])
    .expect("projection template has one scope");

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
            bindings: vec![
                ExecutableProjectionBindingV1 {
                    role: number_role,
                    slot: 0,
                    value_kind: ExecutableValueKindV1::Number,
                },
                ExecutableProjectionBindingV1 {
                    role: boolean_role,
                    slot: 5,
                    value_kind: ExecutableValueKindV1::Boolean,
                },
            ],
            template,
        }),
    }
}

fn checked_program_package(checker_count: usize) -> CheckedProcessPackage {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-vectors/process-v2/positive/process-v2-core.hex"
    ));
    let decoded = decode_process_package(&decode_hex(source)).expect("base package decodes");
    let mut candidate = decoded.candidate().clone();
    candidate.records.clear();

    let mut projection_schema = candidate.snapshot.constitution.schemas[0].clone();
    projection_schema.id = RelationSchemaLocalId::new(2);
    let mut boolean_role = projection_schema.roles[0].clone();
    boolean_role.id = RoleLocalId::new(2);
    projection_schema.roles.push(boolean_role);
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

    let scope = candidate.snapshot.constitution.semantics;
    let term_scope = TermScope {
        universe: candidate.snapshot.constitution.universe,
        semantics: scope,
    };
    let term = headless_program(term_scope)
        .encode_term(term_scope)
        .expect("closed program encodes as a Term");
    let dependency = LocalSemanticDependencyV2::ExternalReference(term);
    candidate.snapshot.constitution.formations[0]
        .direct_dependencies
        .push(dependency.clone());
    candidate.snapshot.constitution.formations[0]
        .direct_dependencies
        .sort();
    for application in &mut candidate.snapshot.constitution.applications {
        application.form.dependency_closure.push(dependency.clone());
        application.form.dependency_closure.sort();
        application.form.dependency_closure.dedup();
    }
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
    candidate.claimed_snapshot =
        derive_program_snapshot_id(&candidate.snapshot).expect("program snapshot is canonical");
    let bytes = encode_process_package(&candidate).expect("program package encodes");
    check_process_package(decode_process_package(&bytes).expect("program package decodes"))
        .expect("program package checks")
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
                        delta: id!(CandidateDeltaId, 80),
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
    let pure_boundary = id!(BoundaryRef, 126);
    let state_boundary = id!(BoundaryRef, 127);
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
    (
        authority,
        CarrierFacts {
            revision: revision.id,
            initial_state,
            session,
            policy,
            session_start,
            root_policy,
            pure_boundary,
            state_boundary,
        },
    )
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
        let (rejected_authority, rejected_facts) = carrier_authority(&rejected_package);
        let mut rejected_runtime = ExecutableProcessRuntimeV1::instantiate(
            rejected_package,
            rejected_authority,
            rejected_application,
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
    let (authority, facts) = carrier_authority(&package);
    let mut runtime = ExecutableProcessRuntimeV1::instantiate(package, authority, application)
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
    let request = WasmProcessRequestV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: ApplicationLocalId::new(1),
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
    assert_eq!(output.observation, id!(ObservationId, 100));
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
    let session = id!(RuntimeSessionId, 120);
    let open = WasmSessionOpenV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: ApplicationLocalId::new(1),
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
        identity_seed: raw_id(210),
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
            state_revision_count,
        } => {
            assert_eq!(opened_package, package_id);
            assert_eq!(actual_session, session);
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
                root_policy: id!(RootPolicyId, 130),
                authorization: AdmissionAuthorizationLocalId::new(0),
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
            let [number, relation, grounded] = term
                .as_raw_triple()
                .expect("fixture projection retains its declared Triple")
                .slots();
            assert_eq!(
                number.as_atom().expect("projected number Atom").kind(),
                b"clause/process-projected-f64-v1"
            );
            assert_eq!(
                f64::from_bits(u64::from_le_bytes(
                    number
                        .as_atom()
                        .expect("projected number Atom")
                        .canonical_payload()
                        .try_into()
                        .expect("projected F64 is exact"),
                )),
                10.0
            );
            assert_eq!(
                relation.as_atom().expect("literal relation Atom").kind(),
                b"clause.test/projected-pair"
            );
            assert_eq!(
                grounded
                    .as_atom()
                    .expect("projected Boolean Atom")
                    .canonical_payload(),
                [1]
            );
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
