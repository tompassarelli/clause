//! Typed acceptance checks for the redundant dependency-impact model.

use std::collections::BTreeMap;

use clause::{
    delta::RevisionDiff,
    derive::{self, SupportStatus},
    elaborate, execution, frontend,
    intervention::{self, AchieveAll, Incomplete, InterventionLimits, PreventAll},
    kernel::{self, Clause, EntityId, Name, RelationId, Revision, RoleId, Term},
    semantic_diff::SemanticDiff,
};

const SOURCE: &str = include_str!("../examples/impact.clause");

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

fn name(value: &str) -> Name {
    Name::new(value.to_owned()).expect("valid stable name")
}

fn relation(value: &str) -> RelationId {
    RelationId::new(name(value)).expect("valid Relation identity")
}

fn role(value: &str) -> RoleId {
    RoleId::new(name(value)).expect("valid Role identity")
}

fn entity(revision: &Revision, local: &str) -> EntityId {
    revision
        .model()
        .entities()
        .iter()
        .find(|candidate| candidate.local().as_str() == local)
        .expect("admitted entity exists")
        .clone()
}

fn assertion(revision: &Revision, relation_name: &str, roles: &[(&str, &str)]) -> Clause {
    Clause::new(
        relation(relation_name),
        roles
            .iter()
            .map(|(role_name, local)| (role(role_name), Term::entity(entity(revision, local))))
            .collect::<BTreeMap<_, _>>(),
    )
    .expect("typed assertion is valid")
}

fn imports(revision: &Revision, consumer: &str, dependency: &str) -> Clause {
    assertion(
        revision,
        "impact/imports",
        &[("consumer", consumer), ("dependency", dependency)],
    )
}

fn changes(revision: &Revision, change: &str, component: &str) -> Clause {
    assertion(
        revision,
        "impact/changes",
        &[("change", change), ("component", component)],
    )
}

fn affected(revision: &Revision, change: &str, consumer: &str) -> Clause {
    assertion(
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

fn canonical_sets(mut sets: Vec<Vec<Clause>>) -> Vec<Vec<Clause>> {
    for set in &mut sets {
        set.sort();
    }
    sets.sort();
    sets
}

#[test]
fn authored_source_has_typed_entities_revisions_and_requests() {
    let program = impact();
    let base = revision(&program, "impact");
    let successor = revision(&program, "impact/adopt-south");
    assert_eq!(base.model().assertions().len(), 5);
    assert_eq!(base.model().entities().len(), 6);
    assert_eq!(
        RevisionDiff::between(&base, &successor)
            .expect("same declarations diff")
            .added(),
        [imports(&base, "South", "North")]
    );
    assert_eq!(program.requests().len(), 5);
}

#[test]
fn north_has_two_independent_minimal_supports_and_four_prevention_sets() {
    let base = revision(&impact(), "impact");
    let target = affected(&base, "compiler-change", "North");
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
                changes(&base, "compiler-change", "Beagle"),
                imports(&base, "North", "Relay"),
                imports(&base, "Relay", "Beagle"),
            ],
            vec![
                changes(&base, "compiler-change", "Beagle"),
                imports(&base, "North", "Store"),
                imports(&base, "Store", "Beagle"),
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
        vec![relation("impact/imports")],
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
                .map(|item| item.delta().withdrawals().to_vec())
                .collect(),
        ),
        canonical_sets(vec![
            vec![
                imports(&base, "North", "Relay"),
                imports(&base, "North", "Store")
            ],
            vec![
                imports(&base, "North", "Relay"),
                imports(&base, "Store", "Beagle")
            ],
            vec![
                imports(&base, "North", "Store"),
                imports(&base, "Relay", "Beagle")
            ],
            vec![
                imports(&base, "Relay", "Beagle"),
                imports(&base, "Store", "Beagle")
            ],
        ]),
    );
}

#[test]
fn successor_retains_consequence_while_losing_one_support() {
    let program = impact();
    let base = revision(&program, "impact");
    let successor = kernel::Delta::new(
        base.identity().clone(),
        Vec::new(),
        vec![
            imports(&base, "North", "Relay"),
            imports(&base, "Relay", "Beagle"),
        ],
    )
    .expect("typed withdrawal Delta")
    .apply(&base)
    .expect("successor applies");
    let target = affected(&base, "compiler-change", "North");
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
fn typed_active_domain_yields_four_south_additions() {
    let base = revision(&impact(), "impact");
    let target = affected(&base, "compiler-change", "South");
    let achieved = intervention::achieve_all_minimal(
        &base,
        target,
        vec![relation("impact/imports")],
        InterventionLimits::new(limits(), 100, 100).with_support_limits(support_limits()),
    )
    .expect("achievement computes");
    let achieved = match achieved {
        AchieveAll::Complete(items) => items,
        AchieveAll::Incomplete {
            interventions,
            reason: Incomplete::CandidateBudgetExhausted,
        } => interventions,
        other => panic!("typed active domain must discover additions: {other:?}"),
    };
    assert_eq!(achieved.len(), 4);
    assert_eq!(
        achieved
            .iter()
            .map(|item| item.delta().admissions().to_vec())
            .collect::<Vec<_>>(),
        vec![
            vec![imports(&base, "South", "Beagle")],
            vec![imports(&base, "South", "North")],
            vec![imports(&base, "South", "Relay")],
            vec![imports(&base, "South", "Store")],
        ],
    );
}
