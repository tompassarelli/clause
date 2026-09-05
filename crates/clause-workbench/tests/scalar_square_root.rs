use clause_package::{Term, decode_canonical_term_bytes};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/scalar-square-root.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().expect("projected object").slots();
        if name.as_atom().unwrap().canonical_payload() == key {
            return value;
        }
        current = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8]) -> f64 {
    let occurrence = w.handler_occurrence(name, &[]).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    let frame = decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap();
    let reading = field(field(&frame, b"meter"), b"reading");
    f64::from_bits(u64::from_le_bytes(reading.as_atom().unwrap().canonical_payload().try_into().unwrap()))
}

#[test]
fn square_root_accepts_zero_and_nonnegative_finite_scalars() {
    for value in [0.0_f64, 2.0, 25.0, f64::MIN_POSITIVE, f64::MAX] {
        let source = SOURCE.replace("reading 25.0", &format!("reading {value:?}"));
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        assert_eq!(run(&mut w, b"measure"), value.sqrt());
    }
}

#[test]
fn square_root_composes_with_law_bindings_and_nested_arithmetic() {
    let source = SOURCE.replace("relation reading", concat!(
        "relation magnitude\n",
        "  reads {input: F64} magnitude {result: F64}\n",
        "  mode given input yields result: maybe\n",
        "law root-magnitude\n",
        "  if\n    ?input >= 0.0\n",
        "  then\n    ?input magnitude sqrt(?input)\n",
        "derive root-magnitude\n\nrelation reading",
    )).replace("    ?meter reading ?value\n  withdraw", "    ?meter reading ?value\n    ?value magnitude ?root\n  withdraw")
      .replace("reading sqrt(?value)", "reading sqrt(?root * ?root + 11.0)");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    assert_eq!(run(&mut w, b"measure"), 6.0);
}

#[test]
fn invalid_square_root_types_are_rejected_and_negative_input_is_atomic() {
    for expression in ["sqrt(true)", "sqrt(\"25\")", "sqrt(?missing)"] {
        let source = SOURCE.replace("sqrt(?value)", expression);
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(), "{expression}");
    }
    let source = SOURCE.replace("reading 25.0", "reading -1.0");
    let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let occurrence = w.handler_occurrence(b"measure", &[]).unwrap();
    let error = w.run_occurrences_to_candidate(&[occurrence]).unwrap_err().to_string();
    assert!(error.contains("NumericDomain"), "{error}");
    assert!(w.recorded_event(b"measure").unwrap().is_none());
    assert_eq!(run(&mut w, b"inspect"), -1.0);
}
