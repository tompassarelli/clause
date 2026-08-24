use clause::{
    derive::{self, Limits},
    elaborate, frontend,
    kernel::{Cardinality, ContentId, ReferentId, RelationalContent, Revision, RoleId, Term},
    wire,
};
use std::collections::BTreeSet;

const SOURCE: &str = r#"Body
Scalar

distance
  result: Scalar distance between left: Body and right: Body
  left right -> result

radius
  result: Scalar radius of subject: Body
  subject -> result

+
  result: Scalar is left: Scalar + right: Scalar
  left right -> result

<
  left: Scalar < right: Scalar
  left -> right*

<=
  left: Scalar <= right: Scalar
  left -> right*

scene
  player ∈ Body
  coin ∈ Body
  distance between player and coin < radius of player + radius of coin

scene/collision-inclusive: DerivationRule
  distance between player and coin <= radius of player + radius of coin
  when:
    distance between player and coin < radius of player + radius of coin
"#;

const MULTI_MODE_SOURCE: &str = r#"Scalar

arithmetic/add: RelationShape
  {result: Scalar} is {left: Scalar} + {right: Scalar}
  mode left, right -> result: one
  mode result, right -> left: one

comparison/less: RelationShape
  {left: Scalar} < {right: Scalar}
  mode left -> right: many

numbers
  a ∈ Scalar
  b ∈ Scalar
  c ∈ Scalar
  a < b + c
"#;

const GROUPED_SOURCE: &str = r#"Scalar

+
  result: Scalar is left: Scalar + right: Scalar
  left right -> result

*
  result: Scalar is left: Scalar * right: Scalar
  left right -> result

<
  left: Scalar < right: Scalar
  left -> right*

math
  a ∈ Scalar
  b ∈ Scalar
  c ∈ Scalar
  a + b * c < (a + b) * c

association
  a ∈ Scalar
  b ∈ Scalar
  c ∈ Scalar
  a + b + c < c
"#;

const NESTED_AMBIGUITY_SOURCE: &str = r#"Quantity

first-combination
  result: Quantity is left: Quantity with right: Quantity
  left right -> result

second-combination
  result: Quantity is left: Quantity with right: Quantity
  left right -> result

<
  left: Quantity < right: Quantity
  left -> right*

ambiguous
  a ∈ Quantity
  b ∈ Quantity
  c ∈ Quantity
  a with b < c
"#;

fn compile(source: &str) -> elaborate::CompiledProgram {
    elaborate::compile(frontend::parse(source).expect("recursive term source parses"))
        .expect("recursive term source elaborates")
}

fn relation(program: &elaborate::CompiledProgram, name: &str) -> ReferentId {
    program
        .designations()
        .global(name)
        .unwrap_or_else(|error| panic!("relation '{name}' resolves: {error}"))
}

fn role(program: &elaborate::CompiledProgram, relation: &ReferentId, name: &str) -> RoleId {
    program
        .designations()
        .role(relation, name)
        .unwrap_or_else(|error| panic!("role '{name}' resolves: {error}"))
}

fn scoped(program: &elaborate::CompiledProgram, revision: &Revision, name: &str) -> ReferentId {
    program
        .designations()
        .scoped(revision.model().id(), name)
        .unwrap_or_else(|error| panic!("scoped referent '{name}' resolves: {error}"))
}

fn application(term: &Term) -> &ContentId {
    let Term::Application(content) = term else {
        panic!("recursive participant must lower to Term::Application: {term:?}");
    };
    content
}

fn referent_term(term: &Term) -> &ReferentId {
    let Term::Referent(referent) = term else {
        panic!("recursive leaf must be a referent: {term:?}");
    };
    referent
}

fn content_for_relation<'a>(
    contents: impl IntoIterator<Item = &'a RelationalContent>,
    relation: &ReferentId,
) -> &'a RelationalContent {
    contents
        .into_iter()
        .find(|content| content.relation() == relation)
        .unwrap_or_else(|| panic!("content for relation '{}' exists", relation.as_str()))
}

fn asserted_comparison<'a>(revision: &'a Revision, relation: &ReferentId) -> &'a RelationalContent {
    content_for_relation(revision.model().admitted_contents(), relation)
}

fn assert_application_contract(revision: &Revision, content: &RelationalContent) {
    let shape = &revision.model().relation_shapes()[content.relation()];
    let supplied = content.roles().keys().cloned().collect::<BTreeSet<_>>();
    let matching = shape
        .lookup()
        .iter()
        .filter(|mode| mode.known().iter().cloned().collect::<BTreeSet<_>>() == supplied)
        .collect::<Vec<_>>();
    let [mode] = matching.as_slice() else {
        panic!("application known roles must select one exact mode");
    };
    assert_eq!(mode.cardinality(), &Cardinality::One);
    let [result] = mode.sought() else {
        panic!("application mode must have one result role");
    };
    assert!(!content.roles().contains_key(result));
}

fn assert_recursive_tree(program: &elaborate::CompiledProgram, revision: &Revision) {
    let less_than = relation(program, "<");
    let distance = relation(program, "distance");
    let radius = relation(program, "radius");
    let addition = relation(program, "+");

    let root = asserted_comparison(revision, &less_than);
    let root_left = role(program, &less_than, "left");
    let root_right = role(program, &less_than, "right");
    let distance_id = application(&root.roles()[&root_left]);
    let addition_id = application(&root.roles()[&root_right]);
    let distance_content = revision
        .model()
        .content(distance_id)
        .expect("distance application is registered before its parent");
    let addition_content = revision
        .model()
        .content(addition_id)
        .expect("addition application is registered before its parent");
    assert_eq!(distance_content.relation(), &distance);
    assert_eq!(addition_content.relation(), &addition);
    assert_application_contract(revision, distance_content);
    assert_application_contract(revision, addition_content);

    let addition_left = role(program, &addition, "left");
    let addition_right = role(program, &addition, "right");
    for nested in [
        &addition_content.roles()[&addition_left],
        &addition_content.roles()[&addition_right],
    ] {
        let radius_content = revision
            .model()
            .content(application(nested))
            .expect("radius application is registered before addition");
        assert_eq!(radius_content.relation(), &radius);
        assert_application_contract(revision, radius_content);
    }

    assert!(
        !revision
            .model()
            .admitted_contents()
            .iter()
            .any(|content| content.id() == distance_id || content.id() == addition_id),
        "nested applications are terms, not independently asserted occurrences"
    );
}

fn assert_grouped_precedence_tree(program: &elaborate::CompiledProgram, revision: &Revision) {
    let less = relation(program, "<");
    let addition = relation(program, "+");
    let multiplication = relation(program, "*");
    let a = scoped(program, revision, "a");
    let b = scoped(program, revision, "b");
    let c = scoped(program, revision, "c");
    let root = asserted_comparison(revision, &less);
    let root_left = role(program, &less, "left");
    let root_right = role(program, &less, "right");
    let addition_left = role(program, &addition, "left");
    let addition_right = role(program, &addition, "right");
    let multiplication_left = role(program, &multiplication, "left");
    let multiplication_right = role(program, &multiplication, "right");

    let left_addition = revision
        .model()
        .content(application(&root.roles()[&root_left]))
        .expect("left addition is registered");
    assert_eq!(left_addition.relation(), &addition);
    assert_eq!(referent_term(&left_addition.roles()[&addition_left]), &a);
    let left_multiplication = revision
        .model()
        .content(application(&left_addition.roles()[&addition_right]))
        .expect("higher-precedence multiplication is registered");
    assert_eq!(left_multiplication.relation(), &multiplication);
    assert_eq!(
        referent_term(&left_multiplication.roles()[&multiplication_left]),
        &b
    );
    assert_eq!(
        referent_term(&left_multiplication.roles()[&multiplication_right]),
        &c
    );

    let right_multiplication = revision
        .model()
        .content(application(&root.roles()[&root_right]))
        .expect("right multiplication is registered");
    assert_eq!(right_multiplication.relation(), &multiplication);
    let grouped_addition = revision
        .model()
        .content(application(
            &right_multiplication.roles()[&multiplication_left],
        ))
        .expect("parenthesized addition is registered");
    assert_eq!(grouped_addition.relation(), &addition);
    assert_eq!(referent_term(&grouped_addition.roles()[&addition_left]), &a);
    assert_eq!(
        referent_term(&grouped_addition.roles()[&addition_right]),
        &b
    );
    assert_eq!(
        referent_term(&right_multiplication.roles()[&multiplication_right]),
        &c
    );

    let application_contents = revision
        .model()
        .relational_contents()
        .values()
        .filter(|content| content.relation() == &addition || content.relation() == &multiplication)
        .collect::<Vec<_>>();
    assert_eq!(application_contents.len(), 4);
    assert!(application_contents.iter().all(|application| {
        !revision
            .model()
            .admitted_contents()
            .iter()
            .any(|admitted| admitted.id() == application.id())
    }));
}

#[test]
fn ground_recursive_operator_tree_round_trips_and_drives_rule() {
    let program = compile(SOURCE);
    let revision = program
        .revision(&frontend::Name("scene".to_owned()))
        .expect("scene Revision resolves");
    assert_recursive_tree(&program, revision);

    let reloaded = wire::reload(&wire::serialize(revision)).expect("recursive wire reloads");
    assert_eq!(&reloaded, revision);
    assert_recursive_tree(&program, &reloaded);

    let less_or_equal = relation(&program, "<=");
    let closure = derive::saturate(revision, Limits::new(16, 4, 64))
        .expect("ground recursive rule saturates");
    let derived = content_for_relation(closure.contents(), &less_or_equal);
    let derived_left = role(&program, &less_or_equal, "left");
    let derived_right = role(&program, &less_or_equal, "right");
    let asserted = asserted_comparison(revision, &relation(&program, "<"));
    let asserted_left = role(&program, asserted.relation(), "left");
    let asserted_right = role(&program, asserted.relation(), "right");
    assert_eq!(
        derived.roles()[&derived_left],
        asserted.roles()[&asserted_left]
    );
    assert_eq!(
        derived.roles()[&derived_right],
        asserted.roles()[&asserted_right]
    );
}

#[test]
fn declared_symbols_do_not_accept_word_substitutions() {
    for source in [
        SOURCE.replace(
            "distance between player and coin < radius of player + radius of coin",
            "distance between player and coin less-than radius of player + radius of coin",
        ),
        SOURCE.replace(
            "radius of player + radius of coin",
            "radius of player plus radius of coin",
        ),
    ] {
        assert!(
            frontend::parse(&source).is_err(),
            "declared ASCII operators must remain exact"
        );
    }
}

#[test]
fn recursive_projection_selects_its_result_mode_from_a_bidirectional_relation() {
    let program = compile(MULTI_MODE_SOURCE);
    let revision = program
        .revision(&frontend::Name("numbers".to_owned()))
        .expect("numbers Revision resolves");
    let less = relation(&program, "comparison/less");
    let addition = relation(&program, "arithmetic/add");
    let root = asserted_comparison(revision, &less);
    let right = role(&program, &less, "right");
    let nested = revision
        .model()
        .content(application(&root.roles()[&right]))
        .expect("addition application is registered");

    assert_eq!(nested.relation(), &addition);
    assert_eq!(
        revision.model().relation_shapes()[&addition].lookup().len(),
        2
    );
    assert_application_contract(revision, nested);
}

#[test]
fn parentheses_override_conventional_operator_precedence() {
    let program = compile(GROUPED_SOURCE);
    let revision = program
        .revision(&frontend::Name("math".to_owned()))
        .expect("math Revision resolves");
    assert_grouped_precedence_tree(&program, revision);

    let reloaded = wire::reload(&wire::serialize(revision)).expect("grouped tree reloads");
    assert_eq!(&reloaded, revision);
    assert_grouped_precedence_tree(&program, &reloaded);
}

#[test]
fn same_tier_ascii_operators_associate_left() {
    let program = compile(GROUPED_SOURCE);
    let revision = program
        .revision(&frontend::Name("association".to_owned()))
        .expect("association Revision resolves");
    let less = relation(&program, "<");
    let addition = relation(&program, "+");
    let a = scoped(&program, revision, "a");
    let b = scoped(&program, revision, "b");
    let c = scoped(&program, revision, "c");
    let root = asserted_comparison(revision, &less);
    let root_left = role(&program, &less, "left");
    let outer = revision
        .model()
        .content(application(&root.roles()[&root_left]))
        .expect("outer addition is registered");
    let left = role(&program, &addition, "left");
    let right = role(&program, &addition, "right");
    assert_eq!(outer.relation(), &addition);
    assert_eq!(referent_term(&outer.roles()[&right]), &c);
    let inner = revision
        .model()
        .content(application(&outer.roles()[&left]))
        .expect("left-associated addition is registered");
    assert_eq!(inner.relation(), &addition);
    assert_eq!(referent_term(&inner.roles()[&left]), &a);
    assert_eq!(referent_term(&inner.roles()[&right]), &b);
}

#[test]
fn malformed_parentheses_are_rejected_at_the_grouping_boundary() {
    for (expression, expected) in [
        ("a + b * c < (a + b * c", "unterminated parenthesized term"),
        ("a + b * c < a + b) * c", "unmatched closing parenthesis"),
    ] {
        let source = GROUPED_SOURCE.replace("a + b * c < (a + b) * c", expression);
        let error = frontend::parse(&source).expect_err("malformed grouping must fail");
        assert_eq!(error.message, expected);
    }
}

#[test]
fn focused_and_flattened_recursive_clauses_have_identical_canonical_identity() {
    let focused_source = GROUPED_SOURCE.replace(
        "  a + b * c < (a + b) * c",
        "  a\n    + b * c < (a + b) * c",
    );
    let focused_program = compile(&focused_source);
    let flattened_program = compile(GROUPED_SOURCE);
    let focused = focused_program
        .revision(&frontend::Name("math".to_owned()))
        .expect("focused math Revision resolves");
    let flattened = flattened_program
        .revision(&frontend::Name("math".to_owned()))
        .expect("flattened math Revision resolves");

    assert_eq!(focused.identity(), flattened.identity());
    assert_eq!(wire::serialize(focused), wire::serialize(flattened));
    assert_grouped_precedence_tree(&focused_program, focused);
}

#[test]
fn nested_ambiguity_names_each_surviving_application_path() {
    let error = frontend::parse(NESTED_AMBIGUITY_SOURCE)
        .expect_err("overlapping nested applications remain ambiguous");
    assert_eq!(
        error.message,
        "ambiguous clause; conflicting candidate paths: <.left -> first-combination [left, right -> result]; <.left -> second-combination [left, right -> result]"
    );
}
