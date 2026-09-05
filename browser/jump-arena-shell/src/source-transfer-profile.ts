// Passive, serialized exact-witness driver. No renderer, rule evaluator or
// synthetic world-state import participates in the profiled operation.
import { file, write, CryptoHasher } from "bun";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";

function completed<T>(action: (done: (value: T) => unknown) => unknown): T {
  const values: T[] = []; action(value => values.push(value));
  if (values.length !== 1) throw new Error("expected one boundary completion"); return values[0];
}
function object(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error("expected profile object");
  return value as Record<string, unknown>;
}
const directory = Bun.argv[2];
const samples = Number(Bun.argv[3] ?? "3");
if (!directory || !Number.isInteger(samples) || samples < 1 || samples > 5) throw new Error("expected artifact directory and 1..5 samples");
const location = (name: string) => `${directory}/${name}`;
const bytes = async (name: string) => [...new Uint8Array(await file(location(name)).arrayBuffer())];
const runtime = await file(location("wasm/clause_runtime_bg.wasm")).arrayBuffer();
const module = await import(location("wasm/clause_runtime.js"));
module.initSync({ module: runtime });
const maximum = Number.MAX_SAFE_INTEGER;
const policy = workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"],
  workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
const port = wasm["create-wasm-cartridge-port"](module, policy);
const variants: Record<string, unknown> = {};
for (const name of ["encounter", "collections"]) {
  const source = await file(location(`${name}/source.clause`)).arrayBuffer();
  const initial = await bytes(`${name}/initial.cwr1`), edited = await bytes(`${name}/edited.cwr1`), witness = await bytes(`${name}/edit.cet1`);
  const observations: unknown[] = [];
  for (let index = 0; index <= samples * 2; ++index) {
    const profiled = index > 0 && index % 2 === 0;
    const checked = completed<workbench.PackageCheck>(done => port.acceptPackage(wasm["->ExactProcessRequest"](initial), done));
    if (checked._tag !== "PackageAccepted") throw new Error(checked.reason);
    const started = completed<workbench.SessionCompletion>(done => port.startSession(checked.acceptedPackage, 1, done));
    if (started._tag !== "SessionStarted") throw new Error(started.reason);
    const key = (code: string) => ({ kind: "keyboard", code, phase: "down", repeat: false });
    const values: unknown[] = [key("BeginEncounter"), key("Attack")];
    if (name === "collections") values.push(...[1, 3].map(value => ({ kind: "scalar-input", channel: "IgniteDuration", value })));
    const inputs = values.map((value, index) => workbench["->InputObservation"](index + 1, workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(value)]))));
    const candidate = completed<workbench.CandidateCompletion>(done => port.runCandidate(started.session, workbench["->FixedTick"](16), workbench["->InputConfiguration"](1, inputs), done));
    if (candidate._tag !== "CandidateProduced") throw new Error(candidate.reason);
    const admitted = completed<workbench.AdmissionCompletion>(done => port.requestAdmission(started.session, candidate.candidate, done));
    if (admitted._tag !== "AdmissionAccepted") throw new Error(admitted.reason);
    if (profiled && !module.clause_source_profile_v1_begin()) throw new Error("profile already active");
    const before = performance.now();
    const result = wasm.editSourceSession(module, started.session, 2, wasm["->ExactProcessRequest"](edited), witness, policy);
    const wallMs = performance.now() - before;
    const profile: unknown = profiled ? JSON.parse(module.clause_source_profile_v1_finish()) : null;
    if (result._tag !== "SessionStarted") throw new Error(result.reason);
    if (profiled && object(profile).truncated !== false) throw new Error("incomplete phase evidence");
    observations.push({ index, warmup: index === 0, profiled, wallMs, profile });
    port.disposeSession(result.session);
  }
  const sha256 = (value: ArrayBuffer | number[]) => new CryptoHasher("sha256").update(value instanceof ArrayBuffer ? value : new Uint8Array(value)).digest("hex");
  variants[name] = { sourceSha256: sha256(source), initialCwr1Sha256: sha256(initial), editedCwr1Sha256: sha256(edited), cet1Sha256: sha256(witness), observations };
}
const native = object(await file(location("native.json")).json());
const report = { compiler: native.compiler, runtimeSha256: new CryptoHasher("sha256").update(runtime).digest("hex"),
  measurement: "Wasm checked source transfer with passive byte adapter; no renderer", samplesPerMode: samples, variants };
await write(location("wasm.json"), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report));
