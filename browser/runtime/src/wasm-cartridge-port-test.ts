import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import * as test from "bun:test";
import { expect } from "bun:test";
import { file } from "bun";
import {
  "clause_session_v1_command_bulk" as clause__session__v1__command__bulk,
  "clause_session_v1_event_bulk" as clause__session__v1__event__bulk,
  "clause_session_v1_open_bulk" as clause__session__v1__open__bulk,
  "clause_session_v1_reclaim_retired" as clause__session__v1__reclaim__retired,
  "initSync" as initSync,
} from "#clause-runtime-wasm";

function equivalent(left: unknown, right: unknown): boolean {
  return (
    Object.is(left, right) ||
    (Array.isArray(left) &&
      Array.isArray(right) &&
      left.length === right.length &&
      Array.prototype.every.call(left, (value, index) =>
        equivalent(value, right[index]),
      ))
  );
}

function concatenate(...values: readonly unknown[]): string {
  return values.map(String).join("");
}

type SessionModule = Readonly<{
  clause_session_v1_open_bulk: (request: Uint8Array<ArrayBuffer>) => number;
  clause_session_v1_command_bulk: (request: Uint8Array<ArrayBuffer>) => number;
  clause_session_v1_event_bulk: () => Uint8Array<ArrayBufferLike>;
  clause_session_v1_reclaim_retired: () => boolean;
}>;

type Cwo1Value =
  | Readonly<{ kind: "boolean"; value: boolean }>
  | Readonly<{ kind: "number"; value: number }>;

function policy(): workbench.WorkbenchPolicy {
  const maximum = Number.MAX_SAFE_INTEGER;
  return workbench["->WorkbenchPolicy"](
    8,
    8,
    32,
    128,
    512,
    workbench["->WorkbenchSequenceLimits"](
      maximum,
      maximum,
      maximum,
      maximum,
      maximum,
    ),
  );
}

function arena_policy(): workbench.WorkbenchPolicy {
  const maximum = Number.MAX_SAFE_INTEGER;
  return workbench["->WorkbenchPolicy"](
    8,
    8,
    32,
    wasm["cse1-projected-term-max-properties"],
    wasm["cse1-projected-term-json-max-source-units"],
    workbench["->WorkbenchSequenceLimits"](
      maximum,
      maximum,
      maximum,
      maximum,
      maximum,
    ),
  );
}

function identity(tag: number): number[] {
  const bytes = new Array(32);
  bytes.fill(0);
  bytes.splice(0, 1, tag);
  bytes.splice(31, 1, tag);
  return bytes;
}

function cwo1(values: readonly Cwo1Value[]): number[] {
  const flat = [67, 87, 79, 49, ...identity(11), ...identity(22)];
  const count = values.length;
  flat.push(count % 256);
  flat.push(Math.trunc(count / 256));
  values.forEach((value) => {
    if (value.kind === "boolean") {
      flat.push(1);
      flat.push(
        ((_truthy) => _truthy !== false && _truthy != null)(value.value)
          ? 1
          : 0,
      );
    } else {
      const packed = new ArrayBuffer(8);
      const view = new DataView(packed);
      const octets = new Uint8Array(packed);
      view.setFloat64(0, value.value, true);
      flat.push(0);
      octets.forEach((octet) => {
        flat.push(octet);
      });
    }
  });
  return flat;
}

function module_for_bang(
  events: number[][],
  requests: number[][],
): SessionModule {
  let current: number[] = [];
  const next_event_bang = (request: Uint8Array<ArrayBuffer>): number => {
    requests.push(Array.from(request));
    const next = events.shift();
    if (next === undefined) throw new Error("test session event is missing");
    current = next;
    return 0;
  };
  return {
    clause_session_v1_open_bulk: next_event_bang,
    clause_session_v1_command_bulk: next_event_bang,
    clause_session_v1_event_bulk: () => new Uint8Array(current),
    clause_session_v1_reclaim_retired: () => true,
  };
}

function append_u32_bang(bytes: number[], value: number): void {
  [1, 256, 65536, 16777216].forEach((divisor) => {
    bytes.push(Math.trunc(value / divisor) % 256);
  });
}

function append_big_u32_bang(bytes: number[], value: number): void {
  [16777216, 65536, 256, 1].forEach((divisor) => {
    bytes.push(Math.trunc(value / divisor) % 256);
  });
}

function projected_atom_node(
  kind: string,
  payload: readonly number[],
): number[] {
  const bytes: number[] = [];
  bytes.push(0);
  append_big_u32_bang(bytes, kind.length);
  Array.from(kind, (character) => character.charCodeAt(0)).forEach((byte) => {
    bytes.push(byte);
  });
  append_big_u32_bang(bytes, payload.length);
  payload.forEach((byte) => {
    bytes.push(byte);
  });
  bytes.push(0);
  return bytes;
}

function projected_atom(kind: string, payload: readonly number[]): number[] {
  return new Array<number>(64)
    .fill(0)
    .concat(projected_atom_node(kind, payload));
}

function projected_triple_node(
  left: readonly number[],
  operator: readonly number[],
  right: readonly number[],
): number[] {
  return [1, ...left, ...operator, ...right];
}

function append_u64_bang(bytes: number[], value: number): void {
  append_u32_bang(bytes, value);
  append_u32_bang(bytes, 0);
}

function append_blob_bang(bytes: number[], value: readonly number[]): void {
  append_u32_bang(bytes, value.length);
  value.forEach((byte) => {
    bytes.push(byte);
  });
}

function allocation_epoch_bang(): number[] {
  const bytes = new Array(304);
  bytes.fill(0);
  bytes.splice(0, 4, 82, 65, 69, 49);
  return bytes;
}

function minimal_cwr1_bang(
  physical_plan: readonly number[] = [8],
  occurrences: readonly (readonly number[])[] = [[1], [2]],
): number[] {
  const bytes = [67, 87, 82, 49];
  append_blob_bang(bytes, [1]);
  append_u32_bang(bytes, 1);
  append_blob_bang(bytes, physical_plan);
  append_blob_bang(bytes, allocation_epoch_bang());
  [1, 2, 3, 4, 5, 6, 7, 8, 9].forEach((tag) => {
    identity(tag).forEach((byte) => {
      bytes.push(byte);
    });
  });
  append_blob_bang(bytes, [9]);
  identity(10).forEach((byte) => {
    bytes.push(byte);
  });
  append_blob_bang(bytes, [10]);
  identity(11).forEach((byte) => {
    bytes.push(byte);
  });
  append_blob_bang(bytes, [11]);
  append_u64_bang(bytes, 100);
  bytes.push(occurrences.length % 256, Math.trunc(occurrences.length / 256));
  occurrences.forEach((occurrence) => append_blob_bang(bytes, occurrence));
  bytes.push(0, 0);
  return bytes;
}

function cse_header_bang(sequence: number, tag: number): number[] {
  const bytes = [67, 83, 69, 49];
  append_u32_bang(bytes, 0);
  append_u32_bang(bytes, 1);
  append_u64_bang(bytes, sequence);
  bytes.push(tag);
  return bytes;
}

test.test("cartridge byte custody survives mutation of the producer's array", () => {
  const source = minimal_cwr1_bang();
  const original = source.slice();
  const actual: number[][] = [];
  const expected: number[][] = [];
  const port = wasm["create-wasm-cartridge-port"](module_for_bang([opened_event_bang()], actual), policy());
  const reference = wasm["create-wasm-cartridge-port"](module_for_bang([opened_event_bang()], expected), policy());
  const accepted = acceptPackage(port, wasm["->ExactProcessRequest"](source));
  source.fill(255);
  startSession(port, accepted.acceptedPackage);
  startSession(reference, acceptPackage(reference, wasm["->ExactProcessRequest"](original)).acceptedPackage);
  expect(actual).toEqual(expected);
  expect(actual).toHaveLength(1);
});

test.test("cartridge byte custody rejects malformed octets and skipped-blob bounds", () => {
  const port = wasm["create-wasm-cartridge-port"](module_for_bang([], []), policy());
  const invalid: number[][] = [];
  for (const value of [NaN, 256, -1, 0.5]) {
    const source = minimal_cwr1_bang(); source[9] = value; invalid.push(source);
  }
  const sparse = minimal_cwr1_bang(); delete sparse[9]; invalid.push(sparse);
  const oversized = minimal_cwr1_bang(); oversized.splice(4, 4, 255, 255, 255, 255); invalid.push(oversized);
  invalid.push(minimal_cwr1_bang().slice(0, 8));
  for (const source of invalid) {
    port.acceptPackage(wasm["->ExactProcessRequest"](source), result => expect(result._tag).toBe("PackageRejected"));
  }
});

test.test("byte validation rechecks caller-owned frozen accessors and each length bound", () => {
  let value = 7;
  const bytes = [0];
  Object.defineProperty(bytes, 0, { get: () => value });
  Object.freeze(bytes);
  expect(wasm["exact-byte-array?"](bytes, 1)).toBe(true);
  value = 256;
  expect(wasm["exact-byte-array?"](bytes, 1)).toBe(false);
  value = 7;
  expect(wasm["exact-byte-array?"](bytes, 0)).toBe(false);
});

function put_identities_bang(bytes: number[], tags: readonly number[]): void {
  tags.forEach((tag) => {
    identity(tag).forEach((byte) => {
      bytes.push(byte);
    });
  });
}

function opened_event_bang(): number[] {
  const bytes = cse_header_bang(0, 1);
  put_identities_bang(bytes, [21, 3, 22, 23, 24]);
  append_u32_bang(bytes, 1);
  append_blob_bang(bytes, allocation_epoch_bang());
  return bytes;
}

function input_event_bang(): number[] {
  const bytes = cse_header_bang(1, 2);
  put_identities_bang(bytes, [31, 23, 24, 32, 33]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function candidate_event_bang(): number[] {
  const bytes = cse_header_bang(2, 3);
  put_identities_bang(bytes, [34, 35, 22, 23, 24]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function issuance_event_bang(): number[] {
  const bytes = cse_header_bang(3, 4);
  put_identities_bang(bytes, [40, 21, 3, 22, 35]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function admission_event_bang(frame: readonly number[] = [40, 41, 42]): number[] {
  const bytes = cse_header_bang(4, 5);
  put_identities_bang(bytes, [22, 36, 37, 38, 41, 42, 3]);
  append_u32_bang(bytes, 2);
  bytes.push(1);
  put_identities_bang(bytes, [39]);
  append_blob_bang(bytes, frame);
  return bytes;
}

function suspended_event_bang(): number[] {
  const bytes = cse_header_bang(2, 8);
  put_identities_bang(bytes, [34, 35, 23, 24, 36, 37]);
  append_u64_bang(bytes, 98);
  append_u32_bang(bytes, 1);
  return bytes;
}

function resumed_event_bang(): number[] {
  const bytes = cse_header_bang(3, 9);
  put_identities_bang(bytes, [38, 39, 35, 23, 24, 37, 40]);
  append_u64_bang(bytes, 97);
  append_u32_bang(bytes, 1);
  return bytes;
}

function resumed_candidate_event_bang(): number[] {
  const bytes = cse_header_bang(4, 3);
  put_identities_bang(bytes, [41, 42, 22, 23, 24]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function resumed_issuance_event_bang(): number[] {
  const bytes = cse_header_bang(5, 4);
  put_identities_bang(bytes, [43, 21, 3, 22, 42]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function resumed_admission_event_bang(): number[] {
  const bytes = cse_header_bang(6, 5);
  put_identities_bang(bytes, [22, 44, 45, 46, 47, 48, 3]);
  append_u32_bang(bytes, 2);
  bytes.push(0);
  return bytes;
}

function disposed_event_bang(): number[] {
  return cse_header_bang(5, 6);
}

function resumed_disposed_event_bang(): number[] {
  return cse_header_bang(7, 6);
}

function throws_p_bang(action: () => unknown): boolean {
  try {
    action();
    return false;
  } catch (error) {
    if (error instanceof Error) return true;
    throw error;
  }
}

function json_string(value: unknown): string {
  const encoded = JSON.stringify(value);
  return equivalent(typeof encoded, "string")
    ? encoded
    : (() => {
        throw new Error("test value is not JSON-encodable");
      })();
}

function initialize_real_session_module(module: ArrayBuffer): SessionModule {
  const input = module;
  const __initialized = initSync(input);
  return Object.freeze({
    clause_session_v1_open_bulk: (request: Uint8Array<ArrayBuffer>) =>
      clause__session__v1__open__bulk(new Uint8Array(request)),
    clause_session_v1_command_bulk: (request: Uint8Array<ArrayBuffer>) =>
      clause__session__v1__command__bulk(new Uint8Array(request)),
    clause_session_v1_event_bulk: () => clause__session__v1__event__bulk(),
    clause_session_v1_reclaim_retired: () =>
      clause__session__v1__reclaim__retired(),
  });
}

function key_configuration(
  input_sequence: number,
  revision: number,
  code: string,
): workbench.InputConfiguration {
  return workbench["->InputConfiguration"](revision, [
    workbench["->InputObservation"](
      input_sequence,
      workbench["create-workbench-envelope"](
        policy(),
        json_string([
          json_string({
            kind: "keyboard",
            code: code,
            phase: "down",
            repeat: false,
          }),
        ]),
      ),
    ),
  ]);
}

function two_key_configuration(
  first_input_sequence: number,
  revision: number,
  first_code: string,
  second_code: string,
): workbench.InputConfiguration {
  return workbench["->InputConfiguration"](revision, [
    workbench["->InputObservation"](
      first_input_sequence,
      workbench["create-workbench-envelope"](
        policy(),
        json_string([
          json_string({
            kind: "keyboard",
            code: first_code,
            phase: "down",
            repeat: false,
          }),
        ]),
      ),
    ),
    workbench["->InputObservation"](
      first_input_sequence + 1,
      workbench["create-workbench-envelope"](
        policy(),
        json_string([
          json_string({
            kind: "keyboard",
            code: second_code,
            phase: "down",
            repeat: false,
          }),
        ]),
      ),
    ),
  ]);
}

function process_configuration(
  input_sequence: number,
  revision: number,
  ordinal: number,
): workbench.InputConfiguration {
  return workbench["->InputConfiguration"](revision, [
    workbench["->InputObservation"](
      input_sequence,
      workbench["create-workbench-envelope"](
        policy(),
        json_string([
          json_string({ kind: "process-occurrence", ordinal: ordinal }),
        ]),
      ),
    ),
  ]);
}

function key_envelope(code: string, phase: string): workbench.WorkbenchEnvelope {
  return workbench["create-workbench-envelope"](
    arena_policy(),
    json_string([
      json_string({
        kind: "keyboard",
        code: code,
        phase: phase,
        repeat: false,
      }),
    ]),
  );
}

function completeSynchronously<T>(
  register: (complete: (result: T) => unknown) => unknown,
): T {
  const completions: T[] = [];
  register((result) => completions.push(result));
  if (completions.length !== 1) {
    throw new Error("test port completion was not synchronous and singular");
  }
  return completions[0];
}

function acceptPackage(
  port: workbench.CartridgePort,
  candidate: unknown,
): workbench.PackageAccepted {
  const completion = completeSynchronously<workbench.PackageCheck>((complete) =>
    port.acceptPackage(candidate, complete),
  );
  if (completion._tag !== "PackageAccepted") {
    throw new Error(completion.reason);
  }
  return completion;
}

function startSession(
  port: workbench.CartridgePort,
  acceptedPackage: unknown,
  generation = 1,
): workbench.SessionStarted {
  const completion = completeSynchronously<workbench.SessionCompletion>(
    (complete) => port.startSession(acceptedPackage, generation, complete),
  );
  if (completion._tag !== "SessionStarted") {
    throw new Error(completion.reason);
  }
  return completion;
}

function runCandidateCompletion(
  port: workbench.CartridgePort,
  session: unknown,
  configuration: workbench.InputConfiguration,
): workbench.CandidateCompletion {
  return completeSynchronously<workbench.CandidateCompletion>((complete) =>
    port.runCandidate(
      session,
      workbench["->FixedTick"](16),
      configuration,
      complete,
    ),
  );
}

function runCandidate(
  port: workbench.CartridgePort,
  session: unknown,
  configuration: workbench.InputConfiguration,
): workbench.CandidateProduced {
  const completion = runCandidateCompletion(port, session, configuration);
  if (completion._tag !== "CandidateProduced") {
    throw new Error(completion.reason);
  }
  return completion;
}

function admitCandidate(
  port: workbench.CartridgePort,
  session: unknown,
  candidate: unknown,
): workbench.AdmissionAccepted {
  const completion = completeSynchronously<workbench.AdmissionCompletion>(
    (complete) => port.requestAdmission(session, candidate, complete),
  );
  if (completion._tag !== "AdmissionAccepted") {
    throw new Error(completion.reason);
  }
  return completion;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function recordValue(value: unknown, label: string): Record<string, unknown> {
  if (!isRecord(value)) {
    throw new Error(`${label} is not an object`);
  }
  return value;
}

function candidateBase(candidate: unknown): wasm.ExactBytes {
  const base = recordValue(candidate, "candidate")["base"];
  if (!wasm["exact-byte-array?"](base, 32) || base.length !== 32) {
    throw new Error("candidate base is not an exact Clause identity");
  }
  return base;
}

function projectedField(
  value: wasm.ProjectedValue,
  ...path: readonly string[]
): wasm.ProjectedValue {
  let current = value;
  for (const key of path) {
    if (!isRecord(current)) {
      throw new Error(`projected ${path.join(".")} is not an object path`);
    }
    const next = current[key];
    if (next === undefined) {
      throw new Error(`projected ${path.join(".")} is missing`);
    }
    current = next;
  }
  return current;
}

function projectedNumber(
  value: wasm.ProjectedValue,
  ...path: readonly string[]
): number {
  const result = projectedField(value, ...path);
  if (typeof result !== "number") {
    throw new Error(`projected ${path.join(".")} is not a number`);
  }
  return result;
}

function projectedString(
  value: wasm.ProjectedValue,
  ...path: readonly string[]
): string {
  const result = projectedField(value, ...path);
  if (typeof result !== "string") {
    throw new Error(`projected ${path.join(".")} is not text`);
  }
  return result;
}

test["test"]("projected relation contracts retain required participation", () => {
  const decode = (contract: number, values: readonly number[] = [2]) => {
    const payload = [6];
    append_u32_bang(payload, 1);
    payload.push(0, 0, contract, 1, 0);
    append_u32_bang(payload, 1);
    payload.push(0);
    append_u32_bang(payload, 2);
    payload.push(values.length, 0);
    for (const value of values) {
      const bytes = new ArrayBuffer(8);
      new DataView(bytes).setFloat64(0, value, true);
      payload.push(0, ...new Uint8Array(bytes));
    }
    return wasm["decode-projected-term-frame"](
      projected_atom("clause/process-projected-relation-table-v1", payload),
    );
  };
  for (const contract of [0, 1, 2, 3]) {
    const table = decode(contract);
    expect(table).toEqual({
      kind: "relation-table", subjectDomain: 1, valueKind: 0,
      cardinality: contract === 3 ? 0 : contract, total: contract === 3,
      rows: [{ subject: { kind: "referent", domain: 1,
        identity: { kind: "declared", value: 2 } }, values: [2] }],
    });
    expect(Object.isFrozen(table)).toBe(true);
  }
  expect(() => decode(4)).toThrow("invalid projected relation cardinality");
  expect(() => decode(3, [2, 3])).toThrow("invalid projected row cardinality");
  expect(() => decode(3, [])).toThrow("invalid projected row cardinality");
  for (const value of [-0, NaN, Infinity, -Infinity]) {
    expect(() => decode(0, [value])).toThrow("CWO1 number is not canonical finite f64");
  }
});

test["test"]("projected Text realizes exact UTF-8", () => {
  const text = wasm["decode-projected-term-frame"](
    projected_atom("clause/process-projected-text-v1", [
      78, 111, 114, 116, 104, 32, 240, 159, 154, 128,
    ]),
  );
  test["expect"](text).toBe("North 🚀");
  test["expect"](() =>
    wasm["decode-projected-term-frame"](
      projected_atom("clause/process-projected-text-v1", [255]),
    ),
  ).toThrow("projected Text is not canonical UTF-8");
});

test["test"](
  "projected Term decoding admits transport-bounded depth beyond 64",
  () => {
    const nesting = 96;
    let term = projected_atom_node("clause/process-projected-text-v1", [111, 107]);
    for (let depth = 0; depth < nesting; depth += 1) {
      term = projected_triple_node(
        projected_atom_node("clause/js-item-v1", []),
        term,
        projected_atom_node("clause/js-array-end-v1", []),
      );
    }
    let projected: unknown = wasm["decode-projected-term-frame"](
      new Array<number>(64).fill(0).concat(term),
    );
    for (let depth = 0; depth < nesting; depth += 1) {
      test["expect"](Array.isArray(projected)).toBe(true);
      test["expect"]((projected as readonly unknown[]).length).toBe(1);
      projected = (projected as readonly unknown[])[0];
    }
    test["expect"](projected).toBe("ok");
  },
);

test["test"]("projected Term decoding retains the CSE1 byte bound", () => {
  test["expect"](() =>
    wasm["decode-projected-term-frame"](
      new Array<number>(1024 * 1024 + 1).fill(0),
    ),
  ).toThrow("projected Term bytes are outside the CSE1 bound");
});

function projectedBoolean(
  value: wasm.ProjectedValue,
  ...path: readonly string[]
): boolean {
  const result = projectedField(value, ...path);
  if (typeof result !== "boolean") {
    throw new Error(`projected ${path.join(".")} is not Boolean`);
  }
  return result;
}

function projectedItem(
  value: wasm.ProjectedValue,
  index: number,
  ...path: readonly string[]
): wasm.ProjectedValue {
  const result = projectedField(value, ...path);
  if (!Array.isArray(result)) {
    throw new Error(`projected ${path.join(".")} is not an array`);
  }
  const item: unknown = result[index];
  if (item === undefined) {
    throw new Error(`projected ${path.join(".")}[${index}] is missing`);
  }
  return item as wasm.ProjectedValue;
}

function admit_real_process_bang(
  port: workbench.CartridgePort,
  request_bytes: string,
): wasm.ProjectedValue {
  const request = wasm["->ExactProcessRequest"](
    wasm["decode-cwr1-hex"](request_bytes),
  );
  const accepted = acceptPackage(port, request);
  const started = startSession(port, accepted.acceptedPackage);
  const candidate = runCandidate(
    port,
    started.session,
    process_configuration(1, 1, 0),
  );
  expect(json_string(started.frame)).toBe("[]");
  expect(json_string(candidateBase(candidate.candidate))).toBe(
    json_string(started.revision),
  );
  const admitted = admitCandidate(port, started.session, candidate.candidate);
  expect(json_string(admitted.revision)).not.toBe(json_string(started.revision));
  const frame = wasm["decode-projected-term-frame"](admitted.frame);
  port.disposeSession(started.session);
  return frame;
}

test["test"](
  "one persistent session sequences physical input candidate issuance Admission and disposal",
  () => {
    const requests: number[][] = [];
    const port = wasm["create-wasm-cartridge-port"](
      module_for_bang(
        [
          opened_event_bang(),
          input_event_bang(),
          candidate_event_bang(),
          issuance_event_bang(),
          admission_event_bang(),
          disposed_event_bang(),
        ],
        requests,
      ),
      policy(),
    );
    const request = wasm["->ExactProcessRequest"](minimal_cwr1_bang());
    const accepted = acceptPackage(port, request);
    const started = startSession(port, accepted.acceptedPackage);
    const candidate = runCandidate(
      port,
      started.session,
      key_configuration(1, 1, "KeyD"),
    );
    test["expect"](concatenate(requests.length)).toBe("3");
    const open_request = requests[0];
    const open_length = open_request.length;
    const allocation_tag = open_length - 18;
    test["expect"](json_string(open_request.slice(0, 4))).toBe("[67,87,83,49]");
    test["expect"](json_string(open_request.slice(13, 18))).toBe("[1,0,0,0,8]");
    test["expect"](concatenate(open_request[allocation_tag])).toBe("0");
    test["expect"](
      json_string(open_request.slice(allocation_tag + 1, allocation_tag + 9)),
    ).toBe("[255,255,255,255,255,255,31,0]");
    test["expect"](
      json_string(open_request.slice(allocation_tag + 9)),
    ).toBe("[0,0,16,0,0,0,16,0,1]");
    test["expect"](json_string([requests[1][20], requests[2][20]])).toBe(
      "[3,4]",
    );
    test["expect"](json_string(started.frame)).toBe("[]");
    const admitted = admitCandidate(
      port,
      started.session,
      candidate.candidate,
    );
    test["expect"](concatenate(requests.length)).toBe("5");
    test["expect"](json_string([requests[3][20], requests[4][20]])).toBe(
      "[5,6]",
    );
    test["expect"](json_string(admitted.revision)).toBe(
      json_string(identity(36)),
    );
    test["expect"](json_string(admitted.frame)).toBe("[40,41,42]");
    port.disposeSession(started.session);
    test["expect"](concatenate(requests[5][20])).toBe("7");
    const after_dispose = runCandidateCompletion(
      port,
      started.session,
      workbench["->InputConfiguration"](0, []),
    );
    if (after_dispose._tag !== "CandidateFailed") {
      throw new Error("disposed session unexpectedly produced a candidate");
    }
    test["expect"](concatenate(after_dispose.reason).split("\n")[0]).toBe(
      "Error: Wasm session is disposed",
    );
  },
);

test.test("admitted bulk snapshot preserves every octet after producer mutation", () => {
  const bytes = Array.from({ length: 256 }, (_, index) => index);
  const module = module_for_bang([opened_event_bang(), input_event_bang(), candidate_event_bang(),
    issuance_event_bang(), admission_event_bang(bytes)], []);
  let producer = new Uint8Array(0);
  const port = wasm["create-wasm-cartridge-port"]({ ...module,
    clause_session_v1_event_bulk: () => (producer = new Uint8Array(module.clause_session_v1_event_bulk())),
  }, arena_policy());
  const accepted = acceptPackage(port, wasm["->ExactProcessRequest"](minimal_cwr1_bang()));
  const started = startSession(port, accepted.acceptedPackage);
  const candidate = runCandidate(port, started.session, key_configuration(1, 1, "KeyD"));
  const admitted = admitCandidate(port, started.session, candidate.candidate);
  producer.fill(0);
  expect(JSON.parse(JSON.stringify(admitted.frame))).toEqual(bytes);
});

test["test"](
  "persistent session open and command requests retain distinct byte envelopes",
  () => {
    const one_mib = 1024 * 1024;
    const requests: number[][] = [];
    const module = module_for_bang([opened_event_bang()], requests);
    const port = wasm["create-wasm-cartridge-port"](module, policy());
    const request = wasm["->ExactProcessRequest"](
      minimal_cwr1_bang(
        new Array<number>(one_mib).fill(8),
        [new Array<number>(one_mib).fill(1)],
      ),
    );
    const accepted = acceptPackage(port, request);
    const started = startSession(port, accepted.acceptedPackage);

    expect(requests).toHaveLength(1);
    expect(requests[0].length).toBeGreaterThan(one_mib);
    expect(requests[0].length).toBeLessThanOrEqual(4 * one_mib);
    expect(() =>
      wasm["advance-session-occurrence!"](module, started.session, 0),
    ).toThrow("persistent session request must carry bounded exact bytes");
    expect(requests).toHaveLength(1);
  },
);

test["test"](
  "persistent CWI1 commands retain continuation custody and exact Admission identities",
  () => {
    const requests: number[][] = [];
    const module = module_for_bang(
      [
        opened_event_bang(),
        input_event_bang(),
        suspended_event_bang(),
        resumed_event_bang(),
        resumed_candidate_event_bang(),
        resumed_issuance_event_bang(),
        resumed_admission_event_bang(),
        resumed_disposed_event_bang(),
      ],
      requests,
    );
    const port = wasm["create-wasm-cartridge-port"](module, policy());
    const request = wasm["->ExactProcessRequest"](minimal_cwr1_bang());
    const accepted = acceptPackage(port, request);
    const started = startSession(port, accepted.acceptedPackage);
    const session = started.session;
    const input = wasm["advance-session-occurrence!"](module, session, 0);
    const suspension = wasm["suspend-session!"](module, session);
    const resumption = wasm["resume-session!"](module, session);
    test["expect"](concatenate(input.kind)).toBe("input");
    test["expect"](json_string(suspension.continuation)).toBe(
      json_string(resumption.continuation),
    );
    test["expect"](concatenate(suspension.remainingBudget)).toBe("98");
    test["expect"](concatenate(resumption.remainingBudget)).toBe("97");
    const candidate = runCandidate(
      port,
      session,
      process_configuration(1, 1, 0),
    );
    const admission = wasm["admit-session-candidate!"](
      module,
      session,
      candidate.candidate,
    );
    test["expect"](json_string(admission.predecessor)).toBe(
      json_string(identity(22)),
    );
    test["expect"](json_string(admission.admissionId)).toBe(
      json_string(identity(45)),
    );
    test["expect"](json_string(admission.judgmentId)).toBe(
      json_string(identity(46)),
    );
    test["expect"](json_string(admission.successor)).toBe(
      json_string(identity(44)),
    );
    port.disposeSession(session);
    test["expect"](
      json_string(requests.slice(1).map((bytes) => bytes[20])),
    ).toBe("[1,8,9,2,5,6,7]");
  },
);

test["test"](
  "strict CWO1 decoding rejects malformed and trailing bytes",
  () => {
    const valid = cwo1([{ kind: "boolean", value: false }]);
    test["expect"](
      throws_p_bang(() => wasm["decode-cwo1-observation"](valid.concat([0])))
        ? "true"
        : "false",
    ).toBe("true");
    const bad = valid.slice();
    bad.splice(0, 1, 88);
    test["expect"](
      throws_p_bang(() => wasm["decode-cwo1-observation"](bad))
        ? "true"
        : "false",
    ).toBe("true");
  },
);

test["test"]("CWR1 hex transport is exact and bounded", () => {
  test["expect"](json_string(wasm["decode-cwr1-hex"]("43 57\n52\t31"))).toBe(
    "[67,87,82,49]",
  );
  (() => {
    ["", "0", "0g", "0A"].forEach((source) => {
      test["expect"](
        throws_p_bang(() => wasm["decode-cwr1-hex"](source)) ? "true" : "false",
      ).toBe("true");
    });
  })();
});

test["test"]("CET1 hex transport is exact", () => {
  test["expect"](json_string(wasm["decode-cet1-hex"]("43 45\n54\t31"))).toBe(
    "[67,69,84,49]",
  );
});

test["test"](
  "real Wasm lowers physical input and exposes only the admitted arena frame",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file("../../test-vectors/browser/wasm-jump-v1/jump-v1.cwr1.hex").text(),
    ]).then((assets) => {
      const port = wasm["create-wasm-cartridge-port"](
        initialize_real_session_module(assets[0]),
        arena_policy(),
      );
      const request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[1]),
      );
      const accepted = acceptPackage(port, request);
      const started = startSession(port, accepted.acceptedPackage);
      const candidate = runCandidate(
        port,
        started.session,
        key_configuration(1, 1, "KeyD"),
      );
      test["expect"](json_string(started.frame)).toBe("[]");
      const admitted = admitCandidate(
        port,
        started.session,
        candidate.candidate,
      );
      const frame = wasm["decode-projected-term-frame"](admitted.frame);
      test["expect"](
        projectedNumber(frame, "player", "position", "x") > 0.0,
      ).toBe(true);
      const airborne_candidate = runCandidate(
        port,
        started.session,
        key_configuration(2, 2, "Space"),
      );
      const airborne_admitted = admitCandidate(
        port,
        started.session,
        airborne_candidate.candidate,
      );
      const jump_frame = wasm["decode-projected-term-frame"](
        airborne_admitted.frame,
      );
      test["expect"](
        projectedNumber(jump_frame, "player", "position", "x") > 0.0 &&
          projectedNumber(jump_frame, "player", "position", "y") > 0.0 &&
          !projectedBoolean(jump_frame, "player", "grounded"),
      ).toBe(true);
      const momentum_candidate = runCandidate(
        port,
        started.session,
        workbench["->InputConfiguration"](3, []),
      );
      const momentum_admitted = admitCandidate(
        port,
        started.session,
        momentum_candidate.candidate,
      );
      const airborne_frame = wasm["decode-projected-term-frame"](
        airborne_admitted.frame,
      );
      const momentum_frame = wasm["decode-projected-term-frame"](
        momentum_admitted.frame,
      );
      test["expect"](
        projectedNumber(momentum_frame, "player", "position", "x") >
          projectedNumber(airborne_frame, "player", "position", "x"),
      ).toBe(true);
      port.disposeSession(started.session);
      return null;
    }),
);

test["test"](
  "real Wasm keeps collect hidden until Admission and projects Clause-owned score",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file("../../test-vectors/browser/wasm-collect-v1/collect-plus-1.cwr1.hex").text(),
      file("../../test-vectors/browser/wasm-collect-v1/collect-plus-4.cwr1.hex").text(),
    ]).then((assets) => {
      const port = wasm["create-wasm-cartridge-port"](
        initialize_real_session_module(assets[0]),
        arena_policy(),
      );
      const base_frame = admit_real_process_bang(port, assets[1]);
      const changed_frame = admit_real_process_bang(port, assets[2]);
      const base_score = projectedNumber(base_frame, "player", "score");
      const changed_score = projectedNumber(changed_frame, "player", "score");
      test["expect"](concatenate(base_score)).toBe("1");
      test["expect"](concatenate(changed_score)).toBe("4");
      return null;
    }),
);

test["test"](
  "real Wasm suspends resumes proposes and admits one domain-neutral process",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file(
        "../../test-vectors/browser/wasm-process-continuation-v1/process-continuation-v1.cwr1.hex",
      ).text(),
    ]).then((assets) => {
      const module = initialize_real_session_module(assets[0]);
      const port = wasm["create-wasm-cartridge-port"](module, policy());
      const request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[1]),
      );
      const accepted = acceptPackage(port, request);
      const started = startSession(port, accepted.acceptedPackage);
      const session = started.session;
      const input = wasm["advance-session-occurrence!"](module, session, 0);
      const suspension = wasm["suspend-session!"](module, session);
      const resumption = wasm["resume-session!"](module, session);
      const suspension_budget = suspension.remainingBudget;
      const resumption_budget = resumption.remainingBudget;
      test["expect"](concatenate(input.kind)).toBe("input");
      test["expect"](json_string(suspension.continuation)).toBe(
        json_string(resumption.continuation),
      );
      test["expect"](json_string(suspension.run)).toBe(
        json_string(resumption.run),
      );
      test["expect"](json_string(suspension.activation)).toBe(
        json_string(resumption.activation),
      );
      test["expect"](concatenate(resumption_budget)).toBe(
        concatenate(suspension_budget - 1),
      );
      const candidate = runCandidate(
        port,
        session,
        process_configuration(1, 1, 1),
      );
      const proposed = candidate.candidate;
      const admission = wasm["admit-session-candidate!"](
        module,
        session,
        proposed,
      );
      test["expect"](json_string(candidateBase(proposed))).toBe(
        json_string(admission.predecessor),
      );
      test["expect"](concatenate(admission.admissionId.length)).toBe("32");
      test["expect"](concatenate(admission.judgmentId.length)).toBe("32");
      test["expect"](
        !(
          json_string(admission.successor) ===
          json_string(admission.predecessor)
        )
          ? "true"
          : "false",
      ).toBe("true");
      port.disposeSession(session);
      return null;
    }),
);

test["test"](
  "real Wasm admits Clause-owned active to collected symbolic state",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file("../../test-vectors/browser/wasm-collect-state-v1/collected.cwr1.hex").text(),
      file("../../test-vectors/browser/wasm-collect-state-v1/spent.cwr1.hex").text(),
    ]).then((assets) => {
      const port = wasm["create-wasm-cartridge-port"](
        initialize_real_session_module(assets[0]),
        arena_policy(),
      );
      const collected_frame = admit_real_process_bang(port, assets[1]);
      const spent_frame = admit_real_process_bang(port, assets[2]);
      const collected_state = projectedString(
        collected_frame,
        "collectible",
        "state",
      );
      const spent_state = projectedString(spent_frame, "collectible", "state");
      test["expect"](collected_state).toBe("collected");
      test["expect"](spent_state).toBe("spent");
      return null;
    }),
);

test["test"](
  "one real Wasm gameplay session collects then automatically launches from Clause contact",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file("../../test-vectors/browser/wasm-gameplay-v1/gameplay-v1.cwr1.hex").text(),
    ]).then((assets) => {
      const port = wasm["create-wasm-cartridge-port"](
        initialize_real_session_module(assets[0]),
        arena_policy(),
      );
      const request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[1]),
      );
      const accepted = acceptPackage(port, request);
      const started = startSession(port, accepted.acceptedPackage);
      const collect_candidate = runCandidate(
        port,
        started.session,
        key_configuration(1, 1, "KeyD"),
      );
      test["expect"](json_string(started.frame)).toBe("[]");
      const collect_admitted = admitCandidate(
        port,
        started.session,
        collect_candidate.candidate,
      );
      const collected_frame = wasm["decode-projected-term-frame"](
        collect_admitted.frame,
      );
      const player_x = projectedNumber(
        collected_frame,
        "player",
        "position",
        "x",
      );
      const collected_collectible = projectedItem(
        collected_frame,
        0,
        "world",
        "collectibles",
      );
      const collected_collectible_state = projectedString(
        collected_collectible,
        "state",
      );
      const collectible_x = projectedNumber(
        collected_collectible,
        "position",
        "x",
      );
      test["expect"](collected_collectible_state).toBe("collected");
      test["expect"](concatenate(player_x)).toBe(concatenate(collectible_x));
      const launch_candidate = runCandidate(
        port,
        started.session,
        workbench["->InputConfiguration"](2, []),
      );
      const launch_admitted = admitCandidate(
        port,
        started.session,
        launch_candidate.candidate,
      );
      const launch_frame = wasm["decode-projected-term-frame"](
        launch_admitted.frame,
      );
      test["expect"](
        concatenate(projectedNumber(launch_frame, "player", "velocity", "y")),
      ).toBe("12");
      test["expect"](
        concatenate(projectedBoolean(launch_frame, "player", "grounded")),
      ).toBe("false");
      const airborne_candidate = runCandidate(
        port,
        started.session,
        workbench["->InputConfiguration"](3, []),
      );
      const airborne_admitted = admitCandidate(
        port,
        started.session,
        airborne_candidate.candidate,
      );
      const airborne_frame = wasm["decode-projected-term-frame"](
        airborne_admitted.frame,
      );
      test["expect"](
        projectedNumber(airborne_frame, "player", "position", "y") > 0.0,
      ).toBe(true);
      port.disposeSession(started.session);
      return null;
    }),
);

test["test"](
  "real Wasm executes a generically allocated Clause source plan",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file(
        "../../test-vectors/browser/wasm-generic-source-v1/generic-source-v1.cwr1.hex",
      ).text(),
    ]).then((assets) => {
      const port = wasm["create-wasm-cartridge-port"](
        initialize_real_session_module(assets[0]),
        arena_policy(),
      );
      const request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[1]),
      );
      const accepted = acceptPackage(port, request);
      const started = startSession(port, accepted.acceptedPackage);
      const candidate = runCandidate(
        port,
        started.session,
        key_configuration(1, 1, "KeyD"),
      );
      test["expect"](json_string(started.frame)).toBe("[]");
      const admitted = admitCandidate(
        port,
        started.session,
        candidate.candidate,
      );
      const frame = wasm["decode-projected-term-frame"](admitted.frame);
      test["expect"](
        projectedNumber(frame, "player", "position", "x") > 0.0,
      ).toBe(true);
      port.disposeSession(started.session);
      return null;
    }),
);

test["test"](
  "one real Wasm session fails resets completes and launches from Clause source",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file("../../test-vectors/browser/wasm-coherent-game-v1/coherent-game-v1.cwr1.hex").text(),
    ]).then((assets) => {
      const port = wasm["create-wasm-cartridge-port"](
        initialize_real_session_module(assets[0]),
        arena_policy(),
      );
      const request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[1]),
      );
      const accepted = acceptPackage(port, request);
      const started = startSession(port, accepted.acceptedPackage);
      const failure_candidate = runCandidate(
        port,
        started.session,
        key_configuration(1, 1, "KeyS"),
      );
      test["expect"](json_string(started.frame)).toBe("[]");
      const failure_admitted = admitCandidate(
        port,
        started.session,
        failure_candidate.candidate,
      );
      const failure_frame = wasm["decode-projected-term-frame"](
        failure_admitted.frame,
      );
      test["expect"](
        projectedString(failure_frame, "world", "objective", "state"),
      ).toBe("failed");
      const reset_candidate = runCandidate(
        port,
        started.session,
        two_key_configuration(2, 2, "KeyW", "KeyR"),
      );
      const reset_admitted = admitCandidate(
        port,
        started.session,
        reset_candidate.candidate,
      );
      const reset_frame = wasm["decode-projected-term-frame"](
        reset_admitted.frame,
      );
      test["expect"](
        projectedString(reset_frame, "world", "objective", "state"),
      ).toBe("playing");
      test["expect"](
        concatenate(projectedNumber(reset_frame, "player", "position", "z")),
      ).toBe("0");
      const completion_candidate = runCandidate(
        port,
        started.session,
        key_configuration(4, 3, "KeyD"),
      );
      const completion_admitted = admitCandidate(
        port,
        started.session,
        completion_candidate.candidate,
      );
      const completion_frame = wasm["decode-projected-term-frame"](
        completion_admitted.frame,
      );
      const completed_collectible = projectedItem(
        completion_frame,
        0,
        "world",
        "collectibles",
      );
      test["expect"](
        projectedString(completion_frame, "world", "objective", "state"),
      ).toBe("completed");
      test["expect"](projectedString(completed_collectible, "state")).toBe(
        "collected",
      );
      const launch_candidate = runCandidate(
        port,
        started.session,
        workbench["->InputConfiguration"](4, []),
      );
      const launch_admitted = admitCandidate(
        port,
        started.session,
        launch_candidate.candidate,
      );
      const launch_frame = wasm["decode-projected-term-frame"](
        launch_admitted.frame,
      );
      test["expect"](
        concatenate(projectedNumber(launch_frame, "player", "velocity", "y")),
      ).toBe("12");
      test["expect"](
        concatenate(projectedBoolean(launch_frame, "player", "grounded")),
      ).toBe("false");
      test["expect"](
        projectedString(launch_frame, "world", "objective", "state"),
      ).toBe("completed");
      port.disposeSession(started.session);
      return null;
    }),
);

test["test"](
  "real Wasm workbench hot-reloads Clause-only collect behavior with fenced prior generation",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file("../../test-vectors/browser/wasm-gameplay-v1/gameplay-v1.cwr1.hex").text(),
      file("../../test-vectors/browser/wasm-gameplay-v1/gameplay-spent-v1.cwr1.hex").text(),
    ]).then((assets) => {
      const raw_port = wasm["create-wasm-cartridge-port"](
        initialize_real_session_module(assets[0]),
        arena_policy(),
      );
      const sessions: unknown[] = [];
      const disposals: unknown[] = [];
      const rendered: workbench.WorkbenchEnvelope[] = [];
      const receipts: workbench.LifecycleReceipt[] = [];
      let scheduled_tick: () => unknown = () => undefined;
      let delay_next_candidate = false;
      const delayed_candidates: Array<() => unknown> = [];
      let changed_start_revision: unknown = null;
      const tracked_port = workbench["->CartridgePort"](
        raw_port.acceptPackage,
        (accepted_package, generation, complete) =>
          raw_port.startSession(accepted_package, generation, (result) => {
            if (result._tag === "SessionStarted") sessions.push(result.session);
            return complete(result);
          }),
        (session, fixed_tick, configuration, complete) =>
          raw_port.runCandidate(
            session,
            fixed_tick,
            configuration,
            (result) => {
              if (delay_next_candidate) {
                delay_next_candidate = false;
                delayed_candidates.push(() => complete(result));
                return undefined;
              } else {
                return complete(result);
              }
            },
          ),
        raw_port.requestAdmission,
        (session) => {
          disposals.push(session);
          return raw_port.disposeSession(session);
        },
      );
      const base_request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[1]),
      );
      const changed_request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[2]),
      );
      const controller = workbench["create-cartridge-workbench!"](
        tracked_port,
        workbench["->FixedTick"](16),
        arena_policy(),
        (__milliseconds, callback) => {
          scheduled_tick = callback;
          return () => undefined;
        },
        (frame) => rendered.push(frame),
        (receipt) => receipts.push(receipt),
        base_request,
      );
      const base_start = controller.snapshot();
      test["expect"](concatenate(base_start.generation)).toBe("1");
      test["expect"](concatenate(base_start.phase)).toBe("ready");
      test["expect"](json_string(base_start.frame)).toBe("[]");
      controller.observeInput(key_envelope("KeyD", "down"));
      scheduled_tick();
      const contact = wasm["decode-projected-term-frame"](
        controller.snapshot().frame,
      );
      const player_x = projectedNumber(contact, "player", "position", "x");
      const contact_collectible = projectedItem(
        contact,
        0,
        "world",
        "collectibles",
      );
      test["expect"](
        concatenate(projectedString(contact_collectible, "state")),
      ).toBe("collected");
      test["expect"](concatenate(player_x)).toBe(
        concatenate(projectedNumber(contact_collectible, "position", "x")),
      );
      controller.observeInput(key_envelope("KeyD", "up"));
      test["expect"](concatenate(controller.snapshot().generation)).toBe("1");
      const old_session = sessions[0];
      delay_next_candidate = true;
      scheduled_tick();
      test["expect"](concatenate(controller.snapshot().phase)).toBe(
        "candidate",
      );
      test["expect"](
        concatenate(controller.reloadPackage(changed_request)),
      ).toBe("true");
      const changed_start = controller.snapshot();
      const changed_revision = changed_start.revision;
      changed_start_revision = changed_revision;
      test["expect"](concatenate(changed_start.phase)).toBe("ready");
      test["expect"](concatenate(changed_start.generation)).toBe("2");
      test["expect"](json_string(changed_start.frame)).toBe("[]");
      test["expect"](concatenate(sessions.length)).toBe("2");
      test["expect"](concatenate(disposals.length)).toBe("1");
      test["expect"](
        concatenate(Object.is(disposals[0], old_session)),
      ).toBe("true");
      const delayed_candidate = delayed_candidates.shift();
      if (delayed_candidate === undefined) {
        throw new Error("tracked port did not retain the delayed candidate");
      }
      delayed_candidate();
      const after_stale = controller.snapshot();
      const stale_events = receipts.filter(
        (receipt) => receipt.event === "completion-stale",
      );
      test["expect"](concatenate(after_stale.generation)).toBe("2");
      test["expect"](
        concatenate(Object.is(after_stale.revision, changed_revision)),
      ).toBe("true");
      test["expect"](json_string(after_stale.frame)).toBe("[]");
      test["expect"](concatenate(stale_events.length)).toBe("1");
      const retired_result = runCandidateCompletion(
        raw_port,
        old_session,
        workbench["->InputConfiguration"](0, []),
      );
      if (retired_result._tag !== "CandidateFailed") {
        throw new Error("retired session unexpectedly produced a candidate");
      }
      test["expect"](concatenate(retired_result.reason).split("\n")[0]).toBe(
        "Error: Wasm session is disposed",
      );
      test["expect"](
        concatenate(
          Object.is(controller.snapshot().revision, changed_revision),
        ),
      ).toBe("true");
      controller.observeInput(key_envelope("KeyD", "down"));
      scheduled_tick();
      const changed_admitted = controller.snapshot();
      const changed_frame = wasm["decode-projected-term-frame"](
        changed_admitted.frame,
      );
      const changed_collectible = projectedItem(
        changed_frame,
        0,
        "world",
        "collectibles",
      );
      const rendered_length = rendered.length;
      const visible_frame = wasm["decode-projected-term-frame"](
        rendered[rendered_length - 1],
      );
      const visible_collectible = projectedItem(
        visible_frame,
        0,
        "world",
        "collectibles",
      );
      test["expect"](
        concatenate(projectedString(changed_collectible, "state")),
      ).toBe("spent");
      test["expect"](
        concatenate(projectedString(visible_collectible, "state")),
      ).toBe("spent");
      test["expect"](concatenate(changed_admitted.generation)).toBe("2");
      test["expect"](
        concatenate(
          !(
            json_string(changed_admitted.revision) ===
            json_string(changed_start_revision)
          ),
        ),
      ).toBe("true");
      test["expect"](concatenate(controller.dispose())).toBe("true");
      test["expect"](concatenate(disposals.length)).toBe("2");
      return null;
    }),
);

test["test"](
  "real Wasm hot-reloads Clause dash jump through Admission and passive rendering",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file("../../test-vectors/browser/wasm-gameplay-v1/gameplay-v1.cwr1.hex").text(),
      file("../../test-vectors/browser/wasm-gameplay-v1/gameplay-dash-jump-v1.cwr1.hex").text(),
    ]).then((assets) => {
      const port = wasm["create-wasm-cartridge-port"](
        initialize_real_session_module(assets[0]),
        arena_policy(),
      );
      const rendered: workbench.WorkbenchEnvelope[] = [];
      let scheduled_tick: () => unknown = () => undefined;
      const base_request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[1]),
      );
      const dash_request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[2]),
      );
      const controller = workbench["create-cartridge-workbench!"](
        port,
        workbench["->FixedTick"](16),
        arena_policy(),
        (__milliseconds, callback) => {
          scheduled_tick = callback;
          return () => undefined;
        },
        (frame) => rendered.push(frame),
        (__receipt) => undefined,
        base_request,
      );
      test["expect"](concatenate(controller.snapshot().generation)).toBe("1");
      test["expect"](concatenate(controller.reloadPackage(dash_request))).toBe(
        "true",
      );
      const dash_start = controller.snapshot();
      const dash_revision = dash_start.revision;
      test["expect"](concatenate(dash_start.generation)).toBe("2");
      controller.observeInput(key_envelope("Space", "down"));
      scheduled_tick();
      const admitted = controller.snapshot();
      const admitted_frame = wasm["decode-projected-term-frame"](
        admitted.frame,
      );
      const admitted_x = projectedNumber(
        admitted_frame,
        "player",
        "position",
        "x",
      );
      const admitted_y = projectedNumber(
        admitted_frame,
        "player",
        "position",
        "y",
      );
      const rendered_length = rendered.length;
      const visible_frame = wasm["decode-projected-term-frame"](
        rendered[rendered_length - 1],
      );
      const visible_x = projectedNumber(
        visible_frame,
        "player",
        "position",
        "x",
      );
      const visible_y = projectedNumber(
        visible_frame,
        "player",
        "position",
        "y",
      );
      test["expect"](concatenate(admitted.phase)).toBe("ready");
      test["expect"](concatenate(admitted.generation)).toBe("2");
      test["expect"](
        concatenate(
          !(json_string(admitted.revision) === json_string(dash_revision)),
        ),
      ).toBe("true");
      test["expect"](admitted_x > 0.0).toBe(true);
      test["expect"](admitted_y > 0.0).toBe(true);
      test["expect"](concatenate(visible_x)).toBe(concatenate(admitted_x));
      test["expect"](concatenate(visible_y)).toBe(concatenate(admitted_y));
      test["expect"](concatenate(controller.dispose())).toBe("true");
      return null;
    }),
);

test["test"](
  "real Wasm refuses an effect before intent admission",
  () =>
    Promise.all([
      file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(),
      file(
        "../../test-vectors/browser/wasm-ongoing-effect-v1/ongoing-effect-v1.cwr1.hex",
      ).text(),
    ]).then((assets) => {
      const module = initialize_real_session_module(assets[0]);
      const port = wasm["create-wasm-cartridge-port"](module, policy());
      const request = wasm["->ExactProcessRequest"](
        wasm["decode-cwr1-hex"](assets[1]),
      );
      const accepted = acceptPackage(port, request);
      const started = startSession(port, accepted.acceptedPackage);
      const session = started.session;
      const absent = wasm["query-pending-effect-intent!"](module, session);
      wasm["advance-session-occurrence!"](module, session, 0);
      const intent = wasm["emit-effect-intent!"](module, session);
      const queried = wasm["query-pending-effect-intent!"](module, session);
      if (absent.kind !== "effect-intent-absent") {
        throw new Error("effect intent unexpectedly existed before input");
      }
      if (queried.kind !== "effect-intent") {
        throw new Error("emitted effect intent was not queryable");
      }
      test["expect"](json_string(queried.intentId)).toBe(json_string(intent.intentId));
      test["expect"](() => wasm["issue-effect-authorization!"](module, session, intent.intentId)).toThrow();
      port.disposeSession(session);
      return null;
    }),
);


test.test("source preparation uses captured custody without advancing sequence", () => {
  const requests: number[][] = [];
  const calls: unknown[][] = [];
  let status = 0;
  const module = {
    ...module_for_bang([opened_event_bang(), cse_header_bang(1, 6)], requests),
    clause_session_v1_prepare_source(slot: number, generation: number, sequence: bigint, bytes: Uint8Array) {
      calls.push([slot, generation, sequence, [...bytes]]);
      bytes.fill(0);
      return status;
    },
  };
  const port = wasm["create-wasm-cartridge-port"](module, policy());
  const started = startSession(port, acceptPackage(port, wasm["->ExactProcessRequest"](minimal_cwr1_bang())).acceptedPackage);
  const bytes = wasm.decodeSourcePreparationHex("43505331ff");
  wasm.prepareSourceSession(module, started.session, bytes);
  wasm.prepareSourceSession(module, started.session, bytes);
  expect(calls).toEqual([[0, 1, 0n, [67, 80, 83, 49, 255]], [0, 1, 0n, [67, 80, 83, 49, 255]]]);
  expect(bytes).toEqual([67, 80, 83, 49, 255]);
  status = 3;
  expect(() => wasm.prepareSourceSession(module, started.session, bytes)).toThrow("checked source preparation rejected: 3");
  expect(() => wasm.prepareSourceSession(module, started.session, [256])).toThrow("source preparation exceeds bound");
  port.disposeSession(started.session);
  expect(() => wasm.prepareSourceSession(module, started.session, bytes)).toThrow("Wasm session is disposed");
});


test.test("compact scalar edits dispatch only the transaction with captured custody", () => {
  const requests: number[][] = [];
  const calls: unknown[][] = [];
  const module = {
    ...module_for_bang([opened_event_bang(), cse_header_bang(1, 6)], requests),
    clause_session_v1_scalar_edit_bulk(slot: number, generation: number, sequence: bigint, bytes: Uint8Array) {
      calls.push([slot, generation, sequence, [...bytes]]);
      bytes.fill(0);
      return 3;
    },
    clause_session_v1_source_edit_bulk() { throw new Error("scalar edit dispatched as structural edit"); },
  };
  const port = wasm["create-wasm-cartridge-port"](module, policy());
  const request = wasm["->ExactProcessRequest"](minimal_cwr1_bang());
  const started = startSession(port, acceptPackage(port, request).acceptedPackage);
  const transaction = wasm["decode-cet1-hex"]("43455831ff");
  const first = wasm.editSourceSession(module, started.session, 2, request, transaction, policy());
  expect(first._tag).toBe("SessionFailed");
  if (first._tag === "SessionFailed") expect(first.reason).toContain("checked source edit rejected: 3");
  expect(wasm.editSourceSession(module, started.session, 2, request, transaction, policy())._tag).toBe("SessionFailed");
  expect(calls).toEqual([[0, 1, 0n, [67, 69, 88, 49, 255]], [0, 1, 0n, [67, 69, 88, 49, 255]]]);
  expect(transaction).toEqual([67, 69, 88, 49, 255]);
  port.disposeSession(started.session);
});


test.test("direct hex request custody preserves every octet and strict transport checks", () => {
  const bytes = minimal_cwr1_bang();
  const hex = bytes.map(byte => byte.toString(16).padStart(2, "0")).join("");
  const request = wasm.decodeProcessRequestHex(hex);
  expect(typeof request.bytes).toBe("string");
  expect(Object.isFrozen(request)).toBe(true);
  const port = wasm["create-wasm-cartridge-port"](module_for_bang([opened_event_bang(), cse_header_bang(1, 6)], []), policy());
  const started = startSession(port, acceptPackage(port, request).acceptedPackage);
  port.disposeSession(started.session);
  const octets = wasm.decodeProcessRequestHex(Array.from({length: 256}, (_, byte) => byte.toString(16).padStart(2, "0")).join(" "));
  expect(typeof octets.bytes === "string" && Array.from(octets.bytes, value => value.charCodeAt(0))).toEqual(Array.from({length:256}, (_, byte) => byte));
  expect(() => wasm.decodeProcessRequestHex("0")).toThrow("incomplete byte");
  expect(() => wasm.decodeProcessRequestHex("AA")).toThrow("non-hex unit");
  expect(() => wasm.decodeProcessRequestHex(" ")).toThrow("empty");
  expect(() => wasm.decodeProcessRequestHex("00".repeat(4 * 1024 * 1024 + 1))).toThrow("byte bound");
});
