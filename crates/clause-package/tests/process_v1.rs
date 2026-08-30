use clause_package::*;

fn raw_id(tag: u8) -> [u8; IDENTITY_BYTES] {
    let mut bytes = [0; IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[IDENTITY_BYTES - 1] = tag;
    bytes
}

macro_rules! id {
    ($type:ident, $tag:expr) => {
        $type::from_bytes(raw_id($tag))
    };
}

fn term(payload: &str) -> Term {
    Term::atom(
        b"process-v1/test".to_vec(),
        payload.as_bytes().to_vec(),
        b"octets/exact/v1".to_vec(),
    )
    .expect("test Atom is canonical")
}

fn snapshot() -> ProgramSnapshotId {
    id!(ProgramSnapshotId, 2)
}

fn schema() -> RelationSchemaId {
    RelationSchemaId {
        snapshot: snapshot(),
        local: RelationSchemaLocalId::new(1),
    }
}

fn role(local: u32) -> RoleId {
    RoleId {
        schema: schema(),
        local: RoleLocalId::new(local),
    }
}

fn operator() -> OperatorRef {
    OperatorRef {
        snapshot: snapshot(),
        local: OperatorLocalId::new(1),
    }
}

fn mode(local: u32) -> ModeId {
    ModeId {
        operator: operator(),
        local: ModeLocalId::new(local),
    }
}

fn application_id() -> ApplicationId {
    ApplicationId {
        snapshot: snapshot(),
        local: ApplicationLocalId::new(1),
    }
}

fn program_revision() -> ProgramRevisionId {
    id!(ProgramRevisionId, 4)
}

fn runtime_session() -> RuntimeSessionId {
    id!(RuntimeSessionId, 6)
}

fn runtime_policy() -> RuntimePolicyId {
    id!(RuntimePolicyId, 7)
}

fn initial_state() -> StateRevisionId {
    id!(StateRevisionId, 8)
}

fn execution_authorization(local: u32) -> ExecutionAuthorizationRef {
    ExecutionAuthorizationRef {
        snapshot: snapshot(),
        local: ExecutionAuthorizationLocalId::new(local),
    }
}

fn admission_authorization() -> AdmissionAuthorizationRef {
    AdmissionAuthorizationRef {
        snapshot: snapshot(),
        local: AdmissionAuthorizationLocalId::new(1),
    }
}

fn base_vector() -> ProcessVector {
    let semantics = id!(ClauseSemanticsId, 1);
    let session_start = id!(SessionStartOccurrenceId, 9);
    ProcessVector {
        constitution: ProgramConstitutionCandidate {
            semantics,
            snapshot: snapshot(),
            schemas: vec![RelationSchemaDeclaration {
                id: schema(),
                roles: vec![role(1), role(2)],
            }],
            operators: vec![OperatorDeclaration {
                id: operator(),
                modes: vec![
                    ModeDeclaration {
                        id: mode(1),
                        schema: schema(),
                        known_roles: vec![role(1)],
                        produced_roles: vec![role(2)],
                        context_requirements: vec![],
                        state_contract: ModeStateContract::Pure,
                        may_suspend: true,
                        may_cancel: true,
                    },
                    ModeDeclaration {
                        id: mode(2),
                        schema: schema(),
                        known_roles: vec![role(1)],
                        produced_roles: vec![role(2)],
                        context_requirements: vec![],
                        state_contract: ModeStateContract::ProposesState,
                        may_suspend: true,
                        may_cancel: true,
                    },
                ],
            }],
        },
        program_revisions: vec![AuthoritativeProgramRevision {
            id: program_revision(),
            program: id!(ProgramId, 3),
            snapshot: snapshot(),
            semantics,
            predecessor: None,
            change: id!(ProgramChangeOccurrenceId, 5),
            execution_authorizations: vec![
                ProgramExecutionAuthorization {
                    reference: execution_authorization(1),
                    scope: ExecutionScope {
                        application: application_id(),
                        mode: mode(1),
                    },
                },
                ProgramExecutionAuthorization {
                    reference: execution_authorization(2),
                    scope: ExecutionScope {
                        application: application_id(),
                        mode: mode(2),
                    },
                },
            ],
            admission_authorizations: vec![ProgramAdmissionAuthorization {
                reference: admission_authorization(),
                scope: AdmissionScope {
                    session: runtime_session(),
                },
            }],
        }],
        root_policies: vec![],
        sessions: vec![RuntimeSession {
            id: runtime_session(),
            program_revision: program_revision(),
            semantics,
            policy: runtime_policy(),
            start: session_start,
            initial_state: initial_state(),
        }],
        initial_states: vec![StateRevision {
            id: initial_state(),
            session: runtime_session(),
            predecessor: None,
            cause: StateRevisionCause::SessionStart(session_start),
            payload: term("world/initial"),
            policy: runtime_policy(),
            semantics,
        }],
        records: vec![],
    }
}

fn application_record() -> ProcessRecord {
    ProcessRecord::Application(ApplicationProposal {
        id: application_id(),
        form: ApplicationFormCandidate {
            term: Term::raw_triple([term("operand"), term("engagement"), term("result")]),
            schema: schema(),
            operator: operator(),
            eligible_modes: vec![mode(1), mode(2)],
            bindings: vec![
                RoleBinding {
                    role: role(1),
                    value: RoleBindingValue::Known(term("input/41")),
                },
                RoleBinding {
                    role: role(2),
                    value: RoleBindingValue::Produced,
                },
            ],
            context_requirements: vec![],
        },
        allocation_authority: ApplicationAllocationAuthority::ProgramRevision(program_revision()),
    })
}

fn pure_pins() -> ActivationPins {
    ActivationPins {
        semantics: id!(ClauseSemanticsId, 1),
        snapshot: snapshot(),
        program_revision: program_revision(),
        runtime_session: None,
        observed_state: None,
        runtime_policy: None,
        budget: Budget {
            remaining_units: 100,
        },
    }
}

fn state_pins() -> ActivationPins {
    ActivationPins {
        semantics: id!(ClauseSemanticsId, 1),
        snapshot: snapshot(),
        program_revision: program_revision(),
        runtime_session: Some(runtime_session()),
        observed_state: Some(initial_state()),
        runtime_policy: Some(runtime_policy()),
        budget: Budget {
            remaining_units: 100,
        },
    }
}

fn activation_record(
    activation_tag: u8,
    run_tag: u8,
    configuration_tag: u8,
    selected_mode: u32,
) -> ProcessRecord {
    let pins = if selected_mode == 1 {
        pure_pins()
    } else {
        state_pins()
    };
    ProcessRecord::Activation(ActivationProposal {
        id: id!(ActivationId, activation_tag),
        application: application_id(),
        mode: mode(selected_mode),
        pins,
        causes: ActivationCauseFrontier {
            origin: ActivationOrigin::RootedBy(RootTrigger::External(id!(
                ExternalTriggerOccurrenceId,
                activation_tag + 1
            ))),
            authorization: ExecutionAuthorizationEvidence::ProgramConstitution {
                revision: program_revision(),
                authorization: execution_authorization(selected_mode),
            },
        },
        membership: RunMembership::RootOf(id!(RunId, run_tag)),
        initial_configuration: ConfigurationProposal {
            id: id!(ConfigurationId, configuration_tag),
            value: term("configuration/initial"),
        },
    })
}

fn continuation_pins(run_tag: u8, activation_tag: u8) -> ContinuationPins {
    ContinuationPins {
        run: id!(RunId, run_tag),
        activation: id!(ActivationId, activation_tag),
        application: application_id(),
        mode: mode(1),
        semantics: id!(ClauseSemanticsId, 1),
        snapshot: snapshot(),
        program_revision: program_revision(),
        runtime_session: None,
        observed_state: None,
        runtime_policy: None,
        remaining_budget: Budget {
            remaining_units: 100,
        },
    }
}

fn first_step(
    step_tag: u8,
    run_tag: u8,
    activation_tag: u8,
    before_tag: u8,
    after_tag: u8,
) -> StepProposal {
    StepProposal {
        id: id!(StepId, step_tag),
        run: id!(RunId, run_tag),
        activation: id!(ActivationId, activation_tag),
        before: id!(ConfigurationId, before_tag),
        after: ConfigurationProposal {
            id: id!(ConfigurationId, after_tag),
            value: term("configuration/after-first"),
        },
        observed_state: None,
        causes: vec![StepCause::ActivationStart(id!(
            ActivationId,
            activation_tag
        ))],
        observations: vec![],
        candidate_delta: None,
        outcome: StepOutcomeProposal::Progress,
    }
}

fn repeated_activation_vector() -> ProcessVector {
    let mut vector = base_vector();
    vector.records = vec![
        application_record(),
        activation_record(20, 30, 40, 1),
        activation_record(22, 32, 42, 1),
    ];
    vector
}

fn pure_completion_vector() -> ProcessVector {
    let mut vector = base_vector();
    let mut step = first_step(50, 30, 20, 40, 41);
    step.observations = vec![ObservationProposal {
        id: id!(ObservationId, 60),
        value: term("value/42"),
    }];
    step.outcome = StepOutcomeProposal::Return(term("value/42"));
    vector.records = vec![
        application_record(),
        activation_record(20, 30, 40, 1),
        ProcessRecord::Steps(vec![step]),
    ];
    vector
}

fn suspend_resume_vector() -> ProcessVector {
    let mut vector = base_vector();
    let activation_tag = 20;
    let run_tag = 30;
    let first = first_step(50, run_tag, activation_tag, 40, 41);
    let continuation = ContinuationProposal {
        id: id!(ContinuationId, 70),
        emitted_by: id!(StepId, 51),
        pins: continuation_pins(run_tag, activation_tag),
        remainder: term("continuation/remainder"),
        linear: true,
    };
    let suspend = StepProposal {
        id: id!(StepId, 51),
        run: id!(RunId, run_tag),
        activation: id!(ActivationId, activation_tag),
        before: id!(ConfigurationId, 41),
        after: ConfigurationProposal {
            id: id!(ConfigurationId, 42),
            value: term("configuration/suspended"),
        },
        observed_state: None,
        causes: vec![StepCause::PriorStep {
            run: id!(RunId, run_tag),
            activation: id!(ActivationId, activation_tag),
            step: id!(StepId, 50),
        }],
        observations: vec![],
        candidate_delta: None,
        outcome: StepOutcomeProposal::Suspend(continuation.clone()),
    };
    let resumption = ResumptionOccurrenceProposal {
        id: id!(ResumptionOccurrenceId, 71),
        continuation: continuation.id,
        run: id!(RunId, run_tag),
        activation: id!(ActivationId, activation_tag),
        pins: continuation.pins,
    };
    let resumed = StepProposal {
        id: id!(StepId, 52),
        run: id!(RunId, run_tag),
        activation: id!(ActivationId, activation_tag),
        before: id!(ConfigurationId, 42),
        after: ConfigurationProposal {
            id: id!(ConfigurationId, 43),
            value: term("configuration/returned"),
        },
        observed_state: None,
        causes: vec![
            StepCause::PriorStep {
                run: id!(RunId, run_tag),
                activation: id!(ActivationId, activation_tag),
                step: id!(StepId, 51),
            },
            StepCause::ContinuationTakeup {
                continuation: continuation.id,
                occurrence: ContinuationTakeupOccurrence::Resumption(resumption.id),
            },
        ],
        observations: vec![ObservationProposal {
            id: id!(ObservationId, 72),
            value: term("value/resumed"),
        }],
        candidate_delta: None,
        outcome: StepOutcomeProposal::Return(term("value/resumed")),
    };
    vector.records = vec![
        application_record(),
        activation_record(activation_tag, run_tag, 40, 1),
        ProcessRecord::Steps(vec![first]),
        ProcessRecord::Steps(vec![suspend]),
        ProcessRecord::Resumption(resumption),
        ProcessRecord::Steps(vec![resumed]),
    ];
    vector
}

fn candidate_delta_vector(admit: bool) -> ProcessVector {
    let mut vector = base_vector();
    let step = StepProposal {
        id: id!(StepId, 80),
        run: id!(RunId, 31),
        activation: id!(ActivationId, 21),
        before: id!(ConfigurationId, 44),
        after: ConfigurationProposal {
            id: id!(ConfigurationId, 45),
            value: term("configuration/candidate"),
        },
        observed_state: Some(initial_state()),
        causes: vec![StepCause::ActivationStart(id!(ActivationId, 21))],
        observations: vec![],
        candidate_delta: Some(CandidateDeltaProposal {
            id: id!(CandidateDeltaId, 81),
            base: initial_state(),
            proposed_payload: term("world/candidate"),
            evidence: vec![term("evidence/transition")],
        }),
        outcome: StepOutcomeProposal::Progress,
    };
    vector.records = vec![
        application_record(),
        activation_record(21, 31, 44, 2),
        ProcessRecord::Steps(vec![step]),
    ];
    if admit {
        vector
            .records
            .push(ProcessRecord::AdmitState(StateAdmissionProposal {
                occurrence: id!(AdmissionOccurrenceId, 82),
                delta: id!(CandidateDeltaId, 81),
                authorization: AdmissionAuthorizationEvidence::ProgramConstitution {
                    revision: program_revision(),
                    authorization: admission_authorization(),
                },
                successor: StateRevision {
                    id: id!(StateRevisionId, 83),
                    session: runtime_session(),
                    predecessor: Some(initial_state()),
                    cause: StateRevisionCause::Admission {
                        occurrence: id!(AdmissionOccurrenceId, 82),
                        run: id!(RunId, 31),
                        activation: id!(ActivationId, 21),
                        step: id!(StepId, 80),
                    },
                    payload: term("world/candidate"),
                    policy: runtime_policy(),
                    semantics: id!(ClauseSemanticsId, 1),
                },
            }));
    }
    vector
}

fn wrong_pins_vector() -> ProcessVector {
    let mut vector = suspend_resume_vector();
    vector.records.truncate(4);
    let mut pins = continuation_pins(30, 20);
    pins.program_revision = id!(ProgramRevisionId, 99);
    vector
        .records
        .push(ProcessRecord::Resumption(ResumptionOccurrenceProposal {
            id: id!(ResumptionOccurrenceId, 71),
            continuation: id!(ContinuationId, 70),
            run: id!(RunId, 30),
            activation: id!(ActivationId, 20),
            pins,
        }));
    vector
}

fn causal_negative_vector(kind: &str) -> ProcessVector {
    let mut vector = base_vector();
    let first = first_step(50, 30, 20, 40, 41);
    let proposal = |id_tag: u8, after_tag: u8, cause_tag: u8| StepProposal {
        id: id!(StepId, id_tag),
        run: id!(RunId, 30),
        activation: id!(ActivationId, 20),
        before: id!(ConfigurationId, 41),
        after: ConfigurationProposal {
            id: id!(ConfigurationId, after_tag),
            value: term("configuration/rejected"),
        },
        observed_state: None,
        causes: vec![StepCause::PriorStep {
            run: id!(RunId, 30),
            activation: id!(ActivationId, 20),
            step: id!(StepId, cause_tag),
        }],
        observations: vec![],
        candidate_delta: None,
        outcome: StepOutcomeProposal::Progress,
    };
    let rejected = match kind {
        "self" => vec![proposal(51, 42, 51)],
        "future" => vec![proposal(51, 42, 52), proposal(52, 43, 50)],
        "cycle" => vec![proposal(51, 42, 52), proposal(52, 43, 51)],
        other => panic!("unknown causal negative {other}"),
    };
    vector.records = vec![
        application_record(),
        activation_record(20, 30, 40, 1),
        ProcessRecord::Steps(vec![first]),
        ProcessRecord::Steps(rejected),
    ];
    vector
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2 + bytes.len() / 32 + 1);
    for (index, byte) in bytes.iter().enumerate() {
        if index != 0 && index % 32 == 0 {
            output.push('\n');
        }
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output.push('\n');
    output
}

fn frozen_hex(source: &str) -> Vec<u8> {
    let digits: Vec<u8> = source
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    assert_eq!(digits.len() % 2, 0, "hex transport has complete octets");
    digits
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).expect("hex digit") as u8;
            let low = (pair[1] as char).to_digit(16).expect("hex digit") as u8;
            (high << 4) | low
        })
        .collect()
}

struct FrozenVector {
    bytes: Vec<u8>,
    expected: ProcessVector,
    verdict: Result<(), ProcessError>,
}

fn frozen_vectors() -> Vec<FrozenVector> {
    vec![
        FrozenVector {
            bytes: frozen_hex(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-vectors/process-v1/positive/repeated-activation.hex"
            ))),
            expected: repeated_activation_vector(),
            verdict: Ok(()),
        },
        FrozenVector {
            bytes: frozen_hex(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-vectors/process-v1/positive/pure-completion.hex"
            ))),
            expected: pure_completion_vector(),
            verdict: Ok(()),
        },
        FrozenVector {
            bytes: frozen_hex(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-vectors/process-v1/positive/suspend-resume.hex"
            ))),
            expected: suspend_resume_vector(),
            verdict: Ok(()),
        },
        FrozenVector {
            bytes: frozen_hex(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-vectors/process-v1/positive/candidate-delta.hex"
            ))),
            expected: candidate_delta_vector(false),
            verdict: Ok(()),
        },
        FrozenVector {
            bytes: frozen_hex(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-vectors/process-v1/positive/state-admission.hex"
            ))),
            expected: candidate_delta_vector(true),
            verdict: Ok(()),
        },
        FrozenVector {
            bytes: frozen_hex(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-vectors/process-v1/negative/wrong-pins.hex"
            ))),
            expected: wrong_pins_vector(),
            verdict: Err(ProcessError::ContinuationPinMismatch),
        },
        FrozenVector {
            bytes: frozen_hex(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-vectors/process-v1/negative/self-cause.hex"
            ))),
            expected: causal_negative_vector("self"),
            verdict: Err(ProcessError::SelfStepCause(id!(StepId, 51))),
        },
        FrozenVector {
            bytes: frozen_hex(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-vectors/process-v1/negative/future-cause.hex"
            ))),
            expected: causal_negative_vector("future"),
            verdict: Err(ProcessError::FutureStepCause(id!(StepId, 52))),
        },
        FrozenVector {
            bytes: frozen_hex(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../test-vectors/process-v1/negative/cycle.hex"
            ))),
            expected: causal_negative_vector("cycle"),
            verdict: Err(ProcessError::CausalCycle(id!(StepId, 51))),
        },
    ]
}

#[test]
fn one_application_has_two_distinct_root_activations() {
    let carrier = ProcessCarrier::replay(&repeated_activation_vector()).expect("vector accepts");
    assert_eq!(carrier.application_count(), 1);
    assert_eq!(carrier.activation_count(), 2);
    assert_eq!(carrier.run_count(), 2);
    assert_ne!(
        carrier
            .activation(id!(ActivationId, 20))
            .expect("first activation")
            .id(),
        carrier
            .activation(id!(ActivationId, 22))
            .expect("second activation")
            .id()
    );
}

#[test]
fn one_activation_progresses_suspends_and_resumes_across_steps() {
    let carrier = ProcessCarrier::replay(&suspend_resume_vector()).expect("vector accepts");
    assert_eq!(carrier.activation_count(), 1);
    assert_eq!(carrier.run_count(), 1);
    assert_eq!(carrier.step_count(), 3);
    assert!(
        carrier
            .continuation(id!(ContinuationId, 70))
            .expect("continuation exists")
            .consumed()
    );
}

#[test]
fn pure_completion_allocates_no_state_revision() {
    let vector = pure_completion_vector();
    let before = vector.initial_states.len();
    let carrier = ProcessCarrier::replay(&vector).expect("vector accepts");
    assert_eq!(carrier.state_revision_count(), before);
    assert_eq!(carrier.admission_count(), 0);
}

#[test]
fn candidate_delta_remains_non_authoritative_until_admission() {
    let candidate =
        ProcessCarrier::replay(&candidate_delta_vector(false)).expect("candidate accepts");
    assert!(
        candidate
            .candidate_delta(id!(CandidateDeltaId, 81))
            .is_some()
    );
    assert_eq!(candidate.state_revision_count(), 1);
    assert_eq!(candidate.admission_count(), 0);
    assert!(candidate.state_revision(id!(StateRevisionId, 83)).is_none());

    let admitted =
        ProcessCarrier::replay(&candidate_delta_vector(true)).expect("admission accepts");
    assert_eq!(admitted.state_revision_count(), 2);
    assert_eq!(admitted.admission_count(), 1);
    assert!(admitted.state_revision(id!(StateRevisionId, 83)).is_some());
}

#[test]
fn wrong_continuation_pins_reject_before_step_allocation() {
    assert_eq!(
        ProcessCarrier::replay(&wrong_pins_vector()).unwrap_err(),
        ProcessError::ContinuationPinMismatch
    );
}

#[test]
fn self_future_and_indirect_cycle_causes_reject_transactionally() {
    assert!(matches!(
        ProcessCarrier::replay(&causal_negative_vector("self")),
        Err(ProcessError::SelfStepCause(id)) if id == id!(StepId, 51)
    ));
    assert!(matches!(
        ProcessCarrier::replay(&causal_negative_vector("future")),
        Err(ProcessError::FutureStepCause(id)) if id == id!(StepId, 52)
    ));
    assert!(matches!(
        ProcessCarrier::replay(&causal_negative_vector("cycle")),
        Err(ProcessError::CausalCycle(_))
    ));
}

#[test]
fn canonical_codec_round_trips_without_granting_authority() {
    for vector in [
        repeated_activation_vector(),
        pure_completion_vector(),
        suspend_resume_vector(),
        candidate_delta_vector(false),
        candidate_delta_vector(true),
        wrong_pins_vector(),
        causal_negative_vector("self"),
        causal_negative_vector("future"),
        causal_negative_vector("cycle"),
    ] {
        let bytes = encode_process_vector(&vector).expect("canonical encode");
        let decoded = decode_process_vector(&bytes).expect("strict decode");
        assert_eq!(decoded.exact_bytes(), bytes);
        assert_eq!(decoded.vector(), &vector);
    }
}

#[test]
fn frozen_corpus_binds_exact_bytes_and_replays_exact_verdicts() {
    for frozen in frozen_vectors() {
        let decoded = decode_process_vector(&frozen.bytes).expect("frozen bytes decode");
        assert_eq!(decoded.exact_bytes(), frozen.bytes);
        assert_eq!(decoded.vector(), &frozen.expected);
        assert_eq!(
            ProcessCarrier::replay(decoded.vector()).map(|_| ()),
            frozen.verdict
        );
        assert_eq!(
            encode_process_vector(decoded.vector()).expect("frozen value reencodes"),
            frozen.bytes
        );
    }
}

#[test]
fn print_frozen_vectors() {
    let requested = std::env::var("CLAUSE_VECTOR").ok();
    for (name, vector) in [
        (
            "positive/repeated-activation.hex",
            repeated_activation_vector(),
        ),
        ("positive/pure-completion.hex", pure_completion_vector()),
        ("positive/suspend-resume.hex", suspend_resume_vector()),
        (
            "positive/candidate-delta.hex",
            candidate_delta_vector(false),
        ),
        ("positive/state-admission.hex", candidate_delta_vector(true)),
        ("negative/wrong-pins.hex", wrong_pins_vector()),
        ("negative/self-cause.hex", causal_negative_vector("self")),
        (
            "negative/future-cause.hex",
            causal_negative_vector("future"),
        ),
        ("negative/cycle.hex", causal_negative_vector("cycle")),
    ] {
        if requested.as_deref().is_some_and(|value| value != name) {
            continue;
        }
        let bytes = encode_process_vector(&vector).expect("canonical encode");
        println!("VECTOR {name} {}\n{}END", bytes.len(), hex(&bytes));
    }
}
