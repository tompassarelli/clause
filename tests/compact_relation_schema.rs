use clause::{
    elaborate::{self, ElaborationContext},
    frontend,
    kernel::{Cardinality, ReferentId, Term},
    wire,
};

const CEREMONIAL: &str = r#"Door
Space

connects: RelationShape
  {door: Door} connects {origin: Space} to {destination: Space}
  mode door, origin -> destination: many

Cellar ∈ Space
Armory ∈ Space
iron-door
  Door
  connects Cellar to Armory
"#;

const COMPACT: &str = r#"Door
Space

connects
  door: Door connects origin: Space to destination: Space
  door origin -> destination*

Cellar ∈ Space
Armory ∈ Space
iron-door
  Door
  connects Cellar to Armory
"#;

const OVERLAPPING: &str = r#"Door
Portal
Space

connects
  door: Door connects origin: Space to destination: Space
  door origin -> destination*

links
  portal: Portal connects source: Space to target: Space
  portal source -> target*

Cellar ∈ Space
Armory ∈ Space
entry ∈ Door
entry ∈ Portal
entry connects Cellar to Armory
"#;

fn model_id() -> ReferentId {
    ReferentId::new(format!("ref-sha256-{}", "7".repeat(64))).expect("fixed Model identity")
}

fn compile_in(source: &str) -> elaborate::CompiledProgram {
    let program = frontend::parse(source).expect("relation schema source parses");
    let declarations = program
        .declarations
        .iter()
        .map(|declaration| {
            format!(
                "{}:{:?}",
                declaration.subject.value.as_str(),
                declaration.kind
            )
        })
        .collect::<Vec<_>>();
    elaborate::compile_in(program, ElaborationContext::new(model_id())).unwrap_or_else(|error| {
        panic!("relation schema source elaborates ({declarations:?}): {error}")
    })
}

#[test]
fn compact_nary_schema_elaborates_to_the_ceremonial_contract() {
    let ceremonial = compile_in(CEREMONIAL);
    let compact = compile_in(COMPACT);
    let ceremonial_revision = ceremonial
        .context_revision()
        .expect("ceremonial context Revision");
    let compact_revision = compact
        .context_revision()
        .expect("compact context Revision");

    let ceremonial_relation = ceremonial
        .designations()
        .global("connects")
        .expect("ceremonial relation identity");
    let compact_relation = compact
        .designations()
        .global("connects")
        .expect("compact relation identity");
    assert_eq!(compact_relation, ceremonial_relation);

    let compact_shape = &compact_revision.model().relation_shapes()[&compact_relation];
    let ceremonial_shape = &ceremonial_revision.model().relation_shapes()[&ceremonial_relation];
    assert_eq!(compact_shape, ceremonial_shape);

    let role = |label| {
        let compact_role = compact
            .designations()
            .role(&compact_relation, label)
            .expect("compact named role identity");
        let ceremonial_role = ceremonial
            .designations()
            .role(&ceremonial_relation, label)
            .expect("ceremonial named role identity");
        assert_eq!(compact_role, ceremonial_role);
        compact_role
    };
    let door = role("door");
    let origin = role("origin");
    let destination = role("destination");

    let [lookup] = compact_shape.lookup() else {
        panic!("compact schema must elaborate one lookup contract");
    };
    assert_eq!(lookup.known().len(), 2);
    assert!(lookup.known().contains(&door));
    assert!(lookup.known().contains(&origin));
    assert_eq!(lookup.sought(), std::slice::from_ref(&destination));
    assert_eq!(lookup.cardinality(), &Cardinality::Many);

    let iron_door = compact
        .designations()
        .scoped(&model_id(), "iron-door")
        .expect("focused referent identity");
    let content = compact_revision
        .model()
        .admitted_contents()
        .iter()
        .find(|content| content.relation() == &compact_relation)
        .expect("focused n-ary content");
    assert_eq!(content.roles()[&door], Term::referent(iron_door));

    assert_eq!(compact_revision.identity(), ceremonial_revision.identity());
    assert_eq!(
        wire::serialize(compact_revision),
        wire::serialize(ceremonial_revision)
    );
}

#[test]
fn overlapping_schemas_name_candidates_and_conflicting_roles() {
    let diagnostic = match frontend::parse(OVERLAPPING) {
        Err(error) => error.to_string(),
        Ok(program) => elaborate::compile_in(program, ElaborationContext::new(model_id()))
            .expect_err("overlapping schemas must be rejected")
            .to_string(),
    };

    for required in [
        "connects",
        "links",
        "door",
        "portal",
        "origin",
        "source",
        "destination",
        "target",
    ] {
        assert!(
            diagnostic.contains(required),
            "ambiguity diagnostic must name '{required}': {diagnostic}"
        );
    }
}

#[test]
fn checked_focus_rejects_an_undeclared_role() {
    let mut program = frontend::parse(COMPACT).expect("compact relation schema parses");
    let relation = program
        .declarations
        .iter_mut()
        .find(|declaration| declaration.subject.value.as_str() == "connects")
        .expect("connects relation exists");
    let frontend::Member::Sentence(sentence) = &mut relation.body[0] else {
        panic!("compact relation starts with a sentence shape");
    };
    sentence.focus.value = frontend::RoleName("owner".to_owned());

    let error = elaborate::compile_in(program, ElaborationContext::new(model_id()))
        .expect_err("focus must resolve through the checked role designations")
        .to_string();
    assert!(error.contains("unknown role 'owner'"), "{error}");
}

#[test]
fn compact_optional_projection_lowers_without_reserving_maybe() {
    let compiled = compile_in(
        "Item\nState\n\nstatus\n  subject: Item has maybe: State\n  subject -> maybe 0..1\n",
    );
    let relation = compiled
        .designations()
        .global("status")
        .expect("status relation identity");
    let revision = compiled.context_revision().expect("context Revision");
    let shape = &revision.model().relation_shapes()[&relation];
    let maybe = compiled
        .designations()
        .role(&relation, "maybe")
        .expect("maybe remains an ordinary role name");
    let [lookup] = shape.lookup() else {
        panic!("status has one projection");
    };
    assert_eq!(lookup.sought(), std::slice::from_ref(&maybe));
    assert_eq!(lookup.cardinality(), &Cardinality::Maybe);
}
