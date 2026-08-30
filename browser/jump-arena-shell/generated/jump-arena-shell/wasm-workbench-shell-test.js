import * as integration from './wasm-workbench-shell.js';
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
  const bytes = [67, 87, 79, 49].concat(identity(11), identity(22));
  const count = values.length;
  bytes.push((count % 256));
  bytes.push(Math.trunc(count / 256));
  values.forEach((value) => {
  if (($$bc$equiv(typeof value, "boolean"))) {
    bytes.push(1);
    bytes.push((((_truthy) => _truthy !== false && _truthy != null)(value) ? 1 : 0));
  } else {
    const packed = new ArrayBuffer(8);
    const view = new DataView(packed);
    view.setFloat64(0, value, true);
    bytes.push(0);
    new Uint8Array(packed).forEach((octet) => {
  bytes.push(octet);
});
  }
});
  return bytes;
}

function module_for_bang(response, dispatches) {
  return {[$$bc$property_key($$bc$keyword("clause_process_v1_reset"))]: () => null, [$$bc$property_key($$bc$keyword("clause_process_v1_request_push"))]: (__byte) => 0, [$$bc$property_key($$bc$keyword("clause_process_v1_dispatch"))]: () => { (() => { const _a = dispatches; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
return 0; }, [$$bc$property_key($$bc$keyword("clause_process_v1_response_len"))]: () => response.length, [$$bc$property_key($$bc$keyword("clause_process_v1_response_byte"))]: (index) => response[index]};
}

test.test("passive shell renders only the admitted CWO1 successor frame", () => { const rendered = [];
const disposed = ({value: 0, watches: {}});
const emitted_input = ({value: null, watches: {}});
const scheduled = ({value: null, watches: {}});
const cancelled = ({value: 0, watches: {}});
const dispatches = ({value: 0, watches: {}});
const receipts = [];
const browser = {[$$bc$property_key($$bc$keyword("setTimeout"))]: (tick, __delay) => { (() => { const _a = scheduled, _v = tick; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return 41; }, [$$bc$property_key($$bc$keyword("clearTimeout"))]: (__token) => (() => { const _a = cancelled; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })()};
const values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 0.25, true, 0.0, -0.25, 0.0, 12.0, 0.5, 12.0];
const composition = integration["create-passive-wasm-workbench-with-shell-factory"]((emit_input) => { (() => { const _a = emitted_input, _v = emit_input; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return {[$$bc$property_key($$bc$keyword("renderFrame"))]: (frame) => rendered.push(frame), [$$bc$property_key($$bc$keyword("dispose"))]: () => (() => { const _a = disposed; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })()}; }, browser, module_for_bang(cwo1(values), dispatches), [67, 87, 82, 49, 1], workbench["->FixedTick"](16), policy(), (receipt) => receipts.push(receipt));
test.expect(rendered).toEqual([]);
test.expect(dispatches.value).toBe(0);
test.expect((composition.controller.snapshot)().phase).toBe("ready");
(emitted_input.value)({[$$bc$property_key($$bc$keyword("kind"))]: "keyboard", [$$bc$property_key($$bc$keyword("phase"))]: "down", [$$bc$property_key($$bc$keyword("code"))]: "Space", [$$bc$property_key($$bc$keyword("repeat"))]: false});
test.expect(dispatches.value).toBe(0);
(scheduled.value)();
test.expect(dispatches.value).toBe(1);
test.expect(rendered.length).toBe(1);
const frame = rendered[0];
test.expect([frame.player.position.x, frame.player.position.y, frame.player.position.z]).toEqual([1, 2, 3]);
test.expect(frame.player.grounded).toBe(true);
test.expect(frame.world.platforms.length).toBe(1);
(composition.dispose)();
(composition.dispose)();
test.expect(disposed.value).toBe(1);
test.expect(cancelled.value).toBe(1);
(scheduled.value)();
return test.expect(rendered.length).toBe(1); });

test.test("browser loader injects bounded module bytes before composition", () => { const received = ({value: null, watches: {}});
const initializer = (input) => { (() => { const _a = received, _v = input; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return Promise.reject(new Error("bounded loader probe")); };
const module_bytes = new Uint8Array([0, 97, 115, 109]);
return integration["load-passive-wasm-workbench-shell!"](initializer, module_bytes, null, {[$$bc$property_key($$bc$keyword("setTimeout"))]: (__tick, __delay) => 1, [$$bc$property_key($$bc$keyword("clearTimeout"))]: (__token) => null}, null, [67, 87, 82, 49, 1], workbench["->FixedTick"](16), policy(), (__receipt) => null).catch((__error) => test.expect(received.value.module_or_path).toBe(module_bytes)); });
//# sourceMappingURL=wasm-workbench-shell-test.js.map
