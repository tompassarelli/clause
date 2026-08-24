//! Ordered request resolution, evaluation, and canonical projection.
#![allow(unexpected_cfgs)]

use std::collections::BTreeMap;

#[cfg(not(clause_generated))]
use crate::elaborate::CompiledProgram;
use crate::{
    derive::{Limits, SupportLimits},
    execution::{Proof, WhyAll},
    intervention::{AchieveAll, AchieveOne, InterventionLimits, PreventAll, PreventOne},
    kernel::{self, PatternId, ReferentId, RelationalContent, Revision, RevisionId, Term},
    semantic_diff::SemanticDiff,
};

mod canonical_rendering;
mod ordered_execution;
#[cfg(not(clause_generated))]
mod resolution;

/// A request with every source navigation name resolved to a semantic identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Request {
    Find {
        revision: RevisionId,
        pattern: RelationalContent,
        sought: PatternId,
    },
    Why {
        revision: RevisionId,
        target: RelationalContent,
        all: bool,
    },
    Prevent {
        revision: RevisionId,
        target: RelationalContent,
        selection: Selection,
        using: Vec<ReferentId>,
    },
    Achieve {
        revision: RevisionId,
        target: RelationalContent,
        selection: Selection,
        using: Vec<ReferentId>,
    },
    Diff {
        base: RevisionId,
        successor: RevisionId,
    },
}

/// The requested intervention termination contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Selection {
    OneMinimal,
    AllMinimal,
}

/// A source-independent ordered program and the exact revisions it references.
#[derive(Clone, Debug)]
pub struct ResolvedProgram {
    revisions: BTreeMap<RevisionId, Revision>,
    requests: Vec<Request>,
}

impl ResolvedProgram {
    pub fn new(
        revisions: BTreeMap<RevisionId, Revision>,
        requests: Vec<Request>,
    ) -> kernel::Result<Self> {
        if revisions
            .iter()
            .any(|(identity, revision)| identity != revision.identity())
        {
            return Err(kernel::KernelError::new(
                "Revision registry key must match sealed Revision identity",
            ));
        }
        for revision in revisions.values() {
            let Some(delta) = revision.delta() else {
                continue;
            };
            let predecessor = revisions.get(delta.base()).ok_or_else(|| {
                kernel::KernelError::new("Revision registry is missing an exact predecessor")
            })?;
            let expected =
                crate::wire::admit_successor(predecessor, revision.model().clone(), delta.clone())?;
            if expected != *revision {
                return Err(kernel::KernelError::new(
                    "Revision registry contains an inexact successor edge",
                ));
            }
        }
        for request in &requests {
            for revision in request.revisions() {
                if !revisions.contains_key(revision) {
                    return Err(kernel::KernelError::new(
                        "request references an unavailable Revision",
                    ));
                }
            }
        }
        Ok(Self {
            revisions,
            requests,
        })
    }

    pub fn revisions(&self) -> &BTreeMap<RevisionId, Revision> {
        &self.revisions
    }
    pub fn requests(&self) -> &[Request] {
        &self.requests
    }
}

impl Request {
    pub fn revisions(&self) -> Vec<&RevisionId> {
        match self {
            Self::Find { revision, .. }
            | Self::Why { revision, .. }
            | Self::Prevent { revision, .. }
            | Self::Achieve { revision, .. } => vec![revision],
            Self::Diff { base, successor } => vec![base, successor],
        }
    }
}

/// Explicit resource bounds for the semantic engines selected by requests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunLimits {
    pub closure: Limits,
    pub support: SupportLimits,
    pub intervention: InterventionLimits,
}

impl Default for RunLimits {
    fn default() -> Self {
        let closure = Limits::new(100, 10, 10_000);
        Self {
            support: SupportLimits::new(closure, 100, 100),
            intervention: InterventionLimits::new(closure, 100, 100),
            closure,
        }
    }
}

/// Resolve every authored request in source order. No request is sorted or regrouped.
#[cfg(not(clause_generated))]
pub fn resolve(program: &CompiledProgram) -> kernel::Result<ResolvedProgram> {
    resolution::resolve(program)
}

/// One result per authored request, retained in exact source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunOutput {
    pub results: Vec<RequestOutput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestOutput {
    Find(Vec<Term>),
    WhyOne(Option<Proof>),
    WhyAll(Option<WhyAll>),
    PreventOne(PreventOne),
    PreventAll(PreventAll),
    AchieveOne(AchieveOne),
    AchieveAll(AchieveAll),
    Diff(SemanticDiff),
}

/// Evaluate requests once, in their authored order, using only the selected semantic engine.
pub fn run(program: &ResolvedProgram, limits: RunLimits) -> kernel::Result<RunOutput> {
    ordered_execution::run(program, limits)
}

impl RunOutput {
    /// The sole deterministic run transcript. Semantic IDs are rendered only here.
    pub fn canonical_bytes(&self) -> String {
        canonical_rendering::canonical_bytes(self)
    }
}

#[cfg(test)]
mod tests;
