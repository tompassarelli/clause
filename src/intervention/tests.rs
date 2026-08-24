use super::*;
use crate::{
    kernel::{
        AssertionOccurrence, Cardinality, DerivationRule, Judgment, JudgmentKind, JudgmentStatus,
        JudgmentTarget, LookupMode, Model, Pattern, PatternId, Referent, ReferentId, RelationShape,
        RevisionLineage, Role, RoleId, RolePredicate, SemanticAtom,
    },
    wire,
};
use std::collections::BTreeMap;

fn referent_id(value: &str) -> ReferentId {
    ReferentId::from_digest(wire::sha256_digest(
        format!("intervention fixture referent: {value}").as_bytes(),
    ))
}
fn scoped_id(scope: &ReferentId, value: &str) -> ReferentId {
    referent_id(&format!("{}: {value}", scope.as_str()))
}
fn relation_id(value: &str) -> ReferentId {
    referent_id(value)
}
fn role_id(value: &str) -> RoleId {
    RoleId::from_digest(wire::sha256_digest(
        format!("intervention fixture role: {value}").as_bytes(),
    ))
}
fn pattern_id(value: &str) -> PatternId {
    PatternId::from_digest(wire::sha256_digest(
        format!("intervention fixture pattern: {value}").as_bytes(),
    ))
}
fn classification_relation() -> ReferentId {
    referent_id("fixture/classification")
}
fn classification_candidate_role() -> RoleId {
    role_id("fixture/classification/candidate")
}
fn classification_class_role() -> RoleId {
    role_id("fixture/classification/class")
}
fn role(value: &str, class: &ReferentId) -> Role {
    Role::new(
        role_id(value),
        vec![
            RolePredicate::new(
                classification_relation(),
                classification_candidate_role(),
                BTreeMap::from([(classification_class_role(), class.clone())]),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
fn relation(
    id: &ReferentId,
    left: (&str, &ReferentId),
    right: (&str, &ReferentId),
) -> RelationShape {
    let left_role = role(left.0, left.1);
    let right_role = role(right.0, right.1);
    RelationShape::new(
        id.clone(),
        BTreeMap::from([
            (left_role.id().clone(), left_role.clone()),
            (right_role.id().clone(), right_role.clone()),
        ]),
        vec![
            LookupMode::finite(
                vec![left_role.id().clone()],
                vec![right_role.id().clone()],
                Cardinality::Many,
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
fn classification_shape() -> RelationShape {
    let candidate = Role::new(classification_candidate_role(), Vec::new()).unwrap();
    let class = Role::new(classification_class_role(), Vec::new()).unwrap();
    RelationShape::new(
        classification_relation(),
        BTreeMap::from([
            (candidate.id().clone(), candidate),
            (class.id().clone(), class),
        ]),
        Vec::new(),
    )
    .unwrap()
}
fn classification(candidate: &ReferentId, class: &ReferentId) -> RelationalContent {
    RelationalContent::new(
        classification_relation(),
        BTreeMap::from([
            (
                classification_candidate_role(),
                Term::referent(candidate.clone()),
            ),
            (classification_class_role(), Term::referent(class.clone())),
        ]),
    )
    .unwrap()
}
fn clause(
    relation: &ReferentId,
    left: (&str, &ReferentId),
    right: (&str, &ReferentId),
) -> RelationalContent {
    RelationalContent::new(
        relation.clone(),
        BTreeMap::from([
            (role_id(left.0), Term::referent(left.1.clone())),
            (role_id(right.0), Term::referent(right.1.clone())),
        ]),
    )
    .unwrap()
}
fn variable(value: &str) -> Term {
    Term::pattern(pattern_id(value))
}

type RuleFixture = (DerivationRule, Vec<RelationalContent>);

fn rule(
    model: &ReferentId,
    id: &str,
    premises: Vec<RelationalContent>,
    conclusion: RelationalContent,
) -> RuleFixture {
    let premise_pattern = Pattern::new(
        premises
            .iter()
            .map(|content| content.id().clone())
            .collect(),
    )
    .unwrap();
    let conclusion_pattern = Pattern::new(vec![conclusion.id().clone()]).unwrap();
    let rule = DerivationRule::new(
        referent_id(id),
        model.clone(),
        model.clone(),
        premise_pattern,
        conclusion_pattern,
    )
    .unwrap();
    let mut contents = premises;
    contents.push(conclusion);
    (rule, contents)
}
fn law(
    model: &ReferentId,
    id: &str,
    premise: RelationalContent,
    conclusion: RelationalContent,
) -> RuleFixture {
    rule(model, id, vec![premise], conclusion)
}
fn rev(
    model: &ReferentId,
    classifications: Vec<(ReferentId, ReferentId)>,
    mut relations: Vec<RelationShape>,
    assertions: Vec<RelationalContent>,
    rules: Vec<RuleFixture>,
) -> Revision {
    relations.push(classification_shape());
    let mut referents = BTreeMap::new();
    for id in std::iter::once(model.clone())
        .chain(
            classifications
                .iter()
                .flat_map(|(candidate, class)| [candidate.clone(), class.clone()]),
        )
        .chain(relations.iter().map(|relation| relation.referent().clone()))
        .chain(rules.iter().map(|(rule, _)| rule.id().clone()))
    {
        referents.insert(id.clone(), Referent::new(id));
    }

    let mut admitted = classifications
        .iter()
        .map(|(candidate, class)| classification(candidate, class))
        .collect::<Vec<_>>();
    admitted.extend(assertions);
    let mut contents = admitted
        .iter()
        .cloned()
        .map(|content| (content.id().clone(), content))
        .collect::<BTreeMap<_, _>>();
    let mut derivation_rules = Vec::new();
    for (rule, forms) in rules {
        derivation_rules.push(rule);
        for content in forms {
            contents.insert(content.id().clone(), content);
        }
    }

    let mut occurrences = Vec::new();
    let mut judgments = Vec::new();
    for (index, content) in admitted.iter().enumerate() {
        let occurrence_id = scoped_id(model, &format!("assertion occurrence {index}"));
        let judgment_id = scoped_id(model, &format!("admission judgment {index}"));
        referents.insert(occurrence_id.clone(), Referent::new(occurrence_id.clone()));
        referents.insert(judgment_id.clone(), Referent::new(judgment_id.clone()));
        occurrences.push(AssertionOccurrence::new(
            occurrence_id.clone(),
            content.id().clone(),
            model.clone(),
            model.clone(),
        ));
        judgments.push(Judgment::new(
            judgment_id,
            model.clone(),
            model.clone(),
            JudgmentTarget::Occurrence(occurrence_id),
            JudgmentKind::Admitted {
                policy: model.clone(),
                basis: Vec::new(),
            },
            JudgmentStatus::Affirmed,
        ));
    }

    let shapes = relations
        .into_iter()
        .map(|relation| (relation.referent().clone(), relation))
        .collect();
    wire::admit(
        Model::with_distinctions(
            model.clone(),
            referents,
            contents,
            shapes,
            occurrences,
            Vec::new(),
            derivation_rules,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            judgments,
        )
        .unwrap(),
    )
}
fn limits() -> InterventionLimits {
    InterventionLimits::new(Limits::new(100, 10, 20_000), 200, 100)
}

#[test]
fn typed_achievement_basis_uses_exact_role_types_and_excludes_existing_assertions() {
    let model = referent_id("plans");
    let place = referent_id("classification/place");
    let permit = referent_id("classification/permit");
    let alpha = scoped_id(&model, "Alpha");
    let beta = scoped_id(&model, "Beta");
    let permit_a = scoped_id(&model, "A");
    let assigned = relation_id("plans/assigned");
    let source = rev(
        &model,
        vec![
            (alpha.clone(), place.clone()),
            (beta.clone(), place.clone()),
            (permit_a.clone(), permit.clone()),
        ],
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
    let model = referent_id("m");
    let class = referent_id("classification/thing");
    let a = scoped_id(&model, "A");
    let b = scoped_id(&model, "B");
    // Identity order makes the canonical deletion pass retain the larger support.
    let input_a = ReferentId::from_digest([1; 32]);
    let input_b = ReferentId::from_digest([2; 32]);
    let goal = relation_id("m/goal");
    let va = variable("a");
    let vb = variable("b");
    let source = rev(
        &model,
        vec![(a.clone(), class.clone()), (b.clone(), class.clone())],
        vec![
            relation(&input_a, ("x", &class), ("y", &class)),
            relation(&input_b, ("x", &class), ("y", &class)),
            relation(&goal, ("x", &class), ("y", &class)),
        ],
        vec![],
        vec![
            law(
                &model,
                "m/a-goal",
                RelationalContent::new(
                    input_a.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
                RelationalContent::new(
                    goal.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
            ),
            rule(
                &model,
                "m/bc-goal",
                vec![
                    RelationalContent::new(
                        input_b.clone(),
                        BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                    )
                    .unwrap(),
                    RelationalContent::new(
                        input_b.clone(),
                        BTreeMap::from([(role_id("x"), vb.clone()), (role_id("y"), va.clone())]),
                    )
                    .unwrap(),
                ],
                RelationalContent::new(
                    goal.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
            ),
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
    assert_eq!(intervention.admissions().len(), 2);
    assert!(
        intervention
            .admissions()
            .iter()
            .all(|item| item.relation() == &input_b)
    );
}

#[test]
fn all_achievement_is_complete_and_impossible_is_explicit() {
    let model = referent_id("m");
    let class = referent_id("classification/thing");
    let a = scoped_id(&model, "A");
    let b = scoped_id(&model, "B");
    let input = relation_id("m/input");
    let goal = relation_id("m/goal");
    let va = variable("a");
    let vb = variable("b");
    let source = rev(
        &model,
        vec![(a.clone(), class.clone()), (b.clone(), class.clone())],
        vec![
            relation(&input, ("x", &class), ("y", &class)),
            relation(&goal, ("x", &class), ("y", &class)),
        ],
        vec![],
        vec![law(
            &model,
            "m/copy",
            RelationalContent::new(
                input.clone(),
                BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
            )
            .unwrap(),
            RelationalContent::new(
                goal.clone(),
                BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
            )
            .unwrap(),
        )],
    );
    let target = clause(&goal, ("x", &a), ("y", &b));
    let all = achieve_all_minimal(&source, target.clone(), vec![input.clone()], limits()).unwrap();
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
        &model,
        vec![(a.clone(), class.clone()), (b.clone(), class.clone())],
        vec![
            relation(&input, ("x", &class), ("y", &class)),
            relation(&goal, ("x", &class), ("y", &class)),
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
    let model = referent_id("m");
    let class = referent_id("classification/thing");
    let a = scoped_id(&model, "A");
    let b = scoped_id(&model, "B");
    let left = relation_id("m/left");
    let right = relation_id("m/right");
    let goal = relation_id("m/goal");
    let va = variable("a");
    let vb = variable("b");
    let c1 = clause(&left, ("x", &a), ("y", &b));
    let c2 = clause(&right, ("x", &a), ("y", &b));
    let source = rev(
        &model,
        vec![(a.clone(), class.clone()), (b.clone(), class.clone())],
        vec![
            relation(&left, ("x", &class), ("y", &class)),
            relation(&right, ("x", &class), ("y", &class)),
            relation(&goal, ("x", &class), ("y", &class)),
        ],
        vec![c1.clone(), c2.clone()],
        vec![
            law(
                &model,
                "m/left-goal",
                RelationalContent::new(
                    left.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
                RelationalContent::new(
                    goal.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
            ),
            law(
                &model,
                "m/right-goal",
                RelationalContent::new(
                    right.clone(),
                    BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
                )
                .unwrap(),
                RelationalContent::new(
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
    assert_eq!(all.interventions()[0].withdrawals(), &[c1, c2]);
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
    let model = referent_id("network");
    let node = referent_id("classification/node");
    let alpha = scoped_id(&model, "Alpha");
    let beta = scoped_id(&model, "Beta");
    let gamma = scoped_id(&model, "Gamma");
    let omega = scoped_id(&model, "Omega");
    let link = relation_id("network/link");
    let reaches = relation_id("network/reaches");
    let subject = variable("subject");
    let middle = variable("middle");
    let destination = variable("destination");
    let alpha_beta = clause(&link, ("x", &alpha), ("y", &beta));
    let beta_omega = clause(&link, ("x", &beta), ("y", &omega));
    let alpha_gamma = clause(&link, ("x", &alpha), ("y", &gamma));
    let gamma_omega = clause(&link, ("x", &gamma), ("y", &omega));
    let source = rev(
        &model,
        vec![
            (alpha.clone(), node.clone()),
            (beta.clone(), node.clone()),
            (gamma.clone(), node.clone()),
            (omega.clone(), node.clone()),
        ],
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
        vec![rule(
            &model,
            "network/path",
            vec![
                RelationalContent::new(
                    link.clone(),
                    BTreeMap::from([
                        (role_id("x"), subject.clone()),
                        (role_id("y"), middle.clone()),
                    ]),
                )
                .unwrap(),
                RelationalContent::new(
                    link.clone(),
                    BTreeMap::from([
                        (role_id("x"), middle.clone()),
                        (role_id("y"), destination.clone()),
                    ]),
                )
                .unwrap(),
            ],
            RelationalContent::new(
                reaches.clone(),
                BTreeMap::from([(role_id("x"), subject), (role_id("y"), destination)]),
            )
            .unwrap(),
        )],
    );
    let target = clause(&reaches, ("x", &alpha), ("y", &omega));
    let expected_base = BTreeSet::from([
        vec![alpha_beta.clone(), alpha_gamma.clone()],
        vec![alpha_beta.clone(), gamma_omega.clone()],
        vec![alpha_gamma.clone(), beta_omega.clone()],
        vec![gamma_omega.clone(), beta_omega.clone()],
    ]);
    let PreventAll::Complete(base) =
        prevent_all_minimal(&source, target.clone(), vec![link.clone()], exhaustive).unwrap()
    else {
        panic!("finite redundant prevention frontier must be complete");
    };
    assert_eq!(
        base.iter()
            .map(|item| item.withdrawals().to_vec())
            .collect::<BTreeSet<_>>(),
        expected_base,
    );

    let occurrence = source
        .model()
        .occurrences()
        .iter()
        .find(|occurrence| occurrence.content() == alpha_beta.id())
        .expect("fixture assertion has an occurrence")
        .clone();
    let judgment = source
        .model()
        .judgments()
        .iter()
        .find(|judgment| {
            matches!(
                judgment.target(),
                JudgmentTarget::Occurrence(id) if id == occurrence.id()
            )
        })
        .expect("fixture assertion occurrence has an admission judgment")
        .clone();
    let withdrawal_atoms = vec![
        SemanticAtom::AssertionOccurrence(occurrence),
        SemanticAtom::Judgment(judgment),
    ];
    let successor_delta = Delta::new(
        source.identity().clone(),
        Vec::new(),
        withdrawal_atoms.clone(),
    )
    .unwrap();
    assert!(successor_delta.admissions().is_empty());
    assert_eq!(successor_delta.withdrawals(), withdrawal_atoms.as_slice());
    let successor = successor_delta.apply(&source).unwrap();
    assert_eq!(
        successor.lineage(),
        &RevisionLineage::Successor(successor_delta)
    );
    let PreventAll::Complete(successor_prevention) =
        prevent_all_minimal(&successor, target, vec![link.clone()], exhaustive).unwrap()
    else {
        panic!("degraded finite prevention frontier must be complete");
    };
    assert_eq!(
        successor_prevention
            .iter()
            .map(|item| item.withdrawals().to_vec())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([vec![alpha_gamma.clone()], vec![gamma_omega.clone()]]),
    );

    let choices = referent_id("choices");
    let start = scoped_id(&choices, "Start");
    let finish = scoped_id(&choices, "Finish");
    let first = relation_id("choices/first");
    let second = relation_id("choices/second");
    let achieved = relation_id("choices/achieved");
    let from = variable("from");
    let to = variable("to");
    let choices_source = rev(
        &choices,
        vec![
            (start.clone(), node.clone()),
            (finish.clone(), node.clone()),
        ],
        vec![
            relation(&first, ("x", &node), ("y", &node)),
            relation(&second, ("x", &node), ("y", &node)),
            relation(&achieved, ("x", &node), ("y", &node)),
        ],
        Vec::new(),
        vec![
            law(
                &choices,
                "choices/first-achieves",
                RelationalContent::new(
                    first.clone(),
                    BTreeMap::from([(role_id("x"), from.clone()), (role_id("y"), to.clone())]),
                )
                .unwrap(),
                RelationalContent::new(
                    achieved.clone(),
                    BTreeMap::from([(role_id("x"), from.clone()), (role_id("y"), to.clone())]),
                )
                .unwrap(),
            ),
            law(
                &choices,
                "choices/second-achieves",
                RelationalContent::new(
                    second.clone(),
                    BTreeMap::from([(role_id("x"), from), (role_id("y"), to)]),
                )
                .unwrap(),
                RelationalContent::new(
                    achieved.clone(),
                    BTreeMap::from([
                        (role_id("x"), variable("from")),
                        (role_id("y"), variable("to")),
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
            .map(|item| item.admissions().to_vec())
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
    let model = referent_id("m");
    let class = referent_id("classification/thing");
    let a = scoped_id(&model, "A");
    let b = scoped_id(&model, "B");
    let input = relation_id("m/input");
    let goal = relation_id("m/goal");
    let va = variable("a");
    let vb = variable("b");
    let asserted = clause(&input, ("x", &a), ("y", &b));
    let source = rev(
        &model,
        vec![(a.clone(), class.clone()), (b.clone(), class.clone())],
        vec![
            relation(&input, ("x", &class), ("y", &class)),
            relation(&goal, ("x", &class), ("y", &class)),
        ],
        vec![asserted.clone()],
        vec![law(
            &model,
            "m/copy",
            RelationalContent::new(
                input.clone(),
                BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
            )
            .unwrap(),
            RelationalContent::new(
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
    assert_eq!(intervention.withdrawals(), &[asserted]);
}

#[test]
fn absent_candidate_basis_is_impossible_not_a_empty_delta() {
    let model = referent_id("m");
    let class = referent_id("classification/thing");
    let a = scoped_id(&model, "A");
    let b = scoped_id(&model, "B");
    let input = relation_id("m/input");
    let goal = relation_id("m/goal");
    let source = rev(
        &model,
        vec![(a.clone(), class.clone()), (b.clone(), class.clone())],
        vec![
            relation(&input, ("x", &class), ("y", &class)),
            relation(&goal, ("x", &class), ("y", &class)),
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
    let model = referent_id("m");
    let class = referent_id("classification/thing");
    let a = scoped_id(&model, "A");
    let b = scoped_id(&model, "B");
    let input = relation_id("m/input");
    let goal = relation_id("m/goal");
    let va = variable("a");
    let vb = variable("b");
    let source = rev(
        &model,
        vec![(a.clone(), class.clone()), (b.clone(), class.clone())],
        vec![
            relation(&input, ("x", &class), ("y", &class)),
            relation(&goal, ("x", &class), ("y", &class)),
        ],
        vec![],
        vec![law(
            &model,
            "m/copy",
            RelationalContent::new(
                input.clone(),
                BTreeMap::from([(role_id("x"), va.clone()), (role_id("y"), vb.clone())]),
            )
            .unwrap(),
            RelationalContent::new(
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
    let model = referent_id("m");
    let alpha = scoped_id(&model, "Alpha");
    let beta = scoped_id(&model, "Beta");
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
