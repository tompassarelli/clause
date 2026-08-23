//! Bounded synthesis of direct fact additions for an absent ground goal.
//!
//! The search space is deliberately explicit: callers name the extensional
//! relations that may receive facts and the finite active domain used to fill
//! their roles.  Each candidate is admitted as a fresh [`Revision`].

use crate::delta::RevisionDelta;
use crate::derive::{self, Limits, Proof};
use crate::kernel::{self, Clause, KernelError, Model, Result, Revision, Term};
use std::collections::BTreeSet;

/// Explicit bounds and inputs for one finite intervention search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AchieveConfig {
    allowed_relations: Vec<String>,
    active_domain: Vec<String>,
    max_candidates: usize,
    max_solutions: usize,
    closure_limits: Limits,
}

impl AchieveConfig {
    pub fn new(
        allowed_relations: Vec<String>,
        active_domain: Vec<String>,
        max_candidates: usize,
        max_solutions: usize,
        closure_limits: Limits,
    ) -> Self {
        let allowed_relations = canonical(allowed_relations);
        let active_domain = canonical(active_domain);
        Self {
            allowed_relations,
            active_domain,
            max_candidates,
            max_solutions,
            closure_limits,
        }
    }

    pub fn allowed_relations(&self) -> &[String] {
        &self.allowed_relations
    }

    pub fn active_domain(&self) -> &[String] {
        &self.active_domain
    }

    pub fn max_candidates(&self) -> usize {
        self.max_candidates
    }

    pub fn max_solutions(&self) -> usize {
        self.max_solutions
    }

    pub fn closure_limits(&self) -> Limits {
        self.closure_limits
    }
}

/// One minimal direct-fact addition set and the immutable revision it admits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intervention {
    additions: Vec<Clause>,
    revision: Revision,
    proof: Proof,
}

impl Intervention {
    pub fn additions(&self) -> &[Clause] {
        &self.additions
    }

    pub fn revision(&self) -> &Revision {
        &self.revision
    }

    /// The selected closure proof for the requested goal in `revision`.
    pub fn proof(&self) -> &Proof {
        &self.proof
    }
}

/// The finite search result.  Limit variants retain any earlier minimal
/// answers in canonical order, but make clear that enumeration was truncated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AchieveResult {
    Solutions(Vec<Intervention>),
    Impossible,
    CandidateLimit(Vec<Intervention>),
    SolutionLimit(Vec<Intervention>),
}

impl AchieveResult {
    pub fn interventions(&self) -> &[Intervention] {
        match self {
            Self::Solutions(interventions)
            | Self::CandidateLimit(interventions)
            | Self::SolutionLimit(interventions) => interventions,
            Self::Impossible => &[],
        }
    }
}

/// Find inclusion-minimal sets of allowed direct facts that entail `goal`.
///
/// Candidates are evaluated in increasing addition count and canonical clause
/// order.  A relation is extensional here precisely when no admitted law has
/// it as a conclusion; allowlisting a derived-only relation is rejected.
pub fn achieve(revision: &Revision, goal: Clause, config: &AchieveConfig) -> Result<AchieveResult> {
    kernel::require(revision, goal.clone())?;
    let source_closure = derive::saturate(revision, config.closure_limits)?;
    if source_closure.proof(&goal).is_some() {
        return Err(KernelError::new("achievement goal is already entailed"));
    }

    let candidates = candidate_facts(revision, config)?;
    let candidates = candidates
        .into_iter()
        .filter(|candidate| revision.model().facts().binary_search(candidate).is_err())
        .collect::<Vec<_>>();
    let mut evaluated = 0usize;
    let mut solutions = Vec::new();

    for size in 1..=candidates.len() {
        if let Some(result) = search_combinations(
            revision,
            &goal,
            config,
            &candidates,
            size,
            0,
            &mut Vec::new(),
            &mut evaluated,
            &mut solutions,
        )? {
            return Ok(result);
        }
    }

    Ok(if solutions.is_empty() {
        AchieveResult::Impossible
    } else {
        AchieveResult::Solutions(solutions)
    })
}

fn candidate_facts(revision: &Revision, config: &AchieveConfig) -> Result<Vec<Clause>> {
    let model = revision.model();
    let derived_relations = model
        .laws()
        .iter()
        .map(|law| law.conclusion().relation())
        .collect::<BTreeSet<_>>();
    let mut candidates = BTreeSet::new();

    for relation_name in config.allowed_relations() {
        let relation = model
            .relations()
            .get(relation_name)
            .ok_or_else(|| KernelError::new("intervention relation is undeclared"))?;
        if derived_relations.contains(relation_name.as_str()) {
            return Err(KernelError::new(format!(
                "intervention relation is derived-only: {relation_name}"
            )));
        }
        let role_names = relation.roles().keys().cloned().collect::<Vec<_>>();
        collect_relation_facts(
            relation_name,
            &role_names,
            config.active_domain(),
            &mut Vec::new(),
            &mut candidates,
        )?;
    }

    Ok(candidates.into_iter().collect())
}

fn collect_relation_facts(
    relation: &str,
    roles: &[String],
    domain: &[String],
    values: &mut Vec<String>,
    candidates: &mut BTreeSet<Clause>,
) -> Result<()> {
    if values.len() == roles.len() {
        let fact = Clause::new(
            relation,
            roles
                .iter()
                .cloned()
                .zip(values.iter().cloned().map(Term::literal))
                .map(|(role, value)| Ok((role, value?)))
                .collect::<Result<Vec<_>>>()?,
        )?;
        candidates.insert(fact);
        return Ok(());
    }
    for value in domain {
        values.push(value.clone());
        collect_relation_facts(relation, roles, domain, values, candidates)?;
        values.pop();
    }
    Ok(())
}

fn candidate_revision(revision: &Revision, additions: Vec<Clause>) -> Result<Revision> {
    let model = revision.model();
    let mut facts = model.facts().to_vec();
    facts.extend(additions);
    Ok(Revision::admit(Model::with_laws_and_intents(
        model.relations().values().cloned().collect(),
        facts,
        model.laws().to_vec(),
        model.query().clone(),
        model.intents().to_vec(),
        model.order().to_owned(),
    )?))
}

#[allow(clippy::too_many_arguments)]
fn search_combinations(
    revision: &Revision,
    goal: &Clause,
    config: &AchieveConfig,
    candidates: &[Clause],
    remaining: usize,
    start: usize,
    chosen: &mut Vec<Clause>,
    evaluated: &mut usize,
    solutions: &mut Vec<Intervention>,
) -> Result<Option<AchieveResult>> {
    if remaining == 0 {
        if solutions
            .iter()
            .any(|solution| is_subset(solution.additions(), chosen))
        {
            return Ok(None);
        }
        if *evaluated == config.max_candidates {
            return Ok(Some(AchieveResult::CandidateLimit(solutions.clone())));
        }
        *evaluated += 1;
        let candidate = candidate_revision(revision, chosen.clone())?;
        let closure = derive::saturate(&candidate, config.closure_limits)?;
        let Some(proof) = closure.proof(goal).cloned() else {
            return Ok(None);
        };
        if solutions.len() == config.max_solutions {
            return Ok(Some(AchieveResult::SolutionLimit(solutions.clone())));
        }
        solutions.push(Intervention {
            additions: chosen.clone(),
            revision: candidate,
            proof,
        });
        return Ok(None);
    }
    for index in start..=candidates.len() - remaining {
        chosen.push(candidates[index].clone());
        if let Some(result) = search_combinations(
            revision,
            goal,
            config,
            candidates,
            remaining - 1,
            index + 1,
            chosen,
            evaluated,
            solutions,
        )? {
            return Ok(Some(result));
        }
        chosen.pop();
    }
    Ok(None)
}

fn is_subset(subset: &[Clause], set: &[Clause]) -> bool {
    subset
        .iter()
        .all(|candidate| set.binary_search(candidate).is_ok())
}

fn canonical(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Bounds for a prevention search.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreventLimits {
    max_candidates: usize,
    max_solutions: usize,
    closure: Limits,
}

impl PreventLimits {
    /// Construct limits using unbounded closure limits.
    pub fn new(max_candidates: usize, max_solutions: usize) -> Self {
        Self {
            max_candidates,
            max_solutions,
            closure: Limits::new(usize::MAX, usize::MAX, usize::MAX),
        }
    }

    /// Bound the fixed-point computation performed for every candidate.
    pub fn with_closure_limits(mut self, closure: Limits) -> Self {
        self.closure = closure;
        self
    }

    pub fn max_candidates(&self) -> usize {
        self.max_candidates
    }

    pub fn max_solutions(&self) -> usize {
        self.max_solutions
    }

    pub fn closure_limits(&self) -> Limits {
        self.closure
    }
}

/// Why a prevention search stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreventStatus {
    /// Every candidate subset was considered and all minimal solutions found.
    Complete,
    /// The source closure did not entail the target, so no withdrawal is
    /// needed or returned.
    AlreadyAbsent,
    /// The candidate enumeration reached `max_candidates`.
    CandidateBudgetExhausted,
    /// The result reached `max_solutions` before enumeration was complete.
    SolutionBudgetExhausted,
}

/// One inclusion-minimal withdrawal and its immutable candidate revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreventSolution {
    withdrawals: Vec<Clause>,
    revision: Revision,
}

impl PreventSolution {
    pub fn withdrawals(&self) -> &[Clause] {
        &self.withdrawals
    }

    /// The source declarations are preserved and only these direct facts are
    /// absent from the candidate revision.
    pub fn revision(&self) -> &Revision {
        &self.revision
    }
}

/// Deterministic, bounded prevention output for one source revision and target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreventReport {
    source_revision: String,
    target: Clause,
    status: PreventStatus,
    candidates_examined: usize,
    solutions: Vec<PreventSolution>,
}

impl PreventReport {
    pub fn source_revision(&self) -> &str {
        &self.source_revision
    }

    pub fn target(&self) -> &Clause {
        &self.target
    }

    pub fn status(&self) -> PreventStatus {
        self.status
    }

    pub fn candidates_examined(&self) -> usize {
        self.candidates_examined
    }

    pub fn solutions(&self) -> &[PreventSolution] {
        &self.solutions
    }
}

/// Enumerate every inclusion-minimal set of direct asserted facts whose
/// withdrawal makes `target` absent from the derived closure.
pub fn prevent(source: &Revision, target: Clause, limits: PreventLimits) -> Result<PreventReport> {
    if target.roles().values().any(|term| term.is_variable()) {
        return Err(KernelError::new("prevent target must be ground"));
    }

    let source_closure = derive::saturate(source, limits.closure)?;
    let source_revision = source.identity().to_owned();
    if source_closure.proof(&target).is_none() {
        return Ok(PreventReport {
            source_revision,
            target,
            status: PreventStatus::AlreadyAbsent,
            candidates_examined: 0,
            solutions: Vec::new(),
        });
    }

    let direct_facts = source.model().facts().to_vec();
    let mut state = PreventSearch {
        source,
        target: &target,
        direct_facts: &direct_facts,
        limits,
        candidates_examined: 0,
        solutions: Vec::new(),
        status: None,
    };

    if limits.max_solutions == 0 {
        state.status = Some(PreventStatus::SolutionBudgetExhausted);
    } else {
        for size in 1..=direct_facts.len() {
            if state.status.is_some() {
                break;
            }
            let mut indexes = Vec::with_capacity(size);
            state.combinations(size, 0, &mut indexes)?;
        }
    }

    let status = state.status.unwrap_or(PreventStatus::Complete);
    Ok(PreventReport {
        source_revision,
        target: target.clone(),
        status,
        candidates_examined: state.candidates_examined,
        solutions: state.solutions,
    })
}

struct PreventSearch<'a> {
    source: &'a Revision,
    target: &'a Clause,
    direct_facts: &'a [Clause],
    limits: PreventLimits,
    candidates_examined: usize,
    solutions: Vec<PreventSolution>,
    status: Option<PreventStatus>,
}

impl PreventSearch<'_> {
    fn combinations(&mut self, size: usize, next: usize, indexes: &mut Vec<usize>) -> Result<()> {
        if self.status.is_some() {
            return Ok(());
        }
        if indexes.len() == size {
            return self.evaluate(indexes);
        }

        let remaining = size - indexes.len();
        let last_start = self.direct_facts.len().saturating_sub(remaining);
        for index in next..=last_start {
            indexes.push(index);
            self.combinations(size, index + 1, indexes)?;
            indexes.pop();
            if self.status.is_some() {
                break;
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, indexes: &[usize]) -> Result<()> {
        if self.candidates_examined >= self.limits.max_candidates {
            self.status = Some(PreventStatus::CandidateBudgetExhausted);
            return Ok(());
        }
        let withdrawals = indexes
            .iter()
            .map(|index| self.direct_facts[*index].clone())
            .collect::<Vec<_>>();

        self.candidates_examined += 1;
        if self
            .solutions
            .iter()
            .any(|solution| is_subset(solution.withdrawals(), &withdrawals))
        {
            return Ok(());
        }

        let candidate =
            RevisionDelta::new(self.source.identity(), Vec::new(), withdrawals.clone())?
                .apply(self.source)?;
        let closure = derive::saturate(&candidate, self.limits.closure)?;
        if closure.proof(self.target).is_none() {
            self.solutions.push(PreventSolution {
                withdrawals,
                revision: candidate,
            });
            if self.solutions.len() >= self.limits.max_solutions {
                self.status = Some(PreventStatus::SolutionBudgetExhausted);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{AchieveConfig, AchieveResult, PreventLimits, PreventStatus, achieve, prevent};
    use crate::derive::{Limits, Witness};
    use crate::kernel::{
        Cardinality, Clause, Law, Mode, Model, Relation, Revision, Role, Sentence, Term,
    };
    use crate::wire;

    fn relation(name: &str) -> Relation {
        Relation::new(
            name,
            vec![
                Role::new("from", "Place").unwrap(),
                Role::new("to", "Place").unwrap(),
            ],
            Sentence::new("from", "reaches", "to").unwrap(),
            vec![Mode::finite(vec!["from".into()], vec!["to".into()], Cardinality::Many).unwrap()],
        )
        .unwrap()
    }

    fn clause(relation: &str, from: &str, to: &str) -> Clause {
        Clause::new(
            relation,
            vec![
                ("from".into(), Term::literal(from).unwrap()),
                ("to".into(), Term::literal(to).unwrap()),
            ],
        )
        .unwrap()
    }

    fn pattern(relation: &str, from: &str, to: &str) -> Clause {
        Clause::new(
            relation,
            vec![
                ("from".into(), Term::variable(from).unwrap()),
                ("to".into(), Term::variable(to).unwrap()),
            ],
        )
        .unwrap()
    }

    fn revision(laws: Vec<Law>) -> Revision {
        Revision::admit(
            Model::with_laws(
                vec![
                    relation("map/input"),
                    relation("map/left"),
                    relation("map/right"),
                    relation("map/middle"),
                    relation("map/goal"),
                ],
                vec![],
                laws,
                Clause::new(
                    "map/goal",
                    vec![
                        ("from".into(), Term::literal("A").unwrap()),
                        ("to".into(), Term::variable("destination").unwrap()),
                    ],
                )
                .unwrap(),
                "ascending",
            )
            .unwrap(),
        )
    }

    fn prevention_revision(facts: Vec<Clause>, laws: Vec<Law>) -> Revision {
        Revision::admit(
            Model::with_laws(
                vec![relation("map/links"), relation("map/reaches")],
                facts,
                laws,
                Clause::new(
                    "map/reaches",
                    vec![
                        ("from".into(), Term::literal("North").unwrap()),
                        ("to".into(), Term::variable("destination").unwrap()),
                    ],
                )
                .unwrap(),
                "ascending",
            )
            .unwrap(),
        )
    }

    fn config(allowed: Vec<&str>, max_candidates: usize, max_solutions: usize) -> AchieveConfig {
        AchieveConfig::new(
            allowed.into_iter().map(str::to_owned).collect(),
            vec!["B".into(), "A".into()],
            max_candidates,
            max_solutions,
            Limits::new(100, 10, 10_000),
        )
    }

    #[test]
    fn direct_fact_achievement_has_an_asserted_proof() {
        let source = revision(vec![]);
        let result = achieve(
            &source,
            clause("map/goal", "A", "B"),
            &config(vec!["map/goal"], 100, 10),
        )
        .unwrap();

        let interventions = result.interventions();
        assert_eq!(interventions.len(), 1);
        assert_eq!(
            interventions[0].additions(),
            &[clause("map/goal", "A", "B")]
        );
        assert!(matches!(
            interventions[0].proof().witness(),
            Witness::Asserted
        ));
    }

    #[test]
    fn two_law_chain_returns_the_enabled_proof_and_preserves_source() {
        let source = revision(vec![
            Law::new(
                "map/01-input-middle",
                vec![pattern("map/input", "from", "to")],
                pattern("map/middle", "from", "to"),
            )
            .unwrap(),
            Law::new(
                "map/02-middle-goal",
                vec![pattern("map/middle", "from", "to")],
                pattern("map/goal", "from", "to"),
            )
            .unwrap(),
        ]);
        let source_wire = wire::serialize(&source);
        let result = achieve(
            &source,
            clause("map/goal", "A", "B"),
            &config(vec!["map/input"], 100, 10),
        )
        .unwrap();

        let intervention = &result.interventions()[0];
        assert_eq!(intervention.additions(), &[clause("map/input", "A", "B")]);
        assert_eq!(intervention.proof().generation(), 2);
        match intervention.proof().witness() {
            Witness::Derived { law, premises, .. } => {
                assert_eq!(law, "map/02-middle-goal");
                assert_eq!(premises, &[clause("map/middle", "A", "B")]);
            }
            Witness::Asserted => panic!("goal should be derived"),
        }
        assert_eq!(wire::serialize(&source), source_wire);
        assert!(source.model().facts().is_empty());
    }

    #[test]
    fn alternate_minimal_additions_are_canonical_and_deterministic() {
        let source = revision(vec![
            Law::new(
                "map/left-goal",
                vec![pattern("map/left", "from", "to")],
                pattern("map/goal", "from", "to"),
            )
            .unwrap(),
            Law::new(
                "map/right-goal",
                vec![pattern("map/right", "from", "to")],
                pattern("map/goal", "from", "to"),
            )
            .unwrap(),
        ]);
        let goal = clause("map/goal", "A", "B");
        let first = achieve(
            &source,
            goal.clone(),
            &config(vec!["map/right", "map/left"], 100, 10),
        )
        .unwrap();
        let second = achieve(
            &source,
            goal,
            &config(vec!["map/left", "map/right"], 100, 10),
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(
            first
                .interventions()
                .iter()
                .map(|intervention| intervention.additions().to_vec())
                .collect::<Vec<_>>(),
            vec![
                vec![clause("map/left", "A", "B")],
                vec![clause("map/right", "A", "B")],
            ]
        );
    }

    #[test]
    fn inclusion_minimal_sets_continue_after_a_smaller_solution() {
        let source = revision(vec![
            Law::new(
                "map/left-goal",
                vec![pattern("map/left", "from", "to")],
                pattern("map/goal", "from", "to"),
            )
            .unwrap(),
            Law::new(
                "map/input-pair-goal",
                vec![
                    pattern("map/input", "from", "middle"),
                    pattern("map/input", "middle", "to"),
                ],
                pattern("map/goal", "from", "to"),
            )
            .unwrap(),
        ]);
        let result = achieve(
            &source,
            clause("map/goal", "A", "B"),
            &config(vec!["map/left", "map/input"], 1_000, 10),
        )
        .unwrap();

        assert_eq!(
            result
                .interventions()
                .iter()
                .map(|intervention| intervention.additions().to_vec())
                .collect::<Vec<_>>(),
            vec![
                vec![clause("map/left", "A", "B")],
                vec![clause("map/input", "A", "A"), clause("map/input", "A", "B"),],
                vec![clause("map/input", "A", "B"), clause("map/input", "B", "B"),],
            ]
        );
    }

    #[test]
    fn derived_only_relations_cannot_be_allowlisted() {
        let source = revision(vec![
            Law::new(
                "map/middle-goal",
                vec![pattern("map/middle", "from", "to")],
                pattern("map/goal", "from", "to"),
            )
            .unwrap(),
        ]);
        let error = achieve(
            &source,
            clause("map/goal", "A", "B"),
            &config(vec!["map/goal"], 100, 10),
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "intervention relation is derived-only: map/goal"
        );
    }

    #[test]
    fn impossible_and_candidate_budget_exhaustion_are_distinct() {
        let source = revision(vec![
            Law::new(
                "map/input-goal",
                vec![pattern("map/input", "from", "to")],
                pattern("map/goal", "from", "to"),
            )
            .unwrap(),
        ]);
        assert_eq!(
            achieve(
                &source,
                clause("map/goal", "A", "B"),
                &config(vec!["map/left"], 100, 10),
            )
            .unwrap(),
            AchieveResult::Impossible
        );
        assert!(matches!(
            achieve(
                &source,
                clause("map/goal", "A", "B"),
                &config(vec!["map/input"], 1, 10),
            )
            .unwrap(),
            AchieveResult::CandidateLimit(interventions) if interventions.is_empty()
        ));
        assert!(matches!(
            achieve(
                &revision(vec![]),
                clause("map/goal", "A", "B"),
                &config(vec!["map/goal"], 100, 0),
            )
            .unwrap(),
            AchieveResult::SolutionLimit(interventions) if interventions.is_empty()
        ));
    }

    #[test]
    fn alternate_paths_require_one_withdrawal_from_each_path() {
        let a = clause("map/links", "North", "A");
        let b = clause("map/links", "A", "Store");
        let c = clause("map/links", "North", "B");
        let d = clause("map/links", "B", "Store");
        let law = Law::new(
            "map/path-reaches",
            vec![
                pattern("map/links", "from", "middle"),
                pattern("map/links", "middle", "to"),
            ],
            pattern("map/reaches", "from", "to"),
        )
        .unwrap();
        let source =
            prevention_revision(vec![a.clone(), b.clone(), c.clone(), d.clone()], vec![law]);
        let original = source.clone();
        let report = prevent(
            &source,
            clause("map/reaches", "North", "Store"),
            PreventLimits::new(100, 100),
        )
        .unwrap();

        assert_eq!(report.status(), PreventStatus::Complete);
        assert_eq!(report.solutions().len(), 4);
        assert_eq!(report.solutions()[0].withdrawals(), &[b.clone(), d.clone()]);
        assert_eq!(report.solutions()[1].withdrawals(), &[b, c.clone()]);
        assert_eq!(report.solutions()[2].withdrawals(), &[d, a.clone()]);
        assert_eq!(report.solutions()[3].withdrawals(), &[a, c]);
        assert_eq!(source, original);
        assert!(report.solutions().iter().all(|solution| {
            solution
                .revision()
                .model()
                .facts()
                .iter()
                .all(|fact| fact.relation() == "map/links")
        }));

        let rerun = prevent(
            &source,
            clause("map/reaches", "North", "Store"),
            PreventLimits::new(100, 100),
        )
        .unwrap();
        assert_eq!(report, rerun);
    }

    #[test]
    fn absent_target_and_budgets_are_explicit() {
        let source = prevention_revision(vec![clause("map/links", "North", "Store")], Vec::new());
        let absent = prevent(
            &source,
            clause("map/reaches", "North", "Store"),
            PreventLimits::new(0, 0),
        )
        .unwrap();
        assert_eq!(absent.status(), PreventStatus::AlreadyAbsent);
        let law = Law::new(
            "map/link-reaches",
            vec![pattern("map/links", "from", "to")],
            pattern("map/reaches", "from", "to"),
        )
        .unwrap();
        let source = prevention_revision(vec![clause("map/links", "North", "Store")], vec![law]);
        let exhausted = prevent(
            &source,
            clause("map/reaches", "North", "Store"),
            PreventLimits::new(0, 10),
        )
        .unwrap();
        assert_eq!(exhausted.status(), PreventStatus::CandidateBudgetExhausted);
    }
}
