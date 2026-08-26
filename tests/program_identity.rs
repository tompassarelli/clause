use clause::{elaborate, frontend, kernel::ClauseSemanticsId, wire};

fn revision(source: &str) -> clause::kernel::Revision {
    elaborate::compile(frontend::parse(source).expect("source parses"))
        .expect("source compiles")
        .revision(&frontend::Name("world".to_owned()))
        .expect("world revision")
        .clone()
}

#[test]
fn snapshot_identity_ignores_revision_lineage() {
    let first = revision("Door\n\nworld\n  iron-door ∈ Door\n");
    let second = revision("Door\n\nworld\n  iron-door ∈ Door\n");
    let semantics = ClauseSemanticsId::current();
    assert_eq!(
        wire::program_snapshot_id(first.model(), &semantics),
        wire::program_snapshot_id(second.model(), &semantics)
    );
}

#[test]
fn snapshot_identity_includes_semantics_epoch() {
    let revision = revision("Door\n\nworld\n  iron-door ∈ Door\n");
    let first = ClauseSemanticsId::new("clause-semantics-v1".to_owned()).unwrap();
    let second = ClauseSemanticsId::new("clause-semantics-v2".to_owned()).unwrap();
    assert_ne!(
        wire::program_snapshot_id(revision.model(), &first),
        wire::program_snapshot_id(revision.model(), &second)
    );
}

#[test]
fn snapshot_preimage_excludes_legacy_lineage_and_revision_identity() {
    let revision = revision("Door\n\nworld\n  iron-door ∈ Door\n");
    let semantics = ClauseSemanticsId::current();
    let snapshot = wire::program_snapshot(revision.model().clone(), semantics.clone());
    assert_eq!(
        snapshot.identity(),
        &wire::program_snapshot_id(snapshot.checked_payload(), &semantics)
    );
    assert_eq!(snapshot.semantics(), &semantics);
    let payload = wire::program_snapshot_payload(snapshot.checked_payload(), &semantics);
    assert!(payload.starts_with("[\"clause-program-snapshot-v1\""));
    assert!(!payload.contains("lineage"));
    assert!(!payload.contains("rev-sha256-"));
}
