//! Process-v2 occurrence provenance and admission-envelope primitives.
//!
//! These values are inert. The pure validators in this module establish only
//! bounded canonical shape and consistency among already-resolved inputs. A
//! process carrier must still resolve identities, authority, pins, causal
//! ancestry, and one-shot admission state before constituting any occurrence.

use std::fmt;

use crate::formation::{CardinalityV2, FormationTargetV2};
use crate::identity::*;
use crate::process::{
    ActivationPins, AdmissionAuthorizationEvidence, CancellationTarget, CheckedConstitutionBinding,
    ContinuationPins, DomainBoundTermV2, ExecutionAuthorizationEvidence, StateRevision,
    StateRevisionCause,
};
use crate::term::Term;

pub const MAX_BOUNDARY_PERMISSIONS: usize = 64;
pub const MAX_OCCURRENCE_CAUSES: usize = 1_000_000;
pub const MAX_EXECUTION_AUTHORIZATIONS: usize = 1_000_000;
pub const MAX_JUDGMENT_AUTHORITIES: usize = 1_000_000;
pub const MAX_SUPPORT_USES: usize = 1_000_000;
pub const MAX_CANDIDATE_OBLIGATIONS: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum JudgmentAuthorityEvidence {
    ProgramConstitution {
        revision: ProgramRevisionId,
        authority: JudgmentAuthorityRef,
    },
    IrreducibleRoot {
        policy: RootPolicyId,
        authority: RootJudgmentAuthorityRef,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EnteredOccurrenceKind {
    ExternalTrigger,
    Resumption,
    Handoff,
    Cancellation,
    Observation,
    Judgment,
    AdmissionDecision,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BoundaryPins {
    pub semantics: ClauseSemanticsId,
    pub snapshot: ProgramSnapshotId,
    pub constitution: CheckedConstitutionBinding,
    pub runtime_session: Option<RuntimeSessionId>,
    pub observed_state: Option<StateRevisionId>,
    pub runtime_policy: Option<RuntimePolicyId>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EnteredCauseKindV2 {
    SessionStart,
    ExternalTrigger,
    Resumption,
    Handoff,
    Cancellation,
    Step,
    Observation,
    CandidateDelta,
    Judgment,
    Admission,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BoundaryCauseRequirementV2 {
    pub kind: EnteredCauseKindV2,
    pub cardinality: CardinalityV2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BoundarySupportSourceKindV2 {
    SessionStart,
    ExternalTrigger,
    Resumption,
    Handoff,
    Cancellation,
    Step,
    Observation,
    Judgment,
    Admission,
}

/// One exact support slot allowed by an external occurrence contract.
/// Boundary support slots are occurrence-level evidence and remain distinct
/// from any supports carried by an Observation or Judgment payload.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BoundarySupportRequirementV2 {
    pub slot: SupportSlotId,
    pub role: Term,
    pub source: BoundarySupportSourceKindV2,
    pub cardinality: CardinalityV2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BoundaryReplayPolicyV2 {
    OneShot,
    Repeatable { maximum_occurrences: Option<u32> },
}

/// One exact occurrence contract constituted at an external boundary.
/// `payload` is the exact Formation target that the independently established
/// boundary evidence must prove for each entered payload Term.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryOccurrencePermissionV2 {
    pub id: BoundaryPermissionLocalId,
    pub kind: EnteredOccurrenceKind,
    pub payload: FormationTargetV2,
    pub pins: BoundaryPins,
    pub cause_schema: Vec<BoundaryCauseRequirementV2>,
    pub support_schema: Vec<BoundarySupportRequirementV2>,
    pub replay: BoundaryReplayPolicyV2,
}

/// An already-constituted ingress or governance boundary. Its presence is not
/// itself authorization for any occurrence admitted through it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstitutedBoundary {
    pub id: BoundaryRef,
    pub permissions: Vec<BoundaryOccurrencePermissionV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryEvidence {
    pub id: ExternalEvidenceRef,
    pub boundary: BoundaryRef,
    pub permission: BoundaryPermissionLocalId,
    pub payload: Term,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StepRef {
    pub run: RunId,
    pub activation: ActivationId,
    pub step: StepId,
}

/// Typed causal references. Variant order is canonical wire order only; it
/// never supplies a semantic edge.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CausalRef {
    SessionStart(SessionStartOccurrenceId),
    ExternalTrigger(ExternalTriggerOccurrenceId),
    Resumption(ResumptionOccurrenceId),
    Handoff(HandoffOccurrenceId),
    Cancellation(CancellationOccurrenceId),
    Step(StepRef),
    Observation(ObservationId),
    CandidateDelta(CandidateDeltaId),
    Judgment(JudgmentOccurrenceId),
    Admission(AdmissionOccurrenceId),
}

impl CausalRef {
    #[must_use]
    pub const fn entered_kind(self) -> EnteredCauseKindV2 {
        match self {
            Self::SessionStart(_) => EnteredCauseKindV2::SessionStart,
            Self::ExternalTrigger(_) => EnteredCauseKindV2::ExternalTrigger,
            Self::Resumption(_) => EnteredCauseKindV2::Resumption,
            Self::Handoff(_) => EnteredCauseKindV2::Handoff,
            Self::Cancellation(_) => EnteredCauseKindV2::Cancellation,
            Self::Step(_) => EnteredCauseKindV2::Step,
            Self::Observation(_) => EnteredCauseKindV2::Observation,
            Self::CandidateDelta(_) => EnteredCauseKindV2::CandidateDelta,
            Self::Judgment(_) => EnteredCauseKindV2::Judgment,
            Self::Admission(_) => EnteredCauseKindV2::Admission,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EnteredThrough {
    pub boundary: BoundaryRef,
    pub evidence: ExternalEvidenceRef,
    pub permission: BoundaryPermissionLocalId,
    pub payload: Term,
    pub supports: Vec<SupportUse>,
    pub causes: Vec<CausalRef>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OccurrenceProvenance {
    ProducedBy(StepRef),
    EnteredThrough(EnteredThrough),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalTriggerOccurrenceV2 {
    pub id: ExternalTriggerOccurrenceId,
    pub provenance: EnteredThrough,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResumptionOccurrenceBodyV2 {
    pub id: ResumptionOccurrenceId,
    pub continuation: ContinuationId,
    pub run: RunId,
    pub activation: ActivationId,
    pub pins: ContinuationPins,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResumptionOccurrenceV2 {
    pub body: ResumptionOccurrenceBodyV2,
    pub provenance: OccurrenceProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffOccurrenceBodyV2 {
    pub id: HandoffOccurrenceId,
    pub continuation: ContinuationId,
    pub run: RunId,
    pub activation: ActivationId,
    pub pins: ContinuationPins,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffOccurrenceV2 {
    pub body: HandoffOccurrenceBodyV2,
    pub provenance: OccurrenceProvenance,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CancellationOccurrenceBodyV2 {
    pub id: CancellationOccurrenceId,
    pub target: CancellationTarget,
    pub pins: ActivationPins,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancellationOccurrenceV2 {
    pub body: CancellationOccurrenceBodyV2,
    pub provenance: OccurrenceProvenance,
}

/// One typed use of static authorization evidence. `kind` selects the exact
/// Mode-declared requirement it satisfies; neither field is a causal edge.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutionAuthorizationUseV2 {
    pub kind: FormationRefV2,
    pub evidence: ExecutionAuthorizationEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationStaticBasis {
    /// A bounded canonical set. Empty is valid when the selected Mode declares
    /// no execution-authorization requirement.
    pub execution_authorizations: Vec<ExecutionAuthorizationUseV2>,
    pub judgment_authorities: Vec<JudgmentAuthorityEvidence>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationPrerequisite {
    Observation(ObservationId),
    Admission(AdmissionOccurrenceId),
}

impl ActivationPrerequisite {
    #[must_use]
    pub const fn kind(self) -> ActivationPrerequisiteKind {
        match self {
            Self::Observation(_) => ActivationPrerequisiteKind::Observation,
            Self::Admission(_) => ActivationPrerequisiteKind::Admission,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationPrerequisiteKind {
    Observation,
    Admission,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PrerequisiteScope {
    SameSemantics,
    SameProgramRevision,
    SameRuntimeSession,
    SameObservedState,
}

/// The typed path from one bound prerequisite value to one occurrence cause.
/// The current admitted-stateful subset carries direct Observation and
/// Admission occurrences, so it has exactly this one path form.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PrerequisiteOccurrencePathV2 {
    BoundOccurrence,
}

/// One named component of a Mode-owned cause-projection schema.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CauseProjectionEntryV2 {
    pub component: CauseComponentLocalId,
    pub path: PrerequisiteOccurrencePathV2,
}

/// One exact value closing one stable prerequisite slot and repeated-value
/// ordinal. Equal values in different slots or ordinals remain distinct.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DynamicPrerequisiteBindingV2 {
    pub slot: PrerequisiteSlotId,
    pub ordinal: u32,
    pub value: ActivationPrerequisite,
}

/// One occurrence-only cause mechanically projected from a prerequisite
/// binding according to the selected Mode's projection schema.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActivationOccurrenceCauseV2 {
    pub slot: PrerequisiteSlotId,
    pub ordinal: u32,
    pub component: CauseComponentLocalId,
    pub occurrence: ActivationPrerequisite,
}

/// Occurrence-exact evidence sources. Candidate deltas are excluded: a
/// support names an actual occurrence or Step, not candidate content.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SupportSource {
    SessionStart(SessionStartOccurrenceId),
    ExternalTrigger(ExternalTriggerOccurrenceId),
    Resumption(ResumptionOccurrenceId),
    Handoff(HandoffOccurrenceId),
    Cancellation(CancellationOccurrenceId),
    Step(StepRef),
    Observation(ObservationId),
    Judgment(JudgmentOccurrenceId),
    Admission(AdmissionOccurrenceId),
}

impl SupportSource {
    #[must_use]
    pub const fn boundary_kind(self) -> BoundarySupportSourceKindV2 {
        match self {
            Self::SessionStart(_) => BoundarySupportSourceKindV2::SessionStart,
            Self::ExternalTrigger(_) => BoundarySupportSourceKindV2::ExternalTrigger,
            Self::Resumption(_) => BoundarySupportSourceKindV2::Resumption,
            Self::Handoff(_) => BoundarySupportSourceKindV2::Handoff,
            Self::Cancellation(_) => BoundarySupportSourceKindV2::Cancellation,
            Self::Step(_) => BoundarySupportSourceKindV2::Step,
            Self::Observation(_) => BoundarySupportSourceKindV2::Observation,
            Self::Judgment(_) => BoundarySupportSourceKindV2::Judgment,
            Self::Admission(_) => BoundarySupportSourceKindV2::Admission,
        }
    }
}

/// Slots preserve dependency multiplicity: two slots may cite one source,
/// while a slot itself may occur only once in one support frontier.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SupportUse {
    pub slot: SupportSlotId,
    pub role: Term,
    pub source: SupportSource,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CandidateObligation {
    pub id: ObligationId,
    pub requirement: Term,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateDeltaV2 {
    pub id: CandidateDeltaId,
    pub base: StateRevisionId,
    pub delta: DomainBoundTermV2,
    pub proposed_payload: Term,
    pub evidence: Vec<SupportUse>,
    pub obligations: Vec<CandidateObligation>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AdmissionDisposition {
    Admit,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ObligationStatus {
    Satisfied,
    Unsatisfied,
    Deferred,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AdmissionJudgmentClaim {
    Verdict(AdmissionDisposition),
    Obligation {
        obligation: ObligationId,
        status: ObligationStatus,
    },
}

/// Immutable judgment content. Issuing equal content twice still creates two
/// independently identified JudgmentOccurrenceV2 values.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AdmissionJudgment {
    pub delta: CandidateDeltaId,
    pub session: RuntimeSessionId,
    pub policy: RuntimePolicyId,
    pub claim: AdmissionJudgmentClaim,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JudgmentOccurrenceBodyV2 {
    pub id: JudgmentOccurrenceId,
    pub judgment: AdmissionJudgment,
    pub authority: JudgmentAuthorityEvidence,
    pub supports: Vec<SupportUse>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JudgmentOccurrenceV2 {
    pub body: JudgmentOccurrenceBodyV2,
    pub provenance: OccurrenceProvenance,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObligationJudgmentUse {
    pub obligation: ObligationId,
    pub judgment: JudgmentOccurrenceId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionRejectionV2 {
    pub reason: Term,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "an admission decision owns its exact successor revision without hidden allocation"
)]
pub enum StateAdmissionOutcomeV2 {
    Admit(StateRevision),
    Reject(AdmissionRejectionV2),
}

impl StateAdmissionOutcomeV2 {
    #[must_use]
    pub const fn disposition(&self) -> AdmissionDisposition {
        match self {
            Self::Admit(_) => AdmissionDisposition::Admit,
            Self::Reject(_) => AdmissionDisposition::Reject,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateAdmissionDecisionV2 {
    pub occurrence: AdmissionOccurrenceId,
    pub delta: CandidateDeltaId,
    pub authorization: AdmissionAuthorizationEvidence,
    pub evidence: Vec<SupportUse>,
    pub verdict: JudgmentOccurrenceId,
    pub obligation_judgments: Vec<ObligationJudgmentUse>,
    pub provenance: EnteredThrough,
    pub outcome: StateAdmissionOutcomeV2,
}

/// Resolved carrier facts supplied to the pure decision-input validator.
pub struct AdmissionDecisionContext<'a> {
    pub base: &'a StateRevision,
    pub producer: StepRef,
    pub prior_decision: Option<AdmissionOccurrenceId>,
}

pub fn validate_boundary(boundary: &ConstitutedBoundary) -> Result<(), ProvenanceError> {
    validate_length(
        "boundary permissions",
        boundary.permissions.len(),
        MAX_BOUNDARY_PERMISSIONS,
    )?;
    if !boundary
        .permissions
        .windows(2)
        .all(|pair| pair[0].id < pair[1].id)
    {
        return Err(ProvenanceError::NonCanonicalSet("boundary permissions"));
    }
    for permission in &boundary.permissions {
        if permission.payload.type_term.scope().semantics != permission.pins.semantics
            || permission.payload.interpretation.scope().semantics != permission.pins.semantics
        {
            return Err(ProvenanceError::BoundaryPayloadSnapshotMismatch);
        }
        if permission.pins.runtime_session.is_some() != permission.pins.runtime_policy.is_some() {
            return Err(ProvenanceError::IncompleteBoundaryRuntimePins);
        }
        validate_sorted_unique_by(
            &permission.cause_schema,
            "boundary cause schema",
            |requirement| requirement.kind,
        )?;
        validate_sorted_unique_by(
            &permission.support_schema,
            "boundary support schema",
            |requirement| requirement.slot,
        )?;
        for requirement in &permission.cause_schema {
            validate_cardinality(requirement.cardinality)?;
        }
        for requirement in &permission.support_schema {
            validate_cardinality(requirement.cardinality)?;
            if requirement
                .cardinality
                .maximum
                .is_none_or(|maximum| maximum > 1)
            {
                return Err(ProvenanceError::BoundarySupportCardinalityTooLarge(
                    requirement.slot,
                ));
            }
        }
        if matches!(
            permission.replay,
            BoundaryReplayPolicyV2::Repeatable {
                maximum_occurrences: Some(0)
            }
        ) {
            return Err(ProvenanceError::EmptyBoundaryReplayContract);
        }
    }
    Ok(())
}

pub fn validate_boundary_evidence(
    boundary: &ConstitutedBoundary,
    evidence: &BoundaryEvidence,
) -> Result<(), ProvenanceError> {
    if evidence.boundary != boundary.id {
        return Err(ProvenanceError::BoundaryEvidenceMismatch);
    }
    let permission = boundary
        .permissions
        .iter()
        .find(|permission| permission.id == evidence.permission)
        .ok_or(ProvenanceError::BoundaryEvidenceMismatch)?;
    if evidence.payload.scope().semantics != permission.pins.semantics {
        return Err(ProvenanceError::BoundaryEvidenceMismatch);
    }
    Ok(())
}

pub fn validate_entered_through(provenance: &EnteredThrough) -> Result<(), ProvenanceError> {
    validate_length(
        "occurrence cause frontier",
        provenance.causes.len(),
        MAX_OCCURRENCE_CAUSES,
    )?;
    validate_sorted_unique(&provenance.causes, "occurrence cause frontier")?;
    validate_support_uses(&provenance.supports)
}

fn validate_cardinality(cardinality: CardinalityV2) -> Result<(), ProvenanceError> {
    if cardinality
        .maximum
        .is_some_and(|maximum| cardinality.minimum > maximum)
    {
        return Err(ProvenanceError::InvalidBoundaryCardinality);
    }
    Ok(())
}

fn validate_sorted_unique_by<T, K: Ord>(
    values: &[T],
    label: &'static str,
    key: impl Fn(&T) -> K,
) -> Result<(), ProvenanceError> {
    if values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1])) {
        Ok(())
    } else {
        Err(ProvenanceError::NonCanonicalSet(label))
    }
}

pub fn validate_occurrence_provenance(
    provenance: &OccurrenceProvenance,
) -> Result<(), ProvenanceError> {
    match provenance {
        OccurrenceProvenance::ProducedBy(_) => Ok(()),
        OccurrenceProvenance::EnteredThrough(entered) => validate_entered_through(entered),
    }
}

pub fn validate_activation_static_basis(
    basis: &ActivationStaticBasis,
) -> Result<(), ProvenanceError> {
    validate_length(
        "execution authorizations",
        basis.execution_authorizations.len(),
        MAX_EXECUTION_AUTHORIZATIONS,
    )?;
    validate_sorted_unique(&basis.execution_authorizations, "execution authorizations")?;
    validate_length(
        "judgment authorities",
        basis.judgment_authorities.len(),
        MAX_JUDGMENT_AUTHORITIES,
    )?;
    validate_sorted_unique(&basis.judgment_authorities, "judgment authorities")
}

pub fn validate_support_uses(supports: &[SupportUse]) -> Result<(), ProvenanceError> {
    validate_length("support uses", supports.len(), MAX_SUPPORT_USES)?;
    for pair in supports.windows(2) {
        if pair[0].slot == pair[1].slot {
            return Err(ProvenanceError::DuplicateSupportSlot(pair[0].slot));
        }
        if pair[0].slot > pair[1].slot {
            return Err(ProvenanceError::NonCanonicalOrder("support uses"));
        }
    }
    Ok(())
}

pub fn validate_candidate_delta(candidate: &CandidateDeltaV2) -> Result<(), ProvenanceError> {
    validate_support_uses(&candidate.evidence)?;
    validate_length(
        "candidate obligations",
        candidate.obligations.len(),
        MAX_CANDIDATE_OBLIGATIONS,
    )?;
    for pair in candidate.obligations.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(ProvenanceError::DuplicateObligation(pair[0].id));
        }
        if pair[0].id > pair[1].id {
            return Err(ProvenanceError::NonCanonicalOrder("candidate obligations"));
        }
    }
    for obligation in &candidate.obligations {
        if obligation.id.delta != candidate.id {
            return Err(ProvenanceError::ObligationDeltaMismatch {
                obligation: obligation.id,
                expected: candidate.id,
            });
        }
    }
    Ok(())
}

pub fn validate_judgment_occurrence(
    occurrence: &JudgmentOccurrenceV2,
) -> Result<(), ProvenanceError> {
    validate_support_uses(&occurrence.body.supports)?;
    validate_occurrence_provenance(&occurrence.provenance)?;
    if let AdmissionJudgmentClaim::Obligation { obligation, .. } = occurrence.body.judgment.claim
        && obligation.delta != occurrence.body.judgment.delta
    {
        return Err(ProvenanceError::JudgmentObligationDeltaMismatch {
            judgment: occurrence.body.id,
            obligation,
            expected: occurrence.body.judgment.delta,
        });
    }
    Ok(())
}

pub fn validate_state_admission_decision_inputs(
    candidate: &CandidateDeltaV2,
    decision: &StateAdmissionDecisionV2,
    verdict: &JudgmentOccurrenceV2,
    obligation_judgments: &[JudgmentOccurrenceV2],
    context: AdmissionDecisionContext<'_>,
) -> Result<(), ProvenanceError> {
    if let Some(prior) = context.prior_decision {
        return Err(ProvenanceError::CandidateAlreadyDecided {
            delta: candidate.id,
            prior,
        });
    }
    validate_candidate_delta(candidate)?;
    validate_support_uses(&decision.evidence)?;
    validate_entered_through(&decision.provenance)?;
    if decision.delta != candidate.id {
        return Err(ProvenanceError::DecisionDeltaMismatch {
            decision: decision.delta,
            candidate: candidate.id,
        });
    }
    if candidate.base != context.base.id {
        return Err(ProvenanceError::CandidateBaseMismatch {
            candidate: candidate.base,
            resolved: context.base.id,
        });
    }

    validate_judgment_occurrence(verdict)?;
    if decision.verdict != verdict.body.id {
        return Err(ProvenanceError::AdmissionVerdictIdMismatch {
            expected: decision.verdict,
            actual: verdict.body.id,
        });
    }
    validate_judgment_context(candidate.id, verdict, context.base)?;
    let claimed_disposition = match verdict.body.judgment.claim {
        AdmissionJudgmentClaim::Verdict(disposition) => disposition,
        AdmissionJudgmentClaim::Obligation { .. } => {
            return Err(ProvenanceError::AdmissionVerdictClaimExpected(
                verdict.body.id,
            ));
        }
    };
    let actual_disposition = decision.outcome.disposition();
    if claimed_disposition != actual_disposition {
        return Err(ProvenanceError::AdmissionVerdictMismatch {
            claimed: claimed_disposition,
            actual: actual_disposition,
        });
    }

    validate_length(
        "obligation judgment uses",
        decision.obligation_judgments.len(),
        MAX_CANDIDATE_OBLIGATIONS,
    )?;
    validate_strict_key_order(
        &decision.obligation_judgments,
        "obligation judgment uses",
        |entry| entry.obligation,
    )?;
    if decision.obligation_judgments.len() != candidate.obligations.len()
        || obligation_judgments.len() != candidate.obligations.len()
    {
        return Err(ProvenanceError::ObligationJudgmentCountMismatch {
            obligations: candidate.obligations.len(),
            uses: decision.obligation_judgments.len(),
            resolved: obligation_judgments.len(),
        });
    }

    for ((obligation, usage), judgment) in candidate
        .obligations
        .iter()
        .zip(&decision.obligation_judgments)
        .zip(obligation_judgments)
    {
        if usage.obligation != obligation.id {
            return Err(ProvenanceError::ObligationSetMismatch {
                expected: obligation.id,
                actual: usage.obligation,
            });
        }
        if usage.judgment != judgment.body.id {
            return Err(ProvenanceError::ObligationJudgmentIdMismatch {
                obligation: obligation.id,
                expected: usage.judgment,
                actual: judgment.body.id,
            });
        }
        validate_judgment_occurrence(judgment)?;
        validate_judgment_context(candidate.id, judgment, context.base)?;
        let (judged_obligation, status) = match judgment.body.judgment.claim {
            AdmissionJudgmentClaim::Obligation { obligation, status } => (obligation, status),
            AdmissionJudgmentClaim::Verdict(_) => {
                return Err(ProvenanceError::ObligationJudgmentClaimExpected(
                    judgment.body.id,
                ));
            }
        };
        if judged_obligation != obligation.id {
            return Err(ProvenanceError::ObligationSetMismatch {
                expected: obligation.id,
                actual: judged_obligation,
            });
        }
        if actual_disposition == AdmissionDisposition::Admit
            && status != ObligationStatus::Satisfied
        {
            return Err(ProvenanceError::UnsatisfiedAdmissionObligation {
                obligation: obligation.id,
                status,
            });
        }
    }

    if let StateAdmissionOutcomeV2::Admit(successor) = &decision.outcome {
        let expected_cause = StateRevisionCause::Admission {
            occurrence: decision.occurrence,
            run: context.producer.run,
            activation: context.producer.activation,
            step: context.producer.step,
        };
        if successor.session != context.base.session
            || successor.predecessor != Some(context.base.id)
            || successor.cause != expected_cause
            || successor.payload != candidate.proposed_payload
            || successor.policy != context.base.policy
            || successor.semantics != context.base.semantics
        {
            return Err(ProvenanceError::SuccessorStateMismatch);
        }
    }

    Ok(())
}

fn validate_judgment_context(
    candidate: CandidateDeltaId,
    judgment: &JudgmentOccurrenceV2,
    base: &StateRevision,
) -> Result<(), ProvenanceError> {
    if judgment.body.judgment.delta != candidate {
        return Err(ProvenanceError::JudgmentDeltaMismatch {
            judgment: judgment.body.id,
            expected: candidate,
            actual: judgment.body.judgment.delta,
        });
    }
    if judgment.body.judgment.session != base.session
        || judgment.body.judgment.policy != base.policy
    {
        return Err(ProvenanceError::JudgmentContextMismatch(judgment.body.id));
    }
    Ok(())
}

fn validate_length(
    field: &'static str,
    count: usize,
    maximum: usize,
) -> Result<(), ProvenanceError> {
    if count > maximum {
        Err(ProvenanceError::FrontierTooLarge {
            field,
            count,
            maximum,
        })
    } else {
        Ok(())
    }
}

fn validate_sorted_unique<T: Ord>(
    values: &[T],
    field: &'static str,
) -> Result<(), ProvenanceError> {
    if values.windows(2).all(|pair| pair[0] < pair[1]) {
        Ok(())
    } else {
        Err(ProvenanceError::NonCanonicalOrder(field))
    }
}

fn validate_strict_key_order<T, K: Ord>(
    values: &[T],
    field: &'static str,
    key: impl Fn(&T) -> K,
) -> Result<(), ProvenanceError> {
    if values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1])) {
        Ok(())
    } else {
        Err(ProvenanceError::NonCanonicalOrder(field))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvenanceError {
    FrontierTooLarge {
        field: &'static str,
        count: usize,
        maximum: usize,
    },
    NonCanonicalOrder(&'static str),
    NonCanonicalSet(&'static str),
    IncompleteBoundaryRuntimePins,
    BoundaryPayloadSnapshotMismatch,
    InvalidBoundaryCardinality,
    BoundarySupportCardinalityTooLarge(SupportSlotId),
    EmptyBoundaryReplayContract,
    BoundaryEvidenceMismatch,
    DuplicateSupportSlot(SupportSlotId),
    DuplicateObligation(ObligationId),
    ObligationDeltaMismatch {
        obligation: ObligationId,
        expected: CandidateDeltaId,
    },
    JudgmentObligationDeltaMismatch {
        judgment: JudgmentOccurrenceId,
        obligation: ObligationId,
        expected: CandidateDeltaId,
    },
    CandidateAlreadyDecided {
        delta: CandidateDeltaId,
        prior: AdmissionOccurrenceId,
    },
    DecisionDeltaMismatch {
        decision: CandidateDeltaId,
        candidate: CandidateDeltaId,
    },
    CandidateBaseMismatch {
        candidate: StateRevisionId,
        resolved: StateRevisionId,
    },
    AdmissionVerdictIdMismatch {
        expected: JudgmentOccurrenceId,
        actual: JudgmentOccurrenceId,
    },
    JudgmentDeltaMismatch {
        judgment: JudgmentOccurrenceId,
        expected: CandidateDeltaId,
        actual: CandidateDeltaId,
    },
    JudgmentContextMismatch(JudgmentOccurrenceId),
    AdmissionVerdictClaimExpected(JudgmentOccurrenceId),
    AdmissionVerdictMismatch {
        claimed: AdmissionDisposition,
        actual: AdmissionDisposition,
    },
    ObligationJudgmentCountMismatch {
        obligations: usize,
        uses: usize,
        resolved: usize,
    },
    ObligationSetMismatch {
        expected: ObligationId,
        actual: ObligationId,
    },
    ObligationJudgmentIdMismatch {
        obligation: ObligationId,
        expected: JudgmentOccurrenceId,
        actual: JudgmentOccurrenceId,
    },
    ObligationJudgmentClaimExpected(JudgmentOccurrenceId),
    UnsatisfiedAdmissionObligation {
        obligation: ObligationId,
        status: ObligationStatus,
    },
    SuccessorStateMismatch,
}

impl fmt::Display for ProvenanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ProvenanceError {}
