//! Canonical RenderPlan snapshots and source-deleted JavaScript data emission.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use clause::{
    elaborate, frontend, generated,
    kernel::{FiniteF32, ReferentId, Term},
    render::{RenderItem, RenderPlan, reload_render_plan},
    runtime::{RuntimePolicy, RuntimeSession, TransitionEvent, reload_session},
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

#[test]
fn exact_state_plans_emit_frozen_source_deleted_esm_and_reconcile_totally() {
    let source = temporary("clause");
    let module = temporary("mjs");
    let harness = temporary("test.mjs");
    fs::write(&source, ONE_COIN_SOURCE).expect("one-coin Clause source writes");
    let authored = fs::read_to_string(&source).expect("one-coin Clause source reads");
    let compiled = elaborate::compile(frontend::parse(&authored).expect("Clause source parses"))
        .expect("Clause source elaborates");
    let [journey] = compiled.runtime_journeys() else {
        panic!("one authored event Model produces one runtime journey");
    };
    let revision = journey.revision();
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
        .replay(policy.clone(), [vec![input.clone()]])
        .expect("one collection commits through Clause runtime semantics");
    let replay = journey
        .replay(policy.clone(), [vec![input]])
        .expect("the same accepted event log deterministically replays");
    assert_eq!(session.canonical_bytes(), replay.canonical_bytes());
    assert_eq!(
        reload_session(&session.canonical_bytes(), revision).unwrap(),
        session
    );

    let initial = &session.states()[0];
    let collected = session.latest();
    let initial_plan = RenderPlan::new(
        revision,
        initial,
        sorted_items(vec![
            RenderItem::new(player.clone(), position(0.0, 0.0)).unwrap(),
            RenderItem::new(coin.clone(), position(10.0, 0.0)).unwrap(),
        ]),
    )
    .expect("initial total desired scene is state-bound");
    let collected_plan = RenderPlan::new(
        revision,
        collected,
        vec![RenderItem::new(player.clone(), position(10.0, 0.0)).unwrap()],
    )
    .expect("collected total desired scene omits the coin");
    assert_eq!(
        reload_render_plan(&initial_plan.canonical_bytes(), revision, initial).unwrap(),
        initial_plan
    );
    assert_eq!(
        reload_render_plan(&collected_plan.canonical_bytes(), revision, collected).unwrap(),
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
            &format!("{:?}", revision.identity().to_string()),
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
    let [journey] = compiled.runtime_journeys() else {
        panic!("one authored event Model produces one runtime journey");
    };
    let revision = journey.revision();
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
    let root = RuntimeSession::start(revision, policy.clone()).unwrap();
    let state = root.latest();
    let different_state = journey
        .replay(
            policy,
            [vec![TransitionEvent::new(
                ReferentId::from_digest([0xf1; 32]),
                event,
                vec![Term::referent(player.clone())],
            )]],
        )
        .unwrap();
    let player_item = RenderItem::new(player, position(0.0, -0.0)).unwrap();
    let coin_item = RenderItem::new(coin, position(10.0, 0.0)).unwrap();
    let plan = RenderPlan::new(
        revision,
        state,
        sorted_items(vec![player_item.clone(), coin_item.clone()]),
    )
    .unwrap();
    assert!(plan.canonical_bytes().contains("\"00000000\""));

    assert!(FiniteF32::from_f32(f32::INFINITY).is_err());
    assert!(
        RenderPlan::new(
            revision,
            state,
            vec![player_item.clone(), player_item.clone()]
        )
        .is_err()
    );
    let mut noncanonical_items = sorted_items(vec![player_item.clone(), coin_item.clone()]);
    noncanonical_items.reverse();
    assert!(RenderPlan::new(revision, state, noncanonical_items).is_err());
    let unknown = RenderItem::new(ReferentId::from_digest([0xfe; 32]), position(0.0, 0.0)).unwrap();
    assert!(RenderPlan::new(revision, state, vec![unknown]).is_err());

    let wrong_model = plan.canonical_bytes().replacen(
        &revision.identity().to_string(),
        &format!("rev-sha256-{}", "0".repeat(64)),
        1,
    );
    assert!(reload_render_plan(&wrong_model, revision, state).is_err());
    let wrong_state = plan.canonical_bytes().replacen(
        state.identity().as_str(),
        &format!("state-sha256-{}", "0".repeat(64)),
        1,
    );
    assert!(reload_render_plan(&wrong_state, revision, state).is_err());
    let tampered_bits = plan
        .canonical_bytes()
        .replacen("\"00000000\"", "\"7f800000\"", 1);
    assert!(reload_render_plan(&tampered_bits, revision, state).is_err());
    assert!(
        reload_render_plan(&(" ".to_owned() + &plan.canonical_bytes()), revision, state).is_err()
    );

    assert!(
        reload_render_plan(&plan.canonical_bytes(), revision, different_state.latest()).is_err()
    );

    assert!(generated::emit_render_plan_javascript(&[]).is_err());
    assert!(generated::emit_render_plan_javascript(&[plan.clone(), plan.clone()]).is_err());

    let other_source = format!("{ONE_COIN_SOURCE}\nExtra\n");
    let other_compiled =
        elaborate::compile(frontend::parse(&other_source).expect("distinct Model source parses"))
            .expect("distinct Model source elaborates");
    let [other_journey] = other_compiled.runtime_journeys() else {
        panic!("distinct Model retains one runtime journey");
    };
    let other_policy = other_compiled
        .designations()
        .scoped(other_journey.revision().model().id(), "replay-policy")
        .expect("distinct Model policy is designated");
    let other_session = RuntimeSession::start(
        other_journey.revision(),
        RuntimePolicy::new(other_policy, 128, 512).unwrap(),
    )
    .unwrap();
    assert!(
        RenderPlan::new(revision, other_session.latest(), Vec::new()).is_err(),
        "a StateRevision from another Model must fail closed"
    );
    let other_plan =
        RenderPlan::new(other_journey.revision(), other_session.latest(), Vec::new()).unwrap();
    assert!(generated::emit_render_plan_javascript(&[plan, other_plan]).is_err());
}

const BUN_HARNESS: &str = r#"
import * as artifact from __MODULE__;
import { createTwoMeshBinding, renderPlanFor } from __HOST__;

const initialExpected = __INITIAL_PLAN__;
const collectedExpected = __COLLECTED_PLAN__;
const initialState = __INITIAL_STATE__;
const collectedState = __COLLECTED_STATE__;
const revisionId = __REVISION__;
const playerId = __PLAYER__;
const coinId = __COIN__;

const initial = renderPlanFor(artifact, initialState, revisionId);
const collected = renderPlanFor(artifact, collectedState, revisionId);
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
const THREE = { Mesh, BoxGeometry: Geometry, CylinderGeometry: Geometry, MeshBasicMaterial: Material };
const scene = { add() {}, remove() {} };
const binding = createTwoMeshBinding(THREE, scene, { revisionId, playerId, coinId });
binding.apply(initial);
binding.apply(collected);
binding.apply(collected);
if (binding.meshes.coin.visible !== false) throw new Error("omitted coin was revived");
if (binding.meshes.player.position.x !== 10 || binding.meshes.player.position.y !== 0 || binding.meshes.player.position.z !== 0) throw new Error("f32x2 did not map to x/y/0");

const snapshot = JSON.stringify({ player: [binding.meshes.player.position.x, binding.meshes.player.position.y, binding.meshes.player.position.z, binding.meshes.player.visible], coin: [binding.meshes.coin.position.x, binding.meshes.coin.position.y, binding.meshes.coin.position.z, binding.meshes.coin.visible] });
const unknownId = `ref-sha256-${"f".repeat(64)}`;
const movedPlayer = ["item", playerId, ["position-f32x2", "41a00000", "00000000"]];
const unknown = ["item", unknownId, ["position-f32x2", "00000000", "00000000"]];
const invalid = ["clause-render-plan-v1", ["model-revision", revisionId], ["state-revision", collectedState], ["items", [movedPlayer, unknown].sort((left, right) => left[1].localeCompare(right[1]))]];
let rejected = false;
try { binding.apply(invalid); } catch { rejected = true; }
if (!rejected || JSON.stringify({ player: [binding.meshes.player.position.x, binding.meshes.player.position.y, binding.meshes.player.position.z, binding.meshes.player.visible], coin: [binding.meshes.coin.position.x, binding.meshes.coin.position.y, binding.meshes.coin.position.z, binding.meshes.coin.visible] }) !== snapshot) throw new Error("unknown item mutated a mesh before rejection");
const duplicate = ["clause-render-plan-v1", ["model-revision", revisionId], ["state-revision", collectedState], ["items", [movedPlayer, movedPlayer]]];
rejected = false;
try { binding.apply(duplicate); } catch { rejected = true; }
if (!rejected || JSON.stringify({ player: [binding.meshes.player.position.x, binding.meshes.player.position.y, binding.meshes.player.position.z, binding.meshes.player.visible], coin: [binding.meshes.coin.position.x, binding.meshes.coin.position.y, binding.meshes.coin.position.z, binding.meshes.coin.visible] }) !== snapshot) throw new Error("duplicate item mutated a mesh before rejection");

console.log(JSON.stringify(initial));
console.log(JSON.stringify(collected));
"#;
