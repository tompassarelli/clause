use std::collections::{BTreeMap, BTreeSet};

use super::SemanticDiff;
use crate::{
    derive::{Limits, SupportLimits},
    kernel::{
        Cardinality, Clause, Delta, EntityId, InlineSentencePart, Law, Mode, Model, ModelId, Name,
        Relation, RelationId, Role, RoleId, SentenceShape, Term, Type, TypeId, VariableId,
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
fn incomplete_opposite_frontier_suppresses_unknown_change() {
    let left = clause("impact/left");
    let right = clause("impact/right");
    let middle = clause("impact/middle");
    let base = revision(vec![left.clone()], false);
    let successor = successor(&base, vec![right, middle], vec![left]);
    let bounded = SupportLimits::new(Limits::new(100, 10, 10_000), 100, 1);
    let diff = SemanticDiff::between(&base, &successor, bounded).unwrap();
    assert!(diff.changed_supports().is_empty());
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
