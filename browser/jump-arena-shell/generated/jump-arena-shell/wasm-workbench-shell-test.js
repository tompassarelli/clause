import * as integration from "./wasm-workbench-shell.js";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import * as test from "bun:test";
import { conj_value as $$bc$conj_value, equivV as $$bc$equiv, keyword as $$bc$keyword, property_key as $$bc$property_key, str as $$bc$str } from 'beagle/core.js';
import { catch_dispatch as $$bd$catch_dispatch } from 'beagle/exception-dispatch.js';

function policy() {
  const maximum = Number.MAX_SAFE_INTEGER;
  return workbench["->WorkbenchPolicy"](8, 8, 64, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"], workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
}

function identity(tag) {
  const bytes = new Array(32);
  bytes.fill(0);
  bytes.splice(0, 1, tag);
  bytes.splice(31, 1, tag);
  return bytes;
}

function append_little_u32_bang(bytes, value) {
  [1, 256, 65536, 16777216].forEach((divisor) => {
  bytes.push((Math.trunc(value / divisor) % 256));
});
}

function append_little_u64_bang(bytes, value) {
  append_little_u32_bang(bytes, value);
  return append_little_u32_bang(bytes, 0);
}

function append_big_u32_bang(bytes, value) {
  [16777216, 65536, 256, 1].forEach((divisor) => {
  bytes.push((Math.trunc(value / divisor) % 256));
});
}

function append_little_blob_bang(bytes, value) {
  append_little_u32_bang(bytes, value.length);
  value.forEach((byte) => {
  bytes.push(byte);
});
}

function ascii(source) {
  return (() => { let index = 0; let bytes = []; while (true) {
    if (($$bc$equiv(index, source.length))) { return bytes; } else { const _recur_0 = (index + 1); const _recur_1 = $$bc$conj_value(bytes, source.charCodeAt(index)); index = _recur_0; bytes = _recur_1; continue; }
  } })();
}

function atom_node_bang(kind, payload) {
  const bytes = [0];
  const kind_bytes = ascii(kind);
  append_big_u32_bang(bytes, kind_bytes.length);
  kind_bytes.forEach((byte) => {
  bytes.push(byte);
});
  append_big_u32_bang(bytes, payload.length);
  payload.forEach((byte) => {
  bytes.push(byte);
});
  bytes.push(0);
  return bytes;
}

function triple_node_bang(left, operator, right) {
  const bytes = [1];
  [left, operator, right].forEach((node) => {
  node.forEach((byte) => {
  bytes.push(byte);
});
});
  return bytes;
}

function number_node_bang(value) {
  const buffer = new ArrayBuffer(8);
  const view = new DataView(buffer);
  view.setFloat64(0, value, true);
  return atom_node_bang("clause/process-projected-f64-v1", new Uint8Array(buffer));
}

function boolean_node_bang(value) {
  return atom_node_bang("clause/process-projected-bool-v1", [(value ? 1 : 0)]);
}

function object_node_bang(fields) {
  if (($$bc$equiv(fields.length, 0))) {
    return atom_node_bang("clause/js-object-end-v1", []);
  } else {
    const field = fields[0];
    return triple_node_bang(atom_node_bang("clause/js-field-v1", ascii(field[0])), field[1], object_node_bang(fields.slice(1)));
  }
}

function array_node_bang(values) {
  return (($$bc$equiv(values.length, 0)) ? atom_node_bang("clause/js-array-end-v1", []) : triple_node_bang(atom_node_bang("clause/js-item-v1", []), values[0], array_node_bang(values.slice(1))));
}

function vec3_node_bang(x, y, z) {
  return object_node_bang([["x", number_node_bang(x)], ["y", number_node_bang(y)], ["z", number_node_bang(z)]]);
}

function arena_term_bytes_bang() {
  const platform = object_node_bang([["position", vec3_node_bang(0.0, -0.25, 0.0)], ["size", vec3_node_bang(12.0, 0.5, 12.0)]]);
  const player = object_node_bang([["position", vec3_node_bang(1.0, 2.0, 3.0)], ["velocity", vec3_node_bang(4.0, 5.0, 6.0)], ["yaw", number_node_bang(0.25)], ["grounded", boolean_node_bang(true)]]);
  const root = object_node_bang([["player", player], ["world", object_node_bang([["platforms", array_node_bang([platform])]])]]);
  const bytes = [];
  [91, 92].forEach((tag) => {
  identity(tag).forEach((byte) => {
  bytes.push(byte);
});
});
  root.forEach((byte) => {
  bytes.push(byte);
});
  return bytes;
}

function minimal_cwr1_bang() {
  const bytes = [67, 87, 82, 49];
  append_little_blob_bang(bytes, [1]);
  append_little_u32_bang(bytes, 1);
  [1, 2, 3, 4, 5, 6, 7, 8, 9].forEach((tag) => {
  identity(tag).forEach((byte) => {
  bytes.push(byte);
});
});
  append_little_blob_bang(bytes, [9]);
  identity(10).forEach((byte) => {
  bytes.push(byte);
});
  append_little_blob_bang(bytes, [10]);
  identity(11).forEach((byte) => {
  bytes.push(byte);
});
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
  tags.forEach((tag) => {
  identity(tag).forEach((byte) => {
  bytes.push(byte);
});
});
}

function opened_event_bang() {
  const bytes = cse_header_bang(0, 1);
  put_identities_bang(bytes, [21, 3, 22, 23, 24]);
  append_little_u32_bang(bytes, 1);
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

function admission_event_bang(term_bytes) {
  const bytes = cse_header_bang(3, 4);
  put_identities_bang(bytes, [22, 36, 37, 38, 3]);
  append_little_u32_bang(bytes, 2);
  bytes.push(1);
  put_identities_bang(bytes, [39]);
  append_little_blob_bang(bytes, term_bytes);
  return bytes;
}

function disposed_event_bang() {
  return cse_header_bang(4, 5);
}

function module_for_bang(events, requests) {
  const request = ({value: [], watches: {}});
  const current = ({value: [], watches: {}});
  const next_event_bang = () => { requests.push(request.value.slice());
(() => { const _a = current, _v = events.shift(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return 0; };
  return {[$$bc$property_key($$bc$keyword("clause_session_v1_io_reset"))]: () => (() => { const _a = request, _v = []; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })(), [$$bc$property_key($$bc$keyword("clause_session_v1_request_push"))]: (byte) => { request.value.push(byte);
return 0; }, [$$bc$property_key($$bc$keyword("clause_session_v1_open"))]: next_event_bang, [$$bc$property_key($$bc$keyword("clause_session_v1_command"))]: next_event_bang, [$$bc$property_key($$bc$keyword("clause_session_v1_event_len"))]: () => current.value.length, [$$bc$property_key($$bc$keyword("clause_session_v1_event_byte"))]: (index) => current.value[index]};
}

function throws_p_bang(action) {
  return (() => { try {
    action();
  return false;
  } catch (_catch_0) {
    switch ($$bd$catch_dispatch(_catch_0, [Error])) {
      case 0: {
        const __error = _catch_0;
        return true;
        break;
      }
    }
  } })();
}

function json_string(value) {
  const encoded = JSON.stringify(value);
  return (($$bc$equiv(typeof encoded, "string")) ? encoded : (() => { throw new Error("test value is not JSON-encodable"); })());
}

test["test"]("persistent composition renders only the Admission-projected Term frame", () => { const term_bytes = arena_term_bytes_bang();
const requests = [];
const rendered = [];
const disposed = ({value: 0, watches: {}});
const scheduled = ({value: null, watches: {}});
const cancellations = ({value: 0, watches: {}});
const module = module_for_bang([opened_event_bang(), input_event_bang(), candidate_event_bang(), admission_event_bang(term_bytes), disposed_event_bang()], requests);
const browser = {[$$bc$property_key($$bc$keyword("setTimeout"))]: (tick, __delay) => { (() => { const _a = scheduled, _v = tick; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return 41; }, [$$bc$property_key($$bc$keyword("clearTimeout"))]: (__token) => (() => { const _a = cancellations; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })()};
const composition = integration["create-passive-wasm-workbench-with-shell-factory"]((__emit_input) => ({[$$bc$property_key($$bc$keyword("renderFrame"))]: (frame) => rendered.push(frame), [$$bc$property_key($$bc$keyword("dispose"))]: () => (() => { const _a = disposed; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })()}), integration["project-persistent-term-frame"], browser, module, minimal_cwr1_bang(), workbench["->FixedTick"](16), policy(), (__receipt) => null);
const direct = integration["project-persistent-term-frame"](term_bytes);
test["expect"]($$bc$str(direct.player.position.x)).toBe("1");
test["expect"](json_string(rendered)).toBe("[]");
test["expect"]($$bc$str(requests.length)).toBe("1");
(scheduled.value)();
test["expect"]($$bc$str(requests.length)).toBe("4");
test["expect"]($$bc$str(rendered.length)).toBe("1");
const frame = rendered[0];
test["expect"](json_string([frame.player.position.x, frame.player.position.y, frame.player.position.z])).toBe("[1,2,3]");
test["expect"]((((_truthy) => _truthy !== false && _truthy != null)(frame.player.grounded) ? "true" : "false")).toBe("true");
test["expect"]($$bc$str(frame.world.platforms.length)).toBe("1");
(composition.dispose)();
(composition.dispose)();
test["expect"]($$bc$str(requests.length)).toBe("5");
test["expect"]($$bc$str(disposed.value)).toBe("1");
test["expect"]($$bc$str(cancellations.value)).toBe("1");
(scheduled.value)();
test["expect"]($$bc$str(rendered.length)).toBe("1");
return null; });

test["test"]("projected Term realization rejects malformed and trailing bytes", () => { const valid = arena_term_bytes_bang();
test["expect"]((throws_p_bang(() => integration["project-persistent-term-frame"](valid.concat([0]))) ? "true" : "false")).toBe("true");
const bad = valid.slice();
bad.splice(64, 1, 9);
test["expect"]((throws_p_bang(() => integration["project-persistent-term-frame"](bad)) ? "true" : "false")).toBe("true");
return null; });
//# sourceMappingURL=wasm-workbench-shell-test.js.map
