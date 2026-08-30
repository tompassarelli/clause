import * as wasm from './wasm-cartridge-port.js';
import * as integration from './wasm-workbench-shell.js';
import * as workbench from './workbench.js';
import { initSync } from '#clause-runtime-wasm';
import { keyword as $$bc$keyword, property_key as $$bc$property_key, str as $$bc$str } from 'beagle/core.js';

const module_path = "./generated/wasm/clause_runtime_bg.wasm";

const request_path = "./fixtures/wasm-jump-v1/jump-v1.cwr1.hex";

function demo_policy() {
  return workbench["->WorkbenchPolicy"](8, 8, 32, 128, 4096, workbench["->WorkbenchSequenceLimits"](64, 16, 8, 8, 16));
}

function fetch_text_bang(path) {
  return fetch(path).then((response) => (((_truthy) => _truthy !== false && _truthy != null)(response.ok) ? response.text() : Promise.reject(new Error($$bc$str("Unable to load ", path)))));
}

function fetch_bytes_bang(path) {
  return fetch(path).then((response) => (((_truthy) => _truthy !== false && _truthy != null)(response.ok) ? response.arrayBuffer() : Promise.reject(new Error($$bc$str("Unable to load ", path)))));
}

const status = document.querySelector("#replay-status");

const result = document.querySelector("#replay-result");

const active_composition = ({value: null, watches: {}});

function create_result_shell_bang(__emit_input) {
  const disposed = ({value: false, watches: {}});
  return Object.freeze({[$$bc$property_key($$bc$keyword("renderFrame"))]: (values) => { if (((_truthy) => _truthy !== false && _truthy != null)(disposed.value)) {
  return (() => { throw new Error("replay result shell is disposed"); })();
} else {
  (status.textContent = "Separate Admission accepted; exact scalar result:");
  (result.textContent = JSON.stringify(values));
  return values;
} }, [$$bc$property_key($$bc$keyword("dispose"))]: () => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  return (() => { const _a = disposed, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
} }});
}

const load_demo = Promise.all([fetch_bytes_bang(module_path), fetch_text_bang(request_path)]).then((assets) => integration["load-passive-wasm-workbench-shell!"](integration["create-sync-wasm-initializer"](initSync), assets[0], create_result_shell_bang, integration["project-nonempty-cwo1-result-frame"], window, wasm["decode-cwr1-hex"](assets[1]), workbench["->FixedTick"](16), demo_policy(), (__receipt) => null)).then((composition) => { (() => { const _a = active_composition, _v = composition; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(status.textContent = "Exact replay loaded; awaiting separate Admission.");
return composition; }).catch((__error) => { (status.textContent = "Replay load failed safely.");
(result.textContent = "");
return null; });

window.addEventListener("beforeunload", () => { const composition = active_composition.value;
if ((!(composition == null))) {
  return (composition.dispose)();
} }, {[$$bc$property_key($$bc$keyword("once"))]: true});
//# sourceMappingURL=demo.js.map
