use clause::{
    elaborate, frontend,
    kernel::{ClauseSemanticsId, Delta},
    wire,
};

fn revision(source: &str) -> clause::kernel::Revision {
    elaborate::compile(frontend::parse(source).expect("source parses"))
        .expect("source compiles")
        .revision(&frontend::Name("world".to_owned()))
        .expect("world revision")
        .clone()
}

#[test]
fn snapshot_identity_ignores_revision_lineage() {
    let parent_a = revision("Door\n\nworld\n  east ∈ Door\n");
    let parent_b = revision("Door\n\nworld\n  west ∈ Door\n");
    let endpoint = revision("Door\n\nworld\n  north ∈ Door\n");
    let semantics = ClauseSemanticsId::current();
    let endpoint_atoms = endpoint.model().atoms();
    let successor = |base: &clause::kernel::Revision| {
        let base_atoms = base.model().atoms();
        let admissions = endpoint_atoms.difference(&base_atoms).cloned().collect();
        let withdrawals = base_atoms.difference(&endpoint_atoms).cloned().collect();
        wire::admit_successor(
            base,
            endpoint.model().clone(),
            Delta::new(base.identity().clone(), admissions, withdrawals).unwrap(),
        )
        .unwrap()
    };
    let successor_a = successor(&parent_a);
    let successor_b = successor(&parent_b);
    assert_ne!(successor_a.predecessor(), successor_b.predecessor());
    assert_ne!(successor_a.identity(), successor_b.identity());
    assert_eq!(successor_a.model(), successor_b.model());
    assert_eq!(
        wire::program_snapshot_id(successor_a.model(), &semantics),
        wire::program_snapshot_id(successor_b.model(), &semantics)
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
    assert!(payload.starts_with("[\"clause/program-snapshot/v1\""));
    assert!(payload.contains("[\"root-scope\",\""));
    assert!(!payload.contains("[\"model\",\""));
    assert!(!payload.contains("lineage"));
    assert!(!payload.contains("rev-sha256-"));
}
