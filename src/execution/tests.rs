use super::*;
use crate::{
    intrinsic::{Intrinsic, IntrinsicRole},
    kernel::{
        AssertionOccurrence, Cardinality, Definition, DerivationRule, Judgment, JudgmentKind,
        JudgmentStatus, JudgmentTarget, LookupMode, Model, Pattern, PatternId, Referent,
        ReferentId, RelationShape, RelationalContent, Revision, Role, RoleId, StructuralContract,
        StructuralForm, Term,
    },
    wire,
};
use std::collections::BTreeMap;

fn referent_id(value: &str) -> ReferentId {
    ReferentId::from_digest(wire::sha256_digest(value.as_bytes()))
}
fn role_id(value: &str) -> RoleId {
    RoleId::from_digest(wire::sha256_digest(value.as_bytes()))
}
fn pattern_id(value: &str) -> PatternId {
    PatternId::from_digest(wire::sha256_digest(value.as_bytes()))
}
fn referent(value: &str) -> Term {
    Term::referent(referent_id(value))
}
fn variable(value: &str) -> Term {
    Term::pattern(pattern_id(value))
}
fn clause(relation: &ReferentId, from: Term, to: Term) -> RelationalContent {
    RelationalContent::new(
        relation.clone(),
        BTreeMap::from([(role_id("from"), from), (role_id("to"), to)]),
    )
    .unwrap()
}
fn relation(id: &ReferentId) -> RelationShape {
    let from = Role::new(role_id("from"), Vec::new()).unwrap();
    let to = Role::new(role_id("to"), Vec::new()).unwrap();
    RelationShape::new(
        id.clone(),
        BTreeMap::from([
            (from.id().clone(), from.clone()),
            (to.id().clone(), to.clone()),
        ]),
        vec![
            LookupMode::finite(
                vec![from.id().clone()],
                vec![to.id().clone()],
                Cardinality::Many,
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn rule(
    identity: &str,
    premises: Vec<RelationalContent>,
    conclusion: RelationalContent,
) -> (DerivationRule, Vec<RelationalContent>) {
    let rule = DerivationRule::new(
        referent_id(identity),
        referent_id("map/scope"),
        referent_id("map/authority"),
        Pattern::new(premises.iter().map(|item| item.id().clone()).collect()).unwrap(),
        Pattern::new(vec![conclusion.id().clone()]).unwrap(),
    )
    .unwrap();
    let mut contents = premises;
    contents.push(conclusion);
    (rule, contents)
}

fn declare(referents: &mut BTreeMap<ReferentId, Referent>, id: ReferentId) {
    referents.insert(id.clone(), Referent::new(id));
}

fn declare_content_referents(
    referents: &mut BTreeMap<ReferentId, Referent>,
    content: &RelationalContent,
) {
    declare(referents, content.relation().clone());
    for term in content.roles().values() {
        if let Term::Referent(id) = term {
            declare(referents, id.clone());
        }
    }
}

fn revision(
    assertions: Vec<RelationalContent>,
    rule_fixtures: Vec<(DerivationRule, Vec<RelationalContent>)>,
) -> Revision {
    let model_id = referent_id("map");
    let links = referent_id("map/links");
    let reaches = referent_id("map/reaches");
    let source = referent_id("map/source");
    let scope = referent_id("map/scope");
    let policy = referent_id("map/admission-policy");
    let mut referents = BTreeMap::new();
    for id in [
        model_id.clone(),
        links.clone(),
        reaches.clone(),
        source.clone(),
        scope.clone(),
        policy.clone(),
        referent_id("map/authority"),
    ] {
        declare(&mut referents, id);
    }
    for value in ["North", "South", "Store", "Relay", "Beagle"] {
        declare(&mut referents, referent_id(value));
    }
    let mut relational_contents = BTreeMap::new();
    for content in &assertions {
        declare_content_referents(&mut referents, content);
        relational_contents.insert(content.id().clone(), content.clone());
    }
    let mut rules = Vec::new();
    for (rule, contents) in rule_fixtures {
        declare(&mut referents, rule.id().clone());
        declare(&mut referents, rule.scope().clone());
        declare(&mut referents, rule.authority().clone());
        for content in contents {
            declare_content_referents(&mut referents, &content);
            relational_contents.insert(content.id().clone(), content);
        }
        rules.push(rule);
    }
    let mut occurrences = Vec::new();
    let mut judgments = Vec::new();
    for assertion in assertions {
        let occurrence_id = referent_id(&format!(
            "map/assertion-occurrence/{}",
            assertion.id().as_str()
        ));
        let judgment_id = referent_id(&format!(
            "map/admission-judgment/{}",
            assertion.id().as_str()
        ));
        declare(&mut referents, occurrence_id.clone());
        declare(&mut referents, judgment_id.clone());
        occurrences.push(AssertionOccurrence::new(
            occurrence_id.clone(),
            assertion.id().clone(),
            source.clone(),
            model_id.clone(),
        ));
        judgments.push(Judgment::new(
            judgment_id,
            model_id.clone(),
            model_id.clone(),
            JudgmentTarget::Occurrence(occurrence_id),
            JudgmentKind::Admitted {
                policy: policy.clone(),
                basis: Vec::new(),
            },
            JudgmentStatus::Affirmed,
        ));
    }
    wire::admit(
        Model::with_distinctions(
            model_id,
            referents,
            relational_contents,
            BTreeMap::from([
                (links.clone(), relation(&links)),
                (reaches.clone(), relation(&reaches)),
            ]),
            BTreeMap::new(),
            occurrences,
            Vec::new(),
            rules,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            judgments,
        )
        .unwrap(),
    )
}
fn limits() -> Limits {
    Limits::new(100, 10, 10_000)
}
fn chain_rules() -> Vec<(DerivationRule, Vec<RelationalContent>)> {
    let links = referent_id("map/links");
    let reaches = referent_id("map/reaches");
    let source = variable("source");
    let middle = variable("middle");
    let destination = variable("destination");
    vec![
        rule(
            "map/direct",
            vec![clause(&links, source.clone(), destination.clone())],
            clause(&reaches, source.clone(), destination.clone()),
        ),
        rule(
            "map/recursive",
            vec![
                clause(&reaches, source.clone(), middle.clone()),
                clause(&links, middle, destination.clone()),
            ],
            clause(&reaches, source, destination),
        ),
    ]
}
fn asserted(relation: &str, from: &str, to: &str) -> RelationalContent {
    clause(&referent_id(relation), referent(from), referent(to))
}
fn find_plan(revision: &Revision, from: &str) -> crate::kernel::FindPlan {
    let target = pattern_id("target");
    crate::kernel::FindPlan::new(
        revision.model(),
        &clause(
            &referent_id("map/reaches"),
            referent(from),
            Term::pattern(target.clone()),
        ),
        target,
    )
    .unwrap()
}

#[test]
fn find_discriminates_known_referent_bindings_and_returns_referent_terms() {
    let revision = revision(
        vec![
            asserted("map/links", "North", "Store"),
            asserted("map/links", "South", "Relay"),
        ],
        chain_rules(),
    );
    assert_eq!(
        find(&revision, &find_plan(&revision, "North"), limits()).unwrap(),
        vec![referent("Store")]
    );
    assert_eq!(
        find(&revision, &find_plan(&revision, "South"), limits()).unwrap(),
        vec![referent("Relay")]
    );
}

#[test]
fn find_returns_recursive_derived_referents_in_canonical_order() {
    let revision = revision(
        vec![
            asserted("map/links", "North", "Store"),
            asserted("map/links", "Store", "Beagle"),
        ],
        chain_rules(),
    );
    let result = find(&revision, &find_plan(&revision, "North"), limits()).unwrap();
    assert_eq!(result, vec![referent("Beagle"), referent("Store")]);
    assert!(result.iter().all(|term| matches!(term, Term::Referent(_))));
}

fn selection_clause(values: [Term; 5]) -> RelationalContent {
    RelationalContent::new(
        referent_id("selection/related"),
        ["scope", "a", "b", "c", "d"]
            .into_iter()
            .zip(values)
            .map(|(role, value)| (role_id(role), value))
            .collect(),
    )
    .unwrap()
}

fn selection_revision(assertions: Vec<RelationalContent>) -> Revision {
    let model_id = referent_id("selection");
    let relation_id = referent_id("selection/related");
    let source = referent_id("selection/source");
    let policy = referent_id("selection/policy");
    let roles = ["scope", "a", "b", "c", "d"]
        .into_iter()
        .map(|name| {
            let id = role_id(name);
            (id.clone(), Role::new(id, Vec::new()).unwrap())
        })
        .collect::<BTreeMap<_, _>>();
    let shape = RelationShape::new(
        relation_id.clone(),
        roles,
        vec![
            LookupMode::finite(
                vec![role_id("scope")],
                ["a", "b", "c", "d"].into_iter().map(role_id).collect(),
                Cardinality::Many,
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let mut referents = BTreeMap::new();
    for id in [
        model_id.clone(),
        relation_id.clone(),
        source.clone(),
        policy.clone(),
        referent_id("World"),
        referent_id("A"),
        referent_id("B"),
        referent_id("C"),
        referent_id("D"),
    ] {
        declare(&mut referents, id);
    }
    let relational_contents = assertions
        .iter()
        .map(|content| (content.id().clone(), content.clone()))
        .collect();
    let mut occurrences = Vec::new();
    let mut judgments = Vec::new();
    for assertion in assertions {
        let occurrence_id =
            referent_id(&format!("selection/occurrence/{}", assertion.id().as_str()));
        let judgment_id = referent_id(&format!("selection/judgment/{}", assertion.id().as_str()));
        declare(&mut referents, occurrence_id.clone());
        declare(&mut referents, judgment_id.clone());
        occurrences.push(AssertionOccurrence::new(
            occurrence_id.clone(),
            assertion.id().clone(),
            source.clone(),
            model_id.clone(),
        ));
        judgments.push(Judgment::new(
            judgment_id,
            model_id.clone(),
            model_id.clone(),
            JudgmentTarget::Occurrence(occurrence_id),
            JudgmentKind::Admitted {
                policy: policy.clone(),
                basis: Vec::new(),
            },
            JudgmentStatus::Affirmed,
        ));
    }
    wire::admit(
        Model::with_distinctions(
            model_id,
            referents,
            relational_contents,
            BTreeMap::from([(relation_id, shape)]),
            BTreeMap::new(),
            occurrences,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            judgments,
        )
        .unwrap(),
    )
}

#[test]
fn select_correlates_named_holes_without_correlating_anonymous_holes() {
    let world = referent("World");
    let revision = selection_revision(vec![
        selection_clause([
            world.clone(),
            referent("A"),
            referent("B"),
            referent("B"),
            referent("C"),
        ]),
        selection_clause([
            world.clone(),
            referent("A"),
            referent("B"),
            referent("C"),
            referent("D"),
        ]),
        selection_clause([
            world.clone(),
            referent("C"),
            referent("B"),
            referent("B"),
            referent("A"),
        ]),
    ]);
    let first = pattern_id("anonymous/first");
    let same = pattern_id("named/same");
    let second = pattern_id("anonymous/second");
    let pattern = selection_clause([
        world,
        Term::pattern(first.clone()),
        Term::pattern(same.clone()),
        Term::pattern(same.clone()),
        Term::pattern(second.clone()),
    ]);
    let plan =
        crate::kernel::QueryPlan::derive(revision.model(), &pattern, vec![first, same, second])
            .unwrap();
    let origins = |roles: &[&str]| {
        let mut roles = roles.iter().map(|role| role_id(role)).collect::<Vec<_>>();
        roles.sort();
        roles
    };
    let mut expected = vec![
        QueryRow {
            cells: vec![
                QueryCell {
                    origins: origins(&["a"]),
                    value: referent("A"),
                },
                QueryCell {
                    origins: origins(&["b", "c"]),
                    value: referent("B"),
                },
                QueryCell {
                    origins: origins(&["d"]),
                    value: referent("C"),
                },
            ],
        },
        QueryRow {
            cells: vec![
                QueryCell {
                    origins: origins(&["a"]),
                    value: referent("C"),
                },
                QueryCell {
                    origins: origins(&["b", "c"]),
                    value: referent("B"),
                },
                QueryCell {
                    origins: origins(&["d"]),
                    value: referent("A"),
                },
            ],
        },
    ];
    expected.sort();
    assert_eq!(select(&revision, &plan, limits()).unwrap(), expected);
}

#[test]
fn why_projects_one_canonical_revision_scoped_proof() {
    let revision = revision(vec![asserted("map/links", "North", "Store")], chain_rules());
    let target = asserted("map/reaches", "North", "Store");
    let proof = why(&revision, &target, limits()).unwrap().unwrap();
    assert_eq!(proof.revision, *revision.identity());
    assert_eq!(proof.why.root, 0);
    assert!(
        matches!(&proof.why.witnesses[0].witness, Witness::Derived { rule, .. } if rule == &referent_id("map/direct"))
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
        chain_rules(),
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
    let revision = revision(vec![asserted("map/links", "North", "Store")], chain_rules());
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
    let forward = revision(assertions.clone(), chain_rules());
    let reverse = revision(assertions.into_iter().rev().collect(), chain_rules());
    assert_eq!(
        why(&forward, &target, limits()).unwrap().unwrap().why,
        why(&reverse, &target, limits()).unwrap().unwrap().why
    );
}

fn intrinsic_shape(intrinsic: Intrinsic) -> RelationShape {
    let result = intrinsic.role(IntrinsicRole::Result);
    let mut roles = intrinsic
        .input_roles()
        .iter()
        .map(|role| {
            let id = intrinsic.role(*role);
            (id.clone(), Role::new(id, Vec::new()).unwrap())
        })
        .collect::<BTreeMap<_, _>>();
    roles.insert(
        result.clone(),
        Role::new(result.clone(), Vec::new()).unwrap(),
    );
    RelationShape::new(
        intrinsic.relation(),
        roles,
        vec![
            LookupMode::finite(
                intrinsic
                    .input_roles()
                    .iter()
                    .map(|role| intrinsic.role(*role))
                    .collect(),
                vec![result],
                Cardinality::One,
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn intrinsic_content(intrinsic: Intrinsic, roles: &[(IntrinsicRole, Term)]) -> RelationalContent {
    RelationalContent::new(
        intrinsic.relation(),
        roles
            .iter()
            .map(|(role, term)| (intrinsic.role(*role), term.clone()))
            .collect(),
    )
    .unwrap()
}

#[test]
fn evaluate_uses_indexed_intrinsics_short_circuits_and_memoizes() {
    let model_id = referent_id("pure/model");
    let result_id = referent_id("pure/result");
    let f32_domain = referent_id("pure/F32");
    let bool_domain = referent_id("pure/Bool");
    let tuple_shape = referent_id("pure/tuple(F32,F32)");
    let length = intrinsic_content(
        Intrinsic::Length,
        &[(
            IntrinsicRole::Input,
            Term::tuple(
                tuple_shape.clone(),
                vec![
                    (f32_domain.clone(), Term::f32(3.0).unwrap()),
                    (f32_domain.clone(), Term::f32(4.0).unwrap()),
                ],
            )
            .unwrap(),
        )],
    );
    let divide = intrinsic_content(
        Intrinsic::Divide,
        &[
            (IntrinsicRole::Left, Term::f32(1.0).unwrap()),
            (IntrinsicRole::Right, Term::f32(0.0).unwrap()),
        ],
    );
    let conditional = intrinsic_content(
        Intrinsic::Conditional,
        &[
            (IntrinsicRole::Condition, Term::boolean(true)),
            (IntrinsicRole::Then, Term::application(length.id().clone())),
            (IntrinsicRole::Else, Term::application(divide.id().clone())),
        ],
    );
    let referents = [
        model_id.clone(),
        result_id.clone(),
        f32_domain.clone(),
        bool_domain.clone(),
        tuple_shape.clone(),
        Intrinsic::Length.relation(),
        Intrinsic::Divide.relation(),
        Intrinsic::Conditional.relation(),
    ]
    .into_iter()
    .map(|id| (id.clone(), Referent::new(id)))
    .collect();
    let contents = [length.clone(), divide.clone(), conditional.clone()]
        .into_iter()
        .map(|content| (content.id().clone(), content))
        .collect();
    let shapes = [
        intrinsic_shape(Intrinsic::Length),
        intrinsic_shape(Intrinsic::Divide),
        intrinsic_shape(Intrinsic::Conditional),
    ]
    .into_iter()
    .map(|shape| (shape.referent().clone(), shape))
    .collect();
    let structural_contracts = [
        StructuralContract::new(f32_domain.clone(), StructuralForm::F32).unwrap(),
        StructuralContract::new(bool_domain, StructuralForm::Bool).unwrap(),
    ]
    .into_iter()
    .map(|contract| (contract.referent().clone(), contract))
    .collect();
    let revision = wire::admit(
        Model::with_distinctions(
            model_id,
            referents,
            contents,
            shapes,
            structural_contracts,
            Vec::new(),
            vec![Definition::new(
                result_id.clone(),
                Term::application(conditional.id().clone()),
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap(),
    );

    assert!(revision.model().occurrences().is_empty());
    assert!(revision.model().judgments().is_empty());
    assert!(revision.model().admitted_contents().is_empty());
    assert_eq!(
        evaluate(&revision, &Term::referent(result_id)).unwrap(),
        Term::f32(5.0).unwrap()
    );
    assert!(evaluate(&revision, &Term::application(divide.id().clone())).is_err());

    let repeated = Term::tuple(
        tuple_shape.clone(),
        vec![
            (f32_domain.clone(), Term::application(length.id().clone())),
            (f32_domain.clone(), Term::application(length.id().clone())),
        ],
    )
    .unwrap();
    let (evaluated, operations) = super::evaluate::evaluate_with_operations(&revision, &repeated)
        .expect("shared application evaluates");
    assert_eq!(
        evaluated,
        Term::tuple(
            tuple_shape,
            vec![
                (f32_domain.clone(), Term::f32(5.0).unwrap()),
                (f32_domain, Term::f32(5.0).unwrap()),
            ],
        )
        .unwrap()
    );
    assert_eq!(operations, 1);
}

#[test]
fn pure_map_and_one_coin_score_have_exact_deterministic_work_budgets() {
    const SOURCE: &str = r#"F32

lengths:
  input: [(3.0, 4.0), (5.0, 12.0)]
  map length over input

frame velocity:
  direction: (1.0, 0.0)
  direction * 300.0

frame next position:
  position: (0.0, 0.0)
  dt: 0.5
  position + frame velocity * dt

frame collision:
  coin: (160.0, 0.0)
  player radius: 12.0
  coin radius: 8.0
  length (frame next position - coin) <= player radius + coin radius

frame collected:
  if frame collision then true else false

frame score:
  if frame collected then 10 else 0
"#;

    let model_id = referent_id("pure/budget-model");
    let program = crate::elaborate::compile_in(
        crate::frontend::parse(SOURCE).expect("budget witness parses"),
        crate::elaborate::ModelContext::new(model_id.clone()),
    )
    .expect("budget witness compiles");
    let revision = program.context_revision().expect("budget Revision");
    let evaluate_named = |name: &str| {
        let id = program
            .designations()
            .scoped(&model_id, name)
            .unwrap_or_else(|error| panic!("definition '{name}' resolves: {error}"));
        super::evaluate::evaluate_with_operations(revision, &Term::referent(id))
            .unwrap_or_else(|error| panic!("definition '{name}' evaluates: {error}"))
    };

    let (lengths, map_operations) = evaluate_named("lengths");
    let f32_domain = program.designations().global("F32").unwrap();
    assert_eq!(
        lengths,
        Term::sequence(
            crate::kernel::structural_sequence_domain(&f32_domain),
            f32_domain,
            vec![Term::f32(5.0).unwrap(), Term::f32(13.0).unwrap()],
        )
        .unwrap()
    );
    assert_eq!(map_operations, 3);

    let (score, coin_operations) = evaluate_named("frame score");
    assert_eq!(score, Term::int(10));
    assert_eq!(coin_operations, 9);
}
