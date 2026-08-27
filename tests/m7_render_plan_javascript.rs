//! Canonical RenderPlan snapshots and source-deleted JavaScript data emission.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use clause::{
    elaborate, frontend, generated,
    kernel::{
        ClauseSemanticsId, FiniteF32, ProgramChangeOccurrence, ProgramChangeOccurrenceId,
        ProgramDelta, ProgramId, ProgramRevision, ReferentId, Term,
    },
    render::{RenderItem, RenderPlan, reload_render_plan},
    runtime::{
        ReplayStep, RuntimeInput, RuntimePolicy, RuntimeProgramRevision, RuntimeSession,
        SessionStartOccurrenceId, TransitionEvent, TransitionOccurrenceId,
        reload_session_with_program,
    },
    wire,
};

const ONE_COIN_SOURCE: &str = "Entity\nState\nOwner\nPolicy\n\ncoin/state: RelationShape\n  {coin: Entity} state {state: State}\n  mode coin -> state: one\n\ncoin/owner: RelationShape\n  {coin: Entity} owner {owner: Owner}\n  mode coin -> owner: one\n\ngame\n  coin ∈ Entity\n  active ∈ State\n  collected ∈ State\n  player ∈ Owner\n  collector ∈ Owner\n  replay-policy ∈ Policy\n  coin state active\n  coin owner player\n\non collect ?actor\n  ?coin state active ~>\n    ?coin state collected\n  if\n    ?coin owner ?actor\n  ?coin owner ?actor ~>\n    ?coin owner collector\n";

fn temporary(extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    env::temp_dir().join(format!(
        "clause-m7-render-plan-{}-{nonce}.{extension}",
        std::process::id()
    ))
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn position(x: f32, y: f32) -> [FiniteF32; 2] {
    [
        FiniteF32::from_f32(x).expect("test position x is finite"),
        FiniteF32::from_f32(y).expect("test position y is finite"),
    ]
}

fn sorted_items(mut items: Vec<RenderItem>) -> Vec<RenderItem> {
    items.sort_by(|left, right| left.id().cmp(right.id()));
    items
}

fn admitted(revision: &clause::kernel::Revision, seed: u8) -> ProgramRevision {
    let semantics = ClauseSemanticsId::current();
    let program = ProgramId::from_referent(revision.model().id().clone());
    let snapshot = wire::program_snapshot(revision.model().clone(), semantics.clone());
    let change = ProgramChangeOccurrence::new(
        ProgramChangeOccurrenceId::from_referent(ReferentId::from_digest([seed; 32])),
        semantics,
        program.clone(),
        None,
        snapshot.identity().clone(),
        ProgramDelta::new(
            snapshot.checked_payload().atoms().into_iter().collect(),
            vec![],
        )
        .unwrap(),
        ReferentId::from_digest([seed.wrapping_add(1); 32]),
        vec![ReferentId::from_digest([seed.wrapping_add(2); 32])],
    )
    .unwrap();
    ProgramRevision::constitute_root(program, snapshot, &change).unwrap()
}

fn runtime_revision<'a>(
    program: &'a ProgramRevision,
    revision: &'a clause::kernel::Revision,
) -> RuntimeProgramRevision<'a> {
    RuntimeProgramRevision::new(program, revision).unwrap()
}

fn start_id(seed: u8) -> SessionStartOccurrenceId {
    SessionStartOccurrenceId::from_digest([seed; 32])
}

fn transition_id(seed: u8) -> TransitionOccurrenceId {
    TransitionOccurrenceId::from_digest([seed; 32])
}

#[test]
fn exact_state_plans_emit_frozen_source_deleted_esm_and_reconcile_totally() {
    let source = temporary("clause");
    let module = temporary("mjs");
    let harness = temporary("test.mjs");
    fs::write(&source, ONE_COIN_SOURCE).expect("one-coin Clause source writes");
    let authored = fs::read_to_string(&source).expect("one-coin Clause source reads");
    let compiled = elaborate::compile(frontend::parse(&authored).expect("Clause source parses"))
        .expect("Clause source elaborates");
    let [journey_ref] = compiled.runtime_journeys() else {
        panic!("one authored event Model produces one runtime journey");
    };
    let journey = journey_ref
        .clone()
        .bind_program_revision(admitted(journey_ref.revision(), 0xe1))
        .unwrap();
    let revision = journey.revision();
    let program = journey.program_revision().unwrap();
    let typed = runtime_revision(program, revision);
    let model = revision.model();
    let event = compiled
        .designations()
        .scoped(model.id(), "collect")
        .expect("collect event is designated");
    let player = compiled
        .designations()
        .scoped(model.id(), "player")
        .expect("player is designated");
    let coin = compiled
        .designations()
        .scoped(model.id(), "coin")
        .expect("coin is designated");
    let policy_id = compiled
        .designations()
        .scoped(model.id(), "replay-policy")
        .expect("runtime policy is designated");
    let policy = RuntimePolicy::new(policy_id, 128, 512).expect("runtime policy is bounded");
    let input = TransitionEvent::new(
        ReferentId::from_digest([0xf0; 32]),
        event,
        vec![Term::referent(player.clone())],
    );
    let session = journey
        .replay_with_occurrences(
            policy.clone(),
            start_id(1),
            [ReplayStep {
                occurrence: transition_id(1),
                input: RuntimeInput::Events(vec![input.clone()]),
            }],
        )
        .expect("one collection commits through Clause runtime semantics");
    let replay = journey
        .replay_with_occurrences(
            policy.clone(),
            start_id(1),
            [ReplayStep {
                occurrence: transition_id(1),
                input: RuntimeInput::Events(vec![input]),
            }],
        )
        .expect("the same accepted event log deterministically replays");
    assert_eq!(session.canonical_bytes(), replay.canonical_bytes());
    assert_eq!(
        reload_session_with_program(&session.canonical_bytes(), &typed).unwrap(),
        session
    );

    let initial = &session.states()[0];
    let collected = session.latest();
    let initial_plan = RenderPlan::new(
        &typed,
        initial,
        sorted_items(vec![
            RenderItem::new(player.clone(), position(0.0, 0.0)).unwrap(),
            RenderItem::new(coin.clone(), position(10.0, 0.0)).unwrap(),
        ]),
    )
    .expect("initial total desired scene is state-bound");
    let collected_plan = RenderPlan::new(
        &typed,
        collected,
        vec![RenderItem::new(player.clone(), position(10.0, 0.0)).unwrap()],
    )
    .expect("collected total desired scene omits the coin");
    assert_eq!(
        reload_render_plan(&initial_plan.canonical_bytes(), &typed, initial).unwrap(),
        initial_plan
    );
    assert_eq!(
        reload_render_plan(&collected_plan.canonical_bytes(), &typed, collected).unwrap(),
        collected_plan
    );

    let emitted =
        generated::emit_render_plan_javascript(&[initial_plan.clone(), collected_plan.clone()])
            .expect("two exact state plans emit import-free JavaScript");
    assert!(emitted.contains("Object.freeze"));
    assert!(emitted.contains("export function renderPlan"));
    for forbidden in [
        ONE_COIN_SOURCE,
        "coin state active",
        "frontend",
        "elaborate",
        "wire decoder",
        "decode",
        "node:",
        "import ",
        "EffectTrace",
        "EffectRequest",
        "receipt",
        "roots",
        "source",
        "proof",
    ] {
        assert!(
            !emitted.contains(forbidden),
            "emitted ESM contains {forbidden:?}"
        );
    }
    fs::write(&module, emitted).expect("generated RenderPlan ESM writes");
    fs::remove_file(&source).expect("Clause source deletes before generated ESM executes");

    let host = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("host/provisional.mjs");
    let script = BUN_HARNESS
        .replace("__MODULE__", &format!("{:?}", file_url(&module)))
        .replace("__HOST__", &format!("{:?}", file_url(&host)))
        .replace("__INITIAL_PLAN__", &initial_plan.canonical_bytes())
        .replace("__COLLECTED_PLAN__", &collected_plan.canonical_bytes())
        .replace(
            "__INITIAL_STATE__",
            &format!("{:?}", initial.identity().as_str()),
        )
        .replace(
            "__COLLECTED_STATE__",
            &format!("{:?}", collected.identity().as_str()),
        )
        .replace(
            "__REVISION__",
            &format!("{:?}", program.identity().as_str()),
        )
        .replace("__PLAYER__", &format!("{:?}", player.as_str()))
        .replace("__COIN__", &format!("{:?}", coin.as_str()));
    fs::write(&harness, script).expect("Bun source-deletion harness writes");
    let actual = Command::new("nix")
        .args(["shell", "nixpkgs#bun", "-c", "bun", "run"])
        .arg(&harness)
        .output()
        .expect("Bun source-deletion harness starts");
    assert!(
        actual.status.success(),
        "{}",
        String::from_utf8_lossy(&actual.stderr)
    );
    assert_eq!(
        actual.stdout,
        format!(
            "{}\n{}\n",
            initial_plan.canonical_bytes(),
            collected_plan.canonical_bytes()
        )
        .as_bytes()
    );

    fs::remove_file(&module).expect("generated RenderPlan ESM cleans up");
    fs::remove_file(&harness).expect("Bun harness cleans up");
}

#[test]
fn render_plan_admission_and_emission_reject_wrong_noncanonical_or_tampered_data() {
    let compiled = elaborate::compile(
        frontend::parse(ONE_COIN_SOURCE).expect("one-coin Clause source parses"),
    )
    .expect("one-coin Clause source elaborates");
    let [journey_ref] = compiled.runtime_journeys() else {
        panic!("one authored event Model produces one runtime journey");
    };
    let journey = journey_ref
        .clone()
        .bind_program_revision(admitted(journey_ref.revision(), 0xe4))
        .unwrap();
    let revision = journey.revision();
    let program = journey.program_revision().unwrap();
    let typed = runtime_revision(program, revision);
    let model = revision.model();
    let player = compiled
        .designations()
        .scoped(model.id(), "player")
        .expect("player is designated");
    let coin = compiled
        .designations()
        .scoped(model.id(), "coin")
        .expect("coin is designated");
    let policy_id = compiled
        .designations()
        .scoped(model.id(), "replay-policy")
        .expect("runtime policy is designated");
    let event = compiled
        .designations()
        .scoped(model.id(), "collect")
        .expect("collect event is designated");
    let policy = RuntimePolicy::new(policy_id, 128, 512).unwrap();
    let root = RuntimeSession::start_with_occurrence(&typed, policy.clone(), start_id(2)).unwrap();
    let state = root.latest();
    let different_state = journey
        .replay_with_occurrences(
            policy,
            start_id(3),
            [ReplayStep {
                occurrence: transition_id(3),
                input: RuntimeInput::Events(vec![TransitionEvent::new(
                    ReferentId::from_digest([0xf1; 32]),
                    event,
                    vec![Term::referent(player.clone())],
                )]),
            }],
        )
        .unwrap();
    let player_item = RenderItem::new(player, position(0.0, -0.0)).unwrap();
    let coin_item = RenderItem::new(coin, position(10.0, 0.0)).unwrap();
    let plan = RenderPlan::new(
        &typed,
        state,
        sorted_items(vec![player_item.clone(), coin_item.clone()]),
    )
    .unwrap();
    assert!(plan.canonical_bytes().contains("\"00000000\""));

    assert!(FiniteF32::from_f32(f32::INFINITY).is_err());
    assert!(
        RenderPlan::new(
            &typed,
            state,
            vec![player_item.clone(), player_item.clone()]
        )
        .is_err()
    );
    let mut noncanonical_items = sorted_items(vec![player_item.clone(), coin_item.clone()]);
    noncanonical_items.reverse();
    assert!(RenderPlan::new(&typed, state, noncanonical_items).is_err());
    let unknown = RenderItem::new(ReferentId::from_digest([0xfe; 32]), position(0.0, 0.0)).unwrap();
    assert!(RenderPlan::new(&typed, state, vec![unknown]).is_err());

    let wrong_model = plan.canonical_bytes().replacen(
        program.identity().as_str(),
        &format!("program-revision-sha256-{}", "0".repeat(64)),
        1,
    );
    assert!(reload_render_plan(&wrong_model, &typed, state).is_err());
    let wrong_state = plan.canonical_bytes().replacen(
        state.identity().as_str(),
        &format!("state-sha256-{}", "0".repeat(64)),
        1,
    );
    assert!(reload_render_plan(&wrong_state, &typed, state).is_err());
    let old_envelope =
        plan.canonical_bytes()
            .replacen("clause-render-plan-v2", "clause-render-plan-v1", 1);
    assert!(reload_render_plan(&old_envelope, &typed, state).is_err());
    let tampered_bits = plan
        .canonical_bytes()
        .replacen("\"00000000\"", "\"7f800000\"", 1);
    assert!(reload_render_plan(&tampered_bits, &typed, state).is_err());
    assert!(
        reload_render_plan(&(" ".to_owned() + &plan.canonical_bytes()), &typed, state).is_err()
    );

    assert!(reload_render_plan(&plan.canonical_bytes(), &typed, different_state.latest()).is_err());

    assert!(generated::emit_render_plan_javascript(&[]).is_err());
    assert!(generated::emit_render_plan_javascript(&[plan.clone(), plan.clone()]).is_err());

    let other_source = format!("{ONE_COIN_SOURCE}\nExtra\n");
    let other_compiled =
        elaborate::compile(frontend::parse(&other_source).expect("distinct Model source parses"))
            .expect("distinct Model source elaborates");
    let [other_journey_ref] = other_compiled.runtime_journeys() else {
        panic!("distinct Model retains one runtime journey");
    };
    let other_journey = other_journey_ref
        .clone()
        .bind_program_revision(admitted(other_journey_ref.revision(), 0xe7))
        .unwrap();
    let other_program = other_journey.program_revision().unwrap();
    let other_typed = runtime_revision(other_program, other_journey.revision());
    let other_policy = other_compiled
        .designations()
        .scoped(other_journey.revision().model().id(), "replay-policy")
        .expect("distinct Model policy is designated");
    let other_session = RuntimeSession::start_with_occurrence(
        &other_typed,
        RuntimePolicy::new(other_policy, 128, 512).unwrap(),
        start_id(4),
    )
    .unwrap();
    assert!(
        RenderPlan::new(&typed, other_session.latest(), Vec::new()).is_err(),
        "a StateRevision from another Model must fail closed"
    );
    let other_plan = RenderPlan::new(&other_typed, other_session.latest(), Vec::new()).unwrap();
    assert!(generated::emit_render_plan_javascript(&[plan, other_plan]).is_err());
}

const BUN_HARNESS: &str = r#"
import * as artifact from __MODULE__;
import { createMeshBinding, renderPlanFor } from __HOST__;

const initialExpected = __INITIAL_PLAN__;
const collectedExpected = __COLLECTED_PLAN__;
const initialState = __INITIAL_STATE__;
const collectedState = __COLLECTED_STATE__;
const programRevisionId = __REVISION__;
const playerId = __PLAYER__;
const coinId = __COIN__;

const initial = renderPlanFor(artifact, initialState, programRevisionId);
const collected = renderPlanFor(artifact, collectedState, programRevisionId);
if (JSON.stringify(initial) !== JSON.stringify(initialExpected)) throw new Error("initial Rust/JS RenderPlan bytes differ");
if (JSON.stringify(collected) !== JSON.stringify(collectedExpected)) throw new Error("collected Rust/JS RenderPlan bytes differ");
if (!Object.isFrozen(artifact.renderPlan(initialState))) throw new Error("generated plan is not frozen");
if (!Object.isFrozen(artifact.renderPlan(initialState)[3][1][0])) throw new Error("generated item is not frozen");

class Geometry { dispose() {} }
class Material { dispose() {} }
class Mesh {
  constructor(geometry, material) {
    this.geometry = geometry;
    this.material = material;
    this.visible = true;
    this.position = { x: 0, y: 0, z: 0, set: (x, y, z) => { this.position.x = x; this.position.y = y; this.position.z = z; } };
  }
}
const scene = { add() {}, remove() {} };
const binding = createMeshBinding(scene, { programRevisionId, meshes: new Map([[playerId, new Mesh(new Geometry(), new Material())], [coinId, new Mesh(new Geometry(), new Material())]]) });
binding.apply(initial);
binding.apply(collected);
binding.apply(collected);
if (binding.mesh(coinId).visible !== false) throw new Error("omitted coin was revived");
if (binding.mesh(playerId).position.x !== 10 || binding.mesh(playerId).position.y !== 0 || binding.mesh(playerId).position.z !== 0) throw new Error("f32x2 did not map to x/y/0");

const snapshot = JSON.stringify({ player: [binding.mesh(playerId).position.x, binding.mesh(playerId).position.y, binding.mesh(playerId).position.z, binding.mesh(playerId).visible], coin: [binding.mesh(coinId).position.x, binding.mesh(coinId).position.y, binding.mesh(coinId).position.z, binding.mesh(coinId).visible] });
const unknownId = `ref-sha256-${"f".repeat(64)}`;
const movedPlayer = ["item", playerId, ["position-f32x2", "41a00000", "00000000"]];
const unknown = ["item", unknownId, ["position-f32x2", "00000000", "00000000"]];
const invalid = ["clause-render-plan-v2", ["program-revision", programRevisionId], ["state-revision", collectedState], ["items", [movedPlayer, unknown].sort((left, right) => left[1].localeCompare(right[1]))]];
let rejected = false;
try { binding.apply(invalid); } catch { rejected = true; }
if (!rejected || JSON.stringify({ player: [binding.mesh(playerId).position.x, binding.mesh(playerId).position.y, binding.mesh(playerId).position.z, binding.mesh(playerId).visible], coin: [binding.mesh(coinId).position.x, binding.mesh(coinId).position.y, binding.mesh(coinId).position.z, binding.mesh(coinId).visible] }) !== snapshot) throw new Error("unknown item mutated a mesh before rejection");
const duplicate = ["clause-render-plan-v2", ["program-revision", programRevisionId], ["state-revision", collectedState], ["items", [movedPlayer, movedPlayer]]];
rejected = false;
try { binding.apply(duplicate); } catch { rejected = true; }
if (!rejected || JSON.stringify({ player: [binding.mesh(playerId).position.x, binding.mesh(playerId).position.y, binding.mesh(playerId).position.z, binding.mesh(playerId).visible], coin: [binding.mesh(coinId).position.x, binding.mesh(coinId).position.y, binding.mesh(coinId).position.z, binding.mesh(coinId).visible] }) !== snapshot) throw new Error("duplicate item mutated a mesh before rejection");

console.log(JSON.stringify(initial));
console.log(JSON.stringify(collected));
"#;
