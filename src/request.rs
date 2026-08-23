//! Ordered request resolution, evaluation, and canonical projection.
#![allow(unexpected_cfgs)]

use std::collections::BTreeMap;

use crate::{
    derive::{Limits, SupportLimits},
    execution::{self, Proof, WhyAll},
    intervention::{
        self, AchieveAll, AchieveOne, Incomplete, Intervention, InterventionLimits, PreventAll,
        PreventOne,
    },
    kernel::{self, Clause, Name, RelationId, Revision, RevisionId, Term, VariableId},
    semantic_diff::SemanticDiff,
};
#[cfg(not(clause_generated))]
use crate::{
    elaborate::{self, CompiledProgram},
    frontend,
};

/// A request with every source navigation name resolved to a semantic identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Request {
    Find {
        revision: RevisionId,
        pattern: Clause,
        sought: VariableId,
    },
    Why {
        revision: RevisionId,
        target: Clause,
        all: bool,
    },
    Prevent {
        revision: RevisionId,
        target: Clause,
        selection: Selection,
        using: Vec<RelationId>,
    },
    Achieve {
        revision: RevisionId,
        target: Clause,
        selection: Selection,
        using: Vec<RelationId>,
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
    let mut revisions = BTreeMap::new();
    let mut requests = Vec::with_capacity(program.requests().len());
    for request in program.requests() {
        let resolved = match request {
            frontend::RequestDecl::Find {
                revision,
                pattern,
                sought,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                let pattern = elaborate::lower_clause(revision, pattern)?;
                let sought = variable(&sought.value)?;
                let _ = kernel::FindPlan::new(revision.model(), &pattern, sought.clone())?;
                Request::Find {
                    revision: revision.identity().clone(),
                    pattern,
                    sought,
                }
            }
            frontend::RequestDecl::Why {
                revision,
                target,
                all,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                Request::Why {
                    revision: revision.identity().clone(),
                    target: elaborate::lower_clause(revision, target)?,
                    all: *all,
                }
            }
            frontend::RequestDecl::Prevent {
                revision,
                target,
                selection: requested_selection,
                using,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                Request::Prevent {
                    revision: revision.identity().clone(),
                    target: elaborate::lower_clause(revision, target)?,
                    selection: lower_selection(*requested_selection),
                    using: relations(using)?,
                }
            }
            frontend::RequestDecl::Achieve {
                revision,
                target,
                selection: requested_selection,
                using,
                ..
            } => {
                let revision = program.revision(&revision.value)?;
                Request::Achieve {
                    revision: revision.identity().clone(),
                    target: elaborate::lower_clause(revision, target)?,
                    selection: lower_selection(*requested_selection),
                    using: relations(using)?,
                }
            }
            frontend::RequestDecl::Diff {
                base, successor, ..
            } => Request::Diff {
                base: program.revision(&base.value)?.identity().clone(),
                successor: program.revision(&successor.value)?.identity().clone(),
            },
        };
        for identity in resolved.revisions() {
            revisions.entry(identity.clone()).or_insert_with(|| {
                program
                    .revisions()
                    .values()
                    .find(|revision| revision.identity() == identity)
                    .expect("compiled request revision is registered")
                    .clone()
            });
        }
        requests.push(resolved);
    }
    ResolvedProgram::new(revisions, requests)
}

#[cfg(not(clause_generated))]
fn variable(value: &frontend::VariableName) -> kernel::Result<VariableId> {
    VariableId::new(Name::new(value.0.clone())?)
}

#[cfg(not(clause_generated))]
fn relations(values: &[frontend::Spanned<frontend::Name>]) -> kernel::Result<Vec<RelationId>> {
    values
        .iter()
        .map(|value| RelationId::new(Name::new(value.value.0.clone())?))
        .collect()
}

#[cfg(not(clause_generated))]
fn lower_selection(value: frontend::InterventionSelection) -> Selection {
    match value {
        frontend::InterventionSelection::OneMinimal => Selection::OneMinimal,
        frontend::InterventionSelection::AllMinimal => Selection::AllMinimal,
    }
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
    let mut results = Vec::with_capacity(program.requests.len());
    for request in &program.requests {
        let output = match request {
            Request::Find {
                revision: identity,
                pattern,
                sought,
            } => {
                let selected = revision(program, identity)?;
                RequestOutput::Find(execution::find(
                    selected,
                    &kernel::FindPlan::new(selected.model(), pattern, sought.clone())?,
                    limits.closure,
                )?)
            }
            Request::Why {
                revision: identity,
                target,
                all: false,
            } => RequestOutput::WhyOne(execution::why(
                revision(program, identity)?,
                target,
                limits.closure,
            )?),
            Request::Why {
                revision: identity,
                target,
                all: true,
            } => RequestOutput::WhyAll(execution::why_all(
                revision(program, identity)?,
                target,
                limits.support,
            )?),
            Request::Prevent {
                revision: identity,
                target,
                selection: Selection::OneMinimal,
                using,
            } => RequestOutput::PreventOne(intervention::prevent_one_minimal(
                revision(program, identity)?,
                target.clone(),
                using.clone(),
                limits.intervention,
            )?),
            Request::Prevent {
                revision: identity,
                target,
                selection: Selection::AllMinimal,
                using,
            } => RequestOutput::PreventAll(intervention::prevent_all_minimal(
                revision(program, identity)?,
                target.clone(),
                using.clone(),
                limits.intervention,
            )?),
            Request::Achieve {
                revision: identity,
                target,
                selection: Selection::OneMinimal,
                using,
            } => RequestOutput::AchieveOne(intervention::achieve_one_minimal(
                revision(program, identity)?,
                target.clone(),
                using.clone(),
                limits.intervention,
            )?),
            Request::Achieve {
                revision: identity,
                target,
                selection: Selection::AllMinimal,
                using,
            } => RequestOutput::AchieveAll(intervention::achieve_all_minimal(
                revision(program, identity)?,
                target.clone(),
                using.clone(),
                limits.intervention,
            )?),
            Request::Diff { base, successor } => RequestOutput::Diff(SemanticDiff::between(
                revision(program, base)?,
                revision(program, successor)?,
                limits.support,
            )?),
        };
        results.push(output);
    }
    Ok(RunOutput { results })
}

fn revision<'a>(
    program: &'a ResolvedProgram,
    identity: &RevisionId,
) -> kernel::Result<&'a Revision> {
    program
        .revisions
        .get(identity)
        .ok_or_else(|| kernel::KernelError::new("request Revision is unavailable"))
}

impl RunOutput {
    /// The sole deterministic run transcript. Semantic IDs are rendered only here.
    pub fn canonical_bytes(&self) -> String {
        format!(
            "[\"clause-run-v1\",[{}]]",
            self.results
                .iter()
                .map(RequestOutput::canonical)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl RequestOutput {
    fn canonical(&self) -> String {
        match self {
            Self::Find(items) => format!(
                "[\"find\",[{}]]",
                items.iter().map(term).collect::<Vec<_>>().join(",")
            ),
            Self::WhyOne(chosen) => format!(
                "[\"why\",{}]",
                chosen.as_ref().map(proof).unwrap_or_else(|| "null".into())
            ),
            Self::WhyAll(frontier) => format!(
                "[\"why-all\",{}]",
                frontier
                    .as_ref()
                    .map(why_all)
                    .unwrap_or_else(|| "null".into())
            ),
            Self::PreventOne(result) => format!("[\"prevent-one\",{}]", prevent_one(result)),
            Self::PreventAll(result) => format!("[\"prevent-all\",{}]", prevent_all(result)),
            Self::AchieveOne(result) => format!("[\"achieve-one\",{}]", achieve_one(result)),
            Self::AchieveAll(result) => format!("[\"achieve-all\",{}]", achieve_all(result)),
            Self::Diff(diff) => diff_json(diff),
        }
    }
}

fn string(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('\"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}
fn term(value: &Term) -> String {
    match value {
        Term::Entity(entity) => format!(
            "[\"entity\",{},{},{}]",
            string(entity.model().as_str()),
            string(entity.local().as_str()),
            string(entity.typ().as_str())
        ),
        Term::Value { typ, canonical } => {
            format!("[\"value\",{},{}]", string(typ.as_str()), string(canonical))
        }
        Term::Variable { id, typ } => format!(
            "[\"variable\",{},{}]",
            string(id.as_str()),
            string(typ.as_str())
        ),
    }
}
fn clause(value: &Clause) -> String {
    format!(
        "[\"clause\",{},[{}]]",
        string(value.relation().as_str()),
        value
            .roles()
            .iter()
            .map(|(role, value)| format!("[{},{}]", string(role.as_str()), term(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn proof(value: &Proof) -> String {
    format!(
        "[\"proof\",{},{}]",
        string(&value.revision.to_string()),
        graph(&value.why)
    )
}
fn graph(value: &execution::WhyGraph) -> String {
    format!(
        "[\"graph\",{},[{}],[{}]]",
        value.root,
        value
            .nodes
            .iter()
            .map(|node| clause(&node.clause))
            .collect::<Vec<_>>()
            .join(","),
        value
            .witnesses
            .iter()
            .map(|edge| format!("[{},{}]", edge.conclusion, witness(&edge.witness)))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn witness(value: &execution::Witness) -> String {
    match value {
        execution::Witness::Asserted => "[\"asserted\"]".into(),
        execution::Witness::Derived {
            law,
            premises,
            substitution,
        } => format!(
            "[\"derived\",{},[{}],[{}]]",
            string(law.as_str()),
            premises
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(","),
            substitution
                .iter()
                .map(|(id, value)| format!("[{},{}]", string(id.as_str()), term(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn why_all(value: &WhyAll) -> String {
    format!(
        "[\"why-all\",{},{},{},{},[{}]]",
        string(&value.revision.to_string()),
        clause(&value.target),
        value.complete,
        value.expansions,
        value
            .alternatives
            .iter()
            .map(|item| format!(
                "[[{}],{}]",
                item.assertions
                    .iter()
                    .map(clause)
                    .collect::<Vec<_>>()
                    .join(","),
                proof(&item.proof)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn intervention(value: &Intervention) -> String {
    format!(
        "[\"intervention\",{},[{}],[{}],{},{}]",
        string(&value.revision().identity().to_string()),
        value
            .delta()
            .admissions()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .delta()
            .withdrawals()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        string(&value.delta().base().to_string()),
        value
            .proof()
            .map(derive_proof)
            .unwrap_or_else(|| "null".into())
    )
}
fn derive_proof(value: &crate::derive::Proof) -> String {
    format!(
        "[\"derivation\",{},{}]",
        value.generation(),
        derive_witness(value.witness())
    )
}
fn derive_witness(value: &crate::derive::Witness) -> String {
    match value {
        crate::derive::Witness::Asserted => "[\"asserted\"]".into(),
        crate::derive::Witness::Derived {
            law,
            premises,
            substitution,
        } => format!(
            "[\"derived\",{},[{}],[{}]]",
            string(law.as_str()),
            premises.iter().map(clause).collect::<Vec<_>>().join(","),
            substitution
                .iter()
                .map(|(id, value)| format!("[{},{}]", string(id.as_str()), term(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn support_proof(value: &crate::derive::SupportProof) -> String {
    format!(
        "[\"support-proof\",{},{}]",
        clause(value.conclusion()),
        support_witness(value.witness())
    )
}
fn support_witness(value: &crate::derive::SupportWitness) -> String {
    match value {
        crate::derive::SupportWitness::Asserted => "[\"asserted\"]".into(),
        crate::derive::SupportWitness::Derived {
            law,
            premises,
            substitution,
        } => format!(
            "[\"derived\",{},[{}],[{}]]",
            string(law.as_str()),
            premises
                .iter()
                .map(support_proof)
                .collect::<Vec<_>>()
                .join(","),
            substitution
                .iter()
                .map(|(id, value)| format!("[{},{}]", string(id.as_str()), term(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn support_frontier(value: &crate::derive::SupportFrontier) -> String {
    format!(
        "[\"support-frontier\",{},{},{},{},[{}]]",
        string(&value.revision().to_string()),
        clause(value.target()),
        string(match value.status() {
            crate::derive::SupportStatus::Complete => "complete",
            crate::derive::SupportStatus::ExpansionBudgetExhausted => "expansion-budget-exhausted",
            crate::derive::SupportStatus::SupportBudgetExhausted => "support-budget-exhausted",
        }),
        value.expansions(),
        value
            .supports()
            .iter()
            .map(|support| format!(
                "[[{}],{}]",
                support
                    .assertions()
                    .iter()
                    .map(clause)
                    .collect::<Vec<_>>()
                    .join(","),
                support_proof(support.proof())
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn incomplete(value: Incomplete) -> &'static str {
    match value {
        Incomplete::CandidateBudgetExhausted => "candidate-budget-exhausted",
        Incomplete::SolutionBudgetExhausted => "solution-budget-exhausted",
        Incomplete::ClosureBudgetExhausted => "closure-budget-exhausted",
        Incomplete::SupportExpansionBudgetExhausted => "support-expansion-budget-exhausted",
        Incomplete::SupportBudgetExhausted => "support-budget-exhausted",
    }
}
fn prevent_one(value: &PreventOne) -> String {
    match value {
        PreventOne::Satisfied(item) => format!("[\"satisfied\",{}]", intervention(item)),
        PreventOne::AlreadyAbsent => "[\"already-absent\"]".into(),
        PreventOne::Impossible => "[\"impossible\"]".into(),
        PreventOne::Incomplete(reason) => {
            format!("[\"incomplete\",{}]", string(incomplete(*reason)))
        }
    }
}
fn achieve_one(value: &AchieveOne) -> String {
    match value {
        AchieveOne::Satisfied(item) => format!("[\"satisfied\",{}]", intervention(item)),
        AchieveOne::AlreadyEntailed => "[\"already-entailed\"]".into(),
        AchieveOne::Impossible => "[\"impossible\"]".into(),
        AchieveOne::Incomplete(reason) => {
            format!("[\"incomplete\",{}]", string(incomplete(*reason)))
        }
    }
}
fn prevent_all(value: &PreventAll) -> String {
    match value {
        PreventAll::Complete(items) => format!(
            "[\"complete\",[{}]]",
            items.iter().map(intervention).collect::<Vec<_>>().join(",")
        ),
        PreventAll::AlreadyAbsent => "[\"already-absent\"]".into(),
        PreventAll::Impossible => "[\"impossible\"]".into(),
        PreventAll::Incomplete {
            interventions,
            reason,
        } => format!(
            "[\"incomplete\",{},[{}]]",
            string(incomplete(*reason)),
            interventions
                .iter()
                .map(intervention)
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn achieve_all(value: &AchieveAll) -> String {
    match value {
        AchieveAll::Complete(items) => format!(
            "[\"complete\",[{}]]",
            items.iter().map(intervention).collect::<Vec<_>>().join(",")
        ),
        AchieveAll::AlreadyEntailed => "[\"already-entailed\"]".into(),
        AchieveAll::Impossible => "[\"impossible\"]".into(),
        AchieveAll::Incomplete {
            interventions,
            reason,
        } => format!(
            "[\"incomplete\",{},[{}]]",
            string(incomplete(*reason)),
            interventions
                .iter()
                .map(intervention)
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
fn diff_json(value: &SemanticDiff) -> String {
    format!(
        "[\"diff\",[\"authored\",{},{},[{}],[{}]],[\"entailed-added\",[{}]],[\"entailed-removed\",[{}]],[\"proof-changes\",[{}]],[\"support-changes\",[{}]]]",
        string(&value.authored().base_revision().to_string()),
        string(&value.authored().successor_revision().to_string()),
        value
            .authored()
            .added()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .authored()
            .removed()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .entailed_added()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .entailed_removed()
            .iter()
            .map(clause)
            .collect::<Vec<_>>()
            .join(","),
        value
            .changed_proofs()
            .iter()
            .map(|change| format!(
                "[{},{},{}]",
                clause(change.consequence()),
                derive_proof(change.base()),
                derive_proof(change.successor())
            ))
            .collect::<Vec<_>>()
            .join(","),
        value
            .changed_supports()
            .iter()
            .map(|change| format!(
                "[{},{},{},[{}],[{}],[{}]]",
                clause(change.consequence()),
                support_frontier(change.base()),
                support_frontier(change.successor()),
                change
                    .added()
                    .iter()
                    .map(|support| support_proof(support.proof()))
                    .collect::<Vec<_>>()
                    .join(","),
                change
                    .removed()
                    .iter()
                    .map(|support| support_proof(support.proof()))
                    .collect::<Vec<_>>()
                    .join(","),
                change
                    .retained()
                    .iter()
                    .map(|support| support_proof(support.proof()))
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::{Request, RequestOutput, RunLimits, Selection, resolve, run};
    use crate::{elaborate, frontend};

    const SOURCE: &str = "Item: Type
link: Relation
    {left: Item} links {right: Item}
    mode left -> right: many
graph: Model
    A: Item
    B: Item
    A links B
graph/add: Revision
    from: graph
    admit:
        B links A
find all ?right in graph:
    A links ?right
why in graph:
    A links B
diff graph -> graph/add
";

    const INTERVENTIONS: &str = "Item: Type
link: Relation
    {left: Item} links {right: Item}
    mode left -> right: many
graph: Model
    A: Item
    B: Item
    A links B
prevent one minimal in graph:
    A links B
using:
    link
prevent all minimal in graph:
    A links B
using:
    link
achieve one minimal in graph:
    B links A
using:
    link
achieve all minimal in graph:
    B links A
using:
    link
";

    fn program(source: &str) -> super::ResolvedProgram {
        resolve(&elaborate::compile(frontend::parse(source).unwrap()).unwrap()).unwrap()
    }

    #[test]
    fn resolves_typed_requests_in_authored_order_and_encodes_one_aggregate() {
        let program = program(SOURCE);
        assert!(matches!(
            program.requests(),
            [
                Request::Find { .. },
                Request::Why { all: false, .. },
                Request::Diff { .. }
            ]
        ));
        let output = run(&program, RunLimits::default()).unwrap();
        assert!(matches!(
            output.results.as_slice(),
            [
                RequestOutput::Find(_),
                RequestOutput::WhyOne(_),
                RequestOutput::Diff(_)
            ]
        ));
        assert_eq!(output.canonical_bytes().matches("[\"find\"").count(), 1);
        assert!(output.canonical_bytes().starts_with("[\"clause-run-v1\","));
    }

    #[test]
    fn dispatches_one_and_all_intervention_contracts() {
        let program = program(INTERVENTIONS);
        assert!(matches!(
            program.requests(),
            [
                Request::Prevent {
                    selection: Selection::OneMinimal,
                    ..
                },
                Request::Prevent {
                    selection: Selection::AllMinimal,
                    ..
                },
                Request::Achieve {
                    selection: Selection::OneMinimal,
                    ..
                },
                Request::Achieve {
                    selection: Selection::AllMinimal,
                    ..
                },
            ]
        ));
        let output = run(&program, RunLimits::default()).unwrap();
        assert!(matches!(
            output.results.as_slice(),
            [
                RequestOutput::PreventOne(_),
                RequestOutput::PreventAll(_),
                RequestOutput::AchieveOne(_),
                RequestOutput::AchieveAll(_),
            ]
        ));
    }
}
