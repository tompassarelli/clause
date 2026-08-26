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

/** Load generated, specialized artifact data without decoding Clause wire data. */
export function loadArtifact(value) {
  if (typeof value === "string") throw new Error("generated artifact must be an ESM/module object");
  if (!value || value.kind !== ARTIFACT_KIND) throw new Error("unsupported Clause runtime artifact");
  requireString(value.revisionId, "artifact revisionId");
  if (!/^rev-sha256-[0-9a-f]{64}$/.test(value.revisionId)) throw new Error("invalid artifact Revision identity");
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
  const plan = artifact.renderPlan(state, revisionId);
  if (!Array.isArray(plan)) throw new Error("artifact renderPlan must return an array");
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

/** Convert a pure plan to two stable meshes; no semantic decisions occur here. */
export function createTwoMeshBinding(THREE, scene) {
  if (!THREE || !scene || typeof scene.add !== "function") throw new Error("Three.js scene is required");
  const player = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), new THREE.MeshBasicMaterial({ color: 0x3366ff }));
  const coin = new THREE.Mesh(new THREE.CylinderGeometry(0.35, 0.35, 0.1, 16), new THREE.MeshBasicMaterial({ color: 0xffcc00 }));
  scene.add(player); scene.add(coin);
  const meshes = { player, coin };
  const apply = (plan) => {
    for (const item of plan) {
      const mesh = meshes[item.id];
      if (!mesh) continue;
      if (Array.isArray(item.position) && item.position.length === 3) mesh.position.set(...item.position);
      if (typeof item.visible === "boolean") mesh.visible = item.visible;
    }
  };
  const dispose = () => Object.values(meshes).forEach((mesh) => {
    scene.remove(mesh); mesh.geometry.dispose(); mesh.material.dispose();
  });
  return Object.freeze({ meshes, apply, dispose });
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
