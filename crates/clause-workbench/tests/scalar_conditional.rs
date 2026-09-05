use clause_package::{Term, decode_canonical_term_bytes};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/scalar-conditional.clause");

#[test]
fn conditional_fields_keep_nested_commas_and_text_inside_each_value() {
    let source = include_str!("../../../test-vectors/authoring/structured-conditional.clause");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let frame = run(&mut w, b"measure");
    let value = field(field(&frame, b"item"), b"reading");
    assert_eq!(field(value, b"amount").as_atom().unwrap().canonical_payload(), 5.0_f64.to_bits().to_le_bytes());
    assert_eq!(field(value, b"enabled").as_atom().unwrap().canonical_payload(), [0]);
    assert_eq!(field(value, b"label").as_atom().unwrap().canonical_payload(), b"c, d");

    let source = include_str!("../../../test-vectors/authoring/structured-keyboard-transition.clause")
        .replace("?velocity-x + 3.0", "if(?velocity-x = 0.0, 3.0, 0.0)");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let frame = run(&mut w, b"planar-burst");
    assert_eq!(field(field(field(&frame, b"player-1"), b"velocity"), b"x").as_atom().unwrap().canonical_payload(), 3.0_f64.to_bits().to_le_bytes());
}

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8]) -> Term {
    let occurrence = w.handler_occurrence(name, &[]).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
}

fn reading(frame: &Term) -> &[u8] {
    field(field(frame, b"meter"), b"reading").as_atom().unwrap().canonical_payload()
}

#[test]
fn conditional_evaluates_only_the_selected_branch_and_preserves_failed_steps() {
    for (initial, expected) in [(0.0, 0.0_f64), (2.0, 5.0)] {
        let source = SOURCE.replace("reading 0.0", &format!("reading {initial:?}"));
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        assert_eq!(reading(&run(&mut w, b"measure")), expected.to_bits().to_le_bytes());
    }
    let source = SOURCE.replace("10.0 / ?value, 0.0", "10.0 / ?value, sqrt(-1.0)");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let occurrence = w.handler_occurrence(b"measure", &[]).unwrap();
    let error = w.run_occurrences_to_candidate(&[occurrence]).unwrap_err().to_string();
    assert!(error.contains("NumericDomain"), "{error}");
    assert!(w.recorded_event(b"measure").unwrap().is_none());
    assert_eq!(reading(&run(&mut w, b"inspect")), 0.0_f64.to_bits().to_le_bytes());
}

#[test]
fn conditional_checks_both_branches_and_requires_boolean_condition() {
    for expression in [
        "if(1.0, 2.0, 3.0)", "if(true, 2.0, false)",
        "if(false, \"wrong\", 3.0)", "if(true, 2.0, ?missing)",
    ] {
        let source = SOURCE.replace("if(?value > 0.0, 10.0 / ?value, 0.0)", expression);
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(), "{expression}");
    }
    for (domain, initial, expression, expected) in [
        ("Bool", "false", "if(?value = false, true, false)", &[1][..]),
        ("Text", "\"left\"", "if(?value = \"left\", \"right\", \"left\")", b"right"),
    ] {
        let source = SOURCE.replace("F64", domain).replace("reading 0.0", &format!("reading {initial}"))
            .replace("if(?value > 0.0, 10.0 / ?value, 0.0)", expression);
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        assert_eq!(reading(&run(&mut w, b"measure")), expected);
    }
}

#[test]
fn conditional_composes_with_laws_queries_guards_and_created_rows() {
    let source = include_str!("../../../test-vectors/authoring/query-law-inputs.clause")
        .replace("?value * 2.0", "if(?value > 0.0, ?value * 2.0, 0.0)")
        .replace("sum ?value * ?weight", "sum if(?value > 0.0, ?value * ?weight, 0.0)")
        .replace("?item amount ?value }", "?item amount ?value; if(?value > 0.0, true, false) = true }");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let frame = run(&mut w, b"measure");
    assert_eq!(field(field(&frame, b"report"), b"total").as_atom().unwrap().canonical_payload(), 12.0_f64.to_bits().to_le_bytes());

    let source = format!("{source}\non spawn ?report\n  when\n    ?report total ?prior\n  create\n    ?new\n      shape: Item\n  include\n    ?new amount if(?prior > 0.0, 3.0, 0.0)\n");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    run(&mut w, b"spawn");
    let frame = run(&mut w, b"measure");
    assert_eq!(field(field(&frame, b"report"), b"total").as_atom().unwrap().canonical_payload(), 24.0_f64.to_bits().to_le_bytes());
}
