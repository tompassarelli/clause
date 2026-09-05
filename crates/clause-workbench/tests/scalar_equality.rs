use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableValueV1, projected_relation_table_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/scalar-equality.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().expect("projected field").slots();
        if name.as_atom().unwrap().canonical_payload() == key {
            return value;
        }
        current = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8]) -> Term {
    let occurrence = w.handler_occurrence(name, &[]).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
}

#[test]
fn boolean_equality_toggles_from_each_steps_actual_prestate() {
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    for expected in [1, 0, 1] {
        let frame = run(&mut w, b"toggle");
        assert_eq!(
            field(field(&frame, b"first"), b"selected")
                .as_atom()
                .unwrap()
                .canonical_payload(),
            [expected]
        );
    }
}

#[test]
fn equality_composes_with_numbers_text_and_comparisons() {
    let source = include_str!("../../../test-vectors/authoring/scalar-comparison.clause");
    for (expression, expected) in [
        ("?value = 25.0", true),
        ("?value = 0.0", false),
        ("(?value > 0.0) = true", true),
        ("\"a\" ++ \"b\" = \"ab\"", true),
        ("\"a\" = \"b\"", false),
    ] {
        let source = source.replace("?value > 0.0", expression);
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        let frame = run(&mut w, b"measure");
        assert_eq!(
            field(field(&frame, b"meter"), b"positive")
                .as_atom()
                .unwrap()
                .canonical_payload(),
            [u8::from(expected)],
            "{expression}"
        );
    }
}

#[test]
fn equality_toggles_runtime_created_rows() {
    let source = format!(
        "{SOURCE}\n{}",
        r#"on spawn ?item
  when
    ?item selected ?prior
  create
    ?new
      shape: Item
  include
    ?new selected true
"#
    );
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    run(&mut w, b"toggle");
    run(&mut w, b"spawn");
    let frame = run(&mut w, b"toggle");
    let selected = projected_relation_table_v1(field(field(&frame, b"relations"), b"selected"))
        .unwrap()
        .unwrap();
    assert_eq!(selected.rows().len(), 2);
    assert!(
        selected
            .rows()
            .values()
            .flatten()
            .all(|value| *value == ExecutableValueV1::Boolean(false))
    );
}

#[test]
fn equality_rejects_mixed_types_and_numeric_use() {
    for expression in [
        "?prior = 0.0",
        "true = 1.0",
        "false = \"false\"",
        "(?prior = false) + 1.0",
    ] {
        let source = SOURCE.replace("?prior = false", expression);
        assert!(
            ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(),
            "{expression}"
        );
    }
    let source = include_str!("../../../test-vectors/authoring/scalar-comparison.clause");
    for expression in ["?value = false", "true = 1.0", "(?value > 0.0) = 0.0"] {
        let source = source.replace("?value > 0.0", expression);
        assert!(
            ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(),
            "{expression}"
        );
    }
}
