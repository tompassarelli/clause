use super::*;
use crate::{
    kernel::{
        AssertionOccurrence, Cardinality, DerivationRule, Judgment, JudgmentKind, JudgmentStatus,
        JudgmentTarget, LookupMode, Model, Pattern, PatternId, Referent, ReferentId, RelationShape,
        RelationalContent, Revision, Role, RoleId, Term,
    },
    wire,
};
use std::collections::BTreeMap;

fn referent_id(value: &str) -> ReferentId {
    ReferentId::from_digest(wire::sha256_digest(value.as_bytes()))
}
fn role_id(value: &str) -> RoleId {
    RoleId::from_digest(wire::sha256_digest(value.as_bytes()))
}
fn pattern_id(value: &str) -> PatternId {
    PatternId::from_digest(wire::sha256_digest(value.as_bytes()))
}
fn referent(value: &str) -> Term {
    Term::referent(referent_id(value))
}
fn variable(value: &str) -> Term {
    Term::pattern(pattern_id(value))
}
fn clause(relation: &ReferentId, from: Term, to: Term) -> RelationalContent {
    RelationalContent::new(
        relation.clone(),
        BTreeMap::from([(role_id("from"), from), (role_id("to"), to)]),
    )
    .unwrap()
}
fn relation(id: &ReferentId) -> RelationShape {
    let from = Role::new(role_id("from"), Vec::new()).unwrap();
    let to = Role::new(role_id("to"), Vec::new()).unwrap();
    RelationShape::new(
        id.clone(),
        BTreeMap::from([
            (from.id().clone(), from.clone()),
            (to.id().clone(), to.clone()),
        ]),
        vec![
            LookupMode::finite(
                vec![from.id().clone()],
                vec![to.id().clone()],
                Cardinality::Many,
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn rule(
    identity: &str,
    premises: Vec<RelationalContent>,
    conclusion: RelationalContent,
) -> (DerivationRule, Vec<RelationalContent>) {
    let rule = DerivationRule::new(
        referent_id(identity),
        referent_id("map/scope"),
        referent_id("map/authority"),
        Pattern::new(premises.iter().map(|item| item.id().clone()).collect()).unwrap(),
        Pattern::new(vec![conclusion.id().clone()]).unwrap(),
    )
    .unwrap();
    let mut contents = premises;
    contents.push(conclusion);
    (rule, contents)
}

fn declare(referents: &mut BTreeMap<ReferentId, Referent>, id: ReferentId) {
    referents.insert(id.clone(), Referent::new(id));
}

fn declare_content_referents(
    referents: &mut BTreeMap<ReferentId, Referent>,
    content: &RelationalContent,
) {
    declare(referents, content.relation().clone());
    for term in content.roles().values() {
        if let Term::Referent(id) = term {
            declare(referents, id.clone());
        }
    }
}

fn revision(
    assertions: Vec<RelationalContent>,
    rule_fixtures: Vec<(DerivationRule, Vec<RelationalContent>)>,
) -> Revision {
    let model_id = referent_id("map");
    let links = referent_id("map/links");
    let reaches = referent_id("map/reaches");
    let source = referent_id("map/source");
    let scope = referent_id("map/scope");
    let policy = referent_id("map/admission-policy");
    let mut referents = BTreeMap::new();
    for id in [
        model_id.clone(),
        links.clone(),
        reaches.clone(),
        source.clone(),
        scope.clone(),
        policy.clone(),
        referent_id("map/authority"),
    ] {
        declare(&mut referents, id);
    }
    for value in ["North", "South", "Store", "Relay", "Beagle"] {
        declare(&mut referents, referent_id(value));
    }
    let mut relational_contents = BTreeMap::new();
    for content in &assertions {
        declare_content_referents(&mut referents, content);
        relational_contents.insert(content.id().clone(), content.clone());
    }
    let mut rules = Vec::new();
    for (rule, contents) in rule_fixtures {
        declare(&mut referents, rule.id().clone());
        declare(&mut referents, rule.scope().clone());
        declare(&mut referents, rule.authority().clone());
        for content in contents {
            declare_content_referents(&mut referents, &content);
            relational_contents.insert(content.id().clone(), content);
        }
        rules.push(rule);
    }
    let mut occurrences = Vec::new();
    let mut judgments = Vec::new();
    for assertion in assertions {
        let occurrence_id = referent_id(&format!(
            "map/assertion-occurrence/{}",
            assertion.id().as_str()
        ));
        let judgment_id = referent_id(&format!(
            "map/admission-judgment/{}",
            assertion.id().as_str()
        ));
        declare(&mut referents, occurrence_id.clone());
        declare(&mut referents, judgment_id.clone());
        occurrences.push(AssertionOccurrence::new(
            occurrence_id.clone(),
            assertion.id().clone(),
            source.clone(),
            model_id.clone(),
        ));
        judgments.push(Judgment::new(
            judgment_id,
            model_id.clone(),
            model_id.clone(),
            JudgmentTarget::Occurrence(occurrence_id),
            JudgmentKind::Admitted {
                policy: policy.clone(),
                basis: Vec::new(),
            },
            JudgmentStatus::Affirmed,
        ));
    }
    wire::admit(
        Model::with_distinctions(
            model_id,
            referents,
            relational_contents,
            BTreeMap::from([
                (links.clone(), relation(&links)),
                (reaches.clone(), relation(&reaches)),
            ]),
            occurrences,
            Vec::new(),
            rules,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            judgments,
        )
        .unwrap(),
    )
}
fn limits() -> Limits {
    Limits::new(100, 10, 10_000)
}
fn chain_rules() -> Vec<(DerivationRule, Vec<RelationalContent>)> {
    let links = referent_id("map/links");
    let reaches = referent_id("map/reaches");
    let source = variable("source");
    let middle = variable("middle");
    let destination = variable("destination");
    vec![
        rule(
            "map/direct",
            vec![clause(&links, source.clone(), destination.clone())],
            clause(&reaches, source.clone(), destination.clone()),
        ),
        rule(
            "map/recursive",
            vec![
                clause(&reaches, source.clone(), middle.clone()),
                clause(&links, middle, destination.clone()),
            ],
            clause(&reaches, source, destination),
        ),
    ]
}
fn asserted(relation: &str, from: &str, to: &str) -> RelationalContent {
    clause(&referent_id(relation), referent(from), referent(to))
}
fn find_plan(revision: &Revision, from: &str) -> crate::kernel::FindPlan {
    let target = pattern_id("target");
    crate::kernel::FindPlan::new(
        revision.model(),
        &clause(
            &referent_id("map/reaches"),
            referent(from),
            Term::pattern(target.clone()),
        ),
        target,
    )
    .unwrap()
}

#[test]
fn find_discriminates_known_referent_bindings_and_returns_referent_terms() {
    let revision = revision(
        vec![
            asserted("map/links", "North", "Store"),
            asserted("map/links", "South", "Relay"),
        ],
        chain_rules(),
    );
    assert_eq!(
        find(&revision, &find_plan(&revision, "North"), limits()).unwrap(),
        vec![referent("Store")]
    );
    assert_eq!(
        find(&revision, &find_plan(&revision, "South"), limits()).unwrap(),
        vec![referent("Relay")]
    );
}

#[test]
fn find_returns_recursive_derived_referents_in_canonical_order() {
    let revision = revision(
        vec![
            asserted("map/links", "North", "Store"),
            asserted("map/links", "Store", "Beagle"),
        ],
        chain_rules(),
    );
    let result = find(&revision, &find_plan(&revision, "North"), limits()).unwrap();
    assert_eq!(result, vec![referent("Beagle"), referent("Store")]);
    assert!(result.iter().all(|term| matches!(term, Term::Referent(_))));
}

#[test]
fn why_projects_one_canonical_revision_scoped_proof() {
    let revision = revision(vec![asserted("map/links", "North", "Store")], chain_rules());
    let target = asserted("map/reaches", "North", "Store");
    let proof = why(&revision, &target, limits()).unwrap().unwrap();
    assert_eq!(proof.revision, *revision.identity());
    assert_eq!(proof.why.root, 0);
    assert!(
        matches!(&proof.why.witnesses[0].witness, Witness::Derived { rule, .. } if rule == &referent_id("map/direct"))
    );
}

#[test]
fn why_all_projects_two_independent_minimal_supports() {
    let revision = revision(
        vec![
            asserted("map/links", "North", "Store"),
            asserted("map/links", "Store", "Beagle"),
            asserted("map/links", "North", "Relay"),
            asserted("map/links", "Relay", "Beagle"),
        ],
        chain_rules(),
    );
    let all = why_all(
        &revision,
        &asserted("map/reaches", "North", "Beagle"),
        SupportLimits::new(limits(), 100, 10),
    )
    .unwrap()
    .unwrap();
    assert!(all.is_complete());
    assert_eq!(all.alternative_count(), 2);
    assert!(
        all.alternatives
            .iter()
            .all(|alternative| alternative.assertions.len() == 2)
    );
}

#[test]
fn why_all_marks_a_bounded_frontier_incomplete() {
    let revision = revision(vec![asserted("map/links", "North", "Store")], chain_rules());
    let all = why_all(
        &revision,
        &asserted("map/reaches", "North", "Store"),
        SupportLimits::new(limits(), 0, 10),
    )
    .unwrap()
    .unwrap();
    assert!(!all.is_complete());
    assert!(all.alternatives.is_empty());
}

#[test]
fn proof_is_deterministic_when_assertion_order_changes() {
    let assertions = vec![
        asserted("map/links", "North", "Store"),
        asserted("map/links", "Store", "Beagle"),
    ];
    let target = asserted("map/reaches", "North", "Beagle");
    let forward = revision(assertions.clone(), chain_rules());
    let reverse = revision(assertions.into_iter().rev().collect(), chain_rules());
    assert_eq!(
        why(&forward, &target, limits()).unwrap().unwrap().why,
        why(&reverse, &target, limits()).unwrap().unwrap().why
    );
}
