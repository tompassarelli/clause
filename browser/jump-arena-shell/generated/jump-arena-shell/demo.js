import * as wasm from "./wasm-cartridge-port.js";
import * as integration from "./wasm-workbench-shell.js";
import * as workbench from "./workbench.js";
import { "BoxGeometry" as BoxGeometry, "Color" as Color, "DirectionalLight" as DirectionalLight, "Group" as Group, "HemisphereLight" as HemisphereLight, "Mesh" as Mesh, "MeshStandardMaterial" as MeshStandardMaterial, "PerspectiveCamera" as PerspectiveCamera, "Scene" as Scene, "WebGLRenderer" as WebGLRenderer } from "three";
import { "clause_session_v1_command" as clause__session__v1__command, "clause_session_v1_event_byte" as clause__session__v1__event__byte, "clause_session_v1_event_len" as clause__session__v1__event__len, "clause_session_v1_io_reset" as clause__session__v1__io__reset, "clause_session_v1_open" as clause__session__v1__open, "clause_session_v1_request_push" as clause__session__v1__request__push, "initSync" as initSync } from "#clause-runtime-wasm";
import { keyword as $$bc$keyword, property_key as $$bc$property_key, str as $$bc$str } from 'beagle/core.js';

const module_path = "./generated/wasm/clause_runtime_bg.wasm";

const request_path = "./fixtures/wasm-jump-v1/jump-v1.cwr1.hex";

function demo_policy() {
  return workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"], workbench["->WorkbenchSequenceLimits"](64, 16, 8, 8, 16));
}

function fetch_text_bang(path) {
  return fetch(path).then((response) => (((_truthy) => _truthy !== false && _truthy != null)(response.ok) ? response.text() : Promise.reject(new Error($$bc$str("Unable to load ", path)))));
}

function fetch_bytes_bang(path) {
  return fetch(path).then((response) => (((_truthy) => _truthy !== false && _truthy != null)(response.ok) ? response.arrayBuffer() : Promise.reject(new Error($$bc$str("Unable to load ", path)))));
}

function initialize_session_module(module) {
  const input = module;
  const __initialized = initSync(input);
  return Object.freeze({[$$bc$property_key($$bc$keyword("clause_session_v1_io_reset"))]: () => clause__session__v1__io__reset(), [$$bc$property_key($$bc$keyword("clause_session_v1_request_push"))]: (byte) => clause__session__v1__request__push(byte), [$$bc$property_key($$bc$keyword("clause_session_v1_open"))]: () => clause__session__v1__open(), [$$bc$property_key($$bc$keyword("clause_session_v1_command"))]: () => clause__session__v1__command(), [$$bc$property_key($$bc$keyword("clause_session_v1_event_len"))]: () => clause__session__v1__event__len(), [$$bc$property_key($$bc$keyword("clause_session_v1_event_byte"))]: (index) => clause__session__v1__event__byte(index)});
}

const arena = document.querySelector("#arena");

const status = document.querySelector("#arena-status");

const active_composition = ({value: null, watches: {}});

const three_module = Object.freeze({[$$bc$property_key($$bc$keyword("BoxGeometry"))]: BoxGeometry, [$$bc$property_key($$bc$keyword("Color"))]: Color, [$$bc$property_key($$bc$keyword("DirectionalLight"))]: DirectionalLight, [$$bc$property_key($$bc$keyword("Group"))]: Group, [$$bc$property_key($$bc$keyword("HemisphereLight"))]: HemisphereLight, [$$bc$property_key($$bc$keyword("Mesh"))]: Mesh, [$$bc$property_key($$bc$keyword("MeshStandardMaterial"))]: MeshStandardMaterial, [$$bc$property_key($$bc$keyword("PerspectiveCamera"))]: PerspectiveCamera, [$$bc$property_key($$bc$keyword("Scene"))]: Scene, [$$bc$property_key($$bc$keyword("WebGLRenderer"))]: WebGLRenderer});

const load_demo = Promise.all([fetch_bytes_bang(module_path), fetch_text_bang(request_path)]).then((assets) => { const module = initialize_session_module(assets[0]);
const composition = integration["create-passive-wasm-workbench-shell!"](arena, window, three_module, module, wasm["decode-cwr1-hex"](assets[1]), workbench["->FixedTick"](16), demo_policy(), (__receipt) => null);
(() => { const _a = active_composition, _v = composition; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(status.textContent = "Ready — click the arena, move with A/D, and jump with Space.");
return composition; }).catch((__error) => { (status.textContent = "Arena load failed safely.");
return null; });

window.addEventListener("beforeunload", () => { const composition = active_composition.value;
if ((!(composition == null))) {
  return (composition.dispose)();
} }, {[$$bc$property_key($$bc$keyword("once"))]: true});
//# sourceMappingURL=demo.js.map
