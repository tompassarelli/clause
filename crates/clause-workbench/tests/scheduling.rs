use clause_package::{
    CanonicalDeclaredFrontendV1, DECLARED_FOCUSED_FRONTEND_SOURCE_V1, Term,
    decode_canonical_term_bytes, print_canonical_source_v1,
    read_canonical_source_with_declared_frontend_v1,
};
use clause_runtime::{ExecutableValueV1 as V, projected_referent_value_v1, projected_relation_table_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &str = include_str!("../../../test-vectors/authoring/scheduling.clause");

fn field<'a>(term: &'a Term, key: &[u8]) -> &'a Term {
    let mut node = term;
    loop {
        let [name, value, rest] = node.as_triple().expect("projected object").slots();
        if name.as_atom().unwrap().canonical_payload() == key { return value; }
        node = rest;
    }
}

fn run(w: &mut ResidentSourceWorkbenchV1, action: Option<(&[u8], V)>) -> Term {
    let Some((name, value)) = action else {
        return w.project_current_world().unwrap();
    };
    let event = w.handler_occurrence(name, &[value]).unwrap();
    w.run_occurrences_to_candidate(&[event]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes()).unwrap()
}

fn referent(frame: &Term, name: &[u8]) -> V {
    V::Referent(projected_referent_value_v1(field(field(frame, name), b"$referent")).unwrap().unwrap())
}

fn rows(frame: &Term, name: &[u8]) -> clause_runtime::ExecutableRelationTableV1 {
    projected_relation_table_v1(field(field(frame, b"relations"), name)).unwrap().unwrap()
}

fn completed(frame: &Term) -> usize {
    rows(frame, b"completed").rows().values().flatten().filter(|v| **v == V::Boolean(true)).count()
}

#[test]
fn scheduling_controls_use_checked_dependencies_and_preserve_identity() {
    assert!(!SOURCE.contains("member of"));
    let mut w = ResidentSourceWorkbenchV1::open(SOURCE.as_bytes()).unwrap();
    assert!(
        w.scalar_effects()
            .unwrap()
            .iter()
            .any(|effect| effect.expression == b"?prior + 2.0"),
        "recursive scheduling laws must not hide the editable extension expression",
    );
    let initial = run(&mut w, None);
    assert_eq!(rows(&initial, b"duration").rows().len(), 5);
    assert_eq!(rows(&initial, b"blocker").rows().values().map(|v| v.len()).sum::<usize>(), 8);
    let design = referent(&initial, b"design");
    let denied = run(&mut w, Some((b"complete", design.clone())));
    assert_eq!(completed(&denied), 0);
    let approval = referent(&initial, b"approval");
    let cleared = run(&mut w, Some((b"resolve", approval)));
    assert_eq!(rows(&cleared, b"blocker").rows().len(), 3);
    let designed = run(&mut w, Some((b"complete", design.clone())));
    assert_eq!(completed(&designed), 1);
    let prototype = referent(&initial, b"prototype");
    assert_eq!(completed(&run(&mut w, Some((b"complete", prototype.clone())))), 1);
    let extended = run(&mut w, Some((b"extend", prototype.clone())));
    assert_eq!(referent(&extended, b"prototype"), prototype);
    assert!(rows(&extended, b"duration").rows().values().flatten().any(|v| v.as_number() == Some(6.0)));
    run(&mut w, Some((b"resolve", referent(&initial, b"components"))));
    for name in [b"prototype".as_slice(), b"validation", b"documentation", b"launch"] {
        run(&mut w, Some((b"complete", referent(&initial, name))));
    }
    let finished = run(&mut w, None);
    assert_eq!(completed(&finished), 5);
    assert!(rows(&finished, b"waiting").rows().is_empty());
    assert!(rows(&finished, b"blocker").rows().is_empty());
    assert_eq!(referent(&finished, b"design"), design);
}

#[test]
fn scheduling_rejects_wrong_input_domains_and_cannot_complete_a_cycle() {
    let invalid = SOURCE.replace("Complete as Task", "Complete as Root");
    assert!(ResidentSourceWorkbenchV1::open(invalid.as_bytes()).is_err());
    let cycle = SOURCE.replace("  title: \"Design the prototype\"", "  title: \"Design the prototype\"\n  prerequisite: launch");
    let mut w = ResidentSourceWorkbenchV1::open(cycle.as_bytes()).unwrap();
    let initial = run(&mut w, None);
    for root in [b"approval".as_slice(), b"components"] {
        run(&mut w, Some((b"resolve", referent(&initial, root))));
    }
    let after = run(&mut w, Some((b"complete", referent(&initial, b"design"))));
    assert!(rows(&after, b"blocker").rows().is_empty());
    assert!(!rows(&after, b"waiting").rows().is_empty());
    assert_eq!(completed(&after), 0);
}

#[test]
fn scheduling_uses_a_changed_declared_reading_without_a_host_reader_change() {
    let declared_frontend = std::str::from_utf8(DECLARED_FOCUSED_FRONTEND_SOURCE_V1)
        .unwrap()
        .replace(
            "      : ?object",
            "      means: ?object",
        );
    let source = SOURCE.replace(": ", " means ");

    assert!(ResidentSourceWorkbenchV1::open(source.as_bytes()).is_err());
    let frontend = CanonicalDeclaredFrontendV1::read(declared_frontend.as_bytes()).unwrap();
    let cst = read_canonical_source_with_declared_frontend_v1(source.as_bytes(), &frontend)
        .expect("the changed Reading owns the new focused syntax");
    let canonical = print_canonical_source_v1(&cst).expect("the same Reading prints its syntax");
    let reparsed = read_canonical_source_with_declared_frontend_v1(&canonical, &frontend)
        .expect("canonical output re-elaborates under the same Reading");
    assert_eq!(print_canonical_source_v1(&reparsed).unwrap(), canonical);

    let mut workbench = ResidentSourceWorkbenchV1::open_with_declared_frontend(
        &canonical,
        declared_frontend.as_bytes(),
    )
    .expect("the changed Reading reaches the resident scheduling consumer");
    let reloaded = String::from_utf8(canonical)
        .unwrap()
        .replace("Design approval", "Design approved");
    workbench.hot_reload(reloaded.as_bytes())
        .expect("reload retains the selected declared frontend");
    let initial = run(&mut workbench, None);
    let design = referent(&initial, b"design");
    let approval = referent(&initial, b"approval");
    run(&mut workbench, Some((b"resolve", approval)));
    let designed = run(&mut workbench, Some((b"complete", design)));
    assert_eq!(completed(&designed), 1);
    let prototype = referent(&initial, b"prototype");
    let extended = run(&mut workbench, Some((b"extend", prototype)));
    assert!(rows(&extended, b"duration").rows().values().flatten()
        .any(|value| value.as_number() == Some(6.0)));
}

#[test]
fn compile_source_exports_a_canonical_runnable_browser_request() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_clause-workbench"))
        .arg("compile-source")
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-vectors/authoring/scheduling.clause"))
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let request = clause_runtime::decode_wasm_process_request_v1(&output.stdout).unwrap();
    assert_eq!(
        clause_runtime::encode_wasm_process_request_v1(&request).unwrap(),
        output.stdout,
        "compile-source output must be exact canonical CWR1",
    );
    let session = clause_runtime::open_fresh_persistent_process_session_v1(&output.stdout).unwrap();
    let initial = session.current_accepted_projection_term().unwrap().unwrap();
    assert_eq!(rows(initial, b"duration").rows().len(), 5);
}
