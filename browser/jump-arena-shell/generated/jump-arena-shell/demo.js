import * as wasm from "./wasm-cartridge-port.js";
import * as integration from "./wasm-workbench-shell.js";
import * as workbench from "./workbench.js";
import { "BoxGeometry" as BoxGeometry, "Color" as Color, "DirectionalLight" as DirectionalLight, "Group" as Group, "HemisphereLight" as HemisphereLight, "Mesh" as Mesh, "MeshStandardMaterial" as MeshStandardMaterial, "PerspectiveCamera" as PerspectiveCamera, "Scene" as Scene, "WebGLRenderer" as WebGLRenderer } from "three";
import { "clause_session_v1_command_bulk" as clause__session__v1__command__bulk, "clause_session_v1_event_bulk" as clause__session__v1__event__bulk, "clause_session_v1_open_bulk" as clause__session__v1__open__bulk, "clause_session_v1_reclaim_retired" as clause__session__v1__reclaim__retired, "initSync" as initSync } from "#clause-runtime-wasm";
import { keyword as $$bc$keyword, property_key as $$bc$property_key, str as $$bc$str } from 'beagle/core.js';

const module_path = "./generated/wasm/clause_runtime_bg.wasm";

const request_path = "./fixtures/wasm-jump-v1/jump-v1.cwr1.hex";

function demo_policy() {
  return workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"], workbench["->WorkbenchSequenceLimits"](64, 16, 8, 8, 16));
}

async function fetch_text_bang(path) {
  const response = await fetch(path);
  return (response.ok ? await response.text() : (() => { throw new Error($$bc$str("Unable to load ", path)); })());
}

async function fetch_bytes_bang(path) {
  const response = await fetch(path);
  return (response.ok ? await response.arrayBuffer() : (() => { throw new Error($$bc$str("Unable to load ", path)); })());
}

function initialize_session_module(module) {
  const input = module;
  const __initialized = initSync(input);
  return Object.freeze({[$$bc$property_key($$bc$keyword("clause_session_v1_open_bulk"))]: (request) => clause__session__v1__open__bulk(new Uint8Array(request)), [$$bc$property_key($$bc$keyword("clause_session_v1_command_bulk"))]: (request) => clause__session__v1__command__bulk(new Uint8Array(request)), [$$bc$property_key($$bc$keyword("clause_session_v1_event_bulk"))]: () => clause__session__v1__event__bulk(), [$$bc$property_key($$bc$keyword("clause_session_v1_reclaim_retired"))]: () => clause__session__v1__reclaim__retired()});
}

const arena = document.querySelector("#arena");

const status = document.querySelector("#arena-status");

const active_composition = ({value: null, watches: {}});

const three_module = Object.freeze({[$$bc$property_key($$bc$keyword("BoxGeometry"))]: BoxGeometry, [$$bc$property_key($$bc$keyword("Color"))]: Color, [$$bc$property_key($$bc$keyword("DirectionalLight"))]: DirectionalLight, [$$bc$property_key($$bc$keyword("Group"))]: Group, [$$bc$property_key($$bc$keyword("HemisphereLight"))]: HemisphereLight, [$$bc$property_key($$bc$keyword("Mesh"))]: Mesh, [$$bc$property_key($$bc$keyword("MeshStandardMaterial"))]: MeshStandardMaterial, [$$bc$property_key($$bc$keyword("PerspectiveCamera"))]: PerspectiveCamera, [$$bc$property_key($$bc$keyword("Scene"))]: Scene, [$$bc$property_key($$bc$keyword("WebGLRenderer"))]: WebGLRenderer});

async function load_demo_bang() {
  const module_request = fetch_bytes_bang(module_path);
  const fixture_request = fetch_text_bang(request_path);
  const module_bytes = await module_request;
  const fixture_text = await fixture_request;
  const module = initialize_session_module(module_bytes);
  const composition = integration["create-passive-wasm-workbench-shell!"](arena, window, three_module, module, wasm["decode-cwr1-hex"](fixture_text), workbench["->FixedTick"](16), demo_policy(), (__receipt) => null);
  (() => { const _a = active_composition, _v = composition; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (status.textContent = "Ready — click the arena, move with A/D, and jump with Space.");
  return null;
}

const load_demo = load_demo_bang().catch((__error) => { (status.textContent = "Arena load failed safely.");
return null; });

window.addEventListener("beforeunload", () => { const composition = active_composition.value;
if ((!(composition == null))) {
  return (composition.dispose)();
} }, {[$$bc$property_key($$bc$keyword("once"))]: true});
//# sourceMappingURL=demo.js.map
