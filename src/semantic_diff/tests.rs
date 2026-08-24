use std::collections::BTreeMap;

use super::SemanticDiff;
use crate::{
    derive::{Limits, SupportLimits},
    kernel::{
        AssertionOccurrence, Cardinality, Delta, DerivationRule, Judgment, JudgmentKind,
        JudgmentStatus, JudgmentTarget, LookupMode, Model, Pattern, PatternId, Referent,
        ReferentId, RelationShape, RelationalContent, Role, RoleId, SemanticAtom, Term,
    },
    wire,
};

#[derive(Clone, Copy)]
enum FixtureRelation {
    Left,
    Middle,
    Right,
    Result,
}

const SUPPORT_RELATIONS: [FixtureRelation; 3] = [
    FixtureRelation::Left,
    FixtureRelation::Middle,
    FixtureRelation::Right,
];
const RELATIONS: [FixtureRelation; 4] = [
    FixtureRelation::Left,
    FixtureRelation::Middle,
    FixtureRelation::Right,
    FixtureRelation::Result,
];

impl FixtureRelation {
    fn offset(self) -> u8 {
        match self {
            Self::Left => 0,
            Self::Middle => 1,
            Self::Right => 2,
            Self::Result => 3,
        }
    }

    fn id(self) -> ReferentId {
        referent(10 + self.offset())
    }

    fn rule_id(self) -> ReferentId {
        referent(20 + self.offset())
    }

    fn occurrence_id(self) -> ReferentId {
        referent(30 + self.offset())
    }

    fn judgment_id(self) -> ReferentId {
        referent(40 + self.offset())
    }
}

fn referent(byte: u8) -> ReferentId {
    ReferentId::from_digest([byte; 32])
}

fn model_id() -> ReferentId {
    referent(1)
}

fn north() -> ReferentId {
    referent(2)
}

fn beagle() -> ReferentId {
    referent(3)
}

fn subject_role() -> RoleId {
    RoleId::from_digest([1; 32])
}

fn object_role() -> RoleId {
    RoleId::from_digest([2; 32])
}

fn subject_pattern() -> PatternId {
    PatternId::from_digest([1; 32])
}

fn object_pattern() -> PatternId {
    PatternId::from_digest([2; 32])
}

fn limits() -> SupportLimits {
    SupportLimits::new(Limits::new(100, 10, 10_000), 100, 100)
}

fn source(relation: FixtureRelation) -> RelationShape {
    let subject_id = subject_role();
    let object_id = object_role();
    let subject = Role::new(subject_id.clone(), Vec::new()).unwrap();
    let object = Role::new(object_id.clone(), Vec::new()).unwrap();
    RelationShape::new(
        relation.id(),
        BTreeMap::from([(subject_id, subject), (object_id, object)]),
        vec![
            LookupMode::finite(vec![subject_role()], vec![object_role()], Cardinality::Many)
                .unwrap(),
        ],
    )
    .unwrap()
}

fn clause(relation: FixtureRelation) -> RelationalContent {
    RelationalContent::new(
        relation.id(),
        BTreeMap::from([
            (subject_role(), Term::referent(north())),
            (object_role(), Term::referent(beagle())),
        ]),
    )
    .unwrap()
}

fn relational_pattern(relation: FixtureRelation) -> RelationalContent {
    RelationalContent::new(
        relation.id(),
        BTreeMap::from([
            (subject_role(), Term::pattern(subject_pattern())),
            (object_role(), Term::pattern(object_pattern())),
        ]),
    )
    .unwrap()
}

fn occurrence(relation: FixtureRelation) -> AssertionOccurrence {
    AssertionOccurrence::new(
        relation.occurrence_id(),
        clause(relation).id().clone(),
        model_id(),
        model_id(),
    )
}

fn judgment(relation: FixtureRelation) -> Judgment {
    Judgment::new(
        relation.judgment_id(),
        model_id(),
        model_id(),
        JudgmentTarget::Occurrence(relation.occurrence_id()),
        JudgmentKind::Admitted {
            policy: model_id(),
            basis: Vec::new(),
        },
        JudgmentStatus::Affirmed,
    )
}

fn model(assertions: Vec<FixtureRelation>, reverse_laws: bool) -> Model {
    let mut laws = SUPPORT_RELATIONS
        .into_iter()
        .map(|relation| {
            let premise = relational_pattern(relation);
            let conclusion = relational_pattern(FixtureRelation::Result);
            DerivationRule::new(
                relation.rule_id(),
                model_id(),
                model_id(),
                Pattern::new(vec![premise.id().clone()]).unwrap(),
                Pattern::new(vec![conclusion.id().clone()]).unwrap(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    if reverse_laws {
        laws.reverse();
    }
    let referents = [model_id(), north(), beagle()]
        .into_iter()
        .chain(RELATIONS.into_iter().map(FixtureRelation::id))
        .chain(SUPPORT_RELATIONS.into_iter().map(FixtureRelation::rule_id))
        .chain(
            assertions
                .iter()
                .copied()
                .flat_map(|relation| [relation.occurrence_id(), relation.judgment_id()]),
        )
        .map(|id| (id.clone(), Referent::new(id)))
        .collect();
    let relational_contents = RELATIONS
        .into_iter()
        .map(relational_pattern)
        .chain(assertions.iter().copied().map(clause))
        .map(|content| (content.id().clone(), content))
        .collect();
    let relation_shapes = RELATIONS
        .into_iter()
        .map(|relation| (relation.id(), source(relation)))
        .collect();
    let occurrences = assertions.iter().copied().map(occurrence).collect();
    let judgments = assertions.iter().copied().map(judgment).collect();
    Model::with_distinctions(
        model_id(),
        referents,
        relational_contents,
        relation_shapes,
        occurrences,
        Vec::new(),
        laws,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        judgments,
    )
    .unwrap()
}

fn revision(assertions: Vec<FixtureRelation>, reverse_laws: bool) -> crate::kernel::Revision {
    wire::admit(model(assertions, reverse_laws))
}

fn successor(
    base: &crate::kernel::Revision,
    admissions: Vec<FixtureRelation>,
    withdrawals: Vec<FixtureRelation>,
) -> crate::kernel::Revision {
    let mut admitted_atoms = Vec::new();
    for relation in admissions {
        let content = clause(relation);
        if !base
            .model()
            .relational_contents()
            .contains_key(content.id())
        {
            admitted_atoms.push(SemanticAtom::RelationalContent(content));
        }
        for id in [relation.occurrence_id(), relation.judgment_id()] {
            if !base.model().referents().contains_key(&id) {
                admitted_atoms.push(SemanticAtom::Referent(Referent::new(id)));
            }
        }
        admitted_atoms.push(SemanticAtom::AssertionOccurrence(occurrence(relation)));
        admitted_atoms.push(SemanticAtom::Judgment(judgment(relation)));
    }
    let withdrawn_atoms = withdrawals
        .into_iter()
        .flat_map(|relation| {
            [
                SemanticAtom::AssertionOccurrence(occurrence(relation)),
                SemanticAtom::Judgment(judgment(relation)),
            ]
        })
        .collect();
    Delta::new(base.identity().clone(), admitted_atoms, withdrawn_atoms)
        .unwrap()
        .apply(base)
        .unwrap()
}

fn assertion_equivalent_successor(base: &crate::kernel::Revision) -> crate::kernel::Revision {
    let identity = referent(99);
    Delta::new(
        base.identity().clone(),
        vec![SemanticAtom::Referent(Referent::new(identity))],
        Vec::new(),
    )
    .unwrap()
    .apply(base)
    .unwrap()
}

fn change(diff: &SemanticDiff) -> &super::SupportChange {
    diff.changed_supports()
        .iter()
        .find(|change| change.consequence() == &clause(FixtureRelation::Result))
        .unwrap()
}

#[test]
fn keeps_asserted_and_entailed_layers_separate() {
    let left = clause(FixtureRelation::Left);
    let base = revision(vec![], false);
    let successor = successor(&base, vec![FixtureRelation::Left], vec![]);
    let diff = SemanticDiff::between(&base, &successor, limits()).unwrap();
    assert_eq!(diff.authored().added(), &[left]);
    assert_eq!(diff.entailed_added(), &[clause(FixtureRelation::Result)]);
}

#[test]
fn retained_consequence_exposes_lost_and_retained_supports() {
    let left = clause(FixtureRelation::Left);
    let right = clause(FixtureRelation::Right);
    let base = revision(vec![FixtureRelation::Left, FixtureRelation::Right], false);
    let successor = successor(&base, vec![], vec![FixtureRelation::Left]);
    let diff = SemanticDiff::between(&base, &successor, limits()).unwrap();
    let change = change(&diff);
    assert_eq!(change.removed()[0].assertions(), &[left]);
    assert_eq!(change.retained()[0].assertions(), &[right]);
}

#[test]
fn disappearing_consequence_retains_its_support_loss() {
    let left = clause(FixtureRelation::Left);
    let base = revision(vec![FixtureRelation::Left], false);
    let successor = successor(&base, vec![], vec![FixtureRelation::Left]);
    let diff = SemanticDiff::between(&base, &successor, limits()).unwrap();
    assert_eq!(diff.entailed_removed(), &[clause(FixtureRelation::Result)]);
    assert_eq!(change(&diff).removed()[0].assertions(), &[left]);
}

#[test]
fn appearing_consequence_retains_its_support_gain() {
    let left = clause(FixtureRelation::Left);
    let base = revision(vec![], false);
    let successor = successor(&base, vec![FixtureRelation::Left], vec![]);
    let diff = SemanticDiff::between(&base, &successor, limits()).unwrap();
    assert_eq!(change(&diff).added()[0].assertions(), &[left]);
}

#[test]
fn incomplete_opposite_frontier_suppresses_unknown_change() {
    let base = revision(vec![FixtureRelation::Left], false);
    let successor = successor(
        &base,
        vec![FixtureRelation::Right, FixtureRelation::Middle],
        vec![FixtureRelation::Left],
    );
    let bounded = SupportLimits::new(Limits::new(100, 10, 10_000), 100, 1);
    let diff = SemanticDiff::between(&base, &successor, bounded).unwrap();
    assert!(diff.changed_supports().is_empty());
}

#[test]
fn reports_only_nonduplicate_observed_support_changes() {
    let left = clause(FixtureRelation::Left);
    let base = revision(vec![FixtureRelation::Left, FixtureRelation::Right], false);
    let successor = successor(&base, vec![], vec![FixtureRelation::Left]);
    let exact = SemanticDiff::between(&base, &successor, limits()).unwrap();
    assert_eq!(exact.authored().removed(), std::slice::from_ref(&left));
    assert!(
        exact
            .changed_supports()
            .iter()
            .all(|change| change.consequence() != &left)
    );

    let bounded = SupportLimits::new(Limits::new(100, 10, 10_000), 100, 1);
    let assertion_equivalent = assertion_equivalent_successor(&base);
    let unknown = SemanticDiff::between(&base, &assertion_equivalent, bounded).unwrap();
    assert!(unknown.changed_supports().is_empty());
}

#[test]
fn support_projection_is_deterministic_under_declaration_reordering() {
    let base = revision(vec![FixtureRelation::Left, FixtureRelation::Right], false);
    let reordered = revision(vec![FixtureRelation::Right, FixtureRelation::Left], true);
    let base_successor = successor(&base, vec![], vec![FixtureRelation::Left]);
    let reordered_successor = successor(&reordered, vec![], vec![FixtureRelation::Left]);
    assert_eq!(base, reordered);
    assert_eq!(
        SemanticDiff::between(&base, &base_successor, limits()).unwrap(),
        SemanticDiff::between(&reordered, &reordered_successor, limits()).unwrap(),
    );
}
