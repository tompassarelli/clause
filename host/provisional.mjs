// Provisional browser host for a generated Clause runtime artifact.
//
// This module deliberately contains no Clause evaluator.  The artifact owns
// transition semantics and render-plan production; this file only adapts
// browser/Three.js lifecycle and retains copies of Clause-validated traces.

const ARTIFACT_KIND = "clause-js-runtime-v1";

const copy = (value) => {
  if (Array.isArray(value)) return Object.freeze(value.map(copy));
  if (value && typeof value === "object") {
    return Object.freeze(Object.fromEntries(Object.entries(value).map(([k, v]) => [k, copy(v)])));
  }
  return value;
};

function requireString(value, name) {
  if (typeof value !== "string" || value.length === 0) throw new Error(`${name} must be a non-empty string`);
  return value;
}

function requireRevisionId(value, name) {
  requireString(value, name);
  if (!/^rev-sha256-[0-9a-f]{64}$/.test(value)) throw new Error(`invalid ${name}`);
  return value;
}

function requireStateRevisionId(value, name) {
  requireString(value, name);
  if (!/^state-sha256-[0-9a-f]{64}$/.test(value)) throw new Error(`invalid ${name}`);
  return value;
}

function requireReferentId(value, name) {
  requireString(value, name);
  if (!/^ref-sha256-[0-9a-f]{64}$/.test(value)) throw new Error(`invalid ${name}`);
  return value;
}

function requireArray(value, length, name) {
  if (!Array.isArray(value) || (length !== undefined && value.length !== length)) throw new Error(`invalid ${name}`);
  return value;
}

function f32FromBits(value) {
  if (!/^[0-9a-f]{8}$/.test(value)) throw new Error("invalid canonical f32 bits");
  const bits = Number.parseInt(value, 16);
  if ((bits & 0x7f800000) === 0x7f800000 || bits === 0x80000000) throw new Error("invalid canonical finite f32 bits");
  const bytes = new ArrayBuffer(4);
  const view = new DataView(bytes);
  view.setUint32(0, bits, false);
  return view.getFloat32(0, false);
}

function validateRenderPlan(plan, expectedRevisionId, expectedStateRevisionId) {
  const envelope = requireArray(plan, 4, "RenderPlan envelope");
  if (envelope[0] !== "clause-render-plan-v1") throw new Error("unsupported RenderPlan");
  const model = requireArray(envelope[1], 2, "RenderPlan Model Revision");
  if (model[0] !== "model-revision") throw new Error("invalid RenderPlan Model Revision tag");
  const revisionId = requireRevisionId(model[1], "RenderPlan Model Revision identity");
  if (expectedRevisionId !== undefined && revisionId !== expectedRevisionId) throw new Error("render plan names the wrong Model Revision");
  const state = requireArray(envelope[2], 2, "RenderPlan StateRevision");
  if (state[0] !== "state-revision") throw new Error("invalid RenderPlan StateRevision tag");
  const stateRevisionId = requireStateRevisionId(state[1], "RenderPlan StateRevision identity");
  if (expectedStateRevisionId !== undefined && stateRevisionId !== expectedStateRevisionId) throw new Error("render plan names the wrong StateRevision");
  const itemsField = requireArray(envelope[3], 2, "RenderPlan items field");
  if (itemsField[0] !== "items") throw new Error("invalid RenderPlan items tag");
  const items = requireArray(itemsField[1], undefined, "RenderPlan items");
  let previousId;
  const decoded = items.map((value) => {
    const item = requireArray(value, 3, "RenderPlan item");
    if (item[0] !== "item") throw new Error("invalid RenderPlan item tag");
    const id = requireReferentId(item[1], "RenderPlan item identity");
    if (previousId !== undefined && previousId >= id) throw new Error("RenderPlan items must be strictly canonical");
    previousId = id;
    const position = requireArray(item[2], 3, "RenderPlan position");
    if (position[0] !== "position-f32x2") throw new Error("invalid RenderPlan position tag");
    return Object.freeze({ id, position: Object.freeze([f32FromBits(position[1]), f32FromBits(position[2])]) });
  });
  return Object.freeze({ revisionId, stateRevisionId, items: Object.freeze(decoded) });
}

function stateIdentity(state) {
  return requireStateRevisionId(
    typeof state === "string" ? state : state?.stateRevisionId,
    "runtime StateRevision identity",
  );
}

/** Load generated, specialized artifact data without decoding Clause wire data. */
export function loadArtifact(value) {
  if (typeof value === "string") throw new Error("generated artifact must be an ESM/module object");
  if (!value || value.kind !== ARTIFACT_KIND) throw new Error("unsupported Clause runtime artifact");
  requireRevisionId(value.revisionId, "artifact Revision identity");
  if (typeof value.createRuntime !== "function") throw new Error("artifact createRuntime is required");
  if (typeof value.renderPlan !== "function") throw new Error("artifact renderPlan is required");
  if (typeof value.createEvent !== "function") throw new Error("artifact createEvent is required");
  if (typeof value.validateEffectTrace !== "function") throw new Error("artifact validateEffectTrace is required");
  if (typeof value.validateTransitionResult !== "function") throw new Error("artifact validateTransitionResult is required");
  const events = value.events ?? {};
  if (!events || typeof events !== "object") throw new Error("artifact events must be an object");
  return Object.freeze({
    kind: ARTIFACT_KIND,
    revisionId: value.revisionId,
    initialState: copy(value.initialState ?? null),
    events: copy(events),
    createRuntime: value.createRuntime,
    createEvent: value.createEvent,
    renderPlan: value.renderPlan,
    capabilities: copy(value.capabilities ?? []),
    validateEffectTrace: value.validateEffectTrace,
    validateTransitionResult: value.validateTransitionResult,
  });
}

/** Pure projection of one exact post-transition state into render data. */
export function renderPlanFor(artifact, state, revisionId = artifact.revisionId) {
  if (revisionId !== artifact.revisionId) throw new Error("render plan names the wrong Model Revision");
  const stateRevisionId = stateIdentity(state);
  const plan = artifact.renderPlan(stateRevisionId, revisionId);
  validateRenderPlan(plan, revisionId, stateRevisionId);
  return copy(plan);
}

/** Deterministic ordered event bridge. Device callbacks only enqueue declared events. */
export function createEventBridge({ artifact, runtime, revisionId = artifact.revisionId, onState, onEffects }) {
  if (revisionId !== artifact.revisionId) throw new Error("event bridge names the wrong Model Revision");
  if (!runtime || typeof runtime.transition !== "function") throw new Error("runtime transition is required");
  let order = 0;
  let closed = false;
  const log = [];
  const dispatch = (name, payload = []) => {
    if (closed) return false;
    const eventId = artifact.events[name];
    if (!eventId) return false;
    const hostOrder = order;
    const event = artifact.createEvent(name, eventId, payload, hostOrder, revisionId);
    if (!event || event.revisionId !== revisionId || event.event !== eventId || typeof event.id !== "string" || event.id.length === 0) throw new Error("artifact supplied an invalid Revision-bound event");
    const result = runtime.transition(event, revisionId);
    if (!result || result.revisionId !== revisionId) throw new Error("runtime transition crossed Revision boundary");
    if (!artifact.validateTransitionResult(result, event, revisionId)) throw new Error("Clause transition authority rejected result");
    log.push(copy(event));
    order += 1;
    if (onState) onState(result.state, revisionId);
    if (onEffects && result.effects) onEffects(result.effects, result.state, revisionId);
    return true;
  };
  return Object.freeze({ dispatch, events: () => copy(log), close: () => { closed = true; } });
}

/** Reconcile a total desired-scene plan to two stable meshes. */
export function createTwoMeshBinding(THREE, scene, { revisionId, playerId, coinId }) {
  if (!THREE || !scene || typeof scene.add !== "function") throw new Error("Three.js scene is required");
  requireRevisionId(revisionId, "binding Model Revision identity");
  requireReferentId(playerId, "player identity");
  requireReferentId(coinId, "coin identity");
  if (playerId === coinId) throw new Error("player and coin identities must be distinct");
  const player = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), new THREE.MeshBasicMaterial({ color: 0x3366ff }));
  const coin = new THREE.Mesh(new THREE.CylinderGeometry(0.35, 0.35, 0.1, 16), new THREE.MeshBasicMaterial({ color: 0xffcc00 }));
  scene.add(player); scene.add(coin);
  const meshes = new Map([[playerId, player], [coinId, coin]]);
  const apply = (plan) => {
    const desired = validateRenderPlan(plan, revisionId);
    if (desired.items.some((item) => !meshes.has(item.id))) throw new Error("RenderPlan names an unregistered mesh identity");
    const byId = new Map(desired.items.map((item) => [item.id, item]));
    for (const [id, mesh] of meshes) {
      const item = byId.get(id);
      if (item) {
        mesh.position.set(item.position[0], item.position[1], 0);
        mesh.visible = true;
      } else {
        mesh.visible = false;
      }
    }
  };
  const dispose = () => meshes.forEach((mesh) => {
    scene.remove(mesh); mesh.geometry.dispose(); mesh.material.dispose();
  });
  return Object.freeze({ meshes: Object.freeze({ player, coin }), apply, dispose });
}

/** Browser RAF/input lifecycle; scheduling remains outside Clause semantics. */
export function startLifecycle({ canvas, runtime, artifact, binding, render, input = window }) {
  const bridge = createEventBridge({ artifact, runtime, onState: (state, revision) => binding.apply(renderPlanFor(artifact, state, revision)) });
  let frame;
  let stopped = false;
  const tick = () => { if (stopped) return; render(); if (!stopped) frame = input.requestAnimationFrame(tick); };
  const key = (event) => { const name = event.key === "ArrowLeft" ? "left" : event.key === "ArrowRight" ? "right" : null; if (name) bridge.dispatch(name); };
  input.addEventListener("keydown", key); frame = input.requestAnimationFrame(tick);
  return Object.freeze({ bridge, stop: () => { if (stopped) return; stopped = true; input.cancelAnimationFrame(frame); input.removeEventListener("keydown", key); bridge.close(); if (binding && typeof binding.dispose === "function") binding.dispose(); } });
}

export function createEffectBridge({ artifact, runtime, revisionId = artifact.revisionId }) {
  if (revisionId !== artifact.revisionId || !runtime || typeof runtime.realizeEffect !== "function" || typeof artifact.validateEffectTrace !== "function") throw new Error("effect authority is required");
  const declared = Object.freeze([...artifact.capabilities]);
  const traces = [];
  return Object.freeze({
    realize(request, outcome) {
      if (!request || !declared.includes(request.capability) || request.revisionId !== revisionId) throw new Error("effect request is not declared or is bound to another Revision");
      const trace = runtime.realizeEffect(request, outcome, revisionId);
      if (!artifact.validateEffectTrace(trace, request, revisionId)) throw new Error("Clause effect authority rejected trace");
      traces.push(copy(trace)); return copy(trace);
    },
    entries: () => copy(traces),
  });
}

export { ARTIFACT_KIND };
