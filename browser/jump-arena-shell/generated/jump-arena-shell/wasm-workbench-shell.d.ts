import * as shell from "./shell.js";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
export type ExactWasmModuleBytes = ArrayBuffer;
export interface BrowserTimer<TToken> {
    setTimeout(tick: () => unknown, delay: number): TToken;
    clearTimeout(token: TToken): unknown;
}
export interface PassiveShell<TFrame> {
    readonly renderFrame: (frame: TFrame) => unknown;
    readonly dispose: () => unknown;
}
export type PassiveShellFactory<TInput, TFrame> = (emitInput: (input: TInput) => unknown) => PassiveShell<TFrame>;
export interface PassiveWasmWorkbenchComposition<TFrame> {
    readonly controller: workbench.CartridgeWorkbench;
    readonly shell: PassiveShell<TFrame>;
    readonly dispose: () => unknown;
}
type NativeShellFactory = (typeof shell)["create-jump-arena-shell!"];
type NativeMount = Parameters<NativeShellFactory>[0];
type NativeBrowser = Parameters<NativeShellFactory>[1];
type NativeThree = Parameters<NativeShellFactory>[2];
declare function create_sync_wasm_initializer<TModule>(initialize_sync: (module: ExactWasmModuleBytes) => TModule): (input: unknown) => Promise<TModule>;
declare function project_persistent_term_frame(term_bytes: unknown): wasm.ProjectedValue;
declare function project_nonempty_cwo1_result_frame(values: unknown): readonly unknown[];
declare function create_composition_bang<TInput, TFrame, TToken>(create_shell: PassiveShellFactory<TInput, TFrame>, project_frame: (values: workbench.WorkbenchEnvelope) => TFrame, browser: BrowserTimer<TToken>, module: unknown, exact_request_bytes: wasm.ExactBytes, fixed_tick: workbench.FixedTick, policy: workbench.WorkbenchPolicy, emit_receipt: (receipt: workbench.LifecycleReceipt) => unknown): PassiveWasmWorkbenchComposition<TFrame>;
declare function create_passive_wasm_workbench_shell_bang<TToken>(mount: NativeMount, browser: NativeBrowser & BrowserTimer<TToken>, three: NativeThree, module: unknown, exact_request_bytes: wasm.ExactBytes, fixed_tick: workbench.FixedTick, policy: workbench.WorkbenchPolicy, emit_receipt: (receipt: workbench.LifecycleReceipt) => unknown): PassiveWasmWorkbenchComposition<wasm.ProjectedValue>;
declare function load_passive_wasm_workbench_shell_bang<TInput, TFrame, TToken, TModule>(initialize_module: (input: {
    readonly module_or_path: ExactWasmModuleBytes;
}) => Promise<TModule>, exact_module_bytes: unknown, create_shell: PassiveShellFactory<TInput, TFrame>, project_frame: (values: workbench.WorkbenchEnvelope) => TFrame, browser: BrowserTimer<TToken>, exact_request_bytes: wasm.ExactBytes, fixed_tick: workbench.FixedTick, policy: workbench.WorkbenchPolicy, emit_receipt: (receipt: workbench.LifecycleReceipt) => unknown): Promise<PassiveWasmWorkbenchComposition<TFrame>>;
declare const create_passive_wasm_workbench_with_shell_factory: typeof create_composition_bang;
export { create_passive_wasm_workbench_shell_bang as "create-passive-wasm-workbench-shell!" };
export { create_passive_wasm_workbench_with_shell_factory as "create-passive-wasm-workbench-with-shell-factory" };
export { create_sync_wasm_initializer as "create-sync-wasm-initializer" };
export { load_passive_wasm_workbench_shell_bang as "load-passive-wasm-workbench-shell!" };
export { project_nonempty_cwo1_result_frame as "project-nonempty-cwo1-result-frame" };
export { project_persistent_term_frame as "project-persistent-term-frame" };
