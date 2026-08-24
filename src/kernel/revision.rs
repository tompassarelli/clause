use super::{
    error::{KernelError, Result},
    identity::RevisionId,
    model::{Model, SemanticAtom},
};

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
