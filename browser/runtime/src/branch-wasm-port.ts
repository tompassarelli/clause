import * as wire from "./wasm-cartridge-port.js";

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

function concatenate(...values: readonly unknown[]): string {
  return values.map(String).join("");
}

const identity_bytes = 32;

const branch_command_max_bytes = 1024 * 1024;

const branch_event_max_bytes = 64 * 1024;

const branch_open_max_bytes = 4 * 1024 * 1024;

type ExactBytes = readonly number[];

interface Cell<T> {
  value: T;
  watches: Record<
    string,
    (key: string, cell: Cell<T>, previous: T, next: T) => void
  >;
}

interface BranchHandle {
  readonly slot: number;
  readonly generation: number;
}

interface BranchPins {
  readonly parentState: ExactBytes;
  readonly programRevision: ExactBytes;
  readonly packageId: ExactBytes;
  readonly applicationSnapshot: ExactBytes;
  readonly applicationLocal: number;
  readonly sessionId: ExactBytes;
  readonly runtimePolicy: ExactBytes;
  readonly rootPolicy: ExactBytes;
  readonly inputEvidence: ExactBytes;
  readonly physicalPlan: ExactBytes;
  readonly budgetUnits: number;
  readonly disconnectTick: number;
}

interface BranchAncestry {
  readonly parentState: ExactBytes;
  readonly run: ExactBytes;
  readonly activation: ExactBytes;
  readonly disconnectStep: ExactBytes;
  readonly suspensionStep: ExactBytes;
  readonly continuation: ExactBytes;
}

interface BranchSuspension {
  readonly step: ExactBytes;
  readonly continuation: ExactBytes;
  readonly run: ExactBytes;
  readonly activation: ExactBytes;
  readonly before: ExactBytes;
  readonly after: ExactBytes;
  readonly remainingBudget: number;
}

interface BranchResumption extends BranchSuspension {
  readonly occurrence: ExactBytes;
}

export interface ProcessCommandEvidenceV1 {
  readonly _tag: "ProcessCommandEvidenceV1";
  readonly occurrence: ExactBytes;
  readonly step: ExactBytes;
  readonly observation: ExactBytes;
}

export interface ReconnectEvidence {
  readonly pins: BranchPins;
  readonly ancestry: BranchAncestry;
  readonly resumption: BranchResumption;
  readonly commandEvidence: readonly ProcessCommandEvidenceV1[];
  readonly candidate: ExactBytes;
  readonly candidateStep: ExactBytes;
  readonly exactBytes: ExactBytes;
}

type CausalIdentityKind = 0 | 1 | 2 | 3 | 4 | 6 | 7 | 8 | 9;

type CausalReference =
  | Readonly<{ kind: CausalIdentityKind; identity: ExactBytes }>
  | Readonly<{
      kind: 5;
      run: ExactBytes;
      activation: ExactBytes;
      step: ExactBytes;
    }>;

interface BranchExplanation {
  readonly pins: BranchPins;
  readonly ancestry: BranchAncestry;
  readonly resumption: BranchResumption;
  readonly branchCommandEvidence: readonly ProcessCommandEvidenceV1[];
  readonly branchCandidate: ExactBytes;
  readonly authoritativeBase: ExactBytes;
  readonly authoritativeRun: ExactBytes;
  readonly authoritativeActivation: ExactBytes;
  readonly authoritativeCommandEvidence: readonly ProcessCommandEvidenceV1[];
  readonly authoritativeCandidate: ExactBytes;
  readonly authorization: ExactBytes;
  readonly judgment: ExactBytes;
  readonly admission: ExactBytes;
  readonly successor: ExactBytes;
  readonly causalRecords: readonly Readonly<{
    occurrence: CausalReference;
    predecessors: readonly CausalReference[];
  }>[];
  readonly exactBytes: ExactBytes;
}

interface BranchEventBase {
  readonly slot: number;
  readonly generation: number;
  readonly sequence: number;
}

type BranchEvent =
  | (BranchEventBase &
      Readonly<{
        kind: "opened";
        pins: BranchPins;
        ancestry: BranchAncestry;
        suspension: BranchSuspension;
      }>)
  | (BranchEventBase &
      Readonly<{
        kind: "authoritative-admission";
        candidate: ExactBytes;
        predecessor: ExactBytes;
        successor: ExactBytes;
        judgment: ExactBytes;
        admission: ExactBytes;
        run: ExactBytes;
        activation: ExactBytes;
      }>)
  | (BranchEventBase &
      Readonly<{ kind: "reconnect-proposed"; evidence: ReconnectEvidence }>)
  | (BranchEventBase &
      Readonly<{
        kind: "reconnect-admission";
        predecessor: ExactBytes;
        successor: ExactBytes;
        branchCandidate: ExactBytes;
        authoritativeCandidate: ExactBytes;
        judgment: ExactBytes;
        admission: ExactBytes;
        projection: null | Readonly<{
          observation: ExactBytes;
          termBytes: ExactBytes;
        }>;
        explanation: BranchExplanation;
      }>)
  | (BranchEventBase &
      Readonly<{ kind: "explanation"; explanation: BranchExplanation }>)
  | (BranchEventBase & Readonly<{ kind: "disposed" }>)
  | (BranchEventBase &
      Readonly<{ kind: "rejected"; reason: number; pin: number | null }>);

export interface WasmProcessBranch {
  readonly _tag: "WasmProcessBranch";
  readonly handle: BranchHandle;
  readonly sequence: Cell<number>;
  readonly disposed: Cell<boolean>;
  readonly opened: Extract<BranchEvent, { kind: "opened" }>;
}

interface BranchWasmModule {
  readonly clause_branch_v1_io_reset: typeof import("#clause-runtime-wasm").clause_branch_v1_io_reset;
  readonly clause_branch_v1_request_push: typeof import("#clause-runtime-wasm").clause_branch_v1_request_push;
  readonly clause_branch_v1_open: typeof import("#clause-runtime-wasm").clause_branch_v1_open;
  readonly clause_branch_v1_command: typeof import("#clause-runtime-wasm").clause_branch_v1_command;
  readonly clause_branch_v1_event_len: typeof import("#clause-runtime-wasm").clause_branch_v1_event_len;
  readonly clause_branch_v1_event_byte: typeof import("#clause-runtime-wasm").clause_branch_v1_event_byte;
}

interface BranchApi {
  readonly reset: () => void;
  readonly push: (byte: number) => number;
  readonly open: () => number;
  readonly command: () => number;
  readonly eventLength: () => number;
  readonly eventByte: (index: number) => number;
}

interface Parsed<T> {
  readonly value: T;
  readonly next: number;
}

interface ParsedValues<T> {
  readonly values: readonly T[];
  readonly next: number;
}

function WasmProcessBranch(
  handle: BranchHandle,
  sequence: Cell<number>,
  disposed: Cell<boolean>,
  opened: Extract<BranchEvent, { kind: "opened" }>,
): WasmProcessBranch {
  return Object.freeze({
    _tag: "WasmProcessBranch",
    handle,
    sequence,
    disposed,
    opened,
  });
}

function wasmprocessbranch_handle(r: WasmProcessBranch): BranchHandle {
  return r.handle;
}

function wasmprocessbranch_sequence(r: WasmProcessBranch): Cell<number> {
  return r.sequence;
}

function wasmprocessbranch_disposed(r: WasmProcessBranch): Cell<boolean> {
  return r.disposed;
}

function wasmprocessbranch_opened(
  r: WasmProcessBranch,
): Extract<BranchEvent, { kind: "opened" }> {
  return r.opened;
}

function ProcessCommandEvidenceV1(
  occurrence: ExactBytes,
  step: ExactBytes,
  observation: ExactBytes,
): ProcessCommandEvidenceV1 {
  return Object.freeze({
    _tag: "ProcessCommandEvidenceV1",
    occurrence,
    step,
    observation,
  });
}

function processcommandevidencev1_occurrence(
  r: ProcessCommandEvidenceV1,
): ExactBytes {
  return r.occurrence;
}

function processcommandevidencev1_step(
  r: ProcessCommandEvidenceV1,
): ExactBytes {
  return r.step;
}

function processcommandevidencev1_observation(
  r: ProcessCommandEvidenceV1,
): ExactBytes {
  return r.observation;
}

function identity_at(bytes: ExactBytes, offset: number): ExactBytes {
  return wire["frozen-byte-range"](bytes, offset, offset + identity_bytes);
}

function append_u16_bang(bytes: number[], value: number): number {
  bytes.push(value % 256);
  return bytes.push(Math.trunc(value / 256) % 256);
}

function append_occurrences_bang(
  bytes: number[],
  occurrences: readonly ExactBytes[],
): void {
  const count = occurrences.length;
  if (equivalent(count, 0) || count > 256) {
    (() => {
      throw new Error("branch occurrence sequence is outside its bound");
    })();
  }
  append_u16_bang(bytes, count);
  occurrences.forEach((occurrence: ExactBytes) => {
    if (!wire["exact-byte-array?"](occurrence, branch_command_max_bytes)) {
      (() => {
        throw new Error("branch occurrence must carry bounded exact bytes");
      })();
    }
    wire["append-blob!"](bytes, occurrence);
  });
}

function parse_command_evidence(
  bytes: ExactBytes,
  offset: number,
  label: string,
): ParsedValues<ProcessCommandEvidenceV1> {
  const count = wire["little-u16"](bytes, offset);
  const start = wire["require-range"](bytes, offset, 2, label);
  if (equivalent(count, 0) || count > 256) {
    (() => {
      throw new Error(concatenate(label, " count is outside its bound"));
    })();
  }
  let cursor = start;
  const values: ProcessCommandEvidenceV1[] = [];
  for (let index = 0; index < count; index += 1) {
    const record = wire["parse-blob"](
      bytes,
      cursor,
      branch_command_max_bytes,
      label,
    );
    if (record.bytes.length === 0)
      throw new Error(concatenate(label, " occurrence is empty"));
    const identity_start = record.next;
    cursor = wire["require-range"](
      bytes,
      identity_start,
      2 * identity_bytes,
      label,
    );
    values.push(
      ProcessCommandEvidenceV1(
        record.bytes,
        identity_at(bytes, identity_start),
        identity_at(bytes, identity_start + identity_bytes),
      ),
    );
  }
  return { values: Object.freeze(values), next: cursor };
}

function parse_pins(bytes: ExactBytes, offset: number): Parsed<BranchPins> {
  const end = wire["require-range"](bytes, offset, 308, "branch pins");
  return {
    value: {
      parentState: identity_at(bytes, offset),
      programRevision: identity_at(bytes, offset + 32),
      packageId: identity_at(bytes, offset + 64),
      applicationSnapshot: identity_at(bytes, offset + 96),
      applicationLocal: wire["little-u32"](bytes, offset + 128),
      sessionId: identity_at(bytes, offset + 132),
      runtimePolicy: identity_at(bytes, offset + 164),
      rootPolicy: identity_at(bytes, offset + 196),
      inputEvidence: identity_at(bytes, offset + 228),
      physicalPlan: identity_at(bytes, offset + 260),
      budgetUnits: wire["little-safe-u64"](bytes, offset + 292),
      disconnectTick: wire["little-safe-u64"](bytes, offset + 300),
    },
    next: end,
  };
}

function parse_ancestry(
  bytes: ExactBytes,
  offset: number,
): Parsed<BranchAncestry> {
  const end = wire["require-range"](
    bytes,
    offset,
    6 * identity_bytes,
    "branch ancestry",
  );
  return {
    value: {
      parentState: identity_at(bytes, offset),
      run: identity_at(bytes, offset + 32),
      activation: identity_at(bytes, offset + 64),
      disconnectStep: identity_at(bytes, offset + 96),
      suspensionStep: identity_at(bytes, offset + 128),
      continuation: identity_at(bytes, offset + 160),
    },
    next: end,
  };
}

function parse_suspension(
  bytes: ExactBytes,
  offset: number,
): Parsed<BranchSuspension> {
  const end = wire["require-range"](
    bytes,
    offset,
    6 * identity_bytes + 8,
    "branch suspension",
  );
  return {
    value: {
      step: identity_at(bytes, offset),
      continuation: identity_at(bytes, offset + 32),
      run: identity_at(bytes, offset + 64),
      activation: identity_at(bytes, offset + 96),
      before: identity_at(bytes, offset + 128),
      after: identity_at(bytes, offset + 160),
      remainingBudget: wire["little-safe-u64"](bytes, offset + 192),
    },
    next: end,
  };
}

function parse_resumption(
  bytes: ExactBytes,
  offset: number,
): Parsed<BranchResumption> {
  const end = wire["require-range"](
    bytes,
    offset,
    7 * identity_bytes + 8,
    "branch resumption",
  );
  return {
    value: {
      occurrence: identity_at(bytes, offset),
      step: identity_at(bytes, offset + 32),
      continuation: identity_at(bytes, offset + 64),
      run: identity_at(bytes, offset + 96),
      activation: identity_at(bytes, offset + 128),
      before: identity_at(bytes, offset + 160),
      after: identity_at(bytes, offset + 192),
      remainingBudget: wire["little-safe-u64"](bytes, offset + 224),
    },
    next: end,
  };
}

function decode_reconnect_evidence(bytes: unknown): ReconnectEvidence {
  if (wire["exact-byte-array?"](bytes, branch_event_max_bytes)) {
    if (
      bytes.length < 4 ||
      !equivalent(wire["frozen-byte-range"](bytes, 0, 4), [67, 82, 69, 49])
    ) {
      (() => {
        throw new Error("reconnect evidence magic is invalid");
      })();
    }
    const pins = parse_pins(bytes, 4);
    const ancestry = parse_ancestry(bytes, pins.next);
    const resumption = parse_resumption(bytes, ancestry.next);
    const commands = parse_command_evidence(
      bytes,
      resumption.next,
      "reconnect command evidence",
    );
    const candidate_offset = commands.next;
    const final_end = wire["require-range"](
      bytes,
      candidate_offset,
      2 * identity_bytes,
      "reconnect candidate",
    );
    const candidate_step = identity_at(
      bytes,
      candidate_offset + identity_bytes,
    );
    const command_values = commands.values;
    const command_count = command_values.length;
    const last_command = command_values[command_count - 1];
    if (last_command === undefined)
      throw new Error("reconnect command evidence is empty");
    if (!equivalent(final_end, bytes.length)) {
      (() => {
        throw new Error("reconnect evidence has trailing bytes");
      })();
    }
    if (!equivalent(last_command.step, candidate_step)) {
      (() => {
        throw new Error(
          "reconnect candidate Step does not match final command evidence",
        );
      })();
    }
    return {
      pins: pins.value,
      ancestry: ancestry.value,
      resumption: resumption.value,
      commandEvidence: commands.values,
      candidate: identity_at(bytes, candidate_offset),
      candidateStep: candidate_step,
      exactBytes: bytes,
    };
  } else {
    return (() => {
      throw new Error("reconnect evidence must carry bounded exact bytes");
    })();
  }
}

function parse_causal_ref(
  bytes: ExactBytes,
  offset: number,
): Parsed<CausalReference> {
  const tag_end = wire["require-range"](bytes, offset, 1, "causal reference");
  const tag = wire["byte-at"](bytes, offset);
  if (tag === 5) {
    const end = wire["require-range"](
      bytes,
      tag_end,
      3 * identity_bytes,
      "causal Step reference",
    );
    return {
      value: {
        kind: 5,
        run: identity_at(bytes, tag_end),
        activation: identity_at(bytes, tag_end + 32),
        step: identity_at(bytes, tag_end + 64),
      },
      next: end,
    };
  } else {
    if (!is_causal_identity_kind(tag)) {
      (() => {
        throw new Error("causal reference tag is invalid");
      })();
    }
    const end = wire["require-range"](
      bytes,
      tag_end,
      identity_bytes,
      "causal identity reference",
    );
    return {
      value: { kind: tag, identity: identity_at(bytes, tag_end) },
      next: end,
    };
  }
}

function is_causal_identity_kind(value: number): value is CausalIdentityKind {
  return Number.isInteger(value) && value >= 0 && value <= 9 && value !== 5;
}

function parse_causal_predecessors(
  bytes: ExactBytes,
  offset: number,
): ParsedValues<CausalReference> {
  const count = wire["little-u16"](bytes, offset);
  const start = wire["require-range"](
    bytes,
    offset,
    2,
    "causal predecessor count",
  );
  let cursor = start;
  const values: CausalReference[] = [];
  for (let index = 0; index < count; index += 1) {
    const reference = parse_causal_ref(bytes, cursor);
    cursor = reference.next;
    values.push(reference.value);
  }
  return { values: Object.freeze(values), next: cursor };
}

function parse_causal_records(
  bytes: ExactBytes,
  offset: number,
): ParsedValues<
  Readonly<{
    occurrence: CausalReference;
    predecessors: readonly CausalReference[];
  }>
> {
  const count = wire["little-u16"](bytes, offset);
  const start = wire["require-range"](bytes, offset, 2, "causal record count");
  if (count > 2048) {
    (() => {
      throw new Error("causal record count exceeds its bound");
    })();
  }
  let cursor = start;
  const values: Array<
    Readonly<{
      occurrence: CausalReference;
      predecessors: readonly CausalReference[];
    }>
  > = [];
  for (let index = 0; index < count; index += 1) {
    const occurrence = parse_causal_ref(bytes, cursor);
    const predecessors = parse_causal_predecessors(bytes, occurrence.next);
    cursor = predecessors.next;
    values.push({
      occurrence: occurrence.value,
      predecessors: predecessors.values,
    });
  }
  return { values: Object.freeze(values), next: cursor };
}

function decode_branch_explanation(bytes: unknown): BranchExplanation {
  if (wire["exact-byte-array?"](bytes, branch_event_max_bytes)) {
    if (
      bytes.length < 4 ||
      !equivalent(wire["frozen-byte-range"](bytes, 0, 4), [67, 66, 88, 49])
    ) {
      (() => {
        throw new Error("branch explanation magic is invalid");
      })();
    }
    const pins = parse_pins(bytes, 4);
    const ancestry = parse_ancestry(bytes, pins.next);
    const resumption = parse_resumption(bytes, ancestry.next);
    const branch_commands = parse_command_evidence(
      bytes,
      resumption.next,
      "explanation branch command evidence",
    );
    const branch_commands_end = branch_commands.next;
    const authority_prefix = wire["require-range"](
      bytes,
      branch_commands_end,
      4 * identity_bytes,
      "explanation authority prefix",
    );
    const authoritative_commands = parse_command_evidence(
      bytes,
      authority_prefix,
      "explanation authoritative command evidence",
    );
    const authoritative_commands_end = authoritative_commands.next;
    const authority_suffix = wire["require-range"](
      bytes,
      authoritative_commands_end,
      5 * identity_bytes,
      "explanation authority suffix",
    );
    const causal = parse_causal_records(bytes, authority_suffix);
    if (!equivalent(causal.next, bytes.length)) {
      (() => {
        throw new Error("branch explanation has trailing bytes");
      })();
    }
    return {
      pins: pins.value,
      ancestry: ancestry.value,
      resumption: resumption.value,
      branchCommandEvidence: branch_commands.values,
      branchCandidate: identity_at(bytes, branch_commands_end),
      authoritativeBase: identity_at(bytes, branch_commands_end + 32),
      authoritativeRun: identity_at(bytes, branch_commands_end + 64),
      authoritativeActivation: identity_at(bytes, branch_commands_end + 96),
      authoritativeCommandEvidence: authoritative_commands.values,
      authoritativeCandidate: identity_at(bytes, authoritative_commands_end),
      authorization: identity_at(bytes, authoritative_commands_end + 32),
      judgment: identity_at(bytes, authoritative_commands_end + 64),
      admission: identity_at(bytes, authoritative_commands_end + 96),
      successor: identity_at(bytes, authoritative_commands_end + 128),
      causalRecords: causal.values,
      exactBytes: bytes,
    };
  } else {
    return (() => {
      throw new Error("branch explanation must carry bounded exact bytes");
    })();
  }
}

function is_branch_wasm_module(module: unknown): module is BranchWasmModule {
  return (
    typeof module === "object" &&
    module !== null &&
    "clause_branch_v1_io_reset" in module &&
    typeof module.clause_branch_v1_io_reset === "function" &&
    "clause_branch_v1_request_push" in module &&
    typeof module.clause_branch_v1_request_push === "function" &&
    "clause_branch_v1_open" in module &&
    typeof module.clause_branch_v1_open === "function" &&
    "clause_branch_v1_command" in module &&
    typeof module.clause_branch_v1_command === "function" &&
    "clause_branch_v1_event_len" in module &&
    typeof module.clause_branch_v1_event_len === "function" &&
    "clause_branch_v1_event_byte" in module &&
    typeof module.clause_branch_v1_event_byte === "function"
  );
}

function branch_module_functions(module: unknown): BranchApi {
  if (!is_branch_wasm_module(module))
    throw new Error("Wasm module lacks the Clause branch ABI");
  const reset = module.clause_branch_v1_io_reset;
  const push = module.clause_branch_v1_request_push;
  const open = module.clause_branch_v1_open;
  const command = module.clause_branch_v1_command;
  const event_length = module.clause_branch_v1_event_len;
  const event_byte = module.clause_branch_v1_event_byte;
  if (
    reset == null ||
    push == null ||
    open == null ||
    command == null ||
    event_length == null ||
    event_byte == null
  ) {
    (() => {
      throw new Error("Wasm module lacks the Clause branch ABI");
    })();
  }
  return {
    reset: reset,
    push: push,
    open: open,
    command: command,
    eventLength: event_length,
    eventByte: event_byte,
  };
}

function dispatch_branch_request(
  module: unknown,
  request: unknown,
  operation: "open" | "command",
): ExactBytes {
  if (
    wire["exact-byte-array?"](
      request,
      equivalent(operation, "open")
        ? branch_open_max_bytes
        : branch_command_max_bytes,
    )
  ) {
    const api = branch_module_functions(module);
    api.reset();
    request.forEach((byte: number) => {
      const status = wire["process-status"](api.push(byte));
      if (!equivalent(status, 0)) {
        (() => {
          throw new Error(
            concatenate("branch byte transfer rejected with status ", status),
          );
        })();
      }
    });
    const status = wire["process-status"](
      equivalent(operation, "open") ? api.open() : api.command(),
    );
    if (!equivalent(status, 0)) {
      (() => {
        throw new Error(
          concatenate(
            "Clause branch ",
            operation,
            " rejected with status ",
            status,
          ),
        );
      })();
    }
    const length = wire["process-status"](api.eventLength());
    if (length < 21 || length > branch_event_max_bytes) {
      (() => {
        throw new Error("CBE1 event length is out of bounds");
      })();
    }
    const values: number[] = [];
    for (let index = 0; index < length; index += 1) {
      const byte = wire["process-status"](api.eventByte(index));
      if (byte < 0 || byte > 255)
        throw new Error("CBE1 event byte is out of bounds");
      values.push(byte);
    }
    return Object.freeze(values);
  } else {
    return (() => {
      throw new Error("branch request must carry bounded exact bytes");
    })();
  }
}

function parse_projection(
  bytes: ExactBytes,
  offset: number,
): Parsed<null | Readonly<{ observation: ExactBytes; termBytes: ExactBytes }>> {
  const tag_end = wire["require-range"](bytes, offset, 1, "branch projection");
  const tag = wire["byte-at"](bytes, offset);
  return equivalent(tag, 0)
    ? { value: null, next: tag_end }
    : equivalent(tag, 1)
      ? (() => {
          const observation_end = wire["require-range"](
            bytes,
            tag_end,
            identity_bytes,
            "branch projection observation",
          );
          const term = wire["parse-blob"](
            bytes,
            observation_end,
            branch_event_max_bytes,
            "branch projected Term",
          );
          return {
            value: {
              observation: identity_at(bytes, tag_end),
              termBytes: term.bytes,
            },
            next: term.next,
          };
        })()
      : (() => {
          throw new Error("branch projection tag is invalid");
        })();
}

function decode_cbe1_event(bytes: unknown): BranchEvent {
  if (wire["exact-byte-array?"](bytes, branch_event_max_bytes)) {
    if (
      bytes.length < 21 ||
      !equivalent(wire["frozen-byte-range"](bytes, 0, 4), [67, 66, 69, 49])
    ) {
      (() => {
        throw new Error("CBE1 event magic is invalid");
      })();
    }
    const slot = wire["little-u32"](bytes, 4);
    const generation = wire["little-u32"](bytes, 8);
    const sequence = wire["little-safe-u64"](bytes, 12);
    const tag = wire["byte-at"](bytes, 20);
    const common = { slot: slot, generation: generation, sequence: sequence };
    return equivalent(tag, 1)
      ? (() => {
          const pins = parse_pins(bytes, 21);
          const ancestry = parse_ancestry(bytes, pins.next);
          const suspension = parse_suspension(bytes, ancestry.next);
          if (!equivalent(suspension.next, bytes.length)) {
            (() => {
              throw new Error("CBE1 Opened event has trailing bytes");
            })();
          }
          return Object.assign(common, {
            kind: "opened" as const,
            pins: pins.value,
            ancestry: ancestry.value,
            suspension: suspension.value,
          });
        })()
      : equivalent(tag, 2)
        ? (() => {
            if (!equivalent(bytes.length, 21 + 7 * identity_bytes)) {
              (() => {
                throw new Error(
                  "CBE1 authoritative Admission has an invalid shape",
                );
              })();
            }
            return Object.assign(common, {
              kind: "authoritative-admission" as const,
              candidate: identity_at(bytes, 21),
              predecessor: identity_at(bytes, 53),
              successor: identity_at(bytes, 85),
              judgment: identity_at(bytes, 117),
              admission: identity_at(bytes, 149),
              run: identity_at(bytes, 181),
              activation: identity_at(bytes, 213),
            });
          })()
        : equivalent(tag, 3)
          ? (() => {
              const record = wire["parse-blob"](
                bytes,
                21,
                branch_event_max_bytes,
                "CBE1 reconnect evidence",
              );
              if (!equivalent(record.next, bytes.length)) {
                (() => {
                  throw new Error("CBE1 reconnect evidence has trailing bytes");
                })();
              }
              return Object.assign(common, {
                kind: "reconnect-proposed" as const,
                evidence: decode_reconnect_evidence(record.bytes),
              });
            })()
          : equivalent(tag, 4)
            ? (() => {
                const prefix_end = wire["require-range"](
                  bytes,
                  21,
                  6 * identity_bytes,
                  "CBE1 reconnect Admission",
                );
                const projection = parse_projection(bytes, prefix_end);
                const record = wire["parse-blob"](
                  bytes,
                  projection.next,
                  branch_event_max_bytes,
                  "CBE1 branch explanation",
                );
                if (!equivalent(record.next, bytes.length)) {
                  (() => {
                    throw new Error(
                      "CBE1 reconnect Admission has trailing bytes",
                    );
                  })();
                }
                return Object.assign(common, {
                  kind: "reconnect-admission" as const,
                  predecessor: identity_at(bytes, 21),
                  successor: identity_at(bytes, 53),
                  branchCandidate: identity_at(bytes, 85),
                  authoritativeCandidate: identity_at(bytes, 117),
                  judgment: identity_at(bytes, 149),
                  admission: identity_at(bytes, 181),
                  projection: projection.value,
                  explanation: decode_branch_explanation(record.bytes),
                });
              })()
            : equivalent(tag, 5)
              ? (() => {
                  const record = wire["parse-blob"](
                    bytes,
                    21,
                    branch_event_max_bytes,
                    "CBE1 retained explanation",
                  );
                  if (!equivalent(record.next, bytes.length)) {
                    (() => {
                      throw new Error("CBE1 explanation has trailing bytes");
                    })();
                  }
                  return Object.assign(common, {
                    kind: "explanation" as const,
                    explanation: decode_branch_explanation(record.bytes),
                  });
                })()
              : equivalent(tag, 6)
                ? (() => {
                    if (!equivalent(bytes.length, 21)) {
                      (() => {
                        throw new Error(
                          "CBE1 Disposed event has trailing bytes",
                        );
                      })();
                    }
                    return Object.assign(common, {
                      kind: "disposed" as const,
                    });
                  })()
                : equivalent(tag, 7)
                  ? (() => {
                      const reason = wire["byte-at"](bytes, 21);
                      const expected = equivalent(reason, 7) ? 23 : 22;
                      if (!equivalent(bytes.length, expected)) {
                        (() => {
                          throw new Error(
                            "CBE1 rejection has an invalid shape",
                          );
                        })();
                      }
                      return Object.assign(common, {
                        kind: "rejected" as const,
                        reason: reason,
                        pin: equivalent(reason, 7)
                          ? wire["byte-at"](bytes, 22)
                          : null,
                      });
                    })()
                  : (() => {
                      throw new Error("CBE1 event tag is invalid");
                    })();
  } else {
    return (() => {
      throw new Error("CBE1 event must carry bounded exact bytes");
    })();
  }
}

function encode_open(
  request: { readonly bytes: ExactBytes },
  disconnect_tick: number,
  disconnect_occurrence: ExactBytes,
  max_commands: number,
): ExactBytes {
  const bytes = [67, 66, 82, 49];
  wire["append-blob!"](bytes, request.bytes);
  wire["append-u64!"](bytes, disconnect_tick);
  wire["append-blob!"](bytes, disconnect_occurrence);
  wire["append-u64!"](bytes, max_commands);
  return Object.freeze(bytes);
}

function encode_command(
  branch: WasmProcessBranch,
  tag: number,
  payload: ExactBytes | null,
): ExactBytes {
  const bytes = [67, 66, 73, 49];
  wire["append-u32!"](bytes, branch.handle.slot);
  wire["append-u32!"](bytes, branch.handle.generation);
  wire["append-u64!"](bytes, branch.sequence.value);
  bytes.push(tag);
  if (!(payload == null)) {
    payload.forEach((byte: number) => {
      bytes.push(byte);
    });
  }
  return Object.freeze(bytes);
}

function is_process_branch(value: unknown): value is WasmProcessBranch {
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

function require_live_branch(value: unknown): WasmProcessBranch {
  if (!is_process_branch(value))
    throw new Error("Wasm process branch is invalid");
  if (value.disposed.value) throw new Error("Wasm process branch is disposed");
  return value;
}

function apply_command_bang(
  module: unknown,
  branch: WasmProcessBranch,
  command: ExactBytes,
): BranchEvent {
  const event = decode_cbe1_event(
    dispatch_branch_request(module, command, "command"),
  );
  const sequence = branch.sequence.value;
  if (
    !equivalent(event.slot, branch.handle.slot) ||
    !equivalent(event.generation, branch.handle.generation) ||
    !equivalent(event.sequence, sequence + 1)
  ) {
    (() => {
      throw new Error("CBE1 event does not advance exact branch custody");
    })();
  }
  (() => {
    const _a = branch.sequence,
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
          "Clause branch operation rejected with reason ",
          event.reason,
        ),
      );
    })();
  }
  return event;
}

function open_process_branch_bang(
  module: unknown,
  request: { readonly bytes: ExactBytes },
  disconnect_tick: number,
  disconnect_occurrence: ExactBytes,
  max_commands: number,
): WasmProcessBranch {
  if (disconnect_tick < 0 || max_commands <= 0) {
    (() => {
      throw new Error("branch tick and command budget are invalid");
    })();
  }
  const event = decode_cbe1_event(
    dispatch_branch_request(
      module,
      encode_open(
        request,
        disconnect_tick,
        disconnect_occurrence,
        max_commands,
      ),
      "open",
    ),
  );
  if (event.kind !== "opened" || event.sequence !== 0) {
    (() => {
      throw new Error("Clause branch did not open exactly once");
    })();
  }
  return WasmProcessBranch(
    Object.freeze({ slot: event.slot, generation: event.generation }),
    { value: 0, watches: {} },
    { value: false, watches: {} },
    event,
  );
}

function occurrence_command_bang(
  branch: WasmProcessBranch,
  tag: number,
  occurrences: readonly ExactBytes[],
): ExactBytes {
  const payload: number[] = [];
  append_occurrences_bang(payload, occurrences);
  return encode_command(branch, tag, payload);
}

function admit_authoritative_occurrences_bang(
  module: unknown,
  incoming_branch: unknown,
  occurrences: readonly ExactBytes[],
): Extract<BranchEvent, { kind: "authoritative-admission" }> {
  const branch = require_live_branch(incoming_branch);
  const event = apply_command_bang(
    module,
    branch,
    occurrence_command_bang(branch, 1, occurrences),
  );
  if (event.kind !== "authoritative-admission") {
    (() => {
      throw new Error("branch authority advance produced no Admission");
    })();
  }
  return event;
}

function propose_branch_reconnect_bang(
  module: unknown,
  incoming_branch: unknown,
  occurrences: readonly ExactBytes[],
): Extract<BranchEvent, { kind: "reconnect-proposed" }> {
  const branch = require_live_branch(incoming_branch);
  const event = apply_command_bang(
    module,
    branch,
    occurrence_command_bang(branch, 2, occurrences),
  );
  if (event.kind !== "reconnect-proposed") {
    (() => {
      throw new Error("branch continuation produced no reconnect evidence");
    })();
  }
  return event;
}

function adjudicate_branch_reconnect_bang(
  module: unknown,
  incoming_branch: unknown,
  proposal: Extract<BranchEvent, { kind: "reconnect-proposed" }>,
  authoritative_base: ExactBytes,
  occurrences: readonly ExactBytes[],
): Extract<BranchEvent, { kind: "reconnect-admission" }> {
  const branch = require_live_branch(incoming_branch);
  const evidence = proposal.evidence;
  const payload: number[] = [];
  if (
    evidence == null ||
    !wire["exact-byte-array?"](evidence.exactBytes, branch_event_max_bytes)
  ) {
    (() => {
      throw new Error("reconnect proposal lacks exact retained evidence");
    })();
  }
  wire["append-blob!"](payload, evidence.exactBytes);
  [evidence.candidate, authoritative_base].forEach((identity: ExactBytes) => {
    if (!wire["exact-byte-array?"](identity, identity_bytes)) {
      (() => {
        throw new Error("reconnect adjudication identity is invalid");
      })();
    }
    identity.forEach((byte: number) => {
      payload.push(byte);
    });
  });
  append_occurrences_bang(payload, occurrences);
  const event = apply_command_bang(
    module,
    branch,
    encode_command(branch, 3, payload),
  );
  if (event.kind !== "reconnect-admission") {
    (() => {
      throw new Error("reconnect adjudication produced no Admission");
    })();
  }
  return event;
}

function explain_process_branch_bang(
  module: unknown,
  incoming_branch: unknown,
): Extract<BranchEvent, { kind: "explanation" }> {
  const branch = require_live_branch(incoming_branch);
  const event = apply_command_bang(
    module,
    branch,
    encode_command(branch, 4, null),
  );
  if (event.kind !== "explanation") {
    (() => {
      throw new Error("retained branch produced no causal explanation");
    })();
  }
  return event;
}

function dispose_process_branch_bang(
  module: unknown,
  incoming_branch: unknown,
): boolean {
  const branch = require_live_branch(incoming_branch);
  const event = apply_command_bang(
    module,
    branch,
    encode_command(branch, 5, null),
  );
  if (event.kind !== "disposed") {
    (() => {
      throw new Error("Clause branch did not dispose");
    })();
  }
  (() => {
    const _a = branch.disposed,
      _v = true;
    const _old = _a.value;
    _a.value = _v;
    for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
    return _v;
  })();
  return true;
}

export { ProcessCommandEvidenceV1 as "->ProcessCommandEvidenceV1" };
export { WasmProcessBranch as "->WasmProcessBranch" };
export { ProcessCommandEvidenceV1 as "ProcessCommandEvidenceV1" };
export { WasmProcessBranch as "WasmProcessBranch" };
export { adjudicate_branch_reconnect_bang as "adjudicate-branch-reconnect!" };
export { admit_authoritative_occurrences_bang as "admit-authoritative-occurrences!" };
export { decode_branch_explanation as "decode-branch-explanation" };
export { decode_reconnect_evidence as "decode-reconnect-evidence" };
export { dispose_process_branch_bang as "dispose-process-branch!" };
export { explain_process_branch_bang as "explain-process-branch!" };
export { open_process_branch_bang as "open-process-branch!" };
export { processcommandevidencev1_observation as "processcommandevidencev1-observation" };
export { processcommandevidencev1_occurrence as "processcommandevidencev1-occurrence" };
export { processcommandevidencev1_step as "processcommandevidencev1-step" };
export { propose_branch_reconnect_bang as "propose-branch-reconnect!" };
export { wasmprocessbranch_disposed as "wasmprocessbranch-disposed" };
export { wasmprocessbranch_handle as "wasmprocessbranch-handle" };
export { wasmprocessbranch_opened as "wasmprocessbranch-opened" };
export { wasmprocessbranch_sequence as "wasmprocessbranch-sequence" };
