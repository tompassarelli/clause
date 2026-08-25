use super::{Limits, SupportLimits, SupportStatus, Witness, saturate, support_frontier};
use crate::{
    kernel::{
        AssertionOccurrence, Cardinality, DerivationRule, Judgment, JudgmentKind, JudgmentStatus,
        JudgmentTarget, LookupMode, Model, Pattern, PatternId, Referent, ReferentId, RelationShape,
        RelationalContent, Revision, Role, RoleId, Term, UniversalLaw,
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
fn role(value: &str) -> Role {
    Role::new(role_id(value), Vec::new()).unwrap()
}
fn variable(value: &str) -> Term {
    Term::pattern(pattern_id(value))
}
fn referent(value: &str) -> Term {
    Term::referent(referent_id(value))
}

fn relation(identity: &ReferentId) -> RelationShape {
    let from = role("from");
    let to = role("to");
    RelationShape::new(
        identity.clone(),
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

fn clause(identity: &ReferentId, from: Term, to: Term) -> RelationalContent {
    RelationalContent::new(
        identity.clone(),
        BTreeMap::from([(role_id("from"), from), (role_id("to"), to)]),
    )
    .unwrap()
}

fn rule(
    identity: &str,
    premises: Vec<RelationalContent>,
    conclusion: RelationalContent,
) -> (DerivationRule, UniversalLaw, Vec<RelationalContent>) {
    let law_id = referent_id(&format!("{identity}/governing-law"));
    let premise_pattern =
        Pattern::new(premises.iter().map(|item| item.id().clone()).collect()).unwrap();
    let conclusion_pattern = Pattern::new(vec![conclusion.id().clone()]).unwrap();
    let rule = DerivationRule::new(
        referent_id(identity),
        law_id.clone(),
        referent_id("map/scope"),
        referent_id("map/authority"),
        premise_pattern.clone(),
        conclusion_pattern.clone(),
    )
    .unwrap();
    let law = UniversalLaw::new(
        law_id,
        referent_id("map/scope"),
        premise_pattern,
        conclusion_pattern,
    );
    let mut contents = premises;
    contents.push(conclusion);
    (rule, law, contents)
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
    rule_fixtures: Vec<(DerivationRule, UniversalLaw, Vec<RelationalContent>)>,
) -> Revision {
    let model_id = referent_id("map");
    let reaches = referent_id("map/reaches");
    let links = referent_id("map/links");
    let source = referent_id("map/source");
    let scope = referent_id("map/scope");
    let policy = referent_id("map/admission-policy");
    let mut referents = BTreeMap::new();
    for id in [
        model_id.clone(),
        reaches.clone(),
        links.clone(),
        source.clone(),
        scope.clone(),
        policy.clone(),
        referent_id("map/authority"),
    ] {
        declare(&mut referents, id);
    }
    for value in [
        "Alpha", "Beta", "First", "North", "One", "Second", "Store", "Two", "Zulu",
    ] {
        declare(&mut referents, referent_id(value));
    }
    let mut relational_contents = BTreeMap::new();
    for content in &assertions {
        declare_content_referents(&mut referents, content);
        relational_contents.insert(content.id().clone(), content.clone());
    }
    let mut rules = Vec::new();
    let mut laws = Vec::new();
    for (rule, law, contents) in rule_fixtures {
        declare(&mut referents, rule.id().clone());
        declare(&mut referents, law.id().clone());
        declare(&mut referents, rule.scope().clone());
        declare(&mut referents, rule.authority().clone());
        for content in contents {
            declare_content_referents(&mut referents, &content);
            relational_contents.insert(content.id().clone(), content);
        }
        rules.push(rule);
        laws.push(law);
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
    let model = Model::with_distinctions(
        model_id,
        referents,
        relational_contents,
        BTreeMap::from([
            (reaches.clone(), relation(&reaches)),
            (links.clone(), relation(&links)),
        ]),
        BTreeMap::new(),
        occurrences,
        Vec::new(),
        rules,
        laws,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        judgments,
    )
    .unwrap();
    wire::admit(model)
}

#[test]
fn multi_round_closure_selects_canonical_referent_witnesses() {
    let reaches = referent_id("map/reaches");
    let links = referent_id("map/links");
    let subject = variable("subject");
    let destination = variable("destination");
    let copy = rule(
        "map/copy",
        vec![clause(&links, subject.clone(), destination.clone())],
        clause(&reaches, subject, destination),
    );
    let closure = saturate(
        &revision(
            vec![clause(&links, referent("North"), referent("Store"))],
            vec![copy],
        ),
        Limits::new(10, 10, 100),
    )
    .unwrap();
    let derived = clause(&reaches, referent("North"), referent("Store"));
    assert_eq!(closure.contents().len(), 2);
    assert_eq!(closure.proof(&derived).unwrap().generation(), 1);
    assert!(matches!(
        closure.proof(&derived).unwrap().witness(),
        Witness::Derived { .. }
    ));
}

#[test]
fn reversed_rule_source_order_admits_the_same_model_and_closure() {
    let reaches = referent_id("map/reaches");
    let links = referent_id("map/links");
    let subject = variable("subject");
    let destination = variable("destination");
    let rule_a = rule(
        "map/a-copy",
        vec![clause(&links, subject.clone(), destination.clone())],
        clause(&reaches, subject.clone(), destination.clone()),
    );
    let rule_z = rule(
        "map/z-copy",
        vec![clause(&links, subject.clone(), destination.clone())],
        clause(&reaches, subject, destination),
    );
    let assertions = vec![clause(&links, referent("North"), referent("Store"))];
    let forward = revision(assertions.clone(), vec![rule_a.clone(), rule_z.clone()]);
    let reversed = revision(assertions, vec![rule_z, rule_a]);
    assert_eq!(forward.model(), reversed.model());
    assert_eq!(
        saturate(&forward, Limits::new(10, 10, 100)).unwrap(),
        saturate(&reversed, Limits::new(10, 10, 100)).unwrap(),
    );
}

#[test]
fn support_frontier_remains_minimal_for_referents() {
    let reaches = referent_id("map/reaches");
    let links = referent_id("map/links");
    let subject = variable("subject");
    let destination = variable("destination");
    let copy = rule(
        "map/copy",
        vec![clause(&links, subject.clone(), destination.clone())],
        clause(&reaches, subject, destination),
    );
    let target = clause(&reaches, referent("North"), referent("Store"));
    let frontier = support_frontier(
        &revision(
            vec![clause(&links, referent("North"), referent("Store"))],
            vec![copy],
        ),
        &target,
        SupportLimits::new(Limits::new(10, 10, 100), 10, 10),
    )
    .unwrap();
    assert_eq!(frontier.status(), SupportStatus::Complete);
    assert_eq!(frontier.supports().len(), 1);
    assert_eq!(frontier.supports()[0].assertions().len(), 1);
}

#[test]
fn support_members_follow_the_canonical_proof_path() {
    let reaches = referent_id("map/reaches");
    let links = referent_id("map/links");
    let first = clause(&links, referent("Zulu"), referent("First"));
    let second = clause(&links, referent("Alpha"), referent("Second"));
    assert!(second < first);
    let intermediate = clause(&reaches, referent("Zulu"), referent("Zulu"));
    assert!(intermediate < second);
    let target = clause(&reaches, referent("North"), referent("Store"));
    let first_step = rule("map/path-first", vec![first.clone()], intermediate.clone());
    let path_rule = rule(
        "map/path-order",
        vec![intermediate, second.clone()],
        target.clone(),
    );
    let frontier = support_frontier(
        &revision(
            vec![second.clone(), first.clone()],
            vec![first_step, path_rule],
        ),
        &target,
        SupportLimits::new(Limits::new(10, 10, 100), 10, 10),
    )
    .unwrap();
    let support = &frontier.supports()[0];
    assert_eq!(support.assertion_key(), &[second.clone(), first.clone()]);
    assert_eq!(support.assertions(), &[first, second]);
}

#[test]
fn incomplete_frontier_does_not_expose_a_provisional_superset() {
    let reaches = referent_id("map/reaches");
    let links = referent_id("map/links");
    let alpha = clause(&links, referent("Alpha"), referent("One"));
    let beta = clause(&links, referent("Beta"), referent("Two"));
    let target = clause(&reaches, referent("North"), referent("Store"));
    let wide = rule(
        "map/a-wide",
        vec![alpha.clone(), beta.clone()],
        target.clone(),
    );
    let narrow = rule("map/z-narrow", vec![alpha.clone()], target.clone());
    let frontier = support_frontier(
        &revision(vec![alpha, beta], vec![wide, narrow]),
        &target,
        SupportLimits::new(Limits::new(10, 10, 100), 1, 10),
    )
    .unwrap();
    assert_eq!(frontier.status(), SupportStatus::ExpansionBudgetExhausted);
    assert!(frontier.supports().is_empty());
}

#[test]
fn absent_target_has_a_complete_empty_frontier_without_support_budget() {
    let reaches = referent_id("map/reaches");
    let links = referent_id("map/links");
    let target = clause(&reaches, referent("North"), referent("Store"));
    let frontier = support_frontier(
        &revision(
            vec![clause(&links, referent("Alpha"), referent("Beta"))],
            Vec::new(),
        ),
        &target,
        SupportLimits::new(Limits::new(10, 10, 100), 0, 0),
    )
    .unwrap();
    assert_eq!(frontier.status(), SupportStatus::Complete);
    assert!(frontier.supports().is_empty());
}
