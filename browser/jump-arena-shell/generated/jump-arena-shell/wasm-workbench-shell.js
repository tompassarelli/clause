import * as shell from './shell.js';
import * as wasm from './wasm-cartridge-port.js';
import * as workbench from './workbench.js';
import { conj_value as $$bc$conj_value, equivV as $$bc$equiv, keyword as $$bc$keyword, property_key as $$bc$property_key } from 'beagle/core.js';

const wasm_module_max_bytes = (16 * 1024 * 1024);

const player_frame_values = 8;

const platform_frame_values = 6;

function exact_module_bytes_p(value) {
  if ((value == null)) {
    return false;
  } else {
    const length = value.byteLength;
    return ((_logical) => (_logical !== false && _logical != null ? ((_logical) => (_logical !== false && _logical != null ? ((1 <= length) && (length <= wasm_module_max_bytes)) : _logical))(Number.isSafeInteger(length)) : _logical))(((value instanceof ArrayBuffer) || ArrayBuffer.isView(value)));
  }
}

function require_module_bytes(value) {
  return (exact_module_bytes_p(value) ? value : (() => { throw new Error("Wasm module bytes are outside the browser bound"); })());
}

function create_sync_wasm_initializer(initialize_sync) {
  return (input) => Promise.resolve(initialize_sync({[$$bc$property_key($$bc$keyword("module"))]: input.module_or_path}));
}

function numeric_frame_value(values, index) {
  const value = values[index];
  return (((_truthy) => _truthy !== false && _truthy != null)((($$bc$equiv(typeof value, "number")) && Number.isFinite(value))) ? value : (() => { throw new Error("CWO1 arena frame requires finite numeric slots"); })());
}

function boolean_frame_value(values, index) {
  const value = values[index];
  return (($$bc$equiv(typeof value, "boolean")) ? value : (() => { throw new Error("CWO1 arena frame requires a boolean grounded slot"); })());
}

function frozen_vec3(x, y, z) {
  return Object.freeze({[$$bc$property_key($$bc$keyword("x"))]: x, [$$bc$property_key($$bc$keyword("y"))]: y, [$$bc$property_key($$bc$keyword("z"))]: z});
}

function project_platform(values, offset) {
  return Object.freeze({[$$bc$property_key($$bc$keyword("position"))]: frozen_vec3(numeric_frame_value(values, offset), numeric_frame_value(values, (offset + 1)), numeric_frame_value(values, (offset + 2))), [$$bc$property_key($$bc$keyword("size"))]: frozen_vec3(numeric_frame_value(values, (offset + 3)), numeric_frame_value(values, (offset + 4)), numeric_frame_value(values, (offset + 5)))});
}

function project_cwo1_arena_frame(values) {
  const length = values.length;
  if (((!((_truthy) => _truthy !== false && _truthy != null)(Array.isArray(values))) || ((!((_truthy) => _truthy !== false && _truthy != null)(Number.isSafeInteger(length))) || ((length < player_frame_values) || (!($$bc$equiv(((length - player_frame_values) % platform_frame_values), 0))))))) {
    (() => { throw new Error("CWO1 arena frame has an invalid scalar shape"); })();
  }
  return (() => { let offset = player_frame_values; let platforms = []; while (true) {
    if ((offset === length)) { return Object.freeze({[$$bc$property_key($$bc$keyword("player"))]: Object.freeze({[$$bc$property_key($$bc$keyword("position"))]: frozen_vec3(numeric_frame_value(values, 0), numeric_frame_value(values, 1), numeric_frame_value(values, 2)), [$$bc$property_key($$bc$keyword("velocity"))]: frozen_vec3(numeric_frame_value(values, 3), numeric_frame_value(values, 4), numeric_frame_value(values, 5)), [$$bc$property_key($$bc$keyword("yaw"))]: numeric_frame_value(values, 6), [$$bc$property_key($$bc$keyword("grounded"))]: boolean_frame_value(values, 7)}), [$$bc$property_key($$bc$keyword("world"))]: Object.freeze({[$$bc$property_key($$bc$keyword("platforms"))]: Object.freeze(platforms)})}); } else { const _recur_0 = (offset + platform_frame_values); const _recur_1 = $$bc$conj_value(platforms, project_platform(values, offset)); offset = _recur_0; platforms = _recur_1; continue; }
  } })();
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
  return create_composition_bang((emit_input) => shell["create-jump-arena-shell!"](mount, browser, three, emit_input), project_cwo1_arena_frame, browser, module, exact_request_bytes, fixed_tick, policy, emit_receipt);
}

function load_passive_wasm_workbench_shell_bang(initialize_module, exact_module_bytes, create_shell, project_frame, browser, exact_request_bytes, fixed_tick, policy, emit_receipt) {
  return initialize_module({[$$bc$property_key($$bc$keyword("module_or_path"))]: require_module_bytes(exact_module_bytes)}).then((module) => create_composition_bang(create_shell, project_frame, browser, module, exact_request_bytes, fixed_tick, policy, emit_receipt));
}

const create_passive_wasm_workbench_with_shell_factory = create_composition_bang;

export { create_passive_wasm_workbench_shell_bang as "create-passive-wasm-workbench-shell!" };
export { create_passive_wasm_workbench_with_shell_factory as "create-passive-wasm-workbench-with-shell-factory" };
export { create_sync_wasm_initializer as "create-sync-wasm-initializer" };
export { load_passive_wasm_workbench_shell_bang as "load-passive-wasm-workbench-shell!" };
export { project_cwo1_arena_frame as "project-cwo1-arena-frame" };
export { project_nonempty_cwo1_result_frame as "project-nonempty-cwo1-result-frame" };
//# sourceMappingURL=wasm-workbench-shell.js.map
