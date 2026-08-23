//! Certified finite intervention synthesis over typed, sealed revisions.
//!
//! `one minimal` and `all minimal` are deliberately separate contracts. A
//! one-result request proves inclusion minimality by exact counterfactual
//! closure checks; it makes no claim about cardinality optimality or the
//! complete frontier. An all-result request is complete only after the finite
//! candidate space has been exhausted.
//!
//! The stable public surface stays at clause::intervention. Private modules
//! separate one-result certification, exhaustive frontier search, candidate
//! construction, and shared bounded closure mechanics.
mod all;
mod basis;
mod closure;
mod one;
mod search;

use crate::{
    derive::{Limits, Proof, SupportLimits},
    kernel::{Clause, Delta, RelationId, Result, Revision},
};

/// Explicit resource bounds for an intervention request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterventionLimits {
    closure: Limits,
    max_candidates: usize,
    max_solutions: usize,
    support: SupportLimits,
}

impl InterventionLimits {
    pub fn new(closure: Limits, max_candidates: usize, max_solutions: usize) -> Self {
        Self {
            closure,
            max_candidates,
            max_solutions,
            support: SupportLimits::new(closure, max_candidates, max_solutions),
        }
    }

    pub fn with_support_limits(mut self, support: SupportLimits) -> Self {
        self.support = support;
        self
    }

    pub fn closure(&self) -> Limits {
        self.closure
    }

    pub fn max_candidates(&self) -> usize {
        self.max_candidates
    }

    pub fn max_solutions(&self) -> usize {
        self.max_solutions
    }

    pub fn support(&self) -> SupportLimits {
        self.support
    }
}

/// A verified Delta and the only successor Revision it admits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intervention {
    delta: Delta,
    revision: Revision,
    proof: Option<Proof>,
}

impl Intervention {
    pub fn delta(&self) -> &Delta {
        &self.delta
    }

    pub fn revision(&self) -> &Revision {
        &self.revision
    }

    pub fn proof(&self) -> Option<&Proof> {
        self.proof.as_ref()
    }

    fn withdrawal(source: &Revision, withdrawals: Vec<Clause>, revision: Revision) -> Result<Self> {
        Ok(Self {
            delta: Delta::new(source.identity().clone(), Vec::new(), withdrawals)?,
            revision,
            proof: None,
        })
    }

    fn admission(
        source: &Revision,
        admissions: Vec<Clause>,
        revision: Revision,
        proof: Proof,
    ) -> Result<Self> {
        Ok(Self {
            delta: Delta::new(source.identity().clone(), admissions, Vec::new())?,
            revision,
            proof: Some(proof),
        })
    }
}

/// A result that may be sound but cannot make a stronger certification claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Incomplete {
    CandidateBudgetExhausted,
    SolutionBudgetExhausted,
    ClosureBudgetExhausted,
    SupportExpansionBudgetExhausted,
    SupportBudgetExhausted,
}

/// Exact outcome for `prevent one minimal`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreventOne {
    Satisfied(Box<Intervention>),
    AlreadyAbsent,
    Impossible,
    Incomplete(Incomplete),
}

/// Exact outcome for `achieve one minimal`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AchieveOne {
    Satisfied(Box<Intervention>),
    AlreadyEntailed,
    Impossible,
    Incomplete(Incomplete),
}

/// Exhaustive finite prevention output. Results retained on an incomplete
/// search are individually verified but are not a complete frontier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreventAll {
    Complete(Vec<Intervention>),
    AlreadyAbsent,
    Impossible,
    Incomplete {
        interventions: Vec<Intervention>,
        reason: Incomplete,
    },
}

impl PreventAll {
    pub fn interventions(&self) -> &[Intervention] {
        match self {
            Self::Complete(items)
            | Self::Incomplete {
                interventions: items,
                ..
            } => items,
            Self::AlreadyAbsent | Self::Impossible => &[],
        }
    }
}

/// Exhaustive finite achievement output. Results retained on an incomplete
/// search are individually verified but are not a complete frontier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AchieveAll {
    Complete(Vec<Intervention>),
    AlreadyEntailed,
    Impossible,
    Incomplete {
        interventions: Vec<Intervention>,
        reason: Incomplete,
    },
}

impl AchieveAll {
    pub fn interventions(&self) -> &[Intervention] {
        match self {
            Self::Complete(items)
            | Self::Incomplete {
                interventions: items,
                ..
            } => items,
            Self::AlreadyEntailed | Self::Impossible => &[],
        }
    }
}

/// Return one canonical inclusion-minimal asserted-clause withdrawal.
///
/// The deletion/restoration algorithm is valid only because the admitted law
/// fragment is positive and monotone. It proves each retained withdrawal is
/// necessary, but intentionally does not prove it has minimum cardinality.
pub fn prevent_one_minimal(
    source: &Revision,
    target: Clause,
    using: Vec<RelationId>,
    limits: InterventionLimits,
) -> Result<PreventOne> {
    one::prevent_one_minimal(source, target, using, limits)
}

/// Return one canonical inclusion-minimal asserted-clause admission.
///
/// This is the dual of [`prevent_one_minimal`]. It proves subset necessity,
/// not cardinality optimality or complete-frontier enumeration.
pub fn achieve_one_minimal(
    source: &Revision,
    target: Clause,
    using: Vec<RelationId>,
    limits: InterventionLimits,
) -> Result<AchieveOne> {
    one::achieve_one_minimal(source, target, using, limits)
}

/// Enumerate every inclusion-minimal withdrawal over the complete support
/// frontier.
pub fn prevent_all_minimal(
    source: &Revision,
    target: Clause,
    using: Vec<RelationId>,
    limits: InterventionLimits,
) -> Result<PreventAll> {
    all::prevent_all_minimal(source, target, using, limits)
}

/// Enumerate every inclusion-minimal addition over the finite typed basis.
pub fn achieve_all_minimal(
    source: &Revision,
    target: Clause,
    using: Vec<RelationId>,
    limits: InterventionLimits,
) -> Result<AchieveAll> {
    all::achieve_all_minimal(source, target, using, limits)
}

#[cfg(test)]
use crate::kernel::Term;
#[cfg(test)]
use basis::achievement_basis;
#[cfg(test)]
use search::{Enumeration, enumerate};
#[cfg(test)]
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "intervention/tests.rs"]
mod tests;
