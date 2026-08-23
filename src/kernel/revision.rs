use super::{
    clause::Clause,
    error::{KernelError, Result},
    identity::RevisionId,
    model::Model,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revision {
    identity: RevisionId,
    model: Model,
}

impl Revision {
    /// Wire admission owns semantic hashing and is the only module that pairs
    /// a checked digest with its admitted model.
    pub(crate) fn reloaded(identity: RevisionId, model: Model) -> Self {
        Self { identity, model }
    }

    pub fn identity(&self) -> &RevisionId {
        &self.identity
    }

    pub fn model(&self) -> &Model {
        &self.model
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delta {
    base: RevisionId,
    admissions: Vec<Clause>,
    withdrawals: Vec<Clause>,
}

impl Delta {
    pub fn new(
        base: RevisionId,
        mut admissions: Vec<Clause>,
        mut withdrawals: Vec<Clause>,
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
            .any(|clause| withdrawals.binary_search(clause).is_ok())
        {
            return Err(KernelError::new("delta admissions and withdrawals overlap"));
        }
        if !admissions.iter().chain(&withdrawals).all(Clause::is_ground) {
            return Err(KernelError::new("delta changes must be ground clauses"));
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

    pub fn admissions(&self) -> &[Clause] {
        &self.admissions
    }

    pub fn withdrawals(&self) -> &[Clause] {
        &self.withdrawals
    }
}
