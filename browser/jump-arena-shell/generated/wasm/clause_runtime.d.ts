/* tslint:disable */
/* eslint-disable */

export function clause_session_v1_open_bulk(request: Uint8Array<ArrayBuffer>): number;
export function clause_session_v1_command_bulk(request: Uint8Array<ArrayBuffer>): number;
export function clause_session_v1_source_edit_bulk(slot: number, generation: number, sequence: bigint, open: Uint8Array<ArrayBuffer>, witness: Uint8Array<ArrayBuffer>): number;
export function clause_session_v1_intervene_bulk(slot: number, generation: number, request: Uint8Array<ArrayBuffer>): Uint8Array;



export function clause_branch_v1_command(): number;

/**
 * Values 0..=255 are event bytes; 256 means an out-of-range index.
 */
export function clause_branch_v1_event_byte(index: number): number;

export function clause_branch_v1_event_len(): number;

export function clause_branch_v1_io_reset(): void;

export function clause_branch_v1_open(): number;

export function clause_branch_v1_request_push(byte: number): number;

export function clause_process_v1_dispatch(): number;

export function clause_process_v1_request_push(byte: number): number;

export function clause_process_v1_reset(): void;

/**
 * Values 0..=255 are response bytes; 256 means an out-of-range index.
 */
export function clause_process_v1_response_byte(index: number): number;

export function clause_process_v1_response_len(): number;

export function clause_session_v1_command(): number;

export function clause_session_v1_event_bulk(): Uint8Array;

/**
 * Values 0..=255 are event bytes; 256 means an out-of-range index.
 */
export function clause_session_v1_event_byte(index: number): number;

export function clause_session_v1_event_len(): number;

export function clause_session_v1_explain_bulk(slot: number, generation: number, entry: number): Uint8Array;

export function clause_session_v1_io_reset(): void;

export function clause_session_v1_open(): number;

export function clause_session_v1_reclaim_retired(): boolean;

export function clause_session_v1_request_push(byte: number): number;

export function clause_session_v1_source_continuity_bulk(slot: number, generation: number): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly clause_process_v1_dispatch: () => number;
    readonly clause_process_v1_request_push: (a: number) => number;
    readonly clause_process_v1_reset: () => void;
    readonly clause_process_v1_response_byte: (a: number) => number;
    readonly clause_process_v1_response_len: () => number;
    readonly clause_branch_v1_command: () => number;
    readonly clause_branch_v1_event_byte: (a: number) => number;
    readonly clause_branch_v1_event_len: () => number;
    readonly clause_branch_v1_io_reset: () => void;
    readonly clause_branch_v1_open: () => number;
    readonly clause_branch_v1_request_push: (a: number) => number;
    readonly clause_session_v1_command: () => number;
    readonly clause_session_v1_command_bulk: (a: number, b: number) => number;
    readonly clause_session_v1_event_bulk: () => [number, number];
    readonly clause_session_v1_event_byte: (a: number) => number;
    readonly clause_session_v1_event_len: () => number;
    readonly clause_session_v1_explain_bulk: (a: number, b: number, c: number) => [number, number, number, number];
    readonly clause_session_v1_intervene_bulk: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly clause_session_v1_io_reset: () => void;
    readonly clause_session_v1_open: () => number;
    readonly clause_session_v1_open_bulk: (a: number, b: number) => number;
    readonly clause_session_v1_reclaim_retired: () => number;
    readonly clause_session_v1_request_push: (a: number) => number;
    readonly clause_session_v1_source_continuity_bulk: (a: number, b: number) => [number, number, number, number];
    readonly clause_session_v1_source_edit_bulk: (a: number, b: number, c: bigint, d: number, e: number, f: number, g: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
