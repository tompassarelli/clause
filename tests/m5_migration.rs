//! Exact old-to-canonical M5 migration parity for the hospital oracle.

use clause::{elaborate, frontend, generated, request, wire};

const LEGACY: &str = include_str!("../examples/hospital.clause");

fn compile(source: &str) -> elaborate::CompiledProgram {
    elaborate::compile(frontend::parse(source).expect("hospital projection parses"))
        .expect("hospital projection elaborates")
}

#[test]
fn hospital_revision_migration_preserves_identity_wire_delta_and_six_outputs() {
    let migration = frontend::migrate(LEGACY).expect("legacy hospital migrates deterministically");
    assert!(migration.source.contains(
        "egress/door-101-withdrawn from egress\n  - Door 101 passed Fire-Marshal-Inspection"
    ));
    assert!(
        migration.source.contains(
            "egress/direct-route:\n  ?origin has a usable egress path to ?destination if"
        )
    );
    assert_eq!(
        migration
            .inferences
            .iter()
            .map(|inference| (
                inference.span,
                inference.before.as_str(),
                inference.after.as_str(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                frontend::Span {
                    line: 37,
                    column: 1,
                    width: "egress/direct-route: DerivationRule".len(),
                },
                "egress/direct-route: DerivationRule",
                "egress/direct-route:",
            ),
            (
                frontend::Span {
                    line: 38,
                    column: 1,
                    width: "  ?origin has a usable egress path to ?destination".len(),
                },
                "  ?origin has a usable egress path to ?destination",
                "  ?origin has a usable egress path to ?destination if",
            ),
            (
                frontend::Span {
                    line: 39,
                    column: 1,
                    width: "  when:".len(),
                },
                "  when:",
                "",
            ),
            (
                frontend::Span {
                    line: 43,
                    column: 1,
                    width: "egress/recursive-route: DerivationRule".len(),
                },
                "egress/recursive-route: DerivationRule",
                "egress/recursive-route:",
            ),
            (
                frontend::Span {
                    line: 44,
                    column: 1,
                    width: "  ?origin has a usable egress path to ?destination".len(),
                },
                "  ?origin has a usable egress path to ?destination",
                "  ?origin has a usable egress path to ?destination if",
            ),
            (
                frontend::Span {
                    line: 45,
                    column: 1,
                    width: "  when:".len(),
                },
                "  when:",
                "",
            ),
            (
                frontend::Span {
                    line: 50,
                    column: 1,
                    width: "egress/door-101-withdrawn: Revision".len(),
                },
                "egress/door-101-withdrawn: Revision",
                "egress/door-101-withdrawn from egress",
            ),
            (
                frontend::Span {
                    line: 51,
                    column: 1,
                    width: "  from: egress".len(),
                },
                "  from: egress",
                "",
            ),
            (
                frontend::Span {
                    line: 52,
                    column: 1,
                    width: "  withdraw:".len(),
                },
                "  withdraw:",
                "",
            ),
            (
                frontend::Span {
                    line: 53,
                    column: 1,
                    width: "    Door 101 passed Fire-Marshal-Inspection".len(),
                },
                "    Door 101 passed Fire-Marshal-Inspection",
                "  - Door 101 passed Fire-Marshal-Inspection",
            ),
        ]
    );

    let legacy = compile(LEGACY);
    let canonical = compile(&migration.source);
    let parity = legacy
        .migration_parity(&canonical)
        .expect("opaque IDs and semantic-v10/revision-v6 bytes are identical");
    assert_eq!(parity.revisions.len(), 2);

    let legacy_successor = legacy
        .revision(&frontend::Name("egress/door-101-withdrawn".into()))
        .expect("legacy successor");
    let canonical_successor = canonical
        .revision(&frontend::Name("egress/door-101-withdrawn".into()))
        .expect("canonical successor");
    assert_eq!(legacy_successor.lineage(), canonical_successor.lineage());
    assert_eq!(
        wire::serialize(legacy_successor),
        wire::serialize(canonical_successor)
    );

    let legacy_requests = request::resolve(&legacy).expect("legacy requests resolve");
    let canonical_requests = request::resolve(&canonical).expect("canonical requests resolve");
    assert_eq!(legacy_requests.requests().len(), 6);
    assert_eq!(canonical_requests.requests().len(), 6);
    let legacy_output = request::run(&legacy_requests, request::RunLimits::default())
        .expect("legacy six outputs run");
    let canonical_output = request::run(&canonical_requests, request::RunLimits::default())
        .expect("canonical six outputs run");
    assert_eq!(
        legacy_output.canonical_bytes(),
        canonical_output.canonical_bytes()
    );
    assert_eq!(
        generated::emit_rust(&legacy_requests, request::RunLimits::default())
            .expect("legacy generated Rust"),
        generated::emit_rust(&canonical_requests, request::RunLimits::default())
            .expect("canonical generated Rust"),
    );
}

#[test]
fn ordinary_multiword_heading_containing_from_is_not_revision_ancestry() {
    let program = frontend::parse("report from North Ward\n")
        .expect("ordinary multiword heading continues through ordinary parsing");
    assert_eq!(program.declarations.len(), 1);
    assert_eq!(
        program.declarations[0].subject.value,
        frontend::Name("report from North Ward".into())
    );
    assert_eq!(program.declarations[0].kind, frontend::Kind::Grounding);
}

#[test]
fn migration_rejects_applied_delta_revisions_instead_of_returning_legacy_ceremony() {
    let applied = LEGACY.replacen(
        "egress/door-101-withdrawn: Revision\n  from: egress\n  withdraw:\n    Door 101 passed Fire-Marshal-Inspection",
        "egress/door-101-delta: Delta\n  from: egress\n  withdraw:\n    Door 101 passed Fire-Marshal-Inspection\n\negress/door-101-withdrawn: Revision\n  from: egress\n  apply: egress/door-101-delta",
        1,
    );
    let apply_line = applied
        .lines()
        .position(|line| line == "  apply: egress/door-101-delta")
        .expect("applied-Delta fixture contains its exact apply line")
        + 1;
    let error = frontend::migrate(&applied)
        .expect_err("applied Delta cannot produce a nominally canonical migration");
    assert_eq!(
        error.span,
        frontend::Span {
            line: apply_line,
            column: 1,
            width: "  apply: egress/door-101-delta".len(),
        }
    );
    assert_eq!(
        error.message,
        "migration cannot canonicalize an applied Delta revision"
    );
}
