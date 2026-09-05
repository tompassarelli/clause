use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableValueV1, projected_relation_table_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/typed-readiness.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().expect("projected field").slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8]) -> Term {
    let occurrence = w.handler_occurrence(name, &[]).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
}

fn message(frame: &Term) -> &str {
    std::str::from_utf8(field(field(frame, b"device"), b"message").as_atom().unwrap().canonical_payload()).unwrap()
}

fn charge(frame: &Term) -> f64 {
    f64::from_le_bytes(field(field(frame, b"device"), b"charge").as_atom().unwrap().canonical_payload().try_into().unwrap())
}

#[test]
fn one_typed_rule_drives_feedback_and_action_eligibility() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    assert_eq!(message(&run(&mut w, b"inspect")), "Ready");
    assert_eq!(charge(&run(&mut w, b"use")), 0.0);
    assert_eq!(message(&run(&mut w, b"inspect")), "Empty");
    assert_eq!(charge(&run(&mut w, b"use")), 0.0);
    let source = SOURCE.replace("device enabled true", "device enabled false");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    assert_eq!(message(&run(&mut w, b"inspect")), "Disabled");
    assert_eq!(charge(&run(&mut w, b"use")), 1.0);
}

#[test]
fn text_results_feed_another_typed_law() {
    let source = SOURCE
        .replace("relation enabled", "relation label\n  reads label {message: Text} as {result: Text}\n  mode given message yields result: maybe\n\nlaw label\n  then\n    label ?message as \"State: \" ++ ?message\nderive label\n\nrelation enabled")
        .replace("?device message ?message\n", "?device message ?label\n")
        .replacen("  withdraw\n", "    label ?message as ?label\n  withdraw\n", 1);
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    assert_eq!(message(&run(&mut w, b"inspect")), "State: Ready");
    run(&mut w, b"use");
    assert_eq!(message(&run(&mut w, b"inspect")), "State: Empty");
}

#[test]
fn typed_laws_use_runtime_created_rows_too() {
    let source = format!("{SOURCE}\n{}", r#"on spawn ?device
  when
    ?device charge ?charge
  create
    ?new
      shape: Device
  include
    ?new enabled false
    ?new charge 2.0
    ?new message "Unchecked"
"#);
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    run(&mut w, b"spawn");
    let frame = run(&mut w, b"inspect");
    let messages = projected_relation_table_v1(field(field(&frame, b"relations"), b"message")).unwrap().unwrap();
    assert_eq!(messages.rows().len(), 2);
    assert!(messages.rows().values().flatten().any(|value| *value == ExecutableValueV1::text("Ready").unwrap()));
    assert!(messages.rows().values().flatten().any(|value| *value == ExecutableValueV1::text("Disabled").unwrap()));
}

#[test]
fn literal_patterns_and_quoted_text_are_preserved() {
    let source = SOURCE
        .replace("    ?enabled = false\n", "")
        .replace("?enabled enabled with charge ?charge reports \"Disabled\"", "false enabled with charge ?charge reports \"Not ready (yet)\"")
        .replace("device enabled true", "device enabled false");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    assert_eq!(message(&run(&mut w, b"inspect")), "Not ready (yet)");
}

#[test]
fn declared_types_survive_law_specialization() {
    for source in [
        SOURCE.replace("reports \"Disabled\"", "reports false"),
        SOURCE.replace("?enabled = false", "?enabled > 0.0"),
        SOURCE.replace("device enabled true", "device enabled false").replace("?enabled enabled with charge ?charge reports ?message", "1.0 enabled with charge ?charge reports ?message"),
        SOURCE.replace("?enabled enabled with charge ?charge reports ?message", "?charge enabled with charge ?charge reports ?message"),
        SOURCE.replace("?enabled enabled with charge ?charge reports ?message", "?enabled enabled with charge \"full\" reports ?message"),
        SOURCE.replace("?charge <= 0.0", "?charge > 0.0"),
    ] {
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err());
    }
}

#[test]
fn readiness_accepts_equality_of_typed_state_referents() {
    let source = SOURCE
        .replace("relation readiness", "Mode\n\nrelation mode\n  reads {device: Device} mode {value: Mode}\n  subject device\n  mode given device yields value: one\n\nrunning\n  shape: Mode\n\nrelation readiness")
        .replace("device enabled true", "device enabled true\ndevice mode running")
        .replace("    ?device enabled ?enabled\n", "    ?device mode ?mode\n")
        .replace("?enabled enabled with charge ?charge reports ?message", "(?mode = running) enabled with charge ?charge reports ?message");
    for expression in ["?mode = running", "running = ?mode", "(?mode = running) = true"] {
        let source = source.replace("?mode = running", expression);
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        assert_eq!(message(&run(&mut w, b"inspect")), "Ready", "{expression}");
        assert_eq!(charge(&run(&mut w, b"use")), 0.0);
        assert_eq!(message(&run(&mut w, b"inspect")), "Empty");
        let source = source.replace("device mode running", "device mode stopped\nstopped\n  shape: Mode");
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        assert_eq!(message(&run(&mut w, b"inspect")), "Disabled", "{expression}");
        assert_eq!(charge(&run(&mut w, b"use")), 1.0);
    }
    let source = format!("{source}\n{}", r#"stopped
  shape: Mode
on spawn ?device
  when
    ?device charge ?charge
  create
    ?new
      shape: Device
  include
    ?new mode stopped
    ?new enabled true
    ?new charge 2.0
    ?new message "Unchecked"
"#);
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    run(&mut w, b"spawn");
    let frame = run(&mut w, b"inspect");
    let messages = projected_relation_table_v1(field(field(&frame, b"relations"), b"message")).unwrap().unwrap();
    assert_eq!(messages.rows().len(), 2);
    assert!(messages.rows().values().flatten().any(|value| *value == ExecutableValueV1::text("Ready").unwrap()));
    assert!(messages.rows().values().flatten().any(|value| *value == ExecutableValueV1::text("Disabled").unwrap()));
}
