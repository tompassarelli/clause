//! Deterministic finite closure for admitted positive laws.

use crate::kernel::{Clause, KernelError, RevisionId, Term, VariableId};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub max_assertions: usize,
    pub max_rounds: usize,
    pub max_join_attempts: usize,
}

impl Limits {
    pub fn new(max_assertions: usize, max_rounds: usize, max_join_attempts: usize) -> Self {
        Self {
            max_assertions,
            max_rounds,
            max_join_attempts,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    generation: usize,
    witness: Witness,
}

impl Proof {
    pub fn generation(&self) -> usize {
        self.generation
    }
    pub fn witness(&self) -> &Witness {
        &self.witness
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Witness {
    Asserted,
    Derived {
        law: crate::kernel::LawId,
        premises: Vec<Clause>,
        substitution: BTreeMap<VariableId, Term>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closure {
    assertions: Vec<Clause>,
    proofs: BTreeMap<Clause, Proof>,
}

impl Closure {
    pub fn assertions(&self) -> &[Clause] {
        &self.assertions
    }
    pub fn proof(&self, clause: &Clause) -> Option<&Proof> {
        self.proofs.get(clause)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupportLimits {
    pub closure: Limits,
    pub max_expansions: usize,
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SupportWitness {
    Asserted,
    Derived {
        law: crate::kernel::LawId,
        premises: Vec<SupportProof>,
        substitution: BTreeMap<VariableId, Term>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Support {
    assertion_key: Vec<Clause>,
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

    pub(crate) fn assertion_key(&self) -> &[Clause] {
        &self.assertion_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportFrontier {
    revision: RevisionId,
    target: Clause,
    limits: SupportLimits,
    status: SupportStatus,
    expansions: usize,
    supports: Vec<Support>,
}

impl SupportFrontier {
    pub fn revision(&self) -> &RevisionId {
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

mod closure;
mod matching;
mod support;

pub use closure::saturate;
pub use support::support_frontier;

fn limit_error(kind: &str, name: &str, value: usize) -> KernelError {
    KernelError::new(format!("closure {kind} limit exceeded ({name}={value})"))
}

#[cfg(test)]
mod tests {
    use super::{Limits, SupportLimits, SupportStatus, Witness, saturate, support_frontier};
    use crate::kernel::{
        Cardinality, Clause, InlineSentencePart, Law, LawId, Mode, Model, ModelId, Name, Relation,
        RelationId, Revision, RevisionId, Role, RoleId, SentenceShape, Term, Type, TypeId,
        VariableId,
    };
    use std::collections::{BTreeMap, BTreeSet};

    fn name(value: &str) -> Name {
        Name::new(value.to_owned()).unwrap()
    }
    fn id(value: &str) -> TypeId {
        TypeId::new(name(value)).unwrap()
    }
    fn relation_id(value: &str) -> RelationId {
        RelationId::new(name(value)).unwrap()
    }
    fn role(value: &str, typ: &TypeId) -> Role {
        Role::new(RoleId::new(name(value)).unwrap(), typ.clone())
    }
    fn variable(value: &str, typ: &TypeId) -> Term {
        Term::variable(VariableId::new(name(value)).unwrap(), typ.clone())
    }
    fn text(value: &str, typ: &TypeId) -> Term {
        Term::value(typ.clone(), value.to_owned()).unwrap()
    }

    fn relation(identity: &RelationId, typ: &TypeId) -> Relation {
        let from = role("from", typ);
        let to = role("to", typ);
        Relation::new(
            identity.clone(),
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

    fn clause(identity: &RelationId, from: Term, to: Term) -> Clause {
        Clause::new(
            identity.clone(),
            BTreeMap::from([
                (RoleId::new(name("from")).unwrap(), from),
                (RoleId::new(name("to")).unwrap(), to),
            ]),
        )
        .unwrap()
    }

    fn revision(assertions: Vec<Clause>, laws: Vec<Law>) -> Revision {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let model = Model::new(
            ModelId::new(name("map")).unwrap(),
            BTreeMap::from([(text_type.clone(), Type::new(text_type.clone()))]),
            BTreeSet::new(),
            BTreeMap::from([
                (reaches.clone(), relation(&reaches, &text_type)),
                (links.clone(), relation(&links, &text_type)),
            ]),
            assertions,
            laws,
        )
        .unwrap();
        Revision::reloaded(RevisionId::from_digest([3; 32]), model)
    }

    #[test]
    fn typed_multi_round_closure_selects_canonical_witnesses() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let subject = variable("subject", &text_type);
        let destination = variable("destination", &text_type);
        let copy = Law::new(
            LawId::new(name("map/copy")).unwrap(),
            vec![clause(&links, subject.clone(), destination.clone())],
            clause(&reaches, subject, destination),
        )
        .unwrap();
        let closure = saturate(
            &revision(
                vec![clause(
                    &links,
                    text("North", &text_type),
                    text("Store", &text_type),
                )],
                vec![copy],
            ),
            Limits::new(10, 10, 100),
        )
        .unwrap();
        let derived = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        assert_eq!(closure.assertions().len(), 2);
        assert_eq!(closure.proof(&derived).unwrap().generation(), 1);
        assert!(matches!(
            closure.proof(&derived).unwrap().witness(),
            Witness::Derived { .. }
        ));
    }

    #[test]
    fn reversed_law_source_order_admits_the_same_model_and_closure() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let subject = variable("subject", &text_type);
        let destination = variable("destination", &text_type);
        let law_a = Law::new(
            LawId::new(name("map/a-copy")).unwrap(),
            vec![clause(&links, subject.clone(), destination.clone())],
            clause(&reaches, subject.clone(), destination.clone()),
        )
        .unwrap();
        let law_z = Law::new(
            LawId::new(name("map/z-copy")).unwrap(),
            vec![clause(&links, subject.clone(), destination.clone())],
            clause(&reaches, subject, destination),
        )
        .unwrap();
        let assertions = vec![clause(
            &links,
            text("North", &text_type),
            text("Store", &text_type),
        )];
        let forward = revision(assertions.clone(), vec![law_a.clone(), law_z.clone()]);
        let reversed = revision(assertions, vec![law_z, law_a]);
        assert_eq!(forward.model(), reversed.model());
        assert_eq!(
            saturate(&forward, Limits::new(10, 10, 100)).unwrap(),
            saturate(&reversed, Limits::new(10, 10, 100)).unwrap(),
        );
    }

    #[test]
    fn support_frontier_remains_minimal_and_typed() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let subject = variable("subject", &text_type);
        let destination = variable("destination", &text_type);
        let copy = Law::new(
            LawId::new(name("map/copy")).unwrap(),
            vec![clause(&links, subject.clone(), destination.clone())],
            clause(&reaches, subject, destination),
        )
        .unwrap();
        let target = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        let frontier = support_frontier(
            &revision(
                vec![clause(
                    &links,
                    text("North", &text_type),
                    text("Store", &text_type),
                )],
                vec![copy],
            ),
            &target,
            SupportLimits::new(Limits::new(10, 10, 100), 10, 10),
        )
        .unwrap();
        assert_eq!(frontier.status(), SupportStatus::Complete);
        assert_eq!(frontier.supports().len(), 1);
        assert_eq!(frontier.supports()[0].assertions().len(), 1);
    }

    #[test]
    fn support_members_follow_the_canonical_proof_path() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let first = clause(&links, text("Zulu", &text_type), text("First", &text_type));
        let second = clause(
            &links,
            text("Alpha", &text_type),
            text("Second", &text_type),
        );
        assert!(second < first);
        let target = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        let law = Law::new(
            LawId::new(name("map/path-order")).unwrap(),
            vec![first.clone(), second.clone()],
            target.clone(),
        )
        .unwrap();
        let frontier = support_frontier(
            &revision(vec![second.clone(), first.clone()], vec![law]),
            &target,
            SupportLimits::new(Limits::new(10, 10, 100), 10, 10),
        )
        .unwrap();
        let support = &frontier.supports()[0];
        assert_eq!(support.assertion_key(), &[second.clone(), first.clone()]);
        assert_eq!(support.assertions(), &[first, second]);
    }

    #[test]
    fn incomplete_frontier_does_not_expose_a_provisional_superset() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let alpha = clause(&links, text("Alpha", &text_type), text("One", &text_type));
        let beta = clause(&links, text("Beta", &text_type), text("Two", &text_type));
        let target = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        let wide = Law::new(
            LawId::new(name("map/a-wide")).unwrap(),
            vec![alpha.clone(), beta.clone()],
            target.clone(),
        )
        .unwrap();
        let narrow = Law::new(
            LawId::new(name("map/z-narrow")).unwrap(),
            vec![alpha.clone()],
            target.clone(),
        )
        .unwrap();
        let frontier = support_frontier(
            &revision(vec![alpha, beta], vec![wide, narrow]),
            &target,
            SupportLimits::new(Limits::new(10, 10, 100), 1, 10),
        )
        .unwrap();
        assert_eq!(frontier.status(), SupportStatus::ExpansionBudgetExhausted);
        assert!(frontier.supports().is_empty());
    }

    #[test]
    fn absent_target_has_a_complete_empty_frontier_without_support_budget() {
        let text_type = id("Text");
        let reaches = relation_id("map/reaches");
        let links = relation_id("map/links");
        let target = clause(
            &reaches,
            text("North", &text_type),
            text("Store", &text_type),
        );
        let frontier = support_frontier(
            &revision(
                vec![clause(
                    &links,
                    text("Alpha", &text_type),
                    text("Beta", &text_type),
                )],
                Vec::new(),
            ),
            &target,
            SupportLimits::new(Limits::new(10, 10, 100), 0, 0),
        )
        .unwrap();
        assert_eq!(frontier.status(), SupportStatus::Complete);
        assert!(frontier.supports().is_empty());
    }
}
