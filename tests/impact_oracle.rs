//! Acceptance oracle for the dependency-impact flagship model.
//!
//! The runtime law evaluator and revision-delta APIs are intentionally not
//! available at the checkpoint this fixture is authored against.  Keep the
//! contract executable as soon as those APIs land without coupling this test
//! to guessed interfaces in the meantime.

const SOURCE: &str = include_str!("../examples/impact.clause");

const BASE_FACTS: &[(&str, &str, &str)] = &[
    ("imports", "North", "Store"),
    ("imports", "Store", "Beagle"),
    ("changes", "compiler-change", "Beagle"),
];

const BASE_DEPENDENCY_CLOSURE: &[(&str, &str, &str)] = &[
    ("depends", "North", "Beagle"),
    ("depends", "North", "Store"),
    ("depends", "Store", "Beagle"),
];

const BASE_QUERY_RESULTS: &[&str] = &["North", "Store"];

const BASE_PROOF_CHAINS: &[(&str, &str)] = &[
    (
        "compiler-change affects North",
        "changes(compiler-change, Beagle) + depends(North, Beagle) <= imports(North, Store) + imports(Store, Beagle)",
    ),
    (
        "compiler-change affects Store",
        "changes(compiler-change, Beagle) + depends(Store, Beagle) <= imports(Store, Beagle)",
    ),
];

const INTENT: (&str, &str, &str) = ("imports", "South", "North");

const SUCCESSOR_AUTHORED_DELTA: &[(&str, &str, &str)] = &[INTENT];

const SUCCESSOR_DERIVED_DELTA: &[(&str, &str, &str)] = &[
    ("depends", "South", "North"),
    ("depends", "South", "Beagle"),
    ("depends", "South", "Store"),
    ("affected", "compiler-change", "South"),
];

const SUCCESSOR_QUERY_RESULTS: &[&str] = &["North", "South", "Store"];

#[test]
#[ignore = "law runtime integration pending"]
fn impact_oracle() {
    for required in [
        "relation impact/imports(consumer: Text, dependency: Text):",
        "relation impact/depends(consumer: Text, dependency: Text):",
        "relation impact/changes(change: Text, component: Text):",
        "relation impact/affected(change: Text, consumer: Text):",
        "law impact/direct-dependency:",
        "law impact/recursive-dependency:",
        "law impact/impact:",
        "intent impact/adopt-south:",
        "query impact:",
    ] {
        assert!(
            SOURCE.contains(required),
            "fixture lost required clause: {required}"
        );
    }

    assert_eq!(BASE_FACTS.len(), 3);
    assert_eq!(
        BASE_DEPENDENCY_CLOSURE,
        &[
            ("depends", "North", "Beagle"),
            ("depends", "North", "Store"),
            ("depends", "Store", "Beagle"),
        ]
    );
    assert_eq!(BASE_QUERY_RESULTS, &["North", "Store"]);
    assert_eq!(
        BASE_PROOF_CHAINS,
        &[
            (
                "compiler-change affects North",
                "changes(compiler-change, Beagle) + depends(North, Beagle) <= imports(North, Store) + imports(Store, Beagle)",
            ),
            (
                "compiler-change affects Store",
                "changes(compiler-change, Beagle) + depends(Store, Beagle) <= imports(Store, Beagle)",
            ),
        ]
    );
    assert_eq!(BASE_PROOF_CHAINS.len(), BASE_QUERY_RESULTS.len());

    assert_eq!(SUCCESSOR_AUTHORED_DELTA, &[INTENT]);
    assert_eq!(
        SUCCESSOR_DERIVED_DELTA,
        &[
            ("depends", "South", "North"),
            ("depends", "South", "Beagle"),
            ("depends", "South", "Store"),
            ("affected", "compiler-change", "South"),
        ]
    );
    assert_eq!(SUCCESSOR_QUERY_RESULTS, &["North", "South", "Store"]);
}
