use std::time::Instant;

use clause_package::{
    CanonicalDeclaredFrontendV1, DECLARED_FOCUSED_FRONTEND_SOURCE_V1, Term,
    print_canonical_source_v1, read_canonical_source_v1,
    read_canonical_source_with_declared_frontend_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/coherent-declarations.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        let [name, value, rest] = current.as_triple().expect("projected field").slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        current = rest;
    }
}

fn number(term: &Term) -> f64 {
    f64::from_le_bytes(term.as_atom().unwrap().canonical_payload().try_into().unwrap())
}

fn settle(w: &mut ResidentSourceWorkbenchV1) -> Term {
    let occurrence = w.handler_occurrence(b"settle", &[]).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    w.admit().unwrap();
    w.project_current_world().unwrap()
}

#[test]
fn one_binding_contract_checks_runs_prints_and_edits_structured_state() {
    let canonical = print_canonical_source_v1(&read_canonical_source_v1(SOURCE.as_bytes()).unwrap()).unwrap();
    assert_eq!(print_canonical_source_v1(&read_canonical_source_v1(&canonical).unwrap()).unwrap(), canonical);
    let started = Instant::now();
    let mut w = ResidentSourceWorkbenchV1::open(&canonical).unwrap();
    let open = started.elapsed();
    let before = w.project_current_world().unwrap();
    let after = settle(&mut w);
    for (name, expected) in [(b"first".as_slice(), 10.0), (b"second", 0.0)] {
        assert_eq!(number(field(field(&after, name), b"charge")), expected);
        for axis in [b"x".as_slice(), b"y"] {
            assert_eq!(field(field(field(&after, name), b"destination"), axis),
                field(field(field(&before, name), b"position"), axis));
        }
    }
    let bindings = w.state_bindings().unwrap();
    let effect = w.scalar_effects().unwrap().into_iter()
        .find(|effect| effect.expression == b"?limited").unwrap();
    let started = Instant::now();
    w.edit_scalar_effect(w.generation().handle, &effect, b"?limited / 2.0").unwrap();
    let edit = started.elapsed();
    assert_eq!(w.state_bindings().unwrap(), bindings);
    assert_eq!(w.project_current_world().unwrap(), after);
    let edited = settle(&mut w);
    assert_eq!(number(field(field(&edited, b"first"), b"charge")), 5.0);
    assert_eq!(number(field(field(&edited, b"second"), b"charge")), 1.5);
    eprintln!("coherent declaration journey: open={open:?}, checked live edit={edit:?}");
}

#[test]
fn declared_focus_changes_contracts_patterns_and_printing_together() {
    let declared = std::str::from_utf8(DECLARED_FOCUSED_FRONTEND_SOURCE_V1).unwrap()
        .replace("      : ?object", "      means: ?object");
    let source = SOURCE.lines().map(|line| {
        if line.starts_with(' ') { line.replace(": ", " means ") } else { line.to_owned() }
    }).collect::<Vec<_>>().join("\n") + "\n";
    assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err());
    let frontend = CanonicalDeclaredFrontendV1::read(declared.as_bytes()).unwrap();
    let cst = read_canonical_source_with_declared_frontend_v1(source.as_bytes(), &frontend).unwrap();
    let canonical = print_canonical_source_v1(&cst).unwrap();
    let reparsed = read_canonical_source_with_declared_frontend_v1(&canonical, &frontend).unwrap();
    assert_eq!(print_canonical_source_v1(&reparsed).unwrap(), canonical);
    let mut w = ResidentSourceWorkbenchV1::open_with_declared_frontend(&canonical, declared.as_bytes()).unwrap();
    assert_eq!(number(field(field(&settle(&mut w), b"first"), b"charge")), 10.0);
}

#[test]
fn declaration_constraints_reject_missing_mistyped_and_duplicate_bindings() {
    for invalid in [
        SOURCE.replace("    maximum: F64\n", ""),
        SOURCE.replace("    maximum: F64", "    maximum: Bool"),
        SOURCE.replace("    maximum: F64", "    maximum: F64\n    maximum: F64"),
        SOURCE.replace("    y: F64", "    y: Bool"),
        SOURCE.replace("y: 3.0", "y: true"),
        SOURCE.replace("given amount minimum maximum", "given amount minimum"),
    ] {
        assert_ne!(invalid, SOURCE);
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
    }
}
