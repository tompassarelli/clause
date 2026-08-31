use clause_package::*;
use clause_runtime::{ProcessRuntime, RuntimeInitError};

fn raw_id(tag: u8) -> [u8; IDENTITY_BYTES] {
    let mut bytes = [0; IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[IDENTITY_BYTES - 1] = tag;
    bytes
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
    assert_eq!(digits.len() % 2, 0);
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

fn checked_fixture(path: &str) -> CheckedProcessPackage {
    let source = match path {
        "core" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/positive/process-v2-core.hex"
        )),
        "handoff" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/negative/linear-double-takeup.hex"
        )),
        _ => panic!("unknown test fixture"),
    };
    let decoded =
        decode_process_package(&decode_hex(source)).expect("fixture has canonical wire bytes");
    if path != "core" {
        return check_process_package(decoded).expect("fixture constitution checks");
    }

    check_process_package(decoded).expect("core fixture constitution checks")
}

fn core_authority(checked: &CheckedProcessPackage) -> AuthorityStore {
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
        .expect("core package has one initial State view");
    assert_eq!(initial_view.session, session);
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
                vec![
                    RootStateAdmissionGrant {
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
                    },
                    RootStateAdmissionGrant {
                        authorization: RootAdmissionAuthorizationRef {
                            policy: root_policy,
                            local: AdmissionAuthorizationLocalId::new(2),
                        },
                        scope: CheckedStateAdmissionScope {
                            package: checked.id(),
                            session,
                            base: initial_state,
                            delta: id!(CandidateDeltaId, 81),
                        },
                    },
                ],
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
        .expect("root authority admits the checked snapshot");
    authority
        .establish_runtime_session(session_anchor)
        .expect("runtime session is established once");

    let pure_boundary = id!(BoundaryRef, 126);
    let state_boundary = id!(BoundaryRef, 127);
    let boundary_target = checked.constitution().preimage().formations[0]
        .target
        .clone();
    let admitted = CheckedConstitutionBinding::Admitted {
        revision: revision.id,
    };
    let pure_pins = BoundaryPins {
        semantics,
        snapshot,
        constitution: admitted,
        runtime_session: None,
        observed_state: None,
        runtime_policy: None,
    };
    let state_pins = BoundaryPins {
        runtime_session: Some(session),
        observed_state: Some(initial_state),
        runtime_policy: Some(policy),
        ..pure_pins
    };
    let exactly_one = CardinalityV2 {
        minimum: 1,
        maximum: Some(1),
    };
    let at_most_one = CardinalityV2 {
        minimum: 0,
        maximum: Some(1),
    };
    let repeatable = BoundaryReplayPolicyV2::Repeatable {
        maximum_occurrences: None,
    };
    authority
        .establish_boundary(BoundaryAnchor {
            boundary: pure_boundary,
            permissions: vec![
                BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(0),
                    kind: EnteredOccurrenceKind::ExternalTrigger,
                    payload: boundary_target.clone(),
                    pins: pure_pins,
                    cause_schema: vec![],
                    support_schema: vec![],
                    replay: repeatable,
                },
                BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(1),
                    kind: EnteredOccurrenceKind::Observation,
                    payload: boundary_target.clone(),
                    pins: pure_pins,
                    cause_schema: vec![
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::ExternalTrigger,
                            cardinality: at_most_one,
                        },
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::Step,
                            cardinality: at_most_one,
                        },
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::Admission,
                            cardinality: at_most_one,
                        },
                    ],
                    support_schema: vec![],
                    replay: repeatable,
                },
                BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(2),
                    kind: EnteredOccurrenceKind::Resumption,
                    payload: boundary_target.clone(),
                    pins: pure_pins,
                    cause_schema: vec![BoundaryCauseRequirementV2 {
                        kind: EnteredCauseKindV2::Observation,
                        cardinality: exactly_one,
                    }],
                    support_schema: vec![],
                    replay: repeatable,
                },
                BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(3),
                    kind: EnteredOccurrenceKind::Handoff,
                    payload: boundary_target.clone(),
                    pins: pure_pins,
                    cause_schema: vec![BoundaryCauseRequirementV2 {
                        kind: EnteredCauseKindV2::Step,
                        cardinality: exactly_one,
                    }],
                    support_schema: vec![],
                    replay: repeatable,
                },
                BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(4),
                    kind: EnteredOccurrenceKind::Cancellation,
                    payload: boundary_target.clone(),
                    pins: pure_pins,
                    cause_schema: vec![
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::ExternalTrigger,
                            cardinality: at_most_one,
                        },
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::Step,
                            cardinality: at_most_one,
                        },
                    ],
                    support_schema: vec![],
                    replay: repeatable,
                },
            ],
        })
        .expect("pure boundary is established once");
    authority
        .establish_boundary(BoundaryAnchor {
            boundary: state_boundary,
            permissions: vec![
                BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(5),
                    kind: EnteredOccurrenceKind::Judgment,
                    payload: boundary_target.clone(),
                    pins: state_pins,
                    cause_schema: vec![BoundaryCauseRequirementV2 {
                        kind: EnteredCauseKindV2::CandidateDelta,
                        cardinality: exactly_one,
                    }],
                    support_schema: vec![],
                    replay: repeatable,
                },
                BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(6),
                    kind: EnteredOccurrenceKind::AdmissionDecision,
                    payload: boundary_target,
                    pins: state_pins,
                    cause_schema: vec![
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::CandidateDelta,
                            cardinality: exactly_one,
                        },
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::Judgment,
                            cardinality: CardinalityV2 {
                                minimum: 1,
                                maximum: None,
                            },
                        },
                    ],
                    support_schema: vec![],
                    replay: repeatable,
                },
            ],
        })
        .expect("state boundary is established once");
    for (tag, boundary) in [
        (181, pure_boundary),
        (182, pure_boundary),
        (183, pure_boundary),
        (184, pure_boundary),
        (185, pure_boundary),
        (186, state_boundary),
        (187, state_boundary),
        (188, state_boundary),
        (189, state_boundary),
        (190, state_boundary),
        (191, state_boundary),
    ] {
        authority
            .establish_evidence(EvidenceAnchor {
                evidence: id!(ExternalEvidenceRef, tag),
                boundary,
                permissions: vec![BoundaryPermissionLocalId::new(match tag {
                    181..=183 => 0,
                    184 => 1,
                    185 => 2,
                    186..=189 => 5,
                    190..=191 => 6,
                    _ => unreachable!("fixture evidence tag is closed"),
                })],
                exact_evidence: vec![tag].into_boxed_slice(),
            })
            .expect("external evidence is established once");
    }
    authority
}

#[test]
fn checked_package_advances_serial_process_and_governed_state_end_to_end() {
    let checked = checked_fixture("core");
    let authority = core_authority(&checked);
    let mut runtime = ProcessRuntime::instantiate(checked, authority)
        .expect("supported checked package instantiates");
    assert_eq!(runtime.carrier().state_revision_count(), 1);
    assert_eq!(runtime.carrier().applied_package_record_count(), 0);

    let mut saw_root = false;
    let mut saw_suspension = false;
    let mut saw_return = false;
    let mut saw_candidate_before_judgment = false;
    let mut saw_judgment_before_admission = false;
    let mut saw_successor = false;
    while let Some(record) = runtime.advance().expect("next package record is accepted") {
        match record {
            ProcessRecordV2::Activation(proposal) if proposal.id == id!(ActivationId, 20) => {
                assert_eq!(proposal.membership, RunMembership::RootOf(id!(RunId, 30)));
                assert_eq!(
                    runtime.carrier().run_root(id!(RunId, 30)),
                    Some(&proposal.id)
                );
                saw_root = true;
            }
            ProcessRecordV2::Steps(steps) if steps[0].id == id!(StepId, 51) => {
                assert_eq!(
                    runtime
                        .carrier()
                        .activation(id!(ActivationId, 20))
                        .expect("Activation exists")
                        .status(),
                    ActivationStatus::Suspended(id!(ContinuationId, 70))
                );
                assert_eq!(
                    runtime
                        .carrier()
                        .configuration(id!(ConfigurationId, 61))
                        .expect("successor Configuration exists")
                        .predecessor,
                    ConfigurationPredecessorV2::ConfigurationAfter(StepRef {
                        run: id!(RunId, 30),
                        activation: id!(ActivationId, 20),
                        step: id!(StepId, 51),
                    })
                );
                saw_suspension = true;
            }
            ProcessRecordV2::Steps(steps) if steps[0].id == id!(StepId, 56) => {
                assert!(
                    runtime
                        .carrier()
                        .candidate_delta(id!(CandidateDeltaId, 80))
                        .is_some()
                );
                assert_eq!(runtime.carrier().state_revision_count(), 1);
                saw_candidate_before_judgment = true;
            }
            ProcessRecordV2::Steps(steps) if steps[0].id == id!(StepId, 58) => {
                assert_eq!(
                    runtime
                        .carrier()
                        .activation(id!(ActivationId, 20))
                        .expect("Activation exists")
                        .status(),
                    ActivationStatus::Terminal(ActivationTerminal::Returned)
                );
                saw_return = true;
            }
            ProcessRecordV2::Judgment(judgment)
                if judgment.body.id == id!(JudgmentOccurrenceId, 90) =>
            {
                assert_eq!(runtime.carrier().state_revision_count(), 1);
                saw_judgment_before_admission = true;
            }
            ProcessRecordV2::AdmissionDecision(decision)
                if decision.occurrence == id!(AdmissionOccurrenceId, 94) =>
            {
                let StateAdmissionOutcomeV2::Admit(successor) = decision.outcome else {
                    panic!("Admission 94 must admit its successor");
                };
                assert!(runtime.carrier().state_revision(successor.id).is_some());
                assert_eq!(runtime.carrier().state_revision_count(), 2);
                saw_successor = true;
            }
            _ => {}
        }
    }

    assert!(saw_root);
    assert!(saw_suspension);
    assert!(saw_return);
    assert!(saw_candidate_before_judgment);
    assert!(saw_judgment_before_admission);
    assert!(saw_successor);
    assert!(runtime.is_complete());
    assert_eq!(runtime.carrier().step_count(), 9);
    assert_eq!(runtime.carrier().decision_count(), 2);
    assert_eq!(runtime.carrier().state_revision_count(), 2);

    let unsupported = checked_fixture("handoff");
    let no_authority = AuthorityStore::new();
    assert_eq!(
        ProcessRuntime::instantiate(unsupported, no_authority).err(),
        Some(RuntimeInitError::Carrier(ProcessError::HandoffUnsupported))
    );
}
