import * as integration from "./wasm-workbench-shell.js";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import * as test from "bun:test";

type ByteSequence = readonly number[] | Uint8Array;
type TermField = readonly [name: string, value: ByteSequence];
type Tick = () => unknown;
type TestArenaInput = Readonly<{
  kind: "keyboard";
  phase: string;
  code: string;
  repeat: boolean;
}>;
type EmitInput = (input: TestArenaInput) => unknown;

interface Cell<T> {
  value: T;
  readonly watches: Record<
    string,
    (key: string, cell: Cell<T>, previous: T, next: T) => void
  >;
}

interface TestSessionModule {
  readonly clause_session_v1_open_bulk: (
    request: Uint8Array<ArrayBuffer>,
  ) => number;
  readonly clause_session_v1_command_bulk: (
    request: Uint8Array<ArrayBuffer>,
  ) => number;
  readonly clause_session_v1_event_bulk: () => Uint8Array<ArrayBuffer>;
  readonly clause_session_v1_reclaim_retired: () => boolean;
}

interface ProjectedVec3 {
  readonly x: number;
  readonly y: number;
  readonly z: number;
}

interface ProjectedArenaFrame {
  readonly player: Readonly<{
    position: ProjectedVec3;
    grounded: boolean;
  }>;
  readonly world: Readonly<{
    platforms: readonly wasm.ProjectedValue[];
  }>;
}

function createCell<T>(value: T): Cell<T> {
  return { value, watches: {} };
}

function setCell<T>(cell: Cell<T>, value: T): T {
  const previous = cell.value;
  cell.value = value;
  for (const [key, watch] of Object.entries(cell.watches)) {
    watch(key, cell, previous, value);
  }
  return value;
}

function concatenate(...values: readonly unknown[]): string {
  return values.map(String).join("");
}

function policy(): workbench.WorkbenchPolicy {
  const maximum = Number.MAX_SAFE_INTEGER;
  return workbench["->WorkbenchPolicy"](
    8,
    8,
    64,
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
  const bytes = new Array<number>(32).fill(0);
  bytes[0] = tag;
  bytes[31] = tag;
  return bytes;
}

function append_little_u32_bang(bytes: number[], value: number): void {
  for (const divisor of [1, 256, 65536, 16777216]) {
    bytes.push(Math.trunc(value / divisor) % 256);
  }
}

function append_little_u64_bang(bytes: number[], value: number): void {
  append_little_u32_bang(bytes, value);
  append_little_u32_bang(bytes, 0);
}

function append_big_u32_bang(bytes: number[], value: number): void {
  for (const divisor of [16777216, 65536, 256, 1]) {
    bytes.push(Math.trunc(value / divisor) % 256);
  }
}

function append_little_blob_bang(
  bytes: number[],
  value: ByteSequence,
): void {
  append_little_u32_bang(bytes, value.length);
  value.forEach((byte) => bytes.push(byte));
}

function allocation_epoch_bang(): number[] {
  const bytes = new Array<number>(304).fill(0);
  bytes.splice(0, 4, 82, 65, 69, 49);
  return bytes;
}

function ascii(source: string): number[] {
  const bytes: number[] = [];
  for (let index = 0; index < source.length; index += 1) {
    bytes.push(source.charCodeAt(index));
  }
  return bytes;
}

function atom_node_bang(kind: string, payload: ByteSequence): number[] {
  const bytes = [0];
  const kind_bytes = ascii(kind);
  append_big_u32_bang(bytes, kind_bytes.length);
  kind_bytes.forEach((byte) => bytes.push(byte));
  append_big_u32_bang(bytes, payload.length);
  payload.forEach((byte) => bytes.push(byte));
  bytes.push(0);
  return bytes;
}

function triple_node_bang(
  left: ByteSequence,
  operator: ByteSequence,
  right: ByteSequence,
): number[] {
  const bytes = [1];
  for (const node of [left, operator, right]) {
    node.forEach((byte) => bytes.push(byte));
  }
  return bytes;
}

function number_node_bang(value: number): number[] {
  const buffer = new ArrayBuffer(8);
  const view = new DataView(buffer);
  view.setFloat64(0, value, true);
  return atom_node_bang(
    "clause/process-projected-f64-v1",
    new Uint8Array(buffer),
  );
}

function boolean_node_bang(value: boolean): number[] {
  return atom_node_bang("clause/process-projected-bool-v1", [value ? 1 : 0]);
}

function object_node_bang(fields: readonly TermField[]): number[] {
  if (fields.length === 0) {
    return atom_node_bang("clause/js-object-end-v1", []);
  }
  const [name, value] = fields[0];
  return triple_node_bang(
    atom_node_bang("clause/js-field-v1", ascii(name)),
    value,
    object_node_bang(fields.slice(1)),
  );
}

function array_node_bang(values: readonly ByteSequence[]): number[] {
  return values.length === 0
    ? atom_node_bang("clause/js-array-end-v1", [])
    : triple_node_bang(
        atom_node_bang("clause/js-item-v1", []),
        values[0],
        array_node_bang(values.slice(1)),
      );
}

function vec3_node_bang(x: number, y: number, z: number): number[] {
  return object_node_bang([
    ["x", number_node_bang(x)],
    ["y", number_node_bang(y)],
    ["z", number_node_bang(z)],
  ]);
}

function arena_term_bytes_bang(): number[] {
  const platform = object_node_bang([
    ["position", vec3_node_bang(0, -0.25, 0)],
    ["size", vec3_node_bang(12, 0.5, 12)],
  ]);
  const player = object_node_bang([
    ["position", vec3_node_bang(1, 2, 3)],
    ["velocity", vec3_node_bang(4, 5, 6)],
    ["yaw", number_node_bang(0.25)],
    ["grounded", boolean_node_bang(true)],
  ]);
  const root = object_node_bang([
    ["player", player],
    ["world", object_node_bang([["platforms", array_node_bang([platform])]])],
  ]);
  const bytes: number[] = [];
  for (const tag of [91, 92]) {
    bytes.push(...identity(tag));
  }
  bytes.push(...root);
  return bytes;
}

function minimal_cwr1_bang(): number[] {
  const bytes = [67, 87, 82, 49];
  append_little_blob_bang(bytes, [1]);
  append_little_u32_bang(bytes, 1);
  append_little_blob_bang(bytes, [8]);
  append_little_blob_bang(bytes, allocation_epoch_bang());
  for (const tag of [1, 2, 3, 4, 5, 6, 7, 8, 9]) {
    bytes.push(...identity(tag));
  }
  append_little_blob_bang(bytes, [9]);
  bytes.push(...identity(10));
  append_little_blob_bang(bytes, [10]);
  bytes.push(...identity(11));
  append_little_blob_bang(bytes, [11]);
  append_little_u64_bang(bytes, 100);
  bytes.push(2, 0);
  append_little_blob_bang(bytes, [1]);
  append_little_blob_bang(bytes, [2]);
  bytes.push(0, 0);
  return bytes;
}

function cse_header_bang(sequence: number, tag: number): number[] {
  const bytes = [67, 83, 69, 49];
  append_little_u32_bang(bytes, 0);
  append_little_u32_bang(bytes, 1);
  append_little_u64_bang(bytes, sequence);
  bytes.push(tag);
  return bytes;
}

function put_identities_bang(bytes: number[], tags: readonly number[]): void {
  for (const tag of tags) {
    bytes.push(...identity(tag));
  }
}

function opened_event_bang(): number[] {
  const bytes = cse_header_bang(0, 1);
  put_identities_bang(bytes, [21, 3, 22, 23, 24]);
  append_little_u32_bang(bytes, 1);
  append_little_blob_bang(bytes, allocation_epoch_bang());
  return bytes;
}

function input_event_bang(): number[] {
  const bytes = cse_header_bang(1, 2);
  put_identities_bang(bytes, [31, 23, 24, 32, 33]);
  append_little_u32_bang(bytes, 1);
  return bytes;
}

function candidate_event_bang(): number[] {
  const bytes = cse_header_bang(2, 3);
  put_identities_bang(bytes, [34, 35, 22, 23, 24]);
  append_little_u32_bang(bytes, 1);
  return bytes;
}

function issuance_event_bang(): number[] {
  const bytes = cse_header_bang(3, 4);
  put_identities_bang(bytes, [40, 21, 3, 22, 35]);
  append_little_u32_bang(bytes, 1);
  return bytes;
}

function admission_event_bang(term_bytes: ByteSequence): number[] {
  const bytes = cse_header_bang(4, 5);
  put_identities_bang(bytes, [22, 36, 37, 38, 3]);
  append_little_u32_bang(bytes, 2);
  bytes.push(1);
  put_identities_bang(bytes, [39]);
  append_little_blob_bang(bytes, term_bytes);
  return bytes;
}

function disposed_event_bang(): number[] {
  return cse_header_bang(5, 6);
}

function module_for_bang(
  events: number[][],
  requests: Uint8Array<ArrayBuffer>[],
): TestSessionModule {
  const current = createCell<Uint8Array<ArrayBuffer>>(new Uint8Array());
  const next_event_bang = (request: Uint8Array<ArrayBuffer>): number => {
    requests.push(request.slice());
    const next = events.shift();
    if (next === undefined) {
      throw new Error("test Wasm module exhausted its event fixture");
    }
    setCell(current, new Uint8Array(next));
    return 0;
  };
  return Object.freeze({
    clause_session_v1_open_bulk: next_event_bang,
    clause_session_v1_command_bulk: next_event_bang,
    clause_session_v1_event_bulk: () => current.value,
    clause_session_v1_reclaim_retired: () => true,
  });
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
  if (typeof encoded !== "string") {
    throw new Error("test value is not JSON-encodable");
  }
  return encoded;
}

function is_projected_object(
  value: wasm.ProjectedValue,
): value is wasm.ProjectedObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function is_projected_array(
  value: wasm.ProjectedValue,
): value is readonly wasm.ProjectedValue[] {
  return Array.isArray(value);
}

function require_projected_object(
  value: wasm.ProjectedValue,
): wasm.ProjectedObject {
  if (!is_projected_object(value)) {
    throw new Error("expected a projected object");
  }
  return value;
}

function require_projected_number(value: wasm.ProjectedValue): number {
  if (typeof value !== "number") {
    throw new Error("expected a projected number");
  }
  return value;
}

function require_projected_boolean(value: wasm.ProjectedValue): boolean {
  if (typeof value !== "boolean") {
    throw new Error("expected a projected boolean");
  }
  return value;
}

function require_projected_array(
  value: wasm.ProjectedValue,
): readonly wasm.ProjectedValue[] {
  if (!is_projected_array(value)) {
    throw new Error("expected a projected array");
  }
  return value;
}

function require_projected_vec3(value: wasm.ProjectedValue): ProjectedVec3 {
  const vector = require_projected_object(value);
  return Object.freeze({
    x: require_projected_number(vector.x),
    y: require_projected_number(vector.y),
    z: require_projected_number(vector.z),
  });
}

function require_projected_arena_frame(
  value: wasm.ProjectedValue,
): ProjectedArenaFrame {
  const frame = require_projected_object(value);
  const player = require_projected_object(frame.player);
  const world = require_projected_object(frame.world);
  return Object.freeze({
    player: Object.freeze({
      position: require_projected_vec3(player.position),
      grounded: require_projected_boolean(player.grounded),
    }),
    world: Object.freeze({
      platforms: require_projected_array(world.platforms),
    }),
  });
}

function require_callback<T extends (...arguments_: never[]) => unknown>(
  callback: T | null,
  description: string,
): T {
  if (callback === null) {
    throw new Error(description);
  }
  return callback;
}

test["test"](
  "persistent composition renders only the Admission-projected Term frame",
  () => {
    const term_bytes = arena_term_bytes_bang();
    const requests: Uint8Array<ArrayBuffer>[] = [];
    const rendered: wasm.ProjectedValue[] = [];
    const disposed = createCell(0);
    const scheduled = createCell<Tick | null>(null);
    const shell_input = createCell<EmitInput | null>(null);
    const cancellations = createCell(0);
    const module = module_for_bang(
      [
        opened_event_bang(),
        input_event_bang(),
        candidate_event_bang(),
        issuance_event_bang(),
        admission_event_bang(term_bytes),
        disposed_event_bang(),
      ],
      requests,
    );
    const browser = {
      setTimeout: (tick: Tick, _delay: number): number => {
        setCell(scheduled, tick);
        return 41;
      },
      clearTimeout: (_token: number): number =>
        setCell(cancellations, cancellations.value + 1),
    };
    const composition = integration[
      "create-passive-wasm-workbench-with-shell-factory"
    ](
      (emit_input: EmitInput) => {
        setCell(shell_input, emit_input);
        return {
          renderFrame: (frame: wasm.ProjectedValue): number =>
            rendered.push(frame),
          dispose: (): number => setCell(disposed, disposed.value + 1),
        };
      },
      integration["project-persistent-term-frame"],
      browser,
      module,
      minimal_cwr1_bang(),
      workbench["->FixedTick"](16),
      policy(),
      (_receipt: workbench.LifecycleReceipt): null => null,
    );
    const direct = require_projected_arena_frame(
      integration["project-persistent-term-frame"](term_bytes),
    );
    test["expect"](concatenate(direct.player.position.x)).toBe("1");
    test["expect"](json_string(rendered)).toBe("[]");
    test["expect"](concatenate(requests.length)).toBe("1");
    require_callback(shell_input.value, "shell input callback was not installed")({
      kind: "keyboard",
      code: "KeyD",
      phase: "down",
      repeat: false,
    });
    require_callback(scheduled.value, "fixed tick was not scheduled")();
    test["expect"](concatenate(requests.length)).toBe("5");
    test["expect"](concatenate(rendered.length)).toBe("1");
    const frame = require_projected_arena_frame(rendered[0]);
    test["expect"](
      json_string([
        frame.player.position.x,
        frame.player.position.y,
        frame.player.position.z,
      ]),
    ).toBe("[1,2,3]");
    test["expect"](frame.player.grounded ? "true" : "false").toBe("true");
    test["expect"](concatenate(frame.world.platforms.length)).toBe("1");
    composition.dispose();
    composition.dispose();
    test["expect"](concatenate(requests.length)).toBe("6");
    test["expect"](concatenate(disposed.value)).toBe("1");
    test["expect"](concatenate(cancellations.value)).toBe("1");
    require_callback(scheduled.value, "fixed tick was not retained")();
    test["expect"](concatenate(rendered.length)).toBe("1");
  },
);

test["test"](
  "projected Term realization rejects malformed and trailing bytes",
  () => {
    const valid = arena_term_bytes_bang();
    test["expect"](
      throws_p_bang(() =>
        integration["project-persistent-term-frame"](valid.concat([0])),
      )
        ? "true"
        : "false",
    ).toBe("true");
    const bad = valid.slice();
    bad[64] = 9;
    test["expect"](
      throws_p_bang(() => integration["project-persistent-term-frame"](bad))
        ? "true"
        : "false",
    ).toBe("true");
  },
);
