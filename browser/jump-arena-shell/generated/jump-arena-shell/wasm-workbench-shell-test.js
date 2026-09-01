import * as integration from "./wasm-workbench-shell.js";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import * as test from "bun:test";
function createCell(value) {
    return { value, watches: {} };
}
function setCell(cell, value) {
    const previous = cell.value;
    cell.value = value;
    for (const [key, watch] of Object.entries(cell.watches)) {
        watch(key, cell, previous, value);
    }
    return value;
}
function concatenate(...values) {
    return values.map(String).join("");
}
function policy() {
    const maximum = Number.MAX_SAFE_INTEGER;
    return workbench["->WorkbenchPolicy"](8, 8, 64, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"], workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
}
function identity(tag) {
    const bytes = new Array(32).fill(0);
    bytes[0] = tag;
    bytes[31] = tag;
    return bytes;
}
function append_little_u32_bang(bytes, value) {
    for (const divisor of [1, 256, 65536, 16777216]) {
        bytes.push(Math.trunc(value / divisor) % 256);
    }
}
function append_little_u64_bang(bytes, value) {
    append_little_u32_bang(bytes, value);
    append_little_u32_bang(bytes, 0);
}
function append_big_u32_bang(bytes, value) {
    for (const divisor of [16777216, 65536, 256, 1]) {
        bytes.push(Math.trunc(value / divisor) % 256);
    }
}
function append_little_blob_bang(bytes, value) {
    append_little_u32_bang(bytes, value.length);
    value.forEach((byte) => bytes.push(byte));
}
function allocation_epoch_bang() {
    const bytes = new Array(304).fill(0);
    bytes.splice(0, 4, 82, 65, 69, 49);
    return bytes;
}
function ascii(source) {
    const bytes = [];
    for (let index = 0; index < source.length; index += 1) {
        bytes.push(source.charCodeAt(index));
    }
    return bytes;
}
function atom_node_bang(kind, payload) {
    const bytes = [0];
    const kind_bytes = ascii(kind);
    append_big_u32_bang(bytes, kind_bytes.length);
    kind_bytes.forEach((byte) => bytes.push(byte));
    append_big_u32_bang(bytes, payload.length);
    payload.forEach((byte) => bytes.push(byte));
    bytes.push(0);
    return bytes;
}
function triple_node_bang(left, operator, right) {
    const bytes = [1];
    for (const node of [left, operator, right]) {
        node.forEach((byte) => bytes.push(byte));
    }
    return bytes;
}
function number_node_bang(value) {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setFloat64(0, value, true);
    return atom_node_bang("clause/process-projected-f64-v1", new Uint8Array(buffer));
}
function boolean_node_bang(value) {
    return atom_node_bang("clause/process-projected-bool-v1", [value ? 1 : 0]);
}
function object_node_bang(fields) {
    if (fields.length === 0) {
        return atom_node_bang("clause/js-object-end-v1", []);
    }
    const [name, value] = fields[0];
    return triple_node_bang(atom_node_bang("clause/js-field-v1", ascii(name)), value, object_node_bang(fields.slice(1)));
}
function array_node_bang(values) {
    return values.length === 0
        ? atom_node_bang("clause/js-array-end-v1", [])
        : triple_node_bang(atom_node_bang("clause/js-item-v1", []), values[0], array_node_bang(values.slice(1)));
}
function vec3_node_bang(x, y, z) {
    return object_node_bang([
        ["x", number_node_bang(x)],
        ["y", number_node_bang(y)],
        ["z", number_node_bang(z)],
    ]);
}
function arena_term_bytes_bang() {
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
    const bytes = [];
    for (const tag of [91, 92]) {
        bytes.push(...identity(tag));
    }
    bytes.push(...root);
    return bytes;
}
function minimal_cwr1_bang() {
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
function cse_header_bang(sequence, tag) {
    const bytes = [67, 83, 69, 49];
    append_little_u32_bang(bytes, 0);
    append_little_u32_bang(bytes, 1);
    append_little_u64_bang(bytes, sequence);
    bytes.push(tag);
    return bytes;
}
function put_identities_bang(bytes, tags) {
    for (const tag of tags) {
        bytes.push(...identity(tag));
    }
}
function opened_event_bang() {
    const bytes = cse_header_bang(0, 1);
    put_identities_bang(bytes, [21, 3, 22, 23, 24]);
    append_little_u32_bang(bytes, 1);
    append_little_blob_bang(bytes, allocation_epoch_bang());
    return bytes;
}
function input_event_bang() {
    const bytes = cse_header_bang(1, 2);
    put_identities_bang(bytes, [31, 23, 24, 32, 33]);
    append_little_u32_bang(bytes, 1);
    return bytes;
}
function candidate_event_bang() {
    const bytes = cse_header_bang(2, 3);
    put_identities_bang(bytes, [34, 35, 22, 23, 24]);
    append_little_u32_bang(bytes, 1);
    return bytes;
}
function issuance_event_bang() {
    const bytes = cse_header_bang(3, 4);
    put_identities_bang(bytes, [40, 21, 3, 22, 35]);
    append_little_u32_bang(bytes, 1);
    return bytes;
}
function admission_event_bang(term_bytes) {
    const bytes = cse_header_bang(4, 5);
    put_identities_bang(bytes, [22, 36, 37, 38, 3]);
    append_little_u32_bang(bytes, 2);
    bytes.push(1);
    put_identities_bang(bytes, [39]);
    append_little_blob_bang(bytes, term_bytes);
    return bytes;
}
function disposed_event_bang() {
    return cse_header_bang(5, 6);
}
function module_for_bang(events, requests) {
    const current = createCell(new Uint8Array());
    const next_event_bang = (request) => {
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
function throws_p_bang(action) {
    try {
        action();
        return false;
    }
    catch (error) {
        if (error instanceof Error)
            return true;
        throw error;
    }
}
function json_string(value) {
    const encoded = JSON.stringify(value);
    if (typeof encoded !== "string") {
        throw new Error("test value is not JSON-encodable");
    }
    return encoded;
}
function is_projected_object(value) {
    return typeof value === "object" && value !== null && !Array.isArray(value);
}
function is_projected_array(value) {
    return Array.isArray(value);
}
function require_projected_object(value) {
    if (!is_projected_object(value)) {
        throw new Error("expected a projected object");
    }
    return value;
}
function require_projected_number(value) {
    if (typeof value !== "number") {
        throw new Error("expected a projected number");
    }
    return value;
}
function require_projected_boolean(value) {
    if (typeof value !== "boolean") {
        throw new Error("expected a projected boolean");
    }
    return value;
}
function require_projected_array(value) {
    if (!is_projected_array(value)) {
        throw new Error("expected a projected array");
    }
    return value;
}
function require_projected_vec3(value) {
    const vector = require_projected_object(value);
    return Object.freeze({
        x: require_projected_number(vector.x),
        y: require_projected_number(vector.y),
        z: require_projected_number(vector.z),
    });
}
function require_projected_arena_frame(value) {
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
function require_callback(callback, description) {
    if (callback === null) {
        throw new Error(description);
    }
    return callback;
}
test["test"]("persistent composition renders only the Admission-projected Term frame", () => {
    const term_bytes = arena_term_bytes_bang();
    const requests = [];
    const rendered = [];
    const disposed = createCell(0);
    const scheduled = createCell(null);
    const shell_input = createCell(null);
    const cancellations = createCell(0);
    const module = module_for_bang([
        opened_event_bang(),
        input_event_bang(),
        candidate_event_bang(),
        issuance_event_bang(),
        admission_event_bang(term_bytes),
        disposed_event_bang(),
    ], requests);
    const browser = {
        setTimeout: (tick, _delay) => {
            setCell(scheduled, tick);
            return 41;
        },
        clearTimeout: (_token) => setCell(cancellations, cancellations.value + 1),
    };
    const composition = integration["create-passive-wasm-workbench-with-shell-factory"]((emit_input) => {
        setCell(shell_input, emit_input);
        return {
            renderFrame: (frame) => rendered.push(frame),
            dispose: () => setCell(disposed, disposed.value + 1),
        };
    }, integration["project-persistent-term-frame"], browser, module, minimal_cwr1_bang(), workbench["->FixedTick"](16), policy(), (_receipt) => null);
    const direct = require_projected_arena_frame(integration["project-persistent-term-frame"](term_bytes));
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
    test["expect"](json_string([
        frame.player.position.x,
        frame.player.position.y,
        frame.player.position.z,
    ])).toBe("[1,2,3]");
    test["expect"](frame.player.grounded ? "true" : "false").toBe("true");
    test["expect"](concatenate(frame.world.platforms.length)).toBe("1");
    composition.dispose();
    composition.dispose();
    test["expect"](concatenate(requests.length)).toBe("6");
    test["expect"](concatenate(disposed.value)).toBe("1");
    test["expect"](concatenate(cancellations.value)).toBe("1");
    require_callback(scheduled.value, "fixed tick was not retained")();
    test["expect"](concatenate(rendered.length)).toBe("1");
});
test["test"]("projected Term realization rejects malformed and trailing bytes", () => {
    const valid = arena_term_bytes_bang();
    test["expect"](throws_p_bang(() => integration["project-persistent-term-frame"](valid.concat([0])))
        ? "true"
        : "false").toBe("true");
    const bad = valid.slice();
    bad[64] = 9;
    test["expect"](throws_p_bang(() => integration["project-persistent-term-frame"](bad))
        ? "true"
        : "false").toBe("true");
});
//# sourceMappingURL=wasm-workbench-shell-test.js.map