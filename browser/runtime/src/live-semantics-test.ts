import { expect, test } from "bun:test";
import { file } from "bun";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";

function completed<T>(action: (done: (value: T) => unknown) => unknown): T {
  const values: T[] = []; action(value => values.push(value));
  expect(values.length).toBe(1); return values[0];
}
function object(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error("expected diagnostic object");
  return value as Record<string, unknown>;
}
const artifact = (name: string) => new URL(`../../../target/live-semantics/${name}`, import.meta.url);
const bytes = async (name: string) => [...new Uint8Array(await file(artifact(name)).arrayBuffer())];

test("fresh live Wasm preserves actual encounter, explains executed contributions, and searches isolated finite interventions", async () => {
  const module = await import(new URL("../../../target/live-semantics/wasm/clause_runtime.js", import.meta.url).href);
  module.initSync({ module: await file(artifact("wasm/clause_runtime_bg.wasm")).arrayBuffer() });
  const entries = object(await file(artifact("entries.json")).json());
  const maximum = Number.MAX_SAFE_INTEGER;
  const policy = workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"],
    workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
  const port = wasm["create-wasm-cartridge-port"](module, policy);
  const initialBytes = await bytes("initial.cwr1");
  const accepted = completed<workbench.PackageCheck>(done => port.acceptPackage(wasm["->ExactProcessRequest"](initialBytes), done));
  if (accepted._tag !== "PackageAccepted") throw new Error(accepted.reason);
  const started = completed<workbench.SessionCompletion>(done => port.startSession(accepted.acceptedPackage, 1, done));
  if (started._tag !== "SessionStarted") throw new Error(started.reason);
  let session = started.session;
  let generation = 1;
  let revision = 0;
  let sequence = 0;
  const candidate = (values: unknown[] = [], milliseconds = 16) => {
    const observations = values.map(value => workbench["->InputObservation"](++sequence, workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(value)]))));
    return completed<workbench.CandidateCompletion>(done => port.runCandidate(session, workbench["->FixedTick"](milliseconds), workbench["->InputConfiguration"](++revision, observations), done));
  };
  const admit = (pending: workbench.CandidateCompletion) => {
    if (pending._tag !== "CandidateProduced") throw new Error(pending.reason);
    const result = completed<workbench.AdmissionCompletion>(done => port.requestAdmission(session, pending.candidate, done));
    if (result._tag !== "AdmissionAccepted") throw new Error(result.reason);
    return object(wasm["decode-projected-term-frame"](result.frame));
  };
  const key = (code: string) => ({ kind: "keyboard", code, phase: "down", repeat: false });
  const initial = admit(candidate());
  admit(candidate([key("BeginEncounter")]));
  let frame = admit(candidate([
    { kind: "scalar-input", channel: "PointerWorldX", value: 2 },
    { kind: "scalar-input", channel: "PointerWorldZ", value: 4 }, key("IssueMove"), key("Attack"),
  ]));
  expect(object(frame["cinder-1"]).vitality).toBe(9);
  const firstExplanation = object(wasm.explainSession(module, session, Number(entries.attack)));
  const selected = Object.values(object(firstExplanation.rules)).map(object).filter(rule => rule.selected);
  expect(selected).toHaveLength(5);
  for (const rule of selected) {
    expect(object(rule.source).designation).toBe("party-attack");
    expect(typeof object(object(rule.source).origin).start).toBe("number");
    expect(Object.keys(object(rule.premises)).length).toBeGreaterThan(0);
  }
  for (let turn = 0; turn < 51; turn += 1) frame = admit(candidate());
  expect(object(wasm.explainSession(module, session, Number(entries.attack))).step).toBe(firstExplanation.step);
  const previousSession = session;
  const previousFrame = frame;
  const witness = await bytes("edit.cet1");
  const replacement = wasm["->ExactProcessRequest"](await bytes("edited.cwr1"));
  const tampered = [...witness]; tampered[4] ^= 1;
  expect(wasm.editSourceSession(module, session, 2, replacement, tampered, policy)._tag).toBe("SessionFailed");
  expect(object(wasm.explainSession(module, session, Number(entries.attack))).step).toBe(firstExplanation.step);
  const editStarted = performance.now();
  const edited = wasm.editSourceSession(module, session, 2, replacement, witness, policy);
  console.log(`checked live Wasm edit milliseconds=${(performance.now() - editStarted).toFixed(2)}`);
  if (edited._tag !== "SessionStarted") throw new Error(edited.reason);
  session = edited.session; generation = 2; revision = 0; sequence = 0;
  expect(() => wasm.explainSession(module, previousSession, Number(entries.attack))).toThrow();
  frame = admit(candidate());
  expect(object(frame["cinder-1"]).vitality).toBe(9);
  expect(object(frame["warrior-1"]).selected).toBe(object(previousFrame["warrior-1"]).selected);
  expect(object(frame["warrior-1"])["$referents"]).not.toEqual(object(previousFrame["warrior-1"])["$referents"]);
  expect(frame["$source-snapshot"]).not.toEqual(previousFrame["$source-snapshot"]);
  const continuity = object(wasm.sourceContinuity(module, session));
  const formations = Object.values(object(continuity.formations)).flatMap(page => Object.values(object(page))).map(object);
  const oldFacets = Object.values(object(object(previousFrame["warrior-1"])["$referents"])).map(object);
  const newFacets = Object.values(object(object(frame["warrior-1"])["$referents"])).map(object);
  expect(oldFacets).toHaveLength(2); expect(newFacets).toHaveLength(2);
  for (const old of oldFacets) {
    const domainMap = formations.find(mapping => mapping.old === old.domain)!;
    const identityMap = formations.find(mapping => mapping.old === object(old.identity).value)!;
    const retained = newFacets.find(reference => reference.domain === domainMap.new)!;
    expect(object(retained.identity).value).toBe(identityMap.new);
    expect(identityMap["occurrence-coordinate"]).toBe(object(old.identity).value);
    expect(identityMap["occurrence-snapshot"]).toBe(continuity["old-snapshot"]);
    expect(identityMap.occurrence).toMatch(/^[0-9a-f]{64}$/);
  }
  const attack = candidate([key("Attack")]);
  const explanation = object(wasm.explainSession(module, session, Number(entries.editedAttack)));
  const states = object(explanation.states);
  const allowed: wasm.FiniteScalarChange[] = [];
  let target = -1;
  for (const [coordinate, incoming] of Object.entries(states)) {
    const state = object(incoming); const source = object(state.source);
    if (source.relation === "selected") allowed.push({ slot: Number(coordinate), value: false });
    if (source.subject === "cinder-1" && source.relation === "vitality") target = Number(coordinate);
  }
  expect(allowed).toHaveLength(5); expect(target).toBeGreaterThanOrEqual(0);
  expect(object(states[String(target)]).before).toBe(9);
  expect(object(states[String(target)]).after).toBe(-173);
  const event = String(explanation.step);
  const answer = object(wasm.interveneSession(module, session, wasm.finiteScalarInterventionQuery(event, allowed, 32, { slot: target, greaterThan: 0 })));
  expect(answer.found).toBe(true); expect(answer.cost).toBe(5); expect(answer.exhausted).toBe(false);
  const bounded = object(wasm.interveneSession(module, session, wasm.finiteScalarInterventionQuery(event, allowed, 1, { slot: target, greaterThan: 0 })));
  expect(bounded.exhausted).toBe(true); expect(bounded.completed).toBe(false); expect(bounded.found).toBe(false);
  const exhaustive = object(wasm.interveneSession(module, session, wasm.finiteScalarInterventionQuery(event, allowed, 32, false)));
  expect(exhaustive.completed).toBe(true); expect(exhaustive.evaluations).toBe(32);
  expect(object(wasm.explainSession(module, session, Number(entries.editedAttack))).step).toBe(event);
  frame = admit(attack);
  expect(object(frame["cinder-1"]).vitality).toBe(-173); // queries did not alter the hidden candidate
  const oldPick = object(object(initial["warrior-1"])["$referents"])[String(object(initial["$referent-inputs"]).Pick)];
  expect(candidate([{ kind: "referent-input", generation: generation - 1, channel: "Pick", value: oldPick }])._tag).toBe("CandidateFailed");
  port.disposeSession(session);
}, 60_000);
