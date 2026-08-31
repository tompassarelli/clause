import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import * as test from "bun:test";
import { "clause_session_v1_command" as clause__session__v1__command, "clause_session_v1_event_byte" as clause__session__v1__event__byte, "clause_session_v1_event_len" as clause__session__v1__event__len, "clause_session_v1_io_reset" as clause__session__v1__io__reset, "clause_session_v1_open" as clause__session__v1__open, "clause_session_v1_request_push" as clause__session__v1__request__push, "initSync" as initSync } from "#clause-runtime-wasm";
import { equivV as $$bc$equiv, keyword as $$bc$keyword, property_key as $$bc$property_key, str as $$bc$str } from 'beagle/core.js';
import { catch_dispatch as $$bd$catch_dispatch } from 'beagle/exception-dispatch.js';

function policy() {
  const maximum = Number.MAX_SAFE_INTEGER;
  return workbench["->WorkbenchPolicy"](8, 8, 32, 128, 512, workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
}

function arena_policy() {
  const maximum = Number.MAX_SAFE_INTEGER;
  return workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"], workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
}

function identity(tag) {
  const bytes = new Array(32);
  bytes.fill(0);
  bytes.splice(0, 1, tag);
  bytes.splice(31, 1, tag);
  return bytes;
}

function cwo1(values) {
  const bytes = [67, 87, 79, 49];
  bytes.push(identity(11));
  bytes.push(identity(22));
  const flat = bytes.flat();
  const count = values.length;
  flat.push((count % 256));
  flat.push(Math.trunc(count / 256));
  values.forEach((value) => {
  if (($$bc$equiv(value.kind, "boolean"))) {
    flat.push(1);
    flat.push((((_truthy) => _truthy !== false && _truthy != null)(value.value) ? 1 : 0));
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

function module_for_bang(events, requests) {
  const request = ({value: [], watches: {}});
  const current = ({value: [], watches: {}});
  const next_event_bang = () => { requests.push(request.value.slice());
(() => { const _a = current, _v = events.shift(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return 0; };
  return {[$$bc$property_key($$bc$keyword("clause_session_v1_io_reset"))]: () => (() => { const _a = request, _v = []; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })(), [$$bc$property_key($$bc$keyword("clause_session_v1_request_push"))]: (byte) => { request.value.push(byte);
return 0; }, [$$bc$property_key($$bc$keyword("clause_session_v1_open"))]: next_event_bang, [$$bc$property_key($$bc$keyword("clause_session_v1_command"))]: next_event_bang, [$$bc$property_key($$bc$keyword("clause_session_v1_event_len"))]: () => current.value.length, [$$bc$property_key($$bc$keyword("clause_session_v1_event_byte"))]: (index) => current.value[index]};
}

function append_u32_bang(bytes, value) {
  [1, 256, 65536, 16777216].forEach((divisor) => {
  bytes.push((Math.trunc(value / divisor) % 256));
});
}

function append_u64_bang(bytes, value) {
  append_u32_bang(bytes, value);
  return append_u32_bang(bytes, 0);
}

function append_blob_bang(bytes, value) {
  append_u32_bang(bytes, value.length);
  value.forEach((byte) => {
  bytes.push(byte);
});
}

function allocation_epoch_bang() {
  const bytes = new Array(304);
  bytes.fill(0);
  bytes.splice(0, 4, 82, 65, 69, 49);
  return bytes;
}

function minimal_cwr1_bang() {
  const bytes = [67, 87, 82, 49];
  append_blob_bang(bytes, [1]);
  append_u32_bang(bytes, 1);
  append_blob_bang(bytes, [8]);
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
  bytes.push(2, 0);
  append_blob_bang(bytes, [1]);
  append_blob_bang(bytes, [2]);
  bytes.push(0, 0);
  return bytes;
}

function cse_header_bang(sequence, tag) {
  const bytes = [67, 83, 69, 49];
  append_u32_bang(bytes, 0);
  append_u32_bang(bytes, 1);
  append_u64_bang(bytes, sequence);
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
  append_u32_bang(bytes, 1);
  append_blob_bang(bytes, allocation_epoch_bang());
  return bytes;
}

function input_event_bang() {
  const bytes = cse_header_bang(1, 2);
  put_identities_bang(bytes, [31, 23, 24, 32, 33]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function candidate_event_bang() {
  const bytes = cse_header_bang(2, 3);
  put_identities_bang(bytes, [34, 35, 22, 23, 24]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function issuance_event_bang() {
  const bytes = cse_header_bang(3, 4);
  put_identities_bang(bytes, [40, 21, 3, 22, 35]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function admission_event_bang() {
  const bytes = cse_header_bang(4, 5);
  put_identities_bang(bytes, [22, 36, 37, 38, 3]);
  append_u32_bang(bytes, 2);
  bytes.push(1);
  put_identities_bang(bytes, [39]);
  append_blob_bang(bytes, [40, 41, 42]);
  return bytes;
}

function disposed_event_bang() {
  return cse_header_bang(5, 6);
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

function initialize_real_session_module(module) {
  const input = module;
  const __initialized = initSync(input);
  return Object.freeze({[$$bc$property_key($$bc$keyword("clause_session_v1_io_reset"))]: () => clause__session__v1__io__reset(), [$$bc$property_key($$bc$keyword("clause_session_v1_request_push"))]: (byte) => clause__session__v1__request__push(byte), [$$bc$property_key($$bc$keyword("clause_session_v1_open"))]: () => clause__session__v1__open(), [$$bc$property_key($$bc$keyword("clause_session_v1_command"))]: () => clause__session__v1__command(), [$$bc$property_key($$bc$keyword("clause_session_v1_event_len"))]: () => clause__session__v1__event__len(), [$$bc$property_key($$bc$keyword("clause_session_v1_event_byte"))]: (index) => clause__session__v1__event__byte(index)});
}

function key_configuration(input_sequence, revision, code) {
  return workbench["->InputConfiguration"](revision, [workbench["->InputObservation"](input_sequence, workbench["create-workbench-envelope"](policy(), json_string([json_string({[$$bc$property_key($$bc$keyword("kind"))]: "keyboard", [$$bc$property_key($$bc$keyword("code"))]: code, [$$bc$property_key($$bc$keyword("phase"))]: "down", [$$bc$property_key($$bc$keyword("repeat"))]: false})])))]);
}

function process_configuration(input_sequence, revision, ordinal) {
  return workbench["->InputConfiguration"](revision, [workbench["->InputObservation"](input_sequence, workbench["create-workbench-envelope"](policy(), json_string([json_string({[$$bc$property_key($$bc$keyword("kind"))]: "process-occurrence", [$$bc$property_key($$bc$keyword("ordinal"))]: ordinal})])))]);
}

function admit_real_collect_bang(port, request_bytes, expected_score) {
  const request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](request_bytes));
  const accepted = ({value: null, watches: {}});
  const started = ({value: null, watches: {}});
  const candidate = ({value: null, watches: {}});
  const admitted = ({value: null, watches: {}});
  (port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
  (port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
  (port.runCandidate)(started.value.session, workbench["->FixedTick"](16), process_configuration(1, 1, 0), (result) => (() => { const _a = candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
  test["expect"](json_string(started.value.frame)).toBe("[]");
  test["expect"](json_string(candidate.value.candidate.base)).toBe(json_string(started.value.revision));
  (port.requestAdmission)(started.value.session, candidate.value.candidate, (result) => (() => { const _a = admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
  test["expect"](((!(json_string(admitted.value.revision) === json_string(started.value.revision))) ? "true" : "false")).toBe("true");
  const frame = wasm["decode-projected-term-frame"](admitted.value.frame);
  const score = frame.player.score;
  test["expect"]($$bc$str(score)).toBe($$bc$str(expected_score));
  (port.disposeSession)(started.value.session);
  return score;
}

test["test"]("one persistent session sequences physical input candidate issuance Admission and disposal", () => { const requests = [];
const port = wasm["create-wasm-cartridge-port"](module_for_bang([opened_event_bang(), input_event_bang(), candidate_event_bang(), issuance_event_bang(), admission_event_bang(), disposed_event_bang()], requests), policy());
const request = wasm["->ExactProcessRequest"](minimal_cwr1_bang());
const accepted = ({value: null, watches: {}});
const started = ({value: null, watches: {}});
const candidate = ({value: null, watches: {}});
const admitted = ({value: null, watches: {}});
(port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), workbench["->InputConfiguration"](1, [workbench["->InputObservation"](1, workbench["create-workbench-envelope"](policy(), json_string([json_string({[$$bc$property_key($$bc$keyword("kind"))]: "keyboard", [$$bc$property_key($$bc$keyword("code"))]: "KeyD", [$$bc$property_key($$bc$keyword("phase"))]: "down", [$$bc$property_key($$bc$keyword("repeat"))]: false})])))]), (result) => (() => { const _a = candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test["expect"]($$bc$str(requests.length)).toBe("3");
const open_request = requests[0];
const open_length = open_request.length;
const allocation_tag = (open_length - 17);
test["expect"](json_string(open_request.slice(0, 4))).toBe("[67,87,83,49]");
test["expect"](json_string(open_request.slice(13, 18))).toBe("[1,0,0,0,8]");
test["expect"]($$bc$str(open_request[allocation_tag])).toBe("0");
test["expect"](json_string(open_request.slice((allocation_tag + 1), (allocation_tag + 9)))).toBe("[0,16,0,0,0,0,0,0]");
test["expect"](json_string([requests[1][20], requests[2][20]])).toBe("[3,4]");
test["expect"](json_string(started.value.frame)).toBe("[]");
(port.requestAdmission)(started.value.session, candidate.value.candidate, (result) => (() => { const _a = admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test["expect"]($$bc$str(requests.length)).toBe("5");
test["expect"](json_string([requests[3][20], requests[4][20]])).toBe("[5,6]");
test["expect"](json_string(admitted.value.revision)).toBe(json_string(identity(36)));
test["expect"](json_string(admitted.value.frame)).toBe("[40,41,42]");
(port.disposeSession)(started.value.session);
test["expect"]($$bc$str(requests[5][20])).toBe("7");
const after_dispose = ({value: null, watches: {}});
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), workbench["->InputConfiguration"](0, []), (result) => (() => { const _a = after_dispose, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test["expect"]($$bc$str(after_dispose.value.reason)).toBe("Wasm session is disposed");
return null; });

test["test"]("strict CWO1 decoding rejects malformed and trailing bytes", () => { const valid = cwo1([{[$$bc$property_key($$bc$keyword("kind"))]: "boolean", [$$bc$property_key($$bc$keyword("value"))]: false}]);
test["expect"]((throws_p_bang(() => wasm["decode-cwo1-observation"](valid.concat([0]))) ? "true" : "false")).toBe("true");
const bad = valid.slice();
bad.splice(0, 1, 88);
test["expect"]((throws_p_bang(() => wasm["decode-cwo1-observation"](bad)) ? "true" : "false")).toBe("true");
return null; });

test["test"]("CWR1 hex transport is exact and bounded", () => { test["expect"](json_string(wasm["decode-cwr1-hex"]("43 57\n52\t31"))).toBe("[67,87,82,49]");
(() => { ["", "0", "0g", "0A"].forEach((source) => {
  test["expect"]((throws_p_bang(() => wasm["decode-cwr1-hex"](source)) ? "true" : "false")).toBe("true");
}); })();
return null; });

const collect_test_runtime = require("bun:test");
const register_collect_test = collect_test_runtime.test;
register_collect_test("real Wasm lowers physical input and exposes only the admitted arena frame", () => Promise.all([Bun.file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), Bun.file("./fixtures/wasm-jump-v1/jump-v1.cwr1.hex").text()]).then((assets) => { const port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
const request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[1]));
const accepted = ({value: null, watches: {}});
const started = ({value: null, watches: {}});
const candidate = ({value: null, watches: {}});
const admitted = ({value: null, watches: {}});
const airborne_candidate = ({value: null, watches: {}});
const airborne_admitted = ({value: null, watches: {}});
const momentum_candidate = ({value: null, watches: {}});
const momentum_admitted = ({value: null, watches: {}});
(port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), key_configuration(1, 1, "KeyD"), (result) => (() => { const _a = candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test["expect"](json_string(started.value.frame)).toBe("[]");
(port.requestAdmission)(started.value.session, candidate.value.candidate, (result) => (() => { const _a = admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const frame = wasm["decode-projected-term-frame"](admitted.value.frame);
test["expect"](((frame.player.position.x > 0.0) ? "true" : "false")).toBe("true");
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), key_configuration(2, 2, "Space"), (result) => (() => { const _a = airborne_candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.requestAdmission)(started.value.session, airborne_candidate.value.candidate, (result) => (() => { const _a = airborne_admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const jump_frame = wasm["decode-projected-term-frame"](airborne_admitted.value.frame);
test["expect"]((((jump_frame.player.position.x > 0.0) && ((jump_frame.player.position.y > 0.0) && ($$bc$equiv(false, jump_frame.player.grounded)))) ? "true" : "false")).toBe("true");
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), workbench["->InputConfiguration"](3, []), (result) => (() => { const _a = momentum_candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.requestAdmission)(started.value.session, momentum_candidate.value.candidate, (result) => (() => { const _a = momentum_admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const airborne_frame = wasm["decode-projected-term-frame"](airborne_admitted.value.frame);
const momentum_frame = wasm["decode-projected-term-frame"](momentum_admitted.value.frame);
test["expect"](((momentum_frame.player.position.x > airborne_frame.player.position.x) ? "true" : "false")).toBe("true");
(port.disposeSession)(started.value.session);
return null; }));

const test_runtime = require("bun:test");
const register_async_test = test_runtime.test;
register_async_test("real Wasm keeps collect hidden until Admission and projects Clause-owned score", () => Promise.all([Bun.file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), Bun.file("./fixtures/wasm-collect-v1/collect-plus-1.cwr1.hex").text(), Bun.file("./fixtures/wasm-collect-v1/collect-plus-4.cwr1.hex").text()]).then((assets) => { const port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
const base_score = admit_real_collect_bang(port, assets[1], 1.0);
const changed_score = admit_real_collect_bang(port, assets[2], 4.0);
test["expect"]($$bc$str(base_score)).toBe("1");
test["expect"]($$bc$str(changed_score)).toBe("4");
return null; }));
//# sourceMappingURL=wasm-cartridge-port-test.js.map
