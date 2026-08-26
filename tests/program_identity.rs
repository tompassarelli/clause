use clause::{
    elaborate, frontend,
    kernel::{
        ClauseSemanticsId, Delta, ProgramChangeOccurrence, ProgramChangeOccurrenceId, ProgramDelta,
        ProgramId, ProgramRevision, ReferentId,
    },
    wire,
};

fn rid(n: u8) -> ReferentId {
    ReferentId::from_digest([n; 32])
}

fn occurrence(
    n: u8,
    program: &ProgramId,
    semantics: &ClauseSemanticsId,
    predecessor: Option<clause::kernel::ProgramRevisionId>,
    snapshot: &clause::kernel::ProgramSnapshotId,
    delta: ProgramDelta,
) -> ProgramChangeOccurrence {
    ProgramChangeOccurrence::new(
        ProgramChangeOccurrenceId::from_referent(rid(n)),
        semantics.clone(),
        program.clone(),
        predecessor,
        snapshot.clone(),
        delta,
        rid(n + 1),
        vec![rid(n + 2)],
    )
    .unwrap()
}

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

#[test]
fn program_history_constitution_and_identity_invariants() {
    let base = revision("Door\n\nworld\n  east ∈ Door\n");
    let endpoint = revision("Door\n\nworld\n  east ∈ Door\n");
    let semantics = ClauseSemanticsId::current();
    let program = ProgramId::from_referent(base.model().id().clone());
    let snapshot = wire::program_snapshot(endpoint.model().clone(), semantics.clone());
    let all: Vec<_> = snapshot.checked_payload().atoms().into_iter().collect();
    let root_change = occurrence(
        10,
        &program,
        &semantics,
        None,
        snapshot.identity(),
        ProgramDelta::new(all.clone(), vec![]).unwrap(),
    );
    let root =
        ProgramRevision::constitute_root(program.clone(), snapshot.clone(), &root_change).unwrap();
    let noop = ProgramDelta::new(vec![], vec![]).unwrap();
    let c1 = occurrence(
        20,
        &program,
        &semantics,
        Some(root.identity().clone()),
        snapshot.identity(),
        noop.clone(),
    );
    let c2 = occurrence(
        30,
        &program,
        &semantics,
        Some(root.identity().clone()),
        snapshot.identity(),
        noop,
    );
    let r1 = ProgramRevision::constitute_successor(&root, snapshot.clone(), &c1).unwrap();
    let r2 = ProgramRevision::constitute_successor(&root, snapshot.clone(), &c2).unwrap();
    assert_ne!(r1.identity(), r2.identity());
    assert_eq!(r1.snapshot().identity(), r2.snapshot().identity());
    assert_eq!(r1.program(), r2.program());
    assert_ne!(r1.identity(), root.identity());
    let other_program = ProgramId::from_referent(rid(77));
    assert_ne!(
        wire::program_revision_id(
            &program,
            &semantics,
            Some(root.identity()),
            snapshot.identity(),
            c1.identity()
        ),
        wire::program_revision_id(
            &other_program,
            &semantics,
            Some(root.identity()),
            snapshot.identity(),
            c1.identity()
        )
    );
    let sem2 = ClauseSemanticsId::new("clause-semantics-v2".to_owned()).unwrap();
    let epoch_change = occurrence(
        60,
        &program,
        &sem2,
        Some(root.identity().clone()),
        snapshot.identity(),
        ProgramDelta::new(vec![], vec![]).unwrap(),
    );
    assert!(ProgramRevision::constitute_successor(&root, snapshot.clone(), &epoch_change).is_err());
    let wrong_delta = occurrence(
        61,
        &program,
        &semantics,
        Some(root.identity().clone()),
        snapshot.identity(),
        ProgramDelta::new(
            vec![clause::kernel::SemanticAtom::Referent(
                clause::kernel::Referent::new(rid(99)),
            )],
            vec![],
        )
        .unwrap(),
    );
    assert!(ProgramRevision::constitute_successor(&root, snapshot, &wrong_delta).is_err());
}

#[test]
fn program_revision_preimage_is_tagged_and_evidence_neutral() {
    let sem = ClauseSemanticsId::current();
    let program = ProgramId::from_referent(rid(1));
    let snap = clause::kernel::ProgramSnapshotId::from_digest([2; 32]);
    let change_a = ProgramChangeOccurrenceId::from_referent(rid(3));
    let change_b = ProgramChangeOccurrenceId::from_referent(rid(4));
    let root = wire::program_revision_payload(&program, &sem, None, &snap, &change_a);
    assert_eq!(
        root,
        "[\"clause/program-revision/v1\",[\"semantics\",\"clause-semantics-v1\"],[\"program\",\"ref-sha256-0101010101010101010101010101010101010101010101010101010101010101\"],[\"predecessor\",[\"root\"]],[\"snapshot\",\"program-snapshot-sha256-0202020202020202020202020202020202020202020202020202020202020202\"],[\"change-occurrence\",\"ref-sha256-0303030303030303030303030303030303030303030303030303030303030303\"]]"
    );
    assert_eq!(
        wire::program_revision_id(&program, &sem, None, &snap, &change_a).as_str(),
        "program-revision-sha256-e4cde6a484e6e3c96fec47834956e935d7f3efdf0b2f346452361048eea332bc"
    );
    assert!(root.contains("[\"predecessor\",[\"root\"]]"));
    assert!(root.contains("[\"change-occurrence\",\"ref-sha256-"));
    let parent = clause::kernel::ProgramRevisionId::new(format!(
        "program-revision-sha256-{}",
        "09".repeat(32)
    ))
    .unwrap();
    let successor = wire::program_revision_payload(&program, &sem, Some(&parent), &snap, &change_b);
    assert!(successor.contains("[\"predecessor\",[\"revision\",\"program-revision-sha256-"));
    assert!(!successor.contains("provenance"));
    assert!(!successor.contains("responsible"));
    assert_ne!(
        wire::program_revision_id(&program, &sem, None, &snap, &change_a),
        wire::program_revision_id(&program, &sem, None, &snap, &change_b)
    );
}

#[test]
fn program_delta_rejects_duplicate_overlap_but_accepts_empty() {
    let atom = clause::kernel::SemanticAtom::Referent(clause::kernel::Referent::new(rid(8)));
    assert!(ProgramDelta::new(vec![], vec![]).is_ok());
    assert!(ProgramDelta::new(vec![atom.clone(), atom.clone()], vec![]).is_err());
    assert!(ProgramDelta::new(vec![atom.clone()], vec![atom]).is_err());
}

#[test]
fn program_change_rejects_duplicate_provenance_and_revision_metadata_mismatch() {
    let sem = ClauseSemanticsId::current();
    let base = revision("Door\n\nworld\n  east ∈ Door\n");
    let snap = wire::program_snapshot(base.model().clone(), sem.clone());
    let program = ProgramId::from_referent(base.model().id().clone());
    let dup = ProgramChangeOccurrence::new(
        ProgramChangeOccurrenceId::from_referent(rid(40)),
        sem.clone(),
        program.clone(),
        None,
        snap.identity().clone(),
        ProgramDelta::new(snap.checked_payload().atoms().into_iter().collect(), vec![]).unwrap(),
        rid(41),
        vec![rid(42), rid(42)],
    );
    assert!(dup.is_err());
    let wrong = occurrence(
        50,
        &ProgramId::from_referent(rid(99)),
        &sem,
        None,
        snap.identity(),
        ProgramDelta::new(snap.checked_payload().atoms().into_iter().collect(), vec![]).unwrap(),
    );
    assert!(ProgramRevision::constitute_root(program, snap, &wrong).is_err());
}
