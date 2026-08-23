//! Immutable asserted-clause transitions and comparisons between revisions.
//!
//! Delta values remain outside revision identity and persistence. Applying a
//! Delta rebuilds only the asserted-clause set and admits the resulting Model
//! through the canonical semantic wire.

use crate::{
    kernel::{Clause, Delta, KernelError, Result, Revision, RevisionId},
    wire,
};

impl Delta {
    /// Apply this transition atomically to its exact base revision.
    pub fn apply(&self, base: &Revision) -> Result<Revision> {
        if self.base() != base.identity() {
            return Err(KernelError::new("delta base revision does not match"));
        }

        let model = base.model();
        for withdrawal in self.withdrawals() {
            if model.assertions().binary_search(withdrawal).is_err() {
                return Err(KernelError::new("delta withdraws a nonexistent assertion"));
            }
        }
        for admission in self.admissions() {
            if model.assertions().binary_search(admission).is_ok() {
                return Err(KernelError::new("delta admits an existing assertion"));
            }
            model.validate_clause(admission, false)?;
        }

        let mut assertions = model
            .assertions()
            .iter()
            .filter(|assertion| self.withdrawals().binary_search(assertion).is_err())
            .cloned()
            .collect::<Vec<_>>();
        assertions.extend(self.admissions().iter().cloned());
        Ok(wire::admit(model.with_assertions(assertions)?))
    }
}

/// The authored assertion difference between two same-declaration revisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionDiff {
    base_revision: RevisionId,
    successor_revision: RevisionId,
    added: Vec<Clause>,
    removed: Vec<Clause>,
}

impl RevisionDiff {
    /// Compare assertions only when all declarations are identical.
    pub fn between(base: &Revision, successor: &Revision) -> Result<Self> {
        let base_model = base.model();
        let successor_model = successor.model();
        if base_model.id() != successor_model.id()
            || base_model.types() != successor_model.types()
            || base_model.entities() != successor_model.entities()
            || base_model.relations() != successor_model.relations()
            || base_model.laws() != successor_model.laws()
        {
            return Err(KernelError::new(
                "cannot diff revisions with different declarations",
            ));
        }

        let added = successor_model
            .assertions()
            .iter()
            .filter(|assertion| base_model.assertions().binary_search(assertion).is_err())
            .cloned()
            .collect();
        let removed = base_model
            .assertions()
            .iter()
            .filter(|assertion| {
                successor_model
                    .assertions()
                    .binary_search(assertion)
                    .is_err()
            })
            .cloned()
            .collect();

        Ok(Self {
            base_revision: base.identity().clone(),
            successor_revision: successor.identity().clone(),
            added,
            removed,
        })
    }

    pub fn base_revision(&self) -> &RevisionId {
        &self.base_revision
    }

    pub fn successor_revision(&self) -> &RevisionId {
        &self.successor_revision
    }

    pub fn added(&self) -> &[Clause] {
        &self.added
    }

    pub fn removed(&self) -> &[Clause] {
        &self.removed
    }
}
