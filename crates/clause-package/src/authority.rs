//! Separately established authority for process-v2 semantic boundaries.
//!
//! Checked package content is inert. It can contribute a checked snapshot
//! claim and declarations that become effective if that snapshot is admitted,
//! but it cannot create a root policy, an admitted revision, a runtime session,
//! or an external provenance anchor.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use crate::formation::ResolvedProgramConstitutionV2;
use crate::hash::{
    StateRevisionCausePreimage, derive_program_revision_id, derive_program_snapshot_id,
    derive_state_revision_id,
};
use crate::identity::{
    AdmissionAuthorizationRef, ApplicationId, BoundaryPermissionLocalId, BoundaryRef,
    CandidateDeltaId, ClauseSemanticsId, ExecutionAuthorizationRef, ExternalEvidenceRef,
    FormationRefV2, JudgmentAuthorityRef, ModeId, ProcessPackageId, ProgramChangeOccurrenceId,
    ProgramId, ProgramRevisionId, ProgramSnapshotId, RootAdmissionAuthorizationIssuerRef,
    RootAdmissionAuthorizationRef, RootExecutionAuthorizationRef, RootJudgmentAuthorityRef,
    RootPolicyId, RuntimePolicyId, RuntimeSessionId, SessionStartOccurrenceId, StateRevisionId,
};
use crate::process::CheckedConstitutionBinding;
use crate::provenance::{
    BoundaryOccurrencePermissionV2, ConstitutedBoundary, MAX_BOUNDARY_PERMISSIONS,
};

/// The complete identity preimage for one candidate Program lineage edge.
///
/// This value is inert until [`AuthorityStore`] admits it. In particular, its
/// bytes and claimed ID supply no admission authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgramRevisionPreimage {
    pub semantics: ClauseSemanticsId,
    pub program: ProgramId,
    pub predecessor: Option<ProgramRevisionId>,
    pub snapshot: ProgramSnapshotId,
    pub change: ProgramChangeOccurrenceId,
}

impl ProgramRevisionPreimage {
    /// Derive the constitutional identity. Identity agreement remains inert;
    /// only admission through [`AuthorityStore`] creates authority.
    #[must_use]
    pub fn derived_claim(self) -> ProgramRevisionClaim {
        ProgramRevisionClaim {
            id: self.canonical_id(),
            preimage: self,
        }
    }

    /// Derive this preimage's identity through an independent implementation.
    ///
    /// This hook lets parity checks validate the identity without granting the
    /// checker any authority over admission.
    #[must_use]
    pub fn derive_id_with(
        &self,
        derive: impl FnOnce(&Self) -> ProgramRevisionId,
    ) -> ProgramRevisionId {
        derive(self)
    }

    /// Validate a claimed identity through an independent derivation hook.
    pub fn validate_id_with(
        &self,
        claimed: ProgramRevisionId,
        derive: impl FnOnce(&Self) -> ProgramRevisionId,
    ) -> Result<(), AuthorityError> {
        let derived = self.derive_id_with(derive);
        if claimed == derived {
            Ok(())
        } else {
            Err(AuthorityError::ProgramRevisionIdMismatch { claimed, derived })
        }
    }

    fn canonical_id(&self) -> ProgramRevisionId {
        derive_program_revision_id(
            self.semantics,
            self.program,
            self.predecessor,
            self.snapshot,
            self.change,
        )
    }
}

/// A candidate Program revision identity paired with its exact preimage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgramRevisionClaim {
    pub id: ProgramRevisionId,
    pub preimage: ProgramRevisionPreimage,
}

impl ProgramRevisionClaim {
    pub fn validate_derived_id(&self) -> Result<(), AuthorityError> {
        self.preimage
            .validate_id_with(self.id, ProgramRevisionPreimage::canonical_id)
    }
}

/// The exact successor action covered by a predecessor declaration.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SuccessorAdmissionScope {
    pub semantics: ClauseSemanticsId,
    pub program: ProgramId,
    pub snapshot: ProgramSnapshotId,
    pub change: ProgramChangeOccurrenceId,
}

impl SuccessorAdmissionScope {
    fn covers(&self, preimage: &ProgramRevisionPreimage) -> bool {
        self.semantics == preimage.semantics
            && self.program == preimage.program
            && self.snapshot == preimage.snapshot
            && self.change == preimage.change
    }
}

/// A successor admission declaration carried by a checked snapshot.
///
/// It remains candidate data until a Program revision selecting that snapshot
/// is admitted. Thereafter it can authorize only an immediate successor of
/// that exact admitted revision.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionSuccessorGrant {
    pub authorization: AdmissionAuthorizationRef,
    pub scope: SuccessorAdmissionScope,
}

/// Static eligibility for one Application/Mode pair.
///
/// A static grant is intentionally orthogonal to an Activation's causal
/// frontier. It neither creates an Activation nor satisfies a dynamic
/// prerequisite, effect authorization, observation, or continuation takeup.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StaticExecutionScope {
    /// Exact Mode requirement kind satisfied by this grant.
    pub kind: FormationRefV2,
    pub application: ApplicationId,
    pub mode: ModeId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionStaticExecutionGrant {
    pub authorization: ExecutionAuthorizationRef,
    pub scope: StaticExecutionScope,
}

/// Exact runtime State-admission action covered by one constitutive grant.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StateAdmissionScope {
    pub session: RuntimeSessionId,
    pub base: StateRevisionId,
    pub delta: CandidateDeltaId,
}

/// An exact admission scope after a checked package has bound the otherwise
/// cyclic package identity outside its canonical snapshot preimage.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CheckedStateAdmissionScope {
    pub package: ProcessPackageId,
    pub session: RuntimeSessionId,
    pub base: StateRevisionId,
    pub delta: CandidateDeltaId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionStateAdmissionGrant {
    pub authorization: AdmissionAuthorizationRef,
    pub scope: StateAdmissionScope,
}

/// Runtime boundary within which an authority may issue governed Judgments.
/// The occurrence itself still binds the exact Judgment, supports, candidate
/// delta, and provenance; this static scope does not create that occurrence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct JudgmentAuthorityScope {
    pub semantics: ClauseSemanticsId,
    pub session: RuntimeSessionId,
    pub policy: RuntimePolicyId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RevisionJudgmentAuthorityGrant {
    pub authority: JudgmentAuthorityRef,
    pub scope: JudgmentAuthorityScope,
}

/// Crate-internal bridge from a checked process package to authority.
///
/// Its fields are private and its constructor is crate-visible so untrusted
/// package users cannot promote arbitrary bytes to a checked snapshot claim.
/// Formation/checking owns construction; this module revalidates the claimed
/// content-derived snapshot identity before retaining it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedSnapshotAuthorityInput {
    package: ProcessPackageId,
    semantics: ClauseSemanticsId,
    snapshot: ProgramSnapshotId,
    canonical_snapshot_preimage: Box<[u8]>,
    successor_grants: Box<[RevisionSuccessorGrant]>,
    static_execution_grants: Box<[RevisionStaticExecutionGrant]>,
    state_admission_grants: Box<[RevisionStateAdmissionGrant]>,
    judgment_authority_grants: Box<[RevisionJudgmentAuthorityGrant]>,
}

impl CheckedSnapshotAuthorityInput {
    #[cfg(test)]
    pub(crate) fn from_checked_process_package_parts(
        package: ProcessPackageId,
        semantics: ClauseSemanticsId,
        snapshot: ProgramSnapshotId,
        canonical_snapshot_preimage: Vec<u8>,
        successor_grants: Vec<RevisionSuccessorGrant>,
        static_execution_grants: Vec<RevisionStaticExecutionGrant>,
    ) -> Result<Self, AuthorityError> {
        if !static_execution_grants.is_empty() {
            return Err(AuthorityError::StaticExecutionGrantRequiresConstitution);
        }
        Self::from_checked_process_package_parts_with_governance(
            package,
            semantics,
            snapshot,
            canonical_snapshot_preimage,
            successor_grants,
            static_execution_grants,
            Vec::new(),
            Vec::new(),
            None,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the checked package boundary keeps each authority domain explicit"
    )]
    pub(crate) fn from_checked_process_package_parts_with_governance(
        package: ProcessPackageId,
        semantics: ClauseSemanticsId,
        snapshot: ProgramSnapshotId,
        canonical_snapshot_preimage: Vec<u8>,
        successor_grants: Vec<RevisionSuccessorGrant>,
        static_execution_grants: Vec<RevisionStaticExecutionGrant>,
        state_admission_grants: Vec<RevisionStateAdmissionGrant>,
        judgment_authority_grants: Vec<RevisionJudgmentAuthorityGrant>,
        constitution: Option<&ResolvedProgramConstitutionV2>,
    ) -> Result<Self, AuthorityError> {
        let derived = derive_program_snapshot_id(semantics, &canonical_snapshot_preimage);
        if snapshot != derived {
            return Err(AuthorityError::ProgramSnapshotIdMismatch {
                claimed: snapshot,
                derived,
            });
        }

        ensure_snapshot_grants(
            semantics,
            snapshot,
            &successor_grants,
            &static_execution_grants,
            &state_admission_grants,
            &judgment_authority_grants,
            constitution,
        )?;

        Ok(Self {
            package,
            semantics,
            snapshot,
            canonical_snapshot_preimage: canonical_snapshot_preimage.into_boxed_slice(),
            successor_grants: successor_grants.into_boxed_slice(),
            static_execution_grants: static_execution_grants.into_boxed_slice(),
            state_admission_grants: state_admission_grants.into_boxed_slice(),
            judgment_authority_grants: judgment_authority_grants.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn package(&self) -> ProcessPackageId {
        self.package
    }

    #[must_use]
    pub const fn semantics(&self) -> ClauseSemanticsId {
        self.semantics
    }

    #[must_use]
    pub const fn snapshot(&self) -> ProgramSnapshotId {
        self.snapshot
    }

    #[must_use]
    pub fn canonical_snapshot_preimage(&self) -> &[u8] {
        &self.canonical_snapshot_preimage
    }

    #[must_use]
    pub fn successor_grants(&self) -> &[RevisionSuccessorGrant] {
        &self.successor_grants
    }

    #[must_use]
    pub fn static_execution_grants(&self) -> &[RevisionStaticExecutionGrant] {
        &self.static_execution_grants
    }

    #[must_use]
    pub fn state_admission_grants(&self) -> &[RevisionStateAdmissionGrant] {
        &self.state_admission_grants
    }

    #[must_use]
    pub fn judgment_authority_grants(&self) -> &[RevisionJudgmentAuthorityGrant] {
        &self.judgment_authority_grants
    }
}

/// The exact genesis action covered by one irreducible root declaration.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RootGenesisScope {
    pub semantics: ClauseSemanticsId,
    pub program: ProgramId,
    pub snapshot: ProgramSnapshotId,
    pub change: ProgramChangeOccurrenceId,
}

impl RootGenesisScope {
    fn covers(&self, preimage: &ProgramRevisionPreimage) -> bool {
        preimage.predecessor.is_none()
            && self.semantics == preimage.semantics
            && self.program == preimage.program
            && self.snapshot == preimage.snapshot
            && self.change == preimage.change
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RootGenesisGrant {
    pub authorization: RootAdmissionAuthorizationRef,
    pub scope: RootGenesisScope,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RootStaticExecutionGrant {
    pub authorization: RootExecutionAuthorizationRef,
    pub scope: StaticExecutionScope,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RootStateAdmissionGrant {
    pub authorization: RootAdmissionAuthorizationRef,
    pub scope: CheckedStateAdmissionScope,
}

/// Exact constitutional/runtime boundary within which a root-governed issuer
/// may create candidate-specific Admission authorization occurrences.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StateAdmissionIssuerScope {
    pub revision: ProgramRevisionId,
    pub package: ProcessPackageId,
    pub session: RuntimeSessionId,
    pub policy: RuntimePolicyId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RootStateAdmissionIssuerGrant {
    pub issuer: RootAdmissionAuthorizationIssuerRef,
    pub scope: StateAdmissionIssuerScope,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RootJudgmentAuthorityGrant {
    pub authority: RootJudgmentAuthorityRef,
    pub scope: JudgmentAuthorityScope,
}

/// An irreducible policy anchor established outside candidate package data.
///
/// This type deliberately has no canonical-wire implementation. Calling
/// [`AuthorityStore::establish_root_policy`] is a trusted external act; decode,
/// hash agreement, checking, or possession of package bytes cannot perform it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootPolicyAnchor {
    id: RootPolicyId,
    genesis_grants: Box<[RootGenesisGrant]>,
    static_execution_grants: Box<[RootStaticExecutionGrant]>,
    state_admission_grants: Box<[RootStateAdmissionGrant]>,
    judgment_authority_grants: Box<[RootJudgmentAuthorityGrant]>,
    state_admission_issuer_grants: Box<[RootStateAdmissionIssuerGrant]>,
}

impl RootPolicyAnchor {
    pub fn establish(
        id: RootPolicyId,
        genesis_grants: Vec<RootGenesisGrant>,
        static_execution_grants: Vec<RootStaticExecutionGrant>,
    ) -> Result<Self, AuthorityError> {
        Self::establish_with_governance(
            id,
            genesis_grants,
            static_execution_grants,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn establish_with_governance(
        id: RootPolicyId,
        genesis_grants: Vec<RootGenesisGrant>,
        static_execution_grants: Vec<RootStaticExecutionGrant>,
        state_admission_grants: Vec<RootStateAdmissionGrant>,
        judgment_authority_grants: Vec<RootJudgmentAuthorityGrant>,
        state_admission_issuer_grants: Vec<RootStateAdmissionIssuerGrant>,
    ) -> Result<Self, AuthorityError> {
        let mut genesis_refs = BTreeSet::new();
        for grant in &genesis_grants {
            if grant.authorization.policy != id {
                return Err(AuthorityError::RootPolicyReferenceMismatch {
                    policy: id,
                    referenced_policy: grant.authorization.policy,
                });
            }
            if !genesis_refs.insert(grant.authorization) {
                return Err(AuthorityError::DuplicateRootAdmissionGrant(
                    grant.authorization,
                ));
            }
        }

        let mut execution_refs = BTreeSet::new();
        for grant in &static_execution_grants {
            if grant.authorization.policy != id {
                return Err(AuthorityError::RootPolicyReferenceMismatch {
                    policy: id,
                    referenced_policy: grant.authorization.policy,
                });
            }
            if !execution_refs.insert(grant.authorization) {
                return Err(AuthorityError::DuplicateRootExecutionGrant(
                    grant.authorization,
                ));
            }
            validate_static_execution_scope(grant.scope)?;
        }

        let mut state_refs = BTreeSet::new();
        for grant in &state_admission_grants {
            if grant.authorization.policy != id {
                return Err(AuthorityError::RootPolicyReferenceMismatch {
                    policy: id,
                    referenced_policy: grant.authorization.policy,
                });
            }
            if !state_refs.insert(grant.authorization) {
                return Err(AuthorityError::DuplicateRootStateAdmissionGrant(
                    grant.authorization,
                ));
            }
            if genesis_refs.contains(&grant.authorization) {
                return Err(AuthorityError::RootAdmissionGrantDomainCollision(
                    grant.authorization,
                ));
            }
        }

        let mut judgment_refs = BTreeSet::new();
        for grant in &judgment_authority_grants {
            if grant.authority.policy != id {
                return Err(AuthorityError::RootPolicyReferenceMismatch {
                    policy: id,
                    referenced_policy: grant.authority.policy,
                });
            }
            if !judgment_refs.insert(grant.authority) {
                return Err(AuthorityError::DuplicateRootJudgmentAuthorityGrant(
                    grant.authority,
                ));
            }
        }

        let mut issuer_refs = BTreeSet::new();
        for grant in &state_admission_issuer_grants {
            if grant.issuer.policy != id {
                return Err(AuthorityError::RootPolicyReferenceMismatch {
                    policy: id,
                    referenced_policy: grant.issuer.policy,
                });
            }
            if !issuer_refs.insert(grant.issuer) {
                return Err(AuthorityError::DuplicateRootStateAdmissionIssuerGrant(
                    grant.issuer,
                ));
            }
        }

        Ok(Self {
            id,
            genesis_grants: genesis_grants.into_boxed_slice(),
            static_execution_grants: static_execution_grants.into_boxed_slice(),
            state_admission_grants: state_admission_grants.into_boxed_slice(),
            judgment_authority_grants: judgment_authority_grants.into_boxed_slice(),
            state_admission_issuer_grants: state_admission_issuer_grants.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> RootPolicyId {
        self.id
    }

    #[must_use]
    pub fn genesis_grants(&self) -> &[RootGenesisGrant] {
        &self.genesis_grants
    }

    #[must_use]
    pub fn static_execution_grants(&self) -> &[RootStaticExecutionGrant] {
        &self.static_execution_grants
    }

    #[must_use]
    pub fn state_admission_grants(&self) -> &[RootStateAdmissionGrant] {
        &self.state_admission_grants
    }

    #[must_use]
    pub fn state_admission_issuer_grants(&self) -> &[RootStateAdmissionIssuerGrant] {
        &self.state_admission_issuer_grants
    }

    #[must_use]
    pub fn judgment_authority_grants(&self) -> &[RootJudgmentAuthorityGrant] {
        &self.judgment_authority_grants
    }
}

/// One admitted Program revision and the exact checked snapshot binding from
/// which its declarations became authoritative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedProgramRevision {
    claim: ProgramRevisionClaim,
    package: ProcessPackageId,
    canonical_snapshot_preimage: Box<[u8]>,
    successor_grants: BTreeMap<AdmissionAuthorizationRef, SuccessorAdmissionScope>,
    static_execution_grants: BTreeMap<ExecutionAuthorizationRef, StaticExecutionScope>,
    state_admission_grants: BTreeMap<AdmissionAuthorizationRef, StateAdmissionScope>,
    judgment_authority_grants: BTreeMap<JudgmentAuthorityRef, JudgmentAuthorityScope>,
}

impl AdmittedProgramRevision {
    #[must_use]
    pub const fn package(&self) -> ProcessPackageId {
        self.package
    }

    #[must_use]
    pub const fn claim(&self) -> ProgramRevisionClaim {
        self.claim
    }

    #[must_use]
    pub fn canonical_snapshot_preimage(&self) -> &[u8] {
        &self.canonical_snapshot_preimage
    }

    #[must_use]
    pub fn successor_scope(
        &self,
        authorization: AdmissionAuthorizationRef,
    ) -> Option<&SuccessorAdmissionScope> {
        self.successor_grants.get(&authorization)
    }

    #[must_use]
    pub fn static_execution_scope(
        &self,
        authorization: ExecutionAuthorizationRef,
    ) -> Option<&StaticExecutionScope> {
        self.static_execution_grants.get(&authorization)
    }

    #[must_use]
    pub fn state_admission_scope(
        &self,
        authorization: AdmissionAuthorizationRef,
    ) -> Option<CheckedStateAdmissionScope> {
        let scope = self.state_admission_grants.get(&authorization)?;
        Some(CheckedStateAdmissionScope {
            package: self.package,
            session: scope.session,
            base: scope.base,
            delta: scope.delta,
        })
    }

    #[must_use]
    pub fn judgment_authority_scope(
        &self,
        authority: JudgmentAuthorityRef,
    ) -> Option<&JudgmentAuthorityScope> {
        self.judgment_authority_grants.get(&authority)
    }
}

/// Exact identity preimage for a RuntimeSession's initial State revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitialStateRevisionPreimage {
    pub semantics: ClauseSemanticsId,
    pub session: RuntimeSessionId,
    pub start: SessionStartOccurrenceId,
    pub policy: RuntimePolicyId,
    pub canonical_state_snapshot: Box<[u8]>,
}

impl InitialStateRevisionPreimage {
    #[must_use]
    pub fn derived_id(&self) -> StateRevisionId {
        derive_state_revision_id(
            self.semantics,
            self.session,
            None,
            StateRevisionCausePreimage::SessionStart(self.start),
            &self.canonical_state_snapshot,
            self.policy,
        )
    }
}

/// An externally established runtime lineage anchor.
///
/// The session and start identities are nominal inputs. The initial State
/// revision is derived from the exact state preimage and retained by the store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSessionAnchor {
    pub session: RuntimeSessionId,
    pub program_revision: ProgramRevisionId,
    pub semantics: ClauseSemanticsId,
    pub policy: RuntimePolicyId,
    pub start: SessionStartOccurrenceId,
    pub initial_state: InitialStateRevisionPreimage,
}

impl RuntimeSessionAnchor {
    #[must_use]
    pub fn establish(
        session: RuntimeSessionId,
        program_revision: ProgramRevisionId,
        semantics: ClauseSemanticsId,
        policy: RuntimePolicyId,
        start: SessionStartOccurrenceId,
        canonical_initial_state_snapshot: Vec<u8>,
    ) -> Self {
        Self {
            session,
            program_revision,
            semantics,
            policy,
            start,
            initial_state: InitialStateRevisionPreimage {
                semantics,
                session,
                start,
                policy,
                canonical_state_snapshot: canonical_initial_state_snapshot.into_boxed_slice(),
            },
        }
    }

    pub fn validate(&self) -> Result<(), AuthorityError> {
        if self.initial_state.session != self.session
            || self.initial_state.semantics != self.semantics
            || self.initial_state.policy != self.policy
            || self.initial_state.start != self.start
        {
            return Err(AuthorityError::InitialStatePinMismatch(self.session));
        }
        Ok(())
    }

    #[must_use]
    pub fn initial_state_id(&self) -> StateRevisionId {
        self.initial_state.derived_id()
    }
}

/// Independently established external boundary used by occurrence provenance.
/// This is an external anchor, not canonical package content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryAnchor {
    pub boundary: BoundaryRef,
    pub permissions: Vec<BoundaryOccurrencePermissionV2>,
}

/// Independently established evidence entering through one exact boundary.
/// Equal evidence bytes under independently established IDs remain distinct.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceAnchor {
    pub evidence: ExternalEvidenceRef,
    pub boundary: BoundaryRef,
    pub permissions: Vec<BoundaryPermissionLocalId>,
    pub exact_evidence: Box<[u8]>,
}

/// In-memory authority state. It is deliberately absent from canonical package
/// encoding and must be re-established from authoritative external inputs.
#[derive(Clone, Debug, Default)]
pub struct AuthorityStore {
    root_policies: BTreeMap<RootPolicyId, RootPolicyAnchor>,
    revisions: BTreeMap<ProgramRevisionId, AdmittedProgramRevision>,
    snapshot_preimages: BTreeMap<ProgramSnapshotId, Box<[u8]>>,
    sessions: BTreeMap<RuntimeSessionId, RuntimeSessionAnchor>,
    session_starts: BTreeMap<SessionStartOccurrenceId, RuntimeSessionId>,
    boundaries: BTreeMap<BoundaryRef, BoundaryAnchor>,
    evidence: BTreeMap<ExternalEvidenceRef, EvidenceAnchor>,
}

impl AuthorityStore {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            root_policies: BTreeMap::new(),
            revisions: BTreeMap::new(),
            snapshot_preimages: BTreeMap::new(),
            sessions: BTreeMap::new(),
            session_starts: BTreeMap::new(),
            boundaries: BTreeMap::new(),
            evidence: BTreeMap::new(),
        }
    }

    pub fn establish_root_policy(
        &mut self,
        anchor: RootPolicyAnchor,
    ) -> Result<(), AuthorityError> {
        insert_exact(
            &mut self.root_policies,
            anchor.id,
            anchor,
            AuthorityError::RootPolicyAlreadyEstablished,
        )
    }

    #[must_use]
    pub fn root_policy(&self, id: RootPolicyId) -> Option<&RootPolicyAnchor> {
        self.root_policies.get(&id)
    }

    /// Admit a root Program revision under one exact external root grant.
    pub fn admit_genesis(
        &mut self,
        claim: ProgramRevisionClaim,
        snapshot: &CheckedSnapshotAuthorityInput,
        policy: RootPolicyId,
        authorization: RootAdmissionAuthorizationRef,
    ) -> Result<(), AuthorityError> {
        claim.validate_derived_id()?;
        if claim.preimage.predecessor.is_some() {
            return Err(AuthorityError::GenesisHasPredecessor(claim.id));
        }
        validate_snapshot_binding(&claim.preimage, snapshot)?;

        let root = self
            .root_policies
            .get(&policy)
            .ok_or(AuthorityError::UnknownRootPolicy(policy))?;
        if authorization.policy != policy {
            return Err(AuthorityError::RootPolicyReferenceMismatch {
                policy,
                referenced_policy: authorization.policy,
            });
        }
        let grant = root
            .genesis_grants
            .iter()
            .find(|grant| grant.authorization == authorization)
            .ok_or(AuthorityError::UnknownRootAdmissionGrant(authorization))?;
        if !grant.scope.covers(&claim.preimage) {
            return Err(AuthorityError::RootAdmissionOutOfScope(authorization));
        }

        self.insert_revision(claim, snapshot)
    }

    /// Admit an immediate successor using only a grant declared by its exact,
    /// already admitted predecessor.
    pub fn admit_successor(
        &mut self,
        claim: ProgramRevisionClaim,
        snapshot: &CheckedSnapshotAuthorityInput,
        authorization: AdmissionAuthorizationRef,
    ) -> Result<(), AuthorityError> {
        claim.validate_derived_id()?;
        let predecessor_id = claim
            .preimage
            .predecessor
            .ok_or(AuthorityError::SuccessorMissingPredecessor(claim.id))?;
        validate_snapshot_binding(&claim.preimage, snapshot)?;

        let predecessor = self
            .revisions
            .get(&predecessor_id)
            .ok_or(AuthorityError::UnknownPredecessor(predecessor_id))?;
        if predecessor.claim.preimage.program != claim.preimage.program {
            return Err(AuthorityError::ProgramLineageMismatch {
                predecessor: predecessor_id,
                candidate_program: claim.preimage.program,
            });
        }
        if authorization.snapshot != predecessor.claim.preimage.snapshot {
            return Err(AuthorityError::PredecessorAuthorizationMismatch {
                predecessor: predecessor_id,
                authorization,
            });
        }
        let scope = predecessor.successor_grants.get(&authorization).ok_or(
            AuthorityError::UnknownPredecessorAdmissionGrant {
                predecessor: predecessor_id,
                authorization,
            },
        )?;
        if !scope.covers(&claim.preimage) {
            return Err(AuthorityError::PredecessorAdmissionOutOfScope {
                predecessor: predecessor_id,
                authorization,
            });
        }

        self.insert_revision(claim, snapshot)
    }

    fn insert_revision(
        &mut self,
        claim: ProgramRevisionClaim,
        snapshot: &CheckedSnapshotAuthorityInput,
    ) -> Result<(), AuthorityError> {
        if self.revisions.contains_key(&claim.id) {
            return Err(AuthorityError::ProgramRevisionAlreadyAdmitted(claim.id));
        }
        if let Some(existing) = self.snapshot_preimages.get(&snapshot.snapshot)
            && existing.as_ref() != snapshot.canonical_snapshot_preimage.as_ref()
        {
            return Err(AuthorityError::SnapshotPreimageBindingMismatch(
                snapshot.snapshot,
            ));
        }

        let admitted = AdmittedProgramRevision {
            claim,
            package: snapshot.package,
            canonical_snapshot_preimage: snapshot.canonical_snapshot_preimage.clone(),
            successor_grants: snapshot
                .successor_grants
                .iter()
                .map(|grant| (grant.authorization, grant.scope))
                .collect(),
            static_execution_grants: snapshot
                .static_execution_grants
                .iter()
                .map(|grant| (grant.authorization, grant.scope))
                .collect(),
            state_admission_grants: snapshot
                .state_admission_grants
                .iter()
                .map(|grant| (grant.authorization, grant.scope))
                .collect(),
            judgment_authority_grants: snapshot
                .judgment_authority_grants
                .iter()
                .map(|grant| (grant.authority, grant.scope))
                .collect(),
        };
        self.snapshot_preimages
            .entry(snapshot.snapshot)
            .or_insert_with(|| snapshot.canonical_snapshot_preimage.clone());
        self.revisions.insert(claim.id, admitted);
        Ok(())
    }

    #[must_use]
    pub fn revision(&self, id: ProgramRevisionId) -> Option<&AdmittedProgramRevision> {
        self.revisions.get(&id)
    }

    #[must_use]
    pub fn snapshot_preimage(&self, id: ProgramSnapshotId) -> Option<&[u8]> {
        self.snapshot_preimages.get(&id).map(AsRef::as_ref)
    }

    #[must_use]
    pub fn revision_state_admission_scope(
        &self,
        revision: ProgramRevisionId,
        authorization: AdmissionAuthorizationRef,
    ) -> Option<CheckedStateAdmissionScope> {
        self.revisions
            .get(&revision)?
            .state_admission_scope(authorization)
    }

    /// Resolve the sole constitutive authorization for one exact State
    /// Admission action. Missing and ambiguous grants both fail closed.
    ///
    /// This lookup selects only by the complete checked scope. It never
    /// derives authority from a candidate, invents a policy, or changes the
    /// admitted Program revision.
    #[must_use]
    pub fn unique_revision_state_admission_authorization(
        &self,
        revision: ProgramRevisionId,
        exact_scope: CheckedStateAdmissionScope,
    ) -> Option<AdmissionAuthorizationRef> {
        let admitted = self.revisions.get(&revision)?;
        let mut matching =
            admitted
                .state_admission_grants
                .iter()
                .filter_map(|(authorization, scope)| {
                    (CheckedStateAdmissionScope {
                        package: admitted.package,
                        session: scope.session,
                        base: scope.base,
                        delta: scope.delta,
                    } == exact_scope)
                        .then_some(*authorization)
                });
        let authorization = matching.next()?;
        matching.next().is_none().then_some(authorization)
    }

    #[must_use]
    pub fn root_state_admission_scope(
        &self,
        policy: RootPolicyId,
        authorization: RootAdmissionAuthorizationRef,
    ) -> Option<CheckedStateAdmissionScope> {
        self.root_policies
            .get(&policy)?
            .state_admission_grants
            .iter()
            .find(|grant| grant.authorization == authorization)
            .map(|grant| grant.scope)
    }

    #[must_use]
    pub fn root_state_admission_issuer_scope(
        &self,
        issuer: RootAdmissionAuthorizationIssuerRef,
    ) -> Option<StateAdmissionIssuerScope> {
        self.root_policies
            .get(&issuer.policy)?
            .state_admission_issuer_grants
            .iter()
            .find(|grant| grant.issuer == issuer)
            .map(|grant| grant.scope)
    }

    #[must_use]
    pub fn revision_static_execution_scope(
        &self,
        revision: ProgramRevisionId,
        authorization: ExecutionAuthorizationRef,
    ) -> Option<&StaticExecutionScope> {
        self.revisions
            .get(&revision)?
            .static_execution_scope(authorization)
    }

    #[must_use]
    pub fn root_static_execution_scope(
        &self,
        policy: RootPolicyId,
        authorization: RootExecutionAuthorizationRef,
    ) -> Option<&StaticExecutionScope> {
        self.root_policies
            .get(&policy)?
            .static_execution_grants
            .iter()
            .find(|grant| grant.authorization == authorization)
            .map(|grant| &grant.scope)
    }

    #[must_use]
    pub fn revision_judgment_authority_scope(
        &self,
        revision: ProgramRevisionId,
        authority: JudgmentAuthorityRef,
    ) -> Option<&JudgmentAuthorityScope> {
        self.revisions
            .get(&revision)?
            .judgment_authority_scope(authority)
    }

    #[must_use]
    pub fn root_judgment_authority_scope(
        &self,
        policy: RootPolicyId,
        authority: RootJudgmentAuthorityRef,
    ) -> Option<&JudgmentAuthorityScope> {
        self.root_policies
            .get(&policy)?
            .judgment_authority_grants
            .iter()
            .find(|grant| grant.authority == authority)
            .map(|grant| &grant.scope)
    }

    pub fn establish_runtime_session(
        &mut self,
        anchor: RuntimeSessionAnchor,
    ) -> Result<(), AuthorityError> {
        anchor.validate()?;
        let revision = self.revisions.get(&anchor.program_revision).ok_or(
            AuthorityError::UnknownProgramRevision(anchor.program_revision),
        )?;
        if revision.claim.preimage.semantics != anchor.semantics {
            return Err(AuthorityError::RuntimeSemanticsMismatch(anchor.session));
        }
        if let Some(existing) = self.session_starts.get(&anchor.start)
            && *existing != anchor.session
        {
            return Err(AuthorityError::SessionStartAlreadyEstablished(anchor.start));
        }
        let session = anchor.session;
        let start = anchor.start;
        insert_exact(
            &mut self.sessions,
            session,
            anchor,
            AuthorityError::RuntimeSessionAlreadyEstablished,
        )?;
        self.session_starts.insert(start, session);
        Ok(())
    }

    #[must_use]
    pub fn runtime_session(&self, id: RuntimeSessionId) -> Option<&RuntimeSessionAnchor> {
        self.sessions.get(&id)
    }

    pub fn establish_boundary(&mut self, anchor: BoundaryAnchor) -> Result<(), AuthorityError> {
        if anchor.permissions.len() > MAX_BOUNDARY_PERMISSIONS {
            return Err(AuthorityError::BoundaryPermissionLimitExceeded(
                anchor.boundary,
            ));
        }
        crate::provenance::validate_boundary(&ConstitutedBoundary {
            id: anchor.boundary,
            permissions: anchor.permissions.clone(),
        })
        .map_err(|_| AuthorityError::InvalidBoundaryContract(anchor.boundary))?;
        for permission in &anchor.permissions {
            let admitted_revision = match permission.pins.constitution {
                CheckedConstitutionBinding::Candidate { snapshot, .. } => {
                    if snapshot != permission.pins.snapshot
                        || permission.pins.runtime_session.is_some()
                        || permission.pins.runtime_policy.is_some()
                    {
                        return Err(AuthorityError::BoundaryCandidateScopeMismatch(
                            anchor.boundary,
                        ));
                    }
                    None
                }
                CheckedConstitutionBinding::Admitted { revision } => {
                    let admitted = self
                        .revisions
                        .get(&revision)
                        .ok_or(AuthorityError::UnknownProgramRevision(revision))?;
                    if admitted.claim.preimage.semantics != permission.pins.semantics
                        || admitted.claim.preimage.snapshot != permission.pins.snapshot
                    {
                        return Err(AuthorityError::BoundaryRevisionPinMismatch(anchor.boundary));
                    }
                    Some(revision)
                }
            };

            match (
                permission.pins.runtime_session,
                permission.pins.runtime_policy,
            ) {
                (None, None) => {}
                (Some(session), Some(policy)) => {
                    let established = self
                        .sessions
                        .get(&session)
                        .ok_or(AuthorityError::UnknownRuntimeSession(session))?;
                    if established.semantics != permission.pins.semantics
                        || Some(established.program_revision) != admitted_revision
                        || established.policy != policy
                    {
                        return Err(AuthorityError::BoundarySessionPinMismatch(anchor.boundary));
                    }
                }
                _ => {
                    return Err(AuthorityError::IncompleteBoundaryRuntimePins(
                        anchor.boundary,
                    ));
                }
            }
        }
        insert_exact(
            &mut self.boundaries,
            anchor.boundary,
            anchor,
            AuthorityError::BoundaryAlreadyEstablished,
        )
    }

    pub fn establish_evidence(&mut self, anchor: EvidenceAnchor) -> Result<(), AuthorityError> {
        if anchor.permissions.is_empty()
            || !anchor.permissions.windows(2).all(|pair| pair[0] < pair[1])
        {
            return Err(AuthorityError::NonCanonicalBoundaryEvidencePermissions(
                anchor.evidence,
            ));
        }
        for permission in &anchor.permissions {
            self.boundary_permission(anchor.boundary, *permission)
                .ok_or(AuthorityError::UnknownBoundaryPermission {
                    boundary: anchor.boundary,
                    permission: *permission,
                })?;
        }
        insert_exact(
            &mut self.evidence,
            anchor.evidence,
            anchor,
            AuthorityError::EvidenceAlreadyEstablished,
        )
    }

    #[must_use]
    pub fn boundary(&self, id: BoundaryRef) -> Option<&BoundaryAnchor> {
        self.boundaries.get(&id)
    }

    #[must_use]
    pub fn boundary_permission(
        &self,
        boundary: BoundaryRef,
        permission: BoundaryPermissionLocalId,
    ) -> Option<&BoundaryOccurrencePermissionV2> {
        self.boundaries
            .get(&boundary)?
            .permissions
            .iter()
            .find(|candidate| candidate.id == permission)
    }

    #[must_use]
    pub fn evidence(&self, id: ExternalEvidenceRef) -> Option<&EvidenceAnchor> {
        self.evidence.get(&id)
    }

    #[must_use]
    pub fn external_provenance_is_anchored(
        &self,
        boundary: BoundaryRef,
        evidence: ExternalEvidenceRef,
        permission: BoundaryPermissionLocalId,
    ) -> bool {
        self.boundaries.contains_key(&boundary)
            && self.evidence.get(&evidence).is_some_and(|anchor| {
                anchor.boundary == boundary && anchor.permissions.binary_search(&permission).is_ok()
            })
    }
}

fn ensure_snapshot_grants(
    semantics: ClauseSemanticsId,
    snapshot: ProgramSnapshotId,
    successor_grants: &[RevisionSuccessorGrant],
    static_execution_grants: &[RevisionStaticExecutionGrant],
    state_admission_grants: &[RevisionStateAdmissionGrant],
    judgment_authority_grants: &[RevisionJudgmentAuthorityGrant],
    constitution: Option<&ResolvedProgramConstitutionV2>,
) -> Result<(), AuthorityError> {
    if let Some(constitution) = constitution {
        if constitution.snapshot() != snapshot || constitution.semantics() != semantics {
            return Err(AuthorityError::ConstitutionSnapshotBindingMismatch(
                snapshot,
            ));
        }
    } else if !static_execution_grants.is_empty() {
        return Err(AuthorityError::StaticExecutionGrantRequiresConstitution);
    }

    let mut successor_refs = BTreeSet::new();
    for grant in successor_grants {
        if grant.authorization.snapshot != snapshot {
            return Err(AuthorityError::SnapshotGrantReferenceMismatch(snapshot));
        }
        if !successor_refs.insert(grant.authorization) {
            return Err(AuthorityError::DuplicateRevisionAdmissionGrant(
                grant.authorization,
            ));
        }
    }

    let mut execution_refs = BTreeSet::new();
    for grant in static_execution_grants {
        if grant.authorization.snapshot != snapshot
            || grant.scope.kind.snapshot != snapshot
            || grant.scope.application.snapshot != snapshot
            || grant.scope.mode.operator.snapshot != snapshot
        {
            return Err(AuthorityError::SnapshotGrantReferenceMismatch(snapshot));
        }
        if !execution_refs.insert(grant.authorization) {
            return Err(AuthorityError::DuplicateRevisionExecutionGrant(
                grant.authorization,
            ));
        }
        validate_static_execution_scope(grant.scope)?;
        let contract = constitution
            .and_then(|constitution| {
                constitution.executable_contract(grant.scope.application, grant.scope.mode)
            })
            .ok_or(AuthorityError::UnknownStaticExecutionContract {
                application: grant.scope.application,
                mode: grant.scope.mode,
            })?;
        if !contract
            .authorization_requirements
            .iter()
            .any(|requirement| requirement.kind == grant.scope.kind)
        {
            return Err(AuthorityError::StaticExecutionRequirementMismatch {
                application: grant.scope.application,
                mode: grant.scope.mode,
                kind: grant.scope.kind,
            });
        }
    }

    let mut state_refs = BTreeSet::new();
    for grant in state_admission_grants {
        if grant.authorization.snapshot != snapshot {
            return Err(AuthorityError::SnapshotGrantReferenceMismatch(snapshot));
        }
        if successor_refs.contains(&grant.authorization) {
            return Err(AuthorityError::AdmissionGrantDomainCollision(
                grant.authorization,
            ));
        }
        if !state_refs.insert(grant.authorization) {
            return Err(AuthorityError::DuplicateRevisionStateAdmissionGrant(
                grant.authorization,
            ));
        }
    }

    let mut judgment_refs = BTreeSet::new();
    for grant in judgment_authority_grants {
        if grant.authority.snapshot != snapshot {
            return Err(AuthorityError::SnapshotGrantReferenceMismatch(snapshot));
        }
        if grant.scope.semantics != semantics {
            return Err(AuthorityError::JudgmentAuthoritySemanticsMismatch(
                grant.authority,
            ));
        }
        if !judgment_refs.insert(grant.authority) {
            return Err(AuthorityError::DuplicateRevisionJudgmentAuthorityGrant(
                grant.authority,
            ));
        }
    }
    Ok(())
}

fn validate_static_execution_scope(scope: StaticExecutionScope) -> Result<(), AuthorityError> {
    let snapshot = scope.application.snapshot;
    if scope.kind.snapshot != snapshot || scope.mode.operator.snapshot != snapshot {
        return Err(AuthorityError::StaticExecutionScopeSnapshotMismatch);
    }
    Ok(())
}

fn validate_snapshot_binding(
    revision: &ProgramRevisionPreimage,
    snapshot: &CheckedSnapshotAuthorityInput,
) -> Result<(), AuthorityError> {
    if revision.snapshot != snapshot.snapshot || revision.semantics != snapshot.semantics {
        return Err(AuthorityError::RevisionSnapshotBindingMismatch {
            revision_snapshot: revision.snapshot,
            checked_snapshot: snapshot.snapshot,
        });
    }
    Ok(())
}

fn insert_exact<K, V>(
    entries: &mut BTreeMap<K, V>,
    key: K,
    value: V,
    duplicate: impl FnOnce(K) -> AuthorityError,
) -> Result<(), AuthorityError>
where
    K: Copy + Ord,
    V: Eq,
{
    match entries.get(&key) {
        None => {
            entries.insert(key, value);
            Ok(())
        }
        Some(existing) if existing == &value => Ok(()),
        Some(_) => Err(duplicate(key)),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorityError {
    ProgramSnapshotIdMismatch {
        claimed: ProgramSnapshotId,
        derived: ProgramSnapshotId,
    },
    ProgramRevisionIdMismatch {
        claimed: ProgramRevisionId,
        derived: ProgramRevisionId,
    },
    SnapshotGrantReferenceMismatch(ProgramSnapshotId),
    DuplicateRevisionAdmissionGrant(AdmissionAuthorizationRef),
    DuplicateRevisionExecutionGrant(ExecutionAuthorizationRef),
    DuplicateRevisionStateAdmissionGrant(AdmissionAuthorizationRef),
    DuplicateRevisionJudgmentAuthorityGrant(JudgmentAuthorityRef),
    AdmissionGrantDomainCollision(AdmissionAuthorizationRef),
    StaticExecutionScopeSnapshotMismatch,
    StaticExecutionGrantRequiresConstitution,
    ConstitutionSnapshotBindingMismatch(ProgramSnapshotId),
    UnknownStaticExecutionContract {
        application: ApplicationId,
        mode: ModeId,
    },
    StaticExecutionRequirementMismatch {
        application: ApplicationId,
        mode: ModeId,
        kind: FormationRefV2,
    },
    JudgmentAuthoritySemanticsMismatch(JudgmentAuthorityRef),
    RootPolicyReferenceMismatch {
        policy: RootPolicyId,
        referenced_policy: RootPolicyId,
    },
    DuplicateRootAdmissionGrant(RootAdmissionAuthorizationRef),
    DuplicateRootExecutionGrant(RootExecutionAuthorizationRef),
    DuplicateRootStateAdmissionGrant(RootAdmissionAuthorizationRef),
    DuplicateRootStateAdmissionIssuerGrant(RootAdmissionAuthorizationIssuerRef),
    DuplicateRootJudgmentAuthorityGrant(RootJudgmentAuthorityRef),
    RootAdmissionGrantDomainCollision(RootAdmissionAuthorizationRef),
    RootPolicyAlreadyEstablished(RootPolicyId),
    UnknownRootPolicy(RootPolicyId),
    UnknownRootAdmissionGrant(RootAdmissionAuthorizationRef),
    RootAdmissionOutOfScope(RootAdmissionAuthorizationRef),
    GenesisHasPredecessor(ProgramRevisionId),
    SuccessorMissingPredecessor(ProgramRevisionId),
    UnknownPredecessor(ProgramRevisionId),
    ProgramLineageMismatch {
        predecessor: ProgramRevisionId,
        candidate_program: ProgramId,
    },
    PredecessorAuthorizationMismatch {
        predecessor: ProgramRevisionId,
        authorization: AdmissionAuthorizationRef,
    },
    UnknownPredecessorAdmissionGrant {
        predecessor: ProgramRevisionId,
        authorization: AdmissionAuthorizationRef,
    },
    PredecessorAdmissionOutOfScope {
        predecessor: ProgramRevisionId,
        authorization: AdmissionAuthorizationRef,
    },
    RevisionSnapshotBindingMismatch {
        revision_snapshot: ProgramSnapshotId,
        checked_snapshot: ProgramSnapshotId,
    },
    SnapshotPreimageBindingMismatch(ProgramSnapshotId),
    ProgramRevisionAlreadyAdmitted(ProgramRevisionId),
    UnknownProgramRevision(ProgramRevisionId),
    InitialStatePinMismatch(RuntimeSessionId),
    RuntimeSemanticsMismatch(RuntimeSessionId),
    RuntimeSessionAlreadyEstablished(RuntimeSessionId),
    SessionStartAlreadyEstablished(SessionStartOccurrenceId),
    UnknownRuntimeSession(RuntimeSessionId),
    BoundaryRevisionPinMismatch(BoundaryRef),
    BoundarySessionPinMismatch(BoundaryRef),
    BoundaryCandidateScopeMismatch(BoundaryRef),
    IncompleteBoundaryRuntimePins(BoundaryRef),
    BoundaryPermissionLimitExceeded(BoundaryRef),
    NonCanonicalBoundaryPermissions(BoundaryRef),
    InvalidBoundaryContract(BoundaryRef),
    BoundaryAlreadyEstablished(BoundaryRef),
    UnknownBoundary(BoundaryRef),
    UnknownBoundaryPermission {
        boundary: BoundaryRef,
        permission: BoundaryPermissionLocalId,
    },
    NonCanonicalBoundaryEvidencePermissions(ExternalEvidenceRef),
    EvidenceAlreadyEstablished(ExternalEvidenceRef),
}

impl fmt::Display for AuthorityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "process authority rejected: {self:?}")
    }
}

impl Error for AuthorityError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{
        AdmissionAuthorizationLocalId, ExecutionAuthorizationLocalId, IDENTITY_BYTES,
    };

    macro_rules! id {
        ($kind:ident, $byte:expr) => {
            $kind::from_bytes([$byte; IDENTITY_BYTES])
        };
    }

    fn checked_snapshot(
        semantics: ClauseSemanticsId,
        bytes: &[u8],
        grants: Vec<RevisionSuccessorGrant>,
    ) -> CheckedSnapshotAuthorityInput {
        let snapshot = derive_program_snapshot_id(semantics, bytes);
        CheckedSnapshotAuthorityInput::from_checked_process_package_parts(
            id!(ProcessPackageId, 0x7f),
            semantics,
            snapshot,
            bytes.to_vec(),
            grants,
            Vec::new(),
        )
        .expect("checked snapshot")
    }

    #[test]
    fn candidate_bytes_cannot_admit_genesis_without_root_authority() {
        let semantics = id!(ClauseSemanticsId, 1);
        let checked = checked_snapshot(semantics, b"genesis", Vec::new());
        let preimage = ProgramRevisionPreimage {
            semantics,
            program: id!(ProgramId, 2),
            predecessor: None,
            snapshot: checked.snapshot(),
            change: id!(ProgramChangeOccurrenceId, 3),
        };
        let claim = ProgramRevisionClaim {
            id: preimage.canonical_id(),
            preimage,
        };
        let mut store = AuthorityStore::new();
        let policy = id!(RootPolicyId, 4);
        let authorization = RootAdmissionAuthorizationRef {
            policy,
            local: AdmissionAuthorizationLocalId::new(0),
        };

        assert_eq!(
            store.admit_genesis(claim, &checked, policy, authorization),
            Err(AuthorityError::UnknownRootPolicy(policy))
        );
        assert!(store.revision(claim.id).is_none());
    }

    #[test]
    fn successor_requires_the_exact_predecessor_declared_scope() {
        let semantics = id!(ClauseSemanticsId, 1);
        let program = id!(ProgramId, 2);
        let root_policy = id!(RootPolicyId, 3);
        let root_authorization = RootAdmissionAuthorizationRef {
            policy: root_policy,
            local: AdmissionAuthorizationLocalId::new(0),
        };
        let successor_bytes = b"successor";
        let successor_snapshot = derive_program_snapshot_id(semantics, successor_bytes);
        let successor_change = id!(ProgramChangeOccurrenceId, 5);

        let genesis_bytes = b"genesis";
        let genesis_snapshot = derive_program_snapshot_id(semantics, genesis_bytes);
        let successor_authorization = AdmissionAuthorizationRef {
            snapshot: genesis_snapshot,
            local: AdmissionAuthorizationLocalId::new(1),
        };
        let genesis_checked = checked_snapshot(
            semantics,
            genesis_bytes,
            vec![RevisionSuccessorGrant {
                authorization: successor_authorization,
                scope: SuccessorAdmissionScope {
                    semantics,
                    program,
                    snapshot: successor_snapshot,
                    change: successor_change,
                },
            }],
        );
        let genesis_preimage = ProgramRevisionPreimage {
            semantics,
            program,
            predecessor: None,
            snapshot: genesis_snapshot,
            change: id!(ProgramChangeOccurrenceId, 4),
        };
        let genesis = ProgramRevisionClaim {
            id: genesis_preimage.canonical_id(),
            preimage: genesis_preimage,
        };

        let mut store = AuthorityStore::new();
        store
            .establish_root_policy(
                RootPolicyAnchor::establish(
                    root_policy,
                    vec![RootGenesisGrant {
                        authorization: root_authorization,
                        scope: RootGenesisScope {
                            semantics,
                            program,
                            snapshot: genesis_snapshot,
                            change: genesis_preimage.change,
                        },
                    }],
                    Vec::new(),
                )
                .expect("root policy"),
            )
            .expect("establish root");
        store
            .admit_genesis(genesis, &genesis_checked, root_policy, root_authorization)
            .expect("admit genesis");

        let successor_checked = checked_snapshot(semantics, successor_bytes, Vec::new());
        let successor_preimage = ProgramRevisionPreimage {
            semantics,
            program,
            predecessor: Some(genesis.id),
            snapshot: successor_snapshot,
            change: successor_change,
        };
        let successor = ProgramRevisionClaim {
            id: successor_preimage.canonical_id(),
            preimage: successor_preimage,
        };
        store
            .admit_successor(successor, &successor_checked, successor_authorization)
            .expect("admit exact successor");

        assert_eq!(
            store
                .revision(successor.id)
                .expect("admitted successor")
                .canonical_snapshot_preimage(),
            successor_bytes
        );
    }

    #[test]
    fn static_grants_do_not_anchor_external_provenance() {
        let store = AuthorityStore::new();
        let boundary = id!(BoundaryRef, 1);
        let evidence = id!(ExternalEvidenceRef, 2);
        assert!(!store.external_provenance_is_anchored(
            boundary,
            evidence,
            BoundaryPermissionLocalId::new(0),
        ));

        let _unused_static_grant = RootStaticExecutionGrant {
            authorization: RootExecutionAuthorizationRef {
                policy: id!(RootPolicyId, 4),
                local: ExecutionAuthorizationLocalId::new(0),
            },
            scope: StaticExecutionScope {
                kind: FormationRefV2 {
                    snapshot: id!(ProgramSnapshotId, 5),
                    local: crate::identity::FormationLocalId::new(0),
                },
                application: ApplicationId {
                    snapshot: id!(ProgramSnapshotId, 5),
                    local: crate::identity::ApplicationLocalId::new(0),
                },
                mode: ModeId {
                    operator: crate::identity::OperatorRef {
                        snapshot: id!(ProgramSnapshotId, 5),
                        local: crate::identity::OperatorLocalId::new(0),
                    },
                    local: crate::identity::ModeLocalId::new(0),
                },
            },
        };
    }
}
