import { expect, test } from "bun:test";
import { file } from "bun";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import { settleRetiredWasmSession } from "./wasm-test-lifecycle.js";

function completed<T>(action: (done: (value: T) => unknown) => unknown): T {
  const values: T[] = []; action(value => values.push(value)); expect(values).toHaveLength(1); return values[0];
}
function object(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error("expected projected object");
  return value as Record<string, unknown>;
}
const artifact = (name: string) => new URL(`../../../target/created-collections/${name}`, import.meta.url);
const bytes = async (name: string) => [...new Uint8Array(await file(artifact(name)).arrayBuffer())];
const scalar = (channel: string, value: number) => ({ kind: "scalar-input", channel, value });

test("fresh Wasm iterates exact created occurrences, preserves them through checked edits, and passively projects non-game sets", async () => {
  const module = await import(new URL("../../../target/created-collections/wasm/clause_runtime.js", import.meta.url).href);
  module.initSync({ module: await file(artifact("wasm/clause_runtime_bg.wasm")).arrayBuffer() });
  const maximum = Number.MAX_SAFE_INTEGER;
  const policy = workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"],
    workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
  const port = wasm["create-wasm-cartridge-port"](module, policy);
  const open = async (name: string) => {
    const request = wasm["->ExactProcessRequest"](await bytes(name));
    const checked = completed<workbench.PackageCheck>(done => port.acceptPackage(request, done));
    if (checked._tag !== "PackageAccepted") throw new Error(checked.reason);
    const started = completed<workbench.SessionCompletion>(done => port.startSession(checked.acceptedPackage, 1, done));
    if (started._tag !== "SessionStarted") throw new Error(started.reason);
    return started.session;
  };
  let session = await open("initial.cwr1"), revision = 0, sequence = 0;
  const run = (inputs: unknown[] = [], milliseconds = 1000) => {
    const observations = inputs.map(value => workbench["->InputObservation"](++sequence, workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(value)]))));
    const pending = completed<workbench.CandidateCompletion>(done => port.runCandidate(session, workbench["->FixedTick"](milliseconds), workbench["->InputConfiguration"](++revision, observations), done));
    if (pending._tag !== "CandidateProduced") throw new Error(pending.reason);
    const admitted = completed<workbench.AdmissionCompletion>(done => port.requestAdmission(session, pending.candidate, done));
    if (admitted._tag !== "AdmissionAccepted") throw new Error(admitted.reason);
    return object(wasm["decode-projected-term-frame"](admitted.frame));
  };
  const rows = (frame: Record<string, unknown>, relation: string) => {
    const table = object(object(frame.relations)[relation]); expect(table.kind).toBe("relation-table");
    if (!Array.isArray(table.rows)) throw new Error("expected exact rows"); return table.rows.map(object);
  };
  let frame = run([scalar("IgniteDuration", 0.5), scalar("IgniteDuration", 1.5)], 250);
  expect(object(frame["cinder-1"]).vitality).toBe(96.5);
  const before = rows(frame, "burn-target").map(row => object(object(row.subject).identity));
  expect(before).toHaveLength(2); expect(before[0]).not.toEqual(before[1]);
  const entries = object(await file(artifact("entries.json")).json());
  const explanation = object(wasm.explainSession(module, session, Number(entries.tick)));
  const selected = Object.values(object(explanation.rules)).map(object).filter(rule => rule.selected);
  expect(selected.every(rule => object(rule.source).designation === "tick")).toBe(true);
  expect(selected.every(rule => typeof object(object(rule.source).origin).start === "number")).toBe(true);
  expect(selected.every(rule => Object.keys(object(object(rule.source).laws)).length > 0)).toBe(true);
  const effects = selected.flatMap(rule => Object.values(object(rule.effects))).map(object).filter(effect => effect.additive);
  expect(effects).toHaveLength(2); expect(effects.every(effect => object(effect.subject).kind === "referent")).toBe(true);
  expect(selected.every(rule => Object.keys(object(rule.bindings)).length > 0)).toBe(true);
  const previous = session;
  const edited = wasm.editSourceSession(module, session, 2, wasm["->ExactProcessRequest"](await bytes("edited.cwr1")), await bytes("edit.cet1"), policy);
  if (edited._tag !== "SessionStarted") throw new Error(edited.reason);
  session = edited.session; revision = 0; sequence = 0;
  expect(() => wasm.explainSession(module, previous, Number(entries.tick))).toThrow();
  // Reclaim revoked physical storage before subsequent continuity checks and
  // the next independent world; file I/O is not a retirement barrier.
  settleRetiredWasmSession(module);
  frame = run([], 125);
  expect(object(frame["cinder-1"]).vitality).toBe(93);
  const after = rows(frame, "burn-target").map(row => object(object(row.subject).identity));
  expect(after).toEqual(before);
  expect(Object.keys(object(wasm.sourceContinuity(module, session))).length).toBeGreaterThan(0);
  frame = run([], 1000);
  expect(object(frame["cinder-1"]).vitality).toBe(77.25);
  frame = run(); expect(object(frame["cinder-1"]).vitality).toBe(75.5);
  frame = run(); expect(rows(frame, "burn-target")).toHaveLength(0);
  expect(object(frame["cinder-1"]).vitality).toBe(75.5);
  port.disposeSession(session);
  session = await open("goals.cwr1"); revision = 0; sequence = 0;
  frame = run([scalar("GoalDuration", 1), scalar("GoalDuration", 3)], 500);
  expect(object(frame.account).balance).toBe(107);
  expect(object(frame.account)["known-goal"]).toHaveLength(2);
  expect(rows(frame, "known-goal")).toHaveLength(1);
  frame = run([], 500); expect(object(frame.account).balance).toBe(114);
  expect(object(frame.account)["known-goal"]).toHaveLength(1);
  frame = run(); expect(object(frame.account).balance).toBe(121);
  port.disposeSession(session);
}, 60_000);
