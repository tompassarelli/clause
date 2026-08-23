//! Certified finite intervention synthesis over typed, sealed revisions.
//!
//! `one minimal` and `all minimal` are deliberately separate contracts. A
//! one-result request proves inclusion minimality by exact counterfactual
//! closure checks; it makes no claim about cardinality optimality or the
//! complete frontier. An all-result request is complete only after the finite
//! candidate space has been exhausted.
//!
//! The stable public surface stays at clause::intervention. Private modules
//! separate one-result certification, exhaustive frontier search, candidate
//! construction, and shared bounded closure mechanics.
mod all;
mod basis;
mod closure;
mod one;
mod search;

use crate::{
    derive::{Limits, Proof, SupportLimits},
    kernel::{Clause, Delta, RelationId, Result, Revision},
};

/// Explicit resource bounds for an intervention request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterventionLimits {
    closure: Limits,
    max_candidates: usize,
    max_solutions: usize,
    support: SupportLimits,
}

impl InterventionLimits {
    pub fn new(closure: Limits, max_candidates: usize, max_solutions: usize) -> Self {
        Self {
            closure,
            max_candidates,
            max_solutions,
            support: SupportLimits::new(closure, max_candidates, max_solutions),
        }
    }

    pub fn with_support_limits(mut self, support: SupportLimits) -> Self {
        self.support = support;
        self
    }

    pub fn closure(&self) -> Limits {
        self.closure
    }

    pub fn max_candidates(&self) -> usize {
        self.max_candidates
    }

    pub fn max_solutions(&self) -> usize {
        self.max_solutions
    }

    pub fn support(&self) -> SupportLimits {
        self.support
    }
}

/// A verified Delta and the only successor Revision it admits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intervention {
    delta: Delta,
    revision: Revision,
    proof: Option<Proof>,
}

impl Intervention {
    pub fn delta(&self) -> &Delta {
        &self.delta
    }

    pub fn revision(&self) -> &Revision {
        &self.revision
    }

    pub fn proof(&self) -> Option<&Proof> {
        self.proof.as_ref()
    }

    fn withdrawal(source: &Revision, withdrawals: Vec<Clause>, revision: Revision) -> Result<Self> {
        Ok(Self {
            delta: Delta::new(source.identity().clone(), Vec::new(), withdrawals)?,
            revision,
            proof: None,
        })
    }

    fn admission(
        source: &Revision,
        admissions: Vec<Clause>,
        revision: Revision,
        proof: Proof,
    ) -> Result<Self> {
        Ok(Self {
            delta: Delta::new(source.identity().clone(), admissions, Vec::new())?,
            revision,
            proof: Some(proof),
        })
    }
}

/// A result that may be sound but cannot make a stronger certification claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Incomplete {
    CandidateBudgetExhausted,
    SolutionBudgetExhausted,
    ClosureBudgetExhausted,
    SupportExpansionBudgetExhausted,
    SupportBudgetExhausted,
}

/// Exact outcome for `prevent one minimal`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreventOne {
    Satisfied(Box<Intervention>),
    AlreadyAbsent,
    Impossible,
    Incomplete(Incomplete),
}

/// Exact outcome for `achieve one minimal`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AchieveOne {
    Satisfied(Box<Intervention>),
    AlreadyEntailed,
    Impossible,
    Incomplete(Incomplete),
}

/// Exhaustive finite prevention output. Results retained on an incomplete
/// search are individually verified but are not a complete frontier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreventAll {
    Complete(Vec<Intervention>),
    AlreadyAbsent,
    Impossible,
    Incomplete {
        interventions: Vec<Intervention>,
        reason: Incomplete,
    },
}

impl PreventAll {
    pub fn interventions(&self) -> &[Intervention] {
        match self {
            Self::Complete(items)
            | Self::Incomplete {
                interventions: items,
                ..
            } => items,
            Self::AlreadyAbsent | Self::Impossible => &[],
        }
    }
}

/// Exhaustive finite achievement output. Results retained on an incomplete
/// search are individually verified but are not a complete frontier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AchieveAll {
    Complete(Vec<Intervention>),
    AlreadyEntailed,
    Impossible,
    Incomplete {
        interventions: Vec<Intervention>,
        reason: Incomplete,
    },
}

impl AchieveAll {
    pub fn interventions(&self) -> &[Intervention] {
        match self {
            Self::Complete(items)
            | Self::Incomplete {
                interventions: items,
                ..
            } => items,
            Self::AlreadyEntailed | Self::Impossible => &[],
        }
    }
}

/// Return one canonical inclusion-minimal asserted-clause withdrawal.
///
/// The deletion/restoration algorithm is valid only because the admitted law
/// fragment is positive and monotone. It proves each retained withdrawal is
/// necessary, but intentionally does not prove it has minimum cardinality.
pub fn prevent_one_minimal(
    source: &Revision,
    target: Clause,
    using: Vec<RelationId>,
    limits: InterventionLimits,
) -> Result<PreventOne> {
    one::prevent_one_minimal(source, target, using, limits)
}

/// Return one canonical inclusion-minimal asserted-clause admission.
///
/// This is the dual of [`prevent_one_minimal`]. It proves subset necessity,
/// not cardinality optimality or complete-frontier enumeration.
pub fn achieve_one_minimal(
    source: &Revision,
    target: Clause,
    using: Vec<RelationId>,
    limits: InterventionLimits,
) -> Result<AchieveOne> {
    one::achieve_one_minimal(source, target, using, limits)
}

/// Enumerate every inclusion-minimal withdrawal over the complete support
/// frontier.
pub fn prevent_all_minimal(
    source: &Revision,
    target: Clause,
    using: Vec<RelationId>,
    limits: InterventionLimits,
) -> Result<PreventAll> {
    all::prevent_all_minimal(source, target, using, limits)
}

/// Enumerate every inclusion-minimal addition over the finite typed basis.
pub fn achieve_all_minimal(
    source: &Revision,
    target: Clause,
    using: Vec<RelationId>,
    limits: InterventionLimits,
) -> Result<AchieveAll> {
    all::achieve_all_minimal(source, target, using, limits)
}

#[cfg(test)]
use crate::kernel::Term;
#[cfg(test)]
use basis::achievement_basis;
#[cfg(test)]
use search::{Enumeration, enumerate};
#[cfg(test)]
use std::collections::BTreeSet;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        kernel::{
            Cardinality, EntityId, InlineSentencePart, Law, LawId, Mode, Model, ModelId, Name,
            Relation, Role, RoleId, SentenceShape, Type, TypeId, VariableId,
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
    fn entity(model: &ModelId, local: &str, typ: &TypeId) -> EntityId {
        EntityId::new(model.clone(), name(local), typ.clone()).unwrap()
    }
    fn role(value: &str, typ: &TypeId) -> Role {
        Role::new(role_id(value), typ.clone())
    }
    fn relation(id: &RelationId, left: (&str, &TypeId), right: (&str, &TypeId)) -> Relation {
        let left_role = role(left.0, left.1);
        let right_role = role(right.0, right.1);
        Relation::new(
            id.clone(),
            SentenceShape::new(vec![
                InlineSentencePart::Role(left_role.clone()),
                InlineSentencePart::Literal("relates".into()),
                InlineSentencePart::Role(right_role.clone()),
            ])
            .unwrap(),
            vec![
                Mode::finite(
                    vec![left_role.id().clone()],
                    vec![right_role.id().clone()],
                    Cardinality::Many,
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }
    fn clause(relation: &RelationId, left: (&str, &EntityId), right: (&str, &EntityId)) -> Clause {
        Clause::new(
            relation.clone(),
            BTreeMap::from([
                (role_id(left.0), Term::entity(left.1.clone())),
                (role_id(right.0), Term::entity(right.1.clone())),
            ]),
        )
        .unwrap()
    }
    fn variable(value: &str, typ: &TypeId) -> Term {
        Term::variable(VariableId::new(name(value)).unwrap(), typ.clone())
    }
    fn law(id: &str, premise: Clause, conclusion: Clause) -> Law {
        Law::new(LawId::new(name(id)).unwrap(), vec![premise], conclusion).unwrap()
    }
    fn rev(
        entities: Vec<EntityId>,
        relations: Vec<Relation>,
        assertions: Vec<Clause>,
        laws: Vec<Law>,
    ) -> Revision {
        let model_id = entities.first().unwrap().model().clone();
        let types = entities
            .iter()
            .map(|entity| (entity.typ().clone(), Type::new(entity.typ().clone())))
            .collect();
        wire::admit(
            Model::new(
                model_id,
                types,
                entities.into_iter().collect(),
                relations
                    .into_iter()
                    .map(|relation| (relation.id().clone(), relation))
                    .collect(),
                assertions,
                laws,
            )
            .unwrap(),
        )
    }
    fn limits() -> InterventionLimits {
        InterventionLimits::new(Limits::new(100, 10, 20_000), 200, 100)
    }

    #[test]
    fn typed_achievement_basis_uses_exact_role_types_and_excludes_existing_assertions() {
        let model = ModelId::new(name("plans")).unwrap();
        let place = type_id("Place");
        let permit = type_id("Permit");
        let alpha = entity(&model, "Alpha", &place);
        let beta = entity(&model, "Beta", &place);
        let permit_a = entity(&model, "A", &permit);
        let assigned = relation_id("plans/assigned");
        let source = rev(
            vec![alpha.clone(), beta.clone(), permit_a.clone()],
            vec![relation(&assigned, ("place", &place), ("permit", &permit))],
            vec![clause(&assigned, ("place", &alpha), ("permit", &permit_a))],
            vec![],
        );
        let basis = achievement_basis(&source, vec![assigned.clone()], usize::MAX)
            .unwrap()
            .clauses;
        assert_eq!(
            basis,
            vec![clause(&assigned, ("place", &beta), ("permit", &permit_a),)]
        );
        let exhausted = achievement_basis(&source, vec![assigned.clone()], 0).unwrap();
        assert!(!exhausted.complete);
        assert!(exhausted.clauses.is_empty());
        let exact = achievement_basis(&source, vec![assigned], 1).unwrap();
        assert!(exact.complete);
        assert_eq!(exact.clauses.len(), 1);
    }

    #[test]
    fn one_minimal_proves_subset_necessity_but_not_minimum_cardinality() {
        let model = ModelId::new(name("m")).unwrap();
        let t = type_id("Thing");
        let a = entity(&model, "A", &t);
        let b = entity(&model, "B", &t);
        let input_a = relation_id("m/a");
        let input_b = relation_id("m/b");
        let goal = relation_id("m/goal");
        let va = variable("a", &t);
        let vb = variable("b", &t);
        let source = rev(
            vec![a.clone(), b.clone()],
            vec![
                relation(&input_a, ("x", &t), ("y", &t)),
                relation(&input_b, ("x", &t), ("y", &t)),
                relation(&goal, ("x", &t), ("y", &t)),
            ],
            vec![],
            vec![
                law(
                    "m/a-goal",
                    Clause::new(
                        input_a.clone(),
                        BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                    )
                    .unwrap(),
                    Clause::new(
                        goal.clone(),
                        BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                    )
                    .unwrap(),
                ),
                Law::new(
                    LawId::new(name("m/bc-goal")).unwrap(),
                    vec![
                        Clause::new(
                            input_b.clone(),
                            BTreeMap::from([
                                (role_id("x"), va.clone()),
                                (role_id("y"), vb.clone()),
                            ]),
                        )
                        .unwrap(),
                        Clause::new(
                            input_b.clone(),
                            BTreeMap::from([
                                (role_id("x"), vb.clone()),
                                (role_id("y"), va.clone()),
                            ]),
                        )
                        .unwrap(),
                    ],
                    Clause::new(
                        goal.clone(),
                        BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ],
        );
        let target = clause(&goal, ("x", &a), ("y", &b));
        let result = achieve_one_minimal(
            &source,
            target,
            vec![input_a.clone(), input_b.clone()],
            limits(),
        )
        .unwrap();
        let AchieveOne::Satisfied(intervention) = result else {
            panic!("expected certified one-minimal result");
        };
        assert_eq!(intervention.delta().admissions().len(), 2);
        assert!(
            intervention
                .delta()
                .admissions()
                .iter()
                .all(|item| item.relation() == &input_b)
        );
    }

    #[test]
    fn all_achievement_is_complete_and_impossible_is_explicit() {
        let model = ModelId::new(name("m")).unwrap();
        let t = type_id("Thing");
        let a = entity(&model, "A", &t);
        let b = entity(&model, "B", &t);
        let input = relation_id("m/input");
        let goal = relation_id("m/goal");
        let va = variable("a", &t);
        let vb = variable("b", &t);
        let source = rev(
            vec![a.clone(), b.clone()],
            vec![
                relation(&input, ("x", &t), ("y", &t)),
                relation(&goal, ("x", &t), ("y", &t)),
            ],
            vec![],
            vec![law(
                "m/copy",
                Clause::new(
                    input.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
                Clause::new(
                    goal.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
            )],
        );
        let target = clause(&goal, ("x", &a), ("y", &b));
        let all =
            achieve_all_minimal(&source, target.clone(), vec![input.clone()], limits()).unwrap();
        assert_eq!(all.interventions().len(), 1);
        assert!(matches!(all, AchieveAll::Complete(_)));
        let exact_limit = achieve_all_minimal(
            &source,
            target.clone(),
            vec![input.clone()],
            InterventionLimits::new(Limits::new(100, 10, 20_000), 20, 1),
        )
        .unwrap();
        assert!(matches!(exact_limit, AchieveAll::Complete(items) if items.len() == 1));
        let impossible_source = rev(
            vec![a.clone(), b.clone()],
            vec![
                relation(&input, ("x", &t), ("y", &t)),
                relation(&goal, ("x", &t), ("y", &t)),
            ],
            vec![],
            vec![],
        );
        assert!(matches!(
            achieve_all_minimal(
                &impossible_source,
                target.clone(),
                vec![input.clone()],
                InterventionLimits::new(Limits::new(100, 10, 20_000), 20, 0),
            )
            .unwrap(),
            AchieveAll::Impossible
        ));
        assert!(matches!(
            achieve_all_minimal(&source, target, vec![goal], limits())
                .unwrap_err()
                .to_string()
                .as_str(),
            "intervention relation is not extensional"
        ));
    }

    #[test]
    fn all_prevention_hits_redundant_supports_and_budget_never_certifies() {
        let model = ModelId::new(name("m")).unwrap();
        let t = type_id("Thing");
        let a = entity(&model, "A", &t);
        let b = entity(&model, "B", &t);
        let left = relation_id("m/left");
        let right = relation_id("m/right");
        let goal = relation_id("m/goal");
        let va = variable("a", &t);
        let vb = variable("b", &t);
        let c1 = clause(&left, ("x", &a), ("y", &b));
        let c2 = clause(&right, ("x", &a), ("y", &b));
        let source = rev(
            vec![a.clone(), b.clone()],
            vec![
                relation(&left, ("x", &t), ("y", &t)),
                relation(&right, ("x", &t), ("y", &t)),
                relation(&goal, ("x", &t), ("y", &t)),
            ],
            vec![c1.clone(), c2.clone()],
            vec![
                law(
                    "m/left-goal",
                    Clause::new(
                        left.clone(),
                        BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                    )
                    .unwrap(),
                    Clause::new(
                        goal.clone(),
                        BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                    )
                    .unwrap(),
                ),
                law(
                    "m/right-goal",
                    Clause::new(
                        right.clone(),
                        BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                    )
                    .unwrap(),
                    Clause::new(
                        goal.clone(),
                        BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                    )
                    .unwrap(),
                ),
            ],
        );
        let target = clause(&goal, ("x", &a), ("y", &b));
        let all = prevent_all_minimal(
            &source,
            target.clone(),
            vec![left.clone(), right.clone()],
            limits(),
        )
        .unwrap();
        assert_eq!(all.interventions().len(), 1);
        assert_eq!(all.interventions()[0].delta().withdrawals(), &[c1, c2]);
        let bounded = prevent_all_minimal(
            &source,
            target,
            vec![left, right],
            InterventionLimits::new(Limits::new(100, 10, 20_000), 0, 10)
                .with_support_limits(SupportLimits::new(Limits::new(100, 10, 20_000), 100, 100)),
        )
        .unwrap();
        assert!(matches!(
            bounded,
            PreventAll::Incomplete {
                reason: Incomplete::CandidateBudgetExhausted,
                ..
            }
        ));
    }

    #[test]
    fn complete_frontiers_cover_redundant_withdrawals_successor_degradation_and_typed_additions() {
        let exhaustive = InterventionLimits::new(Limits::new(100, 10, 20_000), 10_000, 100);
        let model = ModelId::new(name("network")).unwrap();
        let node = type_id("Node");
        let alpha = entity(&model, "Alpha", &node);
        let beta = entity(&model, "Beta", &node);
        let gamma = entity(&model, "Gamma", &node);
        let omega = entity(&model, "Omega", &node);
        let link = relation_id("network/link");
        let reaches = relation_id("network/reaches");
        let subject = variable("subject", &node);
        let middle = variable("middle", &node);
        let destination = variable("destination", &node);
        let alpha_beta = clause(&link, ("x", &alpha), ("y", &beta));
        let beta_omega = clause(&link, ("x", &beta), ("y", &omega));
        let alpha_gamma = clause(&link, ("x", &alpha), ("y", &gamma));
        let gamma_omega = clause(&link, ("x", &gamma), ("y", &omega));
        let source = rev(
            vec![alpha.clone(), beta.clone(), gamma.clone(), omega.clone()],
            vec![
                relation(&link, ("x", &node), ("y", &node)),
                relation(&reaches, ("x", &node), ("y", &node)),
            ],
            vec![
                alpha_beta.clone(),
                beta_omega.clone(),
                alpha_gamma.clone(),
                gamma_omega.clone(),
            ],
            vec![
                Law::new(
                    LawId::new(name("network/path")).unwrap(),
                    vec![
                        Clause::new(
                            link.clone(),
                            BTreeMap::from([
                                (role_id("x"), subject.clone()),
                                (role_id("y"), middle.clone()),
                            ]),
                        )
                        .unwrap(),
                        Clause::new(
                            link.clone(),
                            BTreeMap::from([
                                (role_id("x"), middle.clone()),
                                (role_id("y"), destination.clone()),
                            ]),
                        )
                        .unwrap(),
                    ],
                    Clause::new(
                        reaches.clone(),
                        BTreeMap::from([(role_id("x"), subject), (role_id("y"), destination)]),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ],
        );
        let target = clause(&reaches, ("x", &alpha), ("y", &omega));
        let expected_base = BTreeSet::from([
            vec![alpha_beta.clone(), alpha_gamma.clone()],
            vec![alpha_beta.clone(), gamma_omega.clone()],
            vec![alpha_gamma.clone(), beta_omega.clone()],
            vec![beta_omega.clone(), gamma_omega.clone()],
        ]);
        let PreventAll::Complete(base) =
            prevent_all_minimal(&source, target.clone(), vec![link.clone()], exhaustive).unwrap()
        else {
            panic!("finite redundant prevention frontier must be complete");
        };
        assert_eq!(
            base.iter()
                .map(|item| item.delta().withdrawals().to_vec())
                .collect::<BTreeSet<_>>(),
            expected_base,
        );

        let successor = Delta::new(
            source.identity().clone(),
            Vec::new(),
            vec![alpha_beta.clone()],
        )
        .unwrap()
        .apply(&source)
        .unwrap();
        let PreventAll::Complete(successor_prevention) =
            prevent_all_minimal(&successor, target, vec![link.clone()], exhaustive).unwrap()
        else {
            panic!("degraded finite prevention frontier must be complete");
        };
        assert_eq!(
            successor_prevention
                .iter()
                .map(|item| item.delta().withdrawals().to_vec())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([vec![alpha_gamma.clone()], vec![gamma_omega.clone()]]),
        );

        let choices = ModelId::new(name("choices")).unwrap();
        let start = entity(&choices, "Start", &node);
        let finish = entity(&choices, "Finish", &node);
        let first = relation_id("choices/first");
        let second = relation_id("choices/second");
        let achieved = relation_id("choices/achieved");
        let from = variable("from", &node);
        let to = variable("to", &node);
        let choices_source = rev(
            vec![start.clone(), finish.clone()],
            vec![
                relation(&first, ("x", &node), ("y", &node)),
                relation(&second, ("x", &node), ("y", &node)),
                relation(&achieved, ("x", &node), ("y", &node)),
            ],
            Vec::new(),
            vec![
                law(
                    "choices/first-achieves",
                    Clause::new(
                        first.clone(),
                        BTreeMap::from([(role_id("x"), from.clone()), (role_id("y"), to.clone())]),
                    )
                    .unwrap(),
                    Clause::new(
                        achieved.clone(),
                        BTreeMap::from([(role_id("x"), from.clone()), (role_id("y"), to.clone())]),
                    )
                    .unwrap(),
                ),
                law(
                    "choices/second-achieves",
                    Clause::new(
                        second.clone(),
                        BTreeMap::from([(role_id("x"), from), (role_id("y"), to)]),
                    )
                    .unwrap(),
                    Clause::new(
                        achieved.clone(),
                        BTreeMap::from([
                            (role_id("x"), variable("from", &node)),
                            (role_id("y"), variable("to", &node)),
                        ]),
                    )
                    .unwrap(),
                ),
            ],
        );
        let AchieveAll::Complete(additions) = achieve_all_minimal(
            &choices_source,
            clause(&achieved, ("x", &start), ("y", &finish)),
            vec![first.clone(), second.clone()],
            exhaustive,
        )
        .unwrap() else {
            panic!("finite typed achievement frontier must be complete");
        };
        assert_eq!(
            additions
                .iter()
                .map(|item| item.delta().admissions().to_vec())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                vec![clause(&first, ("x", &start), ("y", &finish))],
                vec![clause(&second, ("x", &start), ("y", &finish))],
            ]),
        );
        let limited = achieve_all_minimal(
            &choices_source,
            clause(&achieved, ("x", &start), ("y", &finish)),
            vec![first, second],
            InterventionLimits::new(Limits::new(100, 10, 20_000), 10_000, 1),
        )
        .unwrap();
        assert!(matches!(
            limited,
            AchieveAll::Incomplete {
                interventions,
                reason: Incomplete::SolutionBudgetExhausted,
            } if interventions.len() == 1
        ));
    }

    #[test]
    fn prevention_one_restoration_returns_an_inclusion_minimal_withdrawal() {
        let model = ModelId::new(name("m")).unwrap();
        let t = type_id("Thing");
        let a = entity(&model, "A", &t);
        let b = entity(&model, "B", &t);
        let input = relation_id("m/input");
        let goal = relation_id("m/goal");
        let va = variable("a", &t);
        let vb = variable("b", &t);
        let asserted = clause(&input, ("x", &a), ("y", &b));
        let source = rev(
            vec![a.clone(), b.clone()],
            vec![
                relation(&input, ("x", &t), ("y", &t)),
                relation(&goal, ("x", &t), ("y", &t)),
            ],
            vec![asserted.clone()],
            vec![law(
                "m/copy",
                Clause::new(
                    input.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
                Clause::new(
                    goal.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
            )],
        );
        let result = prevent_one_minimal(
            &source,
            clause(&goal, ("x", &a), ("y", &b)),
            vec![input],
            limits(),
        )
        .unwrap();
        let PreventOne::Satisfied(intervention) = result else {
            panic!("expected certified prevention");
        };
        assert_eq!(intervention.delta().withdrawals(), &[asserted]);
    }

    #[test]
    fn absent_candidate_basis_is_impossible_not_a_empty_delta() {
        let model = ModelId::new(name("m")).unwrap();
        let t = type_id("Thing");
        let a = entity(&model, "A", &t);
        let b = entity(&model, "B", &t);
        let input = relation_id("m/input");
        let goal = relation_id("m/goal");
        let source = rev(
            vec![a.clone(), b.clone()],
            vec![
                relation(&input, ("x", &t), ("y", &t)),
                relation(&goal, ("x", &t), ("y", &t)),
            ],
            vec![],
            vec![],
        );
        assert!(matches!(
            achieve_one_minimal(
                &source,
                clause(&goal, ("x", &a), ("y", &b)),
                vec![input],
                limits()
            )
            .unwrap(),
            AchieveOne::Impossible
        ));
    }

    #[test]
    fn closure_budget_is_uncertified_not_absence_or_impossibility() {
        let model = ModelId::new(name("m")).unwrap();
        let t = type_id("Thing");
        let a = entity(&model, "A", &t);
        let b = entity(&model, "B", &t);
        let input = relation_id("m/input");
        let goal = relation_id("m/goal");
        let va = variable("a", &t);
        let vb = variable("b", &t);
        let source = rev(
            vec![a.clone(), b.clone()],
            vec![
                relation(&input, ("x", &t), ("y", &t)),
                relation(&goal, ("x", &t), ("y", &t)),
            ],
            vec![],
            vec![law(
                "m/copy",
                Clause::new(
                    input.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
                Clause::new(
                    goal.clone(),
                    BTreeMap::from([(role_id("x"), va), (role_id("y"), vb)]),
                )
                .unwrap(),
            )],
        );
        let tight = InterventionLimits::new(Limits::new(0, 10, 100), 10, 10);
        assert!(matches!(
            achieve_all_minimal(
                &source,
                clause(&goal, ("x", &a), ("y", &b)),
                vec![input],
                tight,
            )
            .unwrap(),
            AchieveAll::Incomplete {
                reason: Incomplete::ClosureBudgetExhausted,
                ..
            }
        ));
    }

    #[test]
    fn enumeration_break_stops_the_search_immediately() {
        let model = ModelId::new(name("m")).unwrap();
        let typ = type_id("Thing");
        let alpha = entity(&model, "Alpha", &typ);
        let beta = entity(&model, "Beta", &typ);
        let relation = relation_id("m/input");
        let basis = vec![
            clause(&relation, ("x", &alpha), ("y", &alpha)),
            clause(&relation, ("x", &alpha), ("y", &beta)),
            clause(&relation, ("x", &beta), ("y", &alpha)),
            clause(&relation, ("x", &beta), ("y", &beta)),
        ];
        let mut visits = 0;
        let control = enumerate(&basis, 1, 0, &mut Vec::new(), &mut |_| {
            visits += 1;
            Ok(Enumeration::Break)
        })
        .unwrap();
        assert_eq!(control, Enumeration::Break);
        assert_eq!(visits, 1);
    }
}
