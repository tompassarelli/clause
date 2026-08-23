//! Deterministic finite closure for admitted positive laws.

use crate::kernel::{self, Clause, KernelError, Law, Result, Revision, Term};
use std::collections::{BTreeMap, BTreeSet};

/// Explicit resource bounds for one closure computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub max_facts: usize,
    pub max_rounds: usize,
    pub max_join_attempts: usize,
}

impl Limits {
    pub fn new(max_facts: usize, max_rounds: usize, max_join_attempts: usize) -> Self {
        Self {
            max_facts,
            max_rounds,
            max_join_attempts,
        }
    }
}

/// The selected proof for a fact in a closure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    generation: usize,
    witness: Witness,
}

impl Proof {
    /// Asserted facts have generation zero; derived facts have the first round
    /// in which they were available.
    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn witness(&self) -> &Witness {
        &self.witness
    }
}

/// The canonical witness selected for a fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Witness {
    Asserted,
    Derived {
        law: String,
        premises: Vec<Clause>,
        substitution: BTreeMap<String, String>,
    },
}

/// A sorted least fixed point and its acyclic chosen proof DAG.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closure {
    facts: Vec<Clause>,
    proofs: BTreeMap<Clause, Proof>,
}

impl Closure {
    pub fn facts(&self) -> &[Clause] {
        &self.facts
    }

    pub fn proof(&self, fact: &Clause) -> Option<&Proof> {
        self.proofs.get(fact)
    }
}

/// Explicit resource bounds for enumerating minimal asserted supports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupportLimits {
    pub closure: Limits,
    /// Maximum number of complete premise-support combinations considered.
    pub max_expansions: usize,
    /// Maximum antichain size retained for any one entailed clause.
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

/// Whether support enumeration reached its fixed point.
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

/// One canonical acyclic proof for a single asserted support.
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

/// The witness at one node of a support proof.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SupportWitness {
    Asserted,
    Derived {
        law: String,
        premises: Vec<SupportProof>,
        substitution: BTreeMap<String, String>,
    },
}

/// An inclusion-minimal sorted set of asserted clauses and its canonical proof.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Support {
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
}

/// Every bounded inclusion-minimal asserted support for one target in one Revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportFrontier {
    revision: String,
    target: Clause,
    limits: SupportLimits,
    status: SupportStatus,
    expansions: usize,
    supports: Vec<Support>,
}

impl SupportFrontier {
    pub fn revision(&self) -> &str {
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Candidate {
    law: String,
    premises: Vec<Clause>,
    substitution: BTreeMap<String, String>,
}

/// Saturate the asserted facts of `revision` under its admitted laws.
///
/// Rounds are level-synchronous: every proof selected in a round refers only
/// to facts from earlier generations. Limits are checked before admitting a
/// fact or performing a join attempt.
pub fn saturate(revision: &Revision, limits: Limits) -> Result<Closure> {
    let mut proofs = revision
        .model()
        .facts()
        .iter()
        .cloned()
        .map(|fact| {
            (
                fact,
                Proof {
                    generation: 0,
                    witness: Witness::Asserted,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    if proofs.len() > limits.max_facts {
        return Err(limit_error("fact", "max_facts", limits.max_facts));
    }

    let mut join_attempts = 0usize;
    let mut generation = 1usize;
    loop {
        let facts = proofs.keys().cloned().collect::<Vec<_>>();
        let mut candidates = BTreeMap::<Clause, Candidate>::new();

        for law in revision.model().laws() {
            collect_law_candidates(law, &facts, &limits, &mut join_attempts, &mut candidates)?;
        }

        candidates.retain(|fact, _| !proofs.contains_key(fact));
        if candidates.is_empty() {
            break;
        }
        if generation > limits.max_rounds {
            return Err(limit_error("round", "max_rounds", limits.max_rounds));
        }
        if candidates.len() > limits.max_facts.saturating_sub(proofs.len()) {
            return Err(limit_error("fact", "max_facts", limits.max_facts));
        }

        for (fact, candidate) in candidates {
            proofs.insert(
                fact,
                Proof {
                    generation,
                    witness: Witness::Derived {
                        law: candidate.law,
                        premises: candidate.premises,
                        substitution: candidate.substitution,
                    },
                },
            );
        }
        generation += 1;
    }

    Ok(Closure {
        facts: proofs.keys().cloned().collect(),
        proofs,
    })
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct GroundDerivation {
    conclusion: Clause,
    law: String,
    premises: Vec<Clause>,
    substitution: BTreeMap<String, String>,
}

/// Enumerate the bounded support frontier for `target` in exactly `revision`.
///
/// A non-complete status means `supports()` is only the deterministic prefix
/// discovered before the named budget was exhausted.
pub fn support_frontier(
    revision: &Revision,
    target: &Clause,
    limits: SupportLimits,
) -> Result<SupportFrontier> {
    kernel::require(revision, target.clone())?;
    let closure = saturate(revision, limits.closure)?;
    if limits.max_supports_per_clause == 0 {
        return Ok(SupportFrontier {
            revision: revision.identity().to_owned(),
            target: target.clone(),
            limits,
            status: SupportStatus::SupportBudgetExhausted,
            expansions: 0,
            supports: Vec::new(),
        });
    }
    let mut derivations = BTreeSet::new();
    let mut join_attempts = 0;
    for law in revision.model().laws() {
        collect_ground_derivations(
            law,
            closure.facts(),
            &limits.closure,
            &mut join_attempts,
            &mut derivations,
            0,
            BTreeMap::new(),
            Vec::new(),
        )?;
    }

    let mut frontiers = revision
        .model()
        .facts()
        .iter()
        .cloned()
        .map(|assertion| {
            let proof = SupportProof {
                conclusion: assertion.clone(),
                witness: SupportWitness::Asserted,
            };
            (
                assertion.clone(),
                BTreeMap::from([(vec![assertion], proof)]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut explored = BTreeSet::<SupportProof>::new();
    let mut expansions = 0;
    let mut status = SupportStatus::Complete;

    'fixed_point: loop {
        let mut changed = false;
        for derivation in &derivations {
            let Some(premise_frontiers) = derivation
                .premises
                .iter()
                .map(|premise| frontiers.get(premise).map(|supports| supports.values()))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let combinations = premise_frontiers
                .into_iter()
                .map(|supports| supports.cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            let mut selected = Vec::with_capacity(combinations.len());
            if let Some(exhausted) = expand_derivation(
                derivation,
                &combinations,
                0,
                &mut selected,
                &mut explored,
                &mut frontiers,
                &limits,
                &mut expansions,
                &mut changed,
            ) {
                status = exhausted;
                break 'fixed_point;
            }
        }
        if !changed {
            break;
        }
    }

    let supports = frontiers
        .remove(target)
        .unwrap_or_default()
        .into_iter()
        .map(|(assertions, proof)| Support { assertions, proof })
        .collect();
    Ok(SupportFrontier {
        revision: revision.identity().to_owned(),
        target: target.clone(),
        limits,
        status,
        expansions,
        supports,
    })
}

#[allow(clippy::too_many_arguments)]
fn expand_derivation(
    derivation: &GroundDerivation,
    frontiers: &[Vec<SupportProof>],
    index: usize,
    selected: &mut Vec<SupportProof>,
    explored: &mut BTreeSet<SupportProof>,
    supports_by_clause: &mut BTreeMap<Clause, BTreeMap<Vec<Clause>, SupportProof>>,
    limits: &SupportLimits,
    expansions: &mut usize,
    changed: &mut bool,
) -> Option<SupportStatus> {
    if index == frontiers.len() {
        if selected
            .iter()
            .any(|premise| premise.contains(&derivation.conclusion))
        {
            return None;
        }
        let proof = SupportProof {
            conclusion: derivation.conclusion.clone(),
            witness: SupportWitness::Derived {
                law: derivation.law.clone(),
                premises: selected.clone(),
                substitution: derivation.substitution.clone(),
            },
        };
        if !explored.insert(proof.clone()) {
            return None;
        }
        if *expansions >= limits.max_expansions {
            return Some(SupportStatus::ExpansionBudgetExhausted);
        }
        *expansions += 1;
        let assertions = proof_assertions(&proof);
        let frontier = supports_by_clause
            .entry(derivation.conclusion.clone())
            .or_default();
        return match insert_support(frontier, assertions, proof, limits.max_supports_per_clause) {
            InsertSupport::Unchanged => None,
            InsertSupport::Changed => {
                *changed = true;
                None
            }
            InsertSupport::BudgetExhausted => Some(SupportStatus::SupportBudgetExhausted),
        };
    }

    for proof in &frontiers[index] {
        selected.push(proof.clone());
        let exhausted = expand_derivation(
            derivation,
            frontiers,
            index + 1,
            selected,
            explored,
            supports_by_clause,
            limits,
            expansions,
            changed,
        );
        selected.pop();
        if exhausted.is_some() {
            return exhausted;
        }
    }
    None
}

fn proof_assertions(proof: &SupportProof) -> Vec<Clause> {
    let mut assertions = BTreeSet::new();
    collect_proof_assertions(proof, &mut assertions);
    assertions.into_iter().collect()
}

fn collect_proof_assertions(proof: &SupportProof, assertions: &mut BTreeSet<Clause>) {
    match &proof.witness {
        SupportWitness::Asserted => {
            assertions.insert(proof.conclusion.clone());
        }
        SupportWitness::Derived { premises, .. } => {
            for premise in premises {
                collect_proof_assertions(premise, assertions);
            }
        }
    }
}

enum InsertSupport {
    Unchanged,
    Changed,
    BudgetExhausted,
}

fn insert_support(
    frontier: &mut BTreeMap<Vec<Clause>, SupportProof>,
    assertions: Vec<Clause>,
    proof: SupportProof,
    max_supports: usize,
) -> InsertSupport {
    if let Some(chosen) = frontier.get_mut(&assertions) {
        if proof < *chosen {
            *chosen = proof;
            return InsertSupport::Changed;
        }
        return InsertSupport::Unchanged;
    }
    if frontier
        .keys()
        .any(|known| sorted_subset(known, &assertions))
    {
        return InsertSupport::Unchanged;
    }

    let supersets = frontier
        .keys()
        .filter(|known| sorted_subset(&assertions, known))
        .cloned()
        .collect::<Vec<_>>();
    let next_len = frontier.len() + 1 - supersets.len();
    if next_len > max_supports {
        return InsertSupport::BudgetExhausted;
    }
    for superset in supersets {
        frontier.remove(&superset);
    }
    frontier.insert(assertions, proof);
    InsertSupport::Changed
}

fn sorted_subset(left: &[Clause], right: &[Clause]) -> bool {
    let mut right_index = 0;
    for wanted in left {
        while right_index < right.len() && right[right_index] < *wanted {
            right_index += 1;
        }
        if right.get(right_index) != Some(wanted) {
            return false;
        }
        right_index += 1;
    }
    true
}

#[allow(clippy::too_many_arguments)]
fn collect_ground_derivations(
    law: &Law,
    facts: &[Clause],
    limits: &Limits,
    join_attempts: &mut usize,
    derivations: &mut BTreeSet<GroundDerivation>,
    premise_index: usize,
    substitution: BTreeMap<String, String>,
    premises: Vec<Clause>,
) -> Result<()> {
    if premise_index == law.premises().len() {
        derivations.insert(GroundDerivation {
            conclusion: instantiate(law.conclusion(), &substitution),
            law: law.name().to_owned(),
            premises,
            substitution,
        });
        return Ok(());
    }

    let pattern = &law.premises()[premise_index];
    for fact in facts {
        if *join_attempts >= limits.max_join_attempts {
            return Err(limit_error(
                "support join attempt",
                "max_join_attempts",
                limits.max_join_attempts,
            ));
        }
        *join_attempts += 1;
        let Some(next_substitution) = unify(pattern, fact, &substitution) else {
            continue;
        };
        let mut next_premises = premises.clone();
        next_premises.push(fact.clone());
        collect_ground_derivations(
            law,
            facts,
            limits,
            join_attempts,
            derivations,
            premise_index + 1,
            next_substitution,
            next_premises,
        )?;
    }
    Ok(())
}

fn collect_law_candidates(
    law: &Law,
    facts: &[Clause],
    limits: &Limits,
    join_attempts: &mut usize,
    candidates: &mut BTreeMap<Clause, Candidate>,
) -> Result<()> {
    collect_joins(
        law,
        facts,
        limits,
        join_attempts,
        candidates,
        0,
        BTreeMap::new(),
        Vec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn collect_joins(
    law: &Law,
    facts: &[Clause],
    limits: &Limits,
    join_attempts: &mut usize,
    candidates: &mut BTreeMap<Clause, Candidate>,
    premise_index: usize,
    substitution: BTreeMap<String, String>,
    premises: Vec<Clause>,
) -> Result<()> {
    if premise_index == law.premises().len() {
        let conclusion = instantiate(law.conclusion(), &substitution);
        let candidate = Candidate {
            law: law.name().to_owned(),
            premises,
            substitution,
        };
        match candidates.get_mut(&conclusion) {
            Some(chosen) if candidate < *chosen => *chosen = candidate,
            None => {
                candidates.insert(conclusion, candidate);
            }
            _ => {}
        }
        return Ok(());
    }

    let pattern = &law.premises()[premise_index];
    for fact in facts {
        if *join_attempts >= limits.max_join_attempts {
            return Err(limit_error(
                "join attempt",
                "max_join_attempts",
                limits.max_join_attempts,
            ));
        }
        *join_attempts += 1;

        let Some(next_substitution) = unify(pattern, fact, &substitution) else {
            continue;
        };
        let mut next_premises = premises.clone();
        next_premises.push(fact.clone());
        collect_joins(
            law,
            facts,
            limits,
            join_attempts,
            candidates,
            premise_index + 1,
            next_substitution,
            next_premises,
        )?;
    }
    Ok(())
}

fn unify(
    pattern: &Clause,
    fact: &Clause,
    substitution: &BTreeMap<String, String>,
) -> Option<BTreeMap<String, String>> {
    if pattern.relation() != fact.relation() || pattern.roles().len() != fact.roles().len() {
        return None;
    }

    let mut substitution = substitution.clone();
    for (role, pattern_term) in pattern.roles() {
        let fact_term = fact.roles().get(role)?;
        if pattern_term.is_variable() {
            match substitution.get(pattern_term.text()) {
                Some(bound) if bound != fact_term.text() => return None,
                Some(_) => {}
                None => {
                    substitution
                        .insert(pattern_term.text().to_owned(), fact_term.text().to_owned());
                }
            }
        } else if pattern_term.text() != fact_term.text() {
            return None;
        }
    }
    Some(substitution)
}

fn instantiate(pattern: &Clause, substitution: &BTreeMap<String, String>) -> Clause {
    Clause::new(
        pattern.relation(),
        pattern
            .roles()
            .iter()
            .map(|(role, term)| {
                let value = if term.is_variable() {
                    substitution
                        .get(term.text())
                        .expect("admitted law conclusions are range-restricted")
                        .as_str()
                } else {
                    term.text()
                };
                (
                    role.clone(),
                    Term::literal(value).expect("admitted terms remain valid literals"),
                )
            })
            .collect(),
    )
    .expect("instantiating an admitted conclusion preserves its complete role map")
}

fn limit_error(kind: &str, name: &str, value: usize) -> KernelError {
    KernelError::new(format!("closure {kind} limit exceeded ({name}={value})"))
}

#[cfg(test)]
mod tests {
    use super::{
        Limits, SupportLimits, SupportStatus, SupportWitness, Witness, saturate, support_frontier,
    };
    use crate::kernel::{
        Cardinality, Clause, Law, Mode, Model, Relation, Revision, Role, Sentence, Term,
    };
    use std::collections::BTreeSet;

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

    fn literal(relation: &str, from: &str, to: &str) -> Clause {
        clause(
            relation,
            Term::literal(from).unwrap(),
            Term::literal(to).unwrap(),
        )
    }

    fn pattern(relation: &str, from: &str, to: &str) -> Clause {
        clause(
            relation,
            Term::variable(from).unwrap(),
            Term::variable(to).unwrap(),
        )
    }

    fn clause(relation: &str, from: Term, to: Term) -> Clause {
        Clause::new(relation, vec![("from".into(), from), ("to".into(), to)]).unwrap()
    }

    fn revision(facts: Vec<Clause>, laws: Vec<Law>) -> Revision {
        let query = clause(
            "map/reaches",
            Term::literal("North").unwrap(),
            Term::variable("destination").unwrap(),
        );
        Revision::admit(
            Model::with_laws(
                vec![
                    relation("map/links"),
                    relation("map/hosts"),
                    relation("map/reaches"),
                    relation("map/a"),
                    relation("map/b"),
                ],
                facts,
                laws,
                query,
                "ascending",
            )
            .unwrap(),
        )
    }

    fn generous() -> Limits {
        Limits::new(100, 10, 10_000)
    }

    #[test]
    fn multi_round_dependency_closure_has_acyclic_proofs() {
        let seed = literal("map/links", "North", "Store");
        let hosted = literal("map/hosts", "Store", "Beagle");
        let first = Law::new(
            "map/01-link-reaches",
            vec![pattern("map/links", "source", "middle")],
            pattern("map/reaches", "source", "middle"),
        )
        .unwrap();
        let second = Law::new(
            "map/02-hosted-reaches",
            vec![
                pattern("map/reaches", "source", "middle"),
                pattern("map/hosts", "middle", "destination"),
            ],
            pattern("map/reaches", "source", "destination"),
        )
        .unwrap();
        let closure = saturate(
            &revision(vec![seed, hosted], vec![second, first]),
            generous(),
        )
        .expect("finite laws saturate");

        let north_store = literal("map/reaches", "North", "Store");
        let north_beagle = literal("map/reaches", "North", "Beagle");
        assert_eq!(closure.proof(&north_store).unwrap().generation(), 1);
        let proof = closure.proof(&north_beagle).unwrap();
        assert_eq!(proof.generation(), 2);
        match proof.witness() {
            Witness::Derived { law, premises, .. } => {
                assert_eq!(law, "map/02-hosted-reaches");
                assert_eq!(
                    premises,
                    &[north_store, literal("map/hosts", "Store", "Beagle")]
                );
                assert!(premises.iter().all(|premise| {
                    closure.proof(premise).unwrap().generation() < proof.generation()
                }));
            }
            Witness::Asserted => panic!("North to Beagle is derived"),
        }
    }

    #[test]
    fn recursive_cycle_terminates_without_replacing_asserted_proof() {
        let a_to_b = Law::new(
            "map/a-to-b",
            vec![pattern("map/a", "left", "right")],
            pattern("map/b", "left", "right"),
        )
        .unwrap();
        let b_to_a = Law::new(
            "map/b-to-a",
            vec![pattern("map/b", "left", "right")],
            pattern("map/a", "left", "right"),
        )
        .unwrap();
        let asserted = literal("map/a", "North", "Store");
        let closure = saturate(
            &revision(vec![asserted.clone()], vec![b_to_a, a_to_b]),
            generous(),
        )
        .unwrap();

        assert_eq!(closure.facts().len(), 2);
        assert_eq!(
            closure.proof(&asserted).unwrap().witness(),
            &Witness::Asserted
        );
        assert_eq!(
            closure
                .proof(&literal("map/b", "North", "Store"))
                .unwrap()
                .generation(),
            1
        );
    }

    #[test]
    fn competing_witness_uses_lexical_law_key() {
        let later = Law::new(
            "map/z-witness",
            vec![pattern("map/links", "left", "right")],
            pattern("map/reaches", "left", "right"),
        )
        .unwrap();
        let earlier = Law::new(
            "map/a-witness",
            vec![pattern("map/links", "left", "right")],
            pattern("map/reaches", "left", "right"),
        )
        .unwrap();
        let closure = saturate(
            &revision(
                vec![literal("map/links", "North", "Store")],
                vec![later, earlier],
            ),
            generous(),
        )
        .unwrap();

        match closure
            .proof(&literal("map/reaches", "North", "Store"))
            .unwrap()
            .witness()
        {
            Witness::Derived { law, .. } => assert_eq!(law, "map/a-witness"),
            Witness::Asserted => panic!("the reachable fact is derived"),
        }
    }

    #[test]
    fn fact_and_law_permutations_produce_identical_closures() {
        let facts = vec![
            literal("map/links", "North", "Store"),
            literal("map/links", "South", "Store"),
        ];
        let laws = vec![
            Law::new(
                "map/z-copy",
                vec![pattern("map/links", "left", "right")],
                pattern("map/reaches", "left", "right"),
            )
            .unwrap(),
            Law::new(
                "map/a-copy",
                vec![pattern("map/links", "left", "right")],
                pattern("map/reaches", "left", "right"),
            )
            .unwrap(),
        ];
        let mut reversed_facts = facts.clone();
        reversed_facts.reverse();
        let mut reversed_laws = laws.clone();
        reversed_laws.reverse();

        assert_eq!(
            saturate(&revision(facts, laws), generous()).unwrap(),
            saturate(&revision(reversed_facts, reversed_laws), generous()).unwrap()
        );
    }

    #[test]
    fn repeated_saturation_is_idempotent() {
        let revision = revision(
            vec![literal("map/links", "North", "Store")],
            vec![
                Law::new(
                    "map/copy",
                    vec![pattern("map/links", "left", "right")],
                    pattern("map/reaches", "left", "right"),
                )
                .unwrap(),
            ],
        );
        let first = saturate(&revision, generous()).unwrap();
        let second = saturate(&revision, generous()).unwrap();

        assert_eq!(first, second);
        assert_eq!(
            first.facts().iter().collect::<BTreeSet<_>>().len(),
            first.facts().len()
        );
    }

    #[test]
    fn every_limit_has_a_stable_error() {
        let seed = literal("map/links", "North", "Store");
        let law = Law::new(
            "map/copy",
            vec![pattern("map/links", "left", "right")],
            pattern("map/reaches", "left", "right"),
        )
        .unwrap();
        let revision = revision(vec![seed], vec![law]);

        assert_eq!(
            saturate(&revision, Limits::new(0, 10, 100))
                .unwrap_err()
                .to_string(),
            "closure fact limit exceeded (max_facts=0)"
        );
        assert_eq!(
            saturate(&revision, Limits::new(10, 0, 100))
                .unwrap_err()
                .to_string(),
            "closure round limit exceeded (max_rounds=0)"
        );
        assert_eq!(
            saturate(&revision, Limits::new(10, 10, 0))
                .unwrap_err()
                .to_string(),
            "closure join attempt limit exceeded (max_join_attempts=0)"
        );
    }

    #[test]
    fn support_frontier_is_minimal_canonical_recursive_and_visibly_bounded() {
        let north_store = literal("map/links", "North", "Store");
        let store_beagle = literal("map/hosts", "Store", "Beagle");
        let north_beagle = literal("map/links", "North", "Beagle");
        let copy_z = Law::new(
            "map/z-copy",
            vec![pattern("map/links", "source", "destination")],
            pattern("map/reaches", "source", "destination"),
        )
        .unwrap();
        let copy_a = Law::new(
            "map/a-copy",
            vec![pattern("map/links", "source", "destination")],
            pattern("map/reaches", "source", "destination"),
        )
        .unwrap();
        let recursive = Law::new(
            "map/recursive",
            vec![
                pattern("map/reaches", "source", "middle"),
                pattern("map/hosts", "middle", "destination"),
            ],
            pattern("map/reaches", "source", "destination"),
        )
        .unwrap();
        let cycle = Law::new(
            "map/reaches-to-a",
            vec![pattern("map/reaches", "source", "destination")],
            pattern("map/a", "source", "destination"),
        )
        .unwrap();
        let cycle_back = Law::new(
            "map/a-to-reaches",
            vec![pattern("map/a", "source", "destination")],
            pattern("map/reaches", "source", "destination"),
        )
        .unwrap();
        let facts = vec![
            north_store.clone(),
            store_beagle.clone(),
            north_beagle.clone(),
        ];
        let laws = vec![copy_z, cycle, recursive, copy_a, cycle_back];
        let target = literal("map/reaches", "North", "Beagle");
        let limits = SupportLimits::new(generous(), 1_000, 10);
        let first =
            support_frontier(&revision(facts.clone(), laws.clone()), &target, limits).unwrap();

        assert_eq!(first.status(), SupportStatus::Complete);
        assert_eq!(first.supports().len(), 2);
        let mut recursive_support = vec![north_store.clone(), store_beagle.clone()];
        recursive_support.sort();
        let mut expected = vec![vec![north_beagle.clone()], recursive_support];
        expected.sort();
        assert_eq!(
            first
                .supports()
                .iter()
                .map(|support| support.assertions().to_vec())
                .collect::<Vec<_>>(),
            expected
        );
        let direct = first
            .supports()
            .iter()
            .find(|support| support.assertions() == [north_beagle.clone()])
            .unwrap();
        match direct.proof().witness() {
            SupportWitness::Derived { law, .. } => assert_eq!(law, "map/a-copy"),
            SupportWitness::Asserted => panic!("the target itself was not asserted"),
        }

        let mut reversed_facts = facts;
        reversed_facts.reverse();
        let mut reversed_laws = laws;
        reversed_laws.reverse();
        assert_eq!(
            first,
            support_frontier(&revision(reversed_facts, reversed_laws), &target, limits).unwrap()
        );

        let asserted =
            support_frontier(&revision(vec![target.clone()], Vec::new()), &target, limits).unwrap();
        assert_eq!(asserted.supports()[0].assertions(), &[target.clone()]);
        assert_eq!(
            asserted.supports()[0].proof().witness(),
            &SupportWitness::Asserted
        );

        let exhausted = support_frontier(
            &revision(
                vec![north_store, store_beagle, north_beagle],
                vec![
                    Law::new(
                        "map/copy",
                        vec![pattern("map/links", "source", "destination")],
                        pattern("map/reaches", "source", "destination"),
                    )
                    .unwrap(),
                ],
            ),
            &target,
            SupportLimits::new(generous(), 0, 10),
        )
        .unwrap();
        assert_eq!(exhausted.status(), SupportStatus::ExpansionBudgetExhausted);
        assert!(!exhausted.status().is_complete());

        let support_exhausted = support_frontier(
            &revision(vec![target.clone()], Vec::new()),
            &target,
            SupportLimits::new(generous(), 10, 0),
        )
        .unwrap();
        assert_eq!(
            support_exhausted.status(),
            SupportStatus::SupportBudgetExhausted
        );
        assert!(support_exhausted.supports().is_empty());
    }
}
