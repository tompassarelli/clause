//! Semantic comparisons of immutable revisions.
//!
//! A semantic diff is deliberately a comparison value only: it is never part
//! of a revision's admitted model or identity.
use crate::{
    delta::RevisionDiff,
    derive::{self, Proof, Support, SupportFrontier, SupportLimits},
    kernel::{Clause, Result, Revision},
};

mod entailment;
mod proofs;
mod supports;

/// A selected derivation that changed for a consequence entailed by both revisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofChange {
    consequence: Clause,
    base: Proof,
    successor: Proof,
}

impl ProofChange {
    pub fn consequence(&self) -> &Clause {
        &self.consequence
    }

    pub fn base(&self) -> &Proof {
        &self.base
    }

    pub fn successor(&self) -> &Proof {
        &self.successor
    }
}

/// Canonical minimal asserted supports that changed for one consequence.
///
/// The frontiers remain attached to make their bounds and completeness explicit:
/// an incomplete frontier is a deterministic prefix, not a claim that no other
/// support exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportChange {
    consequence: Clause,
    base: SupportFrontier,
    successor: SupportFrontier,
    added: Vec<Support>,
    removed: Vec<Support>,
    retained: Vec<Support>,
}

impl SupportChange {
    pub fn consequence(&self) -> &Clause {
        &self.consequence
    }

    pub fn base(&self) -> &SupportFrontier {
        &self.base
    }

    pub fn successor(&self) -> &SupportFrontier {
        &self.successor
    }

    pub fn added(&self) -> &[Support] {
        &self.added
    }

    pub fn removed(&self) -> &[Support] {
        &self.removed
    }

    /// Supports witnessed in both frontiers by the same asserted-clause set.
    ///
    /// This is positive evidence only: an incomplete frontier does not claim
    /// that these are every retained support.
    pub fn retained(&self) -> &[Support] {
        &self.retained
    }
}

/// The authored and entailed differences between same-declaration revisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticDiff {
    authored: RevisionDiff,
    entailed_added: Vec<Clause>,
    entailed_removed: Vec<Clause>,
    changed_proofs: Vec<ProofChange>,
    changed_supports: Vec<SupportChange>,
}

impl SemanticDiff {
    /// Compare exact immutable revisions with explicit closure resource bounds.
    ///
    /// `authored` describes asserted changes. Entailed additions and removals
    /// exclude those asserted changes, leaving only their semantic
    /// consequences. Chosen proofs are compared only for clauses entailed by
    /// both revisions. Support changes cover the canonical union of both
    /// closures, including appearing and disappearing consequences.
    pub fn between(
        base: &Revision,
        successor: &Revision,
        support_limits: SupportLimits,
    ) -> Result<Self> {
        let authored = RevisionDiff::between(base, successor)?;
        let base_closure = derive::saturate(base, support_limits.closure)?;
        let successor_closure = derive::saturate(successor, support_limits.closure)?;
        let (entailed_added, entailed_removed) =
            entailment::changes(&base_closure, &successor_closure, &authored);
        let changed_proofs = proofs::changes(&base_closure, &successor_closure);
        let changed_supports = supports::changes(
            base,
            successor,
            &base_closure,
            &successor_closure,
            &authored,
            support_limits,
        )?;
        Ok(Self {
            authored,
            entailed_added,
            entailed_removed,
            changed_proofs,
            changed_supports,
        })
    }

    pub fn authored(&self) -> &RevisionDiff {
        &self.authored
    }

    pub fn entailed_added(&self) -> &[Clause] {
        &self.entailed_added
    }

    pub fn entailed_removed(&self) -> &[Clause] {
        &self.entailed_removed
    }

    pub fn changed_proofs(&self) -> &[ProofChange] {
        &self.changed_proofs
    }

    pub fn changed_supports(&self) -> &[SupportChange] {
        &self.changed_supports
    }
}

#[cfg(test)]
mod tests;
