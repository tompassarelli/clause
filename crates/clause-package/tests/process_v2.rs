use clause_package::*;

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

fn scope() -> TermScope {
    TermScope {
        universe: id!(UniverseId, 1),
        semantics: id!(ClauseSemanticsId, 2),
    }
}

fn term(payload: &str) -> Term {
    Term::atom(
        scope(),
        b"process-v2/test".to_vec(),
        payload.as_bytes().to_vec(),
        EqualityContract::ExactOctetsV1,
    )
    .expect("fixture Atom is canonical")
}

fn empty_snapshot() -> ProgramSnapshotPreimageV2 {
    ProgramSnapshotPreimageV2 {
        constitution: ProgramConstitutionPreimageV2 {
            semantics: scope().semantics,
            universe: scope().universe,
            formations: vec![],
            schemas: vec![],
            capabilities: vec![],
            operators: vec![],
            applications: vec![],
        },
        successor_grants: vec![],
        static_execution_grants: vec![],
        state_admission_grants: vec![],
        judgment_authority_grants: vec![],
    }
}

fn empty_package() -> ProcessPackageV2 {
    let snapshot = empty_snapshot();
    let claimed_snapshot =
        derive_program_snapshot_id(&snapshot).expect("fixture snapshot is canonical");
    ProcessPackageV2 {
        claimed_snapshot,
        snapshot,
        initial_state_views: vec![],
        records: vec![],
    }
}

fn one() -> CardinalityV2 {
    CardinalityV2 {
        minimum: 1,
        maximum: Some(1),
    }
}

fn target(name: &str) -> FormationTargetV2 {
    FormationTargetV2 {
        type_term: term(&format!("type/{name}")),
        interpretation: term(&format!("interpretation/{name}")),
    }
}

fn make_activation_pins_noncanonical(pins: &mut ActivationPins) {
    assert!(pins.constitutive_dependencies.len() > 1);
    pins.constitutive_dependencies.reverse();
}

fn assert_noncanonical_package_order(package: &ProcessPackageV2, field: &'static str) {
    assert_eq!(
        encode_process_package(package),
        Err(CanonicalEncodeError::NonCanonicalOrder(field))
    );
}

fn nominal_application(local: u32) -> ApplicationDeclarationPreimageV2 {
    ApplicationDeclarationPreimageV2 {
        id: ApplicationLocalId::new(local),
        form: ApplicationFormPreimageV2 {
            formation: FormationLocalId::new(1),
            schema: RelationSchemaLocalId::new(1),
            operator: OperatorLocalId::new(1),
            eligible_modes: vec![ModeLocalId::new(1)],
            bindings: vec![RoleBindingPreimageV2 {
                role: RoleLocalId::new(1),
                occurrence: 0,
                value: RoleBindingValuePreimageV2::Known(FormationLocalId::new(1)),
            }],
            context_requirements: vec![],
            constraint_discharges: vec![],
            result_domain: target("result"),
            direct_dependencies: vec![],
            dependency_closure: vec![
                LocalSemanticDependencyV2::Formation(FormationLocalId::new(1)),
                LocalSemanticDependencyV2::RelationSchema(RelationSchemaLocalId::new(1)),
                LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
                    schema: RelationSchemaLocalId::new(1),
                    role: RoleLocalId::new(1),
                }),
                LocalSemanticDependencyV2::Operator(OperatorLocalId::new(1)),
                LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
                    operator: OperatorLocalId::new(1),
                    mode: ModeLocalId::new(1),
                }),
            ],
        },
    }
}

fn two_application_snapshot() -> ProgramSnapshotPreimageV2 {
    ProgramSnapshotPreimageV2 {
        constitution: ProgramConstitutionPreimageV2 {
            semantics: scope().semantics,
            universe: scope().universe,
            formations: vec![FormationJudgmentPreimageV2 {
                id: FormationLocalId::new(1),
                context: vec![],
                term: term("application/equal-shape"),
                target: target("role"),
                direct_dependencies: vec![
                    LocalSemanticDependencyV2::RelationSchema(RelationSchemaLocalId::new(1)),
                    LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
                        schema: RelationSchemaLocalId::new(1),
                        role: RoleLocalId::new(1),
                    }),
                    LocalSemanticDependencyV2::Operator(OperatorLocalId::new(1)),
                    LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
                        operator: OperatorLocalId::new(1),
                        mode: ModeLocalId::new(1),
                    }),
                ],
            }],
            schemas: vec![RelationSchemaPreimageV2 {
                id: RelationSchemaLocalId::new(1),
                roles: vec![RoleDeclarationPreimageV2 {
                    id: RoleLocalId::new(1),
                    target: target("role"),
                    cardinality: one(),
                    direct_dependencies: vec![],
                }],
                constraints: vec![],
                result_domain: target("result"),
                direct_dependencies: vec![],
            }],
            capabilities: vec![],
            operators: vec![OperatorPreimageV2 {
                id: OperatorLocalId::new(1),
                modes: vec![ModePreimageV2 {
                    id: ModeLocalId::new(1),
                    schema: RelationSchemaLocalId::new(1),
                    known_roles: vec![RoleLocalId::new(1)],
                    produced_roles: vec![],
                    static_basis: StaticActivationBasisPreimageV2 {
                        context_requirements: vec![],
                        constitutive_dependencies: vec![],
                    },
                    authorization_requirements: vec![],
                    dynamic_prerequisites: vec![],
                    contract: ModeContractV2 {
                        determinism: DeterminismContractV2::Deterministic,
                        result_cardinality: one(),
                        result_order: ResultOrderContractV2::UnorderedFiniteSet,
                        failure_domain: None,
                        state_delta_domain: None,
                        budget_exhaustion_domain: None,
                        effect_intents: vec![],
                        formation_checks: vec![],
                        productivity: ProductivityContractV2 {
                            kind: ProductivityKindV2::Partial,
                            obligations: vec![],
                        },
                        scheduling_requirements: vec![],
                        resource_requirements: vec![],
                        capability_requirements: vec![],
                        continuation: ContinuationContractV2::TerminalOnly { may_cancel: false },
                    },
                    direct_dependencies: vec![],
                }],
                direct_dependencies: vec![],
            }],
            applications: vec![nominal_application(1), nominal_application(2)],
        },
        successor_grants: vec![],
        static_execution_grants: vec![],
        state_admission_grants: vec![],
        judgment_authority_grants: vec![],
    }
}

#[test]
fn exact_decode_check_binding_derives_snapshot_shape_and_package_identity() {
    let candidate = empty_package();
    let bytes = encode_process_package(&candidate).expect("canonical process-v2 package");
    assert_eq!(&bytes[..4], b"CLPV");
    assert_eq!(bytes[4], 2);

    let decoded = decode_process_package(&bytes).expect("strict process-v2 decode");
    assert_eq!(decoded.exact_bytes(), bytes);
    assert_eq!(decoded.candidate(), &candidate);
    let checked = check_process_package(decoded).expect("closed package checks");
    assert_eq!(checked.exact_bytes(), bytes);
    assert_eq!(
        checked.constitution().snapshot(),
        candidate.claimed_snapshot
    );
    assert_eq!(
        checked.canonical_snapshot_preimage(),
        checked.constitution().exact_snapshot_preimage_bytes()
    );

    let checked_again = check_process_package(
        decode_process_package(&bytes).expect("fresh exact ingress decodes independently"),
    )
    .expect("fresh exact ingress checks independently");
    assert_eq!(checked_again.id(), checked.id());
    assert_eq!(checked_again.exact_bytes(), checked.exact_bytes());
}

#[test]
fn two_nominal_applications_share_one_derived_shape_without_deduplication() {
    let snapshot = two_application_snapshot();
    let claimed_snapshot =
        derive_program_snapshot_id(&snapshot).expect("snapshot identity derives");
    let candidate = ProcessPackageV2 {
        claimed_snapshot,
        snapshot,
        initial_state_views: vec![],
        records: vec![],
    };
    let bytes = encode_process_package(&candidate).expect("canonical candidate");
    let checked = check_process_package(decode_process_package(&bytes).expect("strict decode"))
        .expect("formation and identities check");
    let carrier = ProcessCarrier::replay(&checked, &AuthorityStore::new())
        .expect("inert declarations need no invented runtime authority");

    let first = ApplicationId {
        snapshot: claimed_snapshot,
        local: ApplicationLocalId::new(1),
    };
    let second = ApplicationId {
        snapshot: claimed_snapshot,
        local: ApplicationLocalId::new(2),
    };
    assert_ne!(first, second);
    assert_eq!(carrier.application_count(), 2);
    assert_eq!(
        carrier
            .application(first)
            .expect("first nominal Application")
            .shape(),
        carrier
            .application(second)
            .expect("second nominal Application")
            .shape()
    );
}

#[test]
fn v1_and_decoder_side_top_level_overflow_have_no_live_path() {
    let mut version_one = encode_process_package(&empty_package()).expect("canonical package");
    version_one[4] = 1;
    assert_eq!(
        decode_process_package(&version_one),
        Err(CanonicalDecodeError::UnsupportedVersion {
            offset: 4,
            found: 1,
        })
    );

    let mut oversized = encode_process_package(&empty_package()).expect("canonical package");
    // CLPV + version + claimed snapshot + semantics + universe places the
    // first top-level declaration count at offset 101.
    oversized[101..105].copy_from_slice(&1_000_001_u32.to_be_bytes());
    assert_eq!(
        decode_process_package(&oversized),
        Err(CanonicalDecodeError::ListTooLong {
            offset: 101,
            count: 1_000_001,
        })
    );
}

#[test]
fn formation_check_targets_and_every_nested_pin_set_are_canonical() {
    let (mut duplicate_checks, _) = finalized_core_package();
    let checks = &mut duplicate_checks.snapshot.constitution.operators[0].modes[0]
        .contract
        .formation_checks;
    checks.push(checks.last().expect("core has Formation checks").clone());
    assert_noncanonical_package_order(&duplicate_checks, "formation check targets");

    let (mut unsorted_checks, _) = finalized_core_package();
    unsorted_checks.snapshot.constitution.operators[0].modes[0]
        .contract
        .formation_checks
        .reverse();
    assert_noncanonical_package_order(&unsorted_checks, "formation check targets");

    let (mut activation, _) = finalized_core_package();
    let pins = activation
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Activation(value) => Some(&mut value.pins),
            _ => None,
        })
        .expect("core has an Activation");
    make_activation_pins_noncanonical(pins);
    assert_noncanonical_package_order(&activation, "Activation constitutive dependencies");

    let (mut resumption, _) = finalized_core_package();
    let pins = resumption
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Resumption(value) => Some(&mut value.body.pins.activation_pins),
            _ => None,
        })
        .expect("core has a Resumption");
    make_activation_pins_noncanonical(pins);
    assert_noncanonical_package_order(&resumption, "Activation constitutive dependencies");

    let (mut handoff, _) = linear_double_takeup_package();
    let pins = handoff
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Handoff(value) => Some(&mut value.body.pins.activation_pins),
            _ => None,
        })
        .expect("handoff fixture has a Handoff");
    make_activation_pins_noncanonical(pins);
    assert_noncanonical_package_order(&handoff, "Activation constitutive dependencies");

    let (mut cancellation, context) = finalized_core_package();
    let mut pins = pure_pins(context);
    make_activation_pins_noncanonical(&mut pins);
    cancellation
        .records
        .push(ProcessRecordV2::Cancellation(CancellationOccurrenceV2 {
            body: CancellationOccurrenceBodyV2 {
                id: id!(CancellationOccurrenceId, 116),
                target: CancellationTarget::Activation(id!(ActivationId, 20)),
                pins,
            },
            provenance: OccurrenceProvenance::EnteredThrough(entered_through(
                context.pure_boundary,
                196,
                vec![],
            )),
        }));
    assert_noncanonical_package_order(&cancellation, "Activation constitutive dependencies");

    let (mut suspension, _) = finalized_core_package();
    let pins = suspension
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => {
                steps.iter_mut().find_map(|step| match &mut step.outcome {
                    StepOutcomeProposalV2::Suspend(value) => Some(&mut value.pins.activation_pins),
                    _ => None,
                })
            }
            _ => None,
        })
        .expect("core has a suspended Continuation");
    make_activation_pins_noncanonical(pins);
    assert_noncanonical_package_order(&suspension, "Activation constitutive dependencies");

    let (mut exhaustion, context) = bounded_exhaustion_candidate(0);
    let pins = exhaustion
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => steps.iter_mut().find_map(|step| {
                let StepOutcomeProposalV2::BudgetExhausted {
                    continuation: slot, ..
                } = &mut step.outcome
                else {
                    return None;
                };
                let mut value = continuation(context);
                make_activation_pins_noncanonical(&mut value.pins.activation_pins);
                *slot = Some(value);
                slot.as_mut().map(|value| &mut value.pins.activation_pins)
            }),
            _ => None,
        })
        .expect("bounded fixture has an exhaustion Continuation slot");
    assert!(
        pins.constitutive_dependencies
            .windows(2)
            .any(|pair| pair[0] > pair[1])
    );
    assert_noncanonical_package_order(&exhaustion, "Activation constitutive dependencies");
}

#[test]
fn wrong_snapshot_claim_and_exact_byte_substitution_change_identity() {
    let mut wrong_claim = empty_package();
    wrong_claim.claimed_snapshot = id!(ProgramSnapshotId, 5);
    let bytes = encode_process_package(&wrong_claim).expect("wrong claim remains inert bytes");
    let error = check_process_package(
        decode_process_package(&bytes).expect("wrong identity does not corrupt the wire"),
    )
    .expect_err("checking derives and rejects the wrong snapshot identity");
    assert!(matches!(
        error,
        ProcessPackageCheckError::SnapshotIdMismatch { claimed, derived }
            if claimed == wrong_claim.claimed_snapshot
                && derived == derive_program_snapshot_id(&wrong_claim.snapshot).unwrap()
    ));

    let first = empty_package();
    let first_bytes = encode_process_package(&first).expect("first package");
    let first_checked =
        check_process_package(decode_process_package(&first_bytes).expect("first decode"))
            .expect("first check");

    let mut second = empty_package();
    second.snapshot.constitution.universe = id!(UniverseId, 6);
    second.claimed_snapshot =
        derive_program_snapshot_id(&second.snapshot).expect("second snapshot identity");
    let second_bytes = encode_process_package(&second).expect("second package");
    let second_checked =
        check_process_package(decode_process_package(&second_bytes).expect("second decode"))
            .expect("second check");

    assert_ne!(first_bytes, second_bytes);
    assert_ne!(
        first_checked.constitution().snapshot(),
        second_checked.constitution().snapshot()
    );
    assert_ne!(first_checked.id(), second_checked.id());
}

#[test]
fn term_equality_is_scoped_and_mixed_scope_compounds_reject() {
    let baseline = term("equal payload");
    let other_universe = Term::atom(
        TermScope {
            universe: id!(UniverseId, 3),
            semantics: scope().semantics,
        },
        b"process-v2/test".to_vec(),
        b"equal payload".to_vec(),
        EqualityContract::ExactOctetsV1,
    )
    .expect("fixture Atom is canonical");
    let other_semantics = Term::atom(
        TermScope {
            universe: scope().universe,
            semantics: id!(ClauseSemanticsId, 4),
        },
        b"process-v2/test".to_vec(),
        b"equal payload".to_vec(),
        EqualityContract::ExactOctetsV1,
    )
    .expect("fixture Atom is canonical");

    assert_ne!(baseline, other_universe);
    assert_ne!(baseline, other_semantics);
    assert_eq!(
        Term::raw_triple([baseline, other_universe, other_semantics]),
        Err(TermError::MixedScopeTriple)
    );
}

#[test]
fn repeated_support_sources_survive_in_distinct_typed_slots() {
    let source = SupportSource::Observation(id!(ObservationId, 10));
    let supports = vec![
        SupportUse {
            slot: SupportSlotId::new(0),
            role: term("premise/left"),
            source,
        },
        SupportUse {
            slot: SupportSlotId::new(1),
            role: term("premise/right"),
            source,
        },
    ];

    validate_support_uses(&supports).expect("one occurrence may satisfy two exact premise slots");
    assert_eq!(supports[0].source, supports[1].source);
    assert_ne!(supports[0].slot, supports[1].slot);

    let mut collapsed = supports;
    collapsed[1].slot = SupportSlotId::new(0);
    assert_eq!(
        validate_support_uses(&collapsed),
        Err(ProvenanceError::DuplicateSupportSlot(SupportSlotId::new(0)))
    );
}

#[test]
fn dynamic_prerequisites_preserve_occurrence_multiplicity() {
    let mode = core_mode(id!(ProgramSnapshotId, 1), 1);
    let occurrence = ActivationPrerequisite::Observation(id!(ObservationId, 11));
    let left = DynamicPrerequisiteBindingV2 {
        slot: PrerequisiteSlotId {
            mode,
            local: PrerequisiteLocalId::new(1),
        },
        ordinal: 0,
        value: occurrence,
    };
    let right = DynamicPrerequisiteBindingV2 {
        slot: PrerequisiteSlotId {
            mode,
            local: PrerequisiteLocalId::new(2),
        },
        ordinal: 0,
        value: occurrence,
    };
    assert_eq!(left.value, right.value);
    assert_ne!(left.slot, right.slot);
}

#[test]
fn empty_static_authorization_is_canonical_but_never_self_authorizing() {
    let basis = ActivationStaticBasis {
        execution_authorizations: vec![],
        judgment_authorities: vec![],
    };
    validate_activation_static_basis(&basis)
        .expect("a Mode may constitutionally require no static authorization");
    assert!(basis.execution_authorizations.is_empty());
}

#[test]
fn program_revision_identity_is_derived_and_every_preimage_field_is_bound() {
    let preimage = ProgramRevisionPreimage {
        semantics: scope().semantics,
        program: id!(ProgramId, 20),
        predecessor: None,
        snapshot: id!(ProgramSnapshotId, 21),
        change: id!(ProgramChangeOccurrenceId, 22),
    };
    let claim = preimage.derived_claim();
    claim
        .validate_derived_id()
        .expect("the public constructor derives the exact identity");

    let wrong = ProgramRevisionClaim {
        id: id!(ProgramRevisionId, 23),
        preimage,
    };
    assert!(matches!(
        wrong.validate_derived_id(),
        Err(AuthorityError::ProgramRevisionIdMismatch { claimed, derived })
            if claimed == wrong.id && derived == claim.id
    ));

    for changed in [
        ProgramRevisionPreimage {
            semantics: id!(ClauseSemanticsId, 24),
            ..preimage
        },
        ProgramRevisionPreimage {
            program: id!(ProgramId, 25),
            ..preimage
        },
        ProgramRevisionPreimage {
            predecessor: Some(id!(ProgramRevisionId, 26)),
            ..preimage
        },
        ProgramRevisionPreimage {
            snapshot: id!(ProgramSnapshotId, 27),
            ..preimage
        },
        ProgramRevisionPreimage {
            change: id!(ProgramChangeOccurrenceId, 28),
            ..preimage
        },
    ] {
        assert_ne!(changed.derived_claim().id, claim.id);
    }
}

fn entered() -> EnteredThrough {
    EnteredThrough {
        boundary: id!(BoundaryRef, 30),
        evidence: id!(ExternalEvidenceRef, 31),
        causes: vec![],
    }
}

fn rejection_fixture() -> (
    CandidateDeltaV2,
    StateRevision,
    StepRef,
    JudgmentOccurrenceV2,
    Vec<JudgmentOccurrenceV2>,
    StateAdmissionDecisionV2,
) {
    let session = id!(RuntimeSessionId, 32);
    let policy = id!(RuntimePolicyId, 33);
    let delta = id!(CandidateDeltaId, 34);
    let obligation = ObligationId {
        delta,
        local: ObligationLocalId::new(0),
    };
    let base = StateRevision {
        id: id!(StateRevisionId, 35),
        session,
        predecessor: None,
        cause: StateRevisionCause::SessionStart(id!(SessionStartOccurrenceId, 36)),
        payload: term("world/base"),
        canonical_state_snapshot: b"canonical base".to_vec().into_boxed_slice(),
        policy,
        semantics: scope().semantics,
    };
    let candidate = CandidateDeltaV2 {
        id: delta,
        base: base.id,
        delta: domain_bound("delta/candidate", 99),
        proposed_payload: term("world/candidate"),
        evidence: vec![],
        obligations: vec![CandidateObligation {
            id: obligation,
            requirement: term("obligation/required"),
        }],
    };
    let root_policy = id!(RootPolicyId, 37);
    let judgment_authority = JudgmentAuthorityEvidence::IrreducibleRoot {
        policy: root_policy,
        authority: RootJudgmentAuthorityRef {
            policy: root_policy,
            local: JudgmentAuthorityLocalId::new(0),
        },
    };
    let verdict = JudgmentOccurrenceV2 {
        body: JudgmentOccurrenceBodyV2 {
            id: id!(JudgmentOccurrenceId, 38),
            judgment: AdmissionJudgment {
                delta,
                session,
                policy,
                claim: AdmissionJudgmentClaim::Verdict(AdmissionDisposition::Reject),
            },
            authority: judgment_authority,
            supports: vec![],
        },
        provenance: OccurrenceProvenance::EnteredThrough(entered()),
    };
    let obligation_judgment = JudgmentOccurrenceV2 {
        body: JudgmentOccurrenceBodyV2 {
            id: id!(JudgmentOccurrenceId, 39),
            judgment: AdmissionJudgment {
                delta,
                session,
                policy,
                claim: AdmissionJudgmentClaim::Obligation {
                    obligation,
                    status: ObligationStatus::Unsatisfied,
                },
            },
            authority: judgment_authority,
            supports: vec![],
        },
        provenance: OccurrenceProvenance::EnteredThrough(entered()),
    };
    let producer = StepRef {
        run: id!(RunId, 40),
        activation: id!(ActivationId, 41),
        step: id!(StepId, 42),
    };
    let decision = StateAdmissionDecisionV2 {
        occurrence: id!(AdmissionOccurrenceId, 43),
        delta,
        authorization: AdmissionAuthorizationEvidence::IrreducibleRoot {
            policy: root_policy,
            authorization: RootAdmissionAuthorizationRef {
                policy: root_policy,
                local: AdmissionAuthorizationLocalId::new(0),
            },
        },
        evidence: vec![],
        verdict: verdict.body.id,
        obligation_judgments: vec![ObligationJudgmentUse {
            obligation,
            judgment: obligation_judgment.body.id,
        }],
        provenance: entered(),
        outcome: StateAdmissionOutcomeV2::Reject(AdmissionRejectionV2 {
            reason: term("rejected/unsatisfied-obligation"),
        }),
    };

    (
        candidate,
        base,
        producer,
        verdict,
        vec![obligation_judgment],
        decision,
    )
}

#[test]
fn governed_rejection_is_a_valid_one_shot_decision_without_a_state_revision() {
    let (candidate, base, producer, verdict, obligation_judgments, decision) = rejection_fixture();
    validate_state_admission_decision_inputs(
        &candidate,
        &decision,
        &verdict,
        &obligation_judgments,
        AdmissionDecisionContext {
            base: &base,
            producer,
            prior_decision: None,
        },
    )
    .expect("a governed rejection records a decision");
    assert!(matches!(
        decision.outcome,
        StateAdmissionOutcomeV2::Reject(_)
    ));

    assert_eq!(
        validate_state_admission_decision_inputs(
            &candidate,
            &decision,
            &verdict,
            &obligation_judgments,
            AdmissionDecisionContext {
                base: &base,
                producer,
                prior_decision: Some(decision.occurrence),
            },
        ),
        Err(ProvenanceError::CandidateAlreadyDecided {
            delta: candidate.id,
            prior: decision.occurrence,
        })
    );
}

#[test]
fn decision_verdict_obligations_session_and_policy_are_exact() {
    let (candidate, base, producer, verdict, obligation_judgments, decision) = rejection_fixture();

    let mut wrong_verdict = verdict.clone();
    wrong_verdict.body.judgment.claim =
        AdmissionJudgmentClaim::Verdict(AdmissionDisposition::Admit);
    assert!(matches!(
        validate_state_admission_decision_inputs(
            &candidate,
            &decision,
            &wrong_verdict,
            &obligation_judgments,
            AdmissionDecisionContext {
                base: &base,
                producer,
                prior_decision: None,
            },
        ),
        Err(ProvenanceError::AdmissionVerdictMismatch {
            claimed: AdmissionDisposition::Admit,
            actual: AdmissionDisposition::Reject,
        })
    ));

    let mut wrong_context = verdict.clone();
    wrong_context.body.judgment.session = id!(RuntimeSessionId, 44);
    assert_eq!(
        validate_state_admission_decision_inputs(
            &candidate,
            &decision,
            &wrong_context,
            &obligation_judgments,
            AdmissionDecisionContext {
                base: &base,
                producer,
                prior_decision: None,
            },
        ),
        Err(ProvenanceError::JudgmentContextMismatch(
            wrong_context.body.id
        ))
    );

    let mut missing_obligation = decision.clone();
    missing_obligation.obligation_judgments.clear();
    assert!(matches!(
        validate_state_admission_decision_inputs(
            &candidate,
            &missing_obligation,
            &verdict,
            &obligation_judgments,
            AdmissionDecisionContext {
                base: &base,
                producer,
                prior_decision: None,
            },
        ),
        Err(ProvenanceError::ObligationJudgmentCountMismatch {
            obligations: 1,
            uses: 0,
            resolved: 1,
        })
    ));
}

#[derive(Clone, Copy)]
struct CoreContext {
    snapshot: ProgramSnapshotId,
    revision: ProgramRevisionClaim,
    session: RuntimeSessionId,
    policy: RuntimePolicyId,
    session_start: SessionStartOccurrenceId,
    initial_state: StateRevisionId,
    root_policy: RootPolicyId,
    pure_boundary: BoundaryRef,
    state_boundary: BoundaryRef,
}

fn core_application(snapshot: ProgramSnapshotId, local: u32) -> ApplicationId {
    ApplicationId {
        snapshot,
        local: ApplicationLocalId::new(local),
    }
}

fn core_mode(snapshot: ProgramSnapshotId, local: u32) -> ModeId {
    ModeId {
        operator: OperatorRef {
            snapshot,
            local: OperatorLocalId::new(1),
        },
        local: ModeLocalId::new(local),
    }
}

fn core_dependency_closure(snapshot: ProgramSnapshotId) -> Vec<SemanticDependencyV2> {
    vec![
        SemanticDependencyV2::Formation(FormationRefV2 {
            snapshot,
            local: FormationLocalId::new(1),
        }),
        SemanticDependencyV2::Formation(FormationRefV2 {
            snapshot,
            local: FormationLocalId::new(2),
        }),
        SemanticDependencyV2::RelationSchema(RelationSchemaId {
            snapshot,
            local: RelationSchemaLocalId::new(1),
        }),
        SemanticDependencyV2::Role(RoleId {
            schema: RelationSchemaId {
                snapshot,
                local: RelationSchemaLocalId::new(1),
            },
            local: RoleLocalId::new(1),
        }),
        SemanticDependencyV2::Operator(OperatorRef {
            snapshot,
            local: OperatorLocalId::new(1),
        }),
        SemanticDependencyV2::Mode(core_mode(snapshot, 1)),
        SemanticDependencyV2::Mode(core_mode(snapshot, 2)),
    ]
}

fn core_application_declaration(local: u32) -> ApplicationDeclarationPreimageV2 {
    ApplicationDeclarationPreimageV2 {
        id: ApplicationLocalId::new(local),
        form: ApplicationFormPreimageV2 {
            formation: FormationLocalId::new(1),
            schema: RelationSchemaLocalId::new(1),
            operator: OperatorLocalId::new(1),
            eligible_modes: vec![ModeLocalId::new(1), ModeLocalId::new(2)],
            bindings: vec![RoleBindingPreimageV2 {
                role: RoleLocalId::new(1),
                occurrence: 0,
                value: RoleBindingValuePreimageV2::Known(FormationLocalId::new(1)),
            }],
            context_requirements: vec![],
            constraint_discharges: vec![],
            result_domain: target("result"),
            direct_dependencies: vec![],
            dependency_closure: vec![
                LocalSemanticDependencyV2::Formation(FormationLocalId::new(1)),
                LocalSemanticDependencyV2::Formation(FormationLocalId::new(2)),
                LocalSemanticDependencyV2::RelationSchema(RelationSchemaLocalId::new(1)),
                LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
                    schema: RelationSchemaLocalId::new(1),
                    role: RoleLocalId::new(1),
                }),
                LocalSemanticDependencyV2::Operator(OperatorLocalId::new(1)),
                LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
                    operator: OperatorLocalId::new(1),
                    mode: ModeLocalId::new(1),
                }),
                LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
                    operator: OperatorLocalId::new(1),
                    mode: ModeLocalId::new(2),
                }),
            ],
        },
    }
}

fn core_snapshot() -> ProgramSnapshotPreimageV2 {
    let application_dependencies = vec![
        LocalSemanticDependencyV2::Formation(FormationLocalId::new(2)),
        LocalSemanticDependencyV2::RelationSchema(RelationSchemaLocalId::new(1)),
        LocalSemanticDependencyV2::Role(LocalRoleRefV2 {
            schema: RelationSchemaLocalId::new(1),
            role: RoleLocalId::new(1),
        }),
        LocalSemanticDependencyV2::Operator(OperatorLocalId::new(1)),
        LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
            operator: OperatorLocalId::new(1),
            mode: ModeLocalId::new(1),
        }),
        LocalSemanticDependencyV2::Mode(LocalModeRefV2 {
            operator: OperatorLocalId::new(1),
            mode: ModeLocalId::new(2),
        }),
    ];
    let mode = |local: u32, stateful: bool| ModePreimageV2 {
        id: ModeLocalId::new(local),
        schema: RelationSchemaLocalId::new(1),
        known_roles: vec![RoleLocalId::new(1)],
        produced_roles: vec![],
        static_basis: StaticActivationBasisPreimageV2 {
            context_requirements: vec![],
            constitutive_dependencies: vec![],
        },
        authorization_requirements: vec![],
        dynamic_prerequisites: vec![DynamicPrerequisiteRequirementPreimageV2 {
            slot: PrerequisiteLocalId::new(1),
            role: Some(RoleLocalId::new(1)),
            requirement: ActivationPrerequisiteKind::Observation,
            expected: FormationLocalId::new(2),
            scope: PrerequisiteScope::SameSemantics,
            cardinality: CardinalityV2 {
                minimum: 0,
                maximum: Some(1),
            },
            cause_projection: vec![CauseProjectionEntryV2 {
                component: CauseComponentLocalId::new(1),
                path: PrerequisiteOccurrencePathV2::BoundOccurrence,
            }],
        }],
        contract: ModeContractV2 {
            determinism: DeterminismContractV2::Deterministic,
            result_cardinality: one(),
            result_order: ResultOrderContractV2::UnorderedFiniteSet,
            failure_domain: None,
            state_delta_domain: stateful.then(|| target("state-delta")),
            budget_exhaustion_domain: None,
            effect_intents: vec![],
            formation_checks: if local == 1 {
                vec![target("result"), target("state-delta")]
            } else {
                vec![]
            },
            productivity: ProductivityContractV2 {
                kind: ProductivityKindV2::Partial,
                obligations: vec![],
            },
            scheduling_requirements: vec![],
            resource_requirements: vec![],
            capability_requirements: vec![],
            continuation: if stateful {
                ContinuationContractV2::TerminalOnly { may_cancel: false }
            } else {
                ContinuationContractV2::Suspensible {
                    use_policy: ContinuationUseV2::Linear,
                    may_handoff: true,
                    may_cancel: false,
                }
            },
        },
        direct_dependencies: vec![],
    };

    ProgramSnapshotPreimageV2 {
        constitution: ProgramConstitutionPreimageV2 {
            semantics: scope().semantics,
            universe: scope().universe,
            formations: vec![
                FormationJudgmentPreimageV2 {
                    id: FormationLocalId::new(1),
                    context: vec![],
                    term: term("application/core"),
                    target: target("role"),
                    direct_dependencies: application_dependencies,
                },
                FormationJudgmentPreimageV2 {
                    id: FormationLocalId::new(2),
                    context: vec![],
                    term: term("requirement/formation-check"),
                    target: target("prerequisite-kind"),
                    direct_dependencies: vec![],
                },
            ],
            schemas: vec![RelationSchemaPreimageV2 {
                id: RelationSchemaLocalId::new(1),
                roles: vec![RoleDeclarationPreimageV2 {
                    id: RoleLocalId::new(1),
                    target: target("role"),
                    cardinality: one(),
                    direct_dependencies: vec![],
                }],
                constraints: vec![],
                result_domain: target("result"),
                direct_dependencies: vec![],
            }],
            capabilities: vec![],
            operators: vec![OperatorPreimageV2 {
                id: OperatorLocalId::new(1),
                modes: vec![mode(1, false), mode(2, true)],
                direct_dependencies: vec![],
            }],
            applications: vec![
                core_application_declaration(1),
                core_application_declaration(2),
            ],
        },
        successor_grants: vec![],
        static_execution_grants: vec![],
        state_admission_grants: vec![],
        judgment_authority_grants: vec![],
    }
}

fn core_context(snapshot: ProgramSnapshotId) -> CoreContext {
    let session = id!(RuntimeSessionId, 120);
    let policy = id!(RuntimePolicyId, 121);
    let session_start = id!(SessionStartOccurrenceId, 122);
    let revision = ProgramRevisionPreimage {
        semantics: scope().semantics,
        program: id!(ProgramId, 123),
        predecessor: None,
        snapshot,
        change: id!(ProgramChangeOccurrenceId, 124),
    }
    .derived_claim();
    let initial_snapshot = canonical_term_bytes(&term("world/initial"))
        .expect("initial State payload has canonical bytes");
    let initial_state = RuntimeSessionAnchor::establish(
        session,
        revision.id,
        scope().semantics,
        policy,
        session_start,
        initial_snapshot,
    )
    .initial_state_id();
    CoreContext {
        snapshot,
        revision,
        session,
        policy,
        session_start,
        initial_state,
        root_policy: id!(RootPolicyId, 125),
        pure_boundary: id!(BoundaryRef, 126),
        state_boundary: id!(BoundaryRef, 127),
    }
}

fn pure_pins(context: CoreContext) -> ActivationPins {
    ActivationPins {
        semantics: scope().semantics,
        snapshot: context.snapshot,
        program_revision: context.revision.id,
        runtime_session: None,
        observed_state: None,
        runtime_policy: None,
        context_requirements: vec![],
        constitutive_dependencies: core_dependency_closure(context.snapshot),
        capabilities: vec![],
        scheduling_requirements: vec![],
        resource_requirements: vec![],
        cancellation_scope: CancellationScope::Activation,
        budget: Budget {
            remaining_units: 100,
        },
    }
}

fn state_pins(context: CoreContext) -> ActivationPins {
    ActivationPins {
        runtime_session: Some(context.session),
        observed_state: Some(context.initial_state),
        runtime_policy: Some(context.policy),
        ..pure_pins(context)
    }
}

fn entered_through(
    boundary: BoundaryRef,
    evidence_tag: u8,
    causes: Vec<CausalRef>,
) -> EnteredThrough {
    EnteredThrough {
        boundary,
        evidence: id!(ExternalEvidenceRef, evidence_tag),
        causes,
    }
}

fn root_activation(
    context: CoreContext,
    activation_tag: u8,
    run_tag: u8,
    configuration_tag: u8,
    mode_local: u32,
    trigger: RootTrigger,
) -> ActivationProposalV2 {
    ActivationProposalV2 {
        id: id!(ActivationId, activation_tag),
        application: core_application(context.snapshot, 1),
        mode: core_mode(context.snapshot, mode_local),
        pins: if mode_local == 1 {
            pure_pins(context)
        } else {
            state_pins(context)
        },
        static_basis: ActivationStaticBasis {
            execution_authorizations: vec![],
            judgment_authorities: vec![],
        },
        prerequisite_bindings: vec![],
        causes: ActivationCauseFrontierV2 {
            origin: ActivationOrigin::RootedBy(trigger),
            prerequisite_occurrences: vec![],
        },
        membership: RunMembership::RootOf(id!(RunId, run_tag)),
        initial_configuration: ConfigurationProposal {
            id: id!(ConfigurationId, configuration_tag),
            value: term("configuration/initial"),
        },
    }
}

fn continuation(context: CoreContext) -> ContinuationProposalV2 {
    ContinuationProposalV2 {
        id: id!(ContinuationId, 70),
        emitted_by: id!(StepId, 51),
        pins: ContinuationPins {
            run: id!(RunId, 30),
            activation: id!(ActivationId, 20),
            application: core_application(context.snapshot, 1),
            mode: core_mode(context.snapshot, 1),
            activation_pins: pure_pins(context),
            remaining_budget: Budget {
                remaining_units: 80,
            },
        },
        remainder: term("configuration/61"),
    }
}

fn budget(before: u64, consumed_units: u64, after: u64) -> StepBudgetTransitionV2 {
    StepBudgetTransitionV2 {
        before: Budget {
            remaining_units: before,
        },
        consumed_units,
        after: Budget {
            remaining_units: after,
        },
    }
}

fn domain_bound(value: &str, evidence_tag: u8) -> DomainBoundTermV2 {
    DomainBoundTermV2 {
        term: term(value),
        evidence: id!(ObservationId, evidence_tag),
    }
}

fn require_formation_observation(
    activation: &mut ActivationProposalV2,
    _context: CoreContext,
    evidence_tag: u8,
) {
    let slot = PrerequisiteSlotId {
        mode: activation.mode,
        local: PrerequisiteLocalId::new(1),
    };
    let value = ActivationPrerequisite::Observation(id!(ObservationId, evidence_tag));
    activation
        .prerequisite_bindings
        .push(DynamicPrerequisiteBindingV2 {
            slot,
            ordinal: 0,
            value,
        });
    activation
        .causes
        .prerequisite_occurrences
        .push(ActivationOccurrenceCauseV2 {
            slot,
            ordinal: 0,
            component: CauseComponentLocalId::new(1),
            occurrence: value,
        });
}

fn formation_observation(
    evidence_tag: u8,
    subject: &str,
    domain: FormationTargetV2,
    source: SupportSource,
) -> ObservationProposalV2 {
    ObservationProposalV2::Formation {
        id: id!(ObservationId, evidence_tag),
        subject: term(subject),
        target: domain,
        supports: vec![SupportUse {
            slot: SupportSlotId::new(0),
            role: term("formation/evidence"),
            source,
        }],
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the fixture keeps every occurrence identity and transition field explicit at call sites"
)]
fn step(
    step_tag: u8,
    run_tag: u8,
    activation_tag: u8,
    before_tag: u8,
    after_tag: u8,
    budget: StepBudgetTransitionV2,
    causes: Vec<StepCause>,
    outcome: StepOutcomeProposalV2,
) -> StepProposalV2 {
    StepProposalV2 {
        id: id!(StepId, step_tag),
        run: id!(RunId, run_tag),
        activation: id!(ActivationId, activation_tag),
        before: id!(ConfigurationId, before_tag),
        after: ConfigurationProposal {
            id: id!(ConfigurationId, after_tag),
            value: term(&format!("configuration/{after_tag}")),
        },
        observed_state: None,
        budget,
        causes,
        observation_outcomes: vec![],
        candidate_delta: None,
        outcome,
    }
}

fn judgment(
    context: CoreContext,
    id_tag: u8,
    delta: CandidateDeltaId,
    claim: AdmissionJudgmentClaim,
    producer: StepRef,
    evidence_tag: u8,
) -> JudgmentOccurrenceV2 {
    JudgmentOccurrenceV2 {
        body: JudgmentOccurrenceBodyV2 {
            id: id!(JudgmentOccurrenceId, id_tag),
            judgment: AdmissionJudgment {
                delta,
                session: context.session,
                policy: context.policy,
                claim,
            },
            authority: JudgmentAuthorityEvidence::IrreducibleRoot {
                policy: context.root_policy,
                authority: RootJudgmentAuthorityRef {
                    policy: context.root_policy,
                    local: JudgmentAuthorityLocalId::new(0),
                },
            },
            supports: vec![SupportUse {
                slot: SupportSlotId::new(0),
                role: term("support/producer"),
                source: SupportSource::Step(producer),
            }],
        },
        provenance: OccurrenceProvenance::EnteredThrough(entered_through(
            context.state_boundary,
            evidence_tag,
            vec![CausalRef::CandidateDelta(delta)],
        )),
    }
}

fn build_core_package_from_snapshot(
    snapshot: ProgramSnapshotPreimageV2,
    successor_id: StateRevisionId,
) -> (ProcessPackageV2, CoreContext) {
    let claimed_snapshot =
        derive_program_snapshot_id(&snapshot).expect("core snapshot identity derives");
    let context = core_context(claimed_snapshot);
    let delta_admit = id!(CandidateDeltaId, 80);
    let delta_reject = id!(CandidateDeltaId, 81);
    let obligation_admit = ObligationId {
        delta: delta_admit,
        local: ObligationLocalId::new(0),
    };
    let obligation_reject = ObligationId {
        delta: delta_reject,
        local: ObligationLocalId::new(0),
    };
    let a1_step1 = StepRef {
        run: id!(RunId, 30),
        activation: id!(ActivationId, 20),
        step: id!(StepId, 50),
    };
    let a1_suspend = StepRef {
        step: id!(StepId, 51),
        ..a1_step1
    };
    let admit_producer = StepRef {
        run: id!(RunId, 33),
        activation: id!(ActivationId, 23),
        step: id!(StepId, 56),
    };
    let reject_producer = StepRef {
        run: id!(RunId, 34),
        activation: id!(ActivationId, 24),
        step: id!(StepId, 57),
    };

    let mut activation_20 = root_activation(
        context,
        20,
        30,
        40,
        1,
        RootTrigger::External(id!(ExternalTriggerOccurrenceId, 10)),
    );
    require_formation_observation(&mut activation_20, context, 88);
    let activation_21 = root_activation(
        context,
        21,
        31,
        41,
        1,
        RootTrigger::External(id!(ExternalTriggerOccurrenceId, 11)),
    );
    let activation_22 = root_activation(
        context,
        22,
        32,
        42,
        1,
        RootTrigger::External(id!(ExternalTriggerOccurrenceId, 12)),
    );
    let mut activation_23 = root_activation(
        context,
        23,
        33,
        43,
        2,
        RootTrigger::SessionStart(context.session_start),
    );
    require_formation_observation(&mut activation_23, context, 84);
    let mut activation_24 = root_activation(
        context,
        24,
        34,
        44,
        2,
        RootTrigger::SessionStart(context.session_start),
    );
    require_formation_observation(&mut activation_24, context, 85);

    let mut checker_step = step(
        53,
        32,
        22,
        42,
        63,
        budget(100, 10, 90),
        vec![StepCause::ActivationStart(id!(ActivationId, 22))],
        StepOutcomeProposalV2::Progress,
    );
    checker_step.observation_outcomes = [
        (84, "delta/admit", target("state-delta")),
        (85, "delta/reject", target("state-delta")),
        (86, "child/left", target("result")),
        (87, "child/right", target("result")),
        (88, "value/resumed", target("result")),
    ]
    .into_iter()
    .map(|(id, subject, domain)| {
        StepObservationOutcomeV2::Observed(formation_observation(
            id,
            subject,
            domain,
            SupportSource::ExternalTrigger(id!(ExternalTriggerOccurrenceId, 12)),
        ))
    })
    .collect();

    let mut first = step(
        50,
        30,
        20,
        40,
        60,
        budget(100, 10, 90),
        vec![StepCause::ActivationStart(id!(ActivationId, 20))],
        StepOutcomeProposalV2::Progress,
    );
    first.observation_outcomes = vec![
        StepObservationOutcomeV2::Observed(ObservationProposalV2::Truth {
            id: id!(ObservationId, 50),
            verdict: TruthVerdict::True,
            proposition: term("proposition/true"),
            supports: vec![
                SupportUse {
                    slot: SupportSlotId::new(0x1122_3344),
                    role: term("premise/left"),
                    source: SupportSource::ExternalTrigger(id!(ExternalTriggerOccurrenceId, 10)),
                },
                SupportUse {
                    slot: SupportSlotId::new(0x5566_7788),
                    role: term("premise/right"),
                    source: SupportSource::ExternalTrigger(id!(ExternalTriggerOccurrenceId, 10)),
                },
            ],
        }),
        StepObservationOutcomeV2::Observed(ObservationProposalV2::Truth {
            id: id!(ObservationId, 51),
            verdict: TruthVerdict::False,
            proposition: term("proposition/false"),
            supports: vec![SupportUse {
                slot: SupportSlotId::new(0x6677_8899),
                role: term("premise/negative"),
                source: SupportSource::ExternalTrigger(id!(ExternalTriggerOccurrenceId, 10)),
            }],
        }),
        StepObservationOutcomeV2::Absent(TruthAbsenceV2 {
            proposition: term("proposition/absent"),
            search_scope: term("search-scope/closed"),
            completion_evidence: vec![SupportUse {
                slot: SupportSlotId::new(0x7788_99aa),
                role: term("search/completed"),
                source: SupportSource::ExternalTrigger(id!(ExternalTriggerOccurrenceId, 10)),
            }],
        }),
    ];

    let mut state_admit_step = step(
        56,
        33,
        23,
        43,
        66,
        budget(100, 10, 90),
        vec![StepCause::ActivationStart(id!(ActivationId, 23))],
        StepOutcomeProposalV2::Progress,
    );
    state_admit_step.observed_state = Some(context.initial_state);
    state_admit_step.candidate_delta = Some(CandidateDeltaV2 {
        id: delta_admit,
        base: context.initial_state,
        delta: domain_bound("delta/admit", 84),
        proposed_payload: term("world/admitted"),
        evidence: vec![SupportUse {
            slot: SupportSlotId::new(0),
            role: term("support/base"),
            source: SupportSource::SessionStart(context.session_start),
        }],
        obligations: vec![CandidateObligation {
            id: obligation_admit,
            requirement: term("obligation/satisfied"),
        }],
    });

    let mut state_reject_step = step(
        57,
        34,
        24,
        44,
        67,
        budget(100, 10, 90),
        vec![StepCause::ActivationStart(id!(ActivationId, 24))],
        StepOutcomeProposalV2::Progress,
    );
    state_reject_step.observed_state = Some(context.initial_state);
    state_reject_step.candidate_delta = Some(CandidateDeltaV2 {
        id: delta_reject,
        base: context.initial_state,
        delta: domain_bound("delta/reject", 85),
        proposed_payload: term("world/rejected"),
        evidence: vec![SupportUse {
            slot: SupportSlotId::new(0),
            role: term("support/base"),
            source: SupportSource::SessionStart(context.session_start),
        }],
        obligations: vec![CandidateObligation {
            id: obligation_reject,
            requirement: term("obligation/unsatisfied"),
        }],
    });

    let admit_verdict = judgment(
        context,
        90,
        delta_admit,
        AdmissionJudgmentClaim::Verdict(AdmissionDisposition::Admit),
        admit_producer,
        186,
    );
    let admit_obligation = judgment(
        context,
        91,
        delta_admit,
        AdmissionJudgmentClaim::Obligation {
            obligation: obligation_admit,
            status: ObligationStatus::Satisfied,
        },
        admit_producer,
        187,
    );
    let reject_verdict = judgment(
        context,
        92,
        delta_reject,
        AdmissionJudgmentClaim::Verdict(AdmissionDisposition::Reject),
        reject_producer,
        188,
    );
    let reject_obligation = judgment(
        context,
        93,
        delta_reject,
        AdmissionJudgmentClaim::Obligation {
            obligation: obligation_reject,
            status: ObligationStatus::Unsatisfied,
        },
        reject_producer,
        189,
    );

    let successor = StateRevision {
        id: successor_id,
        session: context.session,
        predecessor: Some(context.initial_state),
        cause: StateRevisionCause::Admission {
            occurrence: id!(AdmissionOccurrenceId, 94),
            run: admit_producer.run,
            activation: admit_producer.activation,
            step: admit_producer.step,
        },
        payload: term("world/admitted"),
        canonical_state_snapshot: canonical_term_bytes(&term("world/admitted"))
            .expect("successor State payload is canonical")
            .into_boxed_slice(),
        policy: context.policy,
        semantics: scope().semantics,
    };

    let mut activation_25 = root_activation(
        context,
        25,
        35,
        45,
        1,
        RootTrigger::External(id!(ExternalTriggerOccurrenceId, 12)),
    );
    require_formation_observation(&mut activation_25, context, 86);
    let mut activation_26 = root_activation(
        context,
        26,
        36,
        46,
        1,
        RootTrigger::External(id!(ExternalTriggerOccurrenceId, 12)),
    );
    require_formation_observation(&mut activation_26, context, 87);

    let child_left_step = step(
        54,
        35,
        25,
        45,
        64,
        budget(100, 10, 90),
        vec![StepCause::ActivationStart(id!(ActivationId, 25))],
        StepOutcomeProposalV2::Return(domain_bound("child/left", 86)),
    );
    let child_right_step = step(
        55,
        36,
        26,
        46,
        65,
        budget(100, 10, 90),
        vec![StepCause::ActivationStart(id!(ActivationId, 26))],
        StepOutcomeProposalV2::Return(domain_bound("child/right", 87)),
    );
    let resumed_return_step = step(
        58,
        30,
        20,
        61,
        68,
        budget(80, 10, 70),
        vec![StepCause::ContinuationTakeup {
            continuation: id!(ContinuationId, 70),
            occurrence: ContinuationTakeupOccurrence::Resumption(id!(ResumptionOccurrenceId, 83)),
        }],
        StepOutcomeProposalV2::Return(domain_bound("value/resumed", 88)),
    );

    let records = vec![
        ProcessRecordV2::ExternalTrigger(ExternalTriggerOccurrenceV2 {
            id: id!(ExternalTriggerOccurrenceId, 10),
            provenance: entered_through(context.pure_boundary, 181, vec![]),
        }),
        ProcessRecordV2::ExternalTrigger(ExternalTriggerOccurrenceV2 {
            id: id!(ExternalTriggerOccurrenceId, 11),
            provenance: entered_through(context.pure_boundary, 182, vec![]),
        }),
        ProcessRecordV2::ExternalTrigger(ExternalTriggerOccurrenceV2 {
            id: id!(ExternalTriggerOccurrenceId, 12),
            provenance: entered_through(context.pure_boundary, 183, vec![]),
        }),
        ProcessRecordV2::Activation(activation_22),
        ProcessRecordV2::Steps(vec![checker_step]),
        ProcessRecordV2::Activation(activation_20),
        ProcessRecordV2::Activation(activation_21),
        ProcessRecordV2::Activation(activation_23),
        ProcessRecordV2::Activation(activation_24),
        ProcessRecordV2::Steps(vec![first]),
        ProcessRecordV2::Steps(vec![step(
            51,
            30,
            20,
            60,
            61,
            budget(90, 10, 80),
            vec![],
            StepOutcomeProposalV2::Suspend(continuation(context)),
        )]),
        ProcessRecordV2::Steps(vec![step(
            52,
            31,
            21,
            41,
            62,
            budget(100, 10, 90),
            vec![StepCause::ActivationStart(id!(ActivationId, 21))],
            StepOutcomeProposalV2::Progress,
        )]),
        ProcessRecordV2::Activation(activation_25),
        ProcessRecordV2::Activation(activation_26),
        ProcessRecordV2::Steps(vec![child_left_step]),
        ProcessRecordV2::Steps(vec![child_right_step]),
        ProcessRecordV2::Steps(vec![state_admit_step]),
        ProcessRecordV2::Steps(vec![state_reject_step]),
        ProcessRecordV2::EnteredObservation(EnteredObservationV2 {
            observation: ObservationProposalV2::Value {
                id: id!(ObservationId, 82),
                value: term("observation/fresh-resumption-input"),
                supports: vec![],
            },
            provenance: entered_through(
                context.pure_boundary,
                184,
                vec![CausalRef::Step(a1_suspend)],
            ),
        }),
        ProcessRecordV2::Resumption(ResumptionOccurrenceV2 {
            body: ResumptionOccurrenceBodyV2 {
                id: id!(ResumptionOccurrenceId, 83),
                continuation: id!(ContinuationId, 70),
                run: id!(RunId, 30),
                activation: id!(ActivationId, 20),
                pins: continuation(context).pins,
            },
            provenance: OccurrenceProvenance::EnteredThrough(entered_through(
                context.pure_boundary,
                185,
                vec![CausalRef::Observation(id!(ObservationId, 82))],
            )),
        }),
        ProcessRecordV2::Steps(vec![resumed_return_step]),
        ProcessRecordV2::Judgment(admit_verdict),
        ProcessRecordV2::Judgment(admit_obligation),
        ProcessRecordV2::Judgment(reject_verdict),
        ProcessRecordV2::Judgment(reject_obligation),
        ProcessRecordV2::AdmissionDecision(StateAdmissionDecisionV2 {
            occurrence: id!(AdmissionOccurrenceId, 94),
            delta: delta_admit,
            authorization: AdmissionAuthorizationEvidence::IrreducibleRoot {
                policy: context.root_policy,
                authorization: RootAdmissionAuthorizationRef {
                    policy: context.root_policy,
                    local: AdmissionAuthorizationLocalId::new(1),
                },
            },
            evidence: vec![
                SupportUse {
                    slot: SupportSlotId::new(0),
                    role: term("judgment/verdict"),
                    source: SupportSource::Judgment(id!(JudgmentOccurrenceId, 90)),
                },
                SupportUse {
                    slot: SupportSlotId::new(1),
                    role: term("judgment/obligation"),
                    source: SupportSource::Judgment(id!(JudgmentOccurrenceId, 91)),
                },
            ],
            verdict: id!(JudgmentOccurrenceId, 90),
            obligation_judgments: vec![ObligationJudgmentUse {
                obligation: obligation_admit,
                judgment: id!(JudgmentOccurrenceId, 91),
            }],
            provenance: entered_through(
                context.state_boundary,
                190,
                vec![
                    CausalRef::CandidateDelta(delta_admit),
                    CausalRef::Judgment(id!(JudgmentOccurrenceId, 90)),
                    CausalRef::Judgment(id!(JudgmentOccurrenceId, 91)),
                ],
            ),
            outcome: StateAdmissionOutcomeV2::Admit(successor),
        }),
        ProcessRecordV2::AdmissionDecision(StateAdmissionDecisionV2 {
            occurrence: id!(AdmissionOccurrenceId, 95),
            delta: delta_reject,
            authorization: AdmissionAuthorizationEvidence::IrreducibleRoot {
                policy: context.root_policy,
                authorization: RootAdmissionAuthorizationRef {
                    policy: context.root_policy,
                    local: AdmissionAuthorizationLocalId::new(2),
                },
            },
            evidence: vec![
                SupportUse {
                    slot: SupportSlotId::new(0),
                    role: term("judgment/verdict"),
                    source: SupportSource::Judgment(id!(JudgmentOccurrenceId, 92)),
                },
                SupportUse {
                    slot: SupportSlotId::new(1),
                    role: term("judgment/obligation"),
                    source: SupportSource::Judgment(id!(JudgmentOccurrenceId, 93)),
                },
            ],
            verdict: id!(JudgmentOccurrenceId, 92),
            obligation_judgments: vec![ObligationJudgmentUse {
                obligation: obligation_reject,
                judgment: id!(JudgmentOccurrenceId, 93),
            }],
            provenance: entered_through(
                context.state_boundary,
                191,
                vec![
                    CausalRef::CandidateDelta(delta_reject),
                    CausalRef::Judgment(id!(JudgmentOccurrenceId, 92)),
                    CausalRef::Judgment(id!(JudgmentOccurrenceId, 93)),
                ],
            ),
            outcome: StateAdmissionOutcomeV2::Reject(AdmissionRejectionV2 {
                reason: term("rejected/unsatisfied-obligation"),
            }),
        }),
    ];

    (
        ProcessPackageV2 {
            claimed_snapshot,
            snapshot,
            initial_state_views: vec![InitialStateViewV2 {
                session: context.session,
                payload: term("world/initial"),
                canonical_state_snapshot: canonical_term_bytes(&term("world/initial"))
                    .expect("initial State payload is canonical")
                    .into_boxed_slice(),
            }],
            records,
        },
        context,
    )
}

fn establish_core_authority(
    checked: &CheckedProcessPackage,
    context: CoreContext,
    with_external_provenance: bool,
) -> AuthorityStore {
    let delta_admit = id!(CandidateDeltaId, 80);
    let delta_reject = id!(CandidateDeltaId, 81);
    let root_genesis = RootAdmissionAuthorizationRef {
        policy: context.root_policy,
        local: AdmissionAuthorizationLocalId::new(0),
    };
    let mut authority = AuthorityStore::new();
    authority
        .establish_root_policy(
            RootPolicyAnchor::establish_with_governance(
                context.root_policy,
                vec![RootGenesisGrant {
                    authorization: root_genesis,
                    scope: RootGenesisScope {
                        semantics: scope().semantics,
                        program: context.revision.preimage.program,
                        snapshot: context.snapshot,
                        change: context.revision.preimage.change,
                    },
                }],
                vec![],
                vec![
                    RootStateAdmissionGrant {
                        authorization: RootAdmissionAuthorizationRef {
                            policy: context.root_policy,
                            local: AdmissionAuthorizationLocalId::new(1),
                        },
                        scope: CheckedStateAdmissionScope {
                            package: checked.id(),
                            session: context.session,
                            base: context.initial_state,
                            delta: delta_admit,
                        },
                    },
                    RootStateAdmissionGrant {
                        authorization: RootAdmissionAuthorizationRef {
                            policy: context.root_policy,
                            local: AdmissionAuthorizationLocalId::new(2),
                        },
                        scope: CheckedStateAdmissionScope {
                            package: checked.id(),
                            session: context.session,
                            base: context.initial_state,
                            delta: delta_reject,
                        },
                    },
                ],
                vec![RootJudgmentAuthorityGrant {
                    authority: RootJudgmentAuthorityRef {
                        policy: context.root_policy,
                        local: JudgmentAuthorityLocalId::new(0),
                    },
                    scope: JudgmentAuthorityScope {
                        semantics: scope().semantics,
                        session: context.session,
                        policy: context.policy,
                    },
                }],
            )
            .expect("root policy is coherent"),
        )
        .expect("root policy is established once");
    authority
        .admit_genesis(
            context.revision,
            checked.authority_input(),
            context.root_policy,
            root_genesis,
        )
        .expect("external root admits the checked snapshot");
    let initial_state_snapshot = canonical_term_bytes(&term("world/initial"))
        .expect("initial State payload has canonical bytes");
    authority
        .establish_runtime_session(RuntimeSessionAnchor::establish(
            context.session,
            context.revision.id,
            scope().semantics,
            context.policy,
            context.session_start,
            initial_state_snapshot,
        ))
        .expect("runtime session is externally established");
    if with_external_provenance {
        authority
            .establish_boundary(BoundaryAnchor {
                boundary: context.pure_boundary,
                semantics: scope().semantics,
                snapshot: context.snapshot,
                program_revision: context.revision.id,
                runtime_session: None,
                runtime_policy: None,
                permits: vec![
                    EnteredOccurrenceKind::ExternalTrigger,
                    EnteredOccurrenceKind::Resumption,
                    EnteredOccurrenceKind::Handoff,
                    EnteredOccurrenceKind::Cancellation,
                    EnteredOccurrenceKind::Observation,
                ],
            })
            .expect("pure boundary is externally established");
        authority
            .establish_boundary(BoundaryAnchor {
                boundary: context.state_boundary,
                semantics: scope().semantics,
                snapshot: context.snapshot,
                program_revision: context.revision.id,
                runtime_session: Some(context.session),
                runtime_policy: Some(context.policy),
                permits: vec![
                    EnteredOccurrenceKind::Judgment,
                    EnteredOccurrenceKind::AdmissionDecision,
                ],
            })
            .expect("state boundary is externally established");
        for (evidence_tag, boundary) in [
            (181, context.pure_boundary),
            (182, context.pure_boundary),
            (183, context.pure_boundary),
            (184, context.pure_boundary),
            (185, context.pure_boundary),
            (186, context.state_boundary),
            (187, context.state_boundary),
            (188, context.state_boundary),
            (189, context.state_boundary),
            (190, context.state_boundary),
            (191, context.state_boundary),
            (192, context.state_boundary),
            (196, context.pure_boundary),
            (197, context.pure_boundary),
        ] {
            authority
                .establish_evidence(EvidenceAnchor {
                    evidence: id!(ExternalEvidenceRef, evidence_tag),
                    boundary,
                    exact_evidence: vec![evidence_tag].into_boxed_slice(),
                })
                .expect("external evidence identity is established once");
        }
    }
    authority
}

fn finalized_core_package() -> (ProcessPackageV2, CoreContext) {
    finalized_core_package_from_snapshot(core_snapshot())
}

fn finalized_core_package_from_snapshot(
    snapshot: ProgramSnapshotPreimageV2,
) -> (ProcessPackageV2, CoreContext) {
    let (candidate, context) =
        build_core_package_from_snapshot(snapshot.clone(), id!(StateRevisionId, 200));
    let bytes = encode_process_package(&candidate).expect("candidate core package encodes");
    let checked = check_process_package(
        decode_process_package(&bytes).expect("candidate core package decodes"),
    )
    .expect("candidate core package checks before State authority");
    let authority = establish_core_authority(&checked, context, true);
    let derived = match ProcessCarrier::replay(&checked, &authority) {
        Err(ProcessError::StateRevisionIdMismatch { derived, .. }) => derived,
        other => panic!("dummy successor must expose only its derived identity: {other:?}"),
    };
    build_core_package_from_snapshot(snapshot, derived)
}

fn checked_core() -> (CheckedProcessPackage, CoreContext, AuthorityStore) {
    let (candidate, context) = finalized_core_package();
    let bytes = encode_process_package(&candidate).expect("final core package encodes");
    let checked =
        check_process_package(decode_process_package(&bytes).expect("final core package decodes"))
            .expect("final core package checks");
    let authority = establish_core_authority(&checked, context, true);
    (checked, context, authority)
}

fn replay_core_candidate(
    candidate: &ProcessPackageV2,
    context: CoreContext,
) -> Result<ProcessCarrier, ProcessError> {
    let bytes = encode_process_package(candidate).expect("mutated core package encodes");
    let checked = check_process_package(
        decode_process_package(&bytes).expect("mutated core package decodes"),
    )
    .expect("mutated core package retains a valid constitution");
    let authority = establish_core_authority(&checked, context, true);
    ProcessCarrier::replay(&checked, &authority)
}

fn prerequisite_candidate(
    occurrence_kind: ActivationPrerequisiteKind,
    prerequisite_scope: PrerequisiteScope,
    prerequisite: ActivationPrerequisite,
    include_observation: bool,
) -> (ProcessPackageV2, CoreContext) {
    let mut snapshot = core_snapshot();
    snapshot
        .constitution
        .formations
        .push(FormationJudgmentPreimageV2 {
            id: FormationLocalId::new(3),
            context: vec![],
            term: term("activation/prerequisite"),
            target: target("activation-prerequisite"),
            direct_dependencies: vec![],
        });
    snapshot.constitution.operators[0].modes[0].dynamic_prerequisites =
        vec![DynamicPrerequisiteRequirementPreimageV2 {
            slot: PrerequisiteLocalId::new(1),
            role: Some(RoleLocalId::new(1)),
            requirement: occurrence_kind,
            expected: FormationLocalId::new(2),
            scope: prerequisite_scope,
            cardinality: one(),
            cause_projection: vec![CauseProjectionEntryV2 {
                component: CauseComponentLocalId::new(1),
                path: PrerequisiteOccurrencePathV2::BoundOccurrence,
            }],
        }];
    let claimed_snapshot =
        derive_program_snapshot_id(&snapshot).expect("prerequisite snapshot identity derives");
    let context = core_context(claimed_snapshot);
    let mut activation = root_activation(
        context,
        20,
        30,
        40,
        1,
        RootTrigger::External(id!(ExternalTriggerOccurrenceId, 10)),
    );
    let slot = PrerequisiteSlotId {
        mode: activation.mode,
        local: PrerequisiteLocalId::new(1),
    };
    activation.prerequisite_bindings = vec![DynamicPrerequisiteBindingV2 {
        slot,
        ordinal: 0,
        value: prerequisite,
    }];
    activation.causes.prerequisite_occurrences = vec![ActivationOccurrenceCauseV2 {
        slot,
        ordinal: 0,
        component: CauseComponentLocalId::new(1),
        occurrence: prerequisite,
    }];
    let mut records = vec![ProcessRecordV2::ExternalTrigger(
        ExternalTriggerOccurrenceV2 {
            id: id!(ExternalTriggerOccurrenceId, 10),
            provenance: entered_through(context.pure_boundary, 181, vec![]),
        },
    )];
    if include_observation {
        records.push(ProcessRecordV2::EnteredObservation(EnteredObservationV2 {
            observation: ObservationProposalV2::Value {
                id: id!(ObservationId, 82),
                value: term("activation/prerequisite-observation"),
                supports: vec![],
            },
            provenance: entered_through(
                context.pure_boundary,
                184,
                vec![CausalRef::ExternalTrigger(id!(
                    ExternalTriggerOccurrenceId,
                    10
                ))],
            ),
        }));
    }
    records.push(ProcessRecordV2::Activation(activation));
    (
        ProcessPackageV2 {
            claimed_snapshot,
            snapshot,
            initial_state_views: vec![],
            records,
        },
        context,
    )
}

fn bounded_exhaustion_candidate(remaining_units: u64) -> (ProcessPackageV2, CoreContext) {
    let mut snapshot = core_snapshot();
    snapshot
        .constitution
        .formations
        .push(FormationJudgmentPreimageV2 {
            id: FormationLocalId::new(3),
            context: vec![],
            term: term("resource/step-budget"),
            target: target("resource"),
            direct_dependencies: vec![],
        });
    let contract = &mut snapshot.constitution.operators[0].modes[0].contract;
    contract.productivity = ProductivityContractV2 {
        kind: ProductivityKindV2::Bounded,
        obligations: vec![FormationLocalId::new(3)],
    };
    contract.resource_requirements = vec![FormationLocalId::new(3)];
    contract.budget_exhaustion_domain = Some(target("budget-exhaustion"));
    contract.formation_checks.push(target("budget-exhaustion"));
    contract.formation_checks.sort_unstable();
    for application in &mut snapshot.constitution.applications {
        application
            .form
            .dependency_closure
            .push(LocalSemanticDependencyV2::Formation(FormationLocalId::new(
                3,
            )));
        application.form.dependency_closure.sort_unstable();
    }
    let claimed_snapshot =
        derive_program_snapshot_id(&snapshot).expect("bounded snapshot identity derives");
    let context = core_context(claimed_snapshot);
    let mut activation = root_activation(
        context,
        20,
        30,
        40,
        1,
        RootTrigger::External(id!(ExternalTriggerOccurrenceId, 10)),
    );
    let resource = FormationRefV2 {
        snapshot: claimed_snapshot,
        local: FormationLocalId::new(3),
    };
    activation
        .pins
        .constitutive_dependencies
        .push(SemanticDependencyV2::Formation(resource));
    activation.pins.constitutive_dependencies.sort_unstable();
    activation.pins.resource_requirements = vec![resource];
    require_formation_observation(&mut activation, context, 84);
    let mut checker = root_activation(
        context,
        21,
        31,
        41,
        1,
        RootTrigger::External(id!(ExternalTriggerOccurrenceId, 10)),
    );
    checker
        .pins
        .constitutive_dependencies
        .push(SemanticDependencyV2::Formation(resource));
    checker.pins.constitutive_dependencies.sort_unstable();
    checker.pins.resource_requirements = vec![resource];
    let mut checker_step = step(
        49,
        31,
        21,
        41,
        59,
        budget(100, 1, 99),
        vec![StepCause::ActivationStart(id!(ActivationId, 21))],
        StepOutcomeProposalV2::Progress,
    );
    checker_step.observation_outcomes =
        vec![StepObservationOutcomeV2::Observed(formation_observation(
            84,
            "budget/exhausted",
            target("budget-exhaustion"),
            SupportSource::ExternalTrigger(id!(ExternalTriggerOccurrenceId, 10)),
        ))];
    let exhaustion = step(
        50,
        30,
        20,
        40,
        60,
        budget(100, 100 - remaining_units, remaining_units),
        vec![StepCause::ActivationStart(id!(ActivationId, 20))],
        StepOutcomeProposalV2::BudgetExhausted {
            exhaustion: domain_bound("budget/exhausted", 84),
            continuation: None,
            obligations: vec![term("resource/step-budget")],
        },
    );
    (
        ProcessPackageV2 {
            claimed_snapshot,
            snapshot,
            initial_state_views: vec![],
            records: vec![
                ProcessRecordV2::ExternalTrigger(ExternalTriggerOccurrenceV2 {
                    id: id!(ExternalTriggerOccurrenceId, 10),
                    provenance: entered_through(context.pure_boundary, 181, vec![]),
                }),
                ProcessRecordV2::Activation(checker),
                ProcessRecordV2::Steps(vec![checker_step]),
                ProcessRecordV2::Activation(activation),
                ProcessRecordV2::Steps(vec![exhaustion]),
            ],
        },
        context,
    )
}

fn cancellable_core_package() -> (ProcessPackageV2, CoreContext) {
    let mut snapshot = core_snapshot();
    let contract = &mut snapshot.constitution.operators[0].modes[0].contract;
    let ContinuationContractV2::Suspensible { may_cancel, .. } = &mut contract.continuation else {
        panic!("core Mode 1 is suspensible");
    };
    *may_cancel = true;
    contract.failure_domain = Some(target("cancellation"));
    finalized_core_package_from_snapshot(snapshot)
}

fn ready_cancellation_candidate() -> (ProcessPackageV2, CoreContext) {
    let (mut package, context) = cancellable_core_package();
    let step_52_index = package
        .records
        .iter()
        .position(|record| {
            matches!(record, ProcessRecordV2::Steps(steps)
                if steps.iter().any(|step| step.id == id!(StepId, 52)))
        })
        .expect("core has ready Activation 21's first Step");
    package.records.truncate(step_52_index);
    package
        .records
        .push(ProcessRecordV2::Cancellation(CancellationOccurrenceV2 {
            body: CancellationOccurrenceBodyV2 {
                id: id!(CancellationOccurrenceId, 116),
                target: CancellationTarget::Activation(id!(ActivationId, 21)),
                pins: pure_pins(context),
            },
            provenance: OccurrenceProvenance::EnteredThrough(entered_through(
                context.pure_boundary,
                196,
                vec![CausalRef::ExternalTrigger(id!(
                    ExternalTriggerOccurrenceId,
                    11
                ))],
            )),
        }));
    package.records.push(ProcessRecordV2::Steps(vec![step(
        52,
        31,
        21,
        41,
        62,
        budget(100, 10, 90),
        vec![
            StepCause::ActivationStart(id!(ActivationId, 21)),
            StepCause::CancellationRequest(id!(CancellationOccurrenceId, 116)),
        ],
        StepOutcomeProposalV2::Cancel(id!(CancellationOccurrenceId, 116)),
    )]));
    (package, context)
}

#[test]
fn process_v2_core_exercises_process_identity_lifecycle_truth_and_admission() {
    let (checked, context, authority) = checked_core();
    let carrier = ProcessCarrier::replay(&checked, &authority).expect("core package replays");

    let first_application = core_application(context.snapshot, 1);
    let second_application = core_application(context.snapshot, 2);
    assert_eq!(carrier.application_count(), 2);
    assert_eq!(
        carrier.application(first_application).unwrap().shape(),
        carrier.application(second_application).unwrap().shape()
    );
    assert_eq!(carrier.activation_count(), 7);
    assert_eq!(carrier.run_count(), 7);
    assert_eq!(carrier.step_count(), 9);
    assert_eq!(carrier.observation_count(), 8);
    assert_eq!(carrier.continuation_count(), 1);
    assert_eq!(carrier.candidate_delta_count(), 2);
    assert_eq!(carrier.decision_count(), 2);
    assert_eq!(carrier.state_revision_count(), 2);
    assert!(
        carrier
            .step(id!(StepId, 51))
            .expect("serial Step 51 remains inspectable")
            .proposal()
            .causes
            .is_empty(),
        "configuration succession does not manufacture semantic causality"
    );
    assert_eq!(
        carrier
            .configuration(id!(ConfigurationId, 61))
            .expect("serial successor Configuration remains inspectable")
            .predecessor,
        ConfigurationPredecessorV2::ConfigurationAfter(StepRef {
            run: id!(RunId, 30),
            activation: id!(ActivationId, 20),
            step: id!(StepId, 51),
        })
    );
    assert_eq!(
        carrier
            .configuration(id!(ConfigurationId, 40))
            .expect("initial Configuration remains inspectable")
            .predecessor,
        ConfigurationPredecessorV2::ActivationStart(id!(ActivationId, 20))
    );
    assert_eq!(
        carrier.activation(id!(ActivationId, 20)).unwrap().status(),
        ActivationStatus::Terminal(ActivationTerminal::Returned)
    );
    assert_eq!(
        carrier.activation(id!(ActivationId, 21)).unwrap().status(),
        ActivationStatus::Live
    );
    assert!(
        carrier
            .continuation(id!(ContinuationId, 70))
            .unwrap()
            .consumed()
    );
    assert_eq!(carrier.run_members(id!(RunId, 32)).unwrap().len(), 1);
    assert!(matches!(
        carrier.observation(id!(ObservationId, 50)).unwrap().content,
        ObservationContentV2::Truth {
            verdict: TruthVerdict::True,
            ..
        }
    ));
    assert!(matches!(
        carrier.observation(id!(ObservationId, 51)).unwrap().content,
        ObservationContentV2::Truth {
            verdict: TruthVerdict::False,
            ..
        }
    ));
    assert!(matches!(
        carrier.decision(id!(CandidateDeltaId, 80)).unwrap().outcome,
        StateAdmissionOutcomeV2::Admit(_)
    ));
    assert!(matches!(
        carrier.decision(id!(CandidateDeltaId, 81)).unwrap().outcome,
        StateAdmissionOutcomeV2::Reject(_)
    ));
}

fn self_authorizing_admission_package() -> ProcessPackageV2 {
    let mut snapshot = empty_snapshot();
    snapshot
        .state_admission_grants
        .push(RevisionStateAdmissionGrantPreimageV2 {
            authorization: AdmissionAuthorizationLocalId::new(7),
            scope: StateAdmissionScope {
                session: id!(RuntimeSessionId, 7),
                base: id!(StateRevisionId, 8),
                delta: id!(CandidateDeltaId, 9),
            },
        });
    let claimed_snapshot =
        derive_program_snapshot_id(&snapshot).expect("self-authority snapshot derives");
    ProcessPackageV2 {
        claimed_snapshot,
        snapshot,
        initial_state_views: vec![],
        records: vec![],
    }
}

fn exact_byte_substitution_package() -> (ProcessPackageV2, CoreContext) {
    let (mut package, context) = finalized_core_package();
    let Some(ProcessRecordV2::AdmissionDecision(decision)) = package.records.last_mut() else {
        panic!("core package ends in its governed rejection");
    };
    let StateAdmissionOutcomeV2::Reject(rejection) = &mut decision.outcome else {
        panic!("last core decision is the rejection");
    };
    rejection.reason = term("rejected/byte-substitution");
    (package, context)
}

fn open_form_shape_package() -> ProcessPackageV2 {
    let mut snapshot = two_application_snapshot();
    snapshot.constitution.applications[0].form.bindings[0].value =
        RoleBindingValuePreimageV2::Produced;
    let claimed_snapshot =
        derive_program_snapshot_id(&snapshot).expect("open form candidate still encodes");
    ProcessPackageV2 {
        claimed_snapshot,
        snapshot,
        initial_state_views: vec![],
        records: vec![],
    }
}

fn unconstituted_external_trigger_package() -> ProcessPackageV2 {
    let mut package = empty_package();
    package.records.push(ProcessRecordV2::ExternalTrigger(
        ExternalTriggerOccurrenceV2 {
            id: id!(ExternalTriggerOccurrenceId, 10),
            provenance: EnteredThrough {
                boundary: id!(BoundaryRef, 11),
                evidence: id!(ExternalEvidenceRef, 12),
                causes: vec![],
            },
        },
    ));
    package
}

fn resume_missing_fresh_ingress_package() -> (ProcessPackageV2, CoreContext) {
    let (mut package, context) = finalized_core_package();
    let resumption = package
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Resumption(value) => Some(value),
            _ => None,
        })
        .expect("core package contains one resumption");
    resumption.provenance = OccurrenceProvenance::ProducedBy(StepRef {
        run: id!(RunId, 30),
        activation: id!(ActivationId, 20),
        step: id!(StepId, 51),
    });
    (package, context)
}

fn handoff_child_activation(
    context: CoreContext,
    activation_tag: u8,
    configuration_tag: u8,
    handoff_tag: u8,
) -> ActivationProposalV2 {
    let mut pins = pure_pins(context);
    pins.budget = Budget {
        remaining_units: 80,
    };
    ActivationProposalV2 {
        id: id!(ActivationId, activation_tag),
        application: core_application(context.snapshot, 1),
        mode: core_mode(context.snapshot, 1),
        pins,
        static_basis: ActivationStaticBasis {
            execution_authorizations: vec![],
            judgment_authorities: vec![],
        },
        prerequisite_bindings: vec![],
        causes: ActivationCauseFrontierV2 {
            origin: ActivationOrigin::HandoffFrom {
                run: id!(RunId, 30),
                parent_activation: id!(ActivationId, 20),
                parent_step: id!(StepId, 51),
                continuation: id!(ContinuationId, 70),
                handoff: id!(HandoffOccurrenceId, handoff_tag),
            },
            prerequisite_occurrences: vec![],
        },
        membership: RunMembership::ChildIn(id!(RunId, 30)),
        initial_configuration: ConfigurationProposal {
            id: id!(ConfigurationId, configuration_tag),
            value: term("configuration/61"),
        },
    }
}

fn linear_double_takeup_package() -> (ProcessPackageV2, CoreContext) {
    let (mut package, context) = finalized_core_package();
    let suspension_index = package
        .records
        .iter()
        .position(|record| {
            matches!(
                record,
                ProcessRecordV2::Steps(steps)
                    if steps.iter().any(|step| step.id == id!(StepId, 51))
            )
        })
        .expect("core package contains its suspension");
    package.records.truncate(suspension_index + 1);

    for (handoff_tag, evidence_tag) in [(110, 196), (111, 197)] {
        package
            .records
            .push(ProcessRecordV2::Handoff(HandoffOccurrenceV2 {
                body: HandoffOccurrenceBodyV2 {
                    id: id!(HandoffOccurrenceId, handoff_tag),
                    continuation: id!(ContinuationId, 70),
                    run: id!(RunId, 30),
                    activation: id!(ActivationId, 20),
                    pins: continuation(context).pins,
                },
                provenance: OccurrenceProvenance::EnteredThrough(entered_through(
                    context.pure_boundary,
                    evidence_tag,
                    vec![CausalRef::Step(StepRef {
                        run: id!(RunId, 30),
                        activation: id!(ActivationId, 20),
                        step: id!(StepId, 51),
                    })],
                )),
            }));
    }
    for (handoff_tag, child_tag, configuration_tag) in [(110, 112, 113), (111, 114, 115)] {
        package
            .records
            .push(ProcessRecordV2::Activation(handoff_child_activation(
                context,
                child_tag,
                configuration_tag,
                handoff_tag,
            )));
    }
    (package, context)
}

fn admission_redicision_package() -> (ProcessPackageV2, CoreContext) {
    let (mut package, context) = finalized_core_package();
    let mut duplicate = package
        .records
        .iter()
        .find_map(|record| match record {
            ProcessRecordV2::AdmissionDecision(decision)
                if decision.delta == id!(CandidateDeltaId, 80) =>
            {
                Some(decision.clone())
            }
            _ => None,
        })
        .expect("core package contains admitted decision");
    duplicate.occurrence = id!(AdmissionOccurrenceId, 96);
    duplicate.provenance.evidence = id!(ExternalEvidenceRef, 192);
    package
        .records
        .push(ProcessRecordV2::AdmissionDecision(duplicate));
    (package, context)
}

fn collapse_support_slot(bytes: &[u8]) -> Vec<u8> {
    let mut mutated = bytes.to_vec();
    let from = 0x5566_7788_u32.to_be_bytes();
    let to = 0x1122_3344_u32.to_be_bytes();
    let offsets: Vec<_> = mutated
        .windows(from.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == from).then_some(offset))
        .collect();
    assert_eq!(
        offsets.len(),
        1,
        "support slot marker is unique in core bytes"
    );
    mutated[offsets[0]..offsets[0] + from.len()].copy_from_slice(&to);
    mutated
}

fn corpus_bytes() -> Vec<(&'static str, Vec<u8>)> {
    let (core, _) = finalized_core_package();
    let core_bytes = encode_process_package(&core).expect("core corpus package encodes");
    let (substitution, _) = exact_byte_substitution_package();
    let (missing_ingress, _) = resume_missing_fresh_ingress_package();
    let (double_takeup, _) = linear_double_takeup_package();
    let (redicision, _) = admission_redicision_package();
    vec![
        ("positive/process-v2-core.hex", core_bytes.clone()),
        (
            "negative/self-authorizing-admission.hex",
            encode_process_package(&self_authorizing_admission_package())
                .expect("self-authority candidate encodes"),
        ),
        (
            "negative/exact-byte-substitution.hex",
            encode_process_package(&substitution).expect("substitution package encodes"),
        ),
        (
            "negative/open-form-shape.hex",
            encode_process_package(&open_form_shape_package())
                .expect("open form candidate encodes"),
        ),
        (
            "negative/unconstituted-external-trigger.hex",
            encode_process_package(&unconstituted_external_trigger_package())
                .expect("unconstituted trigger package encodes"),
        ),
        (
            "negative/resume-missing-fresh-ingress.hex",
            encode_process_package(&missing_ingress).expect("missing-ingress package encodes"),
        ),
        (
            "negative/linear-double-takeup.hex",
            encode_process_package(&double_takeup).expect("double-takeup package encodes"),
        ),
        (
            "negative/support-collapse.hex",
            collapse_support_slot(&core_bytes),
        ),
        (
            "negative/admission-redicision.hex",
            encode_process_package(&redicision).expect("redicision package encodes"),
        ),
    ]
}

fn frozen_hex(source: &str) -> Vec<u8> {
    let digits: Vec<u8> = source
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    assert_eq!(digits.len() % 2, 0, "hex has complete octets");
    digits
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).expect("hex digit") as u8;
            let low = (pair[1] as char).to_digit(16).expect("hex digit") as u8;
            (high << 4) | low
        })
        .collect()
}

fn formatted_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2 + bytes.len() / 32 + 1);
    for (index, byte) in bytes.iter().enumerate() {
        if index != 0 && index % 32 == 0 {
            output.push('\n');
        }
        write!(&mut output, "{byte:02x}").expect("String writes cannot fail");
    }
    output.push('\n');
    output
}

fn lowercase_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("String writes cannot fail");
    }
    output
}

struct CoreCorpusOracles {
    program_snapshot: ProgramSnapshotId,
    shared_application_shape: ApplicationShapeId,
    process_package: ProcessPackageId,
    program_revision: ProgramRevisionId,
    initial_state: StateRevisionId,
    successor_state: StateRevisionId,
    observations: [(&'static str, usize); 9],
}

fn core_corpus_oracles() -> CoreCorpusOracles {
    let (checked, context, authority) = checked_core();
    let first_shape = checked
        .constitution()
        .application_shape(ApplicationLocalId::new(1))
        .expect("core Application 1 has a checked shape");
    let second_shape = checked
        .constitution()
        .application_shape(ApplicationLocalId::new(2))
        .expect("core Application 2 has a checked shape");
    assert_eq!(
        first_shape, second_shape,
        "the core Applications deliberately share one structural shape"
    );
    let successor_states: Vec<_> = checked
        .records()
        .iter()
        .filter_map(|record| match record {
            ProcessRecordV2::AdmissionDecision(StateAdmissionDecisionV2 {
                outcome: StateAdmissionOutcomeV2::Admit(state),
                ..
            }) => Some(state.id),
            _ => None,
        })
        .collect();
    assert_eq!(
        successor_states.len(),
        1,
        "the core corpus has exactly one admitted successor State"
    );
    let carrier = ProcessCarrier::replay(&checked, &authority).expect("core corpus replays");
    CoreCorpusOracles {
        program_snapshot: context.snapshot,
        shared_application_shape: first_shape,
        process_package: checked.id(),
        program_revision: context.revision.id,
        initial_state: context.initial_state,
        successor_state: successor_states[0],
        observations: [
            ("applications", carrier.application_count()),
            ("activations", carrier.activation_count()),
            ("runs", carrier.run_count()),
            ("steps", carrier.step_count()),
            ("identified_observations", carrier.observation_count()),
            ("continuations", carrier.continuation_count()),
            ("candidate_deltas", carrier.candidate_delta_count()),
            ("admission_decisions", carrier.decision_count()),
            ("state_revisions", carrier.state_revision_count()),
        ],
    }
}

fn parse_checksum_entries(source: &str) -> std::collections::BTreeMap<String, String> {
    let mut entries = std::collections::BTreeMap::new();
    for (line_index, line) in source.lines().enumerate() {
        assert!(!line.is_empty(), "SHA256SUMS contains an empty line");
        let (digest, relative) = line
            .split_once("  ")
            .unwrap_or_else(|| panic!("SHA256SUMS line {} is malformed", line_index + 1));
        assert_eq!(digest.len(), 64, "SHA-256 digest width is exact");
        assert!(
            digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "SHA-256 digest is lowercase hexadecimal"
        );
        assert!(
            relative.ends_with(".hex")
                && (relative.starts_with("negative/") || relative.starts_with("positive/")),
            "SHA256SUMS path is a corpus transport"
        );
        assert!(
            entries
                .insert(relative.to_owned(), digest.to_owned())
                .is_none(),
            "SHA256SUMS duplicates {relative}"
        );
    }
    entries
}

fn disk_transport_paths() -> std::collections::BTreeSet<String> {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-vectors/process-v2");
    let mut paths = std::collections::BTreeSet::new();
    for section in ["negative", "positive"] {
        for entry in std::fs::read_dir(root.join(section)).expect("corpus directory is readable") {
            let entry = entry.expect("corpus directory entry is readable");
            assert!(
                entry
                    .file_type()
                    .expect("corpus file type is readable")
                    .is_file(),
                "corpus sections contain only transport files"
            );
            let name = entry
                .file_name()
                .into_string()
                .expect("corpus transport names are UTF-8");
            assert!(
                name.ends_with(".hex"),
                "corpus sections contain only .hex files"
            );
            assert!(paths.insert(format!("{section}/{name}")));
        }
    }
    paths
}

fn frozen_transport(relative: &str) -> &'static str {
    match relative {
        "positive/process-v2-core.hex" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/positive/process-v2-core.hex"
        )),
        "negative/self-authorizing-admission.hex" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/negative/self-authorizing-admission.hex"
        )),
        "negative/exact-byte-substitution.hex" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/negative/exact-byte-substitution.hex"
        )),
        "negative/open-form-shape.hex" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/negative/open-form-shape.hex"
        )),
        "negative/unconstituted-external-trigger.hex" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/negative/unconstituted-external-trigger.hex"
        )),
        "negative/resume-missing-fresh-ingress.hex" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/negative/resume-missing-fresh-ingress.hex"
        )),
        "negative/linear-double-takeup.hex" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/negative/linear-double-takeup.hex"
        )),
        "negative/support-collapse.hex" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/negative/support-collapse.hex"
        )),
        "negative/admission-redicision.hex" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/process-v2/negative/admission-redicision.hex"
        )),
        other => panic!("unknown process-v2 corpus path {other}"),
    }
}

#[test]
fn frozen_process_v2_corpus_is_the_exact_encoder_and_mutation_output() {
    let generated = corpus_bytes();
    for (relative, expected) in &generated {
        assert_eq!(
            frozen_hex(frozen_transport(relative)),
            *expected,
            "frozen bytes drifted for {relative}"
        );
    }

    let manifest = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-vectors/process-v2/manifest.json"
    ));
    let sums = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-vectors/process-v2/SHA256SUMS"
    ));
    let parsed: serde_json::Value =
        serde_json::from_str(manifest).expect("process-v2 manifest is valid JSON");
    let expected_paths: std::collections::BTreeSet<_> = generated
        .iter()
        .map(|(relative, _)| (*relative).to_owned())
        .collect();
    assert_eq!(
        expected_paths.len(),
        generated.len(),
        "corpus paths are unique"
    );
    let mut manifest_paths = std::collections::BTreeSet::new();
    for section in ["positive", "negative"] {
        for entry in parsed
            .get(section)
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("manifest {section} section is an array"))
        {
            let relative = entry
                .get("file")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("manifest {section} entry has a file"));
            assert!(
                manifest_paths.insert(relative.to_owned()),
                "manifest duplicates {relative}"
            );
        }
    }
    assert_eq!(manifest_paths, expected_paths, "manifest file set is exact");
    assert_eq!(
        disk_transport_paths(),
        expected_paths,
        "on-disk corpus file set is exact"
    );

    let checksum_entries = parse_checksum_entries(sums);
    assert_eq!(
        checksum_entries
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        expected_paths,
        "SHA256SUMS file set is exact"
    );
    for (relative, _) in &generated {
        let expected = checksum_entries
            .get(*relative)
            .unwrap_or_else(|| panic!("SHA256SUMS omits {relative}"));
        use sha2::{Digest as _, Sha256};
        let actual = Sha256::digest(frozen_transport(relative).as_bytes());
        let actual = lowercase_hex(&actual);
        assert_eq!(
            &actual, expected,
            "transport checksum drifted for {relative}"
        );
    }

    let positive = parsed
        .get("positive")
        .and_then(serde_json::Value::as_array)
        .expect("manifest positive section is an array");
    assert_eq!(positive.len(), 1, "the corpus has one positive core");
    let core = &positive[0];
    assert_eq!(
        core.get("file").and_then(serde_json::Value::as_str),
        Some("positive/process-v2-core.hex")
    );
    let identities = core
        .get("identities")
        .and_then(serde_json::Value::as_object)
        .expect("core manifest freezes exact identities");
    assert_eq!(
        identities.len(),
        6,
        "the exact identity oracle set is closed"
    );
    let oracles = core_corpus_oracles();
    for (name, expected) in [
        (
            "program_snapshot",
            lowercase_hex(oracles.program_snapshot.as_bytes()),
        ),
        (
            "shared_application_shape",
            lowercase_hex(oracles.shared_application_shape.as_bytes()),
        ),
        (
            "process_package",
            lowercase_hex(oracles.process_package.as_bytes()),
        ),
        (
            "program_revision",
            lowercase_hex(oracles.program_revision.as_bytes()),
        ),
        (
            "initial_state",
            lowercase_hex(oracles.initial_state.as_bytes()),
        ),
        (
            "successor_state",
            lowercase_hex(oracles.successor_state.as_bytes()),
        ),
    ] {
        assert_eq!(
            identities.get(name).and_then(serde_json::Value::as_str),
            Some(expected.as_str()),
            "{name} identity drifted"
        );
    }
    let observations = core
        .get("observations")
        .and_then(serde_json::Value::as_object)
        .expect("core manifest freezes projection counts");
    assert_eq!(
        observations.len(),
        oracles.observations.len(),
        "the count oracle set is closed"
    );
    for (name, expected) in oracles.observations {
        assert_eq!(
            observations.get(name).and_then(serde_json::Value::as_u64),
            Some(u64::try_from(expected).expect("corpus counts fit U64")),
            "{name} count drifted"
        );
    }
}

#[test]
fn process_v2_core_rematerializes_and_sibling_step_order_is_not_semantic() {
    let (package, context) = finalized_core_package();
    let suspension_index = package
        .records
        .iter()
        .position(|record| {
            matches!(
                record,
                ProcessRecordV2::Steps(steps)
                    if steps.iter().any(|step| step.id == id!(StepId, 51))
            )
        })
        .unwrap();
    let mut prefix = package.clone();
    prefix.records.truncate(suspension_index + 1);
    let prefix_bytes = encode_process_package(&prefix).unwrap();
    let prefix_checked =
        check_process_package(decode_process_package(&prefix_bytes).unwrap()).unwrap();
    let prefix_authority = establish_core_authority(&prefix_checked, context, true);
    let prefix_carrier = ProcessCarrier::replay(&prefix_checked, &prefix_authority).unwrap();
    assert_eq!(
        prefix_carrier
            .activation(id!(ActivationId, 20))
            .unwrap()
            .status(),
        ActivationStatus::Suspended(id!(ContinuationId, 70))
    );
    drop(prefix_carrier);

    let full_bytes = encode_process_package(&package).unwrap();
    let full_checked = check_process_package(decode_process_package(&full_bytes).unwrap()).unwrap();
    let full_authority = establish_core_authority(&full_checked, context, true);
    let full_carrier = ProcessCarrier::replay(&full_checked, &full_authority).unwrap();
    assert_eq!(
        full_carrier
            .activation(id!(ActivationId, 20))
            .unwrap()
            .status(),
        ActivationStatus::Terminal(ActivationTerminal::Returned)
    );

    let mut swapped = package;
    let left = swapped
        .records
        .iter()
        .position(|record| matches!(record, ProcessRecordV2::Steps(steps) if steps[0].id == id!(StepId, 54)))
        .unwrap();
    let right = swapped
        .records
        .iter()
        .position(|record| matches!(record, ProcessRecordV2::Steps(steps) if steps[0].id == id!(StepId, 55)))
        .unwrap();
    swapped.records.swap(left, right);
    let swapped_bytes = encode_process_package(&swapped).unwrap();
    let swapped_checked =
        check_process_package(decode_process_package(&swapped_bytes).unwrap()).unwrap();
    let swapped_authority = establish_core_authority(&swapped_checked, context, true);
    let swapped_carrier = ProcessCarrier::replay(&swapped_checked, &swapped_authority).unwrap();
    for activation in [id!(ActivationId, 25), id!(ActivationId, 26)] {
        assert_eq!(
            swapped_carrier.activation(activation).unwrap().status(),
            ActivationStatus::Terminal(ActivationTerminal::Returned)
        );
    }
}

#[test]
fn process_v2_named_mutations_have_exact_stage_specific_verdicts() {
    let self_authorizing = self_authorizing_admission_package();
    let self_bytes = encode_process_package(&self_authorizing).unwrap();
    let self_checked = check_process_package(decode_process_package(&self_bytes).unwrap()).unwrap();
    let preimage = ProgramRevisionPreimage {
        semantics: scope().semantics,
        program: id!(ProgramId, 1),
        predecessor: None,
        snapshot: self_checked.constitution().snapshot(),
        change: id!(ProgramChangeOccurrenceId, 2),
    };
    let policy = id!(RootPolicyId, 3);
    let authorization = RootAdmissionAuthorizationRef {
        policy,
        local: AdmissionAuthorizationLocalId::new(0),
    };
    assert_eq!(
        AuthorityStore::new().admit_genesis(
            preimage.derived_claim(),
            self_checked.authority_input(),
            policy,
            authorization,
        ),
        Err(AuthorityError::UnknownRootPolicy(policy))
    );

    let (core, _) = finalized_core_package();
    let core_bytes = encode_process_package(&core).unwrap();
    let core_checked = check_process_package(decode_process_package(&core_bytes).unwrap()).unwrap();
    let (substitution, _) = exact_byte_substitution_package();
    let substitution_bytes = encode_process_package(&substitution).unwrap();
    let substitution_checked =
        check_process_package(decode_process_package(&substitution_bytes).unwrap()).unwrap();
    assert_ne!(substitution_checked.id(), core_checked.id());

    let open_bytes = encode_process_package(&open_form_shape_package()).unwrap();
    assert!(matches!(
        check_process_package(decode_process_package(&open_bytes).unwrap()),
        Err(ProcessPackageCheckError::Formation(
            FormationErrorV2::EligibleModeSetMismatch(application)
        )) if application == ApplicationLocalId::new(1)
    ));

    let unconstituted = unconstituted_external_trigger_package();
    let unconstituted_bytes = encode_process_package(&unconstituted).unwrap();
    let unconstituted_checked =
        check_process_package(decode_process_package(&unconstituted_bytes).unwrap()).unwrap();
    assert!(matches!(
        ProcessCarrier::replay(&unconstituted_checked, &AuthorityStore::new()),
        Err(ProcessError::UnanchoredExternalProvenance { .. })
    ));

    let (missing_ingress, context) = resume_missing_fresh_ingress_package();
    let missing_bytes = encode_process_package(&missing_ingress).unwrap();
    let missing_checked =
        check_process_package(decode_process_package(&missing_bytes).unwrap()).unwrap();
    let missing_authority = establish_core_authority(&missing_checked, context, true);
    assert_eq!(
        ProcessCarrier::replay(&missing_checked, &missing_authority).unwrap_err(),
        ProcessError::ResumptionRequiresFreshIngress
    );

    let (double_takeup, context) = linear_double_takeup_package();
    let double_bytes = encode_process_package(&double_takeup).unwrap();
    let double_checked =
        check_process_package(decode_process_package(&double_bytes).unwrap()).unwrap();
    let double_authority = establish_core_authority(&double_checked, context, true);
    assert_eq!(
        ProcessCarrier::replay(&double_checked, &double_authority).unwrap_err(),
        ProcessError::HandoffUnsupported
    );

    let collapsed = collapse_support_slot(&core_bytes);
    assert_eq!(
        decode_process_package(&collapsed),
        Err(CanonicalDecodeError::NonCanonical(
            CanonicalEncodeError::NonCanonicalOrder("support slots")
        ))
    );

    let (redicision, context) = admission_redicision_package();
    let redicision_bytes = encode_process_package(&redicision).unwrap();
    let redicision_checked =
        check_process_package(decode_process_package(&redicision_bytes).unwrap()).unwrap();
    let redicision_authority = establish_core_authority(&redicision_checked, context, true);
    assert_eq!(
        ProcessCarrier::replay(&redicision_checked, &redicision_authority).unwrap_err(),
        ProcessError::CandidateAlreadyDecided(id!(CandidateDeltaId, 80))
    );
}

#[test]
fn runtime_configuration_custody_budget_and_cancellation_are_exact() {
    let (core, context) = finalized_core_package();

    let carrier = replay_core_candidate(&core, context)
        .expect("Configuration custody does not manufacture a semantic PriorStep cause");
    assert_eq!(
        carrier
            .configuration(id!(ConfigurationId, 68))
            .expect("resumed Step installs Configuration 68")
            .predecessor,
        ConfigurationPredecessorV2::ConfigurationAfter(StepRef {
            run: id!(RunId, 30),
            activation: id!(ActivationId, 20),
            step: id!(StepId, 58),
        })
    );

    let mut foreign_cause = core.clone();
    let resumed = foreign_cause
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => {
                steps.iter_mut().find(|step| step.id == id!(StepId, 58))
            }
            _ => None,
        })
        .expect("core has resumed Step 58");
    resumed.causes.push(StepCause::PriorStep(StepRef {
        run: id!(RunId, 31),
        activation: id!(ActivationId, 21),
        step: id!(StepId, 52),
    }));
    resumed.causes.sort_unstable();
    assert_eq!(
        replay_core_candidate(&foreign_cause, context).unwrap_err(),
        ProcessError::StepCauseOwnerMismatch(id!(StepId, 52))
    );

    let mut duplicate_emitter = core.clone();
    let resumed = duplicate_emitter
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => {
                steps.iter_mut().find(|step| step.id == id!(StepId, 58))
            }
            _ => None,
        })
        .expect("core has resumed Step 58");
    let emitter = StepRef {
        run: id!(RunId, 30),
        activation: id!(ActivationId, 20),
        step: id!(StepId, 51),
    };
    resumed.causes.push(StepCause::PriorStep(emitter));
    resumed.causes.sort_unstable();
    assert_eq!(
        replay_core_candidate(&duplicate_emitter, context).unwrap_err(),
        ProcessError::DuplicateContinuationEmitterCause(emitter)
    );

    let (mut wrong_handoff_application, context) = linear_double_takeup_package();
    let child = wrong_handoff_application
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Activation(activation) if activation.id == id!(ActivationId, 112) => {
                Some(activation)
            }
            _ => None,
        })
        .expect("handoff fixture has its first child Activation");
    child.application = core_application(context.snapshot, 2);
    assert_eq!(
        replay_core_candidate(&wrong_handoff_application, context).unwrap_err(),
        ProcessError::HandoffUnsupported
    );

    let mut wrong_cancellation_scope = core.clone();
    let activation_index = wrong_cancellation_scope
        .records
        .iter()
        .position(|record| {
            matches!(record, ProcessRecordV2::Activation(activation)
                if activation.id == id!(ActivationId, 20))
        })
        .expect("core has Activation 20");
    wrong_cancellation_scope
        .records
        .truncate(activation_index + 1);
    wrong_cancellation_scope
        .records
        .push(ProcessRecordV2::Cancellation(CancellationOccurrenceV2 {
            body: CancellationOccurrenceBodyV2 {
                id: id!(CancellationOccurrenceId, 116),
                target: CancellationTarget::Run(id!(RunId, 30)),
                pins: pure_pins(context),
            },
            provenance: OccurrenceProvenance::EnteredThrough(entered_through(
                context.pure_boundary,
                196,
                vec![CausalRef::ExternalTrigger(id!(
                    ExternalTriggerOccurrenceId,
                    10
                ))],
            )),
        }));
    assert_eq!(
        replay_core_candidate(&wrong_cancellation_scope, context).unwrap_err(),
        ProcessError::CancellationScopeMismatch
    );

    let mut inflated_resume = core.clone();
    let resumption = inflated_resume
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Resumption(resumption) => Some(resumption),
            _ => None,
        })
        .expect("core has a Resumption");
    resumption.body.pins.remaining_budget.remaining_units += 1;
    assert_eq!(
        replay_core_candidate(&inflated_resume, context).unwrap_err(),
        ProcessError::ContinuationPinMismatch
    );

    let mut underflow = core;
    let first = underflow
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => {
                steps.iter_mut().find(|step| step.id == id!(StepId, 50))
            }
            _ => None,
        })
        .expect("core has Step 50");
    first.budget = budget(100, 101, 0);
    assert_eq!(
        replay_core_candidate(&underflow, context).unwrap_err(),
        ProcessError::StepBudgetUnderflow
    );
}

#[test]
fn ready_and_run_cancellation_require_exact_occurrence_and_consumer_pins() {
    let (candidate, context) = ready_cancellation_candidate();
    let carrier = replay_core_candidate(&candidate, context)
        .expect("Ready cancellation with the exact two-cause frontier is valid");
    assert_eq!(
        carrier
            .activation(id!(ActivationId, 21))
            .expect("cancelled Activation remains visible")
            .status(),
        ActivationStatus::Terminal(ActivationTerminal::Cancelled)
    );

    for causes in [
        vec![StepCause::ActivationStart(id!(ActivationId, 21))],
        vec![StepCause::CancellationRequest(id!(
            CancellationOccurrenceId,
            116
        ))],
        vec![
            StepCause::ActivationStart(id!(ActivationId, 21)),
            StepCause::CancellationRequest(id!(CancellationOccurrenceId, 117)),
        ],
        vec![
            StepCause::ActivationStart(id!(ActivationId, 21)),
            StepCause::PriorStep(StepRef {
                run: id!(RunId, 31),
                activation: id!(ActivationId, 21),
                step: id!(StepId, 53),
            }),
            StepCause::CancellationRequest(id!(CancellationOccurrenceId, 116)),
        ],
    ] {
        let (mut malformed, context) = ready_cancellation_candidate();
        let step = malformed
            .records
            .iter_mut()
            .find_map(|record| match record {
                ProcessRecordV2::Steps(steps) => {
                    steps.iter_mut().find(|step| step.id == id!(StepId, 52))
                }
                _ => None,
            })
            .expect("cancellation fixture has Step 52");
        step.causes = causes;
        step.causes.sort_unstable();
        assert_eq!(
            replay_core_candidate(&malformed, context).unwrap_err(),
            ProcessError::InvalidFirstStepFrontier
        );
    }

    let (mut wrong_consumer, context) = cancellable_core_package();
    let activation_25 = wrong_consumer
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Activation(value) if value.id == id!(ActivationId, 25) => Some(value),
            _ => None,
        })
        .expect("core has child Activation 25");
    activation_25.pins.cancellation_scope = CancellationScope::Run;
    let cancellation_pins = activation_25.pins.clone();
    let activation_26 = wrong_consumer
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Activation(value) if value.id == id!(ActivationId, 26) => Some(value),
            _ => None,
        })
        .expect("core has child Activation 26");
    activation_26.pins.cancellation_scope = CancellationScope::Run;
    activation_26.pins.budget = Budget {
        remaining_units: 99,
    };
    let activation_26_index = wrong_consumer
        .records
        .iter()
        .position(|record| {
            matches!(record, ProcessRecordV2::Activation(value)
                if value.id == id!(ActivationId, 26))
        })
        .expect("core has child Activation 26");
    wrong_consumer.records.insert(
        activation_26_index + 1,
        ProcessRecordV2::Cancellation(CancellationOccurrenceV2 {
            body: CancellationOccurrenceBodyV2 {
                id: id!(CancellationOccurrenceId, 116),
                target: CancellationTarget::Run(id!(RunId, 32)),
                pins: cancellation_pins,
            },
            provenance: OccurrenceProvenance::EnteredThrough(entered_through(
                context.pure_boundary,
                196,
                vec![CausalRef::Step(StepRef {
                    run: id!(RunId, 32),
                    activation: id!(ActivationId, 22),
                    step: id!(StepId, 53),
                })],
            )),
        }),
    );
    let step_55 = wrong_consumer
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => {
                steps.iter_mut().find(|step| step.id == id!(StepId, 55))
            }
            _ => None,
        })
        .expect("core has child Step 55");
    step_55.budget = budget(99, 10, 89);
    step_55.causes = vec![
        StepCause::ActivationStart(id!(ActivationId, 26)),
        StepCause::CancellationRequest(id!(CancellationOccurrenceId, 116)),
    ];
    step_55.outcome = StepOutcomeProposalV2::Cancel(id!(CancellationOccurrenceId, 116));
    assert_eq!(
        replay_core_candidate(&wrong_consumer, context).unwrap_err(),
        ProcessError::CancellationScopeMismatch
    );
}

#[test]
fn dynamic_prerequisites_enforce_declared_occurrence_kind_and_scope() {
    let (wrong_kind, context) = prerequisite_candidate(
        ActivationPrerequisiteKind::Observation,
        PrerequisiteScope::SameProgramRevision,
        ActivationPrerequisite::Admission(id!(AdmissionOccurrenceId, 94)),
        false,
    );
    assert_eq!(
        replay_core_candidate(&wrong_kind, context).unwrap_err(),
        ProcessError::PrerequisiteOccurrenceKindMismatch {
            slot: PrerequisiteSlotId {
                mode: core_mode(context.snapshot, 1),
                local: PrerequisiteLocalId::new(1),
            },
            expected: ActivationPrerequisiteKind::Observation,
            actual: ActivationPrerequisiteKind::Admission,
        }
    );

    let (wrong_scope, context) = prerequisite_candidate(
        ActivationPrerequisiteKind::Observation,
        PrerequisiteScope::SameObservedState,
        ActivationPrerequisite::Observation(id!(ObservationId, 82)),
        true,
    );
    assert_eq!(
        replay_core_candidate(&wrong_scope, context).unwrap_err(),
        ProcessError::PrerequisiteScopeMismatch
    );

    let (mut wrong_ordinal, context) = prerequisite_candidate(
        ActivationPrerequisiteKind::Observation,
        PrerequisiteScope::SameSemantics,
        ActivationPrerequisite::Observation(id!(ObservationId, 82)),
        true,
    );
    let activation = wrong_ordinal
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Activation(activation) => Some(activation),
            _ => None,
        })
        .expect("prerequisite fixture has one Activation");
    activation.prerequisite_bindings[0].ordinal = 1;
    activation.causes.prerequisite_occurrences[0].ordinal = 1;
    let slot = activation.prerequisite_bindings[0].slot;
    assert_eq!(
        replay_core_candidate(&wrong_ordinal, context).unwrap_err(),
        ProcessError::PrerequisiteOrdinalMismatch {
            slot,
            expected: 0,
            actual: 1,
        }
    );

    let (mut wrong_projection, context) = prerequisite_candidate(
        ActivationPrerequisiteKind::Observation,
        PrerequisiteScope::SameSemantics,
        ActivationPrerequisite::Observation(id!(ObservationId, 82)),
        true,
    );
    let activation = wrong_projection
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Activation(activation) => Some(activation),
            _ => None,
        })
        .expect("prerequisite fixture has one Activation");
    activation.causes.prerequisite_occurrences[0].component = CauseComponentLocalId::new(2);
    assert_eq!(
        replay_core_candidate(&wrong_projection, context).unwrap_err(),
        ProcessError::ActivationCauseProjectionMismatch
    );
}

#[test]
fn formation_evidence_must_be_prior_declared_distinct_and_causal() {
    let (mut undeclared, context) = finalized_core_package();
    let observation = undeclared
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => steps
                .iter_mut()
                .find(|step| step.id == id!(StepId, 53))
                .and_then(|step| step.observation_outcomes.first_mut()),
            _ => None,
        })
        .expect("checker Step has Formation observations");
    let StepObservationOutcomeV2::Observed(ObservationProposalV2::Formation {
        target: found, ..
    }) = observation
    else {
        panic!("checker Step emits Formation observations");
    };
    *found = target("undeclared-output");
    assert_eq!(
        replay_core_candidate(&undeclared, context).unwrap_err(),
        ProcessError::FormationObservationNotDeclared
    );

    let (mut same_step, context) = finalized_core_package();
    let consumer = same_step
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => {
                steps.iter_mut().find(|step| step.id == id!(StepId, 54))
            }
            _ => None,
        })
        .expect("core has child return Step 54");
    consumer
        .observation_outcomes
        .push(StepObservationOutcomeV2::Observed(formation_observation(
            89,
            "child/self",
            target("result"),
            SupportSource::ExternalTrigger(id!(ExternalTriggerOccurrenceId, 10)),
        )));
    consumer.outcome = StepOutcomeProposalV2::Return(domain_bound("child/self", 89));
    assert_eq!(
        replay_core_candidate(&same_step, context).unwrap_err(),
        ProcessError::MissingPriorFormationEvidence(id!(ObservationId, 89))
    );

    let (mut same_activation, context) = finalized_core_package();
    let checker_index = same_activation
        .records
        .iter()
        .position(|record| {
            matches!(record, ProcessRecordV2::Steps(steps)
                if steps.iter().any(|step| step.id == id!(StepId, 53)))
        })
        .expect("core has checker Step 53");
    same_activation.records.insert(
        checker_index + 1,
        ProcessRecordV2::Steps(vec![step(
            59,
            32,
            22,
            63,
            69,
            budget(90, 10, 80),
            vec![StepCause::PriorStep(StepRef {
                run: id!(RunId, 32),
                activation: id!(ActivationId, 22),
                step: id!(StepId, 53),
            })],
            StepOutcomeProposalV2::Return(domain_bound("value/resumed", 88)),
        )]),
    );
    assert_eq!(
        replay_core_candidate(&same_activation, context).unwrap_err(),
        ProcessError::FormationEvidenceRequiresDistinctActivation(id!(ObservationId, 88))
    );

    let (mut noncausal, context) = finalized_core_package();
    let activation = noncausal
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Activation(value) if value.id == id!(ActivationId, 20) => Some(value),
            _ => None,
        })
        .expect("core has Activation 20");
    activation.prerequisite_bindings.clear();
    activation.causes.prerequisite_occurrences.clear();
    assert_eq!(
        replay_core_candidate(&noncausal, context).unwrap_err(),
        ProcessError::FormationEvidenceNotCausal(id!(ObservationId, 88))
    );

    let (mut prior_step, context) = finalized_core_package();
    let checker_index = prior_step
        .records
        .iter()
        .position(|record| {
            matches!(record, ProcessRecordV2::Steps(steps)
                if steps.iter().any(|step| step.id == id!(StepId, 53)))
        })
        .expect("core has checker Step 53");
    let mut activation = root_activation(
        context,
        27,
        37,
        47,
        1,
        RootTrigger::External(id!(ExternalTriggerOccurrenceId, 12)),
    );
    require_formation_observation(&mut activation, context, 88);
    let first = step(
        59,
        37,
        27,
        47,
        69,
        budget(100, 10, 90),
        vec![StepCause::ActivationStart(id!(ActivationId, 27))],
        StepOutcomeProposalV2::Progress,
    );
    let mut causes = vec![StepCause::PriorStep(StepRef {
        run: id!(RunId, 37),
        activation: id!(ActivationId, 27),
        step: id!(StepId, 59),
    })];
    causes.sort_unstable();
    let second = step(
        60,
        37,
        27,
        69,
        70,
        budget(90, 10, 80),
        causes,
        StepOutcomeProposalV2::Return(domain_bound("value/resumed", 88)),
    );
    prior_step.records.splice(
        checker_index + 1..checker_index + 1,
        [
            ProcessRecordV2::Activation(activation),
            ProcessRecordV2::Steps(vec![first]),
            ProcessRecordV2::Steps(vec![second]),
        ],
    );
    let carrier = replay_core_candidate(&prior_step, context)
        .expect("an exact prior checker Step may carry Formation evidence");
    assert_eq!(
        carrier
            .activation(id!(ActivationId, 27))
            .expect("added Activation remains visible")
            .status(),
        ActivationStatus::Terminal(ActivationTerminal::Returned)
    );
}

#[test]
fn budget_exhaustion_is_typed_and_requires_exactly_zero_remaining_budget() {
    let (exhausted, context) = bounded_exhaustion_candidate(0);
    let carrier = replay_core_candidate(&exhausted, context)
        .expect("typed zero-budget exhaustion is a valid terminal outcome");
    assert_eq!(
        carrier
            .activation(id!(ActivationId, 20))
            .expect("bounded Activation remains visible")
            .status(),
        ActivationStatus::Terminal(ActivationTerminal::BudgetExhausted)
    );

    let (nonzero, context) = bounded_exhaustion_candidate(90);
    assert_eq!(
        replay_core_candidate(&nonzero, context).unwrap_err(),
        ProcessError::BudgetExhaustionRequiresZero
    );
}

#[test]
fn local_resumption_does_not_require_handoff_permission() {
    let mut snapshot = core_snapshot();
    let ContinuationContractV2::Suspensible { may_handoff, .. } =
        &mut snapshot.constitution.operators[0].modes[0]
            .contract
            .continuation
    else {
        panic!("core Mode 1 is suspensible");
    };
    *may_handoff = false;
    let (candidate, context) = finalized_core_package_from_snapshot(snapshot);
    let carrier = replay_core_candidate(&candidate, context)
        .expect("local resumption is independent of handoff authority");
    assert_eq!(
        carrier
            .activation(id!(ActivationId, 20))
            .expect("resumed Activation remains visible")
            .status(),
        ActivationStatus::Terminal(ActivationTerminal::Returned)
    );
}

#[test]
fn handoff_occurrence_requires_mode_permission_before_admission() {
    let mut snapshot = core_snapshot();
    let ContinuationContractV2::Suspensible { may_handoff, .. } =
        &mut snapshot.constitution.operators[0].modes[0]
            .contract
            .continuation
    else {
        panic!("core Mode 1 is suspensible");
    };
    *may_handoff = false;
    let (mut candidate, context) = finalized_core_package_from_snapshot(snapshot);
    let suspension_index = candidate
        .records
        .iter()
        .position(|record| {
            matches!(record, ProcessRecordV2::Steps(steps)
                if steps.iter().any(|step| step.id == id!(StepId, 51)))
        })
        .expect("core has suspension Step 51");
    candidate.records.truncate(suspension_index + 1);
    candidate
        .records
        .push(ProcessRecordV2::Handoff(HandoffOccurrenceV2 {
            body: HandoffOccurrenceBodyV2 {
                id: id!(HandoffOccurrenceId, 110),
                continuation: id!(ContinuationId, 70),
                run: id!(RunId, 30),
                activation: id!(ActivationId, 20),
                pins: continuation(context).pins,
            },
            provenance: OccurrenceProvenance::EnteredThrough(entered_through(
                context.pure_boundary,
                196,
                vec![CausalRef::Step(StepRef {
                    run: id!(RunId, 30),
                    activation: id!(ActivationId, 20),
                    step: id!(StepId, 51),
                })],
            )),
        }));
    assert_eq!(
        replay_core_candidate(&candidate, context).unwrap_err(),
        ProcessError::HandoffUnsupported
    );
}

#[test]
fn bounded_progress_and_suspension_must_consume_positive_budget() {
    let (mut progress, context) = bounded_exhaustion_candidate(0);
    let progress_step = progress
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => {
                steps.iter_mut().find(|step| step.id == id!(StepId, 50))
            }
            _ => None,
        })
        .expect("bounded fixture has Step 50");
    progress_step.budget = budget(100, 0, 100);
    progress_step.outcome = StepOutcomeProposalV2::Progress;
    assert_eq!(
        replay_core_candidate(&progress, context).unwrap_err(),
        ProcessError::BoundedProgressRequiresConsumption
    );

    let (mut suspension, context) = bounded_exhaustion_candidate(0);
    let activation = suspension
        .records
        .iter()
        .find_map(|record| match record {
            ProcessRecordV2::Activation(value) if value.id == id!(ActivationId, 20) => Some(value),
            _ => None,
        })
        .expect("bounded fixture has Activation 20");
    let pins = activation.pins.clone();
    let suspension_step = suspension
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => {
                steps.iter_mut().find(|step| step.id == id!(StepId, 50))
            }
            _ => None,
        })
        .expect("bounded fixture has Step 50");
    suspension_step.budget = budget(100, 0, 100);
    suspension_step.outcome = StepOutcomeProposalV2::Suspend(ContinuationProposalV2 {
        id: id!(ContinuationId, 70),
        emitted_by: id!(StepId, 50),
        pins: ContinuationPins {
            run: id!(RunId, 30),
            activation: id!(ActivationId, 20),
            application: core_application(context.snapshot, 1),
            mode: core_mode(context.snapshot, 1),
            activation_pins: pins,
            remaining_budget: Budget {
                remaining_units: 100,
            },
        },
        remainder: term("configuration/bounded-suspension"),
    });
    assert_eq!(
        replay_core_candidate(&suspension, context).unwrap_err(),
        ProcessError::BoundedProgressRequiresConsumption
    );
}

#[test]
fn governed_occurrences_require_exact_authority_basis_and_direct_causes() {
    let (core, context) = finalized_core_package();

    let mut produced_without_basis = core.clone();
    let judgment = produced_without_basis
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Judgment(judgment)
                if judgment.body.id == id!(JudgmentOccurrenceId, 90) =>
            {
                Some(judgment)
            }
            _ => None,
        })
        .expect("core has admission verdict Judgment 90");
    judgment.provenance = OccurrenceProvenance::ProducedBy(StepRef {
        run: id!(RunId, 33),
        activation: id!(ActivationId, 23),
        step: id!(StepId, 56),
    });
    assert_eq!(
        replay_core_candidate(&produced_without_basis, context).unwrap_err(),
        ProcessError::JudgmentAuthorityNotInProducerBasis {
            judgment: id!(JudgmentOccurrenceId, 90),
            activation: id!(ActivationId, 23),
        }
    );

    let mut judgment_missing_candidate = core.clone();
    let judgment = judgment_missing_candidate
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Judgment(judgment)
                if judgment.body.id == id!(JudgmentOccurrenceId, 90) =>
            {
                Some(judgment)
            }
            _ => None,
        })
        .expect("core has admission verdict Judgment 90");
    let OccurrenceProvenance::EnteredThrough(entered) = &mut judgment.provenance else {
        unreachable!("core Judgments enter through governance")
    };
    entered.causes.clear();
    assert_eq!(
        replay_core_candidate(&judgment_missing_candidate, context).unwrap_err(),
        ProcessError::MissingJudgmentCandidateCause {
            judgment: id!(JudgmentOccurrenceId, 90),
            delta: id!(CandidateDeltaId, 80),
        }
    );

    let wrong_revision = id!(ProgramRevisionId, 118);
    let mut judgment_wrong_revision = core.clone();
    let judgment = judgment_wrong_revision
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::Judgment(judgment)
                if judgment.body.id == id!(JudgmentOccurrenceId, 90) =>
            {
                Some(judgment)
            }
            _ => None,
        })
        .expect("core has admission verdict Judgment 90");
    judgment.body.authority = JudgmentAuthorityEvidence::ProgramConstitution {
        revision: wrong_revision,
        authority: JudgmentAuthorityRef {
            snapshot: context.snapshot,
            local: JudgmentAuthorityLocalId::new(0),
        },
    };
    assert_eq!(
        replay_core_candidate(&judgment_wrong_revision, context).unwrap_err(),
        ProcessError::JudgmentProgramRevisionMismatch {
            judgment: id!(JudgmentOccurrenceId, 90),
            expected: context.revision.id,
            actual: wrong_revision,
        }
    );

    let mut admission_wrong_revision = core.clone();
    let decision = admission_wrong_revision
        .records
        .iter_mut()
        .find_map(|record| match record {
            ProcessRecordV2::AdmissionDecision(decision)
                if decision.occurrence == id!(AdmissionOccurrenceId, 94) =>
            {
                Some(decision)
            }
            _ => None,
        })
        .expect("core has admission decision 94");
    decision.authorization = AdmissionAuthorizationEvidence::ProgramConstitution {
        revision: wrong_revision,
        authorization: AdmissionAuthorizationRef {
            snapshot: context.snapshot,
            local: AdmissionAuthorizationLocalId::new(0),
        },
    };
    assert_eq!(
        replay_core_candidate(&admission_wrong_revision, context).unwrap_err(),
        ProcessError::AdmissionProgramRevisionMismatch {
            admission: id!(AdmissionOccurrenceId, 94),
            expected: context.revision.id,
            actual: wrong_revision,
        }
    );

    for (missing, expected) in [
        (
            CausalRef::CandidateDelta(id!(CandidateDeltaId, 80)),
            ProcessError::MissingAdmissionCandidateCause(id!(AdmissionOccurrenceId, 94)),
        ),
        (
            CausalRef::Judgment(id!(JudgmentOccurrenceId, 90)),
            ProcessError::MissingAdmissionVerdictCause {
                admission: id!(AdmissionOccurrenceId, 94),
                judgment: id!(JudgmentOccurrenceId, 90),
            },
        ),
        (
            CausalRef::Judgment(id!(JudgmentOccurrenceId, 91)),
            ProcessError::MissingAdmissionObligationCause {
                admission: id!(AdmissionOccurrenceId, 94),
                judgment: id!(JudgmentOccurrenceId, 91),
            },
        ),
    ] {
        let mut admission_missing_cause = core.clone();
        let decision = admission_missing_cause
            .records
            .iter_mut()
            .find_map(|record| match record {
                ProcessRecordV2::AdmissionDecision(decision)
                    if decision.occurrence == id!(AdmissionOccurrenceId, 94) =>
                {
                    Some(decision)
                }
                _ => None,
            })
            .expect("core has admission decision 94");
        decision.provenance.causes.retain(|cause| *cause != missing);
        assert_eq!(
            replay_core_candidate(&admission_missing_cause, context).unwrap_err(),
            expected
        );
    }
}

#[test]
fn step_batches_roll_back_every_completed_step_after_later_failure() {
    let (mut prefix, context) = finalized_core_package();
    let step_52_index = prefix
        .records
        .iter()
        .position(|record| {
            matches!(record, ProcessRecordV2::Steps(steps)
                if steps.iter().any(|step| step.id == id!(StepId, 52)))
        })
        .expect("core has Step 52");
    let valid = match &prefix.records[step_52_index] {
        ProcessRecordV2::Steps(steps) => steps[0].clone(),
        _ => unreachable!("located a Steps record"),
    };
    prefix.records.truncate(step_52_index);

    let bytes = encode_process_package(&prefix).expect("prefix encodes");
    let checked = check_process_package(decode_process_package(&bytes).expect("prefix decodes"))
        .expect("prefix checks");
    let authority = establish_core_authority(&checked, context, true);
    let mut carrier = ProcessCarrier::replay(&checked, &authority).expect("prefix replays");
    let baseline_step_count = carrier.step_count();
    let baseline_observation_count = carrier.observation_count();
    let package_id = carrier.package_id();
    let package_bytes = carrier.exact_package_bytes().to_vec();

    assert_eq!(
        carrier.apply_ingress(&[], &authority),
        Err(ProcessIngressError::Batch {
            cause: Box::new(ProcessError::EmptyIngressBatch),
        })
    );

    let invalid = step(
        59,
        31,
        21,
        62,
        69,
        budget(90, 91, 0),
        vec![StepCause::PriorStep(StepRef {
            run: id!(RunId, 31),
            activation: id!(ActivationId, 21),
            step: id!(StepId, 52),
        })],
        StepOutcomeProposalV2::Progress,
    );
    let entered = ProcessRecordV2::EnteredObservation(EnteredObservationV2 {
        observation: ObservationProposalV2::Value {
            id: id!(ObservationId, 99),
            value: term("observation/live-ingress"),
            supports: vec![],
        },
        provenance: entered_through(
            context.pure_boundary,
            196,
            vec![CausalRef::Step(StepRef {
                run: id!(RunId, 30),
                activation: id!(ActivationId, 20),
                step: id!(StepId, 51),
            })],
        ),
    });
    assert_eq!(
        carrier.apply_ingress(
            &[
                entered.clone(),
                ProcessRecordV2::Steps(vec![valid.clone(), invalid]),
            ],
            &authority,
        ),
        Err(ProcessIngressError::Step {
            record_index: 1,
            step_index: 1,
            cause: Box::new(ProcessError::StepBudgetUnderflow),
        })
    );
    assert_eq!(carrier.step_count(), baseline_step_count);
    assert_eq!(carrier.observation_count(), baseline_observation_count);
    assert!(carrier.step(id!(StepId, 52)).is_none());
    assert!(carrier.step(id!(StepId, 59)).is_none());
    assert!(carrier.observation(id!(ObservationId, 99)).is_none());
    let activation = carrier
        .activation(id!(ActivationId, 21))
        .expect("Activation 21 remains present");
    assert_eq!(activation.status(), ActivationStatus::Ready);
    assert_eq!(activation.latest_configuration(), id!(ConfigurationId, 41));
    assert_eq!(
        carrier
            .configuration(id!(ConfigurationId, 41))
            .expect("initial Configuration remains present")
            .predecessor,
        ConfigurationPredecessorV2::ActivationStart(id!(ActivationId, 21))
    );
    assert_eq!(
        activation.remaining_budget(),
        Budget {
            remaining_units: 100,
        }
    );
    assert_eq!(carrier.package_id(), package_id);
    assert_eq!(carrier.exact_package_bytes(), package_bytes);

    let duplicate_trigger = prefix
        .records
        .iter()
        .find_map(|record| match record {
            ProcessRecordV2::ExternalTrigger(value)
                if value.id == id!(ExternalTriggerOccurrenceId, 10) =>
            {
                Some(ProcessRecordV2::ExternalTrigger(value.clone()))
            }
            _ => None,
        })
        .expect("prefix has External trigger 10");
    assert_eq!(
        carrier.apply_ingress(&[entered.clone(), duplicate_trigger], &authority),
        Err(ProcessIngressError::Record {
            record_index: 1,
            cause: Box::new(ProcessError::DuplicateExternalTrigger(id!(
                ExternalTriggerOccurrenceId,
                10
            ))),
        })
    );
    assert!(carrier.observation(id!(ObservationId, 99)).is_none());

    carrier
        .apply_ingress(&[entered, ProcessRecordV2::Steps(vec![valid])], &authority)
        .expect("the same valid records apply after complete rollback");
    assert_eq!(carrier.step_count(), baseline_step_count + 1);
    assert_eq!(carrier.observation_count(), baseline_observation_count + 1);
    assert!(carrier.step(id!(StepId, 52)).is_some());
    assert!(carrier.observation(id!(ObservationId, 99)).is_some());
    assert_eq!(carrier.package_id(), package_id);
    assert_eq!(carrier.exact_package_bytes(), package_bytes);
}

#[test]
fn live_ingress_suffix_matches_full_replay_observations_without_rebinding_package_bytes() {
    let (full, context) = finalized_core_package();
    let split = full
        .records
        .iter()
        .position(|record| {
            matches!(record, ProcessRecordV2::Steps(steps)
                if steps.iter().any(|step| step.id == id!(StepId, 51)))
        })
        .expect("core has suspension Step 51")
        + 1;
    let suffix = full.records[split..].to_vec();
    let mut prefix = full.clone();
    prefix.records.truncate(split);

    let prefix_bytes = encode_process_package(&prefix).expect("prefix encodes");
    let prefix_checked =
        check_process_package(decode_process_package(&prefix_bytes).expect("prefix decodes"))
            .expect("prefix checks");
    let prefix_authority = establish_core_authority(&prefix_checked, context, true);
    let mut live =
        ProcessCarrier::replay(&prefix_checked, &prefix_authority).expect("prefix replays");
    let live_package = live.package_id();
    let live_bytes = live.exact_package_bytes().to_vec();
    live.apply_ingress(&suffix, &prefix_authority)
        .expect("the full suffix is accepted as live ingress");

    let full_bytes = encode_process_package(&full).expect("full package encodes");
    let full_checked =
        check_process_package(decode_process_package(&full_bytes).expect("full package decodes"))
            .expect("full package checks");
    let full_authority = establish_core_authority(&full_checked, context, true);
    let replayed =
        ProcessCarrier::replay(&full_checked, &full_authority).expect("full package replays");

    assert_eq!(live.application_count(), replayed.application_count());
    assert_eq!(live.activation_count(), replayed.activation_count());
    assert_eq!(live.run_count(), replayed.run_count());
    assert_eq!(live.step_count(), replayed.step_count());
    assert_eq!(live.observation_count(), replayed.observation_count());
    assert_eq!(live.continuation_count(), replayed.continuation_count());
    assert_eq!(
        live.candidate_delta_count(),
        replayed.candidate_delta_count()
    );
    assert_eq!(live.decision_count(), replayed.decision_count());
    assert_eq!(live.state_revision_count(), replayed.state_revision_count());
    for activation in 20..=26 {
        let id = id!(ActivationId, activation);
        assert_eq!(
            live.activation(id).map(Activation::status),
            replayed.activation(id).map(Activation::status)
        );
    }
    assert_eq!(live.package_id(), live_package);
    assert_eq!(live.exact_package_bytes(), live_bytes);
    assert_ne!(live.package_id(), replayed.package_id());
}

#[test]
#[ignore = "writes the frozen process-v2 corpus only when explicitly requested"]
fn regenerate_process_v2_corpus() {
    use std::fmt::Write as _;

    assert_eq!(
        std::env::var("CLAUSE_REGENERATE_PROCESS_V2").as_deref(),
        Ok("1"),
        "set CLAUSE_REGENERATE_PROCESS_V2=1 to regenerate"
    );
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-vectors/process-v2");
    let mut generated = corpus_bytes();
    generated.sort_unstable_by_key(|(relative, _)| *relative);
    let mut sums = String::new();
    for (relative, bytes) in generated {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let transport = formatted_hex(&bytes);
        std::fs::write(path, &transport).unwrap();
        use sha2::{Digest as _, Sha256};
        writeln!(
            &mut sums,
            "{}  {relative}",
            lowercase_hex(&Sha256::digest(transport.as_bytes()))
        )
        .expect("String writes cannot fail");
    }
    std::fs::write(root.join("SHA256SUMS"), sums).unwrap();

    let oracles = core_corpus_oracles();
    println!(
        "program_snapshot={}",
        lowercase_hex(oracles.program_snapshot.as_bytes())
    );
    println!(
        "shared_application_shape={}",
        lowercase_hex(oracles.shared_application_shape.as_bytes())
    );
    println!(
        "process_package={}",
        lowercase_hex(oracles.process_package.as_bytes())
    );
    println!(
        "program_revision={}",
        lowercase_hex(oracles.program_revision.as_bytes())
    );
    println!(
        "initial_state={}",
        lowercase_hex(oracles.initial_state.as_bytes())
    );
    println!(
        "successor_state={}",
        lowercase_hex(oracles.successor_state.as_bytes())
    );
}
