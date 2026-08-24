//! Ordered request resolution, evaluation, and canonical projection.
#![allow(unexpected_cfgs)]

use std::collections::{BTreeMap, BTreeSet};

#[cfg(not(clause_generated))]
use crate::elaborate::CompiledProgram;
use crate::{
    derive::{Limits, SupportLimits},
    execution::{Proof, QueryRow, WhyAll},
    intervention::{AchieveAll, AchieveOne, InterventionLimits, PreventAll, PreventOne},
    kernel::{
        self, PatternId, QueryPlanColumn, ReferentId, RelationalContent, Revision, RevisionId,
        RoleId, Term,
    },
    semantic_diff::SemanticDiff,
};

mod canonical_rendering;
mod ordered_execution;
#[cfg(not(clause_generated))]
mod resolution;

/// A request with every source navigation name resolved to a semantic identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Request {
    Any {
        revision: RevisionId,
        pattern: RelationalContent,
    },
    Select {
        revision: RevisionId,
        pattern: RelationalContent,
        columns: Vec<QueryColumn>,
    },
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

/// One source-independent query column. The optional label is presentation;
/// stable role origins retain the column's semantic address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryColumn {
    label: Option<String>,
    binder: PatternId,
    origins: Vec<RoleId>,
}

impl QueryColumn {
    pub fn new(label: Option<String>, binder: PatternId, origins: Vec<RoleId>) -> Self {
        Self {
            label,
            binder,
            origins,
        }
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn binder(&self) -> &PatternId {
        &self.binder
    }

    pub fn origins(&self) -> &[RoleId] {
        &self.origins
    }

    fn plan_column(&self) -> QueryPlanColumn {
        QueryPlanColumn::new(self.binder.clone(), self.origins.clone())
    }
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
            match request {
                Request::Any { revision, pattern } => {
                    let selected = revisions
                        .get(revision)
                        .expect("request Revision presence was validated");
                    let _ = any_plan(selected.model(), pattern)?;
                }
                Request::Select {
                    revision,
                    pattern,
                    columns,
                } => {
                    let selected = revisions
                        .get(revision)
                        .expect("request Revision presence was validated");
                    let _ = kernel::QueryPlan::new(
                        selected.model(),
                        pattern,
                        columns.iter().map(QueryColumn::plan_column).collect(),
                    )?;
                }
                _ => {}
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
            Self::Any { revision, .. }
            | Self::Select { revision, .. }
            | Self::Find { revision, .. }
            | Self::Why { revision, .. }
            | Self::Prevent { revision, .. }
            | Self::Achieve { revision, .. } => vec![revision],
            Self::Diff { base, successor } => vec![base, successor],
        }
    }
}

fn any_plan(
    model: &kernel::Model,
    pattern: &RelationalContent,
) -> kernel::Result<kernel::QueryPlan> {
    let binders = pattern
        .roles()
        .values()
        .filter_map(Term::pattern_id)
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    kernel::QueryPlan::derive(model, pattern, binders)
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

/// Canonical results for an ordered selection of pure definitions in one Revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationOutput {
    revision: RevisionId,
    definitions: Vec<(ReferentId, Term)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestOutput {
    Any(bool),
    Select {
        columns: Vec<QueryColumn>,
        rows: Vec<QueryRow>,
    },
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

impl EvaluationOutput {
    /// Preserve one evaluated result for each uniquely requested definition.
    ///
    /// # Errors
    ///
    /// Returns an error when the same definition identity occurs more than once.
    pub fn new(revision: RevisionId, definitions: Vec<(ReferentId, Term)>) -> kernel::Result<Self> {
        let mut unique = std::collections::BTreeSet::new();
        if definitions
            .iter()
            .any(|(definition, _)| !unique.insert(definition))
        {
            return Err(kernel::KernelError::new(
                "evaluation output cannot contain duplicate definitions",
            ));
        }
        Ok(Self {
            revision,
            definitions,
        })
    }

    /// Exact no-newline JSON bytes in requested definition order.
    pub fn canonical_bytes(&self) -> String {
        canonical_rendering::evaluation_bytes(self)
    }
}

#[cfg(test)]
mod tests;
