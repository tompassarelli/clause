use clause_package::*;

pub fn nominal(n: u32) -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[0] = 0x31;
    bytes[28..].copy_from_slice(&n.to_be_bytes());
    bytes
}

pub struct World {
    pub authority: AuthorityStore,
    pub revision: ProgramRevisionClaim,
    pub policy: RootPolicyId,
    pub genesis: RootAdmissionAuthorizationRef,
    pub initial: StateRevisionId,
    pub session: RuntimeSessionId,
    pub runtime_policy: RuntimePolicyId,
    pub start: SessionStartOccurrenceId,
    pub delta: CandidateDeltaId,
    pub admission: AdmissionOccurrenceId,
    pub boundary: BoundaryRef,
    pub evidence: ExternalEvidenceRef,
    pub scope: TermScope,
    pub target: FormationTargetV2,
}

impl World {
    pub fn prepare(package: &CheckedProcessPackage, revision: ProgramRevisionClaim) -> Self {
        let constitution = package.constitution();
        let scope = TermScope {
            universe: constitution.universe(),
            semantics: constitution.semantics(),
        };
        let view = &package.initial_state_views()[0];
        let session = view.session;
        let runtime_policy = RuntimePolicyId::from_bytes(nominal(703));
        let start = SessionStartOccurrenceId::from_bytes(nominal(704));
        let anchor = RuntimeSessionAnchor::establish(
            session,
            revision.id,
            scope.semantics,
            runtime_policy,
            start,
            view.canonical_state_snapshot.to_vec(),
        );
        let initial = anchor.initial_state_id();
        let policy = RootPolicyId::from_bytes(nominal(705));
        let genesis = RootAdmissionAuthorizationRef {
            policy,
            local: AdmissionAuthorizationLocalId::new(0),
        };
        let delta = CandidateDeltaId::from_bytes(nominal(706));
        let admission = AdmissionOccurrenceId::from_bytes(nominal(707));
        let boundary = BoundaryRef::from_bytes(nominal(708));
        let evidence = ExternalEvidenceRef::from_bytes(nominal(709));
        let target = constitution.preimage().schemas[0].result_domain.clone();
        let mut authority = AuthorityStore::new();
        authority
            .establish_root_policy(
                RootPolicyAnchor::establish_with_governance(
                    policy,
                    vec![RootGenesisGrant {
                        authorization: genesis,
                        scope: RootGenesisScope {
                            semantics: scope.semantics,
                            program: revision.preimage.program,
                            snapshot: revision.preimage.snapshot,
                            change: revision.preimage.change,
                        },
                    }],
                    vec![],
                    vec![RootStateAdmissionGrant {
                        authorization: RootAdmissionAuthorizationRef {
                            policy,
                            local: AdmissionAuthorizationLocalId::new(1),
                        },
                        scope: CheckedStateAdmissionScope {
                            package: package.id(),
                            session,
                            base: initial,
                            delta,
                        },
                    }],
                    vec![RootJudgmentAuthorityGrant {
                        authority: RootJudgmentAuthorityRef {
                            policy,
                            local: JudgmentAuthorityLocalId::new(1),
                        },
                        scope: JudgmentAuthorityScope {
                            semantics: scope.semantics,
                            session,
                            policy: runtime_policy,
                        },
                    }],
                    vec![],
                )
                .unwrap(),
            )
            .unwrap();
        // Establishment requires the revision to have crossed outer Admission;
        // callers invoke start_session only after the compiler bridge does so.
        Self {
            authority,
            revision,
            policy,
            genesis,
            initial,
            session,
            runtime_policy,
            start,
            delta,
            admission,
            boundary,
            evidence,
            scope,
            target,
        }
    }

    pub fn start_session(&mut self, package: &CheckedProcessPackage) -> ProcessCarrier {
        self.authority
            .establish_runtime_session(RuntimeSessionAnchor::establish(
                self.session,
                self.revision.id,
                self.scope.semantics,
                self.runtime_policy,
                self.start,
                package.initial_state_views()[0]
                    .canonical_state_snapshot
                    .to_vec(),
            ))
            .unwrap();
        self.authority
            .establish_boundary(BoundaryAnchor {
                boundary: self.boundary,
                permissions: vec![BoundaryOccurrencePermissionV2 {
                    id: BoundaryPermissionLocalId::new(1),
                    kind: EnteredOccurrenceKind::AdmissionDecision,
                    payload: self.target.clone(),
                    pins: BoundaryPins {
                        semantics: self.scope.semantics,
                        snapshot: self.revision.preimage.snapshot,
                        constitution: CheckedConstitutionBinding::Admitted {
                            revision: self.revision.id,
                        },
                        runtime_session: Some(self.session),
                        observed_state: Some(self.initial),
                        runtime_policy: Some(self.runtime_policy),
                    },
                    cause_schema: vec![
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::CandidateDelta,
                            cardinality: CardinalityV2 {
                                minimum: 1,
                                maximum: Some(1),
                            },
                        },
                        BoundaryCauseRequirementV2 {
                            kind: EnteredCauseKindV2::Judgment,
                            cardinality: CardinalityV2 {
                                minimum: 1,
                                maximum: Some(1),
                            },
                        },
                    ],
                    support_schema: vec![],
                    replay: BoundaryReplayPolicyV2::Repeatable {
                        maximum_occurrences: Some(1),
                    },
                }],
            })
            .unwrap();
        self.authority
            .establish_evidence(EvidenceAnchor {
                evidence: self.evidence,
                boundary: self.boundary,
                permissions: vec![BoundaryPermissionLocalId::new(1)],
                exact_evidence: b"isolated test owner authorizes this exact Admission boundary"
                    .to_vec()
                    .into_boxed_slice(),
            })
            .unwrap();
        ProcessCarrier::instantiate(package, &self.authority).unwrap()
    }

    pub fn atom(&self, payload: &[u8]) -> Term {
        Term::atom(
            self.scope,
            b"compiler-language/bytes".to_vec(),
            payload.to_vec(),
            EqualityContract::ExactOctetsV1,
        )
        .unwrap()
    }

    pub fn judgment_authority(&self) -> JudgmentAuthorityEvidence {
        JudgmentAuthorityEvidence::IrreducibleRoot {
            policy: self.policy,
            authority: RootJudgmentAuthorityRef {
                policy: self.policy,
                local: JudgmentAuthorityLocalId::new(1),
            },
        }
    }

    pub fn activation(
        &self,
        carrier: &ProcessCarrier,
        id: u32,
        run: u32,
        mode: u32,
        origin: ActivationOrigin,
        initial: &[u8],
    ) -> ActivationProposalV2 {
        let application = ApplicationId {
            snapshot: self.revision.preimage.snapshot,
            local: ApplicationLocalId::new(1),
        };
        let mode = ModeId {
            operator: OperatorRef {
                snapshot: self.revision.preimage.snapshot,
                local: OperatorLocalId::new(1),
            },
            local: ModeLocalId::new(mode),
        };
        let executable = carrier
            .constitution()
            .executable_contract(application, mode)
            .unwrap();
        let declared = carrier.constitution().mode_by_id(mode).unwrap();
        ActivationProposalV2 {
            id: ActivationId::from_bytes(nominal(id)),
            application,
            mode,
            pins: ActivationPins {
                semantics: self.scope.semantics,
                snapshot: self.revision.preimage.snapshot,
                constitution: CheckedConstitutionBinding::Admitted {
                    revision: self.revision.id,
                },
                runtime_session: Some(self.session),
                observed_state: Some(self.initial),
                runtime_policy: Some(self.runtime_policy),
                context_requirements: executable.application_context_requirements,
                constitutive_dependencies: executable.application_dependency_closure,
                capabilities: declared
                    .contract
                    .capability_requirements
                    .iter()
                    .map(|local| CapabilityRef {
                        snapshot: self.revision.preimage.snapshot,
                        local: *local,
                    })
                    .collect(),
                scheduling_requirements: vec![],
                resource_requirements: vec![],
                cancellation_scope: CancellationScope::Activation,
                budget: Budget {
                    remaining_units: 100,
                },
            },
            static_basis: ActivationStaticBasis {
                execution_authorizations: vec![],
                judgment_authorities: vec![self.judgment_authority()],
            },
            prerequisite_bindings: vec![],
            causes: ActivationCauseFrontierV2 {
                origin,
                prerequisite_occurrences: vec![],
            },
            membership: match origin {
                ActivationOrigin::RootedBy(_) => {
                    RunMembership::RootOf(RunId::from_bytes(nominal(run)))
                }
                _ => RunMembership::ChildIn(RunId::from_bytes(nominal(run))),
            },
            initial_configuration: ConfigurationProposal {
                id: ConfigurationId::from_bytes(nominal(id + 1000)),
                value: self.atom(initial),
            },
        }
    }

    pub fn step(
        &self,
        carrier: &ProcessCarrier,
        id: u32,
        activation: u32,
        before_budget: u64,
        after: &[u8],
        mut causes: Vec<StepCause>,
    ) -> StepProposalV2 {
        let activation = carrier
            .activation(ActivationId::from_bytes(nominal(activation)))
            .unwrap();
        causes.sort();
        StepProposalV2 {
            id: StepId::from_bytes(nominal(id)),
            run: activation.membership().run(),
            activation: activation.id(),
            before: activation.latest_configuration(),
            after: ConfigurationProposal {
                id: ConfigurationId::from_bytes(nominal(id + 2000)),
                value: self.atom(after),
            },
            observed_state: Some(self.initial),
            budget: StepBudgetTransitionV2 {
                before: Budget {
                    remaining_units: before_budget,
                },
                consumed_units: 1,
                after: Budget {
                    remaining_units: before_budget - 1,
                },
            },
            causes,
            observation_outcomes: vec![],
            candidate_delta: None,
            outcome: StepOutcomeProposalV2::Progress,
        }
    }

    pub fn value_observation(
        &self,
        id: u32,
        bytes: &[u8],
        source: SupportSource,
    ) -> StepObservationOutcomeV2 {
        StepObservationOutcomeV2::Observed(ObservationProposalV2::Value {
            id: ObservationId::from_bytes(nominal(id)),
            value: self.atom(bytes),
            supports: vec![SupportUse {
                slot: SupportSlotId::new(0),
                role: self.atom(b"result"),
                source,
            }],
        })
    }

    pub fn formation_observation(
        &self,
        id: u32,
        term: Term,
        source: SupportSource,
    ) -> StepObservationOutcomeV2 {
        StepObservationOutcomeV2::Observed(ObservationProposalV2::Formation {
            id: ObservationId::from_bytes(nominal(id)),
            subject: term,
            target: self.target.clone(),
            supports: vec![SupportUse {
                slot: SupportSlotId::new(0),
                role: self.atom(b"checked byte value"),
                source,
            }],
        })
    }

    pub fn admit(
        &self,
        carrier: &mut ProcessCarrier,
        producer: StepRef,
        payload: Term,
    ) -> StateRevisionId {
        let judgment_id = JudgmentOccurrenceId::from_bytes(nominal(710));
        carrier
            .apply_ingress(
                &[ProcessRecordV2::Judgment(JudgmentOccurrenceV2 {
                    body: JudgmentOccurrenceBodyV2 {
                        id: judgment_id,
                        judgment: AdmissionJudgment {
                            delta: self.delta,
                            session: self.session,
                            policy: self.runtime_policy,
                            claim: AdmissionJudgmentClaim::Verdict(AdmissionDisposition::Admit),
                        },
                        authority: self.judgment_authority(),
                        supports: vec![SupportUse {
                            slot: SupportSlotId::new(0),
                            role: self.atom(b"candidate checked"),
                            source: SupportSource::Step(producer),
                        }],
                    },
                    provenance: OccurrenceProvenance::ProducedBy(producer),
                })],
                &self.authority,
            )
            .unwrap();
        let mut successor = StateRevision {
            id: StateRevisionId::from_bytes([0; 32]),
            session: self.session,
            predecessor: Some(self.initial),
            cause: StateRevisionCause::Admission {
                occurrence: self.admission,
                run: producer.run,
                activation: producer.activation,
                step: producer.step,
            },
            canonical_state_snapshot: canonical_term_bytes(&payload).unwrap().into_boxed_slice(),
            payload,
            policy: self.runtime_policy,
            semantics: self.scope.semantics,
        };
        successor.id = successor.derived_id();
        let id = successor.id;
        carrier
            .apply_ingress(
                &[ProcessRecordV2::AdmissionDecision(
                    StateAdmissionDecisionV2 {
                        occurrence: self.admission,
                        delta: self.delta,
                        authorization: AdmissionAuthorizationEvidence::IrreducibleRoot {
                            policy: self.policy,
                            authorization: RootAdmissionAuthorizationRef {
                                policy: self.policy,
                                local: AdmissionAuthorizationLocalId::new(1),
                            },
                        },
                        evidence: vec![SupportUse {
                            slot: SupportSlotId::new(0),
                            role: self.atom(b"governed verdict"),
                            source: SupportSource::Judgment(judgment_id),
                        }],
                        verdict: judgment_id,
                        obligation_judgments: vec![],
                        provenance: EnteredThrough {
                            boundary: self.boundary,
                            evidence: self.evidence,
                            permission: BoundaryPermissionLocalId::new(1),
                            payload: self.atom(b"admit exact candidate"),
                            supports: vec![],
                            causes: vec![
                                CausalRef::CandidateDelta(self.delta),
                                CausalRef::Judgment(judgment_id),
                            ],
                        },
                        outcome: StateAdmissionOutcomeV2::Admit(successor),
                    },
                )],
                &self.authority,
            )
            .unwrap();
        id
    }
}
