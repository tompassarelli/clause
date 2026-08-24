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

    let leaf = RelationalContent::new(
        leaf_relation.clone(),
        BTreeMap::from([(role_id.clone(), Term::referent(value.clone()))]),
    )
    .unwrap();
    let nested = RelationalContent::new(
        nested_relation.clone(),
        BTreeMap::from([(role_id.clone(), Term::application(leaf.id().clone()))]),
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
    let shapes = [
        premise_relation,
        conclusion_relation,
        nested_relation,
        leaf_relation,
    ]
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
    })
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
