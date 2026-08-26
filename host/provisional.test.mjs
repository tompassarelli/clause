import { expect, test } from "bun:test";
import { createEffectBridge, createEventBridge, createTwoMeshBinding, loadArtifact, renderPlanFor, startLifecycle } from "./provisional.mjs";

const revisionId = `rev-sha256-${"a".repeat(64)}`;
const stateId = (digit) => `state-sha256-${digit.repeat(64)}`;
const refId = (digit) => `ref-sha256-${digit.repeat(64)}`;
const playerId = refId("1");
const coinId = refId("2");

const item = (id, xBits, yBits) => ["item", id, ["position-f32x2", xBits, yBits]];
const plan = (stateRevisionId, items, modelRevisionId = revisionId) => [
  "clause-render-plan-v1",
  ["model-revision", modelRevisionId],
  ["state-revision", stateRevisionId],
  ["items", items],
];

function artifactFor(renderPlan, validateTransitionResult = () => true) {
  return loadArtifact({
    kind: "clause-js-runtime-v1",
    revisionId,
    events: { left: refId("5") },
    createRuntime: () => {},
    createEvent: (name, event, payload, order, revisionId) => ({ id: `canonical-occurrence-${order}`, event, name, payload, order, revisionId }),
    validateTransitionResult,
    validateEffectTrace: () => true,
    renderPlan,
  });
}

test("ordered events and canonical render plans stay bound to exact Revisions", () => {
  const movedState = stateId("b");
  const movedPlan = plan(movedState, [item(playerId, "40000000", "00000000")]);
  const artifact = artifactFor((stateRevisionId) => stateRevisionId === movedState ? movedPlan : undefined);
  const calls = [];
  const runtime = { transition: (event, revision) => { calls.push([event.order, revision]); return { revisionId: revision, state: movedState, effects: [] }; } };
  const bridge = createEventBridge({ artifact, runtime });
  expect(bridge.dispatch("left")).toBe(true);
  expect(bridge.events().map((event) => event.order)).toEqual([0]);
  expect(renderPlanFor(artifact, movedState)).toEqual(movedPlan);
  expect(calls).toEqual([[0, revisionId]]);
  expect(bridge.events()[0].id).not.toBe(bridge.events()[0].event);
  expect(() => renderPlanFor(artifact, stateId("c"))).toThrow();
});

test("rejected and throwing transitions never enter history", () => {
  const artifact = artifactFor(
    (stateRevisionId) => plan(stateRevisionId, []),
    (result) => result.ok === true,
  );
  const rejected = createEventBridge({ artifact, runtime: { transition: () => ({ revisionId, ok: false }) } });
  expect(() => rejected.dispatch("left")).toThrow();
  expect(rejected.events()).toEqual([]);
  const throwing = createEventBridge({ artifact, runtime: { transition: () => { throw new Error("reject"); } } });
  expect(() => throwing.dispatch("left")).toThrow();
  expect(throwing.events()).toEqual([]);
});

test("stop during render does not schedule another frame", () => {
  let callback;
  let scheduled = 0;
  const input = { addEventListener() {}, removeEventListener() {}, requestAnimationFrame(fn) { callback = fn; scheduled += 1; return scheduled; }, cancelAnimationFrame() {} };
  const artifact = artifactFor((stateRevisionId) => plan(stateRevisionId, []));
  let lifecycle;
  lifecycle = startLifecycle({ input, artifact, runtime: { transition: () => ({ revisionId, state: stateId("b"), effects: [] }) }, binding: { apply() {} }, render: () => lifecycle.stop() });
  callback();
  expect(scheduled).toBe(1);
});

test("effect traces require Clause validation and lifecycle stop is idempotent", () => {
  const artifact = loadArtifact({
    kind: "clause-js-runtime-v1",
    revisionId,
    capabilities: ["render"],
    events: {},
    createRuntime: () => {},
    createEvent: () => ({ id: "e", event: "render", revisionId }),
    validateTransitionResult: () => true,
    validateEffectTrace: (trace, request, rev) => trace.request === request && trace.revisionId === rev,
    renderPlan: (stateRevisionId) => plan(stateRevisionId, []),
  });
  const bridge = createEffectBridge({ artifact, runtime: { realizeEffect: (request, outcome, rev) => ({ request, outcome, revisionId: rev }) } });
  expect(() => bridge.realize({ capability: "render", revisionId: "wrong" }, "succeeded")).toThrow();
  expect(bridge.realize({ capability: "render", revisionId }, "failed").outcome).toBe("failed");
  let scheduled = 0;
  let disposed = 0;
  const input = { addEventListener() {}, removeEventListener() {}, requestAnimationFrame() { scheduled += 1; return scheduled; }, cancelAnimationFrame() {} };
  const lifecycle = startLifecycle({ input, artifact, runtime: { transition: () => ({ revisionId, state: stateId("b"), effects: [] }) }, binding: { apply() {}, dispose() { disposed += 1; } }, render() {} });
  lifecycle.stop();
  lifecycle.stop();
  expect(scheduled).toBe(1);
  expect(disposed).toBe(1);
});

test("one Clause-owned coin collection replays into an absent coin render plan", () => {
  const initialState = Object.freeze({
    stateRevisionId: stateId("b"),
    player: Object.freeze({ position: Object.freeze([0, 0]), score: 0 }),
    coin: Object.freeze({ position: Object.freeze([10, 0]), active: true, value: 10 }),
  });
  const collectedStateId = stateId("c");
  const steadyStateId = stateId("d");
  const plans = new Map([
    [initialState.stateRevisionId, plan(initialState.stateRevisionId, [
      item(playerId, "00000000", "00000000"),
      item(coinId, "41200000", "00000000"),
    ].sort((left, right) => left[1].localeCompare(right[1])))],
    [collectedStateId, plan(collectedStateId, [item(playerId, "41200000", "00000000")])],
    [steadyStateId, plan(steadyStateId, [item(playerId, "41200000", "00000000")])],
  ]);
  const artifact = loadArtifact({
    kind: "clause-js-runtime-v1",
    revisionId,
    initialState,
    events: { frame: refId("5") },
    createRuntime: () => {},
    createEvent: (name, event, payload, order, revision) => ({
      id: `occurrence-${order}`,
      event,
      name,
      payload,
      order,
      revisionId: revision,
    }),
    validateTransitionResult: (result, event, revision) => result.revisionId === revision && event.event === refId("5"),
    validateEffectTrace: () => true,
    renderPlan: (stateRevisionId) => plans.get(stateRevisionId),
  });
  const inputLog = [
    { name: "frame", payload: { input: "right", dt: 1 } },
    { name: "frame", payload: { input: "right", dt: 1 } },
  ];
  const execute = (entries) => {
    let state = initialState;
    const forwarded = [];
    const rendered = [];
    const runtime = {
      transition(event, revision) {
        forwarded.push({ event, revision });
        state = event.order === 0
          ? { stateRevisionId: collectedStateId, player: { position: [10, 0], score: state.player.score + state.coin.value }, coin: { ...state.coin, active: false } }
          : { stateRevisionId: steadyStateId, player: { ...state.player }, coin: { ...state.coin } };
        return { revisionId: revision, state, effects: [] };
      },
    };
    const bridge = createEventBridge({
      artifact,
      runtime,
      onState: (nextState, revision) => rendered.push(renderPlanFor(artifact, nextState, revision)),
    });
    for (const { name, payload } of entries) expect(bridge.dispatch(name, payload)).toBe(true);
    return {
      accepted: bridge.events().map(({ id, event, name, payload, order, revisionId }) => ({ id, event, name, payload, order, revisionId })),
      finalState: state,
      forwarded: forwarded.map(({ event, revision }) => ({ occurrence: event.id, event: event.event, input: event.payload, order: event.order, revision })),
      plans: rendered,
    };
  };

  expect(renderPlanFor(artifact, initialState)).toEqual(plans.get(initialState.stateRevisionId));
  const first = execute(inputLog);
  const replayLog = JSON.parse(JSON.stringify(first.accepted.map(({ name, payload }) => ({ name, payload }))));
  const replay = execute(replayLog);

  expect(first.forwarded).toEqual([
    { occurrence: "occurrence-0", event: refId("5"), input: { input: "right", dt: 1 }, order: 0, revision: revisionId },
    { occurrence: "occurrence-1", event: refId("5"), input: { input: "right", dt: 1 }, order: 1, revision: revisionId },
  ]);
  expect(first.accepted.map((event) => event.id)).toEqual(["occurrence-0", "occurrence-1"]);
  expect(first.accepted.every((event) => event.id !== event.event)).toBe(true);
  expect(first.plans).toEqual([plans.get(collectedStateId), plans.get(steadyStateId)]);
  expect(first.finalState.player.score).toBe(10);
  expect(first.finalState.coin.active).toBe(false);
  expect(JSON.stringify(replay)).toBe(JSON.stringify(first));
});

test("mesh reconciliation hides omissions and rejects invalid plans before mutation", () => {
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
  expect(() => createTwoMeshBinding(THREE, scene, { revisionId: "wrong", playerId, coinId })).toThrow();
  const binding = createTwoMeshBinding(THREE, scene, { revisionId, playerId, coinId });
  const first = plan(stateId("b"), [
    item(playerId, "3f800000", "40000000"),
    item(coinId, "40400000", "40800000"),
  ].sort((left, right) => left[1].localeCompare(right[1])));
  const omitted = plan(stateId("c"), [item(playerId, "40a00000", "40c00000")]);
  binding.apply(first);
  binding.apply(omitted);
  binding.apply(omitted);
  expect(binding.meshes.player.position).toMatchObject({ x: 5, y: 6, z: 0 });
  expect(binding.meshes.coin.visible).toBe(false);

  const before = JSON.stringify({
    player: [binding.meshes.player.position.x, binding.meshes.player.position.y, binding.meshes.player.position.z, binding.meshes.player.visible],
    coin: [binding.meshes.coin.position.x, binding.meshes.coin.position.y, binding.meshes.coin.position.z, binding.meshes.coin.visible],
  });
  const unknownId = refId("6");
  const invalid = plan(stateId("d"), [
    item(playerId, "41200000", "00000000"),
    item(unknownId, "00000000", "00000000"),
  ].sort((left, right) => left[1].localeCompare(right[1])));
  expect(() => binding.apply(invalid)).toThrow();
  expect(JSON.stringify({
    player: [binding.meshes.player.position.x, binding.meshes.player.position.y, binding.meshes.player.position.z, binding.meshes.player.visible],
    coin: [binding.meshes.coin.position.x, binding.meshes.coin.position.y, binding.meshes.coin.position.z, binding.meshes.coin.visible],
  })).toBe(before);

  const wrongRevision = `rev-sha256-${"b".repeat(64)}`;
  const crossRevision = plan(
    stateId("e"),
    [item(playerId, "41a00000", "00000000")],
    wrongRevision,
  );
  expect(() => binding.apply(crossRevision)).toThrow();
  expect(JSON.stringify({
    player: [binding.meshes.player.position.x, binding.meshes.player.position.y, binding.meshes.player.position.z, binding.meshes.player.visible],
    coin: [binding.meshes.coin.position.x, binding.meshes.coin.position.y, binding.meshes.coin.position.z, binding.meshes.coin.visible],
  })).toBe(before);

  const duplicate = plan(stateId("f"), [
    item(playerId, "41200000", "00000000"),
    item(playerId, "00000000", "00000000"),
  ]);
  expect(() => binding.apply(duplicate)).toThrow();
  expect(binding.meshes.player.position).toMatchObject({ x: 5, y: 6, z: 0 });
  expect(binding.meshes.coin.visible).toBe(false);
});
