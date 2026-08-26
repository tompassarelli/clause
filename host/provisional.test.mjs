import assert from "node:assert/strict";
import test from "node:test";
import { createEffectBridge, createEventBridge, loadArtifact, renderPlanFor, startLifecycle } from "./provisional.mjs";

const revisionId = `rev-sha256-${"a".repeat(64)}`;

test("ordered events and pure render plans stay bound to one Revision", () => {
  const artifact = loadArtifact({
    kind: "clause-js-runtime-v1", revisionId, events: { left: "ref-sha256-left" },
    createRuntime: () => {}, createEvent: (name, event, payload, order, revisionId) => ({ id: "canonical-event-id", name, event, payload, order, revisionId }), validateEffectTrace: () => true, renderPlan: (state, revision) => [{ id: "player", position: [state.x, 0, 0], revision }],
  });
  const calls = [];
  const runtime = { transition: (event, revision) => { calls.push([event.order, revision]); return { revisionId: revision, state: { x: event.order }, effects: [] }; } };
  const bridge = createEventBridge({ artifact, runtime });
  assert.equal(bridge.dispatch("left"), true);
  assert.deepEqual(bridge.events().map((event) => event.order), [0]);
  assert.deepEqual(renderPlanFor(artifact, { x: 2 }), [{ id: "player", position: [2, 0, 0], revision: revisionId }]);
  assert.deepEqual(calls, [[0, revisionId]]);
});

test("effect traces require Clause validation and lifecycle stop is idempotent", () => {
  const artifact = loadArtifact({ kind: "clause-js-runtime-v1", revisionId, capabilities: ["render"], createRuntime: () => {}, createEvent: () => ({ id: "e", revisionId }), validateEffectTrace: (trace, request, rev) => trace.request === request && trace.revisionId === rev, renderPlan: () => [] });
  const bridge = createEffectBridge({ artifact, runtime: { realizeEffect: (request, outcome, rev) => ({ request, outcome, revisionId: rev }) } });
  assert.throws(() => bridge.realize({ capability: "render", revisionId: "wrong" }, "succeeded"));
  assert.deepEqual(bridge.realize({ capability: "render", revisionId }, "failed").outcome, "failed");
  let scheduled = 0; const input = { addEventListener() {}, removeEventListener() {}, requestAnimationFrame() { scheduled += 1; return scheduled; }, cancelAnimationFrame() {} };
  const lifecycle = startLifecycle({ input, artifact, runtime: { transition: () => ({ revisionId, state: {}, effects: [] }) }, binding: { apply() {} }, render() {} });
  lifecycle.stop(); lifecycle.stop(); assert.equal(scheduled, 1);
});
