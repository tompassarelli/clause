import { expect, test } from "bun:test";
import { createEffectBridge, createEventBridge, loadArtifact, renderPlanFor, startLifecycle } from "./provisional.mjs";

const revisionId = `rev-sha256-${"a".repeat(64)}`;

test("ordered events and pure render plans stay bound to one Revision", () => {
  const artifact = loadArtifact({
    kind: "clause-js-runtime-v1", revisionId, events: { left: "ref-sha256-left" },
    createRuntime: () => {}, createEvent: (name, event, payload, order, revisionId) => ({ id: "canonical-occurrence-id", event, name, payload, order, revisionId }), validateTransitionResult: () => true, validateEffectTrace: () => true, renderPlan: (state, revision) => [{ id: "player", position: [state.x, 0, 0], revision }],
  });
  const calls = [];
  const runtime = { transition: (event, revision) => { calls.push([event.order, revision]); return { revisionId: revision, state: { x: event.order }, effects: [] }; } };
  const bridge = createEventBridge({ artifact, runtime });
  expect(bridge.dispatch("left")).toBe(true);
  expect(bridge.events().map((event) => event.order)).toEqual([0]);
  expect(renderPlanFor(artifact, { x: 2 })).toEqual([{ id: "player", position: [2, 0, 0], revision: revisionId }]);
  expect(calls).toEqual([[0, revisionId]]);
  expect(bridge.events()[0].id).not.toBe(bridge.events()[0].event);
});

test("rejected and throwing transitions never enter history", () => {
  const artifact = loadArtifact({ kind: "clause-js-runtime-v1", revisionId, events: { left: "ref-event" }, createRuntime: () => {}, createEvent: (name, event, payload, order, revisionId) => ({ id: `occ-${order}`, event, revisionId }), validateTransitionResult: (result) => result.ok === true, validateEffectTrace: () => true, renderPlan: () => [] });
  const rejected = createEventBridge({ artifact, runtime: { transition: () => ({ revisionId, ok: false }) } });
  expect(() => rejected.dispatch("left")).toThrow(); expect(rejected.events()).toEqual([]);
  const throwing = createEventBridge({ artifact, runtime: { transition: () => { throw new Error("reject"); } } });
  expect(() => throwing.dispatch("left")).toThrow(); expect(throwing.events()).toEqual([]);
});

test("stop during render does not schedule another frame", () => {
  let callback; let scheduled = 0; const input = { addEventListener() {}, removeEventListener() {}, requestAnimationFrame(fn) { callback = fn; scheduled += 1; return scheduled; }, cancelAnimationFrame() {} };
  const artifact = loadArtifact({ kind: "clause-js-runtime-v1", revisionId, createRuntime: () => {}, createEvent: () => ({ id: "e", event: "x", revisionId }), validateTransitionResult: () => true, validateEffectTrace: () => true, renderPlan: () => [] });
  let lifecycle; lifecycle = startLifecycle({ input, artifact, runtime: { transition: () => ({ revisionId, state: {}, effects: [] }) }, binding: { apply() {} }, render: () => lifecycle.stop() });
  callback(); expect(scheduled).toBe(1);
});

test("effect traces require Clause validation and lifecycle stop is idempotent", () => {
  const artifact = loadArtifact({ kind: "clause-js-runtime-v1", revisionId, capabilities: ["render"], createRuntime: () => {}, createEvent: () => ({ id: "e", event: "render", revisionId }), validateTransitionResult: () => true, validateEffectTrace: (trace, request, rev) => trace.request === request && trace.revisionId === rev, renderPlan: () => [] });
  const bridge = createEffectBridge({ artifact, runtime: { realizeEffect: (request, outcome, rev) => ({ request, outcome, revisionId: rev }) } });
  expect(() => bridge.realize({ capability: "render", revisionId: "wrong" }, "succeeded")).toThrow();
  expect(bridge.realize({ capability: "render", revisionId }, "failed").outcome).toBe("failed");
  let scheduled = 0; let disposed = 0; const input = { addEventListener() {}, removeEventListener() {}, requestAnimationFrame() { scheduled += 1; return scheduled; }, cancelAnimationFrame() {} };
  const lifecycle = startLifecycle({ input, artifact, runtime: { transition: () => ({ revisionId, state: {}, effects: [] }) }, binding: { apply() {}, dispose() { disposed += 1; } }, render() {} });
  lifecycle.stop(); lifecycle.stop(); expect(scheduled).toBe(1); expect(disposed).toBe(1);
});
