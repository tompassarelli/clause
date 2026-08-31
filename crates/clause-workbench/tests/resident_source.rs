use std::time::Instant;

use clause_package::{Term, decode_canonical_term_bytes};
use clause_runtime::{ExecutableOccurrenceV1, ExecutableValueV1, encode_executable_occurrence_v1};
use clause_workbench::ResidentSourceWorkbenchV1;

const WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world.clause"
));
const DASH_WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world-dash-jump.clause"
));
const COLLECT_CONTACT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/collect-contact.clause"
));
const SPRING_PAD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/spring-pad.clause"
));
const OBJECTIVE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/objective.clause"
));

fn coherent_source(objective: &[u8]) -> Vec<u8> {
    let mut source = Vec::with_capacity(
        DASH_WORLD.len() + COLLECT_CONTACT.len() + SPRING_PAD.len() + objective.len() + 3,
    );
    for part in [DASH_WORLD, COLLECT_CONTACT, SPRING_PAD, objective] {
        if !source.is_empty() {
            source.push(b'\n');
        }
        source.extend_from_slice(part);
    }
    source
}

fn occurrence(entry: u16, arguments: &[f64]) -> Vec<u8> {
    encode_executable_occurrence_v1(&ExecutableOccurrenceV1 {
        entry,
        arguments: arguments
            .iter()
            .copied()
            .map(|value| ExecutableValueV1::number(value).expect("finite test number"))
            .collect(),
    })
    .expect("resident occurrence encodes")
}

fn tick_chain(mut prefix: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    prefix.push(occurrence(2, &[0.016]));
    for entry in [3, 4, 5, 6, 7, 9] {
        prefix.push(occurrence(entry, &[]));
    }
    prefix
}

fn projected_object_field<'a>(term: &'a Term, expected: &[u8]) -> &'a Term {
    let mut current = term;
    loop {
        if current.as_atom().is_some() {
            panic!("projected object lacks field {expected:?}");
        }
        let [field, value, rest] = current
            .as_triple()
            .expect("projected object is an entry chain")
            .slots();
        let field = field.as_atom().expect("projected field is an Atom");
        if field.canonical_payload() == expected {
            return value;
        }
        current = rest;
    }
}

fn projected_symbol(term: &Term) -> &[u8] {
    let atom = term.as_atom().expect("projected symbol is an Atom");
    assert_eq!(atom.kind(), b"clause/process-projected-symbol-v1");
    atom.canonical_payload()
}

fn objective_state(exact_term_bytes: &[u8]) -> Vec<u8> {
    let term = decode_canonical_term_bytes(exact_term_bytes).expect("projection term decodes");
    let world = projected_object_field(&term, b"world");
    let objective = projected_object_field(world, b"objective");
    projected_symbol(projected_object_field(objective, b"state")).to_vec()
}

#[test]
fn source_edit_hot_reloads_in_one_workbench_without_admission_custody_leak() {
    let mut workbench =
        ResidentSourceWorkbenchV1::open(WORLD).expect("base source opens in one workbench");
    let base_generation = workbench.generation().clone();
    let base_candidate = workbench
        .run_to_candidate()
        .expect("base source produces one hidden candidate");
    assert_eq!(base_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let base_admission = workbench
        .admit()
        .expect("separate base Admission returns the rendered frame");
    assert_eq!(base_admission.state_revision_count, 2);
    assert_ne!(base_admission.predecessor, base_admission.successor);

    let changed = std::str::from_utf8(WORLD)
        .expect("world source is UTF-8")
        .replacen("jump-arena move speed 5.0", "jump-arena move speed 7.0", 1);
    assert_ne!(changed.as_bytes(), WORLD);
    let changed_generation = workbench
        .hot_reload(changed.as_bytes())
        .expect("source-only edit hot reloads in the resident process");
    assert_eq!(
        changed_generation.handle.generation,
        base_generation.handle.generation + 1
    );
    assert_ne!(
        changed_generation.source_package,
        base_generation.source_package
    );
    assert_ne!(changed_generation.cpp1, base_generation.cpp1);
    assert_ne!(changed_generation.cwr1, base_generation.cwr1);
    assert!(
        workbench
            .rejects_stale_handle(base_generation.handle)
            .expect("stale-handle probe reaches the live boundary")
    );

    let changed_candidate = workbench
        .run_to_candidate()
        .expect("changed source reruns without restarting Rust");
    assert_eq!(changed_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let changed_admission = workbench
        .admit()
        .expect("separate changed-source Admission returns its frame");
    assert_eq!(changed_admission.state_revision_count, 2);
    assert_ne!(
        changed_admission.projection.exact_term_bytes, base_admission.projection.exact_term_bytes,
        "the source edit changes the admitted rendered frame"
    );
}

#[test]
fn coherent_source_fails_resets_completes_and_hot_reloads_in_one_workbench() {
    let source = coherent_source(OBJECTIVE);
    let mut workbench =
        ResidentSourceWorkbenchV1::open(&source).expect("coherent source opens resident workbench");
    let base_generation = workbench.generation().clone();

    let failure = tick_chain(vec![occurrence(0, &[0.0, -1.0])]);
    let failed_candidate = workbench
        .run_occurrences_to_candidate(&failure)
        .expect("hazard produces one hidden candidate");
    assert_eq!(failed_candidate.state_revision_count, 1);
    assert!(workbench.last_projection().is_none());
    let failed = workbench
        .admit()
        .expect("separate Admission exposes the failed frame");
    assert_eq!(
        objective_state(&failed.projection.exact_term_bytes),
        b"failed"
    );

    let reset = tick_chain(vec![occurrence(0, &[0.0, 1.0]), occurrence(8, &[])]);
    let reset_candidate = workbench
        .run_occurrences_to_candidate(&reset)
        .expect("reset produces one hidden candidate");
    assert_eq!(reset_candidate.state_revision_count, 2);
    assert_eq!(
        objective_state(&workbench.last_projection().unwrap().exact_term_bytes),
        b"failed",
        "the admitted renderer remains on failure before reset Admission"
    );
    let reset = workbench
        .admit()
        .expect("separate Admission exposes the reset frame");
    assert_eq!(
        objective_state(&reset.projection.exact_term_bytes),
        b"playing"
    );

    let completion = tick_chain(vec![occurrence(0, &[1.0, 0.0])]);
    workbench
        .run_occurrences_to_candidate(&completion)
        .expect("movement and collection produce hidden completion");
    assert_eq!(
        objective_state(&workbench.last_projection().unwrap().exact_term_bytes),
        b"playing",
        "completion is invisible before Admission"
    );
    let completed = workbench
        .admit()
        .expect("separate Admission exposes completion");
    assert_eq!(
        objective_state(&completed.projection.exact_term_bytes),
        b"completed"
    );

    let changed_objective = std::str::from_utf8(OBJECTIVE)
        .expect("objective source is UTF-8")
        .replacen("?player-x = 0.08", "?player-x = 0.16", 1);
    let changed_source = coherent_source(changed_objective.as_bytes());
    let reload_started = Instant::now();
    let changed_generation = workbench
        .hot_reload(&changed_source)
        .expect("Clause-only objective threshold hot reloads in-process");
    let reload_elapsed = reload_started.elapsed();
    eprintln!(
        "resident coherent source hot reload: {:.3} ms",
        reload_elapsed.as_secs_f64() * 1000.0
    );
    assert_ne!(
        changed_generation.source_package,
        base_generation.source_package
    );
    assert_ne!(changed_generation.cpp1, base_generation.cpp1);
    workbench
        .run_to_candidate()
        .expect("changed source reruns without rebuilding Rust");
    let changed = workbench
        .admit()
        .expect("changed source reaches separate Admission");
    assert_eq!(
        objective_state(&changed.projection.exact_term_bytes),
        b"playing",
        "the edited completion threshold defers the objective by one tick"
    );
}
