use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableValueV1, projected_text_value_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

#[test]
fn text_selectors_match_exact_values_on_declared_and_created_rows() {
    use clause_runtime::projected_relation_table_v1;
    let source = include_str!("../../../test-vectors/authoring/text-selectors.clause");
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    for handler in [b"spawn".as_slice(), b"select".as_slice()] {
        let occurrence = workbench.handler_occurrence(handler, &[]).unwrap();
        workbench.run_occurrences_to_candidate(&[occurrence]).unwrap();
        let projection = workbench.admit().unwrap().projection;
        if handler == b"select" {
            let term = decode_canonical_term_bytes(&projection.exact_term_bytes).unwrap();
            let selected = projected_relation_table_v1(field(field(&term, b"relations"), b"selected")).unwrap().unwrap();
            let values = selected.rows().values().flatten().collect::<Vec<_>>();
            assert_eq!(values.len(), 3);
            assert_eq!(values.iter().filter(|value| ***value == ExecutableValueV1::Boolean(true)).count(), 2);
        }
    }
    let invalid = source.replace("    ?item status \"finished\"", "    ?item status true");
    assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
}

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

#[test]
fn typed_law_domains_survive_relational_specialization() {
    let source = include_str!("../../../test-vectors/authoring/typed-text-law.clause");
    let mut workbench = ResidentSourceWorkbenchV1::open(source.as_bytes()).unwrap();
    let occurrence = workbench.handler_occurrence(b"record", &[ExecutableValueV1::text("世界").unwrap()]).unwrap();
    workbench.run_occurrences_to_candidate(&[occurrence]).unwrap();
    let frame = decode_canonical_term_bytes(&workbench.admit().unwrap().projection.exact_term_bytes).unwrap();
    assert_eq!(projected_text_value_v1(field(field(&frame, b"root"), b"output")).unwrap(), Some("世界"));
    let invalid = source.replace("?item label ?value", "?item label (?value + 1.0)");
    assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
}
