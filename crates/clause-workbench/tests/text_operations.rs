use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableValueV1, projected_text_value_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/text-operations.clause");

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

#[test]
fn text_operations_tokenize_unicode_without_losing_payload_whitespace() {
    let mut workbench = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    for (input, clean, first, rest, prefix) in [
        ("  /goal\tedit  ship it \n", "/goal\tedit  ship it", "/goal", "edit  ship it \n", true),
        ("\u{2003}hello\u{2003}世界 🚀  ", "hello\u{2003}世界 🚀", "hello", "世界 🚀  ", false),
        ("\n\t", "", "", "", false),
        ("", "", "", "", false),
        ("/goal", "/goal", "/goal", "", true),
    ] {
        let occurrence = workbench.handler_occurrence(b"tokenize", &[ExecutableValueV1::text(input).unwrap()]).unwrap();
        workbench.run_occurrences_to_candidate(&[occurrence]).unwrap();
        let frame = decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes).unwrap();
        let document = field(&frame, b"document-main");
        assert_eq!(projected_text_value_v1(field(document, b"cleaned")).unwrap(), Some(clean));
        assert_eq!(projected_text_value_v1(field(document, b"first-word")).unwrap(), Some(first));
        assert_eq!(projected_text_value_v1(field(document, b"remaining-words")).unwrap(), Some(rest));
        assert_eq!(field(document, b"prefixed").as_atom().unwrap().canonical_payload(), &[u8::from(prefix)]);
    }
}

#[test]
fn text_operations_reject_wrong_types_and_wrong_arities() {
    for expression in ["trim(true)", "trim(2.0)", "first-word(false)", "remaining-words(1.0)", "trim()", "trim(?input, ?input)"] {
        let source = SOURCE.replace("trim(?input)", expression);
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(), "{expression}");
    }
    for expression in ["starts-with(true, \"/\")", "starts-with(?input, 2.0)", "starts-with(?input)", "trim(?input)"] {
        let source = SOURCE.replace("starts-with(trim(?input), \"/\")", expression);
        assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err(), "{expression}");
    }
}
