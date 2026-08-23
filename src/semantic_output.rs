//! One canonical projection of a bounded semantic journey.

use crate::{
    delta::RevisionDelta,
    derive::{self, Limits, Support, SupportFrontier, SupportLimits, SupportStatus},
    execution::{self, QueryOutput, WhyAll},
    intervention::{
        self, AchieveConfig, AchieveResult, PreventLimits, PreventReport, PreventStatus,
    },
    kernel::{Clause, KernelError, Result, Revision, Term},
    semantic_diff::{SemanticDiff, SupportChange},
};
use std::collections::BTreeSet;
use std::fmt::Write;

/// The resolved requests and immutable comparison revision for one journey.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticJourney {
    support_loss: Revision,
    support_target: Clause,
    prevent_limits: PreventLimits,
    achievement_goal: Clause,
    achieve_config: AchieveConfig,
    query_limits: Limits,
}

impl SemanticJourney {
    pub fn new(
        support_loss: Revision,
        support_target: Clause,
        prevent_limits: PreventLimits,
        achievement_goal: Clause,
        achieve_config: AchieveConfig,
        query_limits: Limits,
    ) -> Self {
        Self {
            support_loss,
            support_target,
            prevent_limits,
            achievement_goal,
            achieve_config,
            query_limits,
        }
    }

    /// Resolve the flagship journey from generic query, support, delta, and
    /// intervention APIs. The successor identifies the newly achieved query
    /// result; all rendered semantics are evaluated from `base`.
    pub fn from_successor(base: &Revision, successor: &Revision, limits: Limits) -> Result<Self> {
        let base_query = query(base, limits)?;
        let successor_query = query(successor, limits)?;
        let added = successor_query
            .results
            .iter()
            .filter(|result| !base_query.results.contains(result))
            .collect::<Vec<_>>();
        let [added] = added.as_slice() else {
            return Err(KernelError::new(
                "semantic journey requires exactly one newly entailed query result",
            ));
        };
        let achievement_goal = query_fact(successor, added)?;
        let support_limits = SupportLimits::from(limits);

        let mut support_target = None;
        for result in &base_query.results {
            let target = query_fact(base, result)?;
            let Some(all) = execution::why_all(base, &target, support_limits)? else {
                continue;
            };
            if all.is_complete() && all.alternative_count() > 1 {
                support_target = Some((target, all));
                break;
            }
        }
        let (support_target, why_all) = support_target.ok_or_else(|| {
            KernelError::new("semantic journey requires a query result with redundant support")
        })?;
        let support_loss = support_loss_revision(base, &support_target, &why_all, support_limits)?;

        let [intent] = base.model().intents() else {
            return Err(KernelError::new(
                "semantic journey requires exactly one declared intent",
            ));
        };
        let intervention_relation = intent.desired().relation().to_owned();
        let active_domain = intervention_domain(base, intent.desired());
        let relation = base
            .model()
            .relations()
            .get(&intervention_relation)
            .expect("an admitted intent relation is declared");
        let candidate_count = active_domain
            .len()
            .checked_pow(relation.roles().len() as u32)
            .and_then(|count| {
                count.checked_sub(
                    base.model()
                        .facts()
                        .iter()
                        .filter(|fact| fact.relation() == intervention_relation)
                        .count(),
                )
            })
            .ok_or_else(|| KernelError::new("semantic journey candidate bound overflows"))?;
        let prevent_limits = PreventLimits::new(limits.max_facts, limits.max_facts, limits)
            .with_support_limits(support_limits)
            .using_relations(vec![intervention_relation.clone()]);
        let achieve_config = AchieveConfig::new(
            vec![intervention_relation],
            active_domain,
            candidate_count,
            limits.max_facts,
            limits,
        );
        Ok(Self::new(
            support_loss,
            support_target,
            prevent_limits,
            achievement_goal,
            achieve_config,
            limits,
        ))
    }

    pub fn support_loss(&self) -> &Revision {
        &self.support_loss
    }

    pub fn support_target(&self) -> &Clause {
        &self.support_target
    }

    pub fn prevent_limits(&self) -> &PreventLimits {
        &self.prevent_limits
    }

    pub fn achievement_goal(&self) -> &Clause {
        &self.achievement_goal
    }

    pub fn achieve_config(&self) -> &AchieveConfig {
        &self.achieve_config
    }

    pub fn query_limits(&self) -> Limits {
        self.query_limits
    }
}

/// Execute and encode every public semantic direction in one deterministic
/// byte contract. Every support and intervention result retains its bounded
/// completion status.
pub fn canonical_output(base: &Revision, journey: &SemanticJourney) -> Result<String> {
    let query = query(base, journey.query_limits)?;
    let support_limits = journey.prevent_limits.support_limits();
    let frontier = derive::support_frontier(base, &journey.support_target, support_limits)?;
    let why_all = execution::why_all(base, &journey.support_target, support_limits)?
        .ok_or_else(|| KernelError::new("semantic journey support target is absent"))?;
    let diff = SemanticDiff::between(base, &journey.support_loss, support_limits)?;
    let prevention = intervention::prevent(
        base,
        journey.support_target.clone(),
        journey.prevent_limits.clone(),
    )?;
    let achievement = intervention::achieve(
        base,
        journey.achievement_goal.clone(),
        &journey.achieve_config,
    )?;

    Ok(format!(
        "[\"clause-semantic-journey-v1\",[\"find\",{}],[\"support-frontier\",{}],[\"why-all\",{}],[\"support-diff\",{}],[\"prevent\",{}],[\"achieve\",{}]]",
        query_json(&query),
        frontier_json(&frontier),
        why_all_json(&why_all),
        diff_json(&diff),
        prevent_json(&prevention),
        achieve_json(&achievement, &journey.achieve_config),
    ))
}

fn query(revision: &Revision, limits: Limits) -> Result<QueryOutput> {
    execution::execute(revision, &revision.plan()?, limits)
}

fn query_fact(revision: &Revision, result: &str) -> Result<Clause> {
    let roles = revision
        .model()
        .query()
        .roles()
        .iter()
        .map(|(role, term)| {
            Ok((
                role.clone(),
                if term.is_variable() {
                    Term::literal(result)?
                } else {
                    term.clone()
                },
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    Clause::new(revision.model().query().relation(), roles)
}

fn support_loss_revision(
    base: &Revision,
    target: &Clause,
    why_all: &WhyAll,
    limits: SupportLimits,
) -> Result<Revision> {
    for alternative in &why_all.alternatives {
        for assertion in &alternative.assertions {
            let candidate =
                RevisionDelta::new(base.identity(), Vec::new(), vec![assertion.clone()])?
                    .apply(base)?;
            let Some(candidate_all) = execution::why_all(&candidate, target, limits)? else {
                continue;
            };
            if candidate_all.is_complete()
                && candidate_all.alternative_count() > 0
                && candidate_all.alternative_count() < why_all.alternative_count()
            {
                return Ok(candidate);
            }
        }
    }
    Err(KernelError::new(
        "semantic journey requires a support-preserving withdrawal",
    ))
}

fn intervention_domain(base: &Revision, desired: &Clause) -> Vec<String> {
    base.model()
        .facts()
        .iter()
        .filter(|fact| fact.relation() == desired.relation())
        .flat_map(|fact| fact.roles().values())
        .chain(desired.roles().values())
        .map(|term| term.text().to_owned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn query_json(output: &QueryOutput) -> String {
    let results = output
        .results
        .iter()
        .map(|result| quoted(result))
        .collect::<Vec<_>>()
        .join(",");
    format!("[\"results\",[{results}]]")
}

fn support_status(status: SupportStatus) -> &'static str {
    match status {
        SupportStatus::Complete => "complete",
        SupportStatus::ExpansionBudgetExhausted => "expansion-budget-exhausted",
        SupportStatus::SupportBudgetExhausted => "support-budget-exhausted",
    }
}

fn support_json(support: &Support) -> String {
    format!(
        "[\"support\",[\"assertions\",[{}]]]",
        clauses_json(support.assertions())
    )
}

fn frontier_json(frontier: &SupportFrontier) -> String {
    let supports = frontier
        .supports()
        .iter()
        .map(support_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"target\",{}],[\"status\",{}],[\"expansions\",{}],[\"supports\",[{supports}]]",
        clause_json(frontier.target()),
        quoted(support_status(frontier.status())),
        frontier.expansions(),
    )
}

fn why_all_json(all: &WhyAll) -> String {
    let alternatives = all
        .alternatives
        .iter()
        .map(|alternative| {
            format!(
                "[\"alternative\",[\"assertions\",[{}]],[\"proof\",{}]]",
                clauses_json(&alternative.assertions),
                execution::canonical_why_json(&alternative.why),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"target\",{}],[\"status\",{}],[\"expansions\",{}],[\"alternatives\",[{alternatives}]]",
        clause_json(&all.target),
        quoted(if all.is_complete() {
            "complete"
        } else {
            "incomplete"
        }),
        all.expansions,
    )
}

fn support_change_json(change: &SupportChange) -> String {
    let added = change
        .added()
        .iter()
        .map(support_json)
        .collect::<Vec<_>>()
        .join(",");
    let removed = change
        .removed()
        .iter()
        .map(support_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"change\",[\"target\",{}],[\"base\",{}],[\"successor\",{}],[\"added\",[{added}]],[\"removed\",[{removed}]]]",
        clause_json(change.fact()),
        frontier_json(change.base()),
        frontier_json(change.successor()),
    )
}

fn diff_json(diff: &SemanticDiff) -> String {
    let changes = diff
        .changed_supports()
        .iter()
        .map(support_change_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"asserted\",[\"added\",[{}]],[\"removed\",[{}]]],[\"entailed\",[\"added\",[{}]],[\"removed\",[{}]]],[\"support-changes\",[{changes}]]",
        clauses_json(diff.authored().added()),
        clauses_json(diff.authored().removed()),
        clauses_json(diff.entailed_added()),
        clauses_json(diff.entailed_removed()),
    )
}

fn prevent_status(status: PreventStatus) -> &'static str {
    match status {
        PreventStatus::Complete => "complete",
        PreventStatus::AlreadyAbsent => "already-absent",
        PreventStatus::Impossible => "impossible",
        PreventStatus::SupportExpansionBudgetExhausted => "support-expansion-budget-exhausted",
        PreventStatus::SupportBudgetExhausted => "support-budget-exhausted",
        PreventStatus::CandidateBudgetExhausted => "candidate-budget-exhausted",
        PreventStatus::SolutionBudgetExhausted => "solution-budget-exhausted",
    }
}

fn prevent_json(report: &PreventReport) -> String {
    let solutions = report
        .solutions()
        .iter()
        .map(|solution| {
            format!(
                "[\"withdrawals\",[{}]]",
                clauses_json(solution.withdrawals())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"target\",{}],[\"status\",{}],[\"candidates\",{}],[\"solutions\",[{solutions}]]",
        clause_json(report.target()),
        quoted(prevent_status(report.status())),
        report.candidates_examined(),
    )
}

fn achieve_json(result: &AchieveResult, config: &AchieveConfig) -> String {
    let status = match result {
        AchieveResult::Solutions(_) => "complete",
        AchieveResult::Impossible => "impossible",
        AchieveResult::CandidateLimit(_) => "candidate-budget-exhausted",
        AchieveResult::SolutionLimit(_) => "solution-budget-exhausted",
    };
    let interventions = result
        .interventions()
        .iter()
        .map(|intervention| {
            format!(
                "[\"additions\",[{}]]",
                clauses_json(intervention.additions())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"status\",{}],[\"max-candidates\",{}],[\"max-solutions\",{}],[\"interventions\",[{interventions}]]",
        quoted(status),
        config.max_candidates(),
        config.max_solutions(),
    )
}

fn clauses_json(clauses: &[Clause]) -> String {
    clauses
        .iter()
        .map(clause_json)
        .collect::<Vec<_>>()
        .join(",")
}

fn clause_json(clause: &Clause) -> String {
    let roles = clause
        .roles()
        .iter()
        .map(|(name, term)| format!("[{},{}]", quoted(name), quoted(term.text())))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"clause\",\"relation\",{},\"roles\",[{roles}]]",
        quoted(clause.relation())
    )
}

fn quoted(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value <= '\u{1f}' => write!(escaped, "\\u{:04x}", value as u32).unwrap(),
            value => escaped.push(value),
        }
    }
    format!("\"{escaped}\"")
}
