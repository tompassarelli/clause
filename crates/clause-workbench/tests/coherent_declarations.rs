use std::{collections::BTreeMap, time::Instant};

use clause_package::{
    CanonicalDeclaredFrontendV1, CanonicalStatePathV1, DECLARED_FOCUSED_FRONTEND_SOURCE_V1, Term,
    print_canonical_source_v1, read_canonical_source_v1,
    read_canonical_source_with_declared_frontend_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;
use clause_runtime::{ExecutableReferentIdentityV1, projected_referent_value_v1};

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

fn values(term: &Term) -> Vec<&Term> {
    let mut values = Vec::new();
    let mut current = term;
    while let Some(triple) = current.as_triple() {
        let [_, value, rest] = triple.slots();
        values.push(value);
        current = rest;
    }
    values
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
    let continuity = w.source_continuity().unwrap();
    let retained = values(field(&continuity, b"formations"))
        .into_iter()
        .flat_map(values)
        .map(|mapping| (number(field(mapping, b"old")) as u32, number(field(mapping, b"new")) as u32))
        .collect::<BTreeMap<_, _>>();
    let current = w.state_bindings().unwrap();
    assert_eq!(current.len(), bindings.len());
    for prior in &bindings {
        let successor = current.iter().find(|next| {
            next.state.assertion.get() == retained[&prior.state.assertion.get()]
                && match (&prior.state.path, &next.state.path) {
                    (CanonicalStatePathV1::Field { formation: old, .. }, CanonicalStatePathV1::Field { formation: new, .. }) =>
                        new.get() == retained[&old.get()],
                    (CanonicalStatePathV1::Rows, CanonicalStatePathV1::Rows) => true,
                    _ => false,
                }
        }).expect("every state binding has an explicitly retained successor");
        assert_ne!(successor.state.assertion, prior.state.assertion);
    }
    let preserved = w.project_current_world().unwrap();
    for name in [b"first".as_slice(), b"second"] {
        let old = field(&after, name);
        let new = field(&preserved, name);
        for property in [b"position".as_slice(), b"destination", b"charge"] {
            assert_eq!(field(new, property), field(old, property));
        }
        let old_identity = projected_referent_value_v1(field(old, b"$referent")).unwrap().unwrap();
        let new_identity = projected_referent_value_v1(field(new, b"$referent")).unwrap().unwrap();
        let ExecutableReferentIdentityV1::Declared(old_coordinate) = old_identity.identity() else {
            panic!("initial state has declared subject identity");
        };
        assert_eq!(new_identity.identity(), &ExecutableReferentIdentityV1::Declared(retained[old_coordinate]));
        assert_eq!(new_identity.domain(), retained[&old_identity.domain()]);
    }
    let edited = settle(&mut w);
    assert_eq!(number(field(field(&edited, b"first"), b"charge")), 5.0);
    assert_eq!(number(field(field(&edited, b"second"), b"charge")), 1.5);
    eprintln!("coherent declaration journey: open={open:?}, checked live edit={edit:?}");
}

#[test]
fn declared_focus_changes_contracts_patterns_and_printing_together() {
    let declared = std::str::from_utf8(DECLARED_FOCUSED_FRONTEND_SOURCE_V1).unwrap()
        .replace("    : ?object", "    means: ?object");
    let grouped = SOURCE.replace("  charge: 9.0", "\n(charge: 9.0):\n  first first")
        .replace("?amount ?minimum ?maximum ?result", "?amount ?amount ?minimum ?maximum ?result");
    let native = read_canonical_source_v1(grouped.as_bytes()).unwrap();
    let equal = native.applications().iter().filter(|a| a.subject == b"first" && a.role == b"charge").collect::<Vec<_>>();
    assert_eq!(equal.len(), 2);
    assert_eq!(equal[0].object, equal[1].object);
    assert_ne!(equal[0].origin, equal[1].origin);
    for emission in equal {
        assert_eq!(&grouped.as_bytes()[emission.origin.start as usize..emission.origin.end as usize], b"first");
    }
    assert_eq!(native.declarations().find(|d| d.designation == b"limited").unwrap().bindings.len(), 4);
    assert!(read_canonical_source_v1(b"charge: 9.0\n  first\n").is_err());
    let source = grouped.lines().map(|line| {
        if line.starts_with(' ') {
            let trimmed = line.trim();
            if trimmed.ends_with(':') && !trimmed.starts_with(['?', '(']) {
                format!("{} means", line.trim_end_matches(':'))
            } else { line.replace(": ", " means ") }
        } else { line.to_owned() }
    }).collect::<Vec<_>>().join("\n").replace("(charge: 9.0):", "(charge means 9.0):") + "\n";
    assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err());
    let frontend = CanonicalDeclaredFrontendV1::read(declared.as_bytes()).unwrap();
    let cst = read_canonical_source_with_declared_frontend_v1(source.as_bytes(), &frontend).unwrap();
    let canonical = print_canonical_source_v1(&cst).unwrap();
    let reparsed = read_canonical_source_with_declared_frontend_v1(&canonical, &frontend).unwrap();
    assert_eq!(print_canonical_source_v1(&reparsed).unwrap(), canonical);
    let mut w = ResidentSourceWorkbenchV1::open_with_declared_frontend(&canonical, declared.as_bytes()).unwrap();
    assert_eq!(number(field(field(&settle(&mut w), b"first"), b"charge")), 10.0);
    let effect = w.scalar_effects().unwrap().into_iter()
        .find(|effect| effect.expression == b"?limited").unwrap();
    w.edit_scalar_effect(w.generation().handle, &effect, b"?limited / 2.0").unwrap();
    let encoded = w.last_source_edit().unwrap();
    let transaction = clause_runtime::decode_executable_scalar_edit_transaction_v1(encoded).unwrap();
    assert_eq!(clause_runtime::encode_executable_scalar_edit_transaction_v1(&transaction).unwrap(), encoded);
    assert_eq!(w.source_preparation().unwrap(), clause_runtime::encode_executable_source_preparation_v1(
        w.exact_source(), transaction.new_root, declared.as_bytes()).unwrap());
    assert_eq!(number(field(field(&settle(&mut w), b"first"), b"charge")), 5.0);
}

#[test]
fn declaration_constraints_reject_missing_mistyped_and_duplicate_bindings() {
    for invalid in [
        SOURCE.replace("?amount ?minimum ?maximum ?result", "?amount ?minimum ?result"),
        SOURCE.replace("?amount ?minimum ?maximum ?result", "?amount ?minimum ?result\n  ?maximum:\n    shape: Bool"),
        SOURCE.replace("?amount ?minimum ?maximum ?result", "?amount ?minimum ?maximum ?result\n  ?maximum:\n    shape: Bool"),
        SOURCE.replace("and ?maximum as: ?result", "and ?minimum as: ?result"),
        SOURCE.replace("  y: F64", "  y: Bool"),
        SOURCE.replace("  y: F64", "  y: F64\n  y: F64"),
        SOURCE.replace("y: 3.0", "y: true"),
        SOURCE.replace("given amount minimum maximum", "given amount minimum"),
        SOURCE.replace("?charge ?limited", "?charge ?destination"),
        SOURCE.replace("(shape: F64):", "shape: F64"),
    ] {
        assert_ne!(invalid, SOURCE);
        assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
    }
}
