//! Deterministic finite closure for admitted positive laws.

use crate::kernel::{Clause, KernelError, Law, Result, Revision, Term};
use std::collections::BTreeMap;

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
    use super::{Limits, Witness, saturate};
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
}
