import * as workbench from "./workbench.js";

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

function appendValue<T>(values: readonly T[], value: T): T[] {
  return [...values, value];
}

function countValues(values: { readonly length: number }): number {
  return values.length;
}

function concatenate(...values: readonly unknown[]): string {
  return values.map(String).join("");
}

function classifyError(error: unknown): 0 {
  if (error instanceof Error) return 0;
  throw error;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

const cwr1_max_bytes = 4 * 1024 * 1024;

const cwr1_hex_max_source_units = 3 * cwr1_max_bytes;

const cwo1_max_bytes = 64 * 1024;

const cwo1_prefix_bytes = 4 + 32 + 32;

const cwo1_identity_bytes = 32;

const cwo1_max_values = 256;

const cse1_max_bytes = 64 * 1024;

const cse1_projected_term_max_properties = cse1_max_bytes;

const cse1_projected_term_json_max_source_units = 4 * cse1_max_bytes + 1;

const session_command_max_bytes = 1024 * 1024;

const session_command_limit = Number.MAX_SAFE_INTEGER;

const current_admission_trace_retention = 1;

const identity_bytes = 32;

const canonical_term_atom_min_bytes = 1 + 4 + 4 + 1;

const canonical_term_triple_path_min_bytes =
  1 + 2 * canonical_term_atom_min_bytes;

const cse1_projected_term_max_depth = Math.trunc(
  (cse1_max_bytes - 2 * identity_bytes - canonical_term_atom_min_bytes) /
    canonical_term_triple_path_min_bytes,
);

const allocation_epoch_bytes = 304;

export type ExactBytes = readonly number[];

type CanonicalBytes = ExactBytes | string;

interface Cell<T> {
  value: T;
  watches: Record<
    string,
    (key: string, cell: Cell<T>, previous: T, next: T) => void
  >;
}

interface SessionHandle {
  readonly slot: number;
  readonly generation: number;
}

export interface ExactProcessRequest {
  readonly _tag: "ExactProcessRequest";
  readonly bytes: ExactBytes;
}

export interface ExactProcessObservation {
  readonly _tag: "ExactProcessObservation";
  readonly bytes: ExactBytes;
}

interface WasmCandidate {
  readonly _tag: "WasmCandidate";
  readonly candidateId: ExactBytes;
  readonly base: ExactBytes;
}

interface WasmSession {
  readonly _tag: "WasmSession";
  readonly handle: SessionHandle;
  readonly sourceGeneration: number;
  readonly packageId: ExactBytes;
  readonly sessionId: ExactBytes;
  readonly allocation: ExactBytes;
  readonly world: Cell<ExactBytes>;
  readonly sequence: Cell<number>;
  readonly occurrences: readonly ExactBytes[];
  readonly disposed: Cell<boolean>;
}

interface PersistentCartridge {
  readonly _tag: "PersistentCartridge";
  readonly openBytes: ExactBytes;
  readonly occurrences: readonly ExactBytes[];
}

export interface Cwo1Observation {
  readonly _tag: "Cwo1Observation";
  readonly observationId: ExactBytes;
  readonly stateRevisionId: ExactBytes;
  readonly values: readonly (number | boolean)[];
}

interface ParsedBlob {
  readonly bytes: ExactBytes;
  readonly next: number;
}

interface SessionWasmModule {
  readonly clause_session_v1_open_bulk: typeof import("#clause-runtime-wasm").clause_session_v1_open_bulk;
  readonly clause_session_v1_command_bulk: typeof import("#clause-runtime-wasm").clause_session_v1_command_bulk;
  readonly clause_session_v1_event_bulk: typeof import("#clause-runtime-wasm").clause_session_v1_event_bulk;
  readonly clause_session_v1_reclaim_retired: typeof import("#clause-runtime-wasm").clause_session_v1_reclaim_retired;
}

interface SessionApi {
  readonly open: (request: Uint8Array<ArrayBuffer>) => number;
  readonly command: (request: Uint8Array<ArrayBuffer>) => number;
  readonly event: () => Uint8Array;
  readonly reclaim: () => boolean;
}

interface ProcessWasmModule {
  readonly clause_process_v1_reset: typeof import("#clause-runtime-wasm").clause_process_v1_reset;
  readonly clause_process_v1_request_push: typeof import("#clause-runtime-wasm").clause_process_v1_request_push;
  readonly clause_process_v1_dispatch: typeof import("#clause-runtime-wasm").clause_process_v1_dispatch;
  readonly clause_process_v1_response_len: typeof import("#clause-runtime-wasm").clause_process_v1_response_len;
  readonly clause_process_v1_response_byte: typeof import("#clause-runtime-wasm").clause_process_v1_response_byte;
}

interface Cse1EventBase {
  readonly kind: string;
  readonly slot: number;
  readonly generation: number;
  readonly sequence: number;
}

type AdmissionProjection = null | Readonly<{
  observationId: ExactBytes;
  termBytes: ExactBytes;
}>;

export type Cse1Event =
  | (Cse1EventBase &
      Readonly<{
        kind: "opened";
        packageId: ExactBytes;
        sessionId: ExactBytes;
        world: ExactBytes;
        allocation: ExactBytes;
      }>)
  | (Cse1EventBase &
      Readonly<{
        kind: "candidate";
        candidateId: ExactBytes;
        base: ExactBytes;
      }>)
  | (Cse1EventBase &
      Readonly<{
        kind: "issued";
        authorization: ExactBytes;
        packageId: ExactBytes;
        sessionId: ExactBytes;
        base: ExactBytes;
        candidateId: ExactBytes;
      }>)
  | (Cse1EventBase &
      Readonly<{
        kind: "admission";
        predecessor: ExactBytes;
        successor: ExactBytes;
        admissionId: ExactBytes;
        judgmentId: ExactBytes;
        run: ExactBytes;
        activation: ExactBytes;
        sessionId: ExactBytes;
        projection: AdmissionProjection;
      }>)
  | (Cse1EventBase & Readonly<{ kind: "disposed" }>)
  | (Cse1EventBase & Readonly<{ kind: "rejected"; reason: number }>)
  | (Cse1EventBase &
      Readonly<{ kind: "candidate-rejected"; diagnostic: string }>)
  | (Cse1EventBase &
      Readonly<{
        kind: "suspended";
        step: ExactBytes;
        continuation: ExactBytes;
        run: ExactBytes;
        activation: ExactBytes;
        before: ExactBytes;
        after: ExactBytes;
        remainingBudget: number;
        stateRevisionCount: number;
      }>)
  | (Cse1EventBase &
      Readonly<{
        kind: "resumed";
        occurrence: ExactBytes;
        step: ExactBytes;
        continuation: ExactBytes;
        run: ExactBytes;
        activation: ExactBytes;
        before: ExactBytes;
        after: ExactBytes;
        remainingBudget: number;
        stateRevisionCount: number;
      }>)
  | (Cse1EventBase &
      Readonly<{
        kind: "effect-intent";
        intentId: ExactBytes;
        run: ExactBytes;
        activation: ExactBytes;
        step: ExactBytes;
        contractIndex: number;
        capability: Readonly<{ snapshot: ExactBytes; local: number }>;
        scope: Readonly<{
          application: Readonly<{ snapshot: ExactBytes; local: number }>;
          mode: Readonly<{
            snapshot: ExactBytes;
            operator: number;
            local: number;
          }>;
          programRevision: ExactBytes;
          world: ExactBytes;
          sessionId: ExactBytes;
          remainingBudget: number;
        }>;
        actionBytes: ExactBytes;
        resourceBytes: ExactBytes;
        payloadBytes: ExactBytes;
        stateRevisionCount: number;
      }>)
  | (Cse1EventBase &
      Readonly<{ kind: "effect-intent-absent"; stateRevisionCount: number }>)
  | (Cse1EventBase &
      Readonly<{
        kind: "effect-authorization";
        authorizationId: ExactBytes;
        intentId: ExactBytes;
        stateRevisionCount: number;
      }>)
  | (Cse1EventBase &
      Readonly<{
        kind: "effect-attempt";
        attemptId: ExactBytes;
        intentId: ExactBytes;
        authorizationId: ExactBytes;
        actionBytes: ExactBytes;
        resourceBytes: ExactBytes;
        payloadBytes: ExactBytes;
        stateRevisionCount: number;
      }>)
  | (Cse1EventBase &
      Readonly<{
        kind: "effect-settlement";
        intentId: ExactBytes;
        attemptId: ExactBytes;
        receiptId: ExactBytes | null;
        observationId: ExactBytes | null;
        judgmentId: ExactBytes;
        disposition: "receipt-observed" | "no-receipt";
        stateRevisionCount: number;
      }>)
  | (Cse1EventBase & Readonly<{ kind: "input" }>);

type PhysicalObservation =
  | Readonly<{
      kind: "input";
      source:
        | Readonly<{
            kind: "keyboard";
            sequence: number;
            code: ExactBytes;
            phase: number;
          }>
        | Readonly<{
            kind: "scalar";
            sequence: number;
            channel: ExactBytes;
            value: number;
          }>
        | Readonly<{
            kind: "referent";
            sequence: number;
            generation: number;
            channel: ExactBytes;
            value: ProjectedReferent;
          }>;
    }>
  | Readonly<{ kind: "candidate"; ordinal: number }>
  | null;

type TermNode =
  | Readonly<{ kind: "atom"; atomKind: CanonicalBytes; payload: CanonicalBytes }>
  | Readonly<{
      kind: "triple";
      slots: readonly [TermNode, TermNode, TermNode];
    }>;

interface DecodedTermNode {
  readonly node: TermNode;
  readonly next: number;
}

export interface ProjectedObject {
  readonly [key: string]: ProjectedValue;
}

export type ProjectedReferent = Readonly<{
  kind: "referent";
  domain: number;
  identity:
    | Readonly<{ kind: "declared"; value: number }>
    | Readonly<{ kind: "created"; value: readonly number[] }>;
}>;

function checked_referent(value: unknown): ProjectedReferent {
  const u32 = (value: unknown): value is number =>
    typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 0xffffffff;
  if (typeof value !== "object" || value === null || !("kind" in value) ||
      value.kind !== "referent" || !("domain" in value) || !u32(value.domain) ||
      !("identity" in value)) {
    throw new Error("referent input is malformed");
  }
  const identity = value.identity;
  if (typeof identity !== "object" || identity === null ||
      !("kind" in identity) || !("value" in identity)) {
    throw new Error("referent identity is malformed");
  }
  if (identity.kind === "declared" && u32(identity.value)) {
    return Object.freeze({ kind: "referent", domain: value.domain,
      identity: Object.freeze({ kind: "declared", value: identity.value }) });
  }
  if (identity.kind === "created" && Array.isArray(identity.value) &&
      identity.value.length === 32 && identity.value.every((byte: unknown) => u32(byte) && byte <= 255)) {
    return Object.freeze({ kind: "referent", domain: value.domain,
      identity: Object.freeze({ kind: "created", value: Object.freeze([...identity.value]) }) });
  }
  throw new Error("referent identity is malformed");
}

export type ProjectedValue =
  | number
  | boolean
  | string
  | readonly ProjectedValue[]
  | ProjectedObject;

function hex_whitespace_code_p(code: number): boolean {
  return (
    equivalent(code, 9) ||
    equivalent(code, 10) ||
    equivalent(code, 13) ||
    equivalent(code, 32)
  );
}

function lowercase_hex_nibble(code: number): number {
  return 48 <= code && code <= 57
    ? code - 48
    : 97 <= code && code <= 102
      ? code - 97 + 10
      : -1;
}

function decode_cwr1_hex(source: unknown): ExactBytes {
  if (typeof source === "string") {
    const length = source.length;
    if (equivalent(length, 0) || length > cwr1_hex_max_source_units) {
      (() => {
        throw new Error("CWR1 hex transport is outside its source bound");
      })();
    }
    const bytes = [];
    return (() => {
      let index = 0;
      let high = -1;
      while (true) {
        if (index === length) {
          return !equivalent(high, -1)
            ? (() => {
                throw new Error("CWR1 hex transport has an incomplete byte");
              })()
            : equivalent(countValues(bytes), 0)
              ? (() => {
                  throw new Error("CWR1 hex transport is empty");
                })()
              : Object.freeze(bytes);
        } else {
          const code = source.charCodeAt(index);
          const nibble = lowercase_hex_nibble(code);
          if (hex_whitespace_code_p(code)) {
            const _recur_0 = index + 1;
            const _recur_1 = high;
            index = _recur_0;
            high = _recur_1;
            continue;
          } else if (nibble < 0) {
            return (() => {
              throw new Error("CWR1 hex transport contains a non-hex unit");
            })();
          } else if (equivalent(high, -1)) {
            const _recur_0 = index + 1;
            const _recur_1 = nibble;
            index = _recur_0;
            high = _recur_1;
            continue;
          } else if (countValues(bytes) >= cwr1_max_bytes) {
            return (() => {
              throw new Error("CWR1 hex transport exceeds its byte bound");
            })();
          } else {
            bytes.push(high * 16 + nibble);
            const _recur_0 = index + 1;
            const _recur_1 = -1;
            index = _recur_0;
            high = _recur_1;
            continue;
          }
        }
      }
    })();
  } else {
    return (() => {
      throw new Error("CWR1 hex transport must be text");
    })();
  }
}

function ExactProcessRequest(bytes: ExactBytes): ExactProcessRequest {
  return Object.freeze({ _tag: "ExactProcessRequest", bytes });
}

function exactprocessrequest_bytes(r: ExactProcessRequest): ExactBytes {
  return r.bytes;
}

function ExactProcessObservation(bytes: ExactBytes): ExactProcessObservation {
  return Object.freeze({ _tag: "ExactProcessObservation", bytes });
}

function exactprocessobservation_bytes(r: ExactProcessObservation): ExactBytes {
  return r.bytes;
}

function WasmCandidate(
  candidateId: ExactBytes,
  base: ExactBytes,
): WasmCandidate {
  return Object.freeze({ _tag: "WasmCandidate", candidateId, base });
}

function wasmcandidate_candidateId(r: WasmCandidate): ExactBytes {
  return r.candidateId;
}

function wasmcandidate_base(r: WasmCandidate): ExactBytes {
  return r.base;
}

function WasmSession(
  handle: SessionHandle,
  sourceGeneration: number,
  packageId: ExactBytes,
  sessionId: ExactBytes,
  allocation: ExactBytes,
  world: Cell<ExactBytes>,
  sequence: Cell<number>,
  occurrences: readonly ExactBytes[],
  disposed: Cell<boolean>,
): WasmSession {
  return Object.freeze({
    _tag: "WasmSession",
    handle,
    sourceGeneration,
    packageId,
    sessionId,
    allocation,
    world,
    sequence,
    occurrences,
    disposed,
  });
}

function wasmsession_handle(r: WasmSession): SessionHandle {
  return r.handle;
}

function wasmsession_packageId(r: WasmSession): ExactBytes {
  return r.packageId;
}

function wasmsession_sessionId(r: WasmSession): ExactBytes {
  return r.sessionId;
}

function wasmsession_allocation(r: WasmSession): ExactBytes {
  return r.allocation;
}

function wasmsession_world(r: WasmSession): Cell<ExactBytes> {
  return r.world;
}

function wasmsession_sequence(r: WasmSession): Cell<number> {
  return r.sequence;
}

function wasmsession_occurrences(r: WasmSession): readonly ExactBytes[] {
  return r.occurrences;
}

function wasmsession_disposed(r: WasmSession): Cell<boolean> {
  return r.disposed;
}

function PersistentCartridge(
  openBytes: ExactBytes,
  occurrences: readonly ExactBytes[],
): PersistentCartridge {
  return Object.freeze({ _tag: "PersistentCartridge", openBytes, occurrences });
}

function require_persistent_cartridge(value: unknown): PersistentCartridge {
  if (
    !isRecord(value) ||
    value._tag !== "PersistentCartridge" ||
    !exact_byte_array_p(value.openBytes, cwr1_max_bytes) ||
    !Array.isArray(value.occurrences) ||
    value.occurrences.length === 0 ||
    !value.occurrences.every((occurrence: unknown) =>
      exact_byte_array_p(occurrence, cwr1_max_bytes),
    )
  ) {
    throw new Error("persistent cartridge is invalid");
  }
  return PersistentCartridge(
    value.openBytes,
    Object.freeze(value.occurrences as ExactBytes[]),
  );
}

function persistentcartridge_openBytes(r: PersistentCartridge): ExactBytes {
  return r.openBytes;
}

function persistentcartridge_occurrences(
  r: PersistentCartridge,
): readonly ExactBytes[] {
  return r.occurrences;
}

function Cwo1Observation(
  observationId: ExactBytes,
  stateRevisionId: ExactBytes,
  values: readonly (number | boolean)[],
): Cwo1Observation {
  return Object.freeze({
    _tag: "Cwo1Observation",
    observationId,
    stateRevisionId,
    values,
  });
}

function cwo1observation_observationId(r: Cwo1Observation): ExactBytes {
  return r.observationId;
}

function cwo1observation_stateRevisionId(r: Cwo1Observation): ExactBytes {
  return r.stateRevisionId;
}

function cwo1observation_values(
  r: Cwo1Observation,
): readonly (number | boolean)[] {
  return r.values;
}

function exact_byte_array_p(
  bytes: unknown,
  maximum: number,
): bytes is ExactBytes {
  return (
    Array.isArray(bytes) &&
    bytes.length >= 1 &&
    bytes.length <= maximum &&
    bytes.every(
      (byte: unknown) =>
        typeof byte === "number" &&
        Number.isInteger(byte) &&
        byte >= 0 &&
        byte <= 255,
    )
  );
}

function require_request(request: unknown): ExactProcessRequest {
  if (
    typeof request !== "object" ||
    request === null ||
    !("bytes" in request) ||
    !exact_byte_array_p(request.bytes, cwr1_max_bytes)
  ) {
    throw new Error("cartridge request must carry bounded exact bytes");
  }
  return ExactProcessRequest(
    frozen_byte_range(request.bytes, 0, request.bytes.length),
  );
}

function process_status(status: unknown): number {
  return typeof status === "number" && Number.isSafeInteger(status)
    ? status
    : -1;
}

function byte_at(bytes: CanonicalBytes, index: number): number {
  const byte =
    typeof bytes === "string" ? bytes.charCodeAt(index) : bytes[index];
  if (byte === undefined) throw new Error("exact byte index is out of range");
  return byte;
}

function little_u16(bytes: ExactBytes, offset: number): number {
  return byte_at(bytes, offset) + 256 * byte_at(bytes, offset + 1);
}

function little_u32(bytes: ExactBytes, offset: number): number {
  return (
    byte_at(bytes, offset) +
    256 * byte_at(bytes, offset + 1) +
    65536 * byte_at(bytes, offset + 2) +
    16777216 * byte_at(bytes, offset + 3)
  );
}

function big_u32(bytes: CanonicalBytes, offset: number): number {
  return (
    16777216 * byte_at(bytes, offset) +
    65536 * byte_at(bytes, offset + 1) +
    256 * byte_at(bytes, offset + 2) +
    byte_at(bytes, offset + 3)
  );
}

function little_safe_u64(bytes: ExactBytes, offset: number): number {
  const low = little_u32(bytes, offset);
  const high = little_u32(bytes, offset + 4);
  return high > 2097151
    ? (() => {
        throw new Error(
          "64-bit transport value exceeds exact JavaScript range",
        );
      })()
    : low + high * 4294967296;
}

function append_u32_bang(bytes: number[], value: number): number {
  bytes.push(value % 256);
  bytes.push(Math.trunc(value / 256) % 256);
  bytes.push(Math.trunc(value / 65536) % 256);
  return bytes.push(Math.trunc(value / 16777216) % 256);
}

function append_u64_bang(bytes: number[], value: number): number {
  append_u32_bang(bytes, value % 4294967296);
  return append_u32_bang(bytes, Math.trunc(value / 4294967296));
}

function append_blob_bang(bytes: number[], value: ExactBytes): void {
  append_u32_bang(bytes, value.length);
  value.forEach((byte: number) => {
    bytes.push(byte);
  });
}

function require_range(
  bytes: CanonicalBytes,
  offset: number,
  length: number,
  label: string,
): number {
  const end = offset + length;
  return offset < 0 || length < 0 || end > bytes.length
    ? (() => {
        throw new Error(concatenate(label, " is truncated"));
      })()
    : end;
}

function frozen_byte_range(
  bytes: ExactBytes,
  start: number,
  end: number,
): ExactBytes {
  const result: number[] = [];
  return (() => {
    let index = start;
    while (true) {
      if (index === end) {
        return Object.freeze(result);
      } else {
        result.push(byte_at(bytes, index));
        const _recur_0 = index + 1;
        index = _recur_0;
        continue;
      }
    }
  })();
}

function canonical_byte_range(
  bytes: CanonicalBytes,
  start: number,
  end: number,
): CanonicalBytes {
  return typeof bytes === "string"
    ? bytes.slice(start, end)
    : frozen_byte_range(bytes, start, end);
}

function exact_bytes_to_binary_text(bytes: ExactBytes): string {
  const chunks: string[] = [];
  const chunk_size = 4096;
  for (let start = 0; start < bytes.length; start += chunk_size) {
    const end = Math.min(start + chunk_size, bytes.length);
    let chunk = "";
    for (let index = start; index < end; index += 1) {
      chunk += String.fromCharCode(byte_at(bytes, index));
    }
    chunks.push(chunk);
  }
  return chunks.join("");
}

function finite_f64(bytes: CanonicalBytes, offset: number): number {
  const packed = new Uint8Array(8);
  for (let index = 0; index < 8; index += 1) {
    packed[index] = byte_at(bytes, offset + index);
  }
  const view = new DataView(packed.buffer);
  const value = view.getFloat64(0, true);
  return ((_truthy) => _truthy !== false && _truthy != null)(
    ((_logical) =>
      _logical !== false && _logical != null
        ? !(
            equivalent(value, 0.0) &&
            equivalent(byte_at(bytes, offset + 7), 128)
          )
        : _logical)(Number.isFinite(value)),
  )
    ? value
    : (() => {
        throw new Error("CWO1 number is not canonical finite f64");
      })();
}

function decode_cwo1_observation(incoming: unknown): Cwo1Observation {
  if (exact_byte_array_p(incoming, cwo1_max_bytes)) {
    const length = incoming.length;
    if (length < cwo1_prefix_bytes + 2) {
      (() => {
        throw new Error("CWO1 response is truncated");
      })();
    }
    if (
      !equivalent(byte_at(incoming, 0), 67) ||
      !equivalent(byte_at(incoming, 1), 87) ||
      !equivalent(byte_at(incoming, 2), 79) ||
      !equivalent(byte_at(incoming, 3), 49)
    ) {
      (() => {
        throw new Error("CWO1 response magic is invalid");
      })();
    }
    const observation_id = frozen_byte_range(
      incoming,
      4,
      4 + cwo1_identity_bytes,
    );
    const state_revision_id = frozen_byte_range(
      incoming,
      4 + cwo1_identity_bytes,
      cwo1_prefix_bytes,
    );
    const count = little_u16(incoming, cwo1_prefix_bytes);
    if (count > cwo1_max_values) {
      (() => {
        throw new Error("CWO1 render value count is out of bounds");
      })();
    }
    let offset = cwo1_prefix_bytes + 2;
    const values: Array<number | boolean> = [];
    for (let index = 0; index < count; index += 1) {
      if (offset >= length) throw new Error("CWO1 value is truncated");
      const tag = byte_at(incoming, offset);
      if (tag === 0) {
        if (offset + 9 > length) throw new Error("CWO1 number is truncated");
        values.push(finite_f64(incoming, offset + 1));
        offset += 9;
      } else if (tag === 1) {
        if (offset + 2 > length) throw new Error("CWO1 boolean is truncated");
        const value = byte_at(incoming, offset + 1);
        if (value > 1) throw new Error("CWO1 boolean is invalid");
        values.push(value === 1);
        offset += 2;
      } else {
        throw new Error("CWO1 value tag is invalid");
      }
    }
    if (offset !== length) throw new Error("CWO1 response has trailing bytes");
    return Cwo1Observation(
      observation_id,
      state_revision_id,
      Object.freeze(values),
    );
  } else {
    return (() => {
      throw new Error("CWO1 response must carry bounded exact bytes");
    })();
  }
}

function project_frame(
  policy: workbench.WorkbenchPolicy,
  observation: Cwo1Observation,
): workbench.WorkbenchEnvelope {
  return workbench["create-workbench-envelope"](
    policy,
    JSON.stringify(observation.values),
  );
}

function dispatch_exact_request(
  module: ProcessWasmModule,
  request: unknown,
): ExactProcessObservation {
  const checked = require_request(request);
  const reset = module.clause_process_v1_reset;
  const push = module.clause_process_v1_request_push;
  const dispatch = module.clause_process_v1_dispatch;
  const response_length = module.clause_process_v1_response_len;
  const response_byte = module.clause_process_v1_response_byte;
  reset();
  checked.bytes.forEach((byte: number) => {
    const status = process_status(push(byte));
    if (!equivalent(status, 0)) {
      (() => {
        throw new Error(
          concatenate("CWR1 byte transfer rejected with status ", status),
        );
      })();
    }
  });
  const status = process_status(dispatch());
  if (!equivalent(status, 0)) {
    (() => {
      throw new Error(
        concatenate("CWR1 dispatch rejected with status ", status),
      );
    })();
  }
  const length = process_status(response_length());
  if (length < cwo1_prefix_bytes + 2 || length > cwo1_max_bytes) {
    (() => {
      throw new Error("CWO1 response length is out of bounds");
    })();
  }
  const bytes: number[] = [];
  for (let index = 0; index < length; index += 1) {
    const byte = process_status(response_byte(index));
    if (byte < 0 || byte > 255)
      throw new Error("CWO1 response byte is out of bounds");
    bytes.push(byte);
  }
  return ExactProcessObservation(Object.freeze(bytes));
}

function parse_blob(
  bytes: ExactBytes,
  offset: number,
  maximum: number,
  label: string,
): ParsedBlob {
  const header_end = require_range(bytes, offset, 4, label);
  const length = little_u32(bytes, offset);
  const __bound =
    length > maximum
      ? (() => {
          return (() => {
            throw new Error(concatenate(label, " exceeds its bound"));
          })();
        })()
      : null;
  const end = require_range(bytes, header_end, length, label);
  return { bytes: frozen_byte_range(bytes, header_end, end), next: end };
}

function require_allocation_epoch(record: ParsedBlob): ExactBytes {
  const bytes = record.bytes;
  return equivalent(bytes.length, allocation_epoch_bytes) &&
    equivalent(frozen_byte_range(bytes, 0, 4), [82, 65, 69, 49])
    ? bytes
    : (() => {
        throw new Error("runtime allocation epoch has an invalid shape");
      })();
}

function parse_persistent_cartridge_bang(
  request: unknown,
): PersistentCartridge {
  const checked = require_request(request);
  const bytes = checked.bytes;
  if (
    bytes.length < 4 ||
    !equivalent(byte_at(bytes, 0), 67) ||
    !equivalent(byte_at(bytes, 1), 87) ||
    !equivalent(byte_at(bytes, 2), 82) ||
    !equivalent(byte_at(bytes, 3), 49)
  ) {
    (() => {
      throw new Error("persistent cartridge must carry exact CWR1 bytes");
    })();
  }
  const package_record = parse_blob(bytes, 4, cwr1_max_bytes, "CWR1 package");
  const package_end = package_record.next;
  const application_end = require_range(
    bytes,
    package_end,
    4,
    "CWR1 application",
  );
  const physical_plan = parse_blob(
    bytes,
    application_end,
    cwr1_max_bytes,
    "CWR1 physical plan",
  );
  const allocation = parse_blob(
    bytes,
    physical_plan.next,
    allocation_epoch_bytes,
    "CWR1 allocation epoch",
  );
  const allocation_bytes = require_allocation_epoch(allocation);
  const authority_start = allocation.next;
  const identities_end = require_range(
    bytes,
    authority_start,
    9 * identity_bytes,
    "CWR1 authority identities",
  );
  const occurrence_evidence = parse_blob(
    bytes,
    identities_end,
    cwr1_max_bytes,
    "CWR1 occurrence evidence",
  );
  const judgment_id_end = require_range(
    bytes,
    occurrence_evidence.next,
    identity_bytes,
    "CWR1 Judgment evidence identity",
  );
  const judgment_evidence = parse_blob(
    bytes,
    judgment_id_end,
    cwr1_max_bytes,
    "CWR1 Judgment evidence",
  );
  const admission_id_end = require_range(
    bytes,
    judgment_evidence.next,
    identity_bytes,
    "CWR1 Admission evidence identity",
  );
  const admission_evidence = parse_blob(
    bytes,
    admission_id_end,
    cwr1_max_bytes,
    "CWR1 Admission evidence",
  );
  const budget_end = require_range(
    bytes,
    admission_evidence.next,
    8,
    "CWR1 budget",
  );
  const count_end = require_range(
    bytes,
    budget_end,
    2,
    "CWR1 occurrence count",
  );
  const occurrence_count = little_u16(bytes, budget_end);
  let occurrence_offset = count_end;
  const occurrences: ExactBytes[] = [];
  for (let index = 0; index < occurrence_count; index += 1) {
    const occurrence = parse_blob(
      bytes,
      occurrence_offset,
      cwr1_max_bytes,
      "CWR1 occurrence",
    );
    occurrence_offset = occurrence.next;
    occurrences.push(occurrence.bytes);
  }
  const occurrences_result = { next: occurrence_offset, values: occurrences };
  const slot_count_end = require_range(
    bytes,
    occurrences_result.next,
    2,
    "CWR1 projection count",
  );
  const slot_count = little_u16(bytes, occurrences_result.next);
  const final_offset = require_range(
    bytes,
    slot_count_end,
    slot_count * 2,
    "CWR1 legacy projection",
  );
  if (
    !equivalent(final_offset, bytes.length)
  ) {
    (() => {
      throw new Error("CWR1 cartridge shape is incomplete");
    })();
  }
  const open_bytes = [67, 87, 83, 49];
  bytes.slice(4, physical_plan.next).forEach((byte: number) => {
    open_bytes.push(byte);
  });
  bytes.slice(authority_start, budget_end).forEach((byte: number) => {
    open_bytes.push(byte);
  });
  open_bytes.push(0);
  append_u64_bang(open_bytes, session_command_limit);
  append_u32_bang(open_bytes, session_command_max_bytes);
  append_u32_bang(open_bytes, cse1_max_bytes);
  open_bytes.push(current_admission_trace_retention);
  return PersistentCartridge(
    Object.freeze(open_bytes),
    Object.freeze(occurrences_result.values),
  );
}

function process_request_occurrences_bang(
  request: unknown,
): readonly ExactBytes[] {
  return parse_persistent_cartridge_bang(request).occurrences;
}

function is_session_wasm_module(module: unknown): module is SessionWasmModule {
  return (
    typeof module === "object" &&
    module !== null &&
    "clause_session_v1_open_bulk" in module &&
    typeof module.clause_session_v1_open_bulk === "function" &&
    "clause_session_v1_command_bulk" in module &&
    typeof module.clause_session_v1_command_bulk === "function" &&
    "clause_session_v1_event_bulk" in module &&
    typeof module.clause_session_v1_event_bulk === "function" &&
    "clause_session_v1_reclaim_retired" in module &&
    typeof module.clause_session_v1_reclaim_retired === "function"
  );
}

function session_module_functions(module: unknown): SessionApi {
  if (!is_session_wasm_module(module)) {
    throw new Error("Wasm module lacks bulk persistent Clause session I/O");
  }
  const open = module.clause_session_v1_open_bulk;
  const command = module.clause_session_v1_command_bulk;
  const event = module.clause_session_v1_event_bulk;
  const reclaim = module.clause_session_v1_reclaim_retired;
  if (open == null || command == null || event == null || reclaim == null) {
    (() => {
      throw new Error("Wasm module lacks bulk persistent Clause session I/O");
    })();
  }
  return { open: open, command: command, event: event, reclaim: reclaim };
}

function dispatch_session_request(
  module: unknown,
  request: unknown,
  operation: "open" | "command",
): ExactBytes {
  if (exact_byte_array_p(request, session_command_max_bytes)) {
    const api = session_module_functions(module);
    const typed_request = new Uint8Array(request);
    const status = process_status(
      operation === "open"
        ? api.open(typed_request)
        : api.command(typed_request),
    );
    if (!equivalent(status, 0)) {
      (() => {
        throw new Error(
          concatenate(
            "persistent session ",
            operation,
            " rejected with status ",
            status,
          ),
        );
      })();
    }
    const event = Array.from(api.event());
    const length = event.length;
    if (length < 21 || length > cse1_max_bytes) {
      (() => {
        throw new Error("CSE1 event length is out of bounds");
      })();
    }
    return exact_byte_array_p(event, cse1_max_bytes)
      ? Object.freeze(event)
      : (() => {
          throw new Error("CSE1 bulk event byte is out of bounds");
        })();
  } else {
    return (() => {
      throw new Error(
        "persistent session request must carry bounded exact bytes",
      );
    })();
  }
}

function decode_cse1_event(bytes: unknown): Cse1Event {
  if (exact_byte_array_p(bytes, cse1_max_bytes)) {
    if (
      bytes.length < 21 ||
      !equivalent(frozen_byte_range(bytes, 0, 4), [67, 83, 69, 49])
    ) {
      (() => {
        throw new Error("CSE1 event magic is invalid");
      })();
    }
    const slot = little_u32(bytes, 4);
    const generation = little_u32(bytes, 8);
    const sequence = little_safe_u64(bytes, 12);
    const tag = byte_at(bytes, 20);
    const identity_at = (offset: number): ExactBytes =>
      frozen_byte_range(bytes, offset, offset + identity_bytes);
    return equivalent(tag, 1)
      ? (() => {
          const allocation_offset = 21 + 5 * identity_bytes + 4;
          const allocation = parse_blob(
            bytes,
            allocation_offset,
            allocation_epoch_bytes,
            "CSE1 allocation epoch",
          );
          const allocation_bytes = require_allocation_epoch(allocation);
          if (!equivalent(allocation.next, bytes.length)) {
            (() => {
              throw new Error("CSE1 Opened event has an invalid shape");
            })();
          }
          return {
            kind: "opened",
            slot: slot,
            generation: generation,
            sequence: sequence,
            packageId: identity_at(21),
            sessionId: identity_at(53),
            world: identity_at(85),
            allocation: allocation_bytes,
          };
        })()
      : equivalent(tag, 3)
        ? (() => {
            if (!equivalent(bytes.length, 21 + 5 * identity_bytes + 4)) {
              (() => {
                throw new Error("CSE1 Candidate event has an invalid shape");
              })();
            }
            return {
              kind: "candidate",
              slot: slot,
              generation: generation,
              sequence: sequence,
              candidateId: identity_at(53),
              base: identity_at(85),
            };
          })()
        : equivalent(tag, 4)
          ? (() => {
              if (!equivalent(bytes.length, 21 + 5 * identity_bytes + 4)) {
                (() => {
                  throw new Error(
                    "CSE1 issued authority event has an invalid shape",
                  );
                })();
              }
              return {
                kind: "issued",
                slot: slot,
                generation: generation,
                sequence: sequence,
                authorization: identity_at(21),
                packageId: identity_at(53),
                sessionId: identity_at(85),
                base: identity_at(117),
                candidateId: identity_at(149),
              };
            })()
          : equivalent(tag, 5)
            ? (() => {
                const prefix_end = 21 + 7 * identity_bytes + 4;
                const projection_tag = byte_at(bytes, prefix_end);
                const successor = identity_at(53);
                return equivalent(projection_tag, 0)
                  ? (() => {
                      if (!equivalent(bytes.length, prefix_end + 1)) {
                        (() => {
                          throw new Error(
                            "CSE1 Admission event has trailing bytes",
                          );
                        })();
                      }
                      return {
                        kind: "admission",
                        slot: slot,
                        generation: generation,
                        sequence: sequence,
                        predecessor: identity_at(21),
                        successor: successor,
                        admissionId: identity_at(85),
                        judgmentId: identity_at(117),
                        run: identity_at(149),
                        activation: identity_at(181),
                        sessionId: identity_at(213),
                        projection: null,
                      };
                    })()
                  : equivalent(projection_tag, 1)
                    ? (() => {
                        const observation_offset = prefix_end + 1;
                        const term_record = parse_blob(
                          bytes,
                          observation_offset + identity_bytes,
                          cse1_max_bytes,
                          "CSE1 projected Term",
                        );
                        if (!equivalent(term_record.next, bytes.length)) {
                          (() => {
                            throw new Error(
                              "CSE1 Admission projection has trailing bytes",
                            );
                          })();
                        }
                        return {
                          kind: "admission",
                          slot: slot,
                          generation: generation,
                          sequence: sequence,
                          predecessor: identity_at(21),
                          successor: successor,
                          admissionId: identity_at(85),
                          judgmentId: identity_at(117),
                          run: identity_at(149),
                          activation: identity_at(181),
                          sessionId: identity_at(213),
                          projection: {
                            observationId: identity_at(observation_offset),
                            termBytes: term_record.bytes,
                          },
                        };
                      })()
                    : (() => {
                        throw new Error(
                          "CSE1 Admission projection tag is invalid",
                        );
                      })();
              })()
            : equivalent(tag, 6)
              ? (() => {
                  if (!equivalent(bytes.length, 21)) {
                    (() => {
                      throw new Error("CSE1 Disposed event has trailing bytes");
                    })();
                  }
                  return {
                    kind: "disposed",
                    slot: slot,
                    generation: generation,
                    sequence: sequence,
                  };
                })()
              : equivalent(tag, 7)
                ? (() => {
                    if (!equivalent(bytes.length, 25)) {
                      (() => {
                        throw new Error("CSE1 rejection has an invalid shape");
                      })();
                    }
                    return {
                      kind: "rejected",
                      slot: slot,
                      generation: generation,
                      sequence: sequence,
                      reason: little_u32(bytes, 21),
                    };
                  })()
                : equivalent(tag, 15)
                  ? (() => {
                      const diagnostic = parse_blob(
                        bytes,
                        21,
                        cse1_max_bytes,
                        "CSE1 candidate rejection diagnostic",
                      );
                      if (!equivalent(diagnostic.next, bytes.length)) {
                        (() => {
                          throw new Error(
                            "CSE1 candidate rejection has an invalid shape",
                          );
                        })();
                      }
                      return {
                        kind: "candidate-rejected",
                        slot: slot,
                        generation: generation,
                        sequence: sequence,
                        diagnostic: ascii_text(
                          diagnostic.bytes,
                          "CSE1 candidate rejection diagnostic",
                        ),
                      };
                    })()
                : equivalent(tag, 8)
                  ? (() => {
                      if (
                        !equivalent(
                          bytes.length,
                          21 + 6 * identity_bytes + 8 + 4,
                        )
                      ) {
                        (() => {
                          throw new Error(
                            "CSE1 Suspended event has an invalid shape",
                          );
                        })();
                      }
                      return {
                        kind: "suspended",
                        slot: slot,
                        generation: generation,
                        sequence: sequence,
                        step: identity_at(21),
                        continuation: identity_at(53),
                        run: identity_at(85),
                        activation: identity_at(117),
                        before: identity_at(149),
                        after: identity_at(181),
                        remainingBudget: little_safe_u64(bytes, 213),
                        stateRevisionCount: little_u32(bytes, 221),
                      };
                    })()
                  : equivalent(tag, 9)
                    ? (() => {
                        if (
                          !equivalent(
                            bytes.length,
                            21 + 7 * identity_bytes + 8 + 4,
                          )
                        ) {
                          (() => {
                            throw new Error(
                              "CSE1 Resumed event has an invalid shape",
                            );
                          })();
                        }
                        return {
                          kind: "resumed",
                          slot: slot,
                          generation: generation,
                          sequence: sequence,
                          occurrence: identity_at(21),
                          step: identity_at(53),
                          continuation: identity_at(85),
                          run: identity_at(117),
                          activation: identity_at(149),
                          before: identity_at(181),
                          after: identity_at(213),
                          remainingBudget: little_safe_u64(bytes, 245),
                          stateRevisionCount: little_u32(bytes, 253),
                        };
                      })()
                    : equivalent(tag, 10)
                      ? (() => {
                          const action = parse_blob(
                            bytes,
                            369,
                            cse1_max_bytes,
                            "CSE1 effect action",
                          );
                          const resource = parse_blob(
                            bytes,
                            action.next,
                            cse1_max_bytes,
                            "CSE1 effect resource",
                          );
                          const payload = parse_blob(
                            bytes,
                            resource.next,
                            cse1_max_bytes,
                            "CSE1 effect payload",
                          );
                          const count_end = require_range(
                            bytes,
                            payload.next,
                            4,
                            "CSE1 effect StateRevision count",
                          );
                          if (!equivalent(count_end, bytes.length)) {
                            (() => {
                              throw new Error(
                                "CSE1 effect intent has trailing bytes",
                              );
                            })();
                          }
                          return {
                            kind: "effect-intent",
                            slot: slot,
                            generation: generation,
                            sequence: sequence,
                            intentId: identity_at(21),
                            run: identity_at(53),
                            activation: identity_at(85),
                            step: identity_at(117),
                            contractIndex: little_u32(bytes, 149),
                            capability: {
                              snapshot: identity_at(153),
                              local: little_u32(bytes, 185),
                            },
                            scope: {
                              application: {
                                snapshot: identity_at(189),
                                local: little_u32(bytes, 221),
                              },
                              mode: {
                                snapshot: identity_at(225),
                                operator: little_u32(bytes, 257),
                                local: little_u32(bytes, 261),
                              },
                              programRevision: identity_at(265),
                              world: identity_at(297),
                              sessionId: identity_at(329),
                              remainingBudget: little_safe_u64(bytes, 361),
                            },
                            actionBytes: action.bytes,
                            resourceBytes: resource.bytes,
                            payloadBytes: payload.bytes,
                            stateRevisionCount: little_u32(bytes, payload.next),
                          };
                        })()
                      : equivalent(tag, 11)
                        ? (() => {
                            if (!equivalent(bytes.length, 25)) {
                              (() => {
                                throw new Error(
                                  "CSE1 absent effect intent has an invalid shape",
                                );
                              })();
                            }
                            return {
                              kind: "effect-intent-absent",
                              slot: slot,
                              generation: generation,
                              sequence: sequence,
                              stateRevisionCount: little_u32(bytes, 21),
                            };
                          })()
                        : equivalent(tag, 12)
                          ? (() => {
                              if (!equivalent(bytes.length, 89)) {
                                (() => {
                                  throw new Error(
                                    "CSE1 effect authorization has an invalid shape",
                                  );
                                })();
                              }
                              return {
                                kind: "effect-authorization",
                                slot: slot,
                                generation: generation,
                                sequence: sequence,
                                authorizationId: identity_at(21),
                                intentId: identity_at(53),
                                stateRevisionCount: little_u32(bytes, 85),
                              };
                            })()
                          : equivalent(tag, 13)
                            ? (() => {
                                const action = parse_blob(
                                  bytes,
                                  117,
                                  cse1_max_bytes,
                                  "CSE1 attempted action",
                                );
                                const resource = parse_blob(
                                  bytes,
                                  action.next,
                                  cse1_max_bytes,
                                  "CSE1 attempted resource",
                                );
                                const payload = parse_blob(
                                  bytes,
                                  resource.next,
                                  cse1_max_bytes,
                                  "CSE1 attempted payload",
                                );
                                const count_end = require_range(
                                  bytes,
                                  payload.next,
                                  4,
                                  "CSE1 attempt StateRevision count",
                                );
                                if (!equivalent(count_end, bytes.length)) {
                                  (() => {
                                    throw new Error(
                                      "CSE1 effect attempt has trailing bytes",
                                    );
                                  })();
                                }
                                return {
                                  kind: "effect-attempt",
                                  slot: slot,
                                  generation: generation,
                                  sequence: sequence,
                                  attemptId: identity_at(21),
                                  intentId: identity_at(53),
                                  authorizationId: identity_at(85),
                                  actionBytes: action.bytes,
                                  resourceBytes: resource.bytes,
                                  payloadBytes: payload.bytes,
                                  stateRevisionCount: little_u32(
                                    bytes,
                                    payload.next,
                                  ),
                                };
                              })()
                            : equivalent(tag, 14)
                              ? (() => {
                                  const receipt_tag = byte_at(bytes, 85);
                                  const receipt_offset = 86;
                                  const receipt_end = equivalent(receipt_tag, 0)
                                    ? receipt_offset
                                    : equivalent(receipt_tag, 1)
                                      ? require_range(
                                          bytes,
                                          receipt_offset,
                                          identity_bytes,
                                          "CSE1 effect receipt identity",
                                        )
                                      : (() => {
                                          throw new Error(
                                            "CSE1 effect receipt tag is invalid",
                                          );
                                        })();
                                  const observation_tag = byte_at(
                                    bytes,
                                    receipt_end,
                                  );
                                  const observation_offset = receipt_end + 1;
                                  const observation_end = equivalent(
                                    observation_tag,
                                    0,
                                  )
                                    ? observation_offset
                                    : equivalent(observation_tag, 1)
                                      ? require_range(
                                          bytes,
                                          observation_offset,
                                          identity_bytes,
                                          "CSE1 effect Observation identity",
                                        )
                                      : (() => {
                                          throw new Error(
                                            "CSE1 effect Observation tag is invalid",
                                          );
                                        })();
                                  const judgment_offset = observation_end;
                                  const judgment_end = require_range(
                                    bytes,
                                    judgment_offset,
                                    identity_bytes,
                                    "CSE1 effect Judgment identity",
                                  );
                                  const disposition_offset = judgment_end;
                                  const count_offset = require_range(
                                    bytes,
                                    disposition_offset,
                                    1,
                                    "CSE1 effect disposition",
                                  );
                                  const end = require_range(
                                    bytes,
                                    count_offset,
                                    4,
                                    "CSE1 effect StateRevision count",
                                  );
                                  const disposition = byte_at(
                                    bytes,
                                    disposition_offset,
                                  );
                                  if (
                                    !equivalent(end, bytes.length) ||
                                    (!equivalent(disposition, 0) &&
                                      !equivalent(disposition, 1))
                                  ) {
                                    (() => {
                                      throw new Error(
                                        "CSE1 effect settlement has an invalid shape",
                                      );
                                    })();
                                  }
                                  return {
                                    kind: "effect-settlement",
                                    slot: slot,
                                    generation: generation,
                                    sequence: sequence,
                                    intentId: identity_at(21),
                                    attemptId: identity_at(53),
                                    receiptId: equivalent(receipt_tag, 1)
                                      ? identity_at(receipt_offset)
                                      : null,
                                    observationId: equivalent(
                                      observation_tag,
                                      1,
                                    )
                                      ? identity_at(observation_offset)
                                      : null,
                                    judgmentId: identity_at(judgment_offset),
                                    disposition: equivalent(disposition, 0)
                                      ? "receipt-observed"
                                      : "no-receipt",
                                    stateRevisionCount: little_u32(
                                      bytes,
                                      count_offset,
                                    ),
                                  };
                                })()
                              : {
                                  kind: "input",
                                  slot: slot,
                                  generation: generation,
                                  sequence: sequence,
                                };
  } else {
    return (() => {
      throw new Error("CSE1 event must carry bounded exact bytes");
    })();
  }
}

function encode_session_command_bang(
  session: WasmSession,
  tag: number,
  payload: ExactBytes | null,
): ExactBytes {
  const bytes: number[] = [67, 87, 73, 49];
  append_u32_bang(bytes, session.handle.slot);
  append_u32_bang(bytes, session.handle.generation);
  append_u64_bang(bytes, session.sequence.value);
  bytes.push(tag);
  if (!(payload == null)) {
    payload.forEach((byte: number) => {
      bytes.push(byte);
    });
  }
  return Object.freeze(bytes);
}

function blob_command_bang(
  session: WasmSession,
  tag: number,
  payload: ExactBytes,
): ExactBytes {
  const blob: number[] = [];
  append_blob_bang(blob, payload);
  return encode_session_command_bang(session, tag, blob);
}

function ascii_bytes(value: unknown, label: string): ExactBytes {
  if (typeof value === "string") {
    const length = value.length;
    if (equivalent(length, 0) || length > 64) {
      (() => {
        throw new Error(concatenate(label, " is outside its byte bound"));
      })();
    }
    return (() => {
      let index = 0;
      let bytes: number[] = [];
      while (true) {
        if (index === length) {
          return Object.freeze(bytes);
        } else {
          const code = value.charCodeAt(index);
          if (code < 33 || code > 126) {
            return (() => {
              throw new Error(concatenate(label, " must be printable ASCII"));
            })();
          } else {
            const _recur_0 = index + 1;
            const _recur_1 = appendValue(bytes, code);
            index = _recur_0;
            bytes = _recur_1;
            continue;
          }
        }
      }
    })();
  } else {
    return (() => {
      throw new Error(concatenate(label, " must be text"));
    })();
  }
}

function decode_physical_observation(
  observation: workbench.InputObservation,
): PhysicalObservation {
  const envelope = observation.value;
  const encoded = envelope[0];
  if (envelope.length !== 1 || typeof encoded !== "string") {
    throw new Error("input observation lacks its exact physical envelope");
  }

  const value: unknown = JSON.parse(encoded);
  if (!isRecord(value)) return null;

  if (value.kind === "keyboard") {
    const phase_tag =
      value.phase === "down" ? 0 : value.phase === "up" ? 1 : -1;
    if (phase_tag < 0 || typeof value.repeat !== "boolean") {
      throw new Error("keyboard observation has an invalid phase");
    }
    return {
      kind: "input",
      source: {
        kind: "keyboard",
        sequence: observation.sequence,
        code: ascii_bytes(value.code, "keyboard code"),
        phase: phase_tag,
      },
    };
  }

  if (value.kind === "scalar-input") {
    if (typeof value.value !== "number" || !Number.isFinite(value.value)) {
      throw new Error("scalar input observation has a non-finite value");
    }
    return {
      kind: "input",
      source: {
        kind: "scalar",
        sequence: observation.sequence,
        channel: ascii_bytes(value.channel, "scalar input channel"),
        value: value.value,
      },
    };
  }

  if (value.kind === "referent-input") {
    if (typeof value.generation !== "number" || !Number.isInteger(value.generation) || value.generation < 1 || value.generation > 0xffffffff) throw new Error("referent input requires its captured generation");
    return { kind: "input", source: { kind: "referent", sequence: observation.sequence,
      generation: value.generation, channel: ascii_bytes(value.channel, "referent input channel"), value: checked_referent(value.value) } };
  }

  if (value.kind === "process-occurrence") {
    if (
      typeof value.ordinal !== "number" ||
      !Number.isSafeInteger(value.ordinal) ||
      value.ordinal < 0
    ) {
      throw new Error("process occurrence ordinal is invalid");
    }
    return { kind: "candidate", ordinal: value.ordinal };
  }

  return null;
}

function physical_input_command_bang(
  session: WasmSession,
  input: Extract<PhysicalObservation, { kind: "input" }>["source"],
): ExactBytes {
  const payload: number[] = [];
  append_u64_bang(payload, input.sequence);
  if (input.kind === "keyboard") {
    payload.push(0);
    append_blob_bang(payload, input.code);
    payload.push(input.phase);
    payload.push(0);
  } else if (input.kind === "scalar") {
    payload.push(1);
    append_blob_bang(payload, input.channel);
    payload.push(1);
    const encoded = new DataView(new ArrayBuffer(8));
    encoded.setFloat64(0, input.value, true);
    for (let index = 0; index < 8; index += 1) {
      payload.push(encoded.getUint8(index));
    }
  } else {
    if (input.generation !== session.sourceGeneration) throw new Error("referent input belongs to a stale generation");
    payload.push(2);
    append_blob_bang(payload, input.channel);
    payload.push(2);
    append_u32_bang(payload, input.value.domain);
    const identity = input.value.identity;
    if (identity.kind === "declared") {
      payload.push(0);
      append_u32_bang(payload, identity.value);
    } else {
      payload.push(1);
      payload.push(...identity.value);
    }
  }
  return encode_session_command_bang(session, 3, payload);
}

function tick_candidate_command_bang(
  session: WasmSession,
  fixed_tick: workbench.FixedTick,
  configuration: workbench.InputConfiguration,
): ExactBytes {
  const milliseconds = fixed_tick.milliseconds;
  const payload: number[] = [];
  if (
    !((_truthy) => _truthy !== false && _truthy != null)(
      Number.isSafeInteger(milliseconds),
    ) ||
    milliseconds <= 0 ||
    milliseconds > 4294967295
  ) {
    (() => {
      throw new Error("fixed tick is outside the CWI1 bound");
    })();
  }
  append_u64_bang(payload, configuration.revision);
  append_u32_bang(payload, milliseconds);
  return encode_session_command_bang(session, 4, payload);
}

function occurrence_candidate_command_bang(
  session: WasmSession,
  ordinal: number,
): ExactBytes {
  const occurrences = session.occurrences;
  if (ordinal >= occurrences.length) {
    (() => {
      throw new Error("process occurrence ordinal is outside the cartridge");
    })();
  }
  return blob_command_bang(session, 2, occurrences[ordinal]);
}

function occurrence_input_command_bang(
  session: WasmSession,
  ordinal: number,
): ExactBytes {
  const occurrences = session.occurrences;
  if (ordinal < 0 || ordinal >= occurrences.length) {
    (() => {
      throw new Error("process occurrence ordinal is outside the cartridge");
    })();
  }
  return blob_command_bang(session, 1, occurrences[ordinal]);
}

function suspend_command_bang(session: WasmSession): ExactBytes {
  return encode_session_command_bang(session, 8, null);
}

function resume_command_bang(session: WasmSession): ExactBytes {
  return encode_session_command_bang(session, 9, null);
}

function admission_scope_bytes_bang(
  session: WasmSession,
  candidate: WasmCandidate,
): number[] {
  const payload: number[] = [];
  [
    session.packageId,
    session.sessionId,
    candidate.base,
    candidate.candidateId,
  ].forEach((identity: ExactBytes) => {
    identity.forEach((byte: number) => {
      payload.push(byte);
    });
  });
  return payload;
}

function apply_session_command_bang(
  module: unknown,
  session: WasmSession,
  command: ExactBytes,
): Cse1Event {
  const event = decode_cse1_event(
    dispatch_session_request(module, command, "command"),
  );
  const current_sequence = session.sequence.value;
  if (
    !equivalent(event.slot, session.handle.slot) ||
    !equivalent(event.generation, session.handle.generation) ||
    !equivalent(event.sequence, current_sequence + 1)
  ) {
    (() => {
      throw new Error("CSE1 event does not advance the exact live session");
    })();
  }
  (() => {
    const _a = session.sequence,
      _v = event.sequence;
    const _old = _a.value;
    _a.value = _v;
    for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
    return _v;
  })();
  if (event.kind === "rejected") {
    (() => {
      throw new Error(
        concatenate(
          "persistent Clause operation rejected with reason ",
          event.reason,
        ),
      );
    })();
  }
  if (event.kind === "candidate-rejected") {
    (() => {
      throw new Error(event.diagnostic);
    })();
  }
  return event;
}

function parse_canonical_blob(
  bytes: CanonicalBytes,
  offset: number,
  label: string,
): Readonly<{ bytes: CanonicalBytes; next: number }> {
  const header_end = require_range(bytes, offset, 4, label);
  const length = big_u32(bytes, offset);
  const end = require_range(bytes, header_end, length, label);
  return { bytes: canonical_byte_range(bytes, header_end, end), next: end };
}

function ascii_text(bytes: CanonicalBytes, label: string): string {
  if (equivalent(bytes.length, 0) || bytes.length > 128) {
    (() => {
      throw new Error(concatenate(label, " is outside its text bound"));
    })();
  }
  let result = "";
  for (let index = 0; index < bytes.length; index += 1) {
    const byte = byte_at(bytes, index);
    if (byte < 32 || byte > 126)
      throw new Error(concatenate(label, " is not canonical ASCII"));
    result += String.fromCharCode(byte);
  }
  return result;
}

function utf8_text(bytes: CanonicalBytes, label: string): string {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      Uint8Array.from({ length: bytes.length }, (_, index) =>
        byte_at(bytes, index),
      ),
    );
  } catch {
    throw new Error(concatenate(label, " is not canonical UTF-8"));
  }
}

function decode_term_node(
  bytes: CanonicalBytes,
  offset: number,
  depth: number,
): DecodedTermNode {
  if (depth > cse1_projected_term_max_depth) {
    (() => {
      throw new Error("projected Term exceeds its depth bound");
    })();
  }
  const tag_end = require_range(bytes, offset, 1, "projected Term node");
  const tag = byte_at(bytes, offset);
  return tag === 0
    ? (() => {
        const kind = parse_canonical_blob(
          bytes,
          tag_end,
          "projected Atom kind",
        );
        const payload = parse_canonical_blob(
          bytes,
          kind.next,
          "projected Atom payload",
        );
        const equality_end = require_range(
          bytes,
          payload.next,
          1,
          "projected Atom equality",
        );
        if (!equivalent(byte_at(bytes, payload.next), 0)) {
          (() => {
            throw new Error("projected Atom equality contract is invalid");
          })();
        }
        return {
          node: { kind: "atom", atomKind: kind.bytes, payload: payload.bytes },
          next: equality_end,
        };
      })()
    : tag === 1
      ? (() => {
          const left = decode_term_node(bytes, tag_end, depth + 1);
          const operator = decode_term_node(bytes, left.next, depth + 1);
          const right = decode_term_node(bytes, operator.next, depth + 1);
          return {
            node: {
              kind: "triple",
              slots: [left.node, operator.node, right.node],
            },
            next: right.next,
          };
        })()
      : (() => {
          throw new Error("projected Term node tag is invalid");
        })();
}

function decode_canonical_term(bytes: unknown): TermNode {
  const envelope_source = workbench["workbench-byte-envelope-source"](bytes);
  const source = envelope_source === null ? bytes : envelope_source;
  if (
    (typeof source === "string" && source.length <= cse1_max_bytes) ||
    exact_byte_array_p(source, cse1_max_bytes)
  ) {
    const node_start = require_range(
      source,
      0,
      2 * identity_bytes,
      "projected Term scope",
    );
    const result = decode_term_node(source, node_start, 0);
    if (!equivalent(result.next, source.length)) {
      (() => {
        throw new Error("projected Term has trailing bytes");
      })();
    }
    return result.node;
  } else {
    return (() => {
      throw new Error("projected Term bytes are outside the CSE1 bound");
    })();
  }
}

function atom_kind_text(node: TermNode): string {
  if (node.kind !== "atom")
    throw new Error("projected realization expected an Atom");
  return ascii_text(node.atomKind, "projected Atom kind");
}

function projected_number(payload: CanonicalBytes): number {
  return equivalent(payload.length, 8)
    ? finite_f64(payload, 0)
    : (() => {
        throw new Error("projected F64 payload is invalid");
      })();
}

function realize_object(
  realize_node: (node: TermNode) => ProjectedValue,
  first: TermNode,
): Readonly<Record<string, ProjectedValue>> {
  let node = first;
  const fields: Array<readonly [string, ProjectedValue]> = [];
  const keys = new Set<string>();
  while (node.kind === "triple") {
    const [field, value, rest] = node.slots;
    if (
      field.kind !== "atom" ||
      atom_kind_text(field) !== "clause/js-field-v1"
    ) {
      throw new Error("projected object entry lacks a field Atom");
    }
    const key = ascii_text(field.payload, "projected field");
    if (
      key === "__proto__" ||
      key === "prototype" ||
      key === "constructor" ||
      keys.has(key)
    ) {
      throw new Error("projected object field is unsafe or duplicated");
    }
    keys.add(key);
    fields.push([key, realize_node(value)]);
    node = rest;
  }
  if (atom_kind_text(node) !== "clause/js-object-end-v1") {
    throw new Error("projected object has an invalid terminator");
  }
  return Object.freeze(Object.fromEntries(fields));
}

function realize_array(
  realize_node: (node: TermNode) => ProjectedValue,
  first: TermNode,
): readonly ProjectedValue[] {
  let node = first;
  const values: ProjectedValue[] = [];
  while (node.kind === "triple") {
    const [item, value, rest] = node.slots;
    if (item.kind !== "atom" || atom_kind_text(item) !== "clause/js-item-v1") {
      throw new Error("projected array entry lacks an item Atom");
    }
    values.push(realize_node(value));
    node = rest;
  }
  if (atom_kind_text(node) !== "clause/js-array-end-v1") {
    throw new Error("projected array has an invalid terminator");
  }
  return Object.freeze(values);
}

function realize_projection_node(node: TermNode): ProjectedValue {
  if (node.kind === "atom") {
    const kind = atom_kind_text(node);
    const payload = node.payload;
    if (kind === "clause/process-projected-f64-v1")
      return projected_number(payload);
    if (kind === "clause/process-projected-bool-v1") {
      const value = byte_at(payload, 0);
      if (payload.length !== 1 || value === undefined || value > 1) {
        throw new Error("projected Boolean payload is invalid");
      }
      return value === 1;
    }
    if (kind === "clause/process-projected-symbol-v1")
      return ascii_text(payload, "projected symbol");
    if (kind === "clause/process-projected-text-v1")
      return utf8_text(payload, "projected Text");
    if (kind === "clause/process-projected-referent-v1") {
      const bytes = Array.from({ length: payload.length }, (_, index) => byte_at(payload, index));
      if (payload.length === 9 && byte_at(payload, 4) === 0) return checked_referent({ kind: "referent", domain: little_u32(bytes, 0), identity: { kind: "declared", value: little_u32(bytes, 5) } });
      if (payload.length === 37 && byte_at(payload, 4) === 1) return checked_referent({ kind: "referent", domain: little_u32(bytes, 0), identity: { kind: "created", value: frozen_byte_range(bytes, 5, 32) } });
      throw new Error("projected referent is malformed");
    }
    throw new Error("projected scalar Atom is not realizable");
  } else {
    const head = node.slots[0];
    const kind = atom_kind_text(head);
    if (kind === "clause/js-field-v1")
      return realize_object(realize_projection_node, node);
    if (kind === "clause/js-item-v1")
      return realize_array(realize_projection_node, node);
    throw new Error("projected Term lacks a realizable shape");
  }
}

function decode_projected_term_frame(bytes: unknown): ProjectedValue {
  return realize_projection_node(decode_canonical_term(bytes));
}

function is_wasm_session(value: unknown): value is WasmSession {
  return (
    typeof value === "object" &&
    value !== null &&
    "handle" in value &&
    typeof value.handle === "object" &&
    value.handle !== null &&
    "sequence" in value &&
    typeof value.sequence === "object" &&
    value.sequence !== null &&
    "disposed" in value &&
    typeof value.disposed === "object" &&
    value.disposed !== null
  );
}

function require_session(value: unknown): WasmSession {
  if (!is_wasm_session(value)) throw new Error("Wasm session is invalid");
  return value;
}

function require_live_session(value: unknown): WasmSession {
  const session = require_session(value);
  return ((_truthy) => _truthy !== false && _truthy != null)(
    session.disposed.value,
  )
    ? (() => {
        throw new Error("Wasm session is disposed");
      })()
    : session;
}

function require_candidate(value: unknown): WasmCandidate {
  if (
    typeof value !== "object" ||
    value === null ||
    !("candidateId" in value) ||
    !("base" in value) ||
    !exact_byte_array_p(value.candidateId, identity_bytes) ||
    !exact_byte_array_p(value.base, identity_bytes)
  ) {
    throw new Error("Wasm candidate is invalid");
  }
  return WasmCandidate(value.candidateId, value.base);
}

function require_identity(value: unknown, label: string): ExactBytes {
  return exact_byte_array_p(value, identity_bytes) &&
    equivalent(value.length, identity_bytes)
    ? value
    : (() => {
        throw new Error(
          concatenate(label, " must carry one exact Clause identity"),
        );
      })();
}

function advance_session_occurrence_bang(
  module: unknown,
  incoming_session: unknown,
  ordinal: number,
): Extract<Cse1Event, { kind: "input" }> {
  const session = require_live_session(incoming_session);
  const event = apply_session_command_bang(
    module,
    session,
    occurrence_input_command_bang(session, ordinal),
  );
  if (event.kind !== "input") {
    (() => {
      throw new Error("CWI1 process occurrence did not produce InputAccepted");
    })();
  }
  return event;
}

function suspend_session_bang(
  module: unknown,
  incoming_session: unknown,
): Extract<Cse1Event, { kind: "suspended" }> {
  const session = require_live_session(incoming_session);
  const event = apply_session_command_bang(
    module,
    session,
    suspend_command_bang(session),
  );
  if (event.kind !== "suspended") {
    (() => {
      throw new Error("CWI1 suspension did not produce Suspended");
    })();
  }
  return event;
}

function resume_session_bang(
  module: unknown,
  incoming_session: unknown,
): Extract<Cse1Event, { kind: "resumed" }> {
  const session = require_live_session(incoming_session);
  const event = apply_session_command_bang(
    module,
    session,
    resume_command_bang(session),
  );
  if (event.kind !== "resumed") {
    (() => {
      throw new Error("CWI1 resumption did not produce Resumed");
    })();
  }
  return event;
}

function query_pending_effect_intent_bang(
  module: unknown,
  incoming_session: unknown,
): Extract<Cse1Event, { kind: "effect-intent" | "effect-intent-absent" }> {
  const session = require_live_session(incoming_session);
  const event = apply_session_command_bang(
    module,
    session,
    encode_session_command_bang(session, 10, null),
  );
  if (event.kind !== "effect-intent" && event.kind !== "effect-intent-absent") {
    (() => {
      throw new Error("CWI1 effect query produced an invalid event");
    })();
  }
  return event;
}

function emit_effect_intent_bang(
  module: unknown,
  incoming_session: unknown,
): Extract<Cse1Event, { kind: "effect-intent" }> {
  const session = require_live_session(incoming_session);
  const event = apply_session_command_bang(
    module,
    session,
    encode_session_command_bang(session, 11, null),
  );
  if (event.kind !== "effect-intent") {
    (() => {
      throw new Error("CWI1 effect emission did not produce an exact intent");
    })();
  }
  return event;
}

function issue_effect_authorization_bang(
  module: unknown,
  incoming_session: unknown,
  intent_id: unknown,
): Extract<Cse1Event, { kind: "effect-authorization" }> {
  const session = require_live_session(incoming_session);
  const intent = require_identity(intent_id, "effect intent");
  const event = apply_session_command_bang(
    module,
    session,
    encode_session_command_bang(session, 12, intent),
  );
  if (event.kind !== "effect-authorization") {
    (() => {
      throw new Error("CWI1 effect issuance did not produce exact authority");
    })();
  }
  return event;
}

function begin_effect_attempt_bang(
  module: unknown,
  incoming_session: unknown,
  authorization_id: unknown,
): Extract<Cse1Event, { kind: "effect-attempt" }> {
  const session = require_live_session(incoming_session);
  const authorization = require_identity(
    authorization_id,
    "effect authorization",
  );
  const event = apply_session_command_bang(
    module,
    session,
    encode_session_command_bang(session, 13, authorization),
  );
  if (event.kind !== "effect-attempt") {
    (() => {
      throw new Error("CWI1 effect attempt did not reach its boundary");
    })();
  }
  return event;
}

function settle_effect_attempt_bang(
  module: unknown,
  incoming_session: unknown,
  attempt_id: unknown,
  status: unknown,
  exact_receipt: unknown,
): Extract<Cse1Event, { kind: "effect-settlement" }> {
  const session = require_live_session(incoming_session);
  const attempt = require_identity(attempt_id, "effect attempt");
  const payload: number[] = [];
  attempt.forEach((byte: number) => {
    payload.push(byte);
  });
  if (status == null && exact_receipt == null) {
    payload.push(0);
  } else if (
    typeof status !== "number" ||
    !Number.isSafeInteger(status) ||
    status < 0 ||
    status > 4294967295 ||
    !exact_byte_array_p(exact_receipt, session_command_max_bytes)
  ) {
    (() => {
      throw new Error(
        "effect receipt must be absent or exact bounded status and bytes",
      );
    })();
  } else {
    payload.push(1);
    append_u32_bang(payload, status);
    append_blob_bang(payload, exact_receipt);
  }
  const event = apply_session_command_bang(
    module,
    session,
    encode_session_command_bang(session, 14, payload),
  );
  if (event.kind !== "effect-settlement") {
    (() => {
      throw new Error("CWI1 effect settlement did not produce a Judgment");
    })();
  }
  return event;
}

function admit_session_candidate_bang(
  module: unknown,
  incoming_session: unknown,
  incoming_candidate: unknown,
): Extract<Cse1Event, { kind: "admission" }> {
  const session = require_live_session(incoming_session);
  const candidate = require_candidate(incoming_candidate);
  const scope = admission_scope_bytes_bang(session, candidate);
  const issued = apply_session_command_bang(
    module,
    session,
    encode_session_command_bang(session, 5, scope),
  );
  if (issued.kind !== "issued") {
    (() => {
      throw new Error(
        "CWI1 issuance did not produce exact Admission authority",
      );
    })();
  }
  const payload = admission_scope_bytes_bang(session, candidate);
  issued.authorization.forEach((byte: number) => {
    payload.push(byte);
  });
  const event = apply_session_command_bang(
    module,
    session,
    encode_session_command_bang(session, 6, payload),
  );
  if (event.kind !== "admission") {
    (() => {
      throw new Error("CWI1 Admission did not produce AdmissionAccepted");
    })();
  }
  (() => {
    const _a = session.world,
      _v = event.successor;
    const _old = _a.value;
    _a.value = _v;
    for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
    return _v;
  })();
  return event;
}

function reject_reason(error: unknown): string {
  if (!(error instanceof Error)) return "Wasm cartridge boundary rejected";
  return error.stack === undefined ? error.message : error.stack;
}

function reclaim_retired_session_bang(module: unknown): null {
  const api = session_module_functions(module);
  if (((_truthy) => _truthy !== false && _truthy != null)(api.reclaim())) {
    setTimeout(() => reclaim_retired_session_bang(module), 0);
  }
  return null;
}

function create_wasm_cartridge_port_bang(
  module: unknown,
  policy: workbench.WorkbenchPolicy,
): workbench.CartridgePort {
  const active_session: Cell<WasmSession | null> = { value: null, watches: {} };
  return workbench["->CartridgePort"](
    (package_candidate, complete) =>
      (() => {
        try {
          return complete(
            workbench["->PackageAccepted"](
              parse_persistent_cartridge_bang(package_candidate),
            ),
          );
        } catch (_catch_0) {
          switch (classifyError(_catch_0)) {
            case 0: {
              const error = _catch_0;
              return complete(
                workbench["->PackageRejected"](reject_reason(error)),
              );
              break;
            }
          }
        }
      })(),
    (accepted_package, generation, complete) =>
      (() => {
        try {
          const cartridge = require_persistent_cartridge(accepted_package);
          if (!Number.isSafeInteger(generation) || generation < 1) throw new Error("source generation is invalid");
          const event = decode_cse1_event(
            dispatch_session_request(module, cartridge.openBytes, "open"),
          );
          if (event.kind !== "opened" || event.sequence !== 0) {
            throw new Error("persistent session did not open exactly once");
          }
          const session = WasmSession(
            Object.freeze({ slot: event.slot, generation: event.generation }),
            generation,
            event.packageId,
            event.sessionId,
            event.allocation,
            { value: event.world, watches: {} },
            { value: 0, watches: {} },
            cartridge.occurrences,
            { value: false, watches: {} },
          );
          const bootstrap_frame = workbench["create-workbench-envelope"](
            policy,
            "[]",
          );
          const prior = active_session.value;
          if (!(prior == null)) {
            (() => {
              const _a = prior.disposed,
                _v = true;
              const _old = _a.value;
              _a.value = _v;
              for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
              return _v;
            })();
          }
          (() => {
            const _a = active_session,
              _v = session;
            const _old = _a.value;
            _a.value = _v;
            for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
            return _v;
          })();
          complete(
            workbench["->SessionStarted"](
              session,
              event.world,
              bootstrap_frame,
            ),
          );
          return setTimeout(() => reclaim_retired_session_bang(module), 0);
        } catch (_catch_1) {
          switch (classifyError(_catch_1)) {
            case 0: {
              const error = _catch_1;
              return complete(
                workbench["->SessionFailed"](reject_reason(error)),
              );
              break;
            }
          }
        }
      })(),
    (incoming_session, fixed_tick, configuration, complete) =>
      (() => {
        try {
          const session = require_live_session(incoming_session);
          const candidate_ordinal: Cell<number | null> = {
            value: null,
            watches: {},
          };
          (() => {
            configuration.observations.forEach((observation) => {
              const decoded = decode_physical_observation(observation);
              if (!(decoded == null)) {
                if (decoded.kind === "candidate") {
                  if (!(candidate_ordinal.value == null)) {
                    (() => {
                      throw new Error(
                        "one configuration may select only one process occurrence",
                      );
                    })();
                  }
                  (() => {
                    const _a = candidate_ordinal,
                      _v = decoded.ordinal;
                    const _old = _a.value;
                    _a.value = _v;
                    for (const _k in _a.watches)
                      _a.watches[_k](_k, _a, _old, _v);
                    return _v;
                  })();
                } else {
                  const event = apply_session_command_bang(
                    module,
                    session,
                    physical_input_command_bang(session, decoded.source),
                  );
                  if (event.kind !== "input") {
                    (() => {
                      throw new Error(
                        "CWI1 physical input did not produce InputAccepted",
                      );
                    })();
                  }
                }
              }
            });
          })();
          const event = apply_session_command_bang(
            module,
            session,
            candidate_ordinal.value == null
              ? tick_candidate_command_bang(session, fixed_tick, configuration)
              : occurrence_candidate_command_bang(
                  session,
                  candidate_ordinal.value,
                ),
          );
          if (event.kind !== "candidate") {
            (() => {
              throw new Error(
                "CWI1 process occurrence did not produce CandidateAccepted",
              );
            })();
          }
          return complete(
            workbench["->CandidateProduced"](
              WasmCandidate(event.candidateId, event.base),
            ),
          );
        } catch (_catch_2) {
          switch (classifyError(_catch_2)) {
            case 0: {
              const error = _catch_2;
              return complete(
                workbench["->CandidateFailed"](reject_reason(error)),
              );
              break;
            }
          }
        }
      })(),
    (incoming_session, incoming_candidate, complete) =>
      (() => {
        try {
          const session = require_live_session(incoming_session);
          const candidate = require_candidate(incoming_candidate);
          const event = admit_session_candidate_bang(
            module,
            session,
            candidate,
          );
          const projection = event.projection;
          if (projection == null) {
            (() => {
              throw new Error(
                "Admission produced no package-declared frame Observation",
              );
            })();
          }
          const frame = workbench["create-workbench-byte-envelope"](
            policy,
            exact_bytes_to_binary_text(projection.termBytes),
          );
          return complete(
            workbench["->AdmissionAccepted"](session, event.successor, frame),
          );
        } catch (_catch_3) {
          switch (classifyError(_catch_3)) {
            case 0: {
              const error = _catch_3;
              return complete(
                workbench["->AdmissionRejected"](reject_reason(error)),
              );
              break;
            }
          }
        }
      })(),
    (incoming_session) => {
      const session = require_session(incoming_session);
      return !((_truthy) => _truthy !== false && _truthy != null)(
        session.disposed.value,
      )
        ? (() => {
            const event = apply_session_command_bang(
              module,
              session,
              encode_session_command_bang(session, 7, null),
            );
            if (event.kind !== "disposed") {
              (() => {
                throw new Error("CWI1 disposal did not produce Disposed");
              })();
            }
            (() => {
              const _a = session.disposed,
                _v = true;
              const _old = _a.value;
              _a.value = _v;
              for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
              return _v;
            })();
            if (active_session.value === session) {
              return (() => {
                const _a = active_session,
                  _v = null;
                const _old = _a.value;
                _a.value = _v;
                for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                return _v;
              })();
            }
          })()
        : null;
    },
  );
}

const create_wasm_cartridge_port = create_wasm_cartridge_port_bang;

export { Cwo1Observation as "->Cwo1Observation" };
export { ExactProcessObservation as "->ExactProcessObservation" };
export { ExactProcessRequest as "->ExactProcessRequest" };
export { Cwo1Observation as "Cwo1Observation" };
export { ExactProcessObservation as "ExactProcessObservation" };
export { ExactProcessRequest as "ExactProcessRequest" };
export { admit_session_candidate_bang as "admit-session-candidate!" };
export { advance_session_occurrence_bang as "advance-session-occurrence!" };
export { append_blob_bang as "append-blob!" };
export { append_u32_bang as "append-u32!" };
export { append_u64_bang as "append-u64!" };
export { begin_effect_attempt_bang as "begin-effect-attempt!" };
export { byte_at as "byte-at" };
export { create_wasm_cartridge_port as "create-wasm-cartridge-port" };
export { cse1_projected_term_json_max_source_units as "cse1-projected-term-json-max-source-units" };
export { cse1_projected_term_max_properties as "cse1-projected-term-max-properties" };
export { cwo1observation_observationId as "cwo1observation-observationId" };
export { cwo1observation_stateRevisionId as "cwo1observation-stateRevisionId" };
export { cwo1observation_values as "cwo1observation-values" };
export { decode_cwo1_observation as "decode-cwo1-observation" };
export { decode_cwr1_hex as "decode-cwr1-hex" };
export { decode_projected_term_frame as "decode-projected-term-frame" };
export { emit_effect_intent_bang as "emit-effect-intent!" };
export { exact_byte_array_p as "exact-byte-array?" };
export { exactprocessobservation_bytes as "exactprocessobservation-bytes" };
export { exactprocessrequest_bytes as "exactprocessrequest-bytes" };
export { frozen_byte_range as "frozen-byte-range" };
export { issue_effect_authorization_bang as "issue-effect-authorization!" };
export { little_safe_u64 as "little-safe-u64" };
export { little_u16 as "little-u16" };
export { little_u32 as "little-u32" };
export { parse_blob as "parse-blob" };
export { process_request_occurrences_bang as "process-request-occurrences!" };
export { process_status as "process-status" };
export { query_pending_effect_intent_bang as "query-pending-effect-intent!" };
export { require_range as "require-range" };
export { resume_session_bang as "resume-session!" };
export { settle_effect_attempt_bang as "settle-effect-attempt!" };
export { suspend_session_bang as "suspend-session!" };
