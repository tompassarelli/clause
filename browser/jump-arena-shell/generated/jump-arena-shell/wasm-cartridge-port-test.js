import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import * as test from "bun:test";
import { "file" as file } from "bun";
import { "clause_session_v1_command_bulk" as clause__session__v1__command__bulk, "clause_session_v1_event_bulk" as clause__session__v1__event__bulk, "clause_session_v1_open_bulk" as clause__session__v1__open__bulk, "initSync" as initSync } from "#clause-runtime-wasm";
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
  const current = ({value: [], watches: {}});
  const next_event_bang = (request) => { requests.push(request.slice());
(() => { const _a = current, _v = events.shift(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return 0; };
  return {[$$bc$property_key($$bc$keyword("clause_session_v1_open_bulk"))]: next_event_bang, [$$bc$property_key($$bc$keyword("clause_session_v1_command_bulk"))]: next_event_bang, [$$bc$property_key($$bc$keyword("clause_session_v1_event_bulk"))]: () => current.value};
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
  put_identities_bang(bytes, [22, 36, 37, 38, 41, 42, 3]);
  append_u32_bang(bytes, 2);
  bytes.push(1);
  put_identities_bang(bytes, [39]);
  append_blob_bang(bytes, [40, 41, 42]);
  return bytes;
}

function suspended_event_bang() {
  const bytes = cse_header_bang(2, 8);
  put_identities_bang(bytes, [34, 35, 23, 24, 36, 37]);
  append_u64_bang(bytes, 98);
  append_u32_bang(bytes, 1);
  return bytes;
}

function resumed_event_bang() {
  const bytes = cse_header_bang(3, 9);
  put_identities_bang(bytes, [38, 39, 35, 23, 24, 37, 40]);
  append_u64_bang(bytes, 97);
  append_u32_bang(bytes, 1);
  return bytes;
}

function resumed_candidate_event_bang() {
  const bytes = cse_header_bang(4, 3);
  put_identities_bang(bytes, [41, 42, 22, 23, 24]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function resumed_issuance_event_bang() {
  const bytes = cse_header_bang(5, 4);
  put_identities_bang(bytes, [43, 21, 3, 22, 42]);
  append_u32_bang(bytes, 1);
  return bytes;
}

function resumed_admission_event_bang() {
  const bytes = cse_header_bang(6, 5);
  put_identities_bang(bytes, [22, 44, 45, 46, 47, 48, 3]);
  append_u32_bang(bytes, 2);
  bytes.push(0);
  return bytes;
}

function disposed_event_bang() {
  return cse_header_bang(5, 6);
}

function resumed_disposed_event_bang() {
  return cse_header_bang(7, 6);
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
  return Object.freeze({[$$bc$property_key($$bc$keyword("clause_session_v1_open_bulk"))]: (request) => clause__session__v1__open__bulk(new Uint8Array(request)), [$$bc$property_key($$bc$keyword("clause_session_v1_command_bulk"))]: (request) => clause__session__v1__command__bulk(new Uint8Array(request)), [$$bc$property_key($$bc$keyword("clause_session_v1_event_bulk"))]: () => clause__session__v1__event__bulk()});
}

function key_configuration(input_sequence, revision, code) {
  return workbench["->InputConfiguration"](revision, [workbench["->InputObservation"](input_sequence, workbench["create-workbench-envelope"](policy(), json_string([json_string({[$$bc$property_key($$bc$keyword("kind"))]: "keyboard", [$$bc$property_key($$bc$keyword("code"))]: code, [$$bc$property_key($$bc$keyword("phase"))]: "down", [$$bc$property_key($$bc$keyword("repeat"))]: false})])))]);
}

function two_key_configuration(first_input_sequence, revision, first_code, second_code) {
  return workbench["->InputConfiguration"](revision, [workbench["->InputObservation"](first_input_sequence, workbench["create-workbench-envelope"](policy(), json_string([json_string({[$$bc$property_key($$bc$keyword("kind"))]: "keyboard", [$$bc$property_key($$bc$keyword("code"))]: first_code, [$$bc$property_key($$bc$keyword("phase"))]: "down", [$$bc$property_key($$bc$keyword("repeat"))]: false})]))), workbench["->InputObservation"]((first_input_sequence + 1), workbench["create-workbench-envelope"](policy(), json_string([json_string({[$$bc$property_key($$bc$keyword("kind"))]: "keyboard", [$$bc$property_key($$bc$keyword("code"))]: second_code, [$$bc$property_key($$bc$keyword("phase"))]: "down", [$$bc$property_key($$bc$keyword("repeat"))]: false})])))]);
}

function process_configuration(input_sequence, revision, ordinal) {
  return workbench["->InputConfiguration"](revision, [workbench["->InputObservation"](input_sequence, workbench["create-workbench-envelope"](policy(), json_string([json_string({[$$bc$property_key($$bc$keyword("kind"))]: "process-occurrence", [$$bc$property_key($$bc$keyword("ordinal"))]: ordinal})])))]);
}

function key_envelope(code, phase) {
  return workbench["create-workbench-envelope"](arena_policy(), json_string([json_string({[$$bc$property_key($$bc$keyword("kind"))]: "keyboard", [$$bc$property_key($$bc$keyword("code"))]: code, [$$bc$property_key($$bc$keyword("phase"))]: phase, [$$bc$property_key($$bc$keyword("repeat"))]: false})]));
}

function admit_real_process_bang(port, request_bytes) {
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
  (port.disposeSession)(started.value.session);
  return frame;
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

test["test"]("persistent CWI1 commands retain continuation custody and exact Admission identities", () => { const requests = [];
const module = module_for_bang([opened_event_bang(), input_event_bang(), suspended_event_bang(), resumed_event_bang(), resumed_candidate_event_bang(), resumed_issuance_event_bang(), resumed_admission_event_bang(), resumed_disposed_event_bang()], requests);
const port = wasm["create-wasm-cartridge-port"](module, policy());
const request = wasm["->ExactProcessRequest"](minimal_cwr1_bang());
const accepted = ({value: null, watches: {}});
const started = ({value: null, watches: {}});
const candidate = ({value: null, watches: {}});
(port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const session = started.value.session;
const input = wasm["advance-session-occurrence!"](module, session, 0);
const suspension = wasm["suspend-session!"](module, session);
const resumption = wasm["resume-session!"](module, session);
test["expect"]($$bc$str(input.kind)).toBe("input");
test["expect"](json_string(suspension.continuation)).toBe(json_string(resumption.continuation));
test["expect"]($$bc$str(suspension.remainingBudget)).toBe("98");
test["expect"]($$bc$str(resumption.remainingBudget)).toBe("97");
(port.runCandidate)(session, workbench["->FixedTick"](16), process_configuration(1, 1, 0), (result) => (() => { const _a = candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const admission = wasm["admit-session-candidate!"](module, session, candidate.value.candidate);
test["expect"](json_string(admission.predecessor)).toBe(json_string(identity(22)));
test["expect"](json_string(admission.admissionId)).toBe(json_string(identity(45)));
test["expect"](json_string(admission.judgmentId)).toBe(json_string(identity(46)));
test["expect"](json_string(admission.successor)).toBe(json_string(identity(44)));
(port.disposeSession)(session);
test["expect"](json_string(requests.slice(1).map((bytes) => bytes[20]))).toBe("[1,8,9,2,5,6,7]");
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
register_collect_test("real Wasm lowers physical input and exposes only the admitted arena frame", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-jump-v1/jump-v1.cwr1.hex").text()]).then((assets) => { const port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
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
register_async_test("real Wasm keeps collect hidden until Admission and projects Clause-owned score", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-collect-v1/collect-plus-1.cwr1.hex").text(), file("./fixtures/wasm-collect-v1/collect-plus-4.cwr1.hex").text()]).then((assets) => { const port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
const base_frame = admit_real_process_bang(port, assets[1]);
const changed_frame = admit_real_process_bang(port, assets[2]);
const base_score = base_frame.player.score;
const changed_score = changed_frame.player.score;
test["expect"]($$bc$str(base_score)).toBe("1");
test["expect"]($$bc$str(changed_score)).toBe("4");
return null; }));

const continuation_test_runtime = require("bun:test");
const register_continuation_test = continuation_test_runtime.test;
register_continuation_test("real Wasm suspends resumes proposes and admits one domain-neutral process", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-process-continuation-v1/process-continuation-v1.cwr1.hex").text()]).then((assets) => { const module = initialize_real_session_module(assets[0]);
const port = wasm["create-wasm-cartridge-port"](module, policy());
const request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[1]));
const accepted = ({value: null, watches: {}});
const started = ({value: null, watches: {}});
const candidate = ({value: null, watches: {}});
(port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const session = started.value.session;
const input = wasm["advance-session-occurrence!"](module, session, 0);
const suspension = wasm["suspend-session!"](module, session);
const resumption = wasm["resume-session!"](module, session);
const suspension_budget = suspension.remainingBudget;
const resumption_budget = resumption.remainingBudget;
test["expect"]($$bc$str(input.kind)).toBe("input");
test["expect"](json_string(suspension.continuation)).toBe(json_string(resumption.continuation));
test["expect"](json_string(suspension.run)).toBe(json_string(resumption.run));
test["expect"](json_string(suspension.activation)).toBe(json_string(resumption.activation));
test["expect"]($$bc$str(resumption_budget)).toBe($$bc$str((suspension_budget - 1)));
(port.runCandidate)(session, workbench["->FixedTick"](16), process_configuration(1, 1, 1), (result) => (() => { const _a = candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const proposed = candidate.value.candidate;
const admission = wasm["admit-session-candidate!"](module, session, proposed);
test["expect"](json_string(proposed.base)).toBe(json_string(admission.predecessor));
test["expect"]($$bc$str(admission.admissionId.length)).toBe("32");
test["expect"]($$bc$str(admission.judgmentId.length)).toBe("32");
test["expect"](((!(json_string(admission.successor) === json_string(admission.predecessor))) ? "true" : "false")).toBe("true");
(port.disposeSession)(session);
return null; }));

const symbol_test_runtime = require("bun:test");
const register_symbol_test = symbol_test_runtime.test;
register_symbol_test("real Wasm admits Clause-owned active to collected symbolic state", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-collect-state-v1/collected.cwr1.hex").text(), file("./fixtures/wasm-collect-state-v1/spent.cwr1.hex").text()]).then((assets) => { const port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
const collected_frame = admit_real_process_bang(port, assets[1]);
const spent_frame = admit_real_process_bang(port, assets[2]);
const collected_state = collected_frame.collectible.state;
const spent_state = spent_frame.collectible.state;
test["expect"](collected_state).toBe("collected");
test["expect"](spent_state).toBe("spent");
return null; }));

const gameplay_test_runtime = require("bun:test");
const register_gameplay_test = gameplay_test_runtime.test;
register_gameplay_test("one real Wasm gameplay session collects then automatically launches from Clause contact", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-gameplay-v1/gameplay-v1.cwr1.hex").text()]).then((assets) => { const port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
const request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[1]));
const accepted = ({value: null, watches: {}});
const started = ({value: null, watches: {}});
const collect_candidate = ({value: null, watches: {}});
const collect_admitted = ({value: null, watches: {}});
const launch_candidate = ({value: null, watches: {}});
const launch_admitted = ({value: null, watches: {}});
const airborne_candidate = ({value: null, watches: {}});
const airborne_admitted = ({value: null, watches: {}});
(port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), key_configuration(1, 1, "KeyD"), (result) => (() => { const _a = collect_candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test["expect"](json_string(started.value.frame)).toBe("[]");
(port.requestAdmission)(started.value.session, collect_candidate.value.candidate, (result) => (() => { const _a = collect_admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const collected_frame = wasm["decode-projected-term-frame"](collect_admitted.value.frame);
const player_x = collected_frame.player.position.x;
const collected_collectible = collected_frame.world.collectibles[0];
const collected_collectible_state = collected_collectible.state;
const collectible_x = collected_collectible.position.x;
test["expect"](collected_collectible_state).toBe("collected");
test["expect"]($$bc$str(player_x)).toBe($$bc$str(collectible_x));
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), workbench["->InputConfiguration"](2, []), (result) => (() => { const _a = launch_candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.requestAdmission)(started.value.session, launch_candidate.value.candidate, (result) => (() => { const _a = launch_admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const launch_frame = wasm["decode-projected-term-frame"](launch_admitted.value.frame);
const launch_player = launch_frame.player;
test["expect"]($$bc$str(launch_player.velocity.y)).toBe("12");
test["expect"]($$bc$str(launch_player.grounded)).toBe("false");
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), workbench["->InputConfiguration"](3, []), (result) => (() => { const _a = airborne_candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.requestAdmission)(started.value.session, airborne_candidate.value.candidate, (result) => (() => { const _a = airborne_admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const airborne_frame = wasm["decode-projected-term-frame"](airborne_admitted.value.frame);
test["expect"](((airborne_frame.player.position.y > 0.0) ? "true" : "false")).toBe("true");
(port.disposeSession)(started.value.session);
return null; }));

const generic_source_test_runtime = require("bun:test");
const register_generic_source_test = generic_source_test_runtime.test;
register_generic_source_test("real Wasm executes a generically allocated Clause source plan", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-generic-source-v1/generic-source-v1.cwr1.hex").text()]).then((assets) => { const port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
const request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[1]));
const accepted = ({value: null, watches: {}});
const started = ({value: null, watches: {}});
const candidate = ({value: null, watches: {}});
const admitted = ({value: null, watches: {}});
(port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), key_configuration(1, 1, "KeyD"), (result) => (() => { const _a = candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test["expect"](json_string(started.value.frame)).toBe("[]");
(port.requestAdmission)(started.value.session, candidate.value.candidate, (result) => (() => { const _a = admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const frame = wasm["decode-projected-term-frame"](admitted.value.frame);
const player = frame.player;
test["expect"](((player.position.x > 0.0) ? "true" : "false")).toBe("true");
(port.disposeSession)(started.value.session);
return null; }));

const coherent_test_runtime = require("bun:test");
const register_coherent_test = coherent_test_runtime.test;
register_coherent_test("one real Wasm session fails resets completes and launches from Clause source", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-coherent-game-v1/coherent-game-v1.cwr1.hex").text()]).then((assets) => { const port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
const request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[1]));
const accepted = ({value: null, watches: {}});
const started = ({value: null, watches: {}});
const failure_candidate = ({value: null, watches: {}});
const failure_admitted = ({value: null, watches: {}});
const reset_candidate = ({value: null, watches: {}});
const reset_admitted = ({value: null, watches: {}});
const completion_candidate = ({value: null, watches: {}});
const completion_admitted = ({value: null, watches: {}});
const launch_candidate = ({value: null, watches: {}});
const launch_admitted = ({value: null, watches: {}});
(port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), key_configuration(1, 1, "KeyS"), (result) => (() => { const _a = failure_candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test["expect"](json_string(started.value.frame)).toBe("[]");
(port.requestAdmission)(started.value.session, failure_candidate.value.candidate, (result) => (() => { const _a = failure_admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const failure_frame = wasm["decode-projected-term-frame"](failure_admitted.value.frame);
test["expect"]($$bc$str(failure_frame.world.objective.state)).toBe("failed");
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), two_key_configuration(2, 2, "KeyW", "KeyR"), (result) => (() => { const _a = reset_candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.requestAdmission)(started.value.session, reset_candidate.value.candidate, (result) => (() => { const _a = reset_admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const reset_frame = wasm["decode-projected-term-frame"](reset_admitted.value.frame);
test["expect"]($$bc$str(reset_frame.world.objective.state)).toBe("playing");
test["expect"]($$bc$str(reset_frame.player.position.z)).toBe("0");
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), key_configuration(4, 3, "KeyD"), (result) => (() => { const _a = completion_candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.requestAdmission)(started.value.session, completion_candidate.value.candidate, (result) => (() => { const _a = completion_admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const completion_frame = wasm["decode-projected-term-frame"](completion_admitted.value.frame);
const completed_world = completion_frame.world;
const completed_collectible = completed_world.collectibles[0];
test["expect"]($$bc$str(completed_world.objective.state)).toBe("completed");
test["expect"]($$bc$str(completed_collectible.state)).toBe("collected");
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), workbench["->InputConfiguration"](4, []), (result) => (() => { const _a = launch_candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.requestAdmission)(started.value.session, launch_candidate.value.candidate, (result) => (() => { const _a = launch_admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const launch_frame = wasm["decode-projected-term-frame"](launch_admitted.value.frame);
const launch_player = launch_frame.player;
test["expect"]($$bc$str(launch_player.velocity.y)).toBe("12");
test["expect"]($$bc$str(launch_player.grounded)).toBe("false");
test["expect"]($$bc$str(launch_frame.world.objective.state)).toBe("completed");
(port.disposeSession)(started.value.session);
return null; }));

const reload_test_runtime = require("bun:test");
const register_reload_test = reload_test_runtime.test;
register_reload_test("real Wasm workbench hot-reloads Clause-only collect behavior with fenced prior generation", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-gameplay-v1/gameplay-v1.cwr1.hex").text(), file("./fixtures/wasm-gameplay-v1/gameplay-spent-v1.cwr1.hex").text()]).then((assets) => { const raw_port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
const sessions = ({value: [], watches: {}});
const disposals = ({value: [], watches: {}});
const rendered = ({value: [], watches: {}});
const receipts = ({value: [], watches: {}});
const scheduled_tick = ({value: () => null, watches: {}});
const delay_next_candidate = ({value: false, watches: {}});
const delayed_candidate = ({value: null, watches: {}});
const changed_start_revision = ({value: null, watches: {}});
const tracked_port = workbench["->CartridgePort"](raw_port.acceptPackage, (accepted_package, generation, complete) => (raw_port.startSession)(accepted_package, generation, (result) => { sessions.value.push(result.session);
return complete(result); }), (session, fixed_tick, configuration, complete) => (raw_port.runCandidate)(session, fixed_tick, configuration, (result) => { if (delay_next_candidate.value) {
  (() => { const _a = delay_next_candidate, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  return (() => { const _a = delayed_candidate, _v = () => complete(result); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
} else {
  return complete(result);
} }), raw_port.requestAdmission, (session) => { disposals.value.push(session);
return (raw_port.disposeSession)(session); });
const base_request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[1]));
const changed_request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[2]));
const controller = workbench["create-cartridge-workbench!"](tracked_port, workbench["->FixedTick"](16), arena_policy(), (__milliseconds, callback) => { (() => { const _a = scheduled_tick, _v = callback; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return () => null; }, (frame) => rendered.value.push(frame), (receipt) => receipts.value.push(receipt), base_request);
const base_start = (controller.snapshot)();
test["expect"]($$bc$str(base_start.generation)).toBe("1");
test["expect"]($$bc$str(base_start.phase)).toBe("ready");
test["expect"](json_string(base_start.frame)).toBe("[]");
(controller.observeInput)(key_envelope("KeyD", "down"));
(scheduled_tick.value)();
const contact = wasm["decode-projected-term-frame"]((controller.snapshot)().frame);
const player_x = contact.player.position.x;
const contact_collectible = contact.world.collectibles[0];
test["expect"]($$bc$str(contact_collectible.state)).toBe("collected");
test["expect"]($$bc$str(player_x)).toBe($$bc$str(contact_collectible.position.x));
(controller.observeInput)(key_envelope("KeyD", "up"));
test["expect"]($$bc$str((controller.snapshot)().generation)).toBe("1");
const old_session = sessions.value[0];
(() => { const _a = delay_next_candidate, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(scheduled_tick.value)();
test["expect"]($$bc$str((controller.snapshot)().phase)).toBe("candidate");
test["expect"]($$bc$str((controller.reloadPackage)(changed_request))).toBe("true");
const changed_start = (controller.snapshot)();
const changed_revision = changed_start.revision;
(() => { const _a = changed_start_revision, _v = changed_revision; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
test["expect"]($$bc$str(changed_start.phase)).toBe("ready");
test["expect"]($$bc$str(changed_start.generation)).toBe("2");
test["expect"](json_string(changed_start.frame)).toBe("[]");
test["expect"]($$bc$str(sessions.value.length)).toBe("2");
test["expect"]($$bc$str(disposals.value.length)).toBe("1");
test["expect"]($$bc$str(Object.is(disposals.value[0], old_session))).toBe("true");
(delayed_candidate.value)();
const after_stale = (controller.snapshot)();
const stale_events = receipts.value.filter((receipt) => ($$bc$equiv(receipt.event, "completion-stale")));
test["expect"]($$bc$str(after_stale.generation)).toBe("2");
test["expect"]($$bc$str(Object.is(after_stale.revision, changed_revision))).toBe("true");
test["expect"](json_string(after_stale.frame)).toBe("[]");
test["expect"]($$bc$str(stale_events.length)).toBe("1");
const retired_result = ({value: null, watches: {}});
(raw_port.runCandidate)(old_session, workbench["->FixedTick"](16), workbench["->InputConfiguration"](0, []), (result) => (() => { const _a = retired_result, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test["expect"]($$bc$str(retired_result.value.reason)).toBe("Wasm session is disposed");
test["expect"]($$bc$str(Object.is((controller.snapshot)().revision, changed_revision))).toBe("true");
(controller.observeInput)(key_envelope("KeyD", "down"));
(scheduled_tick.value)();
const changed_admitted = (controller.snapshot)();
const changed_frame = wasm["decode-projected-term-frame"](changed_admitted.frame);
const changed_collectible = changed_frame.world.collectibles[0];
const rendered_length = rendered.value.length;
const visible_frame = wasm["decode-projected-term-frame"](rendered.value[(rendered_length - 1)]);
const visible_collectible = visible_frame.world.collectibles[0];
test["expect"]($$bc$str(changed_collectible.state)).toBe("spent");
test["expect"]($$bc$str(visible_collectible.state)).toBe("spent");
test["expect"]($$bc$str(changed_admitted.generation)).toBe("2");
test["expect"]($$bc$str((!(json_string(changed_admitted.revision) === json_string(changed_start_revision.value))))).toBe("true");
test["expect"]($$bc$str((controller.dispose)())).toBe("true");
test["expect"]($$bc$str(disposals.value.length)).toBe("2");
return null; }));

const dash_test_runtime = require("bun:test");
const register_dash_test = dash_test_runtime.test;
register_dash_test("real Wasm hot-reloads Clause dash jump through Admission and passive rendering", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-gameplay-v1/gameplay-v1.cwr1.hex").text(), file("./fixtures/wasm-gameplay-v1/gameplay-dash-jump-v1.cwr1.hex").text()]).then((assets) => { const port = wasm["create-wasm-cartridge-port"](initialize_real_session_module(assets[0]), arena_policy());
const rendered = ({value: [], watches: {}});
const scheduled_tick = ({value: () => null, watches: {}});
const base_request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[1]));
const dash_request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[2]));
const controller = workbench["create-cartridge-workbench!"](port, workbench["->FixedTick"](16), arena_policy(), (__milliseconds, callback) => { (() => { const _a = scheduled_tick, _v = callback; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return () => null; }, (frame) => rendered.value.push(frame), (__receipt) => null, base_request);
test["expect"]($$bc$str((controller.snapshot)().generation)).toBe("1");
test["expect"]($$bc$str((controller.reloadPackage)(dash_request))).toBe("true");
const dash_start = (controller.snapshot)();
const dash_revision = dash_start.revision;
test["expect"]($$bc$str(dash_start.generation)).toBe("2");
(controller.observeInput)(key_envelope("Space", "down"));
(scheduled_tick.value)();
const admitted = (controller.snapshot)();
const admitted_frame = wasm["decode-projected-term-frame"](admitted.frame);
const admitted_position = admitted_frame.player.position;
const rendered_length = rendered.value.length;
const visible_frame = wasm["decode-projected-term-frame"](rendered.value[(rendered_length - 1)]);
const visible_position = visible_frame.player.position;
test["expect"]($$bc$str(admitted.phase)).toBe("ready");
test["expect"]($$bc$str(admitted.generation)).toBe("2");
test["expect"]($$bc$str((!(json_string(admitted.revision) === json_string(dash_revision))))).toBe("true");
test["expect"](((admitted_position.x > 0.0) ? "true" : "false")).toBe("true");
test["expect"](((admitted_position.y > 0.0) ? "true" : "false")).toBe("true");
test["expect"]($$bc$str(visible_position.x)).toBe($$bc$str(admitted_position.x));
test["expect"]($$bc$str(visible_position.y)).toBe($$bc$str(admitted_position.y));
test["expect"]($$bc$str((controller.dispose)())).toBe("true");
return null; }));

const effect_test_runtime = require("bun:test");
const register_effect_test = effect_test_runtime.test;
register_effect_test("real Wasm transports one source-owned ongoing effect lifecycle", () => Promise.all([file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), file("./fixtures/wasm-ongoing-effect-v1/ongoing-effect-v1.cwr1.hex").text()]).then((assets) => { const module = initialize_real_session_module(assets[0]);
const port = wasm["create-wasm-cartridge-port"](module, policy());
const request = wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](assets[1]));
const accepted = ({value: null, watches: {}});
const started = ({value: null, watches: {}});
(port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
const session = started.value.session;
const absent = wasm["query-pending-effect-intent!"](module, session);
const __input = wasm["advance-session-occurrence!"](module, session, 0);
const intent = wasm["emit-effect-intent!"](module, session);
const queried = wasm["query-pending-effect-intent!"](module, session);
const issued = wasm["issue-effect-authorization!"](module, session, intent.intentId);
const attempt = wasm["begin-effect-attempt!"](module, session, issued.authorizationId);
const settled = wasm["settle-effect-attempt!"](module, session, attempt.attemptId, 202, [97, 99, 99, 101, 112, 116, 101, 100]);
const state_count = $$bc$str(absent.stateRevisionCount);
test["expect"]($$bc$str(absent.kind)).toBe("effect-intent-absent");
test["expect"](json_string(queried.intentId)).toBe(json_string(intent.intentId));
test["expect"](json_string(attempt.actionBytes)).toBe(json_string(intent.actionBytes));
test["expect"](json_string(attempt.resourceBytes)).toBe(json_string(intent.resourceBytes));
test["expect"](json_string(attempt.payloadBytes)).toBe(json_string(intent.payloadBytes));
test["expect"]($$bc$str(settled.disposition)).toBe("receipt-observed");
test["expect"](((settled.receiptId == null) ? "false" : "true")).toBe("true");
test["expect"](((settled.observationId == null) ? "false" : "true")).toBe("true");
test["expect"]($$bc$str(settled.stateRevisionCount)).toBe(state_count);
const second_intent = wasm["emit-effect-intent!"](module, session);
const second_issued = wasm["issue-effect-authorization!"](module, session, second_intent.intentId);
const second_attempt = wasm["begin-effect-attempt!"](module, session, second_issued.authorizationId);
const no_receipt = wasm["settle-effect-attempt!"](module, session, second_attempt.attemptId, null, null);
test["expect"]($$bc$str(no_receipt.disposition)).toBe("no-receipt");
test["expect"](((no_receipt.receiptId == null) ? "true" : "false")).toBe("true");
test["expect"](((no_receipt.observationId == null) ? "true" : "false")).toBe("true");
test["expect"]($$bc$str(no_receipt.stateRevisionCount)).toBe(state_count);
(port.disposeSession)(session);
return null; }));
//# sourceMappingURL=wasm-cartridge-port-test.js.map
