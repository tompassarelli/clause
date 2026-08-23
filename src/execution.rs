//! Typed evaluation projections over one sealed Revision.
//!
//! Requests live outside the semantic Model.  This module evaluates the typed
//! `FindPlan` and projects either the canonical chosen proof or the bounded
//! minimal-support frontier; presentation and result encoding belong to the
//! request layer.

use crate::{
    derive::{self, Closure, Limits, SupportLimits, SupportProof, SupportWitness},
    kernel::{Clause, KernelError, LawId, Result, Revision, RevisionId, Term, VariableId},
};
use std::collections::BTreeMap;

/// A ground clause in a revision-scoped explanation graph.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ClauseNode {
    pub clause: Clause,
}

/// One canonical witness for a derived or asserted clause.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Witness {
    Asserted,
    Derived {
        law: LawId,
        premises: Vec<usize>,
        substitution: BTreeMap<VariableId, Term>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WitnessEdge {
    pub conclusion: usize,
    pub witness: Witness,
}

/// An acyclic, canonical proof projection.  Node indices address `nodes`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WhyGraph {
    pub root: usize,
    pub nodes: Vec<ClauseNode>,
    pub witnesses: Vec<WitnessEdge>,
}

/// One canonical proof, explicitly scoped to the Revision that admitted it.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Proof {
    pub revision: RevisionId,
    pub why: WhyGraph,
}

/// One inclusion-minimal asserted support and its exact derivation.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WhySupport {
    pub assertions: Vec<Clause>,
    pub proof: Proof,
}

/// The bounded projection of every discovered inclusion-minimal support.
///
/// `complete` is true only when the support engine exhausted the admitted
/// finite search.  An empty, incomplete frontier is intentionally distinct
/// from a complete proof of no support.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WhyAll {
    pub revision: RevisionId,
    pub target: Clause,
    pub alternatives: Vec<WhySupport>,
    pub complete: bool,
    pub expansions: usize,
}

impl WhyAll {
    pub fn alternative_count(&self) -> usize {
        self.alternatives.len()
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }
}

/// Evaluate the complete bounded derived closure and return canonical typed
/// bindings for the sole sought role in `plan`.
///
/// The complete retained pattern is matched role-by-role.  In particular, two
/// otherwise identical orientations with distinct known entities cannot share
/// results merely because their `known` role sets are the same.
pub fn find(
    revision: &Revision,
    plan: &crate::kernel::FindPlan,
    limits: Limits,
) -> Result<Vec<Term>> {
    revision.model().validate_clause(plan.pattern(), true)?;
    let sought = plan.sought();
    let sought_variable = plan
        .pattern()
        .roles()
        .get(sought)
        .and_then(Term::variable_id)
        .ok_or_else(|| KernelError::new("find plan sought role is not a variable"))?;
    let closure = derive::saturate(revision, limits)?;
    let mut bindings = closure
        .assertions()
        .iter()
        .filter(|candidate| matches_pattern(candidate, plan.pattern(), sought_variable))
        .map(|candidate| {
            candidate
                .roles()
                .get(sought)
                .cloned()
                .ok_or_else(|| KernelError::new("closure clause does not fill sought role"))
        })
        .collect::<Result<Vec<_>>>()?;
    bindings.sort();
    bindings.dedup();
    Ok(bindings)
}

/// Return the deterministic chosen proof for a ground target, if it follows.
pub fn why(revision: &Revision, target: &Clause, limits: Limits) -> Result<Option<Proof>> {
    revision.model().validate_clause(target, false)?;
    let closure = derive::saturate(revision, limits)?;
    graph(&closure, target, revision.identity().clone())
}

/// Return every discovered minimal asserted support for a ground target.
///
/// The complete closure is checked first, so a bounded support search can
/// honestly return `Some(WhyAll { complete: false, alternatives: [] })` for an
/// entailed target whose support frontier was not reached before its budget.
pub fn why_all(
    revision: &Revision,
    target: &Clause,
    limits: SupportLimits,
) -> Result<Option<WhyAll>> {
    revision.model().validate_clause(target, false)?;
    let closure = derive::saturate(revision, limits.closure)?;
    if closure.proof(target).is_none() {
        return Ok(None);
    }
    let frontier = derive::support_frontier(revision, target, limits)?;
    let revision_id = revision.identity().clone();
    let alternatives = frontier
        .supports()
        .iter()
        .map(|support| {
            Ok(WhySupport {
                assertions: support.assertions().to_vec(),
                proof: Proof {
                    revision: revision_id.clone(),
                    why: support_graph(support.proof())?,
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(WhyAll {
        revision: revision_id,
        target: target.clone(),
        alternatives,
        complete: frontier.status().is_complete(),
        expansions: frontier.expansions(),
    }))
}

fn matches_pattern(candidate: &Clause, pattern: &Clause, sought: &VariableId) -> bool {
    candidate.relation() == pattern.relation()
        && pattern.roles().iter().all(|(role, expected)| {
            let Some(actual) = candidate.roles().get(role) else {
                return false;
            };
            match expected.variable_id() {
                Some(variable) => variable == sought,
                None => actual == expected,
            }
        })
}

fn graph(closure: &Closure, root: &Clause, revision: RevisionId) -> Result<Option<Proof>> {
    if closure.proof(root).is_none() {
        return Ok(None);
    }
    let mut clauses = Vec::new();
    let mut indices = BTreeMap::new();
    let mut witnesses = Vec::new();
    let root = add_clause(root, closure, &mut clauses, &mut indices, &mut witnesses)?;
    witnesses.sort_by_key(|edge| edge.conclusion);
    Ok(Some(Proof {
        revision,
        why: WhyGraph {
            root,
            nodes: clauses
                .into_iter()
                .map(|clause| ClauseNode { clause })
                .collect(),
            witnesses,
        },
    }))
}

fn support_graph(root: &SupportProof) -> Result<WhyGraph> {
    let mut clauses = Vec::new();
    let mut indices = BTreeMap::new();
    let mut witnesses = Vec::new();
    let root = add_support_clause(root, &mut clauses, &mut indices, &mut witnesses)?;
    witnesses.sort_by_key(|edge| edge.conclusion);
    Ok(WhyGraph {
        root,
        nodes: clauses
            .into_iter()
            .map(|clause| ClauseNode { clause })
            .collect(),
        witnesses,
    })
}

fn add_support_clause(
    proof: &SupportProof,
    clauses: &mut Vec<Clause>,
    indices: &mut BTreeMap<Clause, usize>,
    witnesses: &mut Vec<WitnessEdge>,
) -> Result<usize> {
    let clause = proof.conclusion();
    if let Some(index) = indices.get(clause) {
        return Ok(*index);
    }
    let conclusion = clauses.len();
    clauses.push(clause.clone());
    indices.insert(clause.clone(), conclusion);
    let witness = match proof.witness() {
        SupportWitness::Asserted => Witness::Asserted,
        SupportWitness::Derived {
            law,
            premises,
            substitution,
        } => Witness::Derived {
            law: law.clone(),
            premises: premises
                .iter()
                .map(|premise| add_support_clause(premise, clauses, indices, witnesses))
                .collect::<Result<Vec<_>>>()?,
            substitution: substitution.clone(),
        },
    };
    witnesses.push(WitnessEdge {
        conclusion,
        witness,
    });
    Ok(conclusion)
}

fn add_clause(
    clause: &Clause,
    closure: &Closure,
    clauses: &mut Vec<Clause>,
    indices: &mut BTreeMap<Clause, usize>,
    witnesses: &mut Vec<WitnessEdge>,
) -> Result<usize> {
    if let Some(index) = indices.get(clause) {
        return Ok(*index);
    }
    let conclusion = clauses.len();
    clauses.push(clause.clone());
    indices.insert(clause.clone(), conclusion);
    let proof = closure
        .proof(clause)
        .ok_or_else(|| KernelError::new("closure clause has no chosen witness"))?;
    let witness = match proof.witness() {
        derive::Witness::Asserted => Witness::Asserted,
        derive::Witness::Derived {
            law,
            premises,
            substitution,
        } => Witness::Derived {
            law: law.clone(),
            premises: premises
                .iter()
                .map(|premise| add_clause(premise, closure, clauses, indices, witnesses))
                .collect::<Result<Vec<_>>>()?,
            substitution: substitution.clone(),
        },
    };
    witnesses.push(WitnessEdge {
        conclusion,
        witness,
    });
    Ok(conclusion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        kernel::{
            Cardinality, EntityId, InlineSentencePart, Law, Mode, Model, ModelId, Name, Relation,
            RelationId, Role, RoleId, SentenceShape, Type, TypeId,
        },
        wire,
    };
    use std::collections::BTreeMap;

    fn name(value: &str) -> Name {
        Name::new(value.to_owned()).unwrap()
    }
    fn type_id(value: &str) -> TypeId {
        TypeId::new(name(value)).unwrap()
    }
    fn relation_id(value: &str) -> RelationId {
        RelationId::new(name(value)).unwrap()
    }
    fn role_id(value: &str) -> RoleId {
        RoleId::new(name(value)).unwrap()
    }
    fn variable_id(value: &str) -> VariableId {
        VariableId::new(name(value)).unwrap()
    }
    fn entity(model: &ModelId, local: &str, typ: &TypeId) -> Term {
        Term::entity(EntityId::new(model.clone(), name(local), typ.clone()).unwrap())
    }
    fn variable(local: &str, typ: &TypeId) -> Term {
        Term::variable(variable_id(local), typ.clone())
    }
    fn clause(relation: &RelationId, from: Term, to: Term) -> Clause {
        Clause::new(
            relation.clone(),
            BTreeMap::from([(role_id("from"), from), (role_id("to"), to)]),
        )
        .unwrap()
    }
    fn relation(id: &RelationId, typ: &TypeId) -> Relation {
        let from = Role::new(role_id("from"), typ.clone());
        let to = Role::new(role_id("to"), typ.clone());
        Relation::new(
            id.clone(),
            SentenceShape::new(vec![
                InlineSentencePart::Role(from.clone()),
                InlineSentencePart::Literal("reaches".to_owned()),
                InlineSentencePart::Role(to.clone()),
            ])
            .unwrap(),
            vec![
                Mode::finite(
                    vec![from.id().clone()],
                    vec![to.id().clone()],
                    Cardinality::Many,
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }
    fn revision(assertions: Vec<Clause>, laws: Vec<Law>) -> Revision {
        let model = ModelId::new(name("map")).unwrap();
        let module = type_id("Module");
        let links = relation_id("map/links");
        let reaches = relation_id("map/reaches");
        let entities = ["North", "South", "Store", "Relay", "Beagle"]
            .into_iter()
            .map(|local| EntityId::new(model.clone(), name(local), module.clone()).unwrap())
            .collect();
        wire::admit(
            Model::new(
                model,
                BTreeMap::from([(module.clone(), Type::new(module.clone()))]),
                entities,
                BTreeMap::from([
                    (links.clone(), relation(&links, &module)),
                    (reaches.clone(), relation(&reaches, &module)),
                ]),
                assertions,
                laws,
            )
            .unwrap(),
        )
    }
    fn limits() -> Limits {
        Limits::new(100, 10, 10_000)
    }
    fn chain_laws() -> Vec<Law> {
        let module = type_id("Module");
        let links = relation_id("map/links");
        let reaches = relation_id("map/reaches");
        let source = variable("source", &module);
        let middle = variable("middle", &module);
        let destination = variable("destination", &module);
        vec![
            Law::new(
                LawId::new(name("map/direct")).unwrap(),
                vec![clause(&links, source.clone(), destination.clone())],
                clause(&reaches, source.clone(), destination.clone()),
            )
            .unwrap(),
            Law::new(
                LawId::new(name("map/recursive")).unwrap(),
                vec![
                    clause(&reaches, source.clone(), middle.clone()),
                    clause(&links, middle, destination.clone()),
                ],
                clause(&reaches, source, destination),
            )
            .unwrap(),
        ]
    }
    fn asserted(relation: &str, from: &str, to: &str) -> Clause {
        let model = ModelId::new(name("map")).unwrap();
        let module = type_id("Module");
        clause(
            &relation_id(relation),
            entity(&model, from, &module),
            entity(&model, to, &module),
        )
    }
    fn find_plan(revision: &Revision, from: &str) -> crate::kernel::FindPlan {
        let model = ModelId::new(name("map")).unwrap();
        let module = type_id("Module");
        let target = variable_id("target");
        crate::kernel::FindPlan::new(
            revision.model(),
            &clause(
                &relation_id("map/reaches"),
                entity(&model, from, &module),
                Term::variable(target.clone(), module),
            ),
            target,
        )
        .unwrap()
    }

    #[test]
    fn find_discriminates_known_entity_bindings_and_returns_typed_terms() {
        let revision = revision(
            vec![
                asserted("map/links", "North", "Store"),
                asserted("map/links", "South", "Relay"),
            ],
            chain_laws(),
        );
        assert_eq!(
            find(&revision, &find_plan(&revision, "North"), limits()).unwrap(),
            vec![entity(
                &ModelId::new(name("map")).unwrap(),
                "Store",
                &type_id("Module")
            )]
        );
        assert_eq!(
            find(&revision, &find_plan(&revision, "South"), limits()).unwrap(),
            vec![entity(
                &ModelId::new(name("map")).unwrap(),
                "Relay",
                &type_id("Module")
            )]
        );
    }

    #[test]
    fn find_returns_recursive_derived_bindings_in_canonical_order() {
        let revision = revision(
            vec![
                asserted("map/links", "North", "Store"),
                asserted("map/links", "Store", "Beagle"),
            ],
            chain_laws(),
        );
        let result = find(&revision, &find_plan(&revision, "North"), limits()).unwrap();
        assert_eq!(
            result,
            vec![
                entity(
                    &ModelId::new(name("map")).unwrap(),
                    "Beagle",
                    &type_id("Module")
                ),
                entity(
                    &ModelId::new(name("map")).unwrap(),
                    "Store",
                    &type_id("Module")
                ),
            ]
        );
        assert!(result.iter().all(|term| matches!(term, Term::Entity(_))));
    }

    #[test]
    fn why_projects_one_canonical_revision_scoped_proof() {
        let revision = revision(vec![asserted("map/links", "North", "Store")], chain_laws());
        let target = asserted("map/reaches", "North", "Store");
        let proof = why(&revision, &target, limits()).unwrap().unwrap();
        assert_eq!(proof.revision, *revision.identity());
        assert_eq!(proof.why.root, 0);
        assert!(
            matches!(proof.why.witnesses[0].witness, Witness::Derived { ref law, .. } if law.as_str() == "map/direct")
        );
    }

    #[test]
    fn why_all_projects_two_independent_minimal_supports() {
        let revision = revision(
            vec![
                asserted("map/links", "North", "Store"),
                asserted("map/links", "Store", "Beagle"),
                asserted("map/links", "North", "Relay"),
                asserted("map/links", "Relay", "Beagle"),
            ],
            chain_laws(),
        );
        let all = why_all(
            &revision,
            &asserted("map/reaches", "North", "Beagle"),
            SupportLimits::new(limits(), 100, 10),
        )
        .unwrap()
        .unwrap();
        assert!(all.is_complete());
        assert_eq!(all.alternative_count(), 2);
        assert!(
            all.alternatives
                .iter()
                .all(|alternative| alternative.assertions.len() == 2)
        );
    }

    #[test]
    fn why_all_marks_a_bounded_frontier_incomplete() {
        let revision = revision(vec![asserted("map/links", "North", "Store")], chain_laws());
        let all = why_all(
            &revision,
            &asserted("map/reaches", "North", "Store"),
            SupportLimits::new(limits(), 0, 10),
        )
        .unwrap()
        .unwrap();
        assert!(!all.is_complete());
        assert!(all.alternatives.is_empty());
    }

    #[test]
    fn proof_is_deterministic_when_assertion_order_changes() {
        let assertions = vec![
            asserted("map/links", "North", "Store"),
            asserted("map/links", "Store", "Beagle"),
        ];
        let target = asserted("map/reaches", "North", "Beagle");
        let forward = revision(assertions.clone(), chain_laws());
        let reverse = revision(assertions.into_iter().rev().collect(), chain_laws());
        assert_eq!(
            why(&forward, &target, limits()).unwrap().unwrap().why,
            why(&reverse, &target, limits()).unwrap().unwrap().why
        );
    }
}
