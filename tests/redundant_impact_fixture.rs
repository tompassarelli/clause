//! Frozen expectations for the redundant impact journey.
//!
//! The support-frontier assertions below are deliberately kept in this fixture
//! rather than reimplementing support search in the kernel.

use clause::{
    elaborate, frontend,
    intervention::{self, AchieveConfig, AchieveResult},
    kernel,
};

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

#[test]
fn explicit_achievement_basis_proves_the_complete_south_frontier() {
    let revision = model();
    let additions = expected_south_additions();
    let result = intervention::achieve(
        &revision,
        affected("compiler-change", "South"),
        &AchieveConfig::new(
            vec!["impact/imports".into()],
            vec![
                "Beagle".into(),
                "North".into(),
                "Relay".into(),
                "South".into(),
                "Store".into(),
            ],
            4,
            100,
            clause::derive::Limits::new(100, 10, 10_000),
        )
        .with_candidate_basis(additions.clone()),
    )
    .expect("explicit achievement frontier computes");

    assert!(matches!(&result, AchieveResult::Solutions(_)));
    assert_eq!(
        result
            .interventions()
            .iter()
            .map(|intervention| intervention.additions().to_vec())
            .collect::<Vec<_>>(),
        additions
            .into_iter()
            .map(|addition| vec![addition])
            .collect::<Vec<_>>(),
    );
}

fn expected_south_additions() -> Vec<kernel::Clause> {
    vec![
        import("South", "Beagle"),
        import("South", "North"),
        import("South", "Relay"),
        import("South", "Store"),
    ]
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
mod support_frontier_acceptance {
    use super::*;
    use clause::{
        delta::RevisionDelta,
        derive::{self, SupportStatus},
        execution,
        intervention::{self, PreventLimits, PreventStatus},
        semantic_diff::SemanticDiff,
    };

    fn limits() -> derive::Limits {
        derive::Limits::new(100, 10, 10_000)
    }

    fn support_limits() -> derive::SupportLimits {
        derive::SupportLimits::new(limits(), 10_000, 100)
    }

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
        let frontier = derive::support_frontier(&revision, &target(), support_limits())
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

        let all = execution::why_all(&revision, &target(), support_limits())
            .expect("why all computes")
            .expect("target is entailed");
        assert!(all.is_complete());
        assert_eq!(all.alternative_count(), 2);
        assert_eq!(
            canonical_sets(
                all.alternatives
                    .iter()
                    .map(|alternative| alternative.assertions.clone())
                    .collect(),
            ),
            canonical_sets(expected_supports()),
        );
        assert_eq!(
            execution::why(&revision, &target(), limits()).expect("why computes"),
            Some(all.alternatives[0].why.clone()),
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
        let base_frontier = derive::support_frontier(&base, &target(), support_limits())
            .expect("base frontier computes");
        let successor_frontier = derive::support_frontier(&successor, &target(), support_limits())
            .expect("successor frontier computes");
        assert_eq!(base_frontier.supports().len(), 2);
        assert_eq!(successor_frontier.supports().len(), 1);
        assert_eq!(
            successor_frontier.supports()[0].assertions(),
            expected_supports()[1].as_slice()
        );

        let diff = SemanticDiff::between(&base, &successor, support_limits())
            .expect("support diff computes");
        assert!(!diff.entailed_removed().contains(&target()));
        let change = diff
            .changed_supports()
            .iter()
            .find(|change| change.fact() == &target())
            .expect("unchanged entailment reports lost support");
        assert!(change.added().is_empty());
        assert_eq!(change.removed().len(), 1);
        assert_eq!(
            change.removed()[0].assertions(),
            expected_supports()[0].as_slice(),
        );
    }

    #[test]
    fn intervention_frontiers_are_frozen_as_exact_antichains() {
        let revision = model();
        let prevention = intervention::prevent(
            &revision,
            target(),
            PreventLimits::new(100, 100, limits())
                .with_support_limits(support_limits())
                .using_relations(vec!["impact/imports".into()]),
        )
        .expect("prevention frontier computes");
        assert_eq!(prevention.status(), PreventStatus::Complete);
        assert_eq!(
            canonical_sets(
                prevention
                    .solutions()
                    .iter()
                    .map(|solution| solution.withdrawals().to_vec())
                    .collect(),
            ),
            canonical_sets(expected_hitting_sets()),
        );
    }
}
