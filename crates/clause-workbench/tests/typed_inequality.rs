use clause_package::{Term, decode_canonical_term_bytes};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = r#"Text
Workshop
phase
  domain: Workshop
  range: Text
  cardinality: one
workshop
  phase: "Workshop"
on launch ?workshop
  when
    ?workshop phase ?phase
    ?phase != "Expedition"
  withdraw
    ?workshop phase ?phase
  include
    ?workshop phase "Launched"
"#;

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().unwrap().slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

#[test]
fn typed_inequality_is_the_complement_of_equality_in_guards_and_expressions() {
    for expression in ["?phase != \"Expedition\"", "?phase!=\"Expedition\"", "(?phase != \"Expedition\") = true"] {
        for (phase, expected) in [("Workshop", "Launched"), ("Returned", "Launched"), ("Expedition", "Expedition")] {
            let source = SOURCE.replace("phase: \"Workshop\"", &format!("phase: \"{phase}\""))
                .replace("?phase != \"Expedition\"", expression);
            let mut w = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
            let event = w.handler_occurrence(b"launch", &[]).unwrap();
            w.run_occurrences_to_candidate(&[event]).unwrap();
            let frame = decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes()).unwrap();
            assert_eq!(field(field(&frame, b"workshop"), b"phase").as_atom().unwrap().canonical_payload(), expected.as_bytes());
        }
    }
    assert!(ResidentSourceWorkbenchV1::open(SOURCE.replace("!= \"Expedition\"", "!= 1.0").as_bytes()).is_err());
}
