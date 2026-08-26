import assert from "node:assert/strict";
import test from "node:test";
import { createEffectBridge, createEventBridge, createReceiptLog, loadArtifact, renderPlanFor } from "./provisional.mjs";

const revisionId = `rev-sha256-${"a".repeat(64)}`;

test("ordered events and pure render plans stay bound to one Revision", () => {
  const artifact = loadArtifact({
    kind: "clause-js-runtime-v1", revisionId, events: { left: "ref-sha256-left" },
    createRuntime: () => {}, createEvent: (name, event, payload, order, revisionId) => ({ id: "canonical-event-id", name, event, payload, order, revisionId }), renderPlan: (state, revision) => [{ id: "player", position: [state.x, 0, 0], revision }],
  });
  const calls = [];
  const runtime = { transition: (event, revision) => { calls.push([event.order, revision]); return { revisionId: revision, state: { x: event.order }, effects: [] }; } };
  const bridge = createEventBridge({ artifact, runtime });
  assert.equal(bridge.dispatch("left"), true);
  assert.deepEqual(bridge.events().map((event) => event.order), [0]);
  assert.deepEqual(renderPlanFor(artifact, { x: 2 }), [{ id: "player", position: [2, 0, 0], revision: revisionId }]);
  assert.deepEqual(calls, [[0, revisionId]]);
});

test("receipt construction stays in Clause authority and capability declarations are snapshotted", () => {
  const log = createReceiptLog(["render"]);
  assert.throws(() => log.realize({ capability: "audio", requestId: "x" }));
  assert.throws(() => log.realize({ capability: "render", requestId: "r" }));
  const caps = ["render"];
  const artifact = loadArtifact({ kind: "clause-js-runtime-v1", revisionId, capabilities: caps, createRuntime: () => {}, createEvent: () => ({ id: "e", revisionId }), renderPlan: () => [] });
  caps[0] = "audio";
  const bridge = createEffectBridge({ artifact, runtime: { realizeEffect: (request, outcome, revisionId) => ({ request, outcome, revisionId }) } });
  assert.deepEqual(bridge.realize({ capability: "render", revisionId }, "failed").outcome, "failed");
  assert.throws(() => bridge.realize({ capability: "audio", revisionId }, "succeeded"));
});
