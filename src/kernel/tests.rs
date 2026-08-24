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
