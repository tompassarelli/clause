//! Generic authoring ellipsis is fully lowered before a Revision is sealed.

use clause::{elaborate, frontend, wire};

const ELLIPSIS: &str = r#"Item: Type
Sensor: Type

pairing/pair: Relation
    {item: Item} paired with {sensor: Sensor}
    mode item -> sensor: many

pairing: Model
    [Item 1..4]: Item
    [Sensor 1..4]: Sensor
    [Item {n}]:
        paired with: [Sensor {n}]
    for n: 1..4
"#;

const EXPLICIT: &str = r#"Item: Type
Sensor: Type

pairing/pair: Relation
    {item: Item} paired with {sensor: Sensor}
    mode item -> sensor: many

pairing: Model
    [Item 1]: Item
    [Item 2]: Item
    [Item 3]: Item
    [Item 4]: Item
    [Sensor 1]: Sensor
    [Sensor 2]: Sensor
    [Sensor 3]: Sensor
    [Sensor 4]: Sensor
    [Item 1] paired with [Sensor 1]
    [Item 2] paired with [Sensor 2]
    [Item 3] paired with [Sensor 3]
    [Item 4] paired with [Sensor 4]
"#;

fn revision(source: &str) -> clause::kernel::Revision {
    elaborate::compile(frontend::parse(source).expect("source parses"))
        .expect("source lowers")
        .revision(&frontend::Name("pairing".to_owned()))
        .expect("base Revision")
        .clone()
}

#[test]
fn finite_groups_and_correlated_focus_lower_to_the_same_sealed_revision() {
    let ellipsis = revision(ELLIPSIS);
    let explicit = revision(EXPLICIT);

    assert_eq!(ellipsis.identity(), explicit.identity());
    assert_eq!(wire::serialize(&ellipsis), wire::serialize(&explicit));
    assert_eq!(
        wire::reload(&wire::serialize(&ellipsis)).expect("canonical wire reloads"),
        ellipsis
    );
    assert_eq!(ellipsis.model().entities().len(), 8);
    assert_eq!(ellipsis.model().assertions().len(), 4);
    assert!(
        ellipsis
            .model()
            .entities()
            .iter()
            .any(|entity| entity.local().as_str() == "Item 1")
    );
}

#[test]
fn focused_slots_report_sorted_ambiguity_and_checked_template_errors() {
    let ambiguous = ELLIPSIS
        .replace("pairing/pair: Relation", "a/pair: Relation")
        .replace(
            "pairing: Model",
            "b/pair: Relation\n    {item: Item} paired with {sensor: Sensor}\n    mode item -> sensor: many\n\npairing: Model",
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
    assert!(
        elaborate::compile(frontend::parse(&wrong_type).unwrap())
            .unwrap_err()
            .to_string()
            .contains("has Type 'Sensor', not 'Item'")
    );

    assert!(frontend::parse(&ELLIPSIS.replace("1..4", "4..1")).is_err());
}
