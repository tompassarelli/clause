//! Semantic comparisons of immutable revisions.
//!
//! A semantic diff is deliberately a comparison value only: it is never part
//! of a revision's admitted model or identity.

use std::collections::BTreeSet;

use crate::{
    delta::RevisionDiff,
    derive::{self, Proof, Support, SupportFrontier, SupportLimits},
    kernel::{Clause, Result, Revision},
};

/// A selected derivation that changed for a consequence entailed by both revisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofChange {
    consequence: Clause,
    base: Proof,
    successor: Proof,
}

impl ProofChange {
    pub fn consequence(&self) -> &Clause {
        &self.consequence
    }

    pub fn base(&self) -> &Proof {
        &self.base
    }

    pub fn successor(&self) -> &Proof {
        &self.successor
    }
}

/// Canonical minimal asserted supports that changed for one consequence.
///
/// The frontiers remain attached to make their bounds and completeness explicit:
/// an incomplete frontier is a deterministic prefix, not a claim that no other
/// support exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportChange {
    consequence: Clause,
    base: SupportFrontier,
    successor: SupportFrontier,
    added: Vec<Support>,
    removed: Vec<Support>,
    retained: Vec<Support>,
}

impl SupportChange {
    pub fn consequence(&self) -> &Clause {
        &self.consequence
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

    /// Supports witnessed in both frontiers by the same asserted-clause set.
    ///
    /// This is positive evidence only: an incomplete frontier does not claim
    /// that these are every retained support.
    pub fn retained(&self) -> &[Support] {
        &self.retained
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
    /// consequences. Chosen proofs are compared only for clauses entailed by
    /// both revisions. Support changes cover the canonical union of both
    /// closures, including appearing and disappearing consequences.
    pub fn between(
        base: &Revision,
        successor: &Revision,
        support_limits: SupportLimits,
    ) -> Result<Self> {
        let authored = RevisionDiff::between(base, successor)?;
        let base_closure = derive::saturate(base, support_limits.closure)?;
        let successor_closure = derive::saturate(successor, support_limits.closure)?;

        let entailed_added = successor_closure
            .assertions()
            .iter()
            .filter(|consequence| {
                base_closure
                    .assertions()
                    .binary_search(consequence)
                    .is_err()
                    && authored.added().binary_search(consequence).is_err()
            })
            .cloned()
            .collect();
        let entailed_removed = base_closure
            .assertions()
            .iter()
            .filter(|consequence| {
                successor_closure
                    .assertions()
                    .binary_search(consequence)
                    .is_err()
                    && authored.removed().binary_search(consequence).is_err()
            })
            .cloned()
            .collect();
        let changed_proofs = base_closure
            .assertions()
            .iter()
            .filter_map(|consequence| {
                let successor_proof = successor_closure.proof(consequence)?;
                let base_proof = base_closure
                    .proof(consequence)
                    .expect("closure clauses always have selected proofs");
                (base_proof != successor_proof).then(|| ProofChange {
                    consequence: consequence.clone(),
                    base: base_proof.clone(),
                    successor: successor_proof.clone(),
                })
            })
            .collect();
        let changed_supports = base_closure
            .assertions()
            .iter()
            .chain(successor_closure.assertions())
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            // Assertion deltas are their own layer. Repeating a directly
            // admitted or withdrawn clause as a one-clause support change
            // would make the semantic layer duplicate authored history.
            .filter(|consequence| {
                authored.added().binary_search(consequence).is_err()
                    && authored.removed().binary_search(consequence).is_err()
            })
            .map(|consequence| support_change(base, successor, &consequence, support_limits))
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
    consequence: &Clause,
    limits: SupportLimits,
) -> Result<Option<SupportChange>> {
    let base = derive::support_frontier(base_revision, consequence, limits)?;
    let successor = derive::support_frontier(successor_revision, consequence, limits)?;
    let retained = base
        .supports()
        .iter()
        .filter(|support| {
            successor
                .supports()
                .iter()
                .any(|candidate| candidate.assertion_key() == support.assertion_key())
        })
        .cloned()
        .collect();
    // A gain is exact only if the base frontier proved that support absent.
    let added: Vec<Support> = if base.status().is_complete() {
        successor
            .supports()
            .iter()
            .filter(|support| {
                !base
                    .supports()
                    .iter()
                    .any(|candidate| candidate.assertion_key() == support.assertion_key())
            })
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    // A loss is exact only if the successor frontier proved that support absent.
    let removed: Vec<Support> = if successor.status().is_complete() {
        base.supports()
            .iter()
            .filter(|support| {
                !successor
                    .supports()
                    .iter()
                    .any(|candidate| candidate.assertion_key() == support.assertion_key())
            })
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    // An incomplete projection with no observed delta is unknown, not a
    // changed support frontier. Emit a change only when a gain or loss is
    // positively witnessed; its frontier statuses retain the exact bounds.
    if added.is_empty() && removed.is_empty() {
        return Ok(None);
    }

    Ok(Some(SupportChange {
        consequence: consequence.clone(),
        base,
        successor,
        added,
        removed,
        retained,
    }))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::SemanticDiff;
    use crate::{
        derive::{Limits, SupportLimits, SupportStatus},
        kernel::{
            Cardinality, Clause, Delta, EntityId, InlineSentencePart, Law, Mode, Model, ModelId,
            Name, Relation, RelationId, Role, RoleId, SentenceShape, Term, Type, TypeId,
            VariableId,
        },
        wire,
    };

    fn name(value: &str) -> Name {
        Name::new(value.to_owned()).unwrap()
    }
    fn model_id() -> ModelId {
        ModelId::new(name("impact")).unwrap()
    }
    fn module() -> TypeId {
        TypeId::new(name("Module")).unwrap()
    }
    fn relation_id(value: &str) -> RelationId {
        RelationId::new(name(value)).unwrap()
    }
    fn role(value: &str) -> RoleId {
        RoleId::new(name(value)).unwrap()
    }
    fn variable(value: &str) -> VariableId {
        VariableId::new(name(value)).unwrap()
    }
    fn entity(value: &str) -> EntityId {
        EntityId::new(model_id(), name(value), module()).unwrap()
    }

    fn limits() -> SupportLimits {
        SupportLimits::new(Limits::new(100, 10, 10_000), 100, 100)
    }

    fn source(relation: &str) -> Relation {
        let subject = Role::new(role("subject"), module());
        let object = Role::new(role("object"), module());
        Relation::new(
            relation_id(relation),
            SentenceShape::new(vec![
                InlineSentencePart::Role(subject),
                InlineSentencePart::Literal("reaches".into()),
                InlineSentencePart::Role(object),
            ])
            .unwrap(),
            vec![
                Mode::finite(
                    vec![role("subject")],
                    vec![role("object")],
                    Cardinality::Many,
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    fn clause(relation: &str) -> Clause {
        Clause::new(
            relation_id(relation),
            BTreeMap::from([
                (role("subject"), Term::entity(entity("North"))),
                (role("object"), Term::entity(entity("Beagle"))),
            ]),
        )
        .unwrap()
    }

    fn pattern(relation: &str) -> Clause {
        Clause::new(
            relation_id(relation),
            BTreeMap::from([
                (
                    role("subject"),
                    Term::variable(variable("subject"), module()),
                ),
                (role("object"), Term::variable(variable("object"), module())),
            ]),
        )
        .unwrap()
    }

    fn model(assertions: Vec<Clause>, reverse_laws: bool) -> Model {
        let mut laws = ["left", "middle", "right"]
            .into_iter()
            .map(|side| {
                Law::new(
                    crate::kernel::LawId::new(name(&format!("impact/{side}-support"))).unwrap(),
                    vec![pattern(&format!("impact/{side}"))],
                    pattern("impact/result"),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        if reverse_laws {
            laws.reverse();
        }
        let relations = ["left", "middle", "right", "result"]
            .into_iter()
            .map(|side| {
                let identity = relation_id(&format!("impact/{side}"));
                (identity, source(&format!("impact/{side}")))
            })
            .collect();
        Model::new(
            model_id(),
            BTreeMap::from([(module(), Type::new(module()))]),
            BTreeSet::from([entity("North"), entity("Beagle")]),
            relations,
            assertions,
            laws,
        )
        .unwrap()
    }

    fn revision(assertions: Vec<Clause>, reverse_laws: bool) -> crate::kernel::Revision {
        wire::admit(model(assertions, reverse_laws))
    }

    fn successor(
        base: &crate::kernel::Revision,
        admissions: Vec<Clause>,
        withdrawals: Vec<Clause>,
    ) -> crate::kernel::Revision {
        Delta::new(base.identity().clone(), admissions, withdrawals)
            .unwrap()
            .apply(base)
            .unwrap()
    }

    fn change(diff: &SemanticDiff) -> &super::SupportChange {
        diff.changed_supports()
            .iter()
            .find(|change| change.consequence() == &clause("impact/result"))
            .unwrap()
    }

    #[test]
    fn keeps_asserted_and_entailed_layers_separate() {
        let left = clause("impact/left");
        let base = revision(vec![], false);
        let successor = successor(&base, vec![left.clone()], vec![]);
        let diff = SemanticDiff::between(&base, &successor, limits()).unwrap();
        assert_eq!(diff.authored().added(), &[left]);
        assert_eq!(diff.entailed_added(), &[clause("impact/result")]);
    }

    #[test]
    fn retained_consequence_exposes_lost_and_retained_supports() {
        let left = clause("impact/left");
        let right = clause("impact/right");
        let base = revision(vec![left.clone(), right.clone()], false);
        let successor = successor(&base, vec![], vec![left.clone()]);
        let diff = SemanticDiff::between(&base, &successor, limits()).unwrap();
        let change = change(&diff);
        assert_eq!(change.removed()[0].assertions(), &[left]);
        assert_eq!(change.retained()[0].assertions(), &[right]);
    }

    #[test]
    fn disappearing_consequence_retains_its_support_loss() {
        let left = clause("impact/left");
        let base = revision(vec![left.clone()], false);
        let successor = successor(&base, vec![], vec![left.clone()]);
        let diff = SemanticDiff::between(&base, &successor, limits()).unwrap();
        assert_eq!(diff.entailed_removed(), &[clause("impact/result")]);
        assert_eq!(change(&diff).removed()[0].assertions(), &[left]);
    }

    #[test]
    fn appearing_consequence_retains_its_support_gain() {
        let left = clause("impact/left");
        let base = revision(vec![], false);
        let successor = successor(&base, vec![left.clone()], vec![]);
        let diff = SemanticDiff::between(&base, &successor, limits()).unwrap();
        assert_eq!(change(&diff).added()[0].assertions(), &[left]);
    }

    #[test]
    fn incomplete_opposite_frontier_withholds_removal_claims() {
        let left = clause("impact/left");
        let right = clause("impact/right");
        let middle = clause("impact/middle");
        let base = revision(vec![left.clone()], false);
        let successor = successor(&base, vec![right, middle], vec![left]);
        let bounded = SupportLimits::new(Limits::new(100, 10, 10_000), 100, 1);
        let diff = SemanticDiff::between(&base, &successor, bounded).unwrap();
        let change = change(&diff);
        assert_eq!(
            change.successor().status(),
            SupportStatus::SupportBudgetExhausted
        );
        assert!(change.removed().is_empty());
    }

    #[test]
    fn reports_only_nonduplicate_observed_support_changes() {
        let left = clause("impact/left");
        let right = clause("impact/right");
        let base = revision(vec![left.clone(), right], false);
        let successor = successor(&base, vec![], vec![left.clone()]);
        let exact = SemanticDiff::between(&base, &successor, limits()).unwrap();
        assert_eq!(exact.authored().removed(), std::slice::from_ref(&left));
        assert!(
            exact
                .changed_supports()
                .iter()
                .all(|change| change.consequence() != &left)
        );

        let bounded = SupportLimits::new(Limits::new(100, 10, 10_000), 100, 1);
        let unknown = SemanticDiff::between(&base, &base, bounded).unwrap();
        assert!(unknown.changed_supports().is_empty());
    }

    #[test]
    fn support_projection_is_deterministic_under_declaration_reordering() {
        let left = clause("impact/left");
        let right = clause("impact/right");
        let base = revision(vec![left.clone(), right.clone()], false);
        let reordered = revision(vec![right, left.clone()], true);
        let base_successor = successor(&base, vec![], vec![left.clone()]);
        let reordered_successor = successor(&reordered, vec![], vec![left]);
        assert_eq!(base, reordered);
        assert_eq!(
            SemanticDiff::between(&base, &base_successor, limits()).unwrap(),
            SemanticDiff::between(&reordered, &reordered_successor, limits()).unwrap(),
        );
    }
}
