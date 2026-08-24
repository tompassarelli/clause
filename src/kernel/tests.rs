use std::collections::BTreeMap;

use super::*;

fn referent(byte: u8) -> ReferentId {
    ReferentId::from_digest([byte; 32])
}

#[test]
fn roles_are_opaque_and_terms_remain_distinct_from_referents() {
    let role = RoleId::from_digest([7; 32]);
    assert!(RoleId::new("role-name".into()).is_err());
    assert_eq!(
        role.as_str(),
        "role-sha256-0707070707070707070707070707070707070707070707070707070707070707"
    );

    let variable = PatternId::from_digest([8; 32]);
    assert!(Term::referent(referent(1)).is_ground());
    assert!(!Term::pattern(variable).is_ground());
}

#[test]
fn relation_contract_uses_stable_roles_without_layout_or_type_identity() {
    let model = referent(1);
    let relation = referent(2);
    let role = RoleId::from_digest([3; 32]);
    let shape = RelationShape::new(
        relation.clone(),
        BTreeMap::from([(role.clone(), Role::new(role, Vec::new()).unwrap())]),
        Vec::new(),
    )
    .unwrap();
    let result = Model::new(
        model.clone(),
        BTreeMap::from([
            (model.clone(), Referent::new(model)),
            (relation.clone(), Referent::new(relation.clone())),
        ]),
        BTreeMap::new(),
        BTreeMap::from([(relation, shape)]),
        Vec::new(),
        Vec::new(),
    );
    assert!(result.is_ok());
}

#[test]
fn only_exact_model_authority_and_scope_make_content_operative() {
    let model = referent(10);
    let relation = referent(11);
    let participant = referent(12);
    let occurrence = referent(13);
    let foreign_authority = referent(14);
    let judgment = referent(15);
    let role_id = RoleId::from_digest([16; 32]);
    let shape = RelationShape::new(
        relation.clone(),
        BTreeMap::from([(
            role_id.clone(),
            Role::new(role_id.clone(), Vec::new()).unwrap(),
        )]),
        Vec::new(),
    )
    .unwrap();
    let content = RelationalContent::new(
        relation.clone(),
        BTreeMap::from([(role_id, Term::referent(participant.clone()))]),
    )
    .unwrap();
    let referents = [
        model.clone(),
        relation.clone(),
        participant,
        occurrence.clone(),
        foreign_authority.clone(),
        judgment.clone(),
    ]
    .into_iter()
    .map(|id| (id.clone(), Referent::new(id)))
    .collect();
    let result = Model::with_distinctions(
        model.clone(),
        referents,
        BTreeMap::from([(content.id().clone(), content.clone())]),
        BTreeMap::from([(relation, shape)]),
        vec![AssertionOccurrence::new(
            occurrence.clone(),
            content.id().clone(),
            foreign_authority.clone(),
            model.clone(),
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![Judgment::new(
            judgment,
            foreign_authority,
            model.clone(),
            JudgmentTarget::Occurrence(occurrence),
            JudgmentKind::Admitted {
                policy: model.clone(),
                basis: Vec::new(),
            },
            JudgmentStatus::Affirmed,
        )],
    )
    .unwrap();

    assert_eq!(result.aggregate_status(&content), OpenWorldStatus::Admitted);
    assert_eq!(
        result.operative_status(&content),
        OpenWorldStatus::Undetermined
    );
    assert!(result.admitted_contents().is_empty());
}

#[test]
fn derived_judgments_validate_recursive_groundness() {
    let model = referent(20);
    let premise_relation = referent(21);
    let conclusion_relation = referent(22);
    let nested_relation = referent(23);
    let leaf_relation = referent(24);
    let value = referent(25);
    let rule_id = referent(26);
    let judgment_id = referent(27);
    let role_id = RoleId::from_digest([28; 32]);
    let binder = PatternId::from_digest([29; 32]);
    let input_role = RoleId::from_digest([31; 32]);
    let result_role = RoleId::from_digest([32; 32]);

    let leaf = RelationalContent::new(
        leaf_relation.clone(),
        BTreeMap::from([(input_role.clone(), Term::referent(value.clone()))]),
    )
    .unwrap();
    let nested = RelationalContent::new(
        nested_relation.clone(),
        BTreeMap::from([(input_role.clone(), Term::application(leaf.id().clone()))]),
    )
    .unwrap();
    let premise_pattern = RelationalContent::new(
        premise_relation.clone(),
        BTreeMap::from([(role_id.clone(), Term::pattern(binder.clone()))]),
    )
    .unwrap();
    let conclusion_pattern = RelationalContent::new(
        conclusion_relation.clone(),
        BTreeMap::from([(role_id.clone(), Term::pattern(binder))]),
    )
    .unwrap();
    let grounded_premise = RelationalContent::new(
        premise_relation.clone(),
        BTreeMap::from([(role_id.clone(), Term::application(nested.id().clone()))]),
    )
    .unwrap();
    let grounded_target = RelationalContent::new(
        conclusion_relation.clone(),
        BTreeMap::from([(role_id.clone(), Term::application(nested.id().clone()))]),
    )
    .unwrap();
    let unresolved_premise = RelationalContent::new(
        premise_relation.clone(),
        BTreeMap::from([(
            role_id.clone(),
            Term::pattern(PatternId::from_digest([30; 32])),
        )]),
    )
    .unwrap();
    let rule = DerivationRule::new(
        rule_id.clone(),
        model.clone(),
        model.clone(),
        Pattern::new(vec![premise_pattern.id().clone()]).unwrap(),
        Pattern::new(vec![conclusion_pattern.id().clone()]).unwrap(),
    )
    .unwrap();
    let referents = [
        model.clone(),
        premise_relation.clone(),
        conclusion_relation.clone(),
        nested_relation.clone(),
        leaf_relation.clone(),
        value,
        rule_id.clone(),
        judgment_id.clone(),
    ]
    .into_iter()
    .map(|id| (id.clone(), Referent::new(id)))
    .collect::<BTreeMap<_, _>>();
    let root_shapes = [premise_relation, conclusion_relation]
        .into_iter()
        .map(|relation| {
            let role = Role::new(role_id.clone(), Vec::new()).unwrap();
            (
                relation.clone(),
                RelationShape::new(
                    relation,
                    BTreeMap::from([(role_id.clone(), role)]),
                    Vec::new(),
                )
                .unwrap(),
            )
        });
    let application_shapes = [nested_relation, leaf_relation]
        .into_iter()
        .map(|relation| {
            let input = Role::new(input_role.clone(), Vec::new()).unwrap();
            let result = Role::new(result_role.clone(), Vec::new()).unwrap();
            (
                relation.clone(),
                RelationShape::new(
                    relation,
                    BTreeMap::from([(input_role.clone(), input), (result_role.clone(), result)]),
                    vec![
                        LookupMode::finite(
                            vec![input_role.clone()],
                            vec![result_role.clone()],
                            Cardinality::One,
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            )
        });
    let shapes = root_shapes
        .chain(application_shapes)
        .collect::<BTreeMap<_, _>>();
    let build_model = |premise: RelationalContent| {
        let premise_id = premise.id().clone();
        Model::with_distinctions(
            model.clone(),
            referents.clone(),
            [
                leaf.clone(),
                nested.clone(),
                premise_pattern.clone(),
                conclusion_pattern.clone(),
                premise,
                grounded_target.clone(),
            ]
            .into_iter()
            .map(|content| (content.id().clone(), content))
            .collect(),
            shapes.clone(),
            Vec::new(),
            Vec::new(),
            vec![rule.clone()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![Judgment::new(
                judgment_id.clone(),
                model.clone(),
                model.clone(),
                JudgmentTarget::Content(grounded_target.id().clone()),
                JudgmentKind::Derived {
                    rule: rule_id.clone(),
                    premises: vec![premise_id],
                },
                JudgmentStatus::Affirmed,
            )],
        )
    };

    assert!(!Term::application(nested.id().clone()).is_ground());
    assert!(build_model(grounded_premise).is_ok());
    assert_eq!(
        build_model(unresolved_premise).unwrap_err().to_string(),
        "derived judgment basis and target do not instantiate its derivation rule"
    );
}

#[test]
fn recursive_application_requires_one_unambiguous_result_contract() {
    let model = referent(40);
    let parent_relation = referent(41);
    let application_relation = referent(42);
    let value = referent(43);
    let parent_role = RoleId::from_digest([44; 32]);
    let input_role = RoleId::from_digest([45; 32]);
    let result_role = RoleId::from_digest([46; 32]);
    let child = RelationalContent::new(
        application_relation.clone(),
        BTreeMap::from([(input_role.clone(), Term::referent(value.clone()))]),
    )
    .unwrap();
    let parent = RelationalContent::new(
        parent_relation.clone(),
        BTreeMap::from([(parent_role.clone(), Term::application(child.id().clone()))]),
    )
    .unwrap();
    let referents = [
        model.clone(),
        parent_relation.clone(),
        application_relation.clone(),
        value,
    ]
    .into_iter()
    .map(|id| (id.clone(), Referent::new(id)))
    .collect::<BTreeMap<_, _>>();
    let parent_shape = RelationShape::new(
        parent_relation.clone(),
        BTreeMap::from([(
            parent_role.clone(),
            Role::new(parent_role, Vec::new()).unwrap(),
        )]),
        Vec::new(),
    )
    .unwrap();
    let application_shape = |lookup| {
        RelationShape::new(
            application_relation.clone(),
            BTreeMap::from([
                (
                    input_role.clone(),
                    Role::new(input_role.clone(), Vec::new()).unwrap(),
                ),
                (
                    result_role.clone(),
                    Role::new(result_role.clone(), Vec::new()).unwrap(),
                ),
            ]),
            lookup,
        )
        .unwrap()
    };
    let build = |application_shape| {
        Model::new(
            model.clone(),
            referents.clone(),
            [child.clone(), parent.clone()]
                .into_iter()
                .map(|content| (content.id().clone(), content))
                .collect(),
            BTreeMap::from([
                (parent_relation.clone(), parent_shape.clone()),
                (application_relation.clone(), application_shape),
            ]),
            Vec::new(),
            Vec::new(),
        )
    };
    let one = LookupMode::finite(
        vec![input_role.clone()],
        vec![result_role.clone()],
        Cardinality::One,
    )
    .unwrap();
    assert!(build(application_shape(vec![one.clone()])).is_ok());

    let maybe = LookupMode::finite(
        vec![input_role.clone()],
        vec![result_role.clone()],
        Cardinality::Maybe,
    )
    .unwrap();
    assert_eq!(
        build(application_shape(vec![one, maybe]))
            .unwrap_err()
            .to_string(),
        "recursive term must match exactly one lookup contract by its known roles"
    );
}

#[test]
fn partial_application_content_cannot_become_an_assertion_root() {
    let model = referent(50);
    let parent_relation = referent(51);
    let application_relation = referent(52);
    let value = referent(53);
    let occurrence = referent(54);
    let parent_role = RoleId::from_digest([55; 32]);
    let input_role = RoleId::from_digest([56; 32]);
    let result_role = RoleId::from_digest([57; 32]);
    let child = RelationalContent::new(
        application_relation.clone(),
        BTreeMap::from([(input_role.clone(), Term::referent(value.clone()))]),
    )
    .unwrap();
    let parent = RelationalContent::new(
        parent_relation.clone(),
        BTreeMap::from([(parent_role.clone(), Term::application(child.id().clone()))]),
    )
    .unwrap();
    let referents = [
        model.clone(),
        parent_relation.clone(),
        application_relation.clone(),
        value,
        occurrence.clone(),
    ]
    .into_iter()
    .map(|id| (id.clone(), Referent::new(id)))
    .collect::<BTreeMap<_, _>>();
    let shapes = BTreeMap::from([
        (
            parent_relation.clone(),
            RelationShape::new(
                parent_relation,
                BTreeMap::from([(
                    parent_role.clone(),
                    Role::new(parent_role, Vec::new()).unwrap(),
                )]),
                Vec::new(),
            )
            .unwrap(),
        ),
        (
            application_relation.clone(),
            RelationShape::new(
                application_relation,
                BTreeMap::from([
                    (
                        input_role.clone(),
                        Role::new(input_role.clone(), Vec::new()).unwrap(),
                    ),
                    (
                        result_role.clone(),
                        Role::new(result_role.clone(), Vec::new()).unwrap(),
                    ),
                ]),
                vec![
                    LookupMode::finite(vec![input_role], vec![result_role], Cardinality::One)
                        .unwrap(),
                ],
            )
            .unwrap(),
        ),
    ]);
    let result = Model::new(
        model.clone(),
        referents,
        [child.clone(), parent]
            .into_iter()
            .map(|content| (content.id().clone(), content))
            .collect(),
        shapes,
        vec![AssertionOccurrence::new(
            occurrence,
            child.id().clone(),
            model.clone(),
            model,
        )],
        Vec::new(),
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "relational content must fill the complete named role map"
    );
}

#[test]
fn structural_terms_are_checked_canonical_and_ordered() {
    assert!(Term::f32(f32::NAN).is_err());
    assert!(Term::f32(f32::INFINITY).is_err());
    assert_eq!(Term::f32(-0.0).unwrap(), Term::f32(0.0).unwrap());

    let tuple = Term::tuple((0..11).map(Term::int).collect()).unwrap();
    let Term::Product(fields) = &tuple else {
        panic!("tuple lowers to one labelled structural product");
    };
    assert_eq!(
        fields.keys().map(Name::as_str).collect::<Vec<_>>(),
        (0..11)
            .map(|index| format!("_{index:020}"))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        fields.values().cloned().collect::<Vec<_>>(),
        (0..11).map(Term::int).collect::<Vec<_>>()
    );
    assert!(tuple.is_ground());

    let binder = Term::pattern(PatternId::from_digest([70; 32]));
    assert_eq!(
        Term::sequence(vec![binder.clone()])
            .unwrap_err()
            .to_string(),
        "pattern is not valid inside a structural term"
    );
    assert_eq!(
        Term::sum(Name::new("some".into()).unwrap(), binder)
            .unwrap_err()
            .to_string(),
        "pattern is not valid inside a structural term"
    );

    let relation = referent(71);
    let role = RoleId::from_digest([72; 32]);
    let forward = RelationalContent::new(
        relation.clone(),
        BTreeMap::from([(
            role.clone(),
            Term::sequence(vec![Term::int(1), Term::int(2)]).unwrap(),
        )]),
    )
    .unwrap();
    let repeated = RelationalContent::new(
        relation.clone(),
        BTreeMap::from([(
            role.clone(),
            Term::sequence(vec![Term::int(1), Term::int(2)]).unwrap(),
        )]),
    )
    .unwrap();
    let reversed = RelationalContent::new(
        relation,
        BTreeMap::from([(
            role,
            Term::sequence(vec![Term::int(2), Term::int(1)]).unwrap(),
        )]),
    )
    .unwrap();
    assert_eq!(forward.id(), repeated.id());
    assert_ne!(forward.id(), reversed.id());
}

#[test]
fn structural_definition_recursively_registers_and_validates_applications() {
    let model = referent(80);
    let relation = referent(81);
    let value = referent(82);
    let definition = referent(83);
    let input = RoleId::from_digest([84; 32]);
    let result = RoleId::from_digest([85; 32]);
    let application = RelationalContent::new(
        relation.clone(),
        BTreeMap::from([(input.clone(), Term::referent(value.clone()))]),
    )
    .unwrap();
    let shape = RelationShape::new(
        relation.clone(),
        BTreeMap::from([
            (input.clone(), Role::new(input.clone(), Vec::new()).unwrap()),
            (
                result.clone(),
                Role::new(result.clone(), Vec::new()).unwrap(),
            ),
        ]),
        vec![LookupMode::finite(vec![input], vec![result], Cardinality::One).unwrap()],
    )
    .unwrap();
    let referents = [model.clone(), relation.clone(), value, definition.clone()]
        .into_iter()
        .map(|id| (id.clone(), Referent::new(id)))
        .collect();
    let structural = Term::sequence(vec![
        Term::product(BTreeMap::from([(
            Name::new("result".into()).unwrap(),
            Term::application(application.id().clone()),
        )]))
        .unwrap(),
    ])
    .unwrap();
    let model = Model::with_distinctions(
        model,
        referents,
        BTreeMap::from([(application.id().clone(), application.clone())]),
        BTreeMap::from([(relation, shape)]),
        Vec::new(),
        vec![Definition::new(definition, structural)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();

    assert_eq!(model.content(application.id()), Some(&application));
    assert!(model.content_is_ground(&application));
}
