import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { decodeSessionEvent, "decode-projected-term-frame" as decodeProjection } from "./wasm-cartridge-port.js";

export interface Difference {
  readonly path: readonly string[];
  readonly expected: unknown;
  readonly actual: unknown;
}

function fields(value: unknown): Map<string, unknown> | undefined {
  return typeof value === "object" && value !== null ? new Map(Object.entries(value)) : undefined;
}

/** Preserve field values, array order, missing fields, and exact numbers. */
export function firstDifference(expected: unknown, actual: unknown, path: readonly string[] = []): Difference | undefined {
  if (Object.is(expected, actual)) return;
  if (Array.isArray(expected) !== Array.isArray(actual)) return { path, expected, actual };
  const left = fields(expected), right = fields(actual);
  if (!left || !right) return { path, expected, actual };
  for (const key of new Set([...left.keys(), ...right.keys()])) {
    if (left.has(key) !== right.has(key)) return { path: [...path, key], expected: left.get(key), actual: right.get(key) };
    const difference = firstDifference(left.get(key), right.get(key), [...path, key]);
    if (difference) return difference;
  }
}

function byteDifference(expected: Uint8Array, actual: Uint8Array): number | undefined {
  for (let i = 0; i < Math.max(expected.length, actual.length); ++i) {
    if (expected[i] !== actual[i]) return i;
  }
}

export interface ReplayMismatch {
  readonly inputIndex: number | null;
  readonly command: number | null;
  readonly phase: string;
  readonly input: string;
  readonly source: string;
  readonly byteOffset: number;
  readonly difference: Difference;
  readonly entity?: unknown;
  readonly sourceState?: unknown;
}

export function compareEvents(expected: Uint8Array, actual: Uint8Array): { byteOffset: number; difference: Difference; entity?: unknown } | undefined {
  const byteOffset = byteDifference(expected, actual);
  if (byteOffset === undefined) return;
  const left = decodeSessionEvent([...expected]), right = decodeSessionEvent([...actual]);
  if (left.kind === "admission" && right.kind === "admission" && left.projection && right.projection) {
    const expectedFrame = decodeProjection(left.projection.termBytes);
    const actualFrame = decodeProjection(right.projection.termBytes);
    const difference = firstDifference(expectedFrame, actualFrame, ["projection"]);
    if (difference) {
      const entity = fields(fields(expectedFrame)?.get(difference.path[1]))?.get("$referents");
      return { byteOffset, difference, entity };
    }
  }
  return { byteOffset, difference: firstDifference(left, right, ["event"])
    ?? { path: ["event", "exactBytes", String(byteOffset)], expected: expected[byteOffset], actual: actual[byteOffset] } };
}

export async function replay(directory: string, modulePath: string): Promise<{ inputs: number; events: number; admissions: number; mismatch?: ReplayMismatch }> {
  const artifact = (name: string) => resolve(directory, name);
  const bytes = async (name: string) => new Uint8Array(await Bun.file(artifact(name)).arrayBuffer());
  const module: typeof import("#clause-runtime-wasm") = await import(pathToFileURL(resolve(modulePath)).href);
  module.initSync({ module: await Bun.file(resolve(modulePath, "../clause_runtime_bg.wasm")).arrayBuffer() });
  const source = artifact("source.clause");
  const inputs = (await Bun.file(artifact("inputs.txt")).text()).split(/\r?\n/).map(line => line.trim()).filter(line => line && !line.startsWith("#"));
  const metadata = decodeProjection([...await bytes("source-metadata.term")]);
  const sourceState = (difference: Difference) => {
    const states = fields(fields(metadata)?.get("states"));
    if (!states || difference.path[0] !== "projection") return;
    for (const page of states.values()) for (const state of fields(page)?.values() ?? []) {
      const values = fields(state);
      if (values?.get("subject") === difference.path[1] && values.get("relation") === difference.path[2]) return state;
    }
  };
  let events = 0, admissions = 0;
  const mismatch = (inputIndex: number | null, command: number | null, phase: string, result: NonNullable<ReturnType<typeof compareEvents>>): ReplayMismatch => ({
    inputIndex, command, phase, input: inputIndex === null ? "initial state" : inputs[inputIndex] ?? phase,
    source, ...result, sourceState: sourceState(result.difference),
  });
  const status = module.clause_session_v1_open_bulk(await bytes("initial.cwi1"));
  if (status !== 0) throw new Error(`initial session rejected with status ${status}; source=${source}`);
  const openedBytes = module.clause_session_v1_event_bulk();
  const opened = decodeSessionEvent([...openedBytes]);
  if (opened.kind !== "opened") throw new Error(`initial session returned ${opened.kind}`);
  const openingDifference = compareEvents(await bytes("opened.cse1"), openedBytes);
  if (openingDifference) return { inputs: inputs.length, events, admissions, mismatch: mismatch(null, null, "open", openingDifference) };
  const expectedInitial = await bytes("initial.term");
  const actualInitial = module.clause_session_v1_project_bulk(opened.slot, opened.generation);
  const offset = byteDifference(expectedInitial, actualInitial);
  if (offset !== undefined) return { inputs: inputs.length, events, admissions, mismatch: mismatch(null, null, "initial projection", {
    byteOffset: offset,
    difference: firstDifference(decodeProjection([...expectedInitial]), decodeProjection([...actualInitial]), ["projection"])
      ?? { path: ["projection", "exactBytes", String(offset)], expected: expectedInitial[offset], actual: actualInitial[offset] },
  }) };
  for (const line of (await Bun.file(artifact("commands.tsv")).text()).trim().split("\n")) {
    const [ordinalText, indexText, phase] = line.split("\t");
    const ordinal = Number(ordinalText), index = Number(indexText);
    if (!Number.isSafeInteger(ordinal) || ordinal !== events || !Number.isSafeInteger(index) || index < 0 || !phase) throw new Error("invalid replay command index");
    const status = module.clause_session_v1_command_bulk(await bytes(`${ordinal}.cwi1`));
    if (status !== 0) throw new Error(`input ${index} (${inputs[index] ?? phase}), command ${ordinal}: Wasm rejected status ${status}; source=${source}`);
    const actual = module.clause_session_v1_event_bulk();
    const result = compareEvents(await bytes(`${ordinal}.cse1`), actual);
    events += 1;
    if (result) return { inputs: inputs.length, events, admissions, mismatch: mismatch(index, ordinal, phase, result) };
    if (phase === "admit") admissions += 1;
  }
  return { inputs: inputs.length, events, admissions };
}

if (import.meta.main) {
  const [directory, modulePath] = Bun.argv.slice(2);
  if (!directory || !modulePath || Bun.argv.length !== 4) throw new Error("usage: bun clause:browser/runtime/src/game-replay.ts RECORD_DIRECTORY WASM_JS_MODULE");
  const result = await replay(directory, modulePath);
  console.log(JSON.stringify(result, null, 2));
  if (result.mismatch) process.exitCode = 1;
}
