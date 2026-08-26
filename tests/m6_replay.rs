//! M6 deterministic incremental state/replay acceptance.

use std::{
    collections::BTreeMap,
    env, fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use clause::{
    elaborate, frontend, generated,
    kernel::{
        AssertionOccurrence, DerivationRule, Judgment, JudgmentKind, JudgmentStatus,
        JudgmentTarget, Model, Pattern, PatternId, Referent, ReferentId, RelationShape,
        RelationalContent, Role, RoleId, Term, UniversalLaw,
    },
    runtime::{
        AuthorizationDecision, EffectOutcome, EffectRequest, EffectTrace, Presence,
        ReceiptOutcome, RuntimeInput, RuntimePolicy, RuntimeSession, StateDelta, StateDiff,
        TransitionEvent, reload_effect_request, reload_effect_trace, reload_session,
    },
    wire,
};

const ONE_COIN_SOURCE: &str = "Entity\nState\nOwner\nPolicy\n\ncoin/state: RelationShape\n  {coin: Entity} state {state: State}\n  mode coin -> state: one\n\ncoin/owner: RelationShape\n  {coin: Entity} owner {owner: Owner}\n  mode coin -> owner: one\n\ngame\n  coin ∈ Entity\n  active ∈ State\n  collected ∈ State\n  player ∈ Owner\n  collector ∈ Owner\n  replay-policy ∈ Policy\n  coin state active\n  coin owner player\n\non collect ?actor\n  ?coin state active ~>\n    ?coin state collected\n  if\n    ?coin owner ?actor\n  ?coin owner ?actor ~>\n    ?coin owner collector\n";

const FUNCTIONAL_CONFLICT_SOURCE: &str = "Entity\nState\nPolicy\n\ncoin/state: RelationShape\n  {coin: Entity} state {state: State}\n  mode coin -> state: one\n\ngame\n  coin ∈ Entity\n  active ∈ State\n  idle ∈ State\n  collected ∈ State\n  replay-policy ∈ Policy\n  coin state active\n\non collect\n  coin state active ~>\n    coin state collected\n  coin state active ~>\n    coin state idle\n";

const ALPHA_STATE_BINDER_SOURCE: &str = "Entity\nState\nOwner\nPolicy\n\ncoin/state: RelationShape\n  {coin: Entity} state {state: State}\n  mode coin -> state: one\n\ncoin/owner: RelationShape\n  {coin: Entity} owner {owner: Owner}\n  mode coin -> owner: one\n\ngame\n  coin ∈ Entity\n  active ∈ State\n  collected ∈ State\n  player ∈ Owner\n  replay-policy ∈ Policy\n  coin state active\n  coin owner player\n\non collect\n  ?coin state active ~>\n    ?coin state collected\n  if\n    ?coin owner ?owner\n";

const CANONICAL_GUARD_SOURCE: &str = "Entity\nState\nOwner\nPolicy\n\ncoin/state: RelationShape\n  {coin: Entity} state {state: State}\n  mode coin -> state: one\n\ncoin/owner: RelationShape\n  {coin: Entity} owner {owner: Owner}\n  mode coin -> owner: one\n\ngame\n  coin ∈ Entity\n  active ∈ State\n  collected ∈ State\n  player ∈ Owner\n  replay-policy ∈ Policy\n  coin state active\n  coin owner player\n\non collect ?actor\n  ?coin state active ~>\n    ?coin state collected\n  if\n    ?coin owner ?actor\n    ?coin state active\n";

fn temporary(extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    env::temp_dir().join(format!(
        "clause-m6-generated-runtime-{}-{nonce}.{extension}",
        std::process::id()
    ))
}

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
        Vec::new(),
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

fn replacement_delta(target: u8, successor: u8) -> StateDelta {
    StateDelta::new(
        vec![referent(target)],
        vec![AssertionOccurrence::new(
            referent(successor),
            ground(COLLECTED).id().clone(),
            referent(COLLECT_ONE_EVENT),
            referent(MODEL),
        )],
    )
    .unwrap()
}

#[test]
fn occurrence_exact_incremental_replay_is_canonical_and_rejects_tampering() {
    let revision = fixture();
    let active = ground(ACTIVE);
    let consequence = ground(CONSEQUENCE);
    let disconnected = ground(DISCONNECTED);
    let collected = ground(COLLECTED);
    let first_delta = replacement_delta(ACTIVE_ONE, COLLECTED_ONE);
    let second_delta = replacement_delta(ACTIVE_TWO, COLLECTED_TWO);

    let root = RuntimeSession::start(&revision, policy()).unwrap();
    assert_eq!(root.latest().support_roots(active.id()).len(), 2);
    assert_eq!(root.latest().support_roots(consequence.id()).len(), 2);
    let disconnected_support = root.latest().support_roots(disconnected.id());

    let after_first = root.apply_delta(&revision, first_delta.clone()).unwrap();
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
    assert_eq!(after_first.latest().work().added_supports(), 1);
    assert_eq!(after_first.latest().work().support_accounting_steps(), 1);

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
        .apply_delta(&revision, second_delta.clone())
        .unwrap();
    assert!(!after_second.latest().contains_content(active.id()));
    assert!(!after_second.latest().contains_content(consequence.id()));
    assert!(after_second.latest().contains_content(disconnected.id()));
    assert_eq!(
        after_second.latest().support_roots(disconnected.id()),
        disconnected_support
    );

    let replayed = RuntimeSession::replay_inputs(
        &revision,
        policy(),
        [
            RuntimeInput::Delta(first_delta.clone()),
            RuntimeInput::Delta(second_delta.clone()),
        ],
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
}

#[test]
fn empty_event_ticks_are_rejected_by_live_replay_and_reload() {
    let revision = fixture();
    let message = "runtime tick needs at least one event";
    assert!(
        RuntimeSession::start(&revision, policy())
            .unwrap()
            .transition(&revision, Vec::new())
            .unwrap_err()
            .to_string()
            .contains(message)
    );
    assert!(
        RuntimeSession::replay(&revision, policy(), [Vec::new()])
            .unwrap_err()
            .to_string()
            .contains(message)
    );

    let one = replacement_delta(ACTIVE_ONE, COLLECTED_ONE);
    let canonical =
        RuntimeSession::replay_inputs(&revision, policy(), [RuntimeInput::Delta(one.clone())])
            .unwrap()
            .canonical_bytes();
    let encoded = format!(
        "[\"delta\",[\"state-delta\",[\"withdraw\",[\"{}\"]],[\"admit\",[[\"occurrence\",\"{}\",\"{}\",\"{}\",\"{}\"]]]]]",
        one.withdrawals()[0].as_str(),
        one.admissions()[0].id().as_str(),
        one.admissions()[0].content().as_str(),
        one.admissions()[0].source().as_str(),
        one.admissions()[0].scope().as_str(),
    );
    let malformed = canonical.replace(&encoded, "[\"events\",[]]");
    assert!(
        reload_session(&malformed, &revision)
            .unwrap_err()
            .to_string()
            .contains(message)
    );
}

#[test]
fn authored_functional_matches_reject_conflicting_keyed_writes() {
    let compiled = elaborate::compile(
        frontend::parse(FUNCTIONAL_CONFLICT_SOURCE).expect("functional conflict source parses"),
    )
    .expect("functional conflict source elaborates into checked artifacts");
    let [journey] = compiled.runtime_journeys() else {
        panic!("one authored conflict event produces one runtime journey");
    };
    assert_eq!(journey.revision().model().transitions().len(), 2);
    let event_id = compiled
        .designations()
        .scoped(journey.revision().model().id(), "collect")
        .expect("authored event has a checked scoped referent");
    let policy_id = compiled
        .designations()
        .scoped(journey.revision().model().id(), "replay-policy")
        .expect("authored policy has a checked scoped referent");
    let error = journey
        .replay(
            RuntimePolicy::new(policy_id, 128, 512).unwrap(),
            [vec![TransitionEvent::new(
                referent(201),
                event_id,
                Vec::new(),
            )]],
        )
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("conflicting writes to one functional relation key")
    );
}

#[test]
fn inferred_state_binders_are_alpha_stable_and_role_distinct() {
    let renamed = ALPHA_STATE_BINDER_SOURCE
        .replace("?coin", "?zebra")
        .replace("?owner", "?aardvark");
    let original = elaborate::compile(
        frontend::parse(ALPHA_STATE_BINDER_SOURCE).expect("original event source parses"),
    )
    .expect("original event source elaborates");
    let renamed =
        elaborate::compile(frontend::parse(&renamed).expect("alpha-renamed event source parses"))
            .expect("alpha-renamed event source elaborates");
    let [original_journey] = original.runtime_journeys() else {
        panic!("original event produces one runtime journey");
    };
    let [renamed_journey] = renamed.runtime_journeys() else {
        panic!("renamed event produces one runtime journey");
    };
    let original_revision = original_journey.revision();
    let renamed_revision = renamed_journey.revision();
    let [original_transition] = original_revision.model().transitions() else {
        panic!("original event produces one transition");
    };
    let [renamed_transition] = renamed_revision.model().transitions() else {
        panic!("renamed event produces one transition");
    };
    let owner_relation = original
        .designations()
        .global("coin/owner")
        .expect("owner relation is designated");
    let coin_role = original
        .designations()
        .role(&owner_relation, "coin")
        .expect("owner relation coin role is designated");
    let owner_role = original
        .designations()
        .role(&owner_relation, "owner")
        .expect("owner relation owner role is designated");
    let original_guard = &original_revision.model().relational_contents()
        [original_transition.guards().first().expect("one guard")];
    let renamed_guard = &renamed_revision.model().relational_contents()
        [renamed_transition.guards().first().expect("one guard")];
    let coin_pattern = original_guard.roles()[&coin_role]
        .pattern_id()
        .expect("coin role retains a binder");
    let owner_pattern = original_guard.roles()[&owner_role]
        .pattern_id()
        .expect("owner role retains a binder");
    assert_ne!(coin_pattern, owner_pattern);
    assert_eq!(
        renamed_guard.roles()[&coin_role].pattern_id(),
        Some(coin_pattern)
    );
    assert_eq!(
        renamed_guard.roles()[&owner_role].pattern_id(),
        Some(owner_pattern)
    );
    assert_eq!(
        original_revision.model().relational_contents(),
        renamed_revision.model().relational_contents()
    );
    assert_eq!(original_transition, renamed_transition);
    assert_eq!(original_revision.identity(), renamed_revision.identity());
    assert_eq!(
        wire::serialize(original_revision),
        wire::serialize(renamed_revision)
    );
}

#[test]
fn guard_conjunctions_are_canonical_before_transition_identity() {
    let reordered_and_duplicated = CANONICAL_GUARD_SOURCE.replace(
        "    ?coin owner ?actor\n    ?coin state active\n",
        "    ?coin state active\n    ?coin owner ?actor\n    ?coin owner ?actor\n",
    );
    let canonical = elaborate::compile(
        frontend::parse(CANONICAL_GUARD_SOURCE).expect("canonical guard source parses"),
    )
    .expect("canonical guard source elaborates");
    let reordered = elaborate::compile(
        frontend::parse(&reordered_and_duplicated).expect("reordered guard source parses"),
    )
    .expect("reordered guard source elaborates");
    let [canonical_journey] = canonical.runtime_journeys() else {
        panic!("canonical guard source produces one runtime journey");
    };
    let [reordered_journey] = reordered.runtime_journeys() else {
        panic!("reordered guard source produces one runtime journey");
    };
    let canonical_revision = canonical_journey.revision();
    let reordered_revision = reordered_journey.revision();
    let [canonical_transition] = canonical_revision.model().transitions() else {
        panic!("canonical guard source produces one transition");
    };
    let [reordered_transition] = reordered_revision.model().transitions() else {
        panic!("reordered guard source produces one transition");
    };
    assert_eq!(canonical_transition.guards().len(), 2);
    assert_eq!(canonical_transition, reordered_transition);
    assert_eq!(canonical_revision.identity(), reordered_revision.identity());
    assert_eq!(
        wire::serialize(canonical_revision),
        wire::serialize(reordered_revision)
    );
}

#[test]
fn authored_event_payload_state_guards_survive_source_deletion_with_replay_parity() {
    let parsed = frontend::parse(ONE_COIN_SOURCE).expect("one-coin Clause source parses");
    assert_eq!(parsed.events.len(), 1);
    assert_eq!(parsed.events[0].payload_bindings.len(), 1);
    assert_eq!(parsed.events[0].state_bindings.len(), 1);
    assert_eq!(parsed.events[0].payload_bindings[0].value.as_str(), "actor");
    assert_eq!(parsed.events[0].state_bindings[0].as_str(), "coin");
    assert_eq!(parsed.events[0].transitions.len(), 2);
    assert_eq!(parsed.events[0].transitions[0].guards.len(), 1);
    let compiled = elaborate::compile(parsed).expect("one-coin Clause source elaborates");
    let [journey] = compiled.runtime_journeys() else {
        panic!("one authored event Model produces one runtime journey");
    };
    assert_eq!(journey.revision().model().transitions().len(), 2);
    let event_id = compiled
        .designations()
        .scoped(journey.revision().model().id(), "collect")
        .expect("authored event has a checked scoped referent");
    let player = compiled
        .designations()
        .scoped(journey.revision().model().id(), "player")
        .expect("authored payload value has a checked scoped referent");
    let collector = compiled
        .designations()
        .scoped(journey.revision().model().id(), "collector")
        .expect("tampered payload value has a checked scoped referent");
    let input = TransitionEvent::new(
        referent(200),
        event_id,
        vec![Term::referent(player.clone())],
    );
    let policy_id = compiled
        .designations()
        .scoped(journey.revision().model().id(), "replay-policy")
        .expect("authored policy has a checked scoped referent");
    let policy = RuntimePolicy::new(policy_id, 128, 512).expect("runtime policy is bounded");

    let expected = journey
        .replay(policy.clone(), [vec![input.clone()]])
        .expect("checked authored transition feeds the canonical runtime fold");
    assert_eq!(expected.inputs().len(), 1);
    let diff = StateDiff::between(&expected.states()[0], expected.latest(), journey.revision())
        .expect("authored runtime state diff is checked");
    assert_eq!(diff.occurrence_withdrawals().len(), 2);
    assert_eq!(diff.occurrence_admissions().len(), 2);
    assert_eq!(
        reload_session(&expected.canonical_bytes(), journey.revision()).unwrap(),
        expected
    );
    let payload = format!("[\"payload\",[[\"referent\",\"{}\"]]]", player.as_str());
    let tampered_payload = expected.canonical_bytes().replace(
        &payload,
        &format!("[\"payload\",[[\"referent\",\"{}\"]]]", collector.as_str()),
    );
    assert!(
        reload_session(&tampered_payload, journey.revision())
            .unwrap_err()
            .to_string()
            .contains("no joint pre-state and guard match")
    );

    let invalid_policy = RuntimePolicy::new(referent(255), 128, 512).unwrap();
    assert!(
        generated::emit_runtime_rust(journey, invalid_policy, vec![vec![input.clone()]])
            .unwrap_err()
            .to_string()
            .contains("runtime policy identity is absent from the checked Model")
    );
    let emitted = generated::emit_runtime_rust(journey, policy, vec![vec![input]])
        .expect("checked authored runtime emits standalone Rust");
    assert!(emitted.contains("runtime::RuntimeSession::replay"));
    assert!(emitted.contains("runtime::StateDiff::between"));
    assert!(emitted.contains("runtime::reload_session"));
    assert!(emitted.contains("runtime::TransitionEvent::new"));
    assert!(!emitted.contains("coin state active"));
    assert!(!emitted.contains("mod frontend"));
    assert!(!emitted.contains("mod elaborate"));

    let source = temporary("clause");
    let rust = temporary("rs");
    let binary = temporary("bin");
    fs::write(&source, ONE_COIN_SOURCE).expect("authored one-coin source writes");
    fs::remove_file(&source).expect("authored source deletes before generated compilation");
    fs::write(&rust, emitted).expect("generated runtime Rust writes");
    let built = Command::new("rustc")
        .args(["--edition=2024", "--cfg", "clause_generated"])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("generated runtime Rust compiler starts");
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let actual = Command::new(&binary)
        .output()
        .expect("source-deleted generated runtime starts");
    assert!(actual.status.success());
    assert_eq!(actual.stdout, expected.canonical_bytes().as_bytes());

    fs::remove_file(&rust).expect("generated runtime Rust cleans up");
    fs::remove_file(&binary).expect("generated runtime binary cleans up");
}

#[test]
fn effect_evidence_is_post_commit_distinct_and_canonical() {
    use std::cell::Cell;

    let compiled = elaborate::compile(
        frontend::parse(ONE_COIN_SOURCE).expect("one-coin Clause source parses"),
    )
    .expect("one-coin Clause source elaborates");
    let [journey] = compiled.runtime_journeys() else {
        panic!("one authored event Model produces one runtime journey");
    };
    let model = journey.revision().model();
    let event = compiled
        .designations()
        .scoped(model.id(), "collect")
        .expect("authored event has a checked referent");
    let player = compiled
        .designations()
        .scoped(model.id(), "player")
        .expect("effect evidence has a checked referent");
    let collector = compiled
        .designations()
        .scoped(model.id(), "collector")
        .expect("failure evidence has a checked referent");
    let policy = compiled
        .designations()
        .scoped(model.id(), "replay-policy")
        .expect("effect authority has a checked referent");
    let event_occurrence = referent(200);
    let root = RuntimeSession::start(
        journey.revision(),
        RuntimePolicy::new(policy.clone(), 128, 512).unwrap(),
    )
    .unwrap();
    let request = || {
        EffectRequest::new(
            model.id().clone(),
            event.clone(),
            policy.clone(),
            event_occurrence.clone(),
            model.id().clone(),
            0,
        )
    };
    let request_wire = request().canonical_bytes();
    assert_eq!(reload_effect_request(&request_wire).unwrap(), request());
    assert!(
        EffectTrace::denied(journey.revision(), root.latest(), request())
            .unwrap_err()
            .to_string()
            .contains("committed successor")
    );

    let committed = root
        .transition(
            journey.revision(),
            vec![TransitionEvent::new(
                event_occurrence.clone(),
                event.clone(),
                vec![Term::referent(player.clone())],
            )],
        )
        .unwrap();
    let state_bytes = committed.latest().canonical_bytes();
    let denied = EffectTrace::denied(journey.revision(), committed.latest(), request()).unwrap();
    assert_eq!(denied.authorization().decision(), AuthorizationDecision::Denied);
    assert!(denied.attempt_record().is_none());
    assert!(denied.receipt().is_none());
    assert!(denied.observation().is_none());
    assert_eq!(committed.latest().canonical_bytes(), state_bytes);
    assert_eq!(
        reload_effect_trace(
            &denied.canonical_bytes(),
            journey.revision(),
            committed.latest()
        )
        .unwrap(),
        denied
    );

    let calls = Cell::new(0);
    let succeeded = EffectTrace::attempt(
        journey.revision(),
        committed.latest(),
        request(),
        |_| {
            calls.set(calls.get() + 1);
            EffectOutcome::Succeeded {
                evidence: player.clone(),
            }
        },
    )
    .unwrap();
    let failed = EffectTrace::attempt(
        journey.revision(),
        committed.latest(),
        request(),
        |_| {
            calls.set(calls.get() + 1);
            EffectOutcome::Failed {
                evidence: collector,
            }
        },
    )
    .unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(
        succeeded.authorization().decision(),
        AuthorizationDecision::Authorized
    );
    assert_eq!(succeeded.receipt().unwrap().outcome(), ReceiptOutcome::Succeeded);
    assert_eq!(failed.receipt().unwrap().outcome(), ReceiptOutcome::Failed);
    assert_ne!(
        succeeded.receipt().unwrap().identity(),
        failed.receipt().unwrap().identity()
    );
    assert_eq!(
        succeeded.authorization().lineage().post_commit_state(),
        committed.latest().identity()
    );
    assert_eq!(committed.latest().canonical_bytes(), state_bytes);
    assert_eq!(
        reload_effect_trace(
            &succeeded.canonical_bytes(),
            journey.revision(),
            committed.latest()
        )
        .unwrap(),
        succeeded
    );
    assert_eq!(
        reload_effect_trace(
            &failed.canonical_bytes(),
            journey.revision(),
            committed.latest()
        )
        .unwrap(),
        failed
    );

    let tampered = succeeded
        .canonical_bytes()
        .replace("[\"outcome\",\"succeeded\"]", "[\"outcome\",\"failed\"]");
    assert!(
        reload_effect_trace(&tampered, journey.revision(), committed.latest())
            .unwrap_err()
            .to_string()
            .contains("Receipt identity")
    );
}
