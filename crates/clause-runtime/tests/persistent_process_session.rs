use clause_package::*;
use clause_runtime::*;
use std::fmt::Write;

const BROWSER_PROCESS_CONTINUATION_ALLOCATION_ROOT_TAG: u8 = 220;
const ONGOING_EFFECT_SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/process-v2/ongoing-effect.clause"
));

macro_rules! id {
    ($kind:ident, $tag:expr) => {
        $kind::from_bytes(raw_id($tag))
    };
}

fn raw_id(tag: u8) -> [u8; IDENTITY_BYTES] {
    let mut bytes = [0; IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[IDENTITY_BYTES - 1] = tag;
    bytes
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
    ExecutableValueV1::number(value).expect("test values are finite")
}

#[test]
fn text_occurrence_transport_has_a_distinct_bounded_utf8_wire_identity() {
    let text = ExecutableOccurrenceV1 {
        entry: 7,
        arguments: vec![ExecutableValueV1::text("North 🚀").expect("Text is bounded")],
    };
    let symbol = ExecutableOccurrenceV1 {
        entry: 7,
        arguments: vec![ExecutableValueV1::symbol(b"North").expect("Symbol is bounded")],
    };

    let text_bytes = encode_executable_occurrence_v1(&text).expect("Text occurrence encodes");
    let symbol_bytes = encode_executable_occurrence_v1(&symbol).expect("Symbol occurrence encodes");
    assert_eq!(text_bytes[8], 4, "Text owns wire value tag 4");
    assert_eq!(symbol_bytes[8], 2, "Symbol retains wire value tag 2");
    assert_ne!(text_bytes, symbol_bytes);
    assert_eq!(
        decode_executable_occurrence_v1(&text_bytes).expect("Text occurrence decodes"),
        text
    );

    let mut malformed_utf8 = text_bytes;
    malformed_utf8[11] = 0xff;
    assert_eq!(
        decode_executable_occurrence_v1(&malformed_utf8),
        Err(ExecutableErrorV1::MalformedProgram)
    );
    assert_eq!(
        ExecutableValueV1::text(&"x".repeat(usize::from(u16::MAX) + 1)),
        Err(ExecutableErrorV1::ResourceLimit)
    );
}

fn executable_program() -> ExecutableProgramV1 {
    ExecutableProgramV1 {
        initial_configuration: vec![number(0.0)],
        rules: vec![
            ExecutableRuleV1 {
                entry: 0,
                predicates: vec![],
                required_present: vec![],
                required_absent: vec![],
                assignments: vec![(0, ExecutableExpressionV1::Argument(0))],
                removals: vec![],
            },
            ExecutableRuleV1 {
                entry: 1,
                predicates: vec![],
                required_present: vec![],
                required_absent: vec![],
                assignments: vec![(
                    0,
                    ExecutableExpressionV1::Add(
                        Box::new(ExecutableExpressionV1::Slot(0)),
                        Box::new(ExecutableExpressionV1::Argument(0)),
                    ),
                )],
                removals: vec![],
            },
        ],
        projection: None,
    }
}

fn checked_program_package() -> CheckedProcessPackage {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-vectors/process-v2/positive/process-v2-core.hex"
    ));
    let decoded = decode_process_package(&decode_hex(source)).expect("base package decodes");
    let mut candidate = decoded.candidate().clone();
    candidate.records.clear();
    candidate.snapshot.constitution.operators[0].modes[1]
        .contract
        .continuation = ContinuationContractV2::Suspensible {
        use_policy: ContinuationUseV2::Linear,
        may_handoff: false,
        may_cancel: false,
    };
    candidate.claimed_snapshot =
        derive_program_snapshot_id(&candidate.snapshot).expect("program snapshot is canonical");
    let bytes = encode_process_package(&candidate).expect("program package encodes");
    check_process_package(decode_process_package(&bytes).expect("program package decodes"))
        .expect("program package checks")
}

fn checked_ongoing_effect_package() -> (CheckedProcessPackage, ApplicationLocalId, ModeLocalId) {
    let scope = TermScope {
        universe: id!(UniverseId, 151),
        semantics: id!(ClauseSemanticsId, 152),
    };
    let cst = read_canonical_source_v1(ONGOING_EFFECT_SOURCE)
        .expect("ongoing effect source reads losslessly");
    let allocation =
        plan_independent_canonical_source_allocations_v1(&cst, id!(ProgramChangeOccurrenceId, 153))
            .expect("ongoing effect source receives rooted semantic allocations");
    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: scope.universe,
            semantics: scope.semantics,
        },
        &allocation,
    )
    .expect("ongoing effect source reaches the checked package boundary");
    assert!(compiled.unsupported.is_empty());

    let decoded = decode_process_package(compiled.checked_package.exact_bytes())
        .expect("source-produced effect package decodes");
    let mut candidate = decoded.candidate().clone();
    let constitution = &mut candidate.snapshot.constitution;
    let schema = constitution.schemas[0].clone();
    let operator = constitution.operators[0].clone();
    let mode = operator.modes[0].clone();
    assert_eq!(
        mode.contract.productivity.kind,
        ProductivityKindV2::Reactive
    );
    assert!(matches!(
        mode.contract.continuation,
        ContinuationContractV2::Suspensible {
            use_policy: ContinuationUseV2::Linear,
            ..
        }
    ));
    assert_eq!(mode.contract.effect_intents.len(), 1);

    let mut next_formation = constitution
        .formations
        .iter()
        .map(|formation| formation.id.get())
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .expect("fixture Formation space remains available");
    let mut bindings = Vec::with_capacity(schema.roles.len());
    let mut binding_formations = Vec::new();
    for role in &schema.roles {
        let value = if mode.known_roles.binary_search(&role.id).is_ok() {
            let id = FormationLocalId::new(next_formation);
            next_formation = next_formation
                .checked_add(1)
                .expect("fixture Formation space remains available");
            let payload = role
                .target
                .type_term
                .as_atom()
                .expect("source role domain is one exact Atom")
                .canonical_payload()
                .to_vec();
            constitution.formations.push(FormationJudgmentPreimageV2 {
                id,
                context: vec![],
                term: Term::atom(
                    scope,
                    b"clause/test-effect-role-value-v1".to_vec(),
                    payload,
                    EqualityContract::ExactOctetsV1,
                )
                .expect("fixture role value is one exact Atom"),
                target: role.target.clone(),
                direct_dependencies: vec![],
            });
            binding_formations.push(id);
            RoleBindingValuePreimageV2::Known(id)
        } else {
            RoleBindingValuePreimageV2::Produced
        };
        bindings.push(RoleBindingPreimageV2 {
            role: role.id,
            occurrence: 0,
            value,
        });
    }

    let application_formation = FormationLocalId::new(next_formation);
    let mut dependencies = vec![
        LocalSemanticDependencyV2::RelationSchema(schema.id),
        LocalSemanticDependencyV2::Operator(operator.id),
        LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
            operator: operator.id,
            mode: mode.id,
        }),
    ];
    dependencies.extend(schema.roles.iter().map(|role| {
        LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
            schema: schema.id,
            role: role.id,
        })
    }));
    dependencies.extend(
        binding_formations
            .iter()
            .copied()
            .map(LocalSemanticDependencyV2::Formation),
    );
    dependencies.extend(
        mode.contract
            .productivity
            .obligations
            .iter()
            .copied()
            .map(LocalSemanticDependencyV2::Formation),
    );
    for capability in &constitution.capabilities {
        dependencies.push(LocalSemanticDependencyV2::Capability(capability.id));
        dependencies.push(LocalSemanticDependencyV2::Formation(capability.formation));
    }
    dependencies.sort();
    dependencies.dedup();
    constitution.formations.push(FormationJudgmentPreimageV2 {
        id: application_formation,
        context: vec![],
        term: Term::atom(
            scope,
            b"clause/test-effect-application-v1".to_vec(),
            b"dispatch".to_vec(),
            EqualityContract::ExactOctetsV1,
        )
        .expect("fixture Application term is one exact Atom"),
        target: schema.result_domain.clone(),
        direct_dependencies: dependencies.clone(),
    });
    constitution
        .formations
        .sort_by_key(|formation| formation.id);

    let application = ApplicationLocalId::new(1);
    let mut dependency_closure = dependencies;
    dependency_closure.push(LocalSemanticDependencyV2::Formation(application_formation));
    dependency_closure.sort();
    constitution
        .applications
        .push(ApplicationDeclarationPreimageV2 {
            id: application,
            form: ApplicationFormPreimageV2 {
                formation: application_formation,
                schema: schema.id,
                operator: operator.id,
                eligible_modes: vec![mode.id],
                bindings,
                context_requirements: vec![],
                constraint_discharges: vec![],
                result_domain: schema.result_domain,
                direct_dependencies: vec![],
                dependency_closure,
            },
        });

    let initial_payload = Term::atom(
        scope,
        b"clause/test-effect-world-v1".to_vec(),
        b"initial".to_vec(),
        EqualityContract::ExactOctetsV1,
    )
    .expect("fixture initial world is one exact Atom");
    candidate.initial_state_views = vec![InitialStateViewV2 {
        session: id!(RuntimeSessionId, 120),
        payload: initial_payload.clone(),
        canonical_state_snapshot: canonical_term_bytes(&initial_payload)
            .expect("fixture initial world has canonical bytes")
            .into_boxed_slice(),
    }];
    candidate.claimed_snapshot =
        derive_program_snapshot_id(&candidate.snapshot).expect("effect snapshot is canonical");
    let bytes = encode_process_package(&candidate).expect("effect package encodes");
    let checked = check_process_package(
        decode_process_package(&bytes).expect("encoded effect package decodes"),
    )
    .expect("source-owned ongoing effect package checks");
    (checked, application, mode.id)
}

fn open_fresh_session() -> PersistentProcessSessionV1 {
    let package = checked_program_package();
    let application = application(&package);
    let plan = physical_plan(&package);
    let (authority, facts) = carrier_authority(&package);
    PersistentProcessSessionV1::open(package, authority, application, plan, facts.executable)
        .expect("fresh conformance session opens")
}

fn physical_plan(package: &CheckedProcessPackage) -> ExecutablePhysicalPlanV1 {
    let constitution = package.constitution();
    let snapshot = constitution.snapshot();
    let application = ApplicationLocalId::new(1);
    let scope = TermScope {
        universe: constitution.universe(),
        semantics: constitution.semantics(),
    };
    let projection_role = LocalRoleRefV2 {
        schema: RelationSchemaLocalId::new(1),
        role: RoleLocalId::new(1),
    };
    let projection_end = Term::atom(
        scope,
        b"clause/js-object-end-v1".to_vec(),
        vec![],
        EqualityContract::ExactOctetsV1,
    )
    .expect("generic fixture projection has one object terminator");
    let projection_field = Term::atom(
        scope,
        b"clause/js-field-v1".to_vec(),
        b"value".to_vec(),
        EqualityContract::ExactOctetsV1,
    )
    .expect("generic fixture projection has one field");
    let projection_value =
        executable_projection_role_term_v1(scope, projection_role, ExecutableValueKindV1::Number)
            .expect("generic fixture projection binds one numeric role");
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
        program: ExecutableProgramV1 {
            projection: Some(ExecutableProjectionV1 {
                bindings: vec![ExecutableProjectionBindingV1 {
                    role: projection_role,
                    slot: 0,
                    value_kind: ExecutableValueKindV1::Number,
                }],
                template: Term::triple([projection_field, projection_value, projection_end])
                    .expect("generic fixture projection is one value object"),
            }),
            ..executable_program()
        },
    }
}

fn ongoing_effect_physical_plan(
    package: &CheckedProcessPackage,
    application: ApplicationLocalId,
    mode: ModeLocalId,
) -> ExecutablePhysicalPlanV1 {
    let constitution = package.constitution();
    let declaration = constitution
        .preimage()
        .applications
        .iter()
        .find(|candidate| candidate.id == application)
        .expect("effect fixture retains its exact Application");
    ExecutablePhysicalPlanV1 {
        application_shape: constitution
            .application_shape(application)
            .expect("effect fixture Application has one exact semantic shape"),
        mode: ModeId {
            operator: OperatorRef {
                snapshot: constitution.snapshot(),
                local: declaration.form.operator,
            },
            local: mode,
        },
        refinement: ExecutableRefinementV1::ClosedApplicationRuleMachineV1,
        target: ExecutablePhysicalTargetV1::PortableScalarInterpreterV1,
        input: None,
        program: ExecutableProgramV1 {
            initial_configuration: vec![number(0.0)],
            rules: vec![],
            projection: None,
        },
    }
}

fn browser_process_continuation_fixture_request() -> WasmProcessRequestV1 {
    let package = checked_program_package();
    let plan = physical_plan(&package);
    let application = application(&package);
    let (_, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(BROWSER_PROCESS_CONTINUATION_ALLOCATION_ROOT_TAG),
        &package,
        application,
        &plan,
        facts.executable,
    )
    .expect("generic continuation fixture records one exact allocation epoch");
    WasmProcessRequestV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: application.local,
        physical_plan_bytes: encode_executable_physical_plan_v1(&plan)
            .expect("generic continuation plan has one exact CWR1 payload"),
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
        occurrences: vec![opaque(0, 2.0), opaque(1, 3.0)],
        render_slots: vec![],
    }
}

fn browser_ongoing_effect_fixture_request() -> WasmProcessRequestV1 {
    let (package, application_local, mode_local) = checked_ongoing_effect_package();
    let plan = ongoing_effect_physical_plan(&package, application_local, mode_local);
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: application_local,
    };
    let (_, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(222),
        &package,
        application,
        &plan,
        facts.executable,
    )
    .expect("browser effect fixture records one exact allocation epoch");
    WasmProcessRequestV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: application_local,
        physical_plan_bytes: encode_executable_physical_plan_v1(&plan)
            .expect("browser effect physical plan encodes"),
        allocation,
        authority: wasm_authority_input(),
        occurrences: vec![opaque(0, 1.0)],
        render_slots: vec![],
    }
}

fn lowercase_hex_lines(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2 + 1);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    encoded.push('\n');
    encoded
}

#[derive(Clone, Copy)]
struct SessionFacts {
    executable: ExecutableAuthorityFactsV1,
    initial_state: StateRevisionId,
}

fn carrier_authority(checked: &CheckedProcessPackage) -> (AuthorityStore, SessionFacts) {
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
        .expect("program package retains one initial State view");
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
    let judgment_authority = RootJudgmentAuthorityRef {
        policy: root_policy,
        local: JudgmentAuthorityLocalId::new(0),
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
                vec![],
                vec![RootJudgmentAuthorityGrant {
                    authority: judgment_authority,
                    scope: JudgmentAuthorityScope {
                        semantics,
                        session,
                        policy,
                    },
                }],
                vec![RootStateAdmissionIssuerGrant {
                    issuer: RootAdmissionAuthorizationIssuerRef {
                        policy: root_policy,
                        local: AdmissionAuthorizationIssuerLocalId::new(0),
                    },
                    scope: StateAdmissionIssuerScope {
                        revision: revision.id,
                        package: checked.id(),
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
    let occurrence_boundary = id!(BoundaryRef, 126);
    let state_boundary = id!(BoundaryRef, 127);
    let boundary_target = checked.constitution().preimage().formations[0]
        .target
        .clone();
    let admitted = CheckedConstitutionBinding::Admitted {
        revision: revision.id,
    };
    authority
        .establish_boundary(executable_occurrence_boundary_anchor_v1(
            occurrence_boundary,
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
        .expect("occurrence boundary is established once");
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
    let occurrence_evidence = id!(ExternalEvidenceRef, 181);
    let judgment_evidence = id!(ExternalEvidenceRef, 186);
    let admission_evidence = id!(ExternalEvidenceRef, 190);
    for (evidence, boundary, permissions, bytes) in [
        (
            occurrence_evidence,
            occurrence_boundary,
            vec![
                EXECUTABLE_TRIGGER_PERMISSION_V1,
                EXECUTABLE_OBSERVATION_PERMISSION_V1,
            ],
            vec![181],
        ),
        (
            judgment_evidence,
            state_boundary,
            vec![EXECUTABLE_JUDGMENT_PERMISSION_V1],
            vec![186],
        ),
        (
            admission_evidence,
            state_boundary,
            vec![
                EXECUTABLE_ADMISSION_PERMISSION_V1,
                EXECUTABLE_ADMISSION_ISSUANCE_PERMISSION_V1,
                EXECUTABLE_RESUMPTION_PERMISSION_V1,
            ],
            vec![190],
        ),
    ] {
        authority
            .establish_evidence(EvidenceAnchor {
                evidence,
                boundary,
                permissions,
                exact_evidence: bytes.into_boxed_slice(),
            })
            .expect("external evidence is established once");
    }
    (
        authority,
        SessionFacts {
            executable: ExecutableAuthorityFactsV1 {
                program_revision: revision.id,
                session,
                initial_state,
                policy,
                session_start,
                root_policy,
                judgment_authority,
                admission_authorization_issuer: RootAdmissionAuthorizationIssuerRef {
                    policy: root_policy,
                    local: AdmissionAuthorizationIssuerLocalId::new(0),
                },
                trigger_ingress: ExecutableBoundaryFactV1 {
                    boundary: occurrence_boundary,
                    evidence: occurrence_evidence,
                    permission: EXECUTABLE_TRIGGER_PERMISSION_V1,
                },
                occurrence_ingress: ExecutableBoundaryFactV1 {
                    boundary: occurrence_boundary,
                    evidence: occurrence_evidence,
                    permission: EXECUTABLE_OBSERVATION_PERMISSION_V1,
                },
                resumption_ingress: ExecutableBoundaryFactV1 {
                    boundary: state_boundary,
                    evidence: admission_evidence,
                    permission: EXECUTABLE_RESUMPTION_PERMISSION_V1,
                },
                judgment_ingress: ExecutableBoundaryFactV1 {
                    boundary: state_boundary,
                    evidence: judgment_evidence,
                    permission: EXECUTABLE_JUDGMENT_PERMISSION_V1,
                },
                admission_issuance_ingress: ExecutableBoundaryFactV1 {
                    boundary: state_boundary,
                    evidence: admission_evidence,
                    permission: EXECUTABLE_ADMISSION_ISSUANCE_PERMISSION_V1,
                },
                admission_ingress: ExecutableBoundaryFactV1 {
                    boundary: state_boundary,
                    evidence: admission_evidence,
                    permission: EXECUTABLE_ADMISSION_PERMISSION_V1,
                },
                budget_units: 100,
            },
            initial_state,
        },
    )
}

fn admission_policy(
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
        .expect("per-candidate root policy is coherent"),
        AdmissionAuthorizationEvidence::IrreducibleRoot {
            policy,
            authorization,
        },
    )
}

fn opaque(entry: u16, argument: f64) -> Vec<u8> {
    encode_executable_occurrence_v1(&ExecutableOccurrenceV1 {
        entry,
        arguments: vec![number(argument)],
    })
    .expect("opaque occurrence encodes")
}

fn application(package: &CheckedProcessPackage) -> ApplicationId {
    ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    }
}

fn wasm_authority_input() -> WasmAuthorityInputV1 {
    WasmAuthorityInputV1 {
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
    }
}

#[test]
fn source_owned_ongoing_effect_lifecycle_remains_non_authoritative() {
    let (package, application_local, mode_local) = checked_ongoing_effect_package();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: application_local,
    };
    let plan = ongoing_effect_physical_plan(&package, application_local, mode_local);
    let physical_mode = plan.mode;
    let mode = package
        .constitution()
        .mode_by_id(plan.mode)
        .expect("effect Mode is retained by its source-produced constitution");
    assert_eq!(
        mode.contract.productivity.kind,
        ProductivityKindV2::Reactive
    );
    assert!(matches!(
        mode.contract.continuation,
        ContinuationContractV2::Suspensible {
            use_policy: ContinuationUseV2::Linear,
            ..
        }
    ));
    assert_eq!(mode.contract.effect_intents.len(), 1);

    let (authority, facts) = carrier_authority(&package);
    let mut session =
        PersistentProcessSessionV1::open(package, authority, application, plan, facts.executable)
            .expect("source-owned effect session opens without a fake state checker");
    let initial_state_count = session
        .carrier()
        .expect("effect session retains one carrier")
        .state_revision_count();
    let activation = session.activation().expect("effect Activation is live");

    let first = session
        .apply_opaque_input(&opaque(0, 1.0))
        .expect("ongoing effect Activation advances once");
    let suspension = session
        .suspend()
        .expect("source-declared linear continuation suspends");
    assert_eq!(suspension.activation, activation);
    session
        .resume()
        .expect("source-declared linear continuation resumes once");
    let second = session
        .apply_opaque_input(&opaque(0, 2.0))
        .expect("same ongoing Activation advances again");
    assert_ne!(first.id, second.id);
    assert_eq!(
        session.activation().expect("Activation remains live"),
        activation
    );

    let intent = session
        .emit_effect_intent()
        .expect("the Mode emits its exact external-effect intent");
    assert_eq!(
        session
            .pending_effect_intent()
            .expect("pending intent query succeeds")
            .map(|pending| pending.id),
        Some(intent.id)
    );
    assert_eq!(intent.scope.application, application);
    assert_eq!(intent.scope.mode, physical_mode);
    assert_eq!(intent.scope.world, facts.initial_state);
    let authorization = session
        .issue_effect_authorization(intent.id)
        .expect("Clause issues exact at-most-once capability use");
    let attempt = session
        .begin_effect_attempt(authorization.id)
        .expect("the issued capability authorizes one attempt");
    let duplicate = session
        .begin_effect_attempt(authorization.id)
        .expect_err("the same capability occurrence cannot authorize twice");
    assert!(matches!(
        duplicate,
        PersistentProcessSessionErrorV1::Carrier(ExecutableCarrierErrorV1::Ingress(
            ProcessIngressError::Record { cause, .. }
        )) if matches!(
            cause.as_ref(),
            ProcessError::EffectAuthorizationAlreadyConsumed(id) if *id == authorization.id
        )
    ));

    let settlement = session
        .settle_effect_attempt(attempt.id, Some((202, b"accepted".to_vec())))
        .expect("the exact foreign receipt is retained and judged");
    assert_eq!(
        settlement.disposition,
        EffectJudgmentDispositionV1::ReceiptObserved
    );
    let receipt = settlement
        .receipt
        .expect("receipt settlement retains receipt identity");
    let observation = settlement
        .observation
        .expect("receipt settlement retains ordinary Observation identity");
    assert!(
        session
            .pending_effect_intent()
            .expect("settled pending-intent query succeeds")
            .is_none()
    );

    let carrier = session
        .carrier()
        .expect("settled lifecycle remains queryable");
    assert_eq!(carrier.effect_intent(intent.id), Some(&intent));
    assert_eq!(
        carrier.issued_effect_authorization(authorization.id),
        Some(&authorization)
    );
    assert_eq!(carrier.effect_attempt(attempt.id), Some(&attempt));
    assert_eq!(
        carrier
            .effect_receipt(receipt)
            .expect("receipt is retained")
            .exact_bytes,
        b"accepted"
    );
    assert!(matches!(
        carrier
            .observation(observation)
            .expect("receipt-backed Observation is retained")
            .provenance,
        OccurrenceProvenance::ReportedByEffectReceipt(id) if id == receipt
    ));
    assert!(
        carrier
            .causal_predecessors(CausalRef::EffectIntent(intent.id))
            .expect("intent has one exact causal frontier")
            .contains(&CausalRef::Step(intent.emitted_by))
    );
    assert!(
        carrier
            .causal_predecessors(CausalRef::EffectAuthorization(authorization.id))
            .expect("authorization cites its intent")
            .contains(&CausalRef::EffectIntent(intent.id))
    );
    assert!(
        carrier
            .causal_predecessors(CausalRef::EffectAttempt(attempt.id))
            .expect("attempt cites its authorization")
            .contains(&CausalRef::EffectAuthorization(authorization.id))
    );
    assert!(
        carrier
            .causal_predecessors(CausalRef::Observation(observation))
            .expect("Observation cites its receipt")
            .contains(&CausalRef::EffectReceipt(receipt))
    );
    assert!(
        carrier
            .causal_predecessors(CausalRef::EffectJudgment(settlement.judgment))
            .expect("Judgment cites its evidence")
            .contains(&CausalRef::Observation(observation))
    );

    let no_receipt_intent = session
        .emit_effect_intent()
        .expect("the ongoing Mode can emit a later independent intent");
    let no_receipt_authorization = session
        .issue_effect_authorization(no_receipt_intent.id)
        .expect("later intent receives its own exact capability use");
    let no_receipt_attempt = session
        .begin_effect_attempt(no_receipt_authorization.id)
        .expect("later capability use begins one attempt");
    let no_receipt = session
        .settle_effect_attempt(no_receipt_attempt.id, None)
        .expect("absence of a receipt is an explicit distinct Judgment");
    assert_eq!(
        no_receipt.disposition,
        EffectJudgmentDispositionV1::NoReceipt
    );
    assert!(no_receipt.receipt.is_none());
    assert!(no_receipt.observation.is_none());

    let carrier = session
        .carrier()
        .expect("complete effect history remains queryable");
    assert_eq!(carrier.state_revision_count(), initial_state_count);
    assert_eq!(carrier.candidate_delta_count(), 0);
    assert!(session.last_admitted().is_none());
}

#[test]
fn persistent_wasm_boundary_transports_the_exact_effect_lifecycle() {
    let (package, application_local, mode_local) = checked_ongoing_effect_package();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: application_local,
    };
    let plan = ongoing_effect_physical_plan(&package, application_local, mode_local);
    let (_, facts) = carrier_authority(&package);
    let allocation = RuntimeAllocationEpochV1::recorded_for(
        raw_id(221),
        &package,
        application,
        &plan,
        facts.executable,
    )
    .expect("effect Wasm fixture records one exact allocation epoch");
    let open = WasmSessionOpenV1 {
        package_bytes: package.exact_bytes().to_vec(),
        application: application_local,
        physical_plan_bytes: encode_executable_physical_plan_v1(&plan)
            .expect("effect physical plan encodes"),
        authority: wasm_authority_input(),
        allocation: WasmSessionAllocationV1::Rematerialize(allocation),
        limits: WasmSessionLimitsV1 {
            max_commands: 16,
            command_bytes: 4096,
            event_bytes: WASM_SESSION_EVENT_LIMIT_V1 as u32,
            trace_retention: WasmSessionTraceRetentionV1::FullUntilCommandLimit,
        },
    };
    let open_bytes = encode_wasm_session_open_v1(&open).expect("effect CWS1 open encodes");
    assert_eq!(
        decode_wasm_session_open_v1(&open_bytes).expect("effect CWS1 open decodes"),
        open
    );
    let mut boundary = WasmPersistentSessionBoundaryV1::new();
    let opened = boundary
        .open(&open_bytes)
        .expect("effect Wasm session opens");
    let handle = opened.handle;
    let initial_state_count = match opened.kind {
        WasmSessionEventKindV1::Opened {
            state_revision_count,
            ..
        } => state_revision_count,
        other => panic!("unexpected effect open event: {other:?}"),
    };
    let command = |expected_sequence, operation| WasmSessionCommandV1 {
        handle,
        expected_sequence,
        operation,
    };
    let apply = |boundary: &mut WasmPersistentSessionBoundaryV1, command: WasmSessionCommandV1| {
        let bytes =
            encode_wasm_session_command_v1(&command).expect("bounded effect command encodes");
        assert_eq!(
            decode_wasm_session_command_v1(&bytes).expect("effect command decodes"),
            command
        );
        let event = boundary.command(&bytes).expect("effect command transports");
        let event_bytes = encode_wasm_session_event_v1(&event);
        assert_eq!(
            decode_wasm_session_event_v1(&event_bytes).expect("effect event decodes"),
            event
        );
        event
    };

    assert!(matches!(
        apply(
            &mut boundary,
            command(0, WasmSessionOperationV1::QueryPendingEffectIntent)
        )
        .kind,
        WasmSessionEventKindV1::EffectIntentAbsent { state_revision_count }
            if state_revision_count == initial_state_count
    ));
    assert!(matches!(
        apply(
            &mut boundary,
            command(1, WasmSessionOperationV1::Input(opaque(0, 1.0)))
        )
        .kind,
        WasmSessionEventKindV1::InputAccepted { state_revision_count, .. }
            if state_revision_count == initial_state_count
    ));
    let emitted = apply(
        &mut boundary,
        command(2, WasmSessionOperationV1::EmitEffectIntent),
    );
    let (intent, action_bytes, resource_bytes, payload_bytes) = match emitted.kind {
        WasmSessionEventKindV1::EffectIntentAvailable {
            intent,
            action_bytes,
            resource_bytes,
            payload_bytes,
            state_revision_count,
            ..
        } => {
            assert_eq!(state_revision_count, initial_state_count);
            (intent, action_bytes, resource_bytes, payload_bytes)
        }
        other => panic!("unexpected effect intent event: {other:?}"),
    };
    let queried = apply(
        &mut boundary,
        command(3, WasmSessionOperationV1::QueryPendingEffectIntent),
    );
    assert!(matches!(
        queried.kind,
        WasmSessionEventKindV1::EffectIntentAvailable {
            intent: queried_intent,
            action_bytes: ref queried_action,
            resource_bytes: ref queried_resource,
            payload_bytes: ref queried_payload,
            ..
        } if queried_intent == intent
            && queried_action == &action_bytes
            && queried_resource == &resource_bytes
            && queried_payload == &payload_bytes
    ));
    let issued = apply(
        &mut boundary,
        command(4, WasmSessionOperationV1::IssueEffectAuthorization(intent)),
    );
    let authorization = match issued.kind {
        WasmSessionEventKindV1::EffectAuthorizationIssued {
            authorization,
            intent: authorized_intent,
            state_revision_count,
        } => {
            assert_eq!(authorized_intent, intent);
            assert_eq!(state_revision_count, initial_state_count);
            authorization
        }
        other => panic!("unexpected effect authorization event: {other:?}"),
    };
    let begun = apply(
        &mut boundary,
        command(5, WasmSessionOperationV1::BeginEffectAttempt(authorization)),
    );
    let attempt = match begun.kind {
        WasmSessionEventKindV1::EffectAttemptBegun {
            attempt,
            intent: attempted_intent,
            authorization: used_authorization,
            action_bytes: attempted_action,
            resource_bytes: attempted_resource,
            payload_bytes: attempted_payload,
            state_revision_count,
        } => {
            assert_eq!(attempted_intent, intent);
            assert_eq!(used_authorization, authorization);
            assert_eq!(attempted_action, action_bytes);
            assert_eq!(attempted_resource, resource_bytes);
            assert_eq!(attempted_payload, payload_bytes);
            assert_eq!(state_revision_count, initial_state_count);
            attempt
        }
        other => panic!("unexpected effect attempt event: {other:?}"),
    };
    let settled = apply(
        &mut boundary,
        command(
            6,
            WasmSessionOperationV1::SettleEffectAttempt {
                attempt,
                receipt: Some(WasmSessionEffectReceiptV1 {
                    status: 202,
                    exact_bytes: b"accepted".to_vec(),
                }),
            },
        ),
    );
    assert!(matches!(
        settled.kind,
        WasmSessionEventKindV1::EffectSettled {
            intent: settled_intent,
            attempt: settled_attempt,
            receipt: Some(_),
            observation: Some(_),
            disposition: EffectJudgmentDispositionV1::ReceiptObserved,
            state_revision_count,
            ..
        } if settled_intent == intent
            && settled_attempt == attempt
            && state_revision_count == initial_state_count
    ));

    let second_intent = match apply(
        &mut boundary,
        command(7, WasmSessionOperationV1::EmitEffectIntent),
    )
    .kind
    {
        WasmSessionEventKindV1::EffectIntentAvailable { intent, .. } => intent,
        other => panic!("unexpected second effect intent event: {other:?}"),
    };
    let second_authorization = match apply(
        &mut boundary,
        command(
            8,
            WasmSessionOperationV1::IssueEffectAuthorization(second_intent),
        ),
    )
    .kind
    {
        WasmSessionEventKindV1::EffectAuthorizationIssued { authorization, .. } => authorization,
        other => panic!("unexpected second effect authorization event: {other:?}"),
    };
    let second_attempt = match apply(
        &mut boundary,
        command(
            9,
            WasmSessionOperationV1::BeginEffectAttempt(second_authorization),
        ),
    )
    .kind
    {
        WasmSessionEventKindV1::EffectAttemptBegun { attempt, .. } => attempt,
        other => panic!("unexpected second effect attempt event: {other:?}"),
    };
    assert!(matches!(
        apply(
            &mut boundary,
            command(
                10,
                WasmSessionOperationV1::SettleEffectAttempt {
                    attempt: second_attempt,
                    receipt: None,
                }
            )
        )
        .kind,
        WasmSessionEventKindV1::EffectSettled {
            receipt: None,
            observation: None,
            disposition: EffectJudgmentDispositionV1::NoReceipt,
            state_revision_count,
            ..
        } if state_revision_count == initial_state_count
    ));
}

#[test]
fn forked_process_branch_reconnects_through_separate_admission_and_retains_exact_explanation() {
    let mut authoritative = open_fresh_session();
    let branch_session = open_fresh_session();
    let parent = authoritative.world_base();
    let mut branch =
        ForkedProcessBranchV1::fork(&authoritative, branch_session, 41, &opaque(1, 0.0))
            .expect("exact equal-base session forks as one non-authoritative branch");
    assert_eq!(branch.pins().parent_state, parent);
    assert_ne!(
        authoritative.allocation().root(),
        branch.suspension().continuation.as_bytes(),
        "physical allocation custody does not substitute for Continuation identity"
    );

    authoritative
        .apply_opaque_input(&opaque(0, 10.0))
        .expect("authoritative execution advances independently");
    authoritative
        .apply_opaque_input_and_emit_candidate(&opaque(1, 1.0))
        .expect("authoritative execution emits its own candidate");
    let first_authorization = authoritative
        .issue_candidate_admission_authorization()
        .expect("authoritative candidate receives exact issued authority");
    let first_state = authoritative
        .admit_issued_candidate_with_projection(first_authorization)
        .expect("only Admission establishes the independently advanced base")
        .0;
    assert_eq!(first_state.predecessor, parent);
    assert_ne!(first_state.id, parent);

    let branch_commands = vec![opaque(0, 2.0), opaque(1, 3.0)];
    let reconnect = branch
        .resume_and_propose(&branch_commands)
        .expect("branch resumes and emits a hidden CandidateDelta");
    let retained = branch
        .retained_candidate()
        .expect("branch candidate remains retained and non-authoritative")
        .clone();
    assert_eq!(retained.id, reconnect.candidate);
    assert_eq!(retained.base, parent);
    assert_eq!(authoritative.world_base(), first_state.id);
    assert_ne!(reconnect.ancestry.run, authoritative.run().unwrap());
    assert_eq!(reconnect.command_evidence.len(), branch_commands.len());
    for (expected, evidence) in branch_commands.iter().zip(&reconnect.command_evidence) {
        assert_eq!(&evidence.occurrence, expected);
    }
    assert_eq!(
        reconnect.candidate_step,
        reconnect
            .command_evidence
            .last()
            .expect("candidate command evidence is retained")
            .step
    );
    assert!(reconnect.command_evidence.windows(2).all(|commands| {
        commands[0].step != commands[1].step && commands[0].observation != commands[1].observation
    }));

    for (tampered, expected) in [
        (
            {
                let mut value = reconnect.clone();
                value.pins.parent_state = id!(StateRevisionId, 201);
                value
            },
            ProcessBranchPinV1::ParentState,
        ),
        (
            {
                let mut value = reconnect.clone();
                value.pins.program_revision = id!(ProgramRevisionId, 202);
                value
            },
            ProcessBranchPinV1::ProgramRevision,
        ),
        (
            {
                let mut value = reconnect.clone();
                value.pins.root_policy = id!(RootPolicyId, 203);
                value
            },
            ProcessBranchPinV1::RootPolicy,
        ),
        (
            {
                let mut value = reconnect.clone();
                value.pins.input_evidence = id!(ExternalEvidenceRef, 204);
                value
            },
            ProcessBranchPinV1::InputEvidence,
        ),
        (
            {
                let mut value = reconnect.clone();
                value.pins.budget_units += 1;
                value
            },
            ProcessBranchPinV1::Budget,
        ),
    ] {
        let plan = CheckedReconnectAdmissionPlanV1 {
            branch_candidate: reconnect.candidate,
            authoritative_base: first_state.id,
            occurrences: branch_commands.clone(),
        };
        assert!(matches!(
            branch.adjudicate(&mut authoritative, &tampered, &plan),
            Err(ProcessBranchErrorV1::PinMismatch(actual)) if actual == expected
        ));
        assert_eq!(authoritative.world_base(), first_state.id);
        assert!(branch.explanation().is_none());
    }

    let mut reordered = reconnect.clone();
    reordered.command_evidence.swap(0, 1);
    let exact_order_plan = CheckedReconnectAdmissionPlanV1 {
        branch_candidate: reconnect.candidate,
        authoritative_base: first_state.id,
        occurrences: branch_commands.clone(),
    };
    assert!(matches!(
        branch.adjudicate(&mut authoritative, &reordered, &exact_order_plan),
        Err(ProcessBranchErrorV1::UnexpectedCandidate)
    ));
    assert_eq!(authoritative.world_base(), first_state.id);
    assert!(branch.explanation().is_none());

    let stale_plan = CheckedReconnectAdmissionPlanV1 {
        branch_candidate: reconnect.candidate,
        authoritative_base: parent,
        occurrences: branch_commands.clone(),
    };
    assert!(matches!(
        branch.adjudicate(&mut authoritative, &reconnect, &stale_plan),
        Err(ProcessBranchErrorV1::PinMismatch(
            ProcessBranchPinV1::AuthoritativeBase
        ))
    ));
    assert_eq!(authoritative.world_base(), first_state.id);

    let adjudication_occurrences = branch_commands.clone();
    let plan = CheckedReconnectAdmissionPlanV1 {
        branch_candidate: reconnect.candidate,
        authoritative_base: first_state.id,
        occurrences: branch_commands,
    };
    let admitted = branch
        .adjudicate(&mut authoritative, &reconnect, &plan)
        .expect("caller-selected replay plan produces one separately admitted successor");
    assert_eq!(admitted.state.predecessor, first_state.id);
    assert_eq!(admitted.state.id, authoritative.world_base());
    assert_ne!(admitted.state.id, retained.base);
    assert_eq!(admitted.explanation.branch_candidate, retained.id);
    assert_eq!(
        admitted.explanation.branch_command_evidence,
        reconnect.command_evidence
    );
    assert_eq!(
        admitted
            .explanation
            .authoritative_command_evidence
            .iter()
            .map(|evidence| &evidence.occurrence)
            .collect::<Vec<_>>(),
        adjudication_occurrences.iter().collect::<Vec<_>>()
    );
    let decision = authoritative
        .carrier()
        .unwrap()
        .decision_by_occurrence(admitted.state.admission)
        .expect("Admission decision remains queryable");
    assert_eq!(admitted.explanation.authoritative_candidate, decision.delta);
    assert_eq!(admitted.explanation.admission, admitted.state.admission);
    assert_eq!(admitted.explanation.successor, admitted.state.id);
    assert!(
        admitted
            .explanation
            .causal_records
            .iter()
            .any(|record| record.occurrence == CausalRef::CandidateDelta(retained.id))
    );
    let authoritative_occurrences = admitted
        .explanation
        .causal_records
        .iter()
        .filter(|record| {
            matches!(
                record.occurrence,
                CausalRef::CandidateDelta(_) | CausalRef::Judgment(_) | CausalRef::Admission(_)
            )
        })
        .collect::<Vec<_>>();
    assert!(
        authoritative_occurrences.iter().all(|record| {
            !record
                .predecessors
                .contains(&CausalRef::CandidateDelta(retained.id))
        }),
        "the retained reconnect evidence is not rewritten into cross-Run order"
    );
    assert_eq!(
        branch.explanation(),
        Some(&admitted.explanation),
        "the old branch remains retained and explainable after authoritative Admission"
    );
}

#[test]
fn public_wasm_branch_boundary_retains_exact_evidence_and_admission_only_successors() {
    let exact_cwr1 =
        encode_wasm_process_request_v1(&browser_process_continuation_fixture_request())
            .expect("generic continuation CWR1 encodes");
    let open = WasmBranchOpenV1 {
        exact_cwr1,
        disconnect_tick: 41,
        disconnect_occurrence: opaque(1, 0.0),
        max_commands: 8,
    };
    let open_bytes = encode_wasm_branch_open_v1(&open).expect("bounded CBR1 open encodes");
    assert_eq!(
        decode_wasm_branch_open_v1(&open_bytes).expect("exact CBR1 open decodes"),
        open
    );

    let mut boundary = WasmProcessBranchBoundaryV1::new();
    let opened = boundary.open(&open_bytes).expect("exact branch opens");
    assert_eq!(
        decode_wasm_branch_event_v1(
            &encode_wasm_branch_event_v1(&opened).expect("opened event encodes")
        )
        .expect("opened event decodes"),
        opened
    );
    let (handle, parent, branch_run) = match opened.kind {
        WasmBranchEventKindV1::Opened { pins, ancestry, .. } => {
            assert_eq!(pins.disconnect_tick, 41);
            assert_eq!(pins.parent_state, ancestry.parent_state);
            (opened.handle, pins.parent_state, ancestry.run)
        }
        other => panic!("unexpected open event: {other:?}"),
    };

    let stale = encode_wasm_branch_command_v1(&WasmBranchCommandV1 {
        handle: WasmBranchHandleV1 {
            slot: handle.slot,
            generation: handle.generation + 1,
        },
        expected_sequence: 0,
        operation: WasmBranchOperationV1::Explain,
    })
    .expect("stale command encodes");
    assert_eq!(
        boundary.command(&stale),
        Err(WasmProcessStatusV1::StaleSessionHandle)
    );

    let authoritative_command = WasmBranchCommandV1 {
        handle,
        expected_sequence: 0,
        operation: WasmBranchOperationV1::AdmitAuthoritativeOccurrences(vec![
            opaque(0, 10.0),
            opaque(1, 1.0),
        ]),
    };
    let authoritative_bytes = encode_wasm_branch_command_v1(&authoritative_command)
        .expect("authoritative command encodes");
    assert_eq!(
        decode_wasm_branch_command_v1(&authoritative_bytes).expect("authoritative command decodes"),
        authoritative_command
    );
    let authoritative = boundary
        .command(&authoritative_bytes)
        .expect("authoritative plan admits");
    let (r1, authoritative_run) = match authoritative.kind {
        WasmBranchEventKindV1::AuthoritativeAdmissionAccepted {
            predecessor,
            successor,
            run,
            ..
        } => {
            assert_eq!(predecessor, parent);
            assert_ne!(successor, parent);
            assert_ne!(run, branch_run);
            (successor, run)
        }
        other => panic!("unexpected authoritative event: {other:?}"),
    };

    let branch_occurrences = vec![opaque(0, 2.0), opaque(1, 3.0)];
    let proposed = boundary
        .command(
            &encode_wasm_branch_command_v1(&WasmBranchCommandV1 {
                handle,
                expected_sequence: 1,
                operation: WasmBranchOperationV1::ProposeReconnect(branch_occurrences.clone()),
            })
            .expect("branch proposal command encodes"),
        )
        .expect("branch proposal succeeds");
    let (branch_candidate, exact_evidence) = match proposed.kind {
        WasmBranchEventKindV1::ReconnectProposed {
            evidence,
            exact_evidence,
        } => {
            assert_eq!(evidence.pins.parent_state, parent);
            assert_eq!(evidence.ancestry.run, branch_run);
            assert_eq!(evidence.command_evidence.len(), branch_occurrences.len());
            for (expected, command) in branch_occurrences.iter().zip(&evidence.command_evidence) {
                assert_eq!(&command.occurrence, expected);
            }
            assert_eq!(
                evidence.candidate_step,
                evidence
                    .command_evidence
                    .last()
                    .expect("candidate command evidence is retained")
                    .step
            );
            assert_eq!(
                exact_evidence,
                encode_process_reconnect_evidence_v1(&evidence)
                    .expect("retained evidence re-encodes exactly")
            );
            (evidence.candidate, exact_evidence)
        }
        other => panic!("unexpected proposal event: {other:?}"),
    };

    let mut mismatched_evidence = exact_evidence.clone();
    *mismatched_evidence
        .last_mut()
        .expect("evidence has one candidate-step byte") ^= 1;
    assert_eq!(
        decode_process_reconnect_evidence_v1(&mismatched_evidence),
        Err(WasmProcessStatusV1::MalformedRequest),
        "CRE1 rejects a candidate Step that no longer matches the final ordered command record"
    );
    let rejected = boundary
        .command(
            &encode_wasm_branch_command_v1(&WasmBranchCommandV1 {
                handle,
                expected_sequence: 2,
                operation: WasmBranchOperationV1::Adjudicate {
                    reconnect_evidence: mismatched_evidence,
                    branch_candidate,
                    authoritative_base: r1,
                    occurrences: branch_occurrences.clone(),
                },
            })
            .expect("tampered evidence command encodes"),
        )
        .expect("typed evidence rejection is one accepted physical command");
    assert!(matches!(
        rejected.kind,
        WasmBranchEventKindV1::Rejected(WasmBranchRejectionV1::EvidenceMismatch)
    ));

    let admitted = boundary
        .command(
            &encode_wasm_branch_command_v1(&WasmBranchCommandV1 {
                handle,
                expected_sequence: 3,
                operation: WasmBranchOperationV1::Adjudicate {
                    reconnect_evidence: exact_evidence,
                    branch_candidate,
                    authoritative_base: r1,
                    occurrences: branch_occurrences.clone(),
                },
            })
            .expect("adjudication command encodes"),
        )
        .expect("checked reconnect plan admits");
    let (r2, exact_explanation) = match admitted.kind {
        WasmBranchEventKindV1::ReconnectAdmissionAccepted {
            predecessor,
            successor,
            branch_candidate: admitted_branch_candidate,
            explanation,
            exact_explanation,
            ..
        } => {
            assert_eq!(predecessor, r1);
            assert_ne!(successor, r1);
            assert_eq!(admitted_branch_candidate, branch_candidate);
            assert_eq!(explanation.branch_candidate, branch_candidate);
            assert_eq!(explanation.authoritative_base, r1);
            assert_eq!(explanation.authoritative_run, authoritative_run);
            assert_eq!(explanation.successor, successor);
            assert_eq!(
                explanation
                    .branch_command_evidence
                    .iter()
                    .map(|evidence| &evidence.occurrence)
                    .collect::<Vec<_>>(),
                branch_occurrences.iter().collect::<Vec<_>>()
            );
            assert_eq!(
                explanation
                    .authoritative_command_evidence
                    .iter()
                    .map(|evidence| &evidence.occurrence)
                    .collect::<Vec<_>>(),
                branch_occurrences.iter().collect::<Vec<_>>()
            );
            assert!(explanation.causal_records.iter().all(|record| {
                if matches!(
                    record.occurrence,
                    CausalRef::CandidateDelta(candidate)
                        if candidate == explanation.authoritative_candidate
                ) || matches!(
                    record.occurrence,
                    CausalRef::Judgment(_) | CausalRef::Admission(_)
                ) {
                    !record
                        .predecessors
                        .contains(&CausalRef::CandidateDelta(branch_candidate))
                } else {
                    true
                }
            }));
            assert_eq!(
                exact_explanation,
                encode_process_branch_explanation_v1(&explanation)
                    .expect("causal explanation re-encodes exactly")
            );
            (successor, exact_explanation)
        }
        other => panic!("unexpected adjudication event: {other:?}"),
    };

    let explained = boundary
        .command(
            &encode_wasm_branch_command_v1(&WasmBranchCommandV1 {
                handle,
                expected_sequence: 4,
                operation: WasmBranchOperationV1::Explain,
            })
            .expect("explanation query encodes"),
        )
        .expect("retained explanation remains queryable");
    match explained.kind {
        WasmBranchEventKindV1::Explanation {
            explanation,
            exact_explanation: queried,
        } => {
            assert_eq!(explanation.successor, r2);
            assert_eq!(queried, exact_explanation);
        }
        other => panic!("unexpected explanation event: {other:?}"),
    }
}

#[test]
fn shipped_process_continuation_cwr1_is_exact() {
    let request = browser_process_continuation_fixture_request();
    let exact = encode_wasm_process_request_v1(&request)
        .expect("generic continuation browser CWR1 fixture encodes");
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../browser/jump-arena-shell/fixtures/wasm-process-continuation-v1/",
        "process-continuation-v1.cwr1.hex"
    );
    if std::env::var_os("CLAUSE_UPDATE_BROWSER_PROCESS_CONTINUATION_CWR1").is_some() {
        std::fs::write(fixture_path, lowercase_hex_lines(&exact))
            .expect("generic continuation browser CWR1 fixture update succeeds");
        return;
    }
    let tracked = decode_hex(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../browser/jump-arena-shell/fixtures/wasm-process-continuation-v1/",
        "process-continuation-v1.cwr1.hex"
    )));
    assert_eq!(tracked, exact);
    assert_eq!(
        decode_wasm_process_request_v1(&tracked)
            .expect("tracked generic continuation CWR1 decodes"),
        request
    );
}

#[test]
fn shipped_ongoing_effect_cwr1_is_exact() {
    let request = browser_ongoing_effect_fixture_request();
    let exact = encode_wasm_process_request_v1(&request)
        .expect("ongoing effect browser CWR1 fixture encodes");
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../browser/jump-arena-shell/fixtures/wasm-ongoing-effect-v1/",
        "ongoing-effect-v1.cwr1.hex"
    );
    if std::env::var_os("CLAUSE_UPDATE_BROWSER_ONGOING_EFFECT_CWR1").is_some() {
        std::fs::write(fixture_path, lowercase_hex_lines(&exact))
            .expect("ongoing effect browser CWR1 fixture update succeeds");
        return;
    }
    let tracked = decode_hex(
        &std::fs::read_to_string(fixture_path)
            .expect("tracked ongoing effect browser CWR1 fixture reads"),
    );
    assert_eq!(tracked, exact);
    assert_eq!(
        decode_wasm_process_request_v1(&tracked)
            .expect("tracked ongoing effect browser CWR1 decodes"),
        request
    );
}

#[test]
fn new_sessions_are_nominally_fresh_and_recorded_occurrences_rematerialize_exactly() {
    let first_package = checked_program_package();
    let first_application = application(&first_package);
    let first_plan = physical_plan(&first_package);
    let exact_plan = encode_executable_physical_plan_v1(&first_plan)
        .expect("physical plan has one exact external encoding");
    assert_eq!(
        decode_executable_physical_plan_v1(&exact_plan)
            .expect("exact physical plan decodes independently"),
        first_plan
    );
    let (first_authority, first_facts) = carrier_authority(&first_package);
    let mut first = PersistentProcessSessionV1::open(
        first_package,
        first_authority,
        first_application,
        first_plan,
        first_facts.executable,
    )
    .expect("first new session receives one fresh allocation root");
    let recorded = first.allocation();

    let second_package = checked_program_package();
    let second_application = application(&second_package);
    let second_plan = physical_plan(&second_package);
    let (second_authority, second_facts) = carrier_authority(&second_package);
    let second = PersistentProcessSessionV1::open(
        second_package,
        second_authority,
        second_application,
        second_plan,
        second_facts.executable,
    )
    .expect("same semantic inputs still allocate a new occurrence family");

    assert_ne!(first.allocation(), second.allocation());
    assert_ne!(first.run().unwrap(), second.run().unwrap());
    assert_ne!(first.activation().unwrap(), second.activation().unwrap());
    assert_ne!(
        first.configuration_id().unwrap(),
        second.configuration_id().unwrap()
    );

    let recorded_bytes = encode_runtime_allocation_epoch_v1(recorded);
    let decoded_record = decode_runtime_allocation_epoch_v1(&recorded_bytes)
        .expect("recorded allocation has one exact encoding");
    assert_eq!(decoded_record, recorded);
    let replay_package = checked_program_package();
    let replay_application = application(&replay_package);
    let replay_plan = physical_plan(&replay_package);
    let (replay_authority, replay_facts) = carrier_authority(&replay_package);
    let mut replay = PersistentProcessSessionV1::rematerialize(
        replay_package,
        replay_authority,
        replay_application,
        replay_plan,
        replay_facts.executable,
        decoded_record,
    )
    .expect("explicit rematerialization preserves the recorded occurrence");

    assert_eq!(replay.allocation(), recorded);
    assert_eq!(replay.run().unwrap(), first.run().unwrap());
    assert_eq!(replay.activation().unwrap(), first.activation().unwrap());
    assert_eq!(
        replay.configuration_id().unwrap(),
        first.configuration_id().unwrap()
    );
    let first_step = first
        .apply_opaque_input(&opaque(0, 7.0))
        .expect("recorded occurrence advances once");
    let replay_step = replay
        .apply_opaque_input(&opaque(0, 7.0))
        .expect("rematerialized occurrence advances identically");
    assert_eq!(replay_step, first_step);
}

#[test]
fn persistent_session_keeps_local_steps_and_advances_only_at_atomic_admission() {
    let package = checked_program_package();
    let package_id = package.id();
    let application = application(&package);
    let physical_plan = physical_plan(&package);
    let session_id = id!(RuntimeSessionId, 120);
    let (authority, facts) = carrier_authority(&package);
    let mut session = PersistentProcessSessionV1::open(
        package,
        authority,
        application,
        physical_plan,
        facts.executable,
    )
    .expect("persistent session opens once");

    let initial_run = session.run().unwrap();
    let initial_activation = session.activation().unwrap();
    let initial_configuration = session.configuration_id().unwrap();
    assert_eq!(session.package().unwrap(), package_id);
    assert_eq!(session.runtime_session(), session_id);
    assert_eq!(session.world_base(), facts.initial_state);

    let input_one = session
        .apply_opaque_input(&opaque(0, 2.0))
        .expect("first opaque input advances local configuration");
    let input_two = session
        .apply_opaque_input(&opaque(0, 3.0))
        .expect("second opaque input advances local configuration");
    let tick_one = session
        .apply_opaque_input(&opaque(1, 1.0))
        .expect("first local tick advances without Admission");
    let tick_two = session
        .apply_opaque_input_and_emit_candidate(&opaque(1, 1.0))
        .expect("second local tick emits one immutable candidate");
    let steps = [input_one, input_two, tick_one, tick_two];
    assert_eq!(steps[0].before, initial_configuration);
    for pair in steps.windows(2) {
        assert_eq!(pair[0].after, pair[1].before);
        assert_ne!(pair[0].id, pair[1].id);
        assert_ne!(pair[0].input_observation, pair[1].input_observation);
    }
    for step in &steps {
        let observation = step
            .input_observation
            .expect("every carrier Step retains its entered Observation");
        assert!(
            session
                .carrier()
                .unwrap()
                .observation(observation)
                .is_some()
        );
        let retained = session
            .carrier()
            .unwrap()
            .step(step.id)
            .expect("local Step is retained");
        assert_eq!(retained.reference().run, initial_run);
        assert_eq!(retained.reference().activation, initial_activation);
    }
    assert_eq!(session.configuration().unwrap()[0].as_number(), Some(5.0));
    let candidate = session
        .candidate()
        .unwrap()
        .expect("candidate is retained before Admission")
        .clone();
    let candidate_id = candidate.id;
    assert_eq!(candidate.base, facts.initial_state);
    let carrier_candidate = &session
        .carrier()
        .unwrap()
        .candidate_delta(candidate.id)
        .expect("candidate is retained by the semantic carrier")
        .proposal;
    assert_eq!(
        carrier_candidate.delta.term,
        carrier_candidate.proposed_payload
    );
    let first_candidate_formation = carrier_candidate.delta.evidence;
    let formation = session
        .carrier()
        .unwrap()
        .observation(first_candidate_formation)
        .expect("candidate cites its fresh Formation Observation");
    let ObservationContentV2::Formation { subject, .. } = &formation.content else {
        panic!("candidate evidence must be a Formation Observation");
    };
    assert_eq!(subject, &carrier_candidate.proposed_payload);
    assert_eq!(session.carrier().unwrap().candidate_delta_count(), 1);
    assert_eq!(session.carrier().unwrap().decision_count(), 0);
    assert_eq!(session.carrier().unwrap().state_revision_count(), 1);
    assert_eq!(session.world_base(), facts.initial_state);

    let rejected_record_count = session.carrier().unwrap().accepted_ingress_record_count();
    assert!(matches!(
        session.apply_opaque_input(&opaque(1, 1.0)),
        Err(PersistentProcessSessionErrorV1::Carrier(
            ExecutableCarrierErrorV1::Executable(ExecutableErrorV1::CandidateAlreadyEmitted)
        ))
    ));
    assert_eq!(
        session.carrier().unwrap().accepted_ingress_record_count(),
        rejected_record_count
    );

    let records_before_admission = session.carrier().unwrap().accepted_ingress_record_count();
    let unknown_issuance = id!(IssuedAdmissionAuthorizationOccurrenceId, 240);
    let missing_issuance = session
        .admit_issued_candidate_with_projection(unknown_issuance)
        .expect_err("an unissued occurrence cannot authorize Admission");
    assert!(matches!(
        missing_issuance,
        PersistentProcessSessionErrorV1::Carrier(ExecutableCarrierErrorV1::Ingress(
            ProcessIngressError::Record { cause, .. }
        )) if matches!(
            *cause,
            ProcessError::UnknownIssuedAdmissionAuthorization(actual)
                if actual == unknown_issuance
        )
    ));
    let issued_authorization = session
        .issue_candidate_admission_authorization()
        .expect("the constituted issuer emits one exact candidate-scoped occurrence");
    assert!(matches!(
        session.issue_candidate_admission_authorization(),
        Err(PersistentProcessSessionErrorV1::Carrier(
            ExecutableCarrierErrorV1::AdmissionAuthorizationAlreadyIssued
        ))
    ));
    let successor = session
        .admit_issued_candidate_with_projection(issued_authorization)
        .map(|(successor, _)| successor)
        .expect("Judgment, Admission, and successor epoch enter atomically");
    assert_eq!(
        session.carrier().unwrap().accepted_ingress_record_count(),
        records_before_admission + 6
    );
    assert_eq!(successor.predecessor, facts.initial_state);
    assert_ne!(successor.id, facts.initial_state);
    assert_eq!(session.world_base(), successor.id);
    assert_eq!(session.runtime_session(), session_id);
    assert_eq!(session.package().unwrap(), package_id);
    assert!(session.candidate().unwrap().is_none());
    assert_eq!(session.carrier().unwrap().decision_count(), 1);
    assert_eq!(session.carrier().unwrap().state_revision_count(), 2);

    let next_run = session.run().unwrap();
    let next_activation = session.activation().unwrap();
    assert_ne!(next_run, initial_run);
    assert_ne!(next_activation, initial_activation);
    let retained_activation = session
        .carrier()
        .unwrap()
        .activation(next_activation)
        .expect("successor-pinned Activation is retained");
    assert_eq!(
        retained_activation.membership(),
        RunMembership::RootOf(next_run)
    );
    assert_eq!(retained_activation.pins().runtime_session, Some(session_id));
    assert_eq!(
        retained_activation.pins().observed_state,
        Some(successor.id)
    );
    assert!(
        retained_activation
            .start_causes()
            .iter()
            .any(|cause| matches!(cause, CausalRef::Admission(_)))
    );
    let successor_formation = retained_activation
        .start_causes()
        .iter()
        .find_map(|cause| match cause {
            CausalRef::Observation(observation) => Some(*observation),
            _ => None,
        })
        .expect("successor Activation is bound to fresh admitted-state Formation");
    assert_ne!(successor_formation, first_candidate_formation);

    let next_tick = session
        .apply_opaque_input(&opaque(1, 1.0))
        .expect("next local tick uses the admitted-root epoch");
    let retained_tick = session
        .carrier()
        .unwrap()
        .step(next_tick.id)
        .expect("next-epoch tick is retained");
    assert_eq!(retained_tick.reference().run, next_run);
    assert_eq!(retained_tick.reference().activation, next_activation);
    assert_eq!(session.configuration().unwrap()[0].as_number(), Some(6.0));

    session
        .apply_opaque_input_and_emit_candidate(&opaque(1, 1.0))
        .expect("second epoch emits a distinct candidate");
    let second_candidate = session.candidate().unwrap().unwrap().clone();
    assert_ne!(second_candidate.id, candidate_id);
    let second_carrier_candidate = &session
        .carrier()
        .unwrap()
        .candidate_delta(second_candidate.id)
        .expect("second candidate is retained")
        .proposal;
    assert_eq!(
        second_carrier_candidate.delta.term,
        second_carrier_candidate.proposed_payload
    );
    assert_ne!(
        second_carrier_candidate.delta.evidence,
        first_candidate_formation
    );
    let records_before_replay = session.carrier().unwrap().accepted_ingress_record_count();
    let replayed_authorization = session
        .admit_issued_candidate_with_projection(issued_authorization)
        .expect_err("an issued occurrence is single-use across successor epochs");
    assert!(matches!(
        replayed_authorization,
        PersistentProcessSessionErrorV1::Carrier(ExecutableCarrierErrorV1::Ingress(
            ProcessIngressError::Record { cause, .. }
        )) if matches!(
            *cause,
            ProcessError::AdmissionAuthorizationAlreadyConsumed(actual)
                if actual == issued_authorization
        )
    ));
    assert_eq!(
        session.carrier().unwrap().accepted_ingress_record_count(),
        records_before_replay
    );
    assert_eq!(
        session.candidate().unwrap().unwrap().id,
        second_candidate.id
    );
    let (second_policy, second_authorization) = admission_policy(
        package_id,
        session_id,
        successor.id,
        second_candidate.id,
        131,
    );
    session
        .establish_root_policy(second_policy)
        .expect("second exact candidate authority is established separately");
    let second_successor = session
        .admit_candidate(second_authorization)
        .expect("second epoch requires and accepts fresh exact authority");
    assert_eq!(second_successor.predecessor, successor.id);
    assert_eq!(session.world_base(), second_successor.id);
    assert_eq!(session.carrier().unwrap().decision_count(), 2);

    assert!(session.dispose());
    assert!(!session.dispose());
    assert!(session.is_disposed());
    assert!(matches!(
        session.apply_opaque_input(&opaque(1, 1.0)),
        Err(PersistentProcessSessionErrorV1::Disposed)
    ));
    assert!(matches!(
        session.carrier(),
        Err(PersistentProcessSessionErrorV1::Disposed)
    ));
}

#[test]
fn admitted_frontier_compaction_bounds_history_and_preserves_execution() {
    let package = checked_program_package();
    let application = application(&package);
    let physical_plan = physical_plan(&package);
    let (authority, facts) = carrier_authority(&package);
    let mut session = PersistentProcessSessionV1::open(
        package,
        authority,
        application,
        physical_plan,
        facts.executable,
    )
    .expect("bounded-history session opens");
    let mut prior = facts.initial_state;
    let mut accepted_records = 0;

    for admission_index in 1..=64 {
        session
            .apply_opaque_input_and_emit_candidate(&opaque(1, 1.0))
            .expect("the next admitted epoch emits a Candidate");
        let authorization = session
            .issue_candidate_admission_authorization()
            .expect("the exact current Candidate receives Admission authority");
        let (successor, _) = session
            .admit_issued_candidate_with_projection(authorization)
            .expect("the current Candidate is admitted");
        assert_eq!(successor.predecessor, prior);
        prior = successor.id;

        let before_compaction = session
            .carrier()
            .expect("live Carrier remains available")
            .accepted_ingress_record_count();
        assert!(before_compaction > accepted_records);
        accepted_records = before_compaction;
        session
            .compact_to_admitted_frontier()
            .expect("completed decoded trace compacts to the live frontier");

        let carrier = session.carrier().expect("compacted Carrier remains live");
        assert_eq!(carrier.state_revision_count(), 1);
        assert_eq!(carrier.decision_count(), 1);
        assert_eq!(carrier.run_count(), 0);
        assert_eq!(carrier.activation_count(), 0);
        assert_eq!(carrier.step_count(), 0);
        assert_eq!(carrier.observation_count(), 0);
        assert_eq!(carrier.candidate_delta_count(), 0);
        assert_eq!(carrier.resource_usage().resident_ingress_records, 1);
        assert_eq!(
            carrier.resource_usage().accepted_ingress_records,
            accepted_records,
        );
        assert_eq!(session.world_base(), successor.id);
        assert_eq!(
            session.configuration().expect("configuration remains live")[0].as_number(),
            Some(f64::from(admission_index)),
        );
    }
}

#[test]
fn failed_admission_rolls_back_its_prepared_judgment_and_epoch() {
    let package = checked_program_package();
    let application = application(&package);
    let physical_plan = physical_plan(&package);
    let session_id = id!(RuntimeSessionId, 120);
    let package_id = package.id();
    let (authority, facts) = carrier_authority(&package);
    let mut session = PersistentProcessSessionV1::open(
        package,
        authority,
        application,
        physical_plan,
        facts.executable,
    )
    .expect("session opens before the separately checked Admission boundary");
    session
        .apply_opaque_input_and_emit_candidate(&opaque(1, 1.0))
        .expect("candidate remains non-authoritative");
    let actual_candidate = session.candidate().unwrap().unwrap().id;
    let unauthorized_candidate = id!(CandidateDeltaId, 250);
    assert_ne!(actual_candidate, unauthorized_candidate);
    let run = session.run().unwrap();
    let activation = session.activation().unwrap();
    let records = session.carrier().unwrap().accepted_ingress_record_count();
    let runs = session.carrier().unwrap().run_count();
    let (wrong_policy, wrong_authorization) = admission_policy(
        package_id,
        session_id,
        facts.initial_state,
        unauthorized_candidate,
        132,
    );
    session
        .establish_root_policy(wrong_policy)
        .expect("wrong candidate grant is still a coherent root policy");

    assert!(matches!(
        session.admit_candidate(wrong_authorization),
        Err(PersistentProcessSessionErrorV1::Carrier(
            ExecutableCarrierErrorV1::Ingress(_)
        ))
    ));
    assert_eq!(
        session.carrier().unwrap().accepted_ingress_record_count(),
        records
    );
    assert_eq!(session.carrier().unwrap().run_count(), runs);
    assert_eq!(session.carrier().unwrap().decision_count(), 0);
    assert_eq!(session.carrier().unwrap().state_revision_count(), 1);
    assert_eq!(session.run().unwrap(), run);
    assert_eq!(session.activation().unwrap(), activation);
    assert_eq!(session.world_base(), facts.initial_state);
    assert!(session.candidate().unwrap().is_some());
}
