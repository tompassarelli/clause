use clause::{
    derive::{self, Limits},
    elaborate, frontend,
    kernel::{ContentId, ReferentId, RelationalContent, Revision, RoleId, Term},
    wire,
};

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

collision/inclusive: DerivationRule
  distance between player and coin <= radius of player + radius of coin
  when:
    distance between player and coin < radius of player + radius of coin
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

fn application(term: &Term) -> &ContentId {
    let Term::Application(content) = term else {
        panic!("recursive participant must lower to Term::Application: {term:?}");
    };
    content
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
