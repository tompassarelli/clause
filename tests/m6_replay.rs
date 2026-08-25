//! M6 deterministic incremental state/replay acceptance.

use std::collections::BTreeMap;

use clause::{
    kernel::{
        AssertionOccurrence, DerivationRule, Judgment, JudgmentKind, JudgmentStatus,
        JudgmentTarget, Model, Pattern, PatternId, Referent, ReferentId, RelationShape,
        RelationalContent, Role, RoleId, Term, Transition, UniversalLaw,
    },
    runtime::{
        Presence, RuntimePolicy, RuntimeSession, StateDelta, StateDiff, TransitionEvent,
        reload_session,
    },
    wire,
};

const MODEL: u8 = 1;
const SUBJECT: u8 = 2;
const SOURCE: u8 = 3;
const ACTIVE: u8 = 10;
const CONSEQUENCE: u8 = 11;
const DISCONNECTED: u8 = 12;
const COLLECTED: u8 = 13;
const ACTIVE_ONE: u8 = 20;
const ACTIVE_TWO: u8 = 21;
const DISCONNECTED_ONE: u8 = 22;
const ACTIVE_ONE_JUDGMENT: u8 = 30;
const ACTIVE_TWO_JUDGMENT: u8 = 31;
const DISCONNECTED_JUDGMENT: u8 = 32;
const POLICY: u8 = 40;
const ACTIVE_TO_CONSEQUENCE_RULE: u8 = 50;
const ACTIVE_TO_CONSEQUENCE_LAW: u8 = 51;
const CONSEQUENCE_RECURSION_RULE: u8 = 52;
const CONSEQUENCE_RECURSION_LAW: u8 = 53;
const COLLECT_TRANSITION: u8 = 60;
const COLLECT_ONE_EVENT: u8 = 61;
const COLLECT_TWO_EVENT: u8 = 62;
const COLLECTED_ONE: u8 = 63;
const COLLECTED_TWO: u8 = 64;

const SUBJECT_ROLE: u8 = 1;
const SUBJECT_PATTERN: u8 = 1;

fn referent(seed: u8) -> ReferentId {
    ReferentId::from_digest([seed; 32])
}

fn role(seed: u8) -> RoleId {
    RoleId::from_digest([seed; 32])
}

fn pattern(seed: u8) -> PatternId {
    PatternId::from_digest([seed; 32])
}

fn content(relation: u8, term: Term) -> RelationalContent {
    RelationalContent::new(
        referent(relation),
        BTreeMap::from([(role(SUBJECT_ROLE), term)]),
    )
    .unwrap()
}

fn ground(relation: u8) -> RelationalContent {
    content(relation, Term::referent(referent(SUBJECT)))
}

fn pattern_content(relation: u8) -> RelationalContent {
    content(relation, Term::pattern(pattern(SUBJECT_PATTERN)))
}

fn shape(relation: u8) -> RelationShape {
    let subject = role(SUBJECT_ROLE);
    RelationShape::new(
        referent(relation),
        BTreeMap::from([(
            subject.clone(),
            Role::new(subject.clone(), Vec::new()).unwrap(),
        )]),
        Vec::new(),
    )
    .unwrap()
}

fn occurrence(id: u8, relation: u8) -> AssertionOccurrence {
    AssertionOccurrence::new(
        referent(id),
        ground(relation).id().clone(),
        referent(SOURCE),
        referent(MODEL),
    )
}

fn judgment(id: u8, occurrence: u8) -> Judgment {
    Judgment::new(
        referent(id),
        referent(MODEL),
        referent(MODEL),
        JudgmentTarget::Occurrence(referent(occurrence)),
        JudgmentKind::Admitted {
            policy: referent(POLICY),
            basis: vec![referent(SOURCE)],
        },
        JudgmentStatus::Affirmed,
    )
}

fn fixture() -> clause::kernel::Revision {
    let active = ground(ACTIVE);
    let consequence = ground(CONSEQUENCE);
    let disconnected = ground(DISCONNECTED);
    let collected = ground(COLLECTED);
    let active_pattern = pattern_content(ACTIVE);
    let consequence_pattern = pattern_content(CONSEQUENCE);

    let active_premise = Pattern::new(vec![active_pattern.id().clone()]).unwrap();
    let consequence_pattern_set = Pattern::new(vec![consequence_pattern.id().clone()]).unwrap();
    let derivation_rules = vec![
        DerivationRule::new(
            referent(ACTIVE_TO_CONSEQUENCE_RULE),
            referent(ACTIVE_TO_CONSEQUENCE_LAW),
            referent(MODEL),
            referent(MODEL),
            active_premise.clone(),
            consequence_pattern_set.clone(),
        )
        .unwrap(),
        DerivationRule::new(
            referent(CONSEQUENCE_RECURSION_RULE),
            referent(CONSEQUENCE_RECURSION_LAW),
            referent(MODEL),
            referent(MODEL),
            consequence_pattern_set.clone(),
            consequence_pattern_set.clone(),
        )
        .unwrap(),
    ];
    let laws = vec![
        UniversalLaw::new(
            referent(ACTIVE_TO_CONSEQUENCE_LAW),
            referent(MODEL),
            active_premise,
            consequence_pattern_set.clone(),
        ),
        UniversalLaw::new(
            referent(CONSEQUENCE_RECURSION_LAW),
            referent(MODEL),
            consequence_pattern_set.clone(),
            consequence_pattern_set,
        ),
    ];
    let all_referents = [
        MODEL,
        SUBJECT,
        SOURCE,
        ACTIVE,
        CONSEQUENCE,
        DISCONNECTED,
        COLLECTED,
        ACTIVE_ONE,
        ACTIVE_TWO,
        DISCONNECTED_ONE,
        ACTIVE_ONE_JUDGMENT,
        ACTIVE_TWO_JUDGMENT,
        DISCONNECTED_JUDGMENT,
        POLICY,
        ACTIVE_TO_CONSEQUENCE_RULE,
        ACTIVE_TO_CONSEQUENCE_LAW,
        CONSEQUENCE_RECURSION_RULE,
        CONSEQUENCE_RECURSION_LAW,
        COLLECT_TRANSITION,
        COLLECT_ONE_EVENT,
        COLLECT_TWO_EVENT,
        COLLECTED_ONE,
        COLLECTED_TWO,
    ];
    let model = Model::with_distinctions(
        referent(MODEL),
        all_referents
            .map(referent)
            .into_iter()
            .map(|id| (id.clone(), Referent::new(id)))
            .collect(),
        [
            active.clone(),
            consequence,
            disconnected,
            collected.clone(),
            active_pattern,
            consequence_pattern,
        ]
        .into_iter()
        .map(|content| (content.id().clone(), content))
        .collect(),
        [ACTIVE, CONSEQUENCE, DISCONNECTED, COLLECTED]
            .map(shape)
            .into_iter()
            .map(|shape| (shape.referent().clone(), shape))
            .collect(),
        BTreeMap::new(),
        vec![
            occurrence(ACTIVE_ONE, ACTIVE),
            occurrence(ACTIVE_TWO, ACTIVE),
            occurrence(DISCONNECTED_ONE, DISCONNECTED),
        ],
        Vec::new(),
        derivation_rules,
        laws,
        Vec::new(),
        Vec::new(),
        vec![
            Transition::new(
                referent(COLLECT_TRANSITION),
                active.id().clone(),
                collected.id().clone(),
            )
            .unwrap(),
        ],
        vec![
            judgment(ACTIVE_ONE_JUDGMENT, ACTIVE_ONE),
            judgment(ACTIVE_TWO_JUDGMENT, ACTIVE_TWO),
            judgment(DISCONNECTED_JUDGMENT, DISCONNECTED_ONE),
        ],
    )
    .unwrap();
    wire::admit(model)
}

fn policy() -> RuntimePolicy {
    RuntimePolicy::new(referent(POLICY), 128, 512).unwrap()
}

fn event(event: u8, target: u8, successor: u8) -> TransitionEvent {
    TransitionEvent::new(
        referent(event),
        referent(COLLECT_TRANSITION),
        referent(target),
        referent(successor),
        referent(MODEL),
    )
}

fn event_wire(event: &TransitionEvent) -> String {
    format!(
        "[\"event\",\"{}\",[\"transition\",\"{}\"],[\"target\",\"{}\"],[\"successor\",\"{}\"],[\"scope\",\"{}\"]]",
        event.id().as_str(),
        event.transition().as_str(),
        event.target_occurrence().as_str(),
        event.successor_occurrence().as_str(),
        event.scope().as_str(),
    )
}

#[test]
fn occurrence_exact_incremental_replay_is_canonical_and_rejects_tampering() {
    let revision = fixture();
    let active = ground(ACTIVE);
    let consequence = ground(CONSEQUENCE);
    let disconnected = ground(DISCONNECTED);
    let collected = ground(COLLECTED);
    let first_event = event(COLLECT_ONE_EVENT, ACTIVE_ONE, COLLECTED_ONE);
    let second_event = event(COLLECT_TWO_EVENT, ACTIVE_TWO, COLLECTED_TWO);

    let root = RuntimeSession::start(&revision, policy()).unwrap();
    assert_eq!(root.latest().support_roots(active.id()).len(), 2);
    assert_eq!(root.latest().support_roots(consequence.id()).len(), 2);
    let disconnected_support = root.latest().support_roots(disconnected.id());

    let after_first = root
        .transition(&revision, vec![first_event.clone()])
        .unwrap();
    assert!(after_first.latest().contains_content(active.id()));
    assert!(after_first.latest().contains_content(consequence.id()));
    assert!(after_first.latest().contains_content(collected.id()));
    assert_eq!(after_first.latest().support_roots(active.id()).len(), 1);
    assert_eq!(
        after_first.latest().support_roots(consequence.id()).len(),
        1
    );
    assert_eq!(
        after_first.latest().support_roots(disconnected.id()),
        disconnected_support
    );
    assert!(
        !after_first
            .latest()
            .work()
            .touched_contents()
            .contains(disconnected.id())
    );

    let first_diff = StateDiff::between(root.latest(), after_first.latest(), &revision).unwrap();
    assert_eq!(first_diff.occurrence_withdrawals().len(), 1);
    assert_eq!(first_diff.occurrence_admissions().len(), 1);
    assert!(first_diff.content_changes().iter().any(|change| {
        change.content() == collected.id()
            && change.before() == Presence::Absent
            && change.after() == Presence::Present
    }));
    assert!(first_diff
        .support_withdrawals()
        .iter()
        .any(|(content, roots)| content == consequence.id() && roots == &[referent(ACTIVE_ONE)]));
    assert!(first_diff.retained_equalities().is_empty());
    assert!(first_diff.authorized_equivalences().is_empty());

    let after_second = after_first
        .transition(&revision, vec![second_event.clone()])
        .unwrap();
    assert!(!after_second.latest().contains_content(active.id()));
    assert!(!after_second.latest().contains_content(consequence.id()));
    assert!(after_second.latest().contains_content(disconnected.id()));
    assert_eq!(
        after_second.latest().support_roots(disconnected.id()),
        disconnected_support
    );

    let replayed = RuntimeSession::replay(
        &revision,
        policy(),
        vec![vec![first_event.clone()], vec![second_event.clone()]],
    )
    .unwrap();
    assert_eq!(replayed, after_second);
    assert_eq!(replayed.canonical_bytes(), after_second.canonical_bytes());
    assert_eq!(
        reload_session(&after_second.canonical_bytes(), &revision).unwrap(),
        after_second
    );

    let fake_state = format!("state-sha256-{}", "00".repeat(32));
    let predecessor_fragment = format!("[\"successor\",\"{}\"", root.latest().identity().as_str());
    let tampered_predecessor = after_second.canonical_bytes().replacen(
        &predecessor_fragment,
        &format!("[\"successor\",\"{fake_state}\""),
        1,
    );
    assert!(reload_session(&tampered_predecessor, &revision).is_err());

    let model_identity = revision.identity().to_string();
    let tampered_model = after_second.canonical_bytes().replacen(
        &model_identity,
        &format!("rev-sha256-{}", "00".repeat(32)),
        1,
    );
    assert!(reload_session(&tampered_model, &revision).is_err());

    let tampered_policy = after_second.canonical_bytes().replacen(
        referent(POLICY).as_str(),
        referent(99).as_str(),
        1,
    );
    assert!(reload_session(&tampered_policy, &revision).is_err());

    let withdrawal_fragment = format!("[\"withdraw\",[\"{}\"]]", referent(ACTIVE_ONE).as_str());
    let tampered_delta = after_second.canonical_bytes().replacen(
        &withdrawal_fragment,
        &format!(
            "[\"withdraw\",[\"{}\"]]",
            referent(DISCONNECTED_ONE).as_str()
        ),
        1,
    );
    assert!(reload_session(&tampered_delta, &revision).is_err());

    let batched = root
        .transition(&revision, vec![first_event.clone(), second_event.clone()])
        .unwrap();
    let ordered_tick = format!(
        "[\"inputs\",[[\"events\",[{},{}]]]]",
        event_wire(&first_event),
        event_wire(&second_event)
    );
    let reversed_tick = format!(
        "[\"inputs\",[[\"events\",[{},{}]]]]",
        event_wire(&second_event),
        event_wire(&first_event)
    );
    let tampered_order = batched
        .canonical_bytes()
        .replacen(&ordered_tick, &reversed_tick, 1);
    assert_ne!(tampered_order, batched.canonical_bytes());
    assert!(reload_session(&tampered_order, &revision).is_err());

    let explicit = root
        .apply_delta(
            &revision,
            StateDelta::new(
                vec![referent(ACTIVE_ONE)],
                vec![AssertionOccurrence::new(
                    referent(COLLECTED_ONE),
                    collected.id().clone(),
                    referent(COLLECT_ONE_EVENT),
                    referent(MODEL),
                )],
            )
            .unwrap(),
        )
        .unwrap();
    assert!(explicit.latest().contains_content(active.id()));
    assert!(explicit.latest().contains_content(consequence.id()));
    assert_eq!(
        reload_session(&explicit.canonical_bytes(), &revision).unwrap(),
        explicit
    );

    let conflict = root.transition(
        &revision,
        vec![
            first_event,
            event(COLLECT_TWO_EVENT, ACTIVE_ONE, COLLECTED_TWO),
        ],
    );
    assert!(
        conflict
            .unwrap_err()
            .to_string()
            .contains("conflicting writes")
    );
}
