use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{
    ExecutableReferentV1, ExecutableRelationTableV1, ExecutableValueV1 as V,
    projected_relation_table_v1,
};
use clause_workbench::ResidentSourceWorkbenchV1;

const SOURCE: &[u8] = include_bytes!("../../../test-vectors/authoring/dynamic-text-goals.clause");
const CHILD_PATH: &str = "CLAUSE_CHECKPOINT_CHILD_PATH";

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
fn table(term: &Term, name: &[u8]) -> ExecutableRelationTableV1 {
    projected_relation_table_v1(field(field(term, b"relations"), name))
        .unwrap()
        .unwrap()
}
fn known(term: &Term) -> Vec<ExecutableReferentV1> {
    table(term, b"known-goal")
        .rows()
        .values()
        .flatten()
        .map(|value| value.as_referent().unwrap().clone())
        .collect()
}
fn text(value: &str) -> V {
    V::text(value).unwrap()
}
fn run(w: &mut ResidentSourceWorkbenchV1, name: &[u8], arguments: &[V]) -> Term {
    let occurrence = w.handler_occurrence(name, arguments).unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    decode_canonical_term_bytes(&w.admit().unwrap().projection.exact_term_bytes).unwrap()
}

#[test]
fn dynamic_world_reopens_in_another_process_and_continues_allocating() {
    if let Some(path) = std::env::var_os(CHILD_PATH) {
        let bytes = std::fs::read(&path).unwrap();
        let mut w = ResidentSourceWorkbenchV1::reopen(SOURCE, &bytes).unwrap();
        assert_eq!(w.checkpoint_admitted().unwrap(), bytes);
        let before = w.project_current_world().unwrap();
        let first = known(&before)[0].clone();
        assert_eq!(
            table(&before, b"goal-title").rows()[&first].first(),
            Some(&text("旅 🚀"))
        );
        assert_eq!(
            table(&before, b"goal-objective").rows()[&first].first(),
            Some(&text("Preserve identities"))
        );
        let after = run(
            &mut w,
            b"redirect-goal",
            &[V::Referent(first.clone()), text("Continue after reopen")],
        );
        assert_eq!(known(&after), vec![first.clone()]);
        assert_eq!(
            table(&after, b"prior-goal-objective").rows()[&first].first(),
            Some(&text("Preserve identities"))
        );
        let after = run(
            &mut w,
            b"create-goal",
            &[text("Second"), text("New identity")],
        );
        let goals = known(&after);
        assert_eq!(goals.len(), 2);
        assert!(goals.contains(&first));
        assert_ne!(goals[0], goals[1]);
        std::fs::write(path, w.checkpoint_admitted().unwrap()).unwrap();
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "clause-checkpoint-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("world.checkpoint");
    let mut w = ResidentSourceWorkbenchV1::open_continuous(SOURCE).unwrap();
    let before = run(
        &mut w,
        b"create-goal",
        &[text("旅 🚀"), text("Preserve identities")],
    );
    let first = known(&before)[0].clone();
    let generation = w.generation().clone();
    let bytes = w.checkpoint_admitted().unwrap();
    std::fs::write(&path, &bytes).unwrap();
    drop(w);
    let reopened =
        ResidentSourceWorkbenchV1::reopen(SOURCE, &std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(reopened.project_current_world().unwrap(), before);
    assert_eq!(reopened.generation(), &generation);
    assert_eq!(reopened.checkpoint_admitted().unwrap(), bytes);
    drop(reopened);
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "dynamic_world_reopens_in_another_process_and_continues_allocating",
            "--nocapture",
        ])
        .env(CHILD_PATH, &path)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    let mut reopened =
        ResidentSourceWorkbenchV1::reopen(SOURCE, &std::fs::read(&path).unwrap()).unwrap();
    let after = reopened.project_current_world().unwrap();
    let prior = known(&after);
    assert_eq!(prior.len(), 2);
    assert!(prior.contains(&first));
    assert_eq!(
        table(&after, b"goal-objective").rows()[&first].first(),
        Some(&text("Continue after reopen"))
    );
    let after = run(
        &mut reopened,
        b"create-goal",
        &[text("Third"), text("Another identity")],
    );
    let current = known(&after);
    assert_eq!(current.len(), 3);
    assert!(prior.iter().all(|referent| current.contains(referent)));
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&directory).unwrap();
}

#[test]
fn checkpoint_rejects_pending_corrupt_and_mismatched_source_without_reset() {
    let mut w = ResidentSourceWorkbenchV1::open_continuous(SOURCE).unwrap();
    let initial = w.checkpoint_admitted().unwrap();
    let reopened = ResidentSourceWorkbenchV1::reopen(SOURCE, &initial).unwrap();
    assert_eq!(
        reopened.project_current_world().unwrap(),
        w.project_current_world().unwrap()
    );
    let occurrence = w
        .handler_occurrence(b"create-goal", &[text("Kept"), text("Never reset")])
        .unwrap();
    w.run_occurrences_to_candidate(&[occurrence]).unwrap();
    assert!(w.checkpoint_admitted().is_err());
    w.admit().unwrap();
    let bytes = w.checkpoint_admitted().unwrap();
    let mut corrupt = bytes.clone();
    let end = corrupt.len() - 1;
    corrupt[end] ^= 1;
    assert!(ResidentSourceWorkbenchV1::reopen(SOURCE, &corrupt).is_err());
    assert!(ResidentSourceWorkbenchV1::reopen(SOURCE, &bytes[..bytes.len() - 1]).is_err());
    let changed = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("north-main", "another-north");
    assert!(ResidentSourceWorkbenchV1::reopen(changed.as_bytes(), &bytes).is_err());
    assert_eq!(w.checkpoint_admitted().unwrap(), bytes);
    assert_eq!(
        ResidentSourceWorkbenchV1::reopen(SOURCE, &bytes)
            .unwrap()
            .project_current_world()
            .unwrap(),
        w.project_current_world().unwrap()
    );
}

#[test]
fn long_text_world_preserves_exact_bytes_and_identities_after_reopen() {
    let title = format!("{}🚀", "x".repeat(65_532));
    let objective = "旅 🚀\n".repeat(51_200);
    assert_eq!(objective.len(), 450 * 1024);
    let mut w = ResidentSourceWorkbenchV1::open_continuous(SOURCE).unwrap();
    let before = run(&mut w, b"create-goal", &[text(&title), text(&objective)]);
    let first = known(&before)[0].clone();
    assert_eq!(table(&before, b"goal-title").rows()[&first], [text(&title)].into());
    assert_eq!(table(&before, b"goal-objective").rows()[&first], [text(&objective)].into());
    let bytes = w.checkpoint_admitted().unwrap();
    let generation = w.generation().clone();
    drop(w);
    let mut reopened = ResidentSourceWorkbenchV1::reopen(SOURCE, &bytes).unwrap();
    assert_eq!(reopened.generation(), &generation);
    assert_eq!(reopened.project_current_world().unwrap(), before);
    assert_eq!(reopened.checkpoint_admitted().unwrap(), bytes);
    let after = run(&mut reopened, b"redirect-goal", &[
        V::Referent(first.clone()), text(&format!("{objective}continued")),
    ]);
    assert_eq!(known(&after), vec![first.clone()]);
    assert_eq!(table(&after, b"prior-goal-objective").rows()[&first], [text(&objective)].into());
    assert_eq!(table(&after, b"goal-objective").rows()[&first], [text(&format!("{objective}continued"))].into());
    let next = reopened.checkpoint_admitted().unwrap();
    drop(reopened);
    let reopened = ResidentSourceWorkbenchV1::reopen(SOURCE, &next).unwrap();
    assert_eq!(reopened.project_current_world().unwrap(), after);
    assert_eq!(reopened.checkpoint_admitted().unwrap(), next);
}

#[test]
fn admitted_world_projection_reopens_above_diagnostic_size() {
    let value = "旅 🚀\n".repeat(51_200);
    let mut w = ResidentSourceWorkbenchV1::open_continuous(SOURCE).unwrap();
    run(&mut w, b"create-goal", &[text(&value), text(&value)]);
    let before = run(&mut w, b"create-goal", &[text(&value), text(&value)]);
    let projection = clause_package::canonical_term_bytes(&before).unwrap();
    assert!(projection.len() > 1024 * 1024);
    let checkpoint = w.checkpoint_admitted().unwrap();
    let generation = w.generation().clone();
    drop(w);
    let reopened = ResidentSourceWorkbenchV1::reopen(SOURCE, &checkpoint).unwrap();
    assert_eq!(reopened.generation(), &generation);
    assert_eq!(reopened.project_current_world().unwrap(), before);
    assert_eq!(reopened.checkpoint_admitted().unwrap(), checkpoint);
}

#[test]
fn aggregate_checkpoint_uses_boundary_capacity_and_reopens() {
    let value = "x".repeat(450 * 1024);
    let mut w = ResidentSourceWorkbenchV1::open_continuous(SOURCE).unwrap();
    for _ in 0..7 {
        run(&mut w, b"create-goal", &[text(&value), text(&value)]);
    }
    let before = w.project_current_world().unwrap();
    let checkpoint = w.checkpoint_admitted().unwrap();
    let mut body = &checkpoint[4..];
    let mut inner = &[][..];
    for _ in 0..3 {
        let count = u32::from_le_bytes(body[..4].try_into().unwrap()) as usize;
        inner = &body[4..4 + count];
        body = &body[4 + count..];
    }
    assert_eq!(&inner[..4], b"CRF1");
    assert!(inner.len() > 16 * 1024 * 1024);
    assert!(checkpoint.len() > 32 * 1024 * 1024);
    let generation = w.generation().clone();
    drop(w);
    let reopened = ResidentSourceWorkbenchV1::reopen(SOURCE, &checkpoint).unwrap();
    assert_eq!(reopened.generation(), &generation);
    assert_eq!(reopened.project_current_world().unwrap(), before);
    assert_eq!(reopened.checkpoint_admitted().unwrap(), checkpoint);
}
