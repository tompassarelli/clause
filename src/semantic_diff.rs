//! Semantic comparisons of immutable revisions.
//!
//! A semantic diff is deliberately a comparison value only: it is never part
//! of a revision's admitted model or identity.

use crate::{
    delta::RevisionDiff,
    derive::{self, Limits, Proof},
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

/// The authored and entailed differences between same-declaration revisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticDiff {
    authored: RevisionDiff,
    entailed_added: Vec<Clause>,
    entailed_removed: Vec<Clause>,
    changed_proofs: Vec<ProofChange>,
}

impl SemanticDiff {
    /// Compare exact immutable revisions with explicit closure resource bounds.
    ///
    /// `authored` describes asserted changes. Entailed additions and removals
    /// exclude those asserted changes, leaving only their semantic
    /// consequences. Chosen proofs are compared only for facts entailed by
    /// both revisions.
    pub fn between(base: &Revision, successor: &Revision, limits: Limits) -> Result<Self> {
        let authored = RevisionDiff::between(base, successor)?;
        let base_closure = derive::saturate(base, limits)?;
        let successor_closure = derive::saturate(successor, limits)?;

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

        Ok(Self {
            authored,
            entailed_added,
            entailed_removed,
            changed_proofs,
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
}

#[cfg(test)]
mod tests {
    use super::SemanticDiff;
    use crate::{
        delta::RevisionDelta,
        derive::Limits,
        kernel::{Cardinality, Clause, Law, Mode, Model, Relation, Revision, Role, Sentence, Term},
    };

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
        let diff = SemanticDiff::between(&base, &successor, Limits::new(100, 10, 10_000)).unwrap();

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
}
