//! Source-deleted generated runtime-v3 execution in real Chrome and Three.js.

use std::{
    env, fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use clause::{
    elaborate, frontend, generated,
    kernel::{
        ClauseSemanticsId, ProgramChangeOccurrence, ProgramChangeOccurrenceId, ProgramDelta,
        ProgramId, ProgramRevision, ReferentId,
    },
    render::{SceneProjectionSpec, project_render_plan},
    runtime::{
        RuntimePolicy, RuntimeProgramRevision, RuntimeSession, SessionStartOccurrenceId,
        TransitionEvent, TransitionOccurrenceId,
    },
    wire,
};

const SOURCE: &str = r#"F32
Entity
State
Policy

Vec2
  x: F32
  y: F32

coin/state: RelationShape
  {coin: Entity} state {state: State}
  mode coin -> state: one

scene/placement: RelationShape
  {item: Entity} scene-position {point: Vec2}
  mode item -> point: one

game
  player ∈ Entity
  coin ∈ Entity
  active ∈ State
  collected ∈ State
  replay-policy ∈ Policy
  player scene-position Vec2 { x: 0.0, y: 0.0 }
  coin state active

coin scene-position Vec2 { x: 10.0, y: 0.0 } if
  coin state active

on collect
  coin state active ~>
    coin state collected
"#;

fn temporary_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock follows epoch")
        .as_nanos();
    env::temp_dir().join(format!(
        "clause-one-coin-live-host-{}-{nonce}",
        std::process::id()
    ))
}

fn admitted(revision: &clause::kernel::Revision) -> ProgramRevision {
    let semantics = ClauseSemanticsId::current();
    let program = ProgramId::from_referent(revision.model().id().clone());
    let snapshot = wire::program_snapshot(revision.model().clone(), semantics.clone());
    let change = ProgramChangeOccurrence::new(
        ProgramChangeOccurrenceId::from_referent(ReferentId::from_digest([0xc1; 32])),
        semantics,
        program.clone(),
        None,
        snapshot.identity().clone(),
        ProgramDelta::new(
            snapshot.checked_payload().atoms().into_iter().collect(),
            vec![],
        )
        .unwrap(),
        ReferentId::from_digest([0xc2; 32]),
        vec![ReferentId::from_digest([0xc3; 32])],
    )
    .unwrap();
    ProgramRevision::constitute_root(program, snapshot, &change).unwrap()
}

#[test]
fn authored_coin_collects_in_source_deleted_generated_esm_inside_chrome_and_three() {
    let root = temporary_root();
    fs::create_dir(&root).expect("acceptance root creates");
    let source = root.join("one-coin.clause");
    let artifact = root.join("artifact.mjs");
    let html = root.join("index.html");
    let chrome_profile = root.join("chrome-profile");
    fs::write(&source, SOURCE).expect("authored Clause source writes");

    let authored = fs::read_to_string(&source).expect("authored Clause source reads");
    let compiled = elaborate::compile(frontend::parse(&authored).expect("source parses"))
        .expect("source elaborates");
    let [journey] = compiled.runtime_journeys() else {
        panic!("one authored event produces one runtime journey");
    };
    let program = admitted(journey.revision());
    let typed = RuntimeProgramRevision::new(&program, journey.revision()).unwrap();
    let model = journey.revision().model();
    let designation = |name| {
        compiled
            .designations()
            .scoped(model.id(), name)
            .unwrap_or_else(|_| panic!("{name} is designated"))
    };
    let relation = compiled.designations().global("scene/placement").unwrap();
    let vec2 = compiled.designations().global("Vec2").unwrap();
    let spec = SceneProjectionSpec::new(
        relation.clone(),
        compiled.designations().role(&relation, "item").unwrap(),
        compiled.designations().role(&relation, "point").unwrap(),
        vec2.clone(),
        compiled.designations().scoped(&vec2, "x").unwrap(),
        compiled.designations().scoped(&vec2, "y").unwrap(),
    )
    .unwrap();
    let player = designation("player");
    let coin = designation("coin");
    let event = designation("collect");
    let policy = RuntimePolicy::new(designation("replay-policy"), 128, 512).unwrap();
    let start = SessionStartOccurrenceId::from_digest([0xc4; 32]);
    let transition = TransitionOccurrenceId::from_digest([0xc5; 32]);
    let event_occurrence = ReferentId::from_digest([0xc6; 32]);
    let initial = RuntimeSession::start_with_occurrence(&typed, policy, start).unwrap();
    let session = initial
        .transition_with_occurrence(
            journey.revision(),
            vec![TransitionEvent::new(event_occurrence, event, Vec::new())],
            transition,
        )
        .unwrap();
    let initial_plan = project_render_plan(&typed, &session.states()[0], &spec).unwrap();
    let collected_plan = project_render_plan(&typed, session.latest(), &spec).unwrap();
    let module = generated::emit_live_runtime_javascript(
        &session,
        &[initial_plan.clone(), collected_plan.clone()],
    )
    .expect("exact runtime edge emits live JavaScript");
    fs::write(&artifact, module).expect("generated live ESM writes");
    fs::remove_file(&source).expect("authored source deletes before browser execution");
    assert!(!source.exists());

    let page = PAGE
        .replace("__SESSION__", &format!("{:?}", session.canonical_bytes()))
        .replace(
            "__INITIAL_PLAN__",
            &format!("{:?}", initial_plan.canonical_bytes()),
        )
        .replace(
            "__COLLECTED_PLAN__",
            &format!("{:?}", collected_plan.canonical_bytes()),
        )
        .replace("__PLAYER__", &format!("{:?}", player.as_str()))
        .replace("__COIN__", &format!("{:?}", coin.as_str()));
    fs::write(&html, page).expect("browser acceptance page writes");

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let server_source = manifest.join("host/acceptance-server.mjs");
    let host_source = manifest.join("host/provisional.mjs");
    let three_source = manifest.join("node_modules/three/build/three.module.js");
    assert!(
        three_source.is_file(),
        "pinned Three.js package is installed"
    );
    let mut server = Command::new("bun")
        .arg("run")
        .arg(&server_source)
        .arg(&root)
        .arg(&host_source)
        .arg(&three_source)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Bun acceptance server starts");
    let mut url = String::new();
    BufReader::new(server.stdout.take().unwrap())
        .read_line(&mut url)
        .expect("Bun server reports its URL");
    let url = url.trim();
    assert!(url.starts_with("http://"), "Bun server returned {url:?}");

    let browser = Command::new("timeout")
        .arg("30s")
        .arg("google-chrome-stable")
        .arg("--headless=new")
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--use-angle=swiftshader")
        .arg("--enable-unsafe-swiftshader")
        .arg("--virtual-time-budget=5000")
        .arg(format!("--user-data-dir={}", chrome_profile.display()))
        .arg("--dump-dom")
        .arg(url)
        .output()
        .expect("real Chrome starts");
    server.kill().expect("Bun acceptance server stops");
    server.wait().expect("Bun acceptance server reaps");
    let dom = String::from_utf8_lossy(&browser.stdout);
    assert!(
        browser.status.success()
            && dom.contains("data-result=\"pass\"")
            && dom.contains("THREE.WebGLRenderer"),
        "Chrome/Three.js live acceptance failed\nstatus: {}\nstdout: {dom}\nstderr: {}",
        browser.status,
        String::from_utf8_lossy(&browser.stderr),
    );
    fs::remove_dir_all(&root).expect("acceptance scratch cleans up");
}

const PAGE: &str = r#"<!doctype html>
<html><body><script>
document.body.dataset.result = "boot";
addEventListener("error", (event) => { document.body.dataset.result = "fail"; document.body.textContent = `FAIL import: ${event.message}`; });
addEventListener("unhandledrejection", (event) => { document.body.dataset.result = "fail"; document.body.textContent = `FAIL import: ${event.reason}`; });
</script><script>
(async () => {
const [generated, THREE, host] = await Promise.all([import("/artifact.mjs"), import("/three.mjs"), import("/host.mjs")]);
const { createEventBridge, createMeshBinding, loadArtifact, renderPlanFor } = host;

try {
  const expectedSession = __SESSION__;
  const expectedInitialPlan = __INITIAL_PLAN__;
  const expectedCollectedPlan = __COLLECTED_PLAN__;
  const playerId = __PLAYER__;
  const coinId = __COIN__;
  const artifact = loadArtifact(generated);
  if (generated.runtimeSessionCanonical !== expectedSession) throw new Error("Rust/generated runtime-v3 bytes differ");
  if (JSON.stringify(renderPlanFor(artifact, generated.initialStateRevisionId)) !== expectedInitialPlan) throw new Error("initial Rust/generated RenderPlan differs");

  const scene = new THREE.Scene();
  const camera = new THREE.OrthographicCamera(-2, 12, 4, -4, 0.1, 100);
  camera.position.z = 10;
  const renderer = new THREE.WebGLRenderer({ antialias: false, preserveDrawingBuffer: true });
  renderer.setSize(96, 64);
  document.body.append(renderer.domElement);
  const player = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), new THREE.MeshBasicMaterial({ color: 0x3366ff }));
  const coin = new THREE.Mesh(new THREE.CylinderGeometry(0.35, 0.35, 0.1, 16), new THREE.MeshBasicMaterial({ color: 0xffcc00 }));
  const binding = createMeshBinding(scene, { programRevisionId: artifact.programRevisionId, meshes: new Map([[playerId, player], [coinId, coin]]) });
  binding.apply(renderPlanFor(artifact, generated.initialStateRevisionId));
  const runtime = artifact.createRuntime();
  const bridge = createEventBridge({
    artifact,
    runtime,
    allocateEventOccurrence: () => generated.eventOccurrenceId,
    allocateTransitionOccurrence: () => generated.transitionOccurrenceId,
    onState: (state) => binding.apply(renderPlanFor(artifact, state)),
  });
  if (!bridge.dispatch(generated.eventName, generated.expectedEventPayload)) throw new Error("generated event was not dispatched");
  renderer.render(scene, camera);
  if (runtime.state() !== generated.finalStateRevisionId) throw new Error("generated runtime did not commit exact final state");
  if (JSON.stringify(renderPlanFor(artifact, runtime.state())) !== expectedCollectedPlan) throw new Error("collected Rust/generated RenderPlan differs");
  if (binding.mesh(coinId).visible || binding.mesh(playerId).visible !== true) throw new Error("generated total plan did not hide only the omitted mesh");
  if (!renderer.isWebGLRenderer || renderer.info.render.calls < 1) throw new Error("actual Three.js WebGLRenderer did not render");
  document.body.dataset.result = "pass";
  document.body.append("THREE.WebGLRenderer runtime-v3 source-deleted PASS");
} catch (error) {
  document.body.dataset.result = "fail";
  document.body.textContent = `FAIL: ${error?.stack ?? error}`;
}
})().catch((error) => { document.body.dataset.result = "fail"; document.body.textContent = `FAIL import: ${error?.stack ?? error}`; });
</script></body></html>"#;
