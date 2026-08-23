use std::collections::{BTreeMap, BTreeSet};

use super::*;

fn name(value: &str) -> Name {
    Name::new(value.to_owned()).unwrap()
}

fn type_id(value: &str) -> TypeId {
    TypeId::new(name(value)).unwrap()
}

fn role(value: &str, typ: &TypeId) -> Role {
    Role::new(RoleId::new(name(value)).unwrap(), typ.clone())
}

fn relation(id: &RelationId, left: &TypeId, right: &TypeId) -> Relation {
    let left_role = role("left", left);
    let right_role = role("right", right);
    Relation::new(
        id.clone(),
        SentenceShape::new(vec![
            InlineSentencePart::Role(left_role.clone()),
            InlineSentencePart::Literal(" relates   to ".to_owned()),
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

fn clause(relation: &RelationId, left: Term, right: Term) -> Clause {
    Clause::new(
        relation.clone(),
        BTreeMap::from([
            (RoleId::new(name("left")).unwrap(), left),
            (RoleId::new(name("right")).unwrap(), right),
        ]),
    )
    .unwrap()
}

fn model(laws: Vec<Law>) -> Result<Model> {
    let model_id = ModelId::new(name("catalog")).unwrap();
    let text = type_id("Text");
    let number = type_id("Number");
    let relation_id = RelationId::new(name("catalog/text")).unwrap();
    Model::new(
        model_id,
        BTreeMap::from([
            (text.clone(), Type::new(text.clone())),
            (number.clone(), Type::new(number)),
        ]),
        BTreeSet::new(),
        BTreeMap::from([(relation_id.clone(), relation(&relation_id, &text, &text))]),
        Vec::new(),
        laws,
    )
}

fn variable(value: &str, typ: &TypeId) -> Term {
    Term::variable(VariableId::new(name(value)).unwrap(), typ.clone())
}

#[test]
fn inline_shape_derives_roles_and_canonicalizes_literals() {
    let text = type_id("Text");
    let relation_id = RelationId::new(name("catalog/mentions")).unwrap();
    let relation = relation(&relation_id, &text, &text);
    assert_eq!(relation.roles().len(), 2);
    assert_eq!(
        relation.shape().parts(),
        &[
            SentencePart::Role(RoleId::new(name("left")).unwrap()),
            SentencePart::Literal("relates to".to_owned()),
            SentencePart::Role(RoleId::new(name("right")).unwrap()),
        ]
    );
}

#[test]
fn model_validation_enforces_types_ground_entities_and_range() {
    let text = type_id("Text");
    let number = type_id("Number");
    let relation_id = RelationId::new(name("catalog/text")).unwrap();
    let malformed = clause(
        &relation_id,
        variable("subject", &text),
        variable("value", &text),
    );
    assert!(
        Model::new(
            ModelId::new(name("catalog")).unwrap(),
            BTreeMap::from([
                (text.clone(), Type::new(text.clone())),
                (number.clone(), Type::new(number)),
            ]),
            BTreeSet::new(),
            BTreeMap::from([(relation_id.clone(), relation(&relation_id, &text, &text))]),
            vec![malformed],
            Vec::new(),
        )
        .is_err()
    );

    let unbound = Law::new(
        LawId::new(name("catalog/unbound")).unwrap(),
        vec![clause(
            &relation_id,
            variable("subject", &text),
            variable("value", &text),
        )],
        clause(
            &relation_id,
            variable("subject", &text),
            variable("fresh", &text),
        ),
    )
    .unwrap();
    assert!(model(vec![unbound]).is_err());
}

#[test]
fn find_plan_is_request_independent_and_mode_checked() {
    let text = type_id("Text");
    let relation_id = RelationId::new(name("catalog/text")).unwrap();
    let model = model(Vec::new()).unwrap();
    let sought = VariableId::new(name("answer")).unwrap();
    let pattern = clause(
        &relation_id,
        Term::value(text.clone(), "known".to_owned()).unwrap(),
        Term::variable(sought.clone(), text),
    );
    let plan = FindPlan::new(&model, &pattern, sought).unwrap();
    assert_eq!(plan.relation(), &relation_id);
    assert_eq!(plan.known().len(), 1);
}

#[test]
fn find_plan_preserves_known_entity_bindings_for_execution() {
    let model_id = ModelId::new(name("catalog")).unwrap();
    let text = type_id("Text");
    let relation_id = RelationId::new(name("catalog/text")).unwrap();
    let left_role = RoleId::new(name("left")).unwrap();
    let right_role = RoleId::new(name("right")).unwrap();
    let first = EntityId::new(model_id.clone(), name("first"), text.clone()).unwrap();
    let second = EntityId::new(model_id.clone(), name("second"), text.clone()).unwrap();
    let first_result = EntityId::new(model_id.clone(), name("first-result"), text.clone()).unwrap();
    let second_result =
        EntityId::new(model_id.clone(), name("second-result"), text.clone()).unwrap();
    let first_fact = clause(
        &relation_id,
        Term::entity(first.clone()),
        Term::entity(first_result.clone()),
    );
    let second_fact = clause(
        &relation_id,
        Term::entity(second.clone()),
        Term::entity(second_result.clone()),
    );
    let model = Model::new(
        model_id,
        BTreeMap::from([(text.clone(), Type::new(text.clone()))]),
        BTreeSet::from([
            first.clone(),
            second.clone(),
            first_result.clone(),
            second_result.clone(),
        ]),
        BTreeMap::from([(relation_id.clone(), relation(&relation_id, &text, &text))]),
        vec![first_fact, second_fact],
        Vec::new(),
    )
    .unwrap();
    let sought = VariableId::new(name("answer")).unwrap();
    let first_pattern = clause(
        &relation_id,
        Term::entity(first.clone()),
        Term::variable(sought.clone(), text.clone()),
    );
    let second_pattern = clause(
        &relation_id,
        Term::entity(second.clone()),
        Term::variable(sought.clone(), text),
    );
    let first_plan = FindPlan::new(&model, &first_pattern, sought.clone()).unwrap();
    let second_plan = FindPlan::new(&model, &second_pattern, sought).unwrap();

    assert_eq!(first_plan.relation(), second_plan.relation());
    assert_eq!(first_plan.known(), second_plan.known());
    assert_eq!(first_plan.sought(), second_plan.sought());
    assert_eq!(first_plan.mode(), second_plan.mode());
    assert_ne!(first_plan.pattern(), second_plan.pattern());

    let execute = |plan: &FindPlan| {
        model
            .assertions()
            .iter()
            .find(|candidate| {
                candidate.relation() == plan.pattern().relation()
                    && candidate.roles().get(&left_role) == plan.pattern().roles().get(&left_role)
            })
            .and_then(|candidate| candidate.roles().get(&right_role))
            .cloned()
    };
    assert_eq!(execute(&first_plan), Some(Term::entity(first_result)));
    assert_eq!(execute(&second_plan), Some(Term::entity(second_result)));
}

#[test]
fn delta_is_canonical_ground_and_scoped() {
    let text = type_id("Text");
    let relation_id = RelationId::new(name("catalog/text")).unwrap();
    let clause = clause(
        &relation_id,
        Term::value(text.clone(), "left".to_owned()).unwrap(),
        Term::value(text, "right".to_owned()).unwrap(),
    );
    let identity = RevisionId::from_digest([7; 32]);
    assert!(Delta::new(identity, vec![clause.clone()], vec![clause]).is_err());
}
