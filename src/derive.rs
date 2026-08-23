//! Deterministic finite closure for admitted positive laws.

use crate::kernel::{Clause, KernelError, RevisionId, Term, VariableId};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub max_assertions: usize,
    pub max_rounds: usize,
    pub max_join_attempts: usize,
}

impl Limits {
    pub fn new(max_assertions: usize, max_rounds: usize, max_join_attempts: usize) -> Self {
        Self {
            max_assertions,
            max_rounds,
            max_join_attempts,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    generation: usize,
    witness: Witness,
}

impl Proof {
    pub fn generation(&self) -> usize {
        self.generation
    }
    pub fn witness(&self) -> &Witness {
        &self.witness
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Witness {
    Asserted,
    Derived {
        law: crate::kernel::LawId,
        premises: Vec<Clause>,
        substitution: BTreeMap<VariableId, Term>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closure {
    assertions: Vec<Clause>,
    proofs: BTreeMap<Clause, Proof>,
}

impl Closure {
    pub fn assertions(&self) -> &[Clause] {
        &self.assertions
    }
    pub fn proof(&self, clause: &Clause) -> Option<&Proof> {
        self.proofs.get(clause)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupportLimits {
    pub closure: Limits,
    pub max_expansions: usize,
    pub max_supports_per_clause: usize,
}

impl SupportLimits {
    pub fn new(closure: Limits, max_expansions: usize, max_supports_per_clause: usize) -> Self {
        Self {
            closure,
            max_expansions,
            max_supports_per_clause,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupportStatus {
    Complete,
    ExpansionBudgetExhausted,
    SupportBudgetExhausted,
}

impl SupportStatus {
    pub fn is_complete(self) -> bool {
        self == Self::Complete
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SupportProof {
    conclusion: Clause,
    witness: SupportWitness,
}

impl SupportProof {
    pub fn conclusion(&self) -> &Clause {
        &self.conclusion
    }
    pub fn witness(&self) -> &SupportWitness {
        &self.witness
    }

    fn contains(&self, clause: &Clause) -> bool {
        self.conclusion == *clause
            || match &self.witness {
                SupportWitness::Asserted => false,
                SupportWitness::Derived { premises, .. } => {
                    premises.iter().any(|premise| premise.contains(clause))
                }
            }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SupportWitness {
    Asserted,
    Derived {
        law: crate::kernel::LawId,
        premises: Vec<SupportProof>,
        substitution: BTreeMap<VariableId, Term>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Support {
    assertion_key: Vec<Clause>,
    assertions: Vec<Clause>,
    proof: SupportProof,
}

impl Support {
    pub fn assertions(&self) -> &[Clause] {
        &self.assertions
    }
    pub fn proof(&self) -> &SupportProof {
        &self.proof
    }

    pub(crate) fn assertion_key(&self) -> &[Clause] {
        &self.assertion_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportFrontier {
    revision: RevisionId,
    target: Clause,
    limits: SupportLimits,
    status: SupportStatus,
    expansions: usize,
    supports: Vec<Support>,
}

impl SupportFrontier {
    pub fn revision(&self) -> &RevisionId {
        &self.revision
    }
    pub fn target(&self) -> &Clause {
        &self.target
    }
    pub fn limits(&self) -> SupportLimits {
        self.limits
    }
    pub fn status(&self) -> SupportStatus {
        self.status
    }
    pub fn expansions(&self) -> usize {
        self.expansions
    }
    pub fn supports(&self) -> &[Support] {
        &self.supports
    }
}

mod closure;
mod matching;
mod support;

pub use closure::saturate;
pub use support::support_frontier;

fn limit_error(kind: &str, name: &str, value: usize) -> KernelError {
    KernelError::new(format!("closure {kind} limit exceeded ({name}={value})"))
}

#[cfg(test)]
#[path = "derive/tests.rs"]
mod tests;
