import * as wasm from "./wasm-cartridge-port.js";
import * as integration from "./wasm-workbench-shell.js";
import * as workbench from "./workbench.js";
import { BoxGeometry, Color, DirectionalLight, Group, HemisphereLight, Mesh, MeshStandardMaterial, PerspectiveCamera, Scene, WebGLRenderer, } from "three";
import { clause_session_v1_command_bulk, clause_session_v1_event_bulk, clause_session_v1_open_bulk, clause_session_v1_reclaim_retired, initSync, } from "#clause-runtime-wasm";
const module_path = "./generated/wasm/clause_runtime_bg.wasm";
const request_path = "./fixtures/wasm-jump-v1/jump-v1.cwr1.hex";
function demo_policy() {
    return workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"], workbench["->WorkbenchSequenceLimits"](64, 16, 8, 8, 16));
}
async function fetch_text_bang(path) {
    const response = await fetch(path);
    if (!response.ok) {
        throw new Error(`Unable to load ${path}`);
    }
    return response.text();
}
async function fetch_bytes_bang(path) {
    const response = await fetch(path);
    if (!response.ok) {
        throw new Error(`Unable to load ${path}`);
    }
    return response.arrayBuffer();
}
function initialize_session_module(module) {
    initSync(module);
    return Object.freeze({
        clause_session_v1_open_bulk: (request) => clause_session_v1_open_bulk(new Uint8Array(request)),
        clause_session_v1_command_bulk: (request) => clause_session_v1_command_bulk(new Uint8Array(request)),
        clause_session_v1_event_bulk: () => clause_session_v1_event_bulk(),
        clause_session_v1_reclaim_retired: () => clause_session_v1_reclaim_retired(),
    });
}
function require_html_element(selector) {
    const element = document.querySelector(selector);
    if (!(element instanceof HTMLElement)) {
        throw new Error(`Missing HTML element ${selector}`);
    }
    return element;
}
const arena = require_html_element("#arena");
const status = require_html_element("#arena-status");
let active_composition = null;
const three_module = Object.freeze({
    BoxGeometry,
    Color,
    DirectionalLight,
    Group,
    HemisphereLight,
    Mesh,
    MeshStandardMaterial,
    PerspectiveCamera,
    Scene,
    WebGLRenderer,
});
async function load_demo_bang() {
    const module_request = fetch_bytes_bang(module_path);
    const fixture_request = fetch_text_bang(request_path);
    const module_bytes = await module_request;
    const fixture_text = await fixture_request;
    const module = initialize_session_module(module_bytes);
    const composition = integration["create-passive-wasm-workbench-shell!"](arena, window, three_module, module, wasm["decode-cwr1-hex"](fixture_text), workbench["->FixedTick"](16), demo_policy(), () => null);
    active_composition = composition;
    status.textContent =
        "Ready — click the arena, move with A/D, and jump with Space.";
    return null;
}
const load_demo = load_demo_bang().catch(() => {
    status.textContent = "Arena load failed safely.";
    return null;
});
window.addEventListener("beforeunload", () => {
    active_composition?.dispose();
}, { once: true });
//# sourceMappingURL=demo.js.map