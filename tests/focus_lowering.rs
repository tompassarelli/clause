//! Generic authoring ellipsis is fully lowered before a Revision is sealed.

use clause::{elaborate, frontend, wire};

const ELLIPSIS: &str = r#"Item
Sensor

pairing/pair: RelationShape
  {item: Item} paired with {sensor: Sensor}
  mode item -> sensor: many

pairing
  [Item 1..4] ∈ Item
  [Sensor 1..4] ∈ Sensor
  [Item {n}]
    paired with [Sensor {n}]
  for n: 1..4
"#;

const EXPLICIT: &str = r#"Item
Sensor

pairing/pair: RelationShape
  {item: Item} paired with {sensor: Sensor}
  mode item -> sensor: many

pairing
  Item 1 ∈ Item
  Item 2 ∈ Item
  Item 3 ∈ Item
  Item 4 ∈ Item
  Sensor 1 ∈ Sensor
  Sensor 2 ∈ Sensor
  Sensor 3 ∈ Sensor
  Sensor 4 ∈ Sensor
  Item 1 paired with Sensor 1
  Item 2 paired with Sensor 2
  Item 3 paired with Sensor 3
  Item 4 paired with Sensor 4
"#;

fn program(source: &str) -> elaborate::CompiledProgram {
    elaborate::compile(frontend::parse(source).expect("source parses")).expect("source lowers")
}

fn revision(program: &elaborate::CompiledProgram) -> clause::kernel::Revision {
    program
        .revision(&frontend::Name("pairing".to_owned()))
        .expect("base Revision")
        .clone()
}

#[test]
fn finite_groups_and_correlated_focus_lower_to_the_same_sealed_revision() {
    let ellipsis_program = program(ELLIPSIS);
    let explicit_program = program(EXPLICIT);
    let ellipsis = revision(&ellipsis_program);
    let explicit = revision(&explicit_program);

    assert_eq!(ellipsis.identity(), explicit.identity());
    assert_eq!(wire::serialize(&ellipsis), wire::serialize(&explicit));
    assert_eq!(
        wire::reload(&wire::serialize(&ellipsis)).expect("canonical wire reloads"),
        ellipsis
    );
    assert_eq!(ellipsis.model().admitted_contents().len(), 12);
    let model = ellipsis_program
        .designations()
        .global("pairing")
        .expect("pairing designation resolves");
    for local in [
        "Item 1", "Item 2", "Item 3", "Item 4", "Sensor 1", "Sensor 2", "Sensor 3", "Sensor 4",
    ] {
        let referent = ellipsis_program
            .designations()
            .scoped(&model, local)
            .expect("focused referent designation resolves");
        assert!(ellipsis.model().referents().contains_key(&referent));
    }
}

#[test]
fn concrete_multiword_referents_reject_retired_brackets() {
    let bracketed = EXPLICIT.replace(
        "Item 1 paired with Sensor 1",
        "[Item 1] paired with Sensor 1",
    );
    assert!(
        frontend::parse(&bracketed)
            .unwrap_err()
            .to_string()
            .contains("bracketed concrete referents are retired")
    );
}

#[test]
fn focused_slots_report_sorted_ambiguity_and_checked_template_errors() {
    let ambiguous = ELLIPSIS
        .replace("pairing/pair: RelationShape", "a/pair: RelationShape")
        .replace(
            "pairing",
            "b/pair: RelationShape\n  {item: Item} paired with {sensor: Sensor}\n  mode item -> sensor: many\n\npairing",
        );
    assert!(
        elaborate::compile(frontend::parse(&ambiguous).unwrap())
            .unwrap_err()
            .to_string()
            .contains("a/pair, b/pair")
    );

    let unbound = ELLIPSIS.replace("[Sensor {n}]", "[Sensor {m}]");
    assert!(
        elaborate::compile(frontend::parse(&unbound).unwrap())
            .unwrap_err()
            .to_string()
            .contains("unbound focus variable 'm'")
    );

    let wrong_type = ELLIPSIS.replace("[Item {n}]", "[Sensor {n}]");
    let wrong_type_error = elaborate::compile(frontend::parse(&wrong_type).unwrap())
        .unwrap_err()
        .to_string();
    assert!(
        wrong_type_error.contains("focused referent 'Sensor 1' is not a member of 'Item'"),
        "{wrong_type_error}"
    );

    assert!(frontend::parse(&ELLIPSIS.replace("1..4", "4..1")).is_err());
}
