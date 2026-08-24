//! Typed acceptance checks for the redundant dependency-impact model.

use std::collections::BTreeMap;

use clause::{
    delta::RevisionDiff,
    derive::{self, SupportStatus},
    elaborate, execution, frontend,
    intervention::{self, AchieveAll, InterventionLimits, PreventAll},
    kernel::{ReferentId, RelationalContent, Revision, RoleId, Term},
    semantic_diff::SemanticDiff,
};

const SOURCE: &str = include_str!("../examples/impact.clause");

const ACHIEVEMENT_SOURCE: &str = r#"Start: Type
Option: Type
State: Type

choice/selects: RelationShape
    {start: Start} selects {option: Option}
    mode start -> option: many

choice/reached: RelationShape
    {start: Start} reached {state: State}
    mode start -> state: many

choice: Model
    South: Start
    Beagle: Option
    North: Option
    Relay: Option
    Store: Option
    Ready: State

choice/selection-reaches-ready: DerivationRule
    ?start reached Ready
    when:
        ?start selects ?option
"#;

fn impact() -> elaborate::CompiledProgram {
    elaborate::compile(frontend::parse(SOURCE).expect("impact source parses"))
        .expect("impact source lowers")
}

fn revision(program: &elaborate::CompiledProgram, name: &str) -> Revision {
    program
        .revision(&frontend::Name(name.to_owned()))
        .expect("named Revision resolves")
        .clone()
}

fn relation(program: &elaborate::CompiledProgram, value: &str) -> ReferentId {
    program
        .designations()
        .global(value)
        .expect("relation designation resolves")
}

fn role(program: &elaborate::CompiledProgram, relation: &ReferentId, value: &str) -> RoleId {
    program
        .designations()
        .role(relation, value)
        .expect("role designation resolves")
}

fn referent(program: &elaborate::CompiledProgram, revision: &Revision, local: &str) -> ReferentId {
    program
        .designations()
        .scoped(revision.model().id(), local)
        .expect("scoped referent designation resolves")
}

fn assertion(
    program: &elaborate::CompiledProgram,
    revision: &Revision,
    relation_name: &str,
    roles: &[(&str, &str)],
) -> RelationalContent {
    let relation = relation(program, relation_name);
    RelationalContent::new(
        relation.clone(),
        roles
            .iter()
            .map(|(role_name, local)| {
                (
                    role(program, &relation, role_name),
                    Term::referent(referent(program, revision, local)),
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
    .expect("typed assertion is valid")
}

fn imports(
    program: &elaborate::CompiledProgram,
    revision: &Revision,
    consumer: &str,
    dependency: &str,
) -> RelationalContent {
    assertion(
        program,
        revision,
        "impact/imports",
        &[("consumer", consumer), ("dependency", dependency)],
    )
}

fn changes(
    program: &elaborate::CompiledProgram,
    revision: &Revision,
    change: &str,
    component: &str,
) -> RelationalContent {
    assertion(
        program,
        revision,
        "impact/changes",
        &[("change", change), ("component", component)],
    )
}

fn affected(
    program: &elaborate::CompiledProgram,
    revision: &Revision,
    change: &str,
    consumer: &str,
) -> RelationalContent {
    assertion(
        program,
        revision,
        "impact/affected",
        &[("change", change), ("consumer", consumer)],
    )
}

fn limits() -> derive::Limits {
    derive::Limits::new(100, 10, 10_000)
}

fn support_limits() -> derive::SupportLimits {
    derive::SupportLimits::new(limits(), 10_000, 100)
}

fn intervention_limits() -> InterventionLimits {
    InterventionLimits::new(limits(), 10_000, 100).with_support_limits(support_limits())
}

fn canonical_sets(mut sets: Vec<Vec<RelationalContent>>) -> Vec<Vec<RelationalContent>> {
    for set in &mut sets {
        set.sort();
    }
    sets.sort();
    sets
}

#[test]
fn authored_source_has_referents_revisions_and_requests() {
    let program = impact();
    let base = revision(&program, "impact");
    let successor = revision(&program, "impact/adopt-south");
    assert_eq!(base.model().admitted_contents().len(), 11);
    for local in [
        "North",
        "Store",
        "Relay",
        "Beagle",
        "South",
        "compiler-change",
    ] {
        assert!(
            base.model()
                .referents()
                .contains_key(&referent(&program, &base, local))
        );
    }
    assert_eq!(
        RevisionDiff::between(&base, &successor)
            .expect("same declarations diff")
            .added(),
        [imports(&program, &base, "South", "North")]
    );
    assert_eq!(program.requests().len(), 5);
}

#[test]
fn north_has_two_independent_minimal_supports_and_four_prevention_sets() {
    let program = impact();
    let base = revision(&program, "impact");
    let target = affected(&program, &base, "compiler-change", "North");
    let supports = derive::support_frontier(&base, &target, support_limits())
        .expect("support frontier computes");
    assert_eq!(supports.status(), SupportStatus::Complete);
    assert_eq!(supports.supports().len(), 2);
    assert_eq!(
        canonical_sets(
            supports
                .supports()
                .iter()
                .map(|support| support.assertions().to_vec())
                .collect(),
        ),
        canonical_sets(vec![
            vec![
                changes(&program, &base, "compiler-change", "Beagle"),
                imports(&program, &base, "North", "Relay"),
                imports(&program, &base, "Relay", "Beagle"),
            ],
            vec![
                changes(&program, &base, "compiler-change", "Beagle"),
                imports(&program, &base, "North", "Store"),
                imports(&program, &base, "Store", "Beagle"),
            ],
        ]),
    );
    let all = execution::why_all(&base, &target, support_limits())
        .expect("why all computes")
        .expect("target follows");
    assert!(all.is_complete());
    assert_eq!(all.alternative_count(), 2);

    let prevented = intervention::prevent_all_minimal(
        &base,
        target,
        vec![relation(&program, "impact/imports")],
        intervention_limits(),
    )
    .expect("prevention computes");
    let PreventAll::Complete(prevented) = prevented else {
        panic!("finite prevention frontier must be complete");
    };
    assert_eq!(prevented.len(), 4);
    assert_eq!(
        canonical_sets(
            prevented
                .iter()
                .map(|item| item.withdrawals().to_vec())
                .collect(),
        ),
        canonical_sets(vec![
            vec![
                imports(&program, &base, "North", "Relay"),
                imports(&program, &base, "North", "Store"),
            ],
            vec![
                imports(&program, &base, "North", "Relay"),
                imports(&program, &base, "Store", "Beagle"),
            ],
            vec![
                imports(&program, &base, "North", "Store"),
                imports(&program, &base, "Relay", "Beagle"),
            ],
            vec![
                imports(&program, &base, "Relay", "Beagle"),
                imports(&program, &base, "Store", "Beagle"),
            ],
        ]),
    );
}

#[test]
fn successor_retains_consequence_while_losing_one_support() {
    let source = SOURCE.replace(
        "impact/adopt-south: Revision",
        "impact/redundant-path-withdrawn: Revision\n    from: impact\n    withdraw:\n        North imports Relay\n        Relay imports Beagle\n\nimpact/adopt-south: Revision",
    );
    let program = elaborate::compile(frontend::parse(&source).expect("impact source parses"))
        .expect("impact source lowers");
    let base = revision(&program, "impact");
    let successor = revision(&program, "impact/redundant-path-withdrawn");
    let target = affected(&program, &base, "compiler-change", "North");
    let diff = SemanticDiff::between(&base, &successor, support_limits()).expect("semantic diff");
    assert!(!diff.entailed_removed().contains(&target));
    let change = diff
        .changed_supports()
        .iter()
        .find(|change| change.consequence() == &target)
        .expect("retained consequence has support change");
    assert!(change.added().is_empty());
    assert_eq!(change.removed().len(), 1);
    assert_eq!(change.retained().len(), 1);
}

#[test]
fn typed_active_domain_yields_four_complete_additions() {
    let program =
        elaborate::compile(frontend::parse(ACHIEVEMENT_SOURCE).expect("achievement source parses"))
            .expect("achievement source lowers");
    let base = revision(&program, "choice");
    let target = assertion(
        &program,
        &base,
        "choice/reached",
        &[("start", "South"), ("state", "Ready")],
    );
    let achieved = intervention::achieve_all_minimal(
        &base,
        target,
        vec![relation(&program, "choice/selects")],
        intervention_limits(),
    )
    .expect("achievement computes");
    let achieved = match achieved {
        AchieveAll::Complete(items) => items,
        other => panic!("typed active domain must have a complete frontier: {other:?}"),
    };
    assert_eq!(achieved.len(), 4);
    assert_eq!(
        canonical_sets(
            achieved
                .iter()
                .map(|item| item.admissions().to_vec())
                .collect(),
        ),
        canonical_sets(vec![
            vec![assertion(
                &program,
                &base,
                "choice/selects",
                &[("start", "South"), ("option", "Beagle")],
            )],
            vec![assertion(
                &program,
                &base,
                "choice/selects",
                &[("start", "South"), ("option", "North")],
            )],
            vec![assertion(
                &program,
                &base,
                "choice/selects",
                &[("start", "South"), ("option", "Relay")],
            )],
            vec![assertion(
                &program,
                &base,
                "choice/selects",
                &[("start", "South"), ("option", "Store")],
            )],
        ]),
    );
}
