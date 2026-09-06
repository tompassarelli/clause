use clause_package::*;
use clause_substrate::compiler_admission::{self, CompilerEvolutionRun, CompilerProgramCandidate};
use clause_substrate::compiler_package_v3 as kernel;
use clause_substrate::evaluator::Evaluator;
#[path = "support/compiler_world.rs"]
mod compiler_world;
use compiler_world::{World, nominal};

const COMPILER0: &[u8] = include_bytes!("fixtures/compiler-host-freeze/compiler0.clcp");

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/compiler-host-freeze")
            .join(name),
    )
    .unwrap()
}

fn bytes(value: &[u8]) -> kernel::Term {
    kernel::Term::Atom {
        kind: b"clause/core-abi/bytes/v1".to_vec(),
        canonical_payload: value.to_vec(),
        equality_contract: b"clause/core/bytes-equal/v1".to_vec(),
    }
}
fn tag(value: u8) -> kernel::Term {
    kernel::Term::Atom {
        kind: b"clause/core-abi/tag/v1".to_vec(),
        canonical_payload: vec![value],
        equality_contract: b"clause/core/bytes-equal/v1".to_vec(),
    }
}
fn triple(a: kernel::Term, b: kernel::Term, c: kernel::Term) -> kernel::Term {
    kernel::Term::Triple(
        kernel::FallibleBox::try_new(a).unwrap(),
        kernel::FallibleBox::try_new(b).unwrap(),
        kernel::FallibleBox::try_new(c).unwrap(),
    )
}
fn record(kind: u8, fields: Vec<kernel::Term>) -> kernel::Term {
    triple(
        tag(kind),
        fields
            .into_iter()
            .rev()
            .fold(tag(0), |tail, head| triple(tag(1), head, tail)),
        tag(0),
    )
}

fn constitute(
    compiler: &[u8],
    selected: &[u8],
    request: &[u8],
    grants: &[u8],
) -> CheckedProcessPackage {
    let decoded = kernel::decode(compiler).unwrap();
    let evaluator = Evaluator::new(&decoded.package().subject.program).unwrap();
    let result = evaluator
        .invoke_entrypoint(
            kernel::Id32(nominal(34)),
            &[kernel::KValue::Term(record(
                0x99,
                vec![bytes(selected), bytes(request), bytes(grants)],
            ))],
            1_000_000_000,
        )
        .unwrap();
    let kernel::KValue::Bytes(exact) = result.value else {
        panic!("compiler constitution has byte sort")
    };
    check_process_package(decode_process_package(&exact).unwrap()).unwrap()
}

fn value_bytes(value: &kernel::KValue) -> Vec<u8> {
    match value {
        kernel::KValue::Term(term) => {
            [vec![1], kernel::encode_canonical_term(term).unwrap()].concat()
        }
        kernel::KValue::Bytes(bytes) => [
            vec![0],
            (bytes.len() as u32).to_be_bytes().to_vec(),
            bytes.clone(),
        ]
        .concat(),
    }
}
fn parts(term: &kernel::Term) -> (&kernel::Term, &kernel::Term, &kernel::Term) {
    let kernel::Term::Triple(a, b, c) = term else {
        panic!("expected neutral triple")
    };
    (a, b, c)
}
fn data(term: &kernel::Term) -> &[u8] {
    let kernel::Term::Atom {
        canonical_payload, ..
    } = term
    else {
        panic!("expected Atom")
    };
    canonical_payload
}
fn field(term: &kernel::Term, index: usize) -> &kernel::Term {
    let mut fields = parts(term).1;
    for _ in 0..index {
        fields = parts(fields).2;
    }
    parts(fields).1
}
fn result_subject(result: &kernel::KValue, tag: u8) -> &[u8] {
    let kernel::KValue::Term(term) = result else {
        panic!("expected Term")
    };
    assert_eq!(data(parts(term).0), [tag]);
    data(field(term, 0))
}

fn records<'a>(term: &'a kernel::Term, tag: u8, out: &mut Vec<&'a kernel::Term>) {
    if let kernel::Term::Triple(a, b, c) = term {
        if matches!(&**a,kernel::Term::Atom{kind,canonical_payload,..}if kind==b"clause/core-abi/tag/v1"&&canonical_payload==&[tag])
        {
            out.push(term);
        }
        records(a, tag, out);
        records(b, tag, out);
        records(c, tag, out);
    }
}
fn analyzed(e: &clause_substrate::evaluator::Evaluation) -> &kernel::Term {
    let kernel::KValue::Term(term) = &e.value else {
        panic!("analyzer returns a Term")
    };
    assert_eq!(data(parts(term).0), [0x95]);
    field(term, 3)
}
fn value_is(e: &clause_substrate::evaluator::Evaluation, expected: kernel::Term) {
    assert_eq!(data(parts(analyzed(e)).0), [0x71]);
    assert_eq!(field(analyzed(e), 1), &expected);
}
fn check_language_specimens(zero: &[u8], one: &[u8], lean: &str) {
    let a = kernel::decode(zero).unwrap();
    let b = kernel::decode(one).unwrap();
    let e0 = Evaluator::new(&a.package().subject.program).unwrap();
    let e1 = Evaluator::new(&b.package().subject.program).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/compiler-host-freeze");
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/host-freeze-cross-host");
    std::fs::create_dir_all(&output).unwrap();
    let mut identities = std::collections::BTreeMap::new();
    for name in [
        "old-binding",
        "new-binding",
        "alpha-renamed",
        "typed-macro",
        "macro-phase",
        "macro-type",
        "macro-without-capability",
        "effect",
        "effect-without-capability",
        "macro-effect",
        "unresolved",
    ] {
        let request = fixture(&format!("{name}.request.term"));
        assert_eq!(
            data(field(&kernel::decode_canonical_term(&request).unwrap(), 0)),
            fixture(&format!("{name}.source"))
        );
        let first = e0
            .invoke_entrypoint(
                kernel::Id32(nominal(26)),
                &[kernel::KValue::Term(
                    kernel::decode_canonical_term(&request).unwrap(),
                )],
                10_000_000,
            )
            .unwrap();
        let second = e1
            .invoke_entrypoint(
                kernel::Id32(nominal(26)),
                &[kernel::KValue::Term(
                    kernel::decode_canonical_term(&request).unwrap(),
                )],
                10_000_000,
            )
            .unwrap();
        for (index, result) in [(0, &first), (1, &second)] {
            let path = output.join(format!("{name}.lean{index}.value"));
            assert!(
                std::process::Command::new(lean)
                    .arg("evaluate")
                    .arg(root.join(format!("compiler{index}.clcp")))
                    .arg(root.join("analyze.id"))
                    .arg(root.join(format!("{name}.request.term")))
                    .arg("10000000")
                    .arg(&path)
                    .status()
                    .unwrap()
                    .success()
            );
            assert_eq!(std::fs::read(&path).unwrap(), value_bytes(&result.value));
            assert_eq!(
                std::fs::read(format!("{}.observations", path.display())).unwrap(),
                kernel::encode_canonical_term(&result.observations.try_to_term().unwrap()).unwrap()
            );
            assert_eq!(
                std::fs::read_to_string(format!("{}.fuel", path.display())).unwrap(),
                format!("{}\n", result.remaining_fuel)
            );
        }
        match name {
            "old-binding" => {
                value_is(&first, triple(bytes(b"inner"), bytes(b"outer"), tag(0)));
                assert_eq!(first.value, second.value);
            }
            "new-binding" | "alpha-renamed" => {
                assert_eq!(data(parts(analyzed(&first)).0), [0x15]);
                value_is(&second, triple(bytes(b"inner"), bytes(b"outer"), tag(0)));
                let mut uses = vec![];
                records(analyzed(&second), 0x73, &mut uses);
                identities.insert(
                    name,
                    uses.iter()
                        .map(|r| (data(field(r, 0)).to_vec(), data(field(r, 1)).to_vec()))
                        .collect::<Vec<_>>(),
                );
            }
            "typed-macro" => {
                assert_eq!(data(parts(analyzed(&first)).0), [0x15]);
                value_is(
                    &second,
                    triple(bytes(b"generated"), bytes(b"caller"), tag(0)),
                );
                let mut binders = vec![];
                records(analyzed(&second), 0x72, &mut binders);
                assert_eq!(
                    binders
                        .iter()
                        .map(|r| data(field(r, 0)))
                        .collect::<std::collections::BTreeSet<_>>()
                        .len(),
                    2
                );
                let mut origins = vec![];
                records(analyzed(&second), 0x82, &mut origins);
                assert_eq!(origins.len(), 1);
            }
            "macro-phase"
            | "macro-type"
            | "macro-without-capability"
            | "effect-without-capability" => assert_eq!(data(parts(analyzed(&second)).0), [0x15]),
            "effect" => {
                assert_eq!(data(parts(analyzed(&first)).0), [0x15]);
                value_is(&second, bytes(b"message"));
                let mut effects = vec![];
                records(analyzed(&second), 0x9a, &mut effects);
                assert_eq!(effects.len(), 1);
                execute_effect(effects[0]);
            }
            "macro-effect" => {
                value_is(
                    &second,
                    triple(bytes(b"one message"), bytes(b"body"), tag(0)),
                );
                let mut effects = vec![];
                records(analyzed(&second), 0x9a, &mut effects);
                assert_eq!(effects.len(), 1);
                let checked = check_process_package(
                    decode_process_package(data(field(effects[0], 7))).unwrap(),
                )
                .unwrap();
                assert_eq!(
                    checked
                        .constitution()
                        .formation(FormationLocalId::new(3))
                        .unwrap()
                        .term
                        .as_atom()
                        .unwrap()
                        .canonical_payload(),
                    b"one message"
                );
            }
            "unresolved" => {
                assert_eq!(data(parts(analyzed(&first)).0), [0x15]);
                assert_eq!(data(parts(analyzed(&second)).0), [0x15]);
                let mut left = vec![];
                let mut right = vec![];
                records(analyzed(&first), 0x91, &mut left);
                records(analyzed(&second), 0x91, &mut right);
                assert_eq!((left.len(), right.len()), (1, 1));
                for index in 0..6 {
                    assert_eq!(field(left[0], index), field(right[0], index));
                }
                assert_ne!(field(left[0], 6), field(right[0], 6));
                assert_ne!(field(left[0], 7), field(right[0], 7));
            }
            _ => unreachable!(),
        }
        println!("{name}: exact Rust/Lean value, observations and fuel; package behavior checked");
    }
    assert_eq!(
        identities["new-binding"], identities["alpha-renamed"],
        "alpha-renaming preserves exact use-to-binder edges"
    );
}

fn execute_effect(effect: &kernel::Term) {
    let checked =
        check_process_package(decode_process_package(data(field(effect, 7))).unwrap()).unwrap();
    let revision = ProgramRevisionPreimage {
        semantics: checked.constitution().semantics(),
        program: ProgramId::from_bytes(nominal(800)),
        predecessor: None,
        snapshot: checked.constitution().snapshot(),
        change: ProgramChangeOccurrenceId::from_bytes(nominal(801)),
    }
    .derived_claim();
    let mut world = World::prepare(&checked, revision);
    world
        .authority
        .admit_genesis(
            revision,
            checked.authority_input(),
            world.policy,
            world.genesis,
        )
        .unwrap();
    let mut carrier = world.start_session(&checked);
    let mut journal = vec![];
    let record = |carrier: &mut ProcessCarrier,
                  journal: &mut Vec<ProcessRecordV2>,
                  record: ProcessRecordV2| {
        carrier
            .apply_ingress(std::slice::from_ref(&record), &world.authority)
            .unwrap();
        journal.push(record);
    };
    let root = world.activation(
        &carrier,
        810,
        811,
        1,
        ActivationOrigin::RootedBy(RootTrigger::SessionStart(world.start)),
        data(field(effect, 3)),
    );
    let scope = EffectScopeV1 {
        application: root.application,
        mode: root.mode,
        program_revision: revision.id,
        world: world.initial,
        session: world.session,
        budget: Budget {
            remaining_units: 99,
        },
    };
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::Activation(root),
    );
    let admitted_payload = world.atom(b"one admitted mailbox intent");
    let mut emit = world.step(
        &carrier,
        820,
        810,
        100,
        data(field(effect, 3)),
        vec![StepCause::ActivationStart(ActivationId::from_bytes(
            nominal(810),
        ))],
    );
    emit.observation_outcomes = vec![world.formation_observation(
        830,
        admitted_payload.clone(),
        SupportSource::SessionStart(world.start),
    )];
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::Steps(vec![emit]),
    );
    let emitted_by = carrier
        .step(StepId::from_bytes(nominal(820)))
        .unwrap()
        .reference();
    let contract = &checked
        .constitution()
        .mode_by_id(scope.mode)
        .unwrap()
        .contract
        .effect_intents[0];
    let role = |role: RoleLocalId| {
        let binding = checked
            .constitution()
            .application_by_id(scope.application)
            .unwrap()
            .form
            .bindings
            .iter()
            .find(|b| b.role == role && b.occurrence == 0)
            .unwrap();
        let RoleBindingValuePreimageV2::Known(formation) = binding.value else {
            panic!("effect input must be known")
        };
        checked
            .constitution()
            .formation(formation)
            .unwrap()
            .term
            .clone()
    };
    let intent = EffectIntentOccurrenceV1 {
        id: EffectIntentId::from_bytes(data(field(effect, 5)).try_into().unwrap()),
        emitted_by,
        contract_index: 0,
        required_capability: CapabilityRef {
            snapshot: revision.preimage.snapshot,
            local: contract.required_capability,
        },
        scope,
        action: role(contract.action_role),
        resource: role(contract.resource_role),
        payload: role(contract.payload_role),
    };
    assert_eq!(
        intent.action.as_atom().unwrap().canonical_payload(),
        data(field(effect, 0))
    );
    assert_eq!(
        intent.resource.as_atom().unwrap().canonical_payload(),
        data(field(effect, 2))
    );
    assert_eq!(
        intent.payload.as_atom().unwrap().canonical_payload(),
        data(field(effect, 3))
    );
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::EffectIntent(intent.clone()),
    );
    let mut authorization = IssuedEffectAuthorizationV1 {
        id: IssuedEffectAuthorizationOccurrenceId::from_bytes(nominal(840)),
        intent: intent.id,
        admission: world.admission,
        activation: ActivationId::from_bytes(nominal(813)),
        capability: intent.required_capability,
        scope: intent.scope,
        action: intent.action.clone(),
        resource: intent.resource.clone(),
        payload: intent.payload.clone(),
    };
    assert!(
        matches!(carrier.apply_ingress(&[ProcessRecordV2::IssuedEffectAuthorization(authorization.clone())],&world.authority),Err(ProcessIngressError::Record{cause,..})if matches!(*cause,ProcessError::EffectIntentNotAdmitted(_)))
    );
    let checker = world.activation(
        &carrier,
        812,
        811,
        2,
        ActivationOrigin::ChildOf {
            run: emitted_by.run,
            parent_activation: emitted_by.activation,
            parent_step: emitted_by.step,
        },
        b"check proposed intent admission",
    );
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::Activation(checker),
    );
    let mut check = world.step(
        &carrier,
        821,
        812,
        100,
        b"checked exact intent",
        vec![
            StepCause::ActivationStart(ActivationId::from_bytes(nominal(812))),
            StepCause::PriorStep(emitted_by),
        ],
    );
    check.observation_outcomes = vec![world.formation_observation(
        831,
        admitted_payload.clone(),
        SupportSource::Step(emitted_by),
    )];
    check.outcome = StepOutcomeProposalV2::Return(DomainBoundTermV2 {
        term: admitted_payload.clone(),
        evidence: ObservationId::from_bytes(nominal(830)),
    });
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::Steps(vec![check]),
    );
    let checked_by = carrier
        .step(StepId::from_bytes(nominal(821)))
        .unwrap()
        .reference();
    let mut propose = world.step(
        &carrier,
        822,
        810,
        99,
        b"propose intent admission",
        vec![StepCause::PriorStep(checked_by)],
    );
    let bound = DomainBoundTermV2 {
        term: admitted_payload.clone(),
        evidence: ObservationId::from_bytes(nominal(831)),
    };
    propose.candidate_delta = Some(CandidateDeltaV2 {
        id: world.delta,
        base: world.initial,
        delta: bound.clone(),
        proposed_payload: admitted_payload.clone(),
        evidence: vec![SupportUse {
            slot: SupportSlotId::new(0),
            role: world.atom(b"intent producer"),
            source: SupportSource::Step(emitted_by),
        }],
        obligations: vec![],
        effect_intents: vec![intent.id],
    });
    propose.outcome = StepOutcomeProposalV2::Return(bound);
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::Steps(vec![propose]),
    );
    let producer = carrier
        .step(StepId::from_bytes(nominal(822)))
        .unwrap()
        .reference();
    let state = world.admit(&mut carrier, producer, admitted_payload);
    journal.push(ProcessRecordV2::Judgment(
        carrier
            .judgment(JudgmentOccurrenceId::from_bytes(nominal(710)))
            .unwrap()
            .clone(),
    ));
    journal.push(ProcessRecordV2::AdmissionDecision(
        carrier
            .decision_by_occurrence(world.admission)
            .unwrap()
            .clone(),
    ));
    let mut physical = world.activation(
        &carrier,
        813,
        814,
        1,
        ActivationOrigin::RootedBy(RootTrigger::Admitted(world.admission)),
        b"perform admitted intent",
    );
    physical.pins.observed_state = Some(state);
    authorization.scope.world = state;
    authorization.scope.budget = physical.pins.budget;
    assert_ne!(physical.id, intent.emitted_by.activation);
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::Activation(physical),
    );
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::IssuedEffectAuthorization(authorization.clone()),
    );
    let attempt = EffectAttemptOccurrenceV1 {
        id: EffectAttemptId::from_bytes(nominal(841)),
        intent: intent.id,
        authorization: authorization.id,
        activation: authorization.activation,
        scope: authorization.scope,
        action: authorization.action.clone(),
        resource: authorization.resource.clone(),
        payload: authorization.payload.clone(),
    };
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::EffectAttempt(attempt.clone()),
    );
    // The only physical operation is an explicitly supplied test mailbox. Its
    // implementation receives authorized bytes; it has no source-form cases.
    let mailbox = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/host-freeze-cross-host/mailbox");
    std::fs::write(&mailbox, []).unwrap();
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(&mailbox)
        .unwrap()
        .write_all(attempt.payload.as_atom().unwrap().canonical_payload())
        .unwrap();
    let observed = std::fs::read(&mailbox).unwrap();
    assert_eq!(observed, b"message");
    record(
        &mut carrier,
        &mut journal,
        ProcessRecordV2::EffectReceipt(EffectReceiptOccurrenceV1 {
            id: EffectReceiptId::from_bytes(nominal(842)),
            attempt: attempt.id,
            status: 0,
            exact_bytes: observed.clone(),
        }),
    );
    let mut duplicate = attempt.clone();
    duplicate.id = EffectAttemptId::from_bytes(nominal(843));
    assert!(
        matches!(carrier.apply_ingress(&[ProcessRecordV2::EffectAttempt(duplicate)],&world.authority),Err(ProcessIngressError::Record{cause,..})if matches!(*cause,ProcessError::EffectAuthorizationAlreadyConsumed(_)))
    );
    let mut replay = ProcessCarrier::instantiate(&checked, &world.authority).unwrap();
    replay.apply_ingress(&journal, &world.authority).unwrap();
    assert_eq!(replay.effect_attempt(attempt.id), Some(&attempt));
    assert_eq!(
        std::fs::read(&mailbox).unwrap(),
        observed,
        "replay performs no mailbox operation"
    );
    println!(
        "effect: governed intent Admission, distinct authorized Activation, one mailbox attempt, no replay attempt"
    );
}

#[test]
#[ignore = "requires prebuilt frozen Lean executable and the full compiler succession fixtures"]
fn compiler0_successor_crosses_governed_admission_with_frozen_hosts() {
    use kernel::{
        CompilerEvidence, FinalPackageIdentityInput, GenesisAuthorizationRequest, OwnerAnchorInput,
        OwnerAnchorObservation, OwnerAnchorWitness, SuccessorAuthorizationRequest,
    };
    let lean = std::env::var("CLAUSE_FROZEN_LEAN")
        .expect("provide the already-built frozen Lean executable");
    let freeze = || {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        for path in [
            std::env::current_exe().unwrap(),
            std::path::PathBuf::from(&lean),
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/compiler_runtime/host-mechanics.tsv"),
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/compiler_runtime/source-ast-mechanics.tsv"),
        ] {
            hash.update(std::fs::read(path).unwrap());
        }
        hash.finalize().to_vec()
    };
    let frozen = freeze();
    let one = fixture("compiler1.clcp");
    let zero = kernel::decode(COMPILER0).unwrap();
    let candidate = kernel::decode(&one).unwrap();
    assert_eq!(zero.exact_core_manifest(), candidate.exact_core_manifest());
    assert_eq!(
        zero.package().subject.interface,
        candidate.package().subject.interface
    );
    let changed = zero
        .package()
        .subject
        .program
        .iter()
        .zip(&candidate.package().subject.program)
        .filter_map(|(a, b)| {
            assert_eq!(
                (a.id, &a.arguments, a.result),
                (b.id, &b.arguments, b.result)
            );
            (a.body != b.body).then_some(a.id)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        changed,
        vec![30, 31, 32, 33]
            .into_iter()
            .map(|n| kernel::Id32(nominal(n)))
            .collect::<Vec<_>>()
    );
    let build = kernel::encode_canonical_term(&candidate.package().subject.build_request).unwrap();
    let process1 = constitute(COMPILER0, &one, &build, &[0; 4]);
    let program = ProgramId::from_bytes(nominal(720));
    let change = ProgramChangeOccurrenceId::from_bytes(nominal(722));
    let grants = [
        1u32.to_be_bytes().as_slice(),
        1u32.to_be_bytes().as_slice(),
        process1.constitution().semantics().as_bytes(),
        program.as_bytes(),
        process1.constitution().snapshot().as_bytes(),
        change.as_bytes(),
    ]
    .concat();
    let process0 = constitute(COMPILER0, COMPILER0, &build, &grants);
    let revision0 = ProgramRevisionPreimage {
        semantics: process0.constitution().semantics(),
        program,
        predecessor: None,
        snapshot: process0.constitution().snapshot(),
        change: ProgramChangeOccurrenceId::from_bytes(nominal(721)),
    }
    .derived_claim();
    let revision1 = ProgramRevisionPreimage {
        semantics: process1.constitution().semantics(),
        program,
        predecessor: Some(revision0.id),
        snapshot: process1.constitution().snapshot(),
        change,
    }
    .derived_claim();
    let mut world = World::prepare(&process0, revision0);
    let predecessor = compiler_admission::admit_genesis(
        &mut world.authority,
        CompilerProgramCandidate {
            exact_compiler_bytes: COMPILER0,
            checked_process: &process0,
            compiler_formation: FormationLocalId::new(1),
            revision: revision0,
        },
        GenesisAuthorizationRequest {
            owner_anchor: OwnerAnchorInput::Supplied(OwnerAnchorWitness::from_external_selection(
                OwnerAnchorObservation {
                    exact_selected_bytes: COMPILER0,
                    selected_byte_length: COMPILER0.len() as u64,
                    selected_package_hash: kernel::compiler_package_hash(COMPILER0),
                },
            )),
            build_request: &zero.package().subject.build_request,
            evidence: &zero.package().evidence,
            compile_fuel_limit: 1_000_000_000,
            admission_fuel_limit: 1_000_000_000,
            final_identity: FinalPackageIdentityInput {
                package_hash: kernel::compiler_package_hash(COMPILER0),
                exact_package_bytes: COMPILER0,
            },
        },
        world.policy,
        world.genesis,
    )
    .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/compiler-host-freeze");
    assert!(
        std::process::Command::new(&lean)
            .args(["test-genesis"])
            .arg(root.join("compiler0.clcp"))
            .arg(root.join("compiler0.clcp"))
            .status()
            .unwrap()
            .success()
    );
    let mut carrier = world.start_session(&process0);
    let activation = world.activation(
        &carrier,
        730,
        731,
        1,
        ActivationOrigin::RootedBy(RootTrigger::SessionStart(world.start)),
        &build,
    );
    carrier
        .apply_ingress(&[ProcessRecordV2::Activation(activation)], &world.authority)
        .unwrap();
    let evaluator = Evaluator::new(&zero.package().subject.program).unwrap();
    let compiled = evaluator
        .invoke_entrypoint(
            zero.package().subject.interface.compile,
            &[kernel::KValue::Term(
                kernel::decode_canonical_term(&build).unwrap(),
            )],
            1_000_000_000,
        )
        .unwrap();
    assert_eq!(
        result_subject(&compiled.value, 0x14),
        candidate.exact_subject()
    );
    let compile_observations =
        kernel::encode_canonical_term(&compiled.observations.try_to_term().unwrap()).unwrap();
    let mut compile_step = world.step(
        &carrier,
        740,
        730,
        100,
        &value_bytes(&compiled.value),
        vec![StepCause::ActivationStart(ActivationId::from_bytes(
            nominal(730),
        ))],
    );
    compile_step.observation_outcomes = vec![
        world.value_observation(
            750,
            &compile_observations,
            SupportSource::SessionStart(world.start),
        ),
        world.value_observation(
            751,
            &compiled.remaining_fuel.to_be_bytes(),
            SupportSource::SessionStart(world.start),
        ),
    ];
    compile_step.observation_outcomes.sort();
    carrier
        .apply_ingress(
            &[ProcessRecordV2::Steps(vec![compile_step])],
            &world.authority,
        )
        .unwrap();
    let compile_ref = carrier
        .step(StepId::from_bytes(nominal(740)))
        .unwrap()
        .reference();
    let proposal_argument = record(
        0x16,
        vec![
            kernel::decode_canonical_term(&build).unwrap(),
            bytes(candidate.exact_subject()),
            kernel::decode_canonical_term(&compile_observations).unwrap(),
        ],
    );
    let proposed = evaluator
        .invoke_entrypoint(
            zero.package().subject.interface.admit_propose,
            &[kernel::KValue::Term(proposal_argument)],
            1_000_000_000,
        )
        .unwrap();
    assert_eq!(
        result_subject(&proposed.value, 0x17),
        candidate.exact_subject()
    );
    let proposed_payload = world.atom(process1.exact_bytes());
    let mut proposal_step = world.step(
        &carrier,
        741,
        730,
        99,
        &value_bytes(&proposed.value),
        vec![StepCause::PriorStep(compile_ref)],
    );
    proposal_step.observation_outcomes = vec![
        world.value_observation(
            752,
            &kernel::encode_canonical_term(&proposed.observations.try_to_term().unwrap()).unwrap(),
            SupportSource::Step(compile_ref),
        ),
        world.value_observation(
            753,
            &proposed.remaining_fuel.to_be_bytes(),
            SupportSource::Step(compile_ref),
        ),
        world.formation_observation(
            754,
            proposed_payload.clone(),
            SupportSource::Step(compile_ref),
        ),
    ];
    proposal_step.observation_outcomes.sort();
    carrier
        .apply_ingress(
            &[ProcessRecordV2::Steps(vec![proposal_step])],
            &world.authority,
        )
        .unwrap();
    let proposal_ref = carrier
        .step(StepId::from_bytes(nominal(741)))
        .unwrap()
        .reference();
    let checker = world.activation(
        &carrier,
        732,
        731,
        2,
        ActivationOrigin::ChildOf {
            run: proposal_ref.run,
            parent_activation: proposal_ref.activation,
            parent_step: proposal_ref.step,
        },
        &one,
    );
    carrier
        .apply_ingress(&[ProcessRecordV2::Activation(checker)], &world.authority)
        .unwrap();
    let successor_request = || SuccessorAuthorizationRequest {
        predecessor: predecessor.predecessor(COMPILER0),
        build_request: &candidate.package().subject.build_request,
        evidence: &candidate.package().evidence,
        final_identity: FinalPackageIdentityInput {
            package_hash: kernel::compiler_package_hash(&one),
            exact_package_bytes: &one,
        },
    };
    assert_eq!(
        kernel::authorize_successor(&one, successor_request()).unwrap(),
        kernel::AuthorizationVerdict::Authorized(one.clone())
    );
    assert!(
        std::process::Command::new(&lean)
            .arg("test-successor")
            .arg(root.join("compiler0.clcp"))
            .arg(root.join("compiler0.clcp"))
            .arg(root.join("compiler1.clcp"))
            .status()
            .unwrap()
            .success()
    );
    let mut checker_step = world.step(
        &carrier,
        742,
        732,
        100,
        &one,
        vec![
            StepCause::ActivationStart(ActivationId::from_bytes(nominal(732))),
            StepCause::PriorStep(proposal_ref),
        ],
    );
    checker_step.observation_outcomes = vec![world.formation_observation(
        755,
        proposed_payload.clone(),
        SupportSource::Step(proposal_ref),
    )];
    checker_step.outcome = StepOutcomeProposalV2::Return(DomainBoundTermV2 {
        term: proposed_payload.clone(),
        evidence: ObservationId::from_bytes(nominal(754)),
    });
    carrier
        .apply_ingress(
            &[ProcessRecordV2::Steps(vec![checker_step])],
            &world.authority,
        )
        .unwrap();
    let checker_ref = carrier
        .step(StepId::from_bytes(nominal(742)))
        .unwrap()
        .reference();
    let mut candidate_step = world.step(
        &carrier,
        743,
        730,
        98,
        process1.exact_bytes(),
        vec![StepCause::PriorStep(checker_ref)],
    );
    let bound = DomainBoundTermV2 {
        term: proposed_payload.clone(),
        evidence: ObservationId::from_bytes(nominal(755)),
    };
    candidate_step.candidate_delta = Some(CandidateDeltaV2 {
        id: world.delta,
        base: world.initial,
        delta: bound.clone(),
        proposed_payload: proposed_payload.clone(),
        evidence: vec![SupportUse {
            slot: SupportSlotId::new(0),
            role: world.atom(b"checked candidate"),
            source: SupportSource::Step(checker_ref),
        }],
        obligations: vec![],
        effect_intents: vec![],
    });
    candidate_step.outcome = StepOutcomeProposalV2::Return(bound);
    carrier
        .apply_ingress(
            &[ProcessRecordV2::Steps(vec![candidate_step])],
            &world.authority,
        )
        .unwrap();
    assert!(
        world.authority.revision(revision1.id).is_none(),
        "neither compilation nor checker verdict admits a Program"
    );
    let candidate_ref = carrier
        .step(StepId::from_bytes(nominal(743)))
        .unwrap()
        .reference();
    world.admit(&mut carrier, candidate_ref, proposed_payload);
    let admitted = compiler_admission::admit_successor(
        &mut world.authority,
        &predecessor,
        CompilerProgramCandidate {
            exact_compiler_bytes: &one,
            checked_process: &process1,
            compiler_formation: FormationLocalId::new(1),
            revision: revision1,
        },
        successor_request(),
        CompilerEvolutionRun {
            carrier: &carrier,
            run: RunId::from_bytes(nominal(731)),
            compiler_role: RoleLocalId::new(1),
            compile_step: StepId::from_bytes(nominal(740)),
            compile_observations: ObservationId::from_bytes(nominal(750)),
            compile_remaining_fuel: ObservationId::from_bytes(nominal(751)),
            proposal_step: StepId::from_bytes(nominal(741)),
            proposal_observations: ObservationId::from_bytes(nominal(752)),
            proposal_remaining_fuel: ObservationId::from_bytes(nominal(753)),
            checker_step: StepId::from_bytes(nominal(742)),
            candidate_step: StepId::from_bytes(nominal(743)),
            candidate_delta: world.delta,
            admission: world.admission,
        },
        AdmissionAuthorizationRef {
            snapshot: revision0.preimage.snapshot,
            local: AdmissionAuthorizationLocalId::new(1),
        },
    )
    .unwrap();
    assert_eq!(admitted.exact_bytes(), one);
    assert!(world.authority.revision(revision1.id).is_some());
    assert!(matches!(
        carrier
            .activation(ActivationId::from_bytes(nominal(730)))
            .unwrap()
            .status(),
        ActivationStatus::Terminal(ActivationTerminal::Returned)
    ));
    assert!(matches!(
        carrier
            .activation(ActivationId::from_bytes(nominal(732)))
            .unwrap()
            .status(),
        ActivationStatus::Terminal(ActivationTerminal::Returned)
    ));
    let CompilerEvidence::Successor {
        compile_receipt,
        admission_receipt,
    } = &candidate.package().evidence
    else {
        panic!("successor evidence")
    };
    assert_eq!(
        compiled.remaining_fuel,
        compile_receipt.expected_remaining_fuel
    );
    assert_eq!(
        proposed.remaining_fuel,
        admission_receipt.expected_remaining_fuel
    );
    check_language_specimens(COMPILER0, &one, &lean);
    assert_eq!(
        frozen,
        freeze(),
        "both executable files and both mechanics manifests remain byte-identical"
    );
}

#[test]
fn package_constructs_the_compiler_application_and_exact_input_roles() {
    let checked = constitute(COMPILER0, COMPILER0, b"exact build request", &[0; 4]);
    let revision = ProgramRevisionPreimage {
        semantics: checked.constitution().semantics(),
        program: ProgramId::from_bytes(nominal(720)),
        predecessor: None,
        snapshot: checked.constitution().snapshot(),
        change: ProgramChangeOccurrenceId::from_bytes(nominal(721)),
    }
    .derived_claim();
    let mut world = World::prepare(&checked, revision);
    world
        .authority
        .admit_genesis(
            revision,
            checked.authority_input(),
            world.policy,
            world.genesis,
        )
        .unwrap();
    let mut carrier = world.start_session(&checked);
    let activation = world.activation(
        &carrier,
        730,
        731,
        1,
        ActivationOrigin::RootedBy(RootTrigger::SessionStart(world.start)),
        b"exact build request",
    );
    carrier
        .apply_ingress(&[ProcessRecordV2::Activation(activation)], &world.authority)
        .unwrap();
    assert_eq!(carrier.activation_count(), 1);
    assert_eq!(
        checked
            .constitution()
            .formation(FormationLocalId::new(1))
            .unwrap()
            .term
            .as_atom()
            .unwrap()
            .canonical_payload(),
        COMPILER0
    );
    assert_eq!(
        checked
            .constitution()
            .formation(FormationLocalId::new(2))
            .unwrap()
            .term
            .as_atom()
            .unwrap()
            .canonical_payload(),
        b"exact build request"
    );
}
