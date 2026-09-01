import * as shell from "./shell.js";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
const wasm_module_max_bytes = 16 * 1024 * 1024;
function exact_module_bytes_p(value) {
    return (value instanceof ArrayBuffer &&
        Number.isSafeInteger(value.byteLength) &&
        value.byteLength >= 1 &&
        value.byteLength <= wasm_module_max_bytes);
}
function require_module_bytes(value) {
    if (!exact_module_bytes_p(value)) {
        throw new Error("Wasm module bytes are outside the browser bound");
    }
    return value;
}
function module_or_path(input) {
    if (typeof input !== "object" ||
        input === null ||
        !("module_or_path" in input)) {
        throw new Error("Wasm initialization input lacks module_or_path");
    }
    return input.module_or_path;
}
function create_sync_wasm_initializer(initialize_sync) {
    return (input) => Promise.resolve(initialize_sync(require_module_bytes(module_or_path(input))));
}
function project_persistent_term_frame(term_bytes) {
    return wasm["decode-projected-term-frame"](term_bytes);
}
function project_nonempty_cwo1_result_frame(values) {
    if (!Array.isArray(values) || values.length === 0) {
        throw new Error("CWO1 result view awaits an admitted scalar projection");
    }
    return values;
}
function schedule_fixed_tick(browser, delay, tick) {
    const token = browser.setTimeout(tick, delay);
    return () => browser.clearTimeout(token);
}
function encode_input(input) {
    const source = JSON.stringify([JSON.stringify(input)]);
    if (typeof source !== "string") {
        throw new Error("browser input is not JSON-encodable");
    }
    return source;
}
function create_composition_bang(create_shell, project_frame, browser, module, exact_request_bytes, fixed_tick, policy, emit_receipt) {
    let controller = null;
    const passive_shell = create_shell((input) => {
        const active = controller;
        if (active !== null) {
            return active.observeInput(workbench["create-workbench-envelope"](policy, encode_input(input)));
        }
        return undefined;
    });
    const port = wasm["create-wasm-cartridge-port"](module, policy);
    const active = workbench["create-cartridge-workbench!"](port, fixed_tick, policy, (delay, tick) => schedule_fixed_tick(browser, delay, tick), (values) => passive_shell.renderFrame(project_frame(values)), emit_receipt, wasm["->ExactProcessRequest"](exact_request_bytes));
    let disposed = false;
    const dispose = () => {
        if (disposed) {
            return undefined;
        }
        disposed = true;
        active.dispose();
        return passive_shell.dispose();
    };
    controller = active;
    return Object.freeze({
        controller: active,
        shell: passive_shell,
        dispose,
    });
}
function create_passive_wasm_workbench_shell_bang(mount, browser, three, module, exact_request_bytes, fixed_tick, policy, emit_receipt) {
    return create_composition_bang((emit_input) => shell["create-jump-arena-shell!"](mount, browser, three, emit_input), project_persistent_term_frame, browser, module, exact_request_bytes, fixed_tick, policy, emit_receipt);
}
function load_passive_wasm_workbench_shell_bang(initialize_module, exact_module_bytes, create_shell, project_frame, browser, exact_request_bytes, fixed_tick, policy, emit_receipt) {
    return initialize_module({
        module_or_path: require_module_bytes(exact_module_bytes),
    }).then((module) => create_composition_bang(create_shell, project_frame, browser, module, exact_request_bytes, fixed_tick, policy, emit_receipt));
}
const create_passive_wasm_workbench_with_shell_factory = create_composition_bang;
export { create_passive_wasm_workbench_shell_bang as "create-passive-wasm-workbench-shell!" };
export { create_passive_wasm_workbench_with_shell_factory as "create-passive-wasm-workbench-with-shell-factory" };
export { create_sync_wasm_initializer as "create-sync-wasm-initializer" };
export { load_passive_wasm_workbench_shell_bang as "load-passive-wasm-workbench-shell!" };
export { project_nonempty_cwo1_result_frame as "project-nonempty-cwo1-result-frame" };
export { project_persistent_term_frame as "project-persistent-term-frame" };
//# sourceMappingURL=wasm-workbench-shell.js.map