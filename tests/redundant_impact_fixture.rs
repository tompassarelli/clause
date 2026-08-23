//! Frozen expectations for the redundant impact journey.
//!
//! The support-frontier assertions below are deliberately kept in this fixture
//! rather than reimplementing support search in the kernel.  The executable
//! support tests are gated until the `derive::SupportFrontier` seam lands; the
//! parser/model checks remain live on the clean candidate in the meantime.

use clause::{elaborate, frontend, kernel};

const SOURCE: &str = include_str!("../examples/impact.clause");

fn model() -> kernel::Revision {
    let parsed = frontend::parse(SOURCE).expect("redundant impact fixture parses");
    kernel::Revision::admit(elaborate::program(parsed).expect("fixture elaborates"))
}

fn fact(relation: &str, consumer: &str, dependency: &str) -> kernel::Clause {
    kernel::Clause::new(
        relation,
        vec![
            ("consumer".into(), kernel::Term::literal(consumer).unwrap()),
            (
                "dependency".into(),
                kernel::Term::literal(dependency).unwrap(),
            ),
        ],
    )
    .unwrap()
}

fn changes(change: &str, component: &str) -> kernel::Clause {
    kernel::Clause::new(
        "impact/changes",
        vec![
            ("change".into(), kernel::Term::literal(change).unwrap()),
            (
                "component".into(),
                kernel::Term::literal(component).unwrap(),
            ),
        ],
    )
    .unwrap()
}

#[allow(dead_code)]
fn affected(change: &str, consumer: &str) -> kernel::Clause {
    kernel::Clause::new(
        "impact/affected",
        vec![
            ("change".into(), kernel::Term::literal(change).unwrap()),
            ("consumer".into(), kernel::Term::literal(consumer).unwrap()),
        ],
    )
    .unwrap()
}

fn import(consumer: &str, dependency: &str) -> kernel::Clause {
    fact("impact/imports", consumer, dependency)
}

#[test]
fn source_freezes_two_independent_north_routes_and_one_intent() {
    let revision = model();
    assert_eq!(
        revision.model().facts(),
        [
            changes("compiler-change", "Beagle"),
            import("North", "Relay"),
            import("North", "Store"),
            import("Relay", "Beagle"),
            import("Store", "Beagle"),
        ]
    );
    assert_eq!(revision.model().intents().len(), 1);
    assert_eq!(
        revision.model().intents()[0].desired(),
        &import("South", "North")
    );
    assert_eq!(revision.model().query().relation(), "impact/affected");
}

/// Exact support-frontier, prevention, achievement, and support-loss oracle.
///
/// This module waits for the agreed `derive::SupportFrontier` API and the
/// corresponding intervention/diff projections.  Keep every assertion exact
/// when enabling it; do not replace it with counts or a selected proof.
#[cfg(feature = "support-frontier-api")]
mod support_frontier_acceptance {
    use super::*;
    use clause::{delta::RevisionDelta, derive, derive::SupportStatus};

    fn target() -> kernel::Clause {
        affected("compiler-change", "North")
    }

    fn expected_supports() -> Vec<Vec<kernel::Clause>> {
        vec![
            vec![
                changes("compiler-change", "Beagle"),
                import("North", "Relay"),
                import("Relay", "Beagle"),
            ],
            vec![
                changes("compiler-change", "Beagle"),
                import("North", "Store"),
                import("Store", "Beagle"),
            ],
        ]
    }

    fn expected_hitting_sets() -> Vec<Vec<kernel::Clause>> {
        vec![
            vec![import("North", "Relay"), import("North", "Store")],
            vec![import("North", "Relay"), import("Store", "Beagle")],
            vec![import("North", "Store"), import("Relay", "Beagle")],
            vec![import("Relay", "Beagle"), import("Store", "Beagle")],
        ]
    }

    fn expected_south_additions() -> Vec<kernel::Clause> {
        vec![
            import("South", "Beagle"),
            import("South", "North"),
            import("South", "Relay"),
            import("South", "Store"),
        ]
    }

    fn canonical_sets(mut sets: Vec<Vec<kernel::Clause>>) -> Vec<Vec<kernel::Clause>> {
        for set in &mut sets {
            set.sort();
        }
        sets.sort();
        sets
    }

    #[test]
    fn affected_north_has_exactly_two_minimal_supports() {
        let revision = model();
        let frontier = derive::support_frontier(
            &revision,
            &target(),
            derive::SupportLimits::new(derive::Limits::new(100, 10, 10_000), 10_000, 100),
        )
        .expect("support frontier computes");
        assert_eq!(frontier.status(), SupportStatus::Complete);
        assert_eq!(frontier.supports().len(), 2);
        assert_eq!(
            canonical_sets(
                frontier
                    .supports()
                    .iter()
                    .map(|support| support.assertions().to_vec())
                    .collect(),
            ),
            canonical_sets(expected_supports()),
        );
    }

    #[test]
    fn successor_keeps_entailment_but_loses_the_relay_support() {
        let base = model();
        let successor = RevisionDelta::new(
            base.identity(),
            Vec::new(),
            vec![import("North", "Relay"), import("Relay", "Beagle")],
        )
        .expect("withdrawal delta admits")
        .apply(&base)
        .expect("successor applies");
        let base_frontier = derive::support_frontier(
            &base,
            &target(),
            derive::SupportLimits::new(derive::Limits::new(100, 10, 10_000), 10_000, 100),
        )
        .expect("base frontier computes");
        let successor_frontier = derive::support_frontier(
            &successor,
            &target(),
            derive::SupportLimits::new(derive::Limits::new(100, 10, 10_000), 10_000, 100),
        )
        .expect("successor frontier computes");
        assert_eq!(base_frontier.supports().len(), 2);
        assert_eq!(successor_frontier.supports().len(), 1);
        assert_eq!(
            successor_frontier.supports()[0].assertions(),
            expected_supports()[1].as_slice()
        );
    }

    // The four exact prevention and four one-import achievement sets are kept
    // as named expected values for the intervention projection to consume.
    #[test]
    fn intervention_frontiers_are_frozen_as_exact_antichains() {
        assert_eq!(expected_hitting_sets().len(), 4);
        assert_eq!(expected_south_additions().len(), 4);
    }
}
