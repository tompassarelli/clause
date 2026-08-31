use clause_package::*;
use clause_runtime::*;

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

fn executable_program() -> ExecutableProgramV1 {
    ExecutableProgramV1 {
        initial_configuration: vec![number(0.0)],
        rules: vec![
            ExecutableRuleV1 {
                entry: 0,
                predicates: vec![],
                assignments: vec![(0, ExecutableExpressionV1::Argument(0))],
            },
            ExecutableRuleV1 {
                entry: 1,
                predicates: vec![],
                assignments: vec![(
                    0,
                    ExecutableExpressionV1::Add(
                        Box::new(ExecutableExpressionV1::Slot(0)),
                        Box::new(ExecutableExpressionV1::Argument(0)),
                    ),
                )],
            },
        ],
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

    let term = executable_program()
        .encode_term(TermScope {
            universe: candidate.snapshot.constitution.universe,
            semantics: candidate.snapshot.constitution.semantics,
        })
        .expect("program encodes as an exact Term");
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
    }
    candidate.claimed_snapshot =
        derive_program_snapshot_id(&candidate.snapshot).expect("snapshot is canonical");
    let bytes = encode_process_package(&candidate).expect("program package encodes");
    check_process_package(decode_process_package(&bytes).expect("program package decodes"))
        .expect("program package checks")
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
    authority
        .establish_boundary(BoundaryAnchor {
            boundary: occurrence_boundary,
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
        .expect("occurrence boundary is established once");
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
    let occurrence_evidence = id!(ExternalEvidenceRef, 181);
    let judgment_evidence = id!(ExternalEvidenceRef, 186);
    let admission_evidence = id!(ExternalEvidenceRef, 190);
    for (evidence, boundary, bytes) in [
        (occurrence_evidence, occurrence_boundary, vec![181]),
        (judgment_evidence, state_boundary, vec![186]),
        (admission_evidence, state_boundary, vec![190]),
    ] {
        authority
            .establish_evidence(EvidenceAnchor {
                evidence,
                boundary,
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
                occurrence_ingress: ExecutableBoundaryFactV1 {
                    boundary: occurrence_boundary,
                    evidence: occurrence_evidence,
                },
                judgment_ingress: ExecutableBoundaryFactV1 {
                    boundary: state_boundary,
                    evidence: judgment_evidence,
                },
                admission_ingress: ExecutableBoundaryFactV1 {
                    boundary: state_boundary,
                    evidence: admission_evidence,
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

#[test]
fn persistent_session_keeps_local_steps_and_advances_only_at_atomic_admission() {
    let package = checked_program_package();
    let package_id = package.id();
    let application = application(&package);
    let session_id = id!(RuntimeSessionId, 120);
    let candidate_id = PersistentProcessSessionV1::candidate_id_for(session_id, 0);
    let (authority, facts) = carrier_authority(&package);
    let mut session =
        PersistentProcessSessionV1::open(package, authority, application, facts.executable)
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
    assert_eq!(candidate.id, candidate_id);
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
    let (first_policy, first_authorization) = admission_policy(
        package_id,
        session_id,
        facts.initial_state,
        candidate.id,
        130,
    );
    session
        .establish_root_policy(first_policy)
        .expect("first exact candidate authority is established");
    let successor = session
        .admit_candidate(first_authorization)
        .expect("Judgment, Admission, and successor epoch enter atomically");
    assert_eq!(
        session.carrier().unwrap().accepted_ingress_record_count(),
        records_before_admission + 5
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
    assert_eq!(
        second_candidate.id,
        PersistentProcessSessionV1::candidate_id_for(session_id, 1)
    );
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
fn failed_admission_rolls_back_its_prepared_judgment_and_epoch() {
    let package = checked_program_package();
    let application = application(&package);
    let session_id = id!(RuntimeSessionId, 120);
    let actual_candidate = PersistentProcessSessionV1::candidate_id_for(session_id, 0);
    let unauthorized_candidate = PersistentProcessSessionV1::candidate_id_for(session_id, 1);
    assert_ne!(actual_candidate, unauthorized_candidate);
    let package_id = package.id();
    let (authority, facts) = carrier_authority(&package);
    let mut session =
        PersistentProcessSessionV1::open(package, authority, application, facts.executable)
            .expect("session opens before the separately checked Admission boundary");
    session
        .apply_opaque_input_and_emit_candidate(&opaque(1, 1.0))
        .expect("candidate remains non-authoritative");
    assert_eq!(session.candidate().unwrap().unwrap().id, actual_candidate);
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
