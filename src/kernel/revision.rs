use super::{
    error::{KernelError, Result},
    identity::{
        ClauseSemanticsId, ProgramChangeOccurrenceId, ProgramId, ProgramRevisionId,
        ProgramSnapshotId, ReferentId, RevisionId,
    },
    model::{Model, SemanticAtom},
};

/// Canonical endpoint difference between two checked program snapshots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramDelta {
    admissions: Vec<SemanticAtom>,
    withdrawals: Vec<SemanticAtom>,
}

impl ProgramDelta {
    pub fn new(
        mut admissions: Vec<SemanticAtom>,
        mut withdrawals: Vec<SemanticAtom>,
    ) -> Result<Self> {
        admissions.sort();
        withdrawals.sort();
        if admissions.windows(2).any(|p| p[0] == p[1])
            || withdrawals.windows(2).any(|p| p[0] == p[1])
        {
            return Err(KernelError::new(
                "program delta changes cannot contain duplicates",
            ));
        }
        if admissions
            .iter()
            .any(|a| withdrawals.binary_search(a).is_ok())
        {
            return Err(KernelError::new(
                "program delta admissions and withdrawals overlap",
            ));
        }
        Ok(Self {
            admissions,
            withdrawals,
        })
    }
    pub fn admissions(&self) -> &[SemanticAtom] {
        &self.admissions
    }
    pub fn withdrawals(&self) -> &[SemanticAtom] {
        &self.withdrawals
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramChangeOccurrence {
    identity: ProgramChangeOccurrenceId,
    semantics: ClauseSemanticsId,
    program: ProgramId,
    predecessor: Option<ProgramRevisionId>,
    snapshot: ProgramSnapshotId,
    endpoint_delta: ProgramDelta,
    responsible: ReferentId,
    provenance: Vec<ReferentId>,
}

impl ProgramChangeOccurrence {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identity: ProgramChangeOccurrenceId,
        semantics: ClauseSemanticsId,
        program: ProgramId,
        predecessor: Option<ProgramRevisionId>,
        snapshot: ProgramSnapshotId,
        endpoint_delta: ProgramDelta,
        responsible: ReferentId,
        mut provenance: Vec<ReferentId>,
    ) -> Result<Self> {
        provenance.sort();
        if provenance.windows(2).any(|p| p[0] == p[1]) {
            return Err(KernelError::new(
                "program change provenance cannot contain duplicates",
            ));
        }
        Ok(Self {
            identity,
            semantics,
            program,
            predecessor,
            snapshot,
            endpoint_delta,
            responsible,
            provenance,
        })
    }
    pub fn identity(&self) -> &ProgramChangeOccurrenceId {
        &self.identity
    }
    pub fn semantics(&self) -> &ClauseSemanticsId {
        &self.semantics
    }
    pub fn program(&self) -> &ProgramId {
        &self.program
    }
    pub fn predecessor(&self) -> Option<&ProgramRevisionId> {
        self.predecessor.as_ref()
    }
    pub fn snapshot(&self) -> &ProgramSnapshotId {
        &self.snapshot
    }
    pub fn endpoint_delta(&self) -> &ProgramDelta {
        &self.endpoint_delta
    }
    pub fn responsible(&self) -> &ReferentId {
        &self.responsible
    }
    pub fn provenance(&self) -> &[ReferentId] {
        &self.provenance
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramRevision {
    identity: ProgramRevisionId,
    program: ProgramId,
    semantics: ClauseSemanticsId,
    predecessor: Option<ProgramRevisionId>,
    snapshot: ProgramSnapshot,
    change_occurrence: ProgramChangeOccurrenceId,
}

impl ProgramRevision {
    pub fn constitute_root(
        program: ProgramId,
        snapshot: ProgramSnapshot,
        change: &ProgramChangeOccurrence,
    ) -> Result<Self> {
        let expected: Vec<_> = snapshot.checked_payload().atoms().into_iter().collect();
        if change.program() != &program
            || change.semantics() != snapshot.semantics()
            || change.predecessor().is_some()
            || change.snapshot() != snapshot.identity()
            || !change.endpoint_delta().withdrawals().is_empty()
            || change.endpoint_delta().admissions() != expected.as_slice()
        {
            return Err(KernelError::new(
                "root change occurrence does not match snapshot",
            ));
        }
        Self::build(program, snapshot, None, change)
    }
    pub fn constitute_successor(
        predecessor: &ProgramRevision,
        snapshot: ProgramSnapshot,
        change: &ProgramChangeOccurrence,
    ) -> Result<Self> {
        if change.program() != predecessor.program()
            || change.semantics() != predecessor.semantics()
            || change.semantics() != snapshot.semantics()
            || change.predecessor() != Some(predecessor.identity())
            || change.snapshot() != snapshot.identity()
        {
            return Err(KernelError::new(
                "successor change occurrence metadata mismatch",
            ));
        }
        let mut atoms = predecessor.snapshot.checked_payload().atoms();
        for a in change.endpoint_delta().withdrawals() {
            if !atoms.remove(a) {
                return Err(KernelError::new("program delta withdraws absent atom"));
            }
        }
        for a in change.endpoint_delta().admissions() {
            if !atoms.insert(a.clone()) {
                return Err(KernelError::new("program delta admits existing atom"));
            }
        }
        if atoms != snapshot.checked_payload().atoms() {
            return Err(KernelError::new(
                "program delta does not produce endpoint snapshot",
            ));
        }
        Self::build(
            predecessor.program.clone(),
            snapshot,
            Some(predecessor.identity.clone()),
            change,
        )
    }
    fn build(
        program: ProgramId,
        snapshot: ProgramSnapshot,
        predecessor: Option<ProgramRevisionId>,
        change: &ProgramChangeOccurrence,
    ) -> Result<Self> {
        let identity = crate::wire::program_revision_id(
            &program,
            snapshot.semantics(),
            predecessor.as_ref(),
            snapshot.identity(),
            change.identity(),
        );
        Ok(Self {
            identity,
            program,
            semantics: snapshot.semantics.clone(),
            predecessor,
            snapshot,
            change_occurrence: change.identity.clone(),
        })
    }
    pub fn identity(&self) -> &ProgramRevisionId {
        &self.identity
    }
    pub fn program(&self) -> &ProgramId {
        &self.program
    }
    pub fn semantics(&self) -> &ClauseSemanticsId {
        &self.semantics
    }
    pub fn predecessor(&self) -> Option<&ProgramRevisionId> {
        self.predecessor.as_ref()
    }
    pub fn snapshot(&self) -> &ProgramSnapshot {
        &self.snapshot
    }
    pub fn change_occurrence(&self) -> &ProgramChangeOccurrenceId {
        &self.change_occurrence
    }
}

/// Typed seam for the exact checked program content selected by a revision.
/// Construction and hashing remain in the wire/admission layer during the
/// migration; this value deliberately carries no lineage or mutable evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramSnapshot {
    identity: ProgramSnapshotId,
    semantics: ClauseSemanticsId,
    model: Model,
}

impl ProgramSnapshot {
    pub(crate) fn from_parts(
        identity: ProgramSnapshotId,
        semantics: ClauseSemanticsId,
        model: Model,
    ) -> Self {
        Self {
            identity,
            semantics,
            model,
        }
    }

    pub fn identity(&self) -> &ProgramSnapshotId {
        &self.identity
    }
    pub fn semantics(&self) -> &ClauseSemanticsId {
        &self.semantics
    }
    pub fn checked_payload(&self) -> &Model {
        &self.model
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevisionLineage {
    Root,
    Successor(Delta),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revision {
    identity: RevisionId,
    lineage: RevisionLineage,
    model: Model,
}

impl Revision {
    /// Wire admission owns hashing and is the only module that can pair a
    /// canonical lineage/model payload with its content-derived identity.
    pub(crate) fn reloaded(identity: RevisionId, lineage: RevisionLineage, model: Model) -> Self {
        Self {
            identity,
            lineage,
            model,
        }
    }

    pub fn identity(&self) -> &RevisionId {
        &self.identity
    }
    pub fn lineage(&self) -> &RevisionLineage {
        &self.lineage
    }
    pub fn predecessor(&self) -> Option<&RevisionId> {
        match &self.lineage {
            RevisionLineage::Root => None,
            RevisionLineage::Successor(delta) => Some(delta.base()),
        }
    }
    pub fn delta(&self) -> Option<&Delta> {
        match &self.lineage {
            RevisionLineage::Root => None,
            RevisionLineage::Successor(delta) => Some(delta),
        }
    }
    pub fn model(&self) -> &Model {
        &self.model
    }
}

/// One exact signed semantic edge from an immutable predecessor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delta {
    base: RevisionId,
    admissions: Vec<SemanticAtom>,
    withdrawals: Vec<SemanticAtom>,
}

impl Delta {
    pub fn new(
        base: RevisionId,
        mut admissions: Vec<SemanticAtom>,
        mut withdrawals: Vec<SemanticAtom>,
    ) -> Result<Self> {
        if admissions.is_empty() && withdrawals.is_empty() {
            return Err(KernelError::new("delta needs an admission or withdrawal"));
        }
        admissions.sort();
        withdrawals.sort();
        if admissions.windows(2).any(|pair| pair[0] == pair[1])
            || withdrawals.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(KernelError::new("delta changes cannot contain duplicates"));
        }
        if admissions
            .iter()
            .any(|atom| withdrawals.binary_search(atom).is_ok())
        {
            return Err(KernelError::new("delta admissions and withdrawals overlap"));
        }
        Ok(Self {
            base,
            admissions,
            withdrawals,
        })
    }

    pub fn base(&self) -> &RevisionId {
        &self.base
    }
    pub fn admissions(&self) -> &[SemanticAtom] {
        &self.admissions
    }
    pub fn withdrawals(&self) -> &[SemanticAtom] {
        &self.withdrawals
    }
}
