use super::{
    error::{KernelError, Result},
    identity::{ClauseSemanticsId, ProgramSnapshotId, RevisionId},
    model::{Model, SemanticAtom},
};

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
