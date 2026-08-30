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
    clamp(
        add(s(0), mul(mul(s(4), s(8)), a(0))),
        s(10),
        s(11),
    )
}

fn next_vertical_velocity() -> ExecutableExpressionV1 {
    add(s(3), mul(s(6), a(0)))
}

fn next_y() -> ExecutableExpressionV1 {
    add(s(1), mul(next_vertical_velocity(), a(0)))
}

fn headless_program() -> ExecutableProgramV1 {
    let horizontal_assignments = || {
        vec![
            (0, next_x()),
            (2, div(sub(next_x(), s(0)), a(0))),
        ]
    };
    let mut grounded_tick = horizontal_assignments();
    grounded_tick.extend([(1, s(9)), (3, n(0.0))]);
    let mut airborne_tick = horizontal_assignments();
    airborne_tick.extend([(1, next_y()), (3, next_vertical_velocity())]);
    let mut landing_tick = horizontal_assignments();
    landing_tick.extend([(1, s(9)), (3, n(0.0)), (5, b(true))]);

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
    }
}

fn checked_program_package() -> (CheckedProcessPackage, Vec<ProcessRecordV2>) {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-vectors/process-v2/positive/process-v2-core.hex"
    ));
    let decoded = decode_process_package(&decode_hex(source)).expect("base package decodes");
    let mut candidate = decoded.candidate().clone();
    let templates = candidate.records.clone();
    candidate.records.clear();

    let scope = candidate.snapshot.constitution.semantics;
    let term = headless_program()
        .encode_term(TermScope {
            universe: candidate.snapshot.constitution.universe,
            semantics: scope,
        })
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
    }
    candidate.claimed_snapshot =
        derive_program_snapshot_id(&candidate.snapshot).expect("program snapshot is canonical");
    let bytes = encode_process_package(&candidate).expect("program package encodes");
    let checked = check_process_package(
        decode_process_package(&bytes).expect("program package decodes"),
    )
    .expect("program package checks");
    (checked, templates)
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

fn activation_template(records: &[ProcessRecordV2], tag: u8) -> ActivationProposalV2 {
    records
        .iter()
        .find_map(|record| match record {
            ProcessRecordV2::Activation(proposal) if proposal.id == id!(ActivationId, tag) => {
                Some(proposal.clone())
            }
            _ => None,
        })
        .expect("activation template exists")
}

fn step_template(records: &[ProcessRecordV2], tag: u8) -> StepProposalV2 {
    records
        .iter()
        .find_map(|record| match record {
            ProcessRecordV2::Steps(steps) => steps
                .iter()
                .find(|step| step.id == id!(StepId, tag))
                .cloned(),
            _ => None,
        })
        .expect("Step template exists")
}

fn external_trigger_template(
    records: &[ProcessRecordV2],
    tag: u8,
) -> ExternalTriggerOccurrenceV2 {
    records
        .iter()
        .find_map(|record| match record {
            ProcessRecordV2::ExternalTrigger(trigger)
                if trigger.id == id!(ExternalTriggerOccurrenceId, tag) =>
            {
                Some(trigger.clone())
            }
            _ => None,
        })
        .expect("external-trigger template exists")
}

fn judgment_template(records: &[ProcessRecordV2], tag: u8) -> JudgmentOccurrenceV2 {
    records
        .iter()
        .find_map(|record| match record {
            ProcessRecordV2::Judgment(judgment)
                if judgment.body.id == id!(JudgmentOccurrenceId, tag) =>
            {
                Some(judgment.clone())
            }
            _ => None,
        })
        .expect("Judgment template exists")
}

fn admission_template(records: &[ProcessRecordV2]) -> StateAdmissionDecisionV2 {
    records
        .iter()
        .find_map(|record| match record {
            ProcessRecordV2::AdmissionDecision(decision)
                if decision.delta == id!(CandidateDeltaId, 80) =>
            {
                Some(decision.clone())
            }
            _ => None,
        })
        .expect("Admission template exists")
}

fn rebind_activation(
    proposal: &mut ActivationProposalV2,
    package: &CheckedProcessPackage,
    facts: CarrierFacts,
) {
    let snapshot = package.constitution().snapshot();
    proposal.application.snapshot = snapshot;
    proposal.mode.operator.snapshot = snapshot;
    proposal.pins.snapshot = snapshot;
    proposal.pins.program_revision = facts.revision;
    if proposal.pins.runtime_session.is_some() {
        proposal.pins.observed_state = Some(facts.initial_state);
    }
    let executable = package
        .constitution()
        .executable_contract(proposal.application, proposal.mode)
        .expect("rebound Application and Mode remain executable");
    proposal.pins.context_requirements = executable.application_context_requirements;
    proposal.pins.constitutive_dependencies = executable.application_dependency_closure;
    proposal.static_basis.execution_authorizations.clear();
    for prerequisite in &mut proposal.causes.prerequisites {
        prerequisite.kind.snapshot = snapshot;
    }
}

fn configuration_term(
    package: &CheckedProcessPackage,
    values: &[ExecutableValueV1],
) -> Term {
    executable_configuration_term_v1(
        TermScope {
            universe: package.constitution().universe(),
            semantics: package.constitution().semantics(),
        },
        values,
    )
    .expect("Configuration values encode canonically")
}

fn carry_latest_step(
    runtime: &mut ExecutableProcessRuntimeV1<'_>,
    package: &CheckedProcessPackage,
    facts: CarrierFacts,
    prior: Option<StepRef>,
    remaining_budget: u64,
    candidate_delta: Option<CandidateDeltaV2>,
) -> (Option<StepRef>, u64) {
    let bridge_step = runtime
        .steps()
        .last()
        .expect("the executable bridge just advanced")
        .clone();
    let ordinal = u8::try_from(runtime.steps().len()).expect("bounded scenario Step count");
    let occurrence = EnteredObservationV2 {
        observation: ObservationProposalV2::Value {
            id: ObservationId::from_bytes(raw_id(130 + ordinal)),
            value: executable_occurrence_term_v1(
                TermScope {
                    universe: package.constitution().universe(),
                    semantics: package.constitution().semantics(),
                },
                &bridge_step.occurrence,
            )
            .expect("opaque occurrence encodes canonically"),
            supports: vec![],
        },
        provenance: EnteredThrough {
            boundary: facts.pure_boundary,
            evidence: id!(ExternalEvidenceRef, 181),
            causes: vec![CausalRef::ExternalTrigger(id!(
                ExternalTriggerOccurrenceId,
                12
            ))],
        },
    };
    let reference = StepRef {
        run: runtime.run(),
        activation: runtime.activation(),
        step: bridge_step.id,
    };
    let after_budget = remaining_budget - 1;
    let step = StepProposalV2 {
        id: bridge_step.id,
        run: reference.run,
        activation: reference.activation,
        before: bridge_step.before,
        after: ConfigurationProposal {
            id: bridge_step.after,
            value: configuration_term(package, runtime.configuration()),
        },
        observed_state: Some(facts.initial_state),
        budget: StepBudgetTransitionV2 {
            before: Budget {
                remaining_units: remaining_budget,
            },
            consumed_units: 1,
            after: Budget {
                remaining_units: after_budget,
            },
        },
        causes: vec![prior.map_or(
            StepCause::ActivationStart(runtime.activation()),
            StepCause::PriorStep,
        )],
        observations: vec![],
        candidate_delta,
        outcome: StepOutcomeProposalV2::Progress,
    };
    runtime
        .apply_carrier_ingress(&[
            ProcessRecordV2::EnteredObservation(occurrence),
            ProcessRecordV2::Steps(vec![step]),
        ])
        .expect("computed occurrence and Step enter the checked carrier atomically");
    (Some(reference), after_budget)
}

#[test]
fn package_owned_headless_scenario_reaches_one_admitted_render_state() {
    let (package, templates) = checked_program_package();
    let application = ApplicationId {
        snapshot: package.constitution().snapshot(),
        local: ApplicationLocalId::new(1),
    };
    let (authority, facts) = carrier_authority(&package);
    let mut runtime = ExecutableProcessRuntimeV1::instantiate(&package, &authority, application)
        .expect("checked package executable instantiates");

    let mut checker_activation = activation_template(&templates, 22);
    rebind_activation(&mut checker_activation, &package, facts);
    let checker_step = step_template(&templates, 53);
    let mut state_activation = activation_template(&templates, 23);
    rebind_activation(&mut state_activation, &package, facts);
    state_activation.id = runtime.activation();
    state_activation.membership = RunMembership::RootOf(runtime.run());
    state_activation.initial_configuration.id = runtime.configuration_id();
    state_activation.initial_configuration.value =
        configuration_term(&package, runtime.configuration());
    runtime
        .apply_carrier_ingress(&[
            ProcessRecordV2::ExternalTrigger(external_trigger_template(&templates, 12)),
            ProcessRecordV2::Activation(checker_activation),
            ProcessRecordV2::Steps(vec![checker_step]),
            ProcessRecordV2::Activation(state_activation),
        ])
        .expect("carrier accepts the package-bound activation basis");
    assert_eq!(runtime.carrier().carrier().state_revision_count(), 1);

    let mut prior = None;
    let mut remaining_budget = 100;

    runtime.advance(occurrence(0, &[1.0])).expect("input applies");
    (prior, remaining_budget) = carry_latest_step(
        &mut runtime,
        &package,
        facts,
        prior,
        remaining_budget,
        None,
    );
    runtime.advance(occurrence(2, &[0.25])).expect("ground tick applies");
    (prior, remaining_budget) = carry_latest_step(
        &mut runtime,
        &package,
        facts,
        prior,
        remaining_budget,
        None,
    );
    assert_eq!(value(runtime.configuration(), 0), 10.0);
    assert_eq!(value(runtime.configuration(), 2), 2.0);

    runtime.advance(occurrence(1, &[])).expect("grounded impulse applies");
    (prior, remaining_budget) = carry_latest_step(
        &mut runtime,
        &package,
        facts,
        prior,
        remaining_budget,
        None,
    );
    assert_eq!(value(runtime.configuration(), 3), 8.0);
    assert_eq!(runtime.configuration()[5].as_boolean(), Some(false));

    let before_rejected = runtime.configuration().to_vec();
    let rejected = runtime
        .advance(occurrence(1, &[]))
        .expect("unmatched occurrence still advances Configuration custody");
    assert!(!rejected.rule_applied);
    assert_eq!(runtime.configuration(), before_rejected);
    (prior, remaining_budget) = carry_latest_step(
        &mut runtime,
        &package,
        facts,
        prior,
        remaining_budget,
        None,
    );

    for _ in 0..6 {
        runtime.advance(occurrence(2, &[0.25])).expect("airborne tick applies");
        (prior, remaining_budget) = carry_latest_step(
            &mut runtime,
            &package,
            facts,
            prior,
            remaining_budget,
            None,
        );
    }
    runtime.advance(occurrence(2, &[0.25])).expect("landing tick applies");
    assert_eq!(value(runtime.configuration(), 1), 0.0);
    assert_eq!(value(runtime.configuration(), 3), 0.0);
    assert_eq!(runtime.configuration()[5].as_boolean(), Some(true));
    assert!(runtime.candidate().is_none());
    assert!(runtime.judgment().is_none());
    assert!(runtime.admission().is_none());

    let base = facts.initial_state;
    let candidate = runtime.emit_candidate(base).expect("candidate is emitted").clone();
    assert_eq!(candidate.base, base);
    assert!(runtime.judgment().is_none());
    assert!(runtime.admission().is_none());

    let mut carrier_candidate = step_template(&templates, 56)
        .candidate_delta
        .expect("stateful template carries a candidate");
    carrier_candidate.id = candidate.id;
    carrier_candidate.base = base;
    carrier_candidate.proposed_payload = configuration_term(&package, runtime.configuration());
    carrier_candidate.obligations[0].id.delta = candidate.id;
    let (producer, after_candidate_budget) = carry_latest_step(
        &mut runtime,
        &package,
        facts,
        prior,
        remaining_budget,
        Some(carrier_candidate),
    );
    let producer = producer.expect("candidate-producing Step entered the carrier");
    remaining_budget = after_candidate_budget;
    assert_eq!(candidate.produced_by, producer.step);
    assert_eq!(
        runtime
            .carrier()
            .carrier()
            .candidate_delta(candidate.id)
            .expect("candidate is owned by ProcessCarrier")
            .produced_by,
        producer
    );
    assert_eq!(runtime.carrier().carrier().state_revision_count(), 1);

    let judgment = runtime.judge(true).expect("candidate is judged").clone();
    assert_eq!(judgment.candidate, candidate.id);
    assert!(runtime.admission().is_none());

    let mut verdict = judgment_template(&templates, 90);
    let mut obligation = judgment_template(&templates, 91);
    for record in [&mut verdict, &mut obligation] {
        record.body.judgment.delta = candidate.id;
        record.body.judgment.session = facts.session;
        record.body.judgment.policy = facts.policy;
        record.body.supports[0].source = SupportSource::Step(producer);
        record.provenance = OccurrenceProvenance::EnteredThrough(EnteredThrough {
            boundary: facts.state_boundary,
            evidence: if record.body.id == judgment.id {
                id!(ExternalEvidenceRef, 186)
            } else {
                id!(ExternalEvidenceRef, 187)
            },
            causes: vec![CausalRef::CandidateDelta(candidate.id)],
        });
    }
    runtime
        .apply_carrier_ingress(&[
            ProcessRecordV2::Judgment(verdict),
            ProcessRecordV2::Judgment(obligation),
        ])
        .expect("carrier accepts separate verdict and obligation Judgments");
    assert!(runtime
        .carrier()
        .carrier()
        .judgment(judgment.id)
        .is_some());
    assert!(runtime.carrier().carrier().decision(candidate.id).is_none());

    let mut admission = admission_template(&templates);
    admission.delta = candidate.id;
    admission.provenance = EnteredThrough {
        boundary: facts.state_boundary,
        evidence: id!(ExternalEvidenceRef, 190),
        causes: vec![
            CausalRef::CandidateDelta(candidate.id),
            CausalRef::Judgment(id!(JudgmentOccurrenceId, 90)),
            CausalRef::Judgment(id!(JudgmentOccurrenceId, 91)),
        ],
    };
    let StateAdmissionOutcomeV2::Admit(ref mut carrier_successor) = admission.outcome else {
        panic!("selected template is an admitting decision");
    };
    carrier_successor.session = facts.session;
    carrier_successor.predecessor = Some(base);
    carrier_successor.cause = StateRevisionCause::Admission {
        occurrence: admission.occurrence,
        run: producer.run,
        activation: producer.activation,
        step: producer.step,
    };
    carrier_successor.payload = configuration_term(&package, runtime.configuration());
    carrier_successor.canonical_state_snapshot = canonical_term_bytes(&carrier_successor.payload)
        .expect("successor payload is canonical")
        .into_boxed_slice();
    carrier_successor.policy = facts.policy;
    carrier_successor.semantics = package.constitution().semantics();
    carrier_successor.id = carrier_successor.derived_id();
    let successor_id = carrier_successor.id;
    runtime
        .apply_carrier_ingress(&[ProcessRecordV2::AdmissionDecision(admission)])
        .expect("carrier admits the judged candidate into one successor");
    let successor = runtime
        .admit_with_state_id(successor_id)
        .expect("bridge records the carrier-derived successor")
        .clone();
    assert_eq!(successor.predecessor, base);
    assert_ne!(successor.id, base);

    runtime
        .apply_carrier_ingress(&[ProcessRecordV2::EnteredObservation(
            EnteredObservationV2 {
                observation: ObservationProposalV2::Value {
                    id: id!(ObservationId, 100),
                    value: configuration_term(&package, runtime.configuration()),
                    supports: vec![SupportUse {
                        slot: SupportSlotId::new(0),
                        role: Term::atom(
                            TermScope {
                                universe: package.constitution().universe(),
                                semantics: package.constitution().semantics(),
                            },
                            b"clause/process-observed-state-v1".to_vec(),
                            successor.id.as_bytes().to_vec(),
                            EqualityContract::ExactOctetsV1,
                        )
                        .expect("Observation role is canonical"),
                        source: SupportSource::Admission(id!(AdmissionOccurrenceId, 94)),
                    }],
                },
                provenance: EnteredThrough {
                    boundary: facts.pure_boundary,
                    evidence: id!(ExternalEvidenceRef, 181),
                    causes: vec![CausalRef::Admission(id!(AdmissionOccurrenceId, 94))],
                },
            },
        )])
        .expect("render Observation enters after and through the exact Admission");
    let observation = runtime.observe(&[0, 1, 3, 5]).expect("render projection exists");
    assert_eq!(observation.state, successor.id);
    assert_eq!(observation.value[0].as_number(), Some(10.0));
    assert_eq!(observation.value[1].as_number(), Some(0.0));
    assert_eq!(observation.value[2].as_number(), Some(0.0));
    assert_eq!(observation.value[3].as_boolean(), Some(true));

    assert_eq!(runtime.package(), package.id());
    assert_eq!(runtime.application(), application);
    assert!(runtime.carrier().carrier().accepted_ingress_record_count() > 0);
    assert_eq!(runtime.carrier().carrier().candidate_delta_count(), 1);
    assert_eq!(runtime.carrier().carrier().decision_count(), 1);
    assert_eq!(runtime.carrier().carrier().state_revision_count(), 2);
    assert!(runtime
        .carrier()
        .carrier()
        .observation(observation.id)
        .is_some());
    assert_eq!(remaining_budget, 89);
}
