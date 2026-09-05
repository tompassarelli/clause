import { expect, test } from "bun:test";
import { file } from "bun";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";

function completed<T>(action: (done: (value: T) => unknown) => unknown): T {
  const values: T[] = [];
  action(value => values.push(value));
  expect(values.length).toBe(1);
  return values[0];
}

function object(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error("expected a projected object");
  return value as Record<string, unknown>;
}

test("fresh Wasm transports exact projected referents through the passive browser input adapter", async () => {
  const generated = new URL("../../../target/referent-wasm/clause_runtime.js", import.meta.url);
  const module = await import(generated.href);
  module.initSync({ module: await file(new URL("../../../target/referent-wasm/clause_runtime_bg.wasm", import.meta.url)).arrayBuffer() });
  const maximum = Number.MAX_SAFE_INTEGER;
  const policy = workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"], workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
  const port = wasm["create-wasm-cartridge-port"](module, policy);
  const bytes = [...new Uint8Array(await file(new URL("../../../target/referent-input.cwr1", import.meta.url)).arrayBuffer())];
  const accepted = completed<workbench.PackageCheck>(done => port.acceptPackage(wasm["->ExactProcessRequest"](bytes), done));
  if (accepted._tag !== "PackageAccepted") throw new Error(accepted.reason);
  const start = (generation: number) => {
    const result = completed<workbench.SessionCompletion>(done => port.startSession(accepted.acceptedPackage, generation, done));
    if (result._tag !== "SessionStarted") throw new Error(result.reason);
    return result;
  };
  const started = start(1);
  let revision = 0;
  let sequence = 0;
  const run = (session: unknown, value?: unknown) => {
    const observations = value === undefined ? [] : [workbench["->InputObservation"](++sequence, workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(value)])))];
    return completed<workbench.CandidateCompletion>(done => port.runCandidate(session, workbench["->FixedTick"](100), workbench["->InputConfiguration"](++revision, observations), done));
  };
  const admit = (session: unknown, candidate: workbench.CandidateCompletion) => {
    if (candidate._tag !== "CandidateProduced") throw new Error(candidate.reason);
    const result = completed<workbench.AdmissionCompletion>(done => port.requestAdmission(session, candidate.candidate, done));
    if (result._tag !== "AdmissionAccepted") throw new Error(result.reason);
    return object(wasm["decode-projected-term-frame"](result.frame));
  };
  const initial = admit(started.session, run(started.session));
  const first = object(initial.first)["$referent"];
  const second = object(initial.second)["$referent"];
  expect(first).not.toEqual(second);
  expect(object(first).domain).toEqual(object(second).domain);
  const picked = admit(started.session, run(started.session, { kind: "referent-input", generation: 1, channel: "Pick", value: first }));
  expect(object(picked.first).selected).toBe(true);
  expect(object(picked.second).selected).toBe(false);
  expect(object(picked.first).progress).toBe(0.1);
  expect(object(picked.second).progress).toBe(0);
  const both = admit(started.session, run(started.session, { kind: "referent-input", generation: 1, channel: "Pick", value: second }));
  expect(object(both.second).progress).toBe(0.1);
  for (const value of [3, { ...object(first), domain: Number(object(first).domain) + 1 }, { kind: "referent", domain: object(first).domain, identity: { kind: "created", value: [1, 2] } }]) {
    expect(run(started.session, { kind: "referent-input", generation: 1, channel: "Pick", value })._tag).toBe("CandidateFailed");
  }
  const replacement = start(2);
  const stale = run(replacement.session, { kind: "referent-input", generation: 1, channel: "Pick", value: first });
  expect(stale._tag).toBe("CandidateFailed");
  if (stale._tag === "CandidateFailed") expect(stale.reason).toContain("stale generation");
  const unchanged = admit(replacement.session, run(replacement.session));
  expect(object(unchanged.first).selected).toBe(false);
  port.disposeSession(replacement.session);
});
