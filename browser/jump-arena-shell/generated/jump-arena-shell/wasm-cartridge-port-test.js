import * as wasm from './wasm-cartridge-port.js';
import * as workbench from './workbench.js';
import * as test from 'bun:test';
import { equivV as $$bc$equiv, keyword as $$bc$keyword, property_key as $$bc$property_key } from 'beagle/core.js';

function policy() {
  const maximum = Number.MAX_SAFE_INTEGER;
  return workbench["->WorkbenchPolicy"](8, 8, 32, 128, 512, workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
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

function module_for(response, calls) {
  return {[$$bc$property_key($$bc$keyword("clause_process_v1_reset"))]: () => calls.push("reset"), [$$bc$property_key($$bc$keyword("clause_process_v1_request_push"))]: (byte) => { calls.push(byte);
return 0; }, [$$bc$property_key($$bc$keyword("clause_process_v1_dispatch"))]: () => { calls.push("dispatch");
return 0; }, [$$bc$property_key($$bc$keyword("clause_process_v1_response_len"))]: () => response.length, [$$bc$property_key($$bc$keyword("clause_process_v1_response_byte"))]: (index) => response[index]};
}

test.test("candidate stays opaque and Wasm dispatch occurs only at admission", () => { const response = cwo1([{[$$bc$property_key($$bc$keyword("kind"))]: "number", [$$bc$property_key($$bc$keyword("value"))]: 2.5}, {[$$bc$property_key($$bc$keyword("kind"))]: "boolean", [$$bc$property_key($$bc$keyword("value"))]: true}]);
const calls = [];
const port = wasm["create-wasm-cartridge-port"](module_for(response, calls), policy());
const request = wasm["->ExactProcessRequest"]([67, 87, 82, 49, 0]);
const accepted = ({value: null, watches: {}});
const started = ({value: null, watches: {}});
const candidate = ({value: null, watches: {}});
const admitted = ({value: null, watches: {}});
(port.acceptPackage)(request, (result) => (() => { const _a = accepted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.startSession)(accepted.value.acceptedPackage, 1, (result) => (() => { const _a = started, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
(port.runCandidate)(started.value.session, workbench["->FixedTick"](16), workbench["->InputConfiguration"](0, []), (result) => (() => { const _a = candidate, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test.expect(calls).toEqual([]);
test.expect(candidate.value.candidate.request.bytes).toEqual(request.bytes);
test.expect(Object.isFrozen(candidate.value.candidate.request.bytes)).toBe(true);
test.expect(started.value.revision).toBe(null);
test.expect(started.value.frame).toEqual([]);
(port.requestAdmission)(started.value.session, candidate.value.candidate, (result) => (() => { const _a = admitted, _v = result; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })());
test.expect(calls[0]).toBe("reset");
test.expect(calls[6]).toBe("dispatch");
test.expect(admitted.value.revision).toEqual(identity(22));
return test.expect(admitted.value.frame).toEqual([2.5, true]); });

test.test("strict CWO1 decoding rejects malformed and trailing bytes", () => { const valid = cwo1([{[$$bc$property_key($$bc$keyword("kind"))]: "boolean", [$$bc$property_key($$bc$keyword("value"))]: false}]);
test.expect(() => wasm["decode-cwo1-observation"](valid.concat([0]))).toThrow();
const bad = valid.slice();
bad.splice(0, 1, 88);
return test.expect(() => wasm["decode-cwo1-observation"](bad)).toThrow(); });
//# sourceMappingURL=wasm-cartridge-port-test.js.map
