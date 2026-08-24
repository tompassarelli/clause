//! Typed evaluation projections over one sealed Revision.
//!
//! Requests live outside the semantic Model.  This module evaluates the typed
//! `FindPlan` and projects either the canonical chosen proof or the bounded
//! minimal-support frontier; presentation and result encoding belong to the
//! request layer.
use crate::{
    derive::{Limits, SupportLimits},
    kernel::{PatternId, ReferentId, RelationalContent, Result, Revision, RevisionId, Term},
};
use std::collections::BTreeMap;

mod explain;
mod query;

/// A ground clause in a revision-scoped explanation graph.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ClauseNode {
    pub clause: RelationalContent,
}

/// One canonical witness for a derived or asserted clause.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Witness {
    Asserted,
    Derived {
        rule: ReferentId,
        premises: Vec<usize>,
        substitution: BTreeMap<PatternId, Term>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WitnessEdge {
    pub conclusion: usize,
    pub witness: Witness,
}

/// An acyclic, canonical proof projection.  Node indices address `nodes`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WhyGraph {
    pub root: usize,
    pub nodes: Vec<ClauseNode>,
    pub witnesses: Vec<WitnessEdge>,
}

/// One canonical proof, explicitly scoped to the Revision that admitted it.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Proof {
    pub revision: RevisionId,
    pub why: WhyGraph,
}

/// One inclusion-minimal asserted support and its exact derivation.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WhySupport {
    pub assertions: Vec<RelationalContent>,
    pub proof: Proof,
}

/// The bounded projection of every discovered inclusion-minimal support.
///
/// `complete` is true only when the support engine exhausted the admitted
/// finite search.  An empty, incomplete frontier is intentionally distinct
/// from a complete proof of no support.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WhyAll {
    pub revision: RevisionId,
    pub target: RelationalContent,
    pub alternatives: Vec<WhySupport>,
    pub complete: bool,
    pub expansions: usize,
}

impl WhyAll {
    pub fn alternative_count(&self) -> usize {
        self.alternatives.len()
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }
}

/// Evaluate the complete bounded derived closure and return canonical typed
/// bindings for the sole sought role in `plan`.
///
/// The complete retained pattern is matched role-by-role.  In particular, two
/// otherwise identical orientations with distinct known entities cannot share
/// results merely because their `known` role sets are the same.
pub fn find(
    revision: &Revision,
    plan: &crate::kernel::FindPlan,
    limits: Limits,
) -> Result<Vec<Term>> {
    query::find(revision, plan, limits)
}

/// Return the deterministic chosen proof for a ground target, if it follows.
pub fn why(
    revision: &Revision,
    target: &RelationalContent,
    limits: Limits,
) -> Result<Option<Proof>> {
    explain::why(revision, target, limits)
}

/// Return every discovered minimal asserted support for a ground target.
///
/// The complete closure is checked first, so a bounded support search can
/// honestly return `Some(WhyAll { complete: false, alternatives: [] })` for an
/// entailed target whose support frontier was not reached before its budget.
pub fn why_all(
    revision: &Revision,
    target: &RelationalContent,
    limits: SupportLimits,
) -> Result<Option<WhyAll>> {
    explain::why_all(revision, target, limits)
}

#[cfg(test)]
mod tests;
