use clause_package::{Term, decode_canonical_term_bytes};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/scalar-comparison.clause");

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

fn measure(w: &mut ResidentSourceWorkbenchV1) -> bool {
    let occurrence = w.handler_occurrence(b"measure", &[]).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    let frame = decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap();
    let meter = field(&frame, b"meter");
    assert_eq!(field(meter, b"reading").as_atom().unwrap().canonical_payload(), 0_f64.to_bits().to_le_bytes());
    match field(meter, b"positive").as_atom().unwrap().canonical_payload() {
        [0] => false,
        [1] => true,
        value => panic!("not Boolean: {value:?}"),
    }
}

#[test]
fn ordered_comparisons_are_boolean_expressions_over_the_same_pre_state() {
    for (expression, first, second) in [
        ("?value > 0.0", true, false),
        ("?value >= 25.0", true, false),
        ("?value <= 0.0", false, true),
        ("?value < 25.0", false, true),
        ("sqrt(?value) * 2.0 + 1.0 > 10.0", true, false),
    ] {
        let source = SOURCE.replace("?value > 0.0", expression);
        let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
        assert_eq!(measure(&mut w), first, "{expression}");
        assert_eq!(measure(&mut w), second, "{expression}");
    }
}

#[test]
fn comparisons_reject_wrong_operands_and_numeric_use_of_a_boolean() {
    for expression in ["true > 0.0", "?value <= false", "(?value > 0.0) + 1.0", "sqrt((?value > 0.0))"] {
        let source = SOURCE.replace("?value > 0.0", expression);
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(), "{expression}");
    }
}
