//! Exact-byte bridge between compiler-machine checking and outer Program
//! Admission. Checking alone never constructs [`AdmittedCompiler`].

use std::fmt;

use clause_package::{
    AdmissionAuthorizationRef, AdmissionOccurrenceId, AuthorityError, AuthorityStore,
    CandidateDeltaId, CheckedProcessPackage, FormationLocalId, ObservationId, ProcessCarrier,
    ProgramRevisionClaim, ProgramRevisionId, RoleBindingValuePreimageV2, RoleLocalId,
    RootAdmissionAuthorizationRef, RootPolicyId, RunId, StateAdmissionOutcomeV2, StepCause, StepId,
};

use crate::compiler_package_v3::checker::AcceptedExact;
use crate::compiler_package_v3::{
    AuthorizationCheckError, AuthorizationFailure, AuthorizationVerdict,
    GenesisAuthorizationRequest, PredecessorInput, SuccessorAuthorizationRequest,
    authorize_genesis, authorize_successor,
};

#[derive(Debug)]
pub enum CompilerAdmissionError {
    Check(AuthorizationCheckError),
    Rejected(AuthorizationFailure),
    ExactBytesMismatch,
    FormationMissing,
    FormationBytesMismatch,
    RevisionMismatch,
    Authority(AuthorityError),
    EvolutionRunMismatch,
    EvolutionEvidenceMismatch,
}

impl fmt::Display for CompilerAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Check(error) => write!(formatter, "compiler check: {error}"),
            Self::Rejected(failure) => write!(formatter, "compiler rejected: {failure:?}"),
            Self::ExactBytesMismatch => formatter.write_str("checked compiler bytes differ"),
            Self::FormationMissing => formatter.write_str("compiler formation is absent"),
            Self::FormationBytesMismatch => formatter.write_str("compiler formation bytes differ"),
            Self::RevisionMismatch => formatter.write_str("compiler Program revision differs"),
            Self::Authority(error) => write!(formatter, "outer Program Admission: {error}"),
            Self::EvolutionRunMismatch => formatter.write_str("compiler evolution Run differs"),
            Self::EvolutionEvidenceMismatch => {
                formatter.write_str("compiler evolution evidence differs")
            }
        }
    }
}

/// Explicit occurrence bindings for the Clause-owned compiler evolution.
/// The carrier has already checked causality, typing, governed Judgment, and
/// the exact Admission decision. These references are not inferred from names.
pub struct CompilerEvolutionRun<'a> {
    pub carrier: &'a ProcessCarrier,
    pub run: RunId,
    pub compiler_role: RoleLocalId,
    pub compile_step: StepId,
    pub compile_observations: ObservationId,
    pub compile_remaining_fuel: ObservationId,
    pub proposal_step: StepId,
    pub proposal_observations: ObservationId,
    pub proposal_remaining_fuel: ObservationId,
    pub checker_step: StepId,
    pub candidate_step: StepId,
    pub candidate_delta: CandidateDeltaId,
    pub admission: AdmissionOccurrenceId,
}

impl CompilerEvolutionRun<'_> {
    fn check(
        &self,
        predecessor: &AdmittedCompiler<'_>,
        candidate: &CompilerProgramCandidate<'_>,
        request: &SuccessorAuthorizationRequest<'_>,
    ) -> Result<(), CompilerAdmissionError> {
        use crate::compiler_package_v3::{CompilerEvidence, domain_hash, encode_canonical_term};
        let root_id = self
            .carrier
            .run_root(self.run)
            .ok_or(CompilerAdmissionError::EvolutionRunMismatch)?;
        let root = self
            .carrier
            .activation(*root_id)
            .ok_or(CompilerAdmissionError::EvolutionRunMismatch)?;
        if root.pins().constitution.admitted_revision() != Some(predecessor.revision) {
            return Err(CompilerAdmissionError::EvolutionRunMismatch);
        }
        let application = self
            .carrier
            .constitution()
            .application_by_id(root.proposal().application)
            .ok_or(CompilerAdmissionError::EvolutionRunMismatch)?;
        let binding = application
            .form
            .bindings
            .iter()
            .find(|b| b.role == self.compiler_role && b.occurrence == 0)
            .ok_or(CompilerAdmissionError::EvolutionRunMismatch)?;
        let RoleBindingValuePreimageV2::Known(formation) = binding.value else {
            return Err(CompilerAdmissionError::EvolutionRunMismatch);
        };
        let value = &self
            .carrier
            .constitution()
            .formation(formation)
            .ok_or(CompilerAdmissionError::EvolutionRunMismatch)?
            .term;
        if value.as_atom().map(|a| a.canonical_payload()) != Some(predecessor.exact_bytes) {
            return Err(CompilerAdmissionError::EvolutionRunMismatch);
        }
        let exact_request = encode_canonical_term(request.build_request)
            .map_err(|_| CompilerAdmissionError::EvolutionEvidenceMismatch)?;
        if root
            .proposal()
            .initial_configuration
            .value
            .as_atom()
            .map(|a| a.canonical_payload())
            != Some(exact_request.as_slice())
        {
            return Err(CompilerAdmissionError::EvolutionEvidenceMismatch);
        }
        let ids = [
            self.compile_step,
            self.proposal_step,
            self.checker_step,
            self.candidate_step,
        ];
        let mut steps = Vec::with_capacity(ids.len());
        for id in ids {
            let step = self
                .carrier
                .step(id)
                .ok_or(CompilerAdmissionError::EvolutionRunMismatch)?;
            if step.proposal().run != self.run
                || steps
                    .iter()
                    .any(|prior: &&clause_package::Step| prior.reference() == step.reference())
            {
                return Err(CompilerAdmissionError::EvolutionRunMismatch);
            }
            steps.push(step);
        }
        for pair in steps.windows(2) {
            let causal = self
                .carrier
                .causal_predecessors(clause_package::CausalRef::Step(pair[1].reference()))
                .is_some_and(|causes| {
                    causes.contains(&clause_package::CausalRef::Step(pair[0].reference()))
                });
            if !causal
                && !pair[1]
                    .proposal()
                    .causes
                    .contains(&StepCause::PriorStep(pair[0].reference()))
            {
                return Err(CompilerAdmissionError::EvolutionRunMismatch);
            }
        }
        let CompilerEvidence::Successor {
            compile_receipt,
            admission_receipt,
        } = request.evidence
        else {
            return Err(CompilerAdmissionError::EvolutionEvidenceMismatch);
        };
        for (step, receipt) in [(&steps[0], compile_receipt), (&steps[1], admission_receipt)] {
            let bytes = step
                .proposal()
                .after
                .value
                .as_atom()
                .ok_or(CompilerAdmissionError::EvolutionEvidenceMismatch)?
                .canonical_payload();
            if domain_hash("clause/eval-receipt-value/v1", &[bytes]) != receipt.expected_value_hash
            {
                return Err(CompilerAdmissionError::EvolutionEvidenceMismatch);
            }
        }
        for (step, observations, fuel, receipt) in [
            (
                steps[0],
                self.compile_observations,
                self.compile_remaining_fuel,
                compile_receipt,
            ),
            (
                steps[1],
                self.proposal_observations,
                self.proposal_remaining_fuel,
                admission_receipt,
            ),
        ] {
            let observation_bytes = |id| {
                let observed = self
                    .carrier
                    .observation(id)
                    .ok_or(CompilerAdmissionError::EvolutionEvidenceMismatch)?;
                if observed.provenance
                    != clause_package::OccurrenceProvenance::ProducedBy(step.reference())
                {
                    return Err(CompilerAdmissionError::EvolutionEvidenceMismatch);
                }
                let clause_package::ObservationContentV2::Value { value, .. } = &observed.content
                else {
                    return Err(CompilerAdmissionError::EvolutionEvidenceMismatch);
                };
                value
                    .as_atom()
                    .map(|a| a.canonical_payload())
                    .ok_or(CompilerAdmissionError::EvolutionEvidenceMismatch)
            };
            if domain_hash(
                "clause/eval-receipt-observations/v1",
                &[observation_bytes(observations)?],
            ) != receipt.expected_observations_hash
                || observation_bytes(fuel)? != receipt.expected_remaining_fuel.to_be_bytes()
            {
                return Err(CompilerAdmissionError::EvolutionEvidenceMismatch);
            }
        }
        if steps[2]
            .proposal()
            .after
            .value
            .as_atom()
            .map(|a| a.canonical_payload())
            != Some(candidate.exact_compiler_bytes)
        {
            return Err(CompilerAdmissionError::EvolutionEvidenceMismatch);
        }
        let delta = self
            .carrier
            .candidate_delta(self.candidate_delta)
            .ok_or(CompilerAdmissionError::EvolutionEvidenceMismatch)?;
        if delta.produced_by != steps[3].reference()
            || delta
                .proposal
                .proposed_payload
                .as_atom()
                .map(|a| a.canonical_payload())
                != Some(candidate.checked_process.exact_bytes())
        {
            return Err(CompilerAdmissionError::EvolutionEvidenceMismatch);
        }
        let decision = self
            .carrier
            .decision_by_occurrence(self.admission)
            .ok_or(CompilerAdmissionError::EvolutionEvidenceMismatch)?;
        if decision.delta != self.candidate_delta
            || !matches!(decision.outcome, StateAdmissionOutcomeV2::Admit(_))
        {
            return Err(CompilerAdmissionError::EvolutionEvidenceMismatch);
        }
        Ok(())
    }
}

impl std::error::Error for CompilerAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Check(error) => Some(error),
            Self::Authority(error) => Some(error),
            _ => None,
        }
    }
}

/// One exact compiler selected by an actually admitted Program revision.
/// The borrowed bytes cannot be replaced while this capability exists.
#[derive(Debug)]
pub struct AdmittedCompiler<'a> {
    exact_bytes: &'a [u8],
    revision: ProgramRevisionId,
}

impl<'a> AdmittedCompiler<'a> {
    #[must_use]
    pub const fn exact_bytes(&self) -> &'a [u8] {
        self.exact_bytes
    }

    #[must_use]
    pub const fn revision(&self) -> ProgramRevisionId {
        self.revision
    }

    #[must_use]
    pub fn predecessor(&self, offered_bytes: &'a [u8]) -> PredecessorInput<'a> {
        PredecessorInput::Accepted {
            exact_bytes: self.exact_bytes,
            acceptance: AcceptedExact::from_outer_admission(self.exact_bytes),
            offered_bytes,
        }
    }
}

/// The caller selects the formation explicitly. No kind, semantic identifier,
/// source spelling, or package-local definition selects a host implementation.
pub struct CompilerProgramCandidate<'a> {
    pub exact_compiler_bytes: &'a [u8],
    pub checked_process: &'a CheckedProcessPackage,
    pub compiler_formation: FormationLocalId,
    pub revision: ProgramRevisionClaim,
}

impl CompilerProgramCandidate<'_> {
    fn check_binding(&self) -> Result<(), CompilerAdmissionError> {
        let constitution = self.checked_process.constitution();
        if self.revision.preimage.snapshot != constitution.snapshot()
            || self.revision.preimage.semantics != constitution.semantics()
        {
            return Err(CompilerAdmissionError::RevisionMismatch);
        }
        let formation = constitution
            .formation(self.compiler_formation)
            .ok_or(CompilerAdmissionError::FormationMissing)?;
        if formation
            .term
            .as_atom()
            .map(|atom| atom.canonical_payload())
            != Some(self.exact_compiler_bytes)
        {
            return Err(CompilerAdmissionError::FormationBytesMismatch);
        }
        Ok(())
    }
}

fn require_exact_verdict(
    exact_bytes: &[u8],
    verdict: AuthorizationVerdict,
) -> Result<(), CompilerAdmissionError> {
    match verdict {
        AuthorizationVerdict::Authorized(bytes) if bytes == exact_bytes => Ok(()),
        AuthorizationVerdict::Authorized(_) => Err(CompilerAdmissionError::ExactBytesMismatch),
        AuthorizationVerdict::Unauthorized(failure) => {
            Err(CompilerAdmissionError::Rejected(failure))
        }
    }
}

pub fn admit_genesis<'a>(
    authority: &mut AuthorityStore,
    candidate: CompilerProgramCandidate<'a>,
    request: GenesisAuthorizationRequest<'_>,
    policy: RootPolicyId,
    authorization: RootAdmissionAuthorizationRef,
) -> Result<AdmittedCompiler<'a>, CompilerAdmissionError> {
    candidate.check_binding()?;
    let verdict = authorize_genesis(candidate.exact_compiler_bytes, request)
        .map_err(CompilerAdmissionError::Check)?;
    require_exact_verdict(candidate.exact_compiler_bytes, verdict)?;
    authority
        .admit_genesis(
            candidate.revision,
            candidate.checked_process.authority_input(),
            policy,
            authorization,
        )
        .map_err(CompilerAdmissionError::Authority)?;
    Ok(AdmittedCompiler {
        exact_bytes: candidate.exact_compiler_bytes,
        revision: candidate.revision.id,
    })
}

pub fn admit_successor<'a>(
    authority: &mut AuthorityStore,
    predecessor: &AdmittedCompiler<'_>,
    candidate: CompilerProgramCandidate<'a>,
    request: SuccessorAuthorizationRequest<'_>,
    evolution: CompilerEvolutionRun<'_>,
    authorization: AdmissionAuthorizationRef,
) -> Result<AdmittedCompiler<'a>, CompilerAdmissionError> {
    candidate.check_binding()?;
    if candidate.revision.preimage.predecessor != Some(predecessor.revision) {
        return Err(CompilerAdmissionError::RevisionMismatch);
    }
    match &request.predecessor {
        PredecessorInput::Accepted { exact_bytes, .. }
            if *exact_bytes == predecessor.exact_bytes => {}
        _ => return Err(CompilerAdmissionError::ExactBytesMismatch),
    }
    evolution.check(predecessor, &candidate, &request)?;
    let verdict = authorize_successor(candidate.exact_compiler_bytes, request)
        .map_err(CompilerAdmissionError::Check)?;
    require_exact_verdict(candidate.exact_compiler_bytes, verdict)?;
    authority
        .admit_successor(
            candidate.revision,
            candidate.checked_process.authority_input(),
            authorization,
        )
        .map_err(CompilerAdmissionError::Authority)?;
    Ok(AdmittedCompiler {
        exact_bytes: candidate.exact_compiler_bytes,
        revision: candidate.revision.id,
    })
}
