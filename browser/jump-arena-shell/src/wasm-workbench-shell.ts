import * as shell from "./shell.js";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";

const wasm_module_max_bytes = 16 * 1024 * 1024;

export type ExactWasmModuleBytes = ArrayBuffer;

export interface BrowserTimer<TToken> {
  setTimeout(tick: () => unknown, delay: number): TToken;
  clearTimeout(token: TToken): unknown;
}

export interface PassiveShell<TFrame> {
  readonly renderFrame: (frame: TFrame) => unknown;
  readonly dispose: () => unknown;
}

export type PassiveShellFactory<TInput, TFrame> = (
  emitInput: (input: TInput) => unknown,
) => PassiveShell<TFrame>;

export interface PassiveWasmWorkbenchComposition<TFrame> {
  readonly controller: workbench.CartridgeWorkbench;
  readonly shell: PassiveShell<TFrame>;
  readonly dispose: () => unknown;
}

type NativeShellFactory = (typeof shell)["create-jump-arena-shell!"];
type NativeMount = Parameters<NativeShellFactory>[0];
type NativeBrowser = Parameters<NativeShellFactory>[1];
type NativeThree = Parameters<NativeShellFactory>[2];
type NativeInput = Parameters<NativeShellFactory>[3] extends (
  input: infer TInput,
) => unknown
  ? TInput
  : never;

function exact_module_bytes_p(value: unknown): value is ExactWasmModuleBytes {
  return (
    value instanceof ArrayBuffer &&
    Number.isSafeInteger(value.byteLength) &&
    value.byteLength >= 1 &&
    value.byteLength <= wasm_module_max_bytes
  );
}

function require_module_bytes(value: unknown): ExactWasmModuleBytes {
  if (!exact_module_bytes_p(value)) {
    throw new Error("Wasm module bytes are outside the browser bound");
  }
  return value;
}

function module_or_path(input: unknown): unknown {
  if (
    typeof input !== "object" ||
    input === null ||
    !("module_or_path" in input)
  ) {
    throw new Error("Wasm initialization input lacks module_or_path");
  }
  return input.module_or_path;
}

function create_sync_wasm_initializer<TModule>(
  initialize_sync: (module: ExactWasmModuleBytes) => TModule,
): (input: unknown) => Promise<TModule> {
  return (input: unknown) =>
    Promise.resolve(
      initialize_sync(require_module_bytes(module_or_path(input))),
    );
}

function project_persistent_term_frame(
  term_bytes: unknown,
): wasm.ProjectedValue {
  return wasm["decode-projected-term-frame"](term_bytes);
}

function project_nonempty_cwo1_result_frame(
  values: unknown,
): readonly unknown[] {
  if (!Array.isArray(values) || values.length === 0) {
    throw new Error("CWO1 result view awaits an admitted scalar projection");
  }
  return values;
}

function schedule_fixed_tick<TToken>(
  browser: BrowserTimer<TToken>,
  delay: number,
  tick: () => unknown,
): () => unknown {
  const token = browser.setTimeout(tick, delay);
  return () => browser.clearTimeout(token);
}

function encode_input(input: unknown): string {
  const source = JSON.stringify([JSON.stringify(input)]);
  if (typeof source !== "string") {
    throw new Error("browser input is not JSON-encodable");
  }
  return source;
}

function create_composition_bang<TInput, TFrame, TToken>(
  create_shell: PassiveShellFactory<TInput, TFrame>,
  project_frame: (values: workbench.WorkbenchEnvelope) => TFrame,
  browser: BrowserTimer<TToken>,
  module: unknown,
  exact_request_bytes: wasm.ExactBytes,
  fixed_tick: workbench.FixedTick,
  policy: workbench.WorkbenchPolicy,
  emit_receipt: (receipt: workbench.LifecycleReceipt) => unknown,
): PassiveWasmWorkbenchComposition<TFrame> {
  let controller: workbench.CartridgeWorkbench | null = null;
  const passive_shell = create_shell((input: TInput) => {
    const active = controller;
    if (active !== null) {
      return active.observeInput(
        workbench["create-workbench-envelope"](
          policy,
          encode_input(input),
        ),
      );
    }
    return undefined;
  });
  const port = wasm["create-wasm-cartridge-port"](module, policy);
  const active = workbench["create-cartridge-workbench!"](
    port,
    fixed_tick,
    policy,
    (delay: number, tick: () => unknown) =>
      schedule_fixed_tick(browser, delay, tick),
    (values: workbench.WorkbenchEnvelope) =>
      passive_shell.renderFrame(project_frame(values)),
    emit_receipt,
    wasm["->ExactProcessRequest"](exact_request_bytes),
  );
  let disposed = false;
  const dispose = (): unknown => {
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

function create_passive_wasm_workbench_shell_bang<TToken>(
  mount: NativeMount,
  browser: NativeBrowser & BrowserTimer<TToken>,
  three: NativeThree,
  module: unknown,
  exact_request_bytes: wasm.ExactBytes,
  fixed_tick: workbench.FixedTick,
  policy: workbench.WorkbenchPolicy,
  emit_receipt: (receipt: workbench.LifecycleReceipt) => unknown,
): PassiveWasmWorkbenchComposition<wasm.ProjectedValue> {
  return create_composition_bang(
    (emit_input: (input: NativeInput) => unknown) =>
      shell["create-jump-arena-shell!"](mount, browser, three, emit_input),
    project_persistent_term_frame,
    browser,
    module,
    exact_request_bytes,
    fixed_tick,
    policy,
    emit_receipt,
  );
}

function load_passive_wasm_workbench_shell_bang<
  TInput,
  TFrame,
  TToken,
  TModule,
>(
  initialize_module: (input: {
    readonly module_or_path: ExactWasmModuleBytes;
  }) => Promise<TModule>,
  exact_module_bytes: unknown,
  create_shell: PassiveShellFactory<TInput, TFrame>,
  project_frame: (values: workbench.WorkbenchEnvelope) => TFrame,
  browser: BrowserTimer<TToken>,
  exact_request_bytes: wasm.ExactBytes,
  fixed_tick: workbench.FixedTick,
  policy: workbench.WorkbenchPolicy,
  emit_receipt: (receipt: workbench.LifecycleReceipt) => unknown,
): Promise<PassiveWasmWorkbenchComposition<TFrame>> {
  return initialize_module({
    module_or_path: require_module_bytes(exact_module_bytes),
  }).then((module: TModule) =>
    create_composition_bang(
      create_shell,
      project_frame,
      browser,
      module,
      exact_request_bytes,
      fixed_tick,
      policy,
      emit_receipt,
    ),
  );
}

const create_passive_wasm_workbench_with_shell_factory =
  create_composition_bang;

export { create_passive_wasm_workbench_shell_bang as "create-passive-wasm-workbench-shell!" };
export { create_passive_wasm_workbench_with_shell_factory as "create-passive-wasm-workbench-with-shell-factory" };
export { create_sync_wasm_initializer as "create-sync-wasm-initializer" };
export { load_passive_wasm_workbench_shell_bang as "load-passive-wasm-workbench-shell!" };
export { project_nonempty_cwo1_result_frame as "project-nonempty-cwo1-result-frame" };
export { project_persistent_term_frame as "project-persistent-term-frame" };
