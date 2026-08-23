//! Semantic comparisons of immutable revisions.
//!
//! A semantic diff is deliberately a comparison value only: it is never part
//! of a revision's admitted model or identity.

use crate::{
    delta::RevisionDiff,
    derive::{self, Proof, Support, SupportFrontier, SupportLimits},
    kernel::{Clause, Result, Revision},
};

/// A selected derivation that changed for a fact entailed by both revisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofChange {
    fact: Clause,
    base: Proof,
    successor: Proof,
}

impl ProofChange {
    pub fn fact(&self) -> &Clause {
        &self.fact
    }

    pub fn base(&self) -> &Proof {
        &self.base
    }

    pub fn successor(&self) -> &Proof {
        &self.successor
    }
}

/// Canonical minimal asserted supports that changed for one shared consequence.
///
/// The frontiers remain attached to make their bounds and completeness explicit:
/// an incomplete frontier is a deterministic prefix, not a claim that no other
/// support exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportChange {
    fact: Clause,
    base: SupportFrontier,
    successor: SupportFrontier,
    added: Vec<Support>,
    removed: Vec<Support>,
}

impl SupportChange {
    pub fn fact(&self) -> &Clause {
        &self.fact
    }

    pub fn base(&self) -> &SupportFrontier {
        &self.base
    }

    pub fn successor(&self) -> &SupportFrontier {
        &self.successor
    }

    pub fn added(&self) -> &[Support] {
        &self.added
    }

    pub fn removed(&self) -> &[Support] {
        &self.removed
    }
}

/// The authored and entailed differences between same-declaration revisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticDiff {
    authored: RevisionDiff,
    entailed_added: Vec<Clause>,
    entailed_removed: Vec<Clause>,
    changed_proofs: Vec<ProofChange>,
    changed_supports: Vec<SupportChange>,
}

impl SemanticDiff {
    /// Compare exact immutable revisions with explicit closure resource bounds.
    ///
    /// `authored` describes asserted changes. Entailed additions and removals
    /// exclude those asserted changes, leaving only their semantic
    /// consequences. Chosen proofs are compared only for facts entailed by
    /// both revisions.
    pub fn between(
        base: &Revision,
        successor: &Revision,
        support_limits: SupportLimits,
    ) -> Result<Self> {
        let authored = RevisionDiff::between(base, successor)?;
        let base_closure = derive::saturate(base, support_limits.closure)?;
        let successor_closure = derive::saturate(successor, support_limits.closure)?;

        let entailed_added = successor_closure
            .facts()
            .iter()
            .filter(|fact| {
                base_closure.facts().binary_search(fact).is_err()
                    && authored.added().binary_search(fact).is_err()
            })
            .cloned()
            .collect();
        let entailed_removed = base_closure
            .facts()
            .iter()
            .filter(|fact| {
                successor_closure.facts().binary_search(fact).is_err()
                    && authored.removed().binary_search(fact).is_err()
            })
            .cloned()
            .collect();
        let changed_proofs = base_closure
            .facts()
            .iter()
            .filter_map(|fact| {
                let successor_proof = successor_closure.proof(fact)?;
                let base_proof = base_closure
                    .proof(fact)
                    .expect("closure facts always have selected proofs");
                (base_proof != successor_proof).then(|| ProofChange {
                    fact: fact.clone(),
                    base: base_proof.clone(),
                    successor: successor_proof.clone(),
                })
            })
            .collect();
        let changed_supports = base_closure
            .facts()
            .iter()
            .filter(|fact| successor_closure.facts().binary_search(fact).is_ok())
            .map(|fact| support_change(base, successor, fact, support_limits))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(Self {
            authored,
            entailed_added,
            entailed_removed,
            changed_proofs,
            changed_supports,
        })
    }

    pub fn authored(&self) -> &RevisionDiff {
        &self.authored
    }

    pub fn entailed_added(&self) -> &[Clause] {
        &self.entailed_added
    }

    pub fn entailed_removed(&self) -> &[Clause] {
        &self.entailed_removed
    }

    pub fn changed_proofs(&self) -> &[ProofChange] {
        &self.changed_proofs
    }

    pub fn changed_supports(&self) -> &[SupportChange] {
        &self.changed_supports
    }
}

fn support_change(
    base_revision: &Revision,
    successor_revision: &Revision,
    fact: &Clause,
    limits: SupportLimits,
) -> Result<Option<SupportChange>> {
    let base = derive::support_frontier(base_revision, fact, limits)?;
    let successor = derive::support_frontier(successor_revision, fact, limits)?;
    let added: Vec<Support> = successor
        .supports()
        .iter()
        .filter(|support| {
            !base
                .supports()
                .iter()
                .any(|candidate| candidate.assertions() == support.assertions())
        })
        .cloned()
        .collect();
    let removed: Vec<Support> = base
        .supports()
        .iter()
        .filter(|support| {
            !successor
                .supports()
                .iter()
                .any(|candidate| candidate.assertions() == support.assertions())
        })
        .cloned()
        .collect();

    if added.is_empty()
        && removed.is_empty()
        && base.status().is_complete()
        && successor.status().is_complete()
    {
        return Ok(None);
    }

    Ok(Some(SupportChange {
        fact: fact.clone(),
        base,
        successor,
        added,
        removed,
    }))
}

#[cfg(test)]
mod tests {
    use super::SemanticDiff;
    use crate::{
        delta::RevisionDelta,
        derive::{Limits, SupportLimits},
        kernel::{Cardinality, Clause, Law, Mode, Model, Relation, Revision, Role, Sentence, Term},
    };

    fn support_limits() -> SupportLimits {
        SupportLimits::new(Limits::new(100, 10, 10_000), 100, 100)
    }

    fn relation(name: &str, roles: &[&str]) -> Relation {
        Relation::new(
            name,
            roles
                .iter()
                .map(|name| Role::new(*name, "Text").unwrap())
                .collect(),
            Sentence::new(roles[0], "relates to", roles[1]).unwrap(),
            vec![
                Mode::finite(
                    vec![roles[0].into()],
                    vec![roles[1].into()],
                    Cardinality::Many,
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    fn fact(relation: &str, roles: &[(&str, &str)]) -> Clause {
        Clause::new(
            relation,
            roles
                .iter()
                .map(|(role, value)| ((*role).into(), Term::literal(*value).unwrap()))
                .collect(),
        )
        .unwrap()
    }

    fn pattern(relation: &str, roles: &[(&str, &str)]) -> Clause {
        Clause::new(
            relation,
            roles
                .iter()
                .map(|(role, variable)| ((*role).into(), Term::variable(*variable).unwrap()))
                .collect(),
        )
        .unwrap()
    }

    fn model() -> Model {
        let imports = |consumer, dependency| {
            pattern(
                "impact/imports",
                &[("consumer", consumer), ("dependency", dependency)],
            )
        };
        let depends = |consumer, dependency| {
            pattern(
                "impact/depends",
                &[("consumer", consumer), ("dependency", dependency)],
            )
        };
        let changes = |change, component| {
            pattern(
                "impact/changes",
                &[("change", change), ("component", component)],
            )
        };
        let affected = |change, consumer| {
            pattern(
                "impact/affected",
                &[("change", change), ("consumer", consumer)],
            )
        };
        Model::with_laws(
            vec![
                relation("impact/imports", &["consumer", "dependency"]),
                relation("impact/depends", &["consumer", "dependency"]),
                relation("impact/changes", &["change", "component"]),
                relation("impact/affected", &["change", "consumer"]),
            ],
            vec![
                fact(
                    "impact/imports",
                    &[("consumer", "North"), ("dependency", "Store")],
                ),
                fact(
                    "impact/imports",
                    &[("consumer", "Store"), ("dependency", "Beagle")],
                ),
                fact(
                    "impact/changes",
                    &[("change", "compiler-change"), ("component", "Beagle")],
                ),
            ],
            vec![
                Law::new(
                    "impact/direct-dependency",
                    vec![imports("consumer", "dependency")],
                    depends("consumer", "dependency"),
                )
                .unwrap(),
                Law::new(
                    "impact/recursive-dependency",
                    vec![
                        imports("consumer", "dependency"),
                        depends("dependency", "transitive"),
                    ],
                    depends("consumer", "transitive"),
                )
                .unwrap(),
                Law::new(
                    "impact/impact",
                    vec![
                        changes("change", "component"),
                        depends("consumer", "component"),
                    ],
                    affected("change", "consumer"),
                )
                .unwrap(),
            ],
            Clause::new(
                "impact/imports",
                vec![
                    ("consumer".into(), Term::literal("North").unwrap()),
                    ("dependency".into(), Term::variable("dependency").unwrap()),
                ],
            )
            .unwrap(),
            "ascending",
        )
        .unwrap()
    }

    fn support_fact(relation: &str) -> Clause {
        fact(relation, &[("consumer", "North"), ("dependency", "Beagle")])
    }

    fn support_model(facts: Vec<Clause>, reverse_laws: bool) -> Model {
        let input = |relation, consumer, dependency| {
            pattern(
                relation,
                &[("consumer", consumer), ("dependency", dependency)],
            )
        };
        let mut laws = vec![
            Law::new(
                "impact/left-support",
                vec![input("impact/left", "consumer", "dependency")],
                input("impact/result", "consumer", "dependency"),
            )
            .unwrap(),
            Law::new(
                "impact/right-support",
                vec![input("impact/right", "consumer", "dependency")],
                input("impact/result", "consumer", "dependency"),
            )
            .unwrap(),
        ];
        if reverse_laws {
            laws.reverse();
        }
        Model::with_laws(
            vec![
                relation("impact/left", &["consumer", "dependency"]),
                relation("impact/right", &["consumer", "dependency"]),
                relation("impact/result", &["consumer", "dependency"]),
            ],
            facts,
            laws,
            Clause::new(
                "impact/result",
                vec![
                    ("consumer".into(), Term::literal("North").unwrap()),
                    ("dependency".into(), Term::variable("dependency").unwrap()),
                ],
            )
            .unwrap(),
            "ascending",
        )
        .unwrap()
    }

    #[test]
    fn south_to_north_distinguishes_authored_and_entailed_additions() {
        let base = Revision::admit(model());
        let successor = RevisionDelta::new(
            base.identity(),
            vec![fact(
                "impact/imports",
                &[("consumer", "South"), ("dependency", "North")],
            )],
            Vec::new(),
        )
        .unwrap()
        .apply(&base)
        .unwrap();
        let diff = SemanticDiff::between(&base, &successor, support_limits()).unwrap();

        assert_eq!(diff.authored().added().len(), 1);
        assert_eq!(diff.authored().removed(), []);
        assert_eq!(
            diff.entailed_added(),
            [
                fact(
                    "impact/affected",
                    &[("change", "compiler-change"), ("consumer", "South")]
                ),
                fact(
                    "impact/depends",
                    &[("consumer", "South"), ("dependency", "Beagle")]
                ),
                fact(
                    "impact/depends",
                    &[("consumer", "South"), ("dependency", "North")]
                ),
                fact(
                    "impact/depends",
                    &[("consumer", "South"), ("dependency", "Store")]
                ),
            ]
        );
        assert_eq!(diff.entailed_removed(), []);
        assert_eq!(diff.changed_proofs(), []);
    }

    #[test]
    fn support_loss_remains_visible_when_entailment_is_unchanged() {
        let left = support_fact("impact/left");
        let right = support_fact("impact/right");
        let target = support_fact("impact/result");
        let base = Revision::admit(support_model(vec![left.clone(), right.clone()], false));
        let successor = RevisionDelta::new(base.identity(), Vec::new(), vec![left.clone()])
            .unwrap()
            .apply(&base)
            .unwrap();

        let diff = SemanticDiff::between(&base, &successor, support_limits()).unwrap();

        assert!(diff.entailed_removed().is_empty());
        let change = diff
            .changed_supports()
            .iter()
            .find(|change| change.fact() == &target)
            .expect("the retained consequence exposes its lost support");
        assert_eq!(change.added(), []);
        assert_eq!(change.removed().len(), 1);
        assert_eq!(change.removed()[0].assertions(), &[left]);
        assert!(change.base().status().is_complete());
        assert!(change.successor().status().is_complete());
    }

    #[test]
    fn support_gain_does_not_require_a_new_consequence() {
        let left = support_fact("impact/left");
        let right = support_fact("impact/right");
        let target = support_fact("impact/result");
        let base = Revision::admit(support_model(vec![left], false));
        let successor = RevisionDelta::new(base.identity(), vec![right.clone()], Vec::new())
            .unwrap()
            .apply(&base)
            .unwrap();

        let diff = SemanticDiff::between(&base, &successor, support_limits()).unwrap();

        assert!(diff.entailed_added().is_empty());
        let change = diff
            .changed_supports()
            .iter()
            .find(|change| change.fact() == &target)
            .expect("the existing consequence exposes its gained support");
        assert_eq!(change.removed(), []);
        assert_eq!(change.added().len(), 1);
        assert_eq!(change.added()[0].assertions(), &[right]);
    }

    #[test]
    fn support_changes_are_deterministic_under_reordering() {
        let left = support_fact("impact/left");
        let right = support_fact("impact/right");
        let base = Revision::admit(support_model(vec![left.clone(), right.clone()], false));
        let reordered = Revision::admit(support_model(vec![right.clone(), left.clone()], true));
        let successor = RevisionDelta::new(base.identity(), Vec::new(), vec![left.clone()])
            .unwrap()
            .apply(&base)
            .unwrap();
        let reordered_successor = RevisionDelta::new(reordered.identity(), Vec::new(), vec![left])
            .unwrap()
            .apply(&reordered)
            .unwrap();

        assert_eq!(base, reordered);
        assert_eq!(
            SemanticDiff::between(&base, &successor, support_limits()).unwrap(),
            SemanticDiff::between(&reordered, &reordered_successor, support_limits()).unwrap()
        );
    }
}
