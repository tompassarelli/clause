import * as shell from "./shell.js";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import { keyword as $$bc$keyword, property_key as $$bc$property_key } from 'beagle/core.js';

const wasm_module_max_bytes = (16 * 1024 * 1024);

function exact_module_bytes_p(value) {
  if ((value == null)) {
    return false;
  } else {
    const length = value.byteLength;
    return ((value instanceof ArrayBuffer) && ((_logical) => (_logical !== false && _logical != null ? ((1 <= length) && (length <= wasm_module_max_bytes)) : _logical))(Number.isSafeInteger(length)));
  }
}

function require_module_bytes(value) {
  return (exact_module_bytes_p(value) ? value : (() => { throw new Error("Wasm module bytes are outside the browser bound"); })());
}

function create_sync_wasm_initializer(initialize_sync) {
  return (input) => Promise.resolve(initialize_sync(require_module_bytes(input.module_or_path)));
}

function project_persistent_term_frame(term_bytes) {
  return wasm["decode-projected-term-frame"](term_bytes);
}

function project_nonempty_cwo1_result_frame(values) {
  return (((_truthy) => _truthy !== false && _truthy != null)(((_logical) => (_logical !== false && _logical != null ? (values.length > 0) : _logical))(Array.isArray(values))) ? values : (() => { throw new Error("CWO1 result view awaits an admitted scalar projection"); })());
}

function schedule_fixed_tick(browser, delay, tick) {
  const token = browser.setTimeout(tick, delay);
  return () => browser.clearTimeout(token);
}

function create_composition_bang(create_shell, project_frame, browser, module, exact_request_bytes, fixed_tick, policy, emit_receipt) {
  const controller = ({value: null, watches: {}});
  const passive_shell = create_shell((input) => { const active = controller.value;
if ((!(active == null))) {
  return (active.observeInput)(workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(input)])));
} });
  const port = wasm["create-wasm-cartridge-port"](module, policy);
  const active = workbench["create-cartridge-workbench!"](port, fixed_tick, policy, (delay, tick) => schedule_fixed_tick(browser, delay, tick), (values) => (passive_shell.renderFrame)(project_frame(values)), emit_receipt, wasm["->ExactProcessRequest"](exact_request_bytes));
  const disposed = ({value: false, watches: {}});
  const dispose = () => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  (() => { const _a = disposed, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (active.dispose)();
  return (passive_shell.dispose)();
} };
  (() => { const _a = controller, _v = active; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  return Object.freeze({[$$bc$property_key($$bc$keyword("controller"))]: active, [$$bc$property_key($$bc$keyword("shell"))]: passive_shell, [$$bc$property_key($$bc$keyword("dispose"))]: dispose});
}

function create_passive_wasm_workbench_shell_bang(mount, browser, three, module, exact_request_bytes, fixed_tick, policy, emit_receipt) {
  return create_composition_bang((emit_input) => shell["create-jump-arena-shell!"](mount, browser, three, emit_input), project_persistent_term_frame, browser, module, exact_request_bytes, fixed_tick, policy, emit_receipt);
}

function load_passive_wasm_workbench_shell_bang(initialize_module, exact_module_bytes, create_shell, project_frame, browser, exact_request_bytes, fixed_tick, policy, emit_receipt) {
  return initialize_module({[$$bc$property_key($$bc$keyword("module_or_path"))]: require_module_bytes(exact_module_bytes)}).then((module) => create_composition_bang(create_shell, project_frame, browser, module, exact_request_bytes, fixed_tick, policy, emit_receipt));
}

const create_passive_wasm_workbench_with_shell_factory = create_composition_bang;

export { create_passive_wasm_workbench_shell_bang as "create-passive-wasm-workbench-shell!" };
export { create_passive_wasm_workbench_with_shell_factory as "create-passive-wasm-workbench-with-shell-factory" };
export { create_sync_wasm_initializer as "create-sync-wasm-initializer" };
export { load_passive_wasm_workbench_shell_bang as "load-passive-wasm-workbench-shell!" };
export { project_nonempty_cwo1_result_frame as "project-nonempty-cwo1-result-frame" };
export { project_persistent_term_frame as "project-persistent-term-frame" };
//# sourceMappingURL=wasm-workbench-shell.js.map
