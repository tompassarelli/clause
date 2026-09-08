/* @ts-self-types="./clause_runtime.d.ts" */

/**
 * @returns {number}
 */
export function clause_branch_v1_command() {
    const ret = wasm.clause_branch_v1_command();
    return ret >>> 0;
}

/**
 * Values 0..=255 are event bytes; 256 means an out-of-range index.
 * @param {number} index
 * @returns {number}
 */
export function clause_branch_v1_event_byte(index) {
    const ret = wasm.clause_branch_v1_event_byte(index);
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function clause_branch_v1_event_len() {
    const ret = wasm.clause_branch_v1_event_len();
    return ret >>> 0;
}

export function clause_branch_v1_io_reset() {
    wasm.clause_branch_v1_io_reset();
}

/**
 * @returns {number}
 */
export function clause_branch_v1_open() {
    const ret = wasm.clause_branch_v1_open();
    return ret >>> 0;
}

/**
 * @param {number} byte
 * @returns {number}
 */
export function clause_branch_v1_request_push(byte) {
    const ret = wasm.clause_branch_v1_request_push(byte);
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function clause_process_v1_dispatch() {
    const ret = wasm.clause_process_v1_dispatch();
    return ret >>> 0;
}

/**
 * @param {number} byte
 * @returns {number}
 */
export function clause_process_v1_request_push(byte) {
    const ret = wasm.clause_process_v1_request_push(byte);
    return ret >>> 0;
}

export function clause_process_v1_reset() {
    wasm.clause_process_v1_reset();
}

/**
 * Values 0..=255 are response bytes; 256 means an out-of-range index.
 * @param {number} index
 * @returns {number}
 */
export function clause_process_v1_response_byte(index) {
    const ret = wasm.clause_process_v1_response_byte(index);
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function clause_process_v1_response_len() {
    const ret = wasm.clause_process_v1_response_len();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function clause_session_v1_command() {
    const ret = wasm.clause_session_v1_command();
    return ret >>> 0;
}

/**
 * @param {Uint8Array} request
 * @returns {number}
 */
export function clause_session_v1_command_bulk(request) {
    const ptr0 = passArray8ToWasm0(request, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.clause_session_v1_command_bulk(ptr0, len0);
    return ret >>> 0;
}

/**
 * @returns {Uint8Array}
 */
export function clause_session_v1_event_bulk() {
    const ret = wasm.clause_session_v1_event_bulk();
    var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v1;
}

/**
 * Values 0..=255 are event bytes; 256 means an out-of-range index.
 * @param {number} index
 * @returns {number}
 */
export function clause_session_v1_event_byte(index) {
    const ret = wasm.clause_session_v1_event_byte(index);
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function clause_session_v1_event_len() {
    const ret = wasm.clause_session_v1_event_len();
    return ret >>> 0;
}

/**
 * @param {number} slot
 * @param {number} generation
 * @param {number} entry
 * @returns {Uint8Array}
 */
export function clause_session_v1_explain_bulk(slot, generation, entry) {
    const ret = wasm.clause_session_v1_explain_bulk(slot, generation, entry);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v1;
}

/**
 * @param {number} slot
 * @param {number} generation
 * @param {Uint8Array} request
 * @returns {Uint8Array}
 */
export function clause_session_v1_intervene_bulk(slot, generation, request) {
    const ptr0 = passArray8ToWasm0(request, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.clause_session_v1_intervene_bulk(slot, generation, ptr0, len0);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v2;
}

export function clause_session_v1_io_reset() {
    wasm.clause_session_v1_io_reset();
}

/**
 * @returns {number}
 */
export function clause_session_v1_open() {
    const ret = wasm.clause_session_v1_open();
    return ret >>> 0;
}

/**
 * @param {Uint8Array} request
 * @returns {number}
 */
export function clause_session_v1_open_bulk(request) {
    const ptr0 = passArray8ToWasm0(request, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.clause_session_v1_open_bulk(ptr0, len0);
    return ret >>> 0;
}

/**
 * @param {number} slot
 * @param {number} generation
 * @param {bigint} sequence
 * @param {Uint8Array} preparation
 * @returns {number}
 */
export function clause_session_v1_prepare_source(slot, generation, sequence, preparation) {
    const ptr0 = passArray8ToWasm0(preparation, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.clause_session_v1_prepare_source(slot, generation, sequence, ptr0, len0);
    return ret >>> 0;
}

/**
 * @param {number} slot
 * @param {number} generation
 * @returns {Uint8Array}
 */
export function clause_session_v1_project_bulk(slot, generation) {
    const ret = wasm.clause_session_v1_project_bulk(slot, generation);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v1;
}

/**
 * @returns {boolean}
 */
export function clause_session_v1_reclaim_retired() {
    const ret = wasm.clause_session_v1_reclaim_retired();
    return ret !== 0;
}

/**
 * @param {number} byte
 * @returns {number}
 */
export function clause_session_v1_request_push(byte) {
    const ret = wasm.clause_session_v1_request_push(byte);
    return ret >>> 0;
}

/**
 * @param {number} slot
 * @param {number} generation
 * @param {bigint} sequence
 * @param {Uint8Array} transaction
 * @returns {number}
 */
export function clause_session_v1_scalar_edit_bulk(slot, generation, sequence, transaction) {
    const ptr0 = passArray8ToWasm0(transaction, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.clause_session_v1_scalar_edit_bulk(slot, generation, sequence, ptr0, len0);
    return ret >>> 0;
}

/**
 * @param {number} slot
 * @param {number} generation
 * @returns {Uint8Array}
 */
export function clause_session_v1_source_continuity_bulk(slot, generation) {
    const ret = wasm.clause_session_v1_source_continuity_bulk(slot, generation);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v1;
}

/**
 * @param {number} slot
 * @param {number} generation
 * @param {bigint} sequence
 * @param {Uint8Array} open
 * @param {Uint8Array} witness
 * @returns {number}
 */
export function clause_session_v1_source_edit_bulk(slot, generation, sequence, open, witness) {
    const ptr0 = passArray8ToWasm0(open, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(witness, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.clause_session_v1_source_edit_bulk(slot, generation, sequence, ptr0, len0, ptr1, len1);
    return ret >>> 0;
}

/**
 * @returns {boolean}
 */
export function clause_source_profile_v1_begin() {
    const ret = wasm.clause_source_profile_v1_begin();
    return ret !== 0;
}

/**
 * @returns {string}
 */
export function clause_source_profile_v1_finish() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.clause_source_profile_v1_finish();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_8c4e43fe74559d73: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg___wbindgen_is_function_0095a73b8b156f76: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_number_get_8ff4255516ccad3e: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_buffer_7b5f53e46557d8f1: function(arg0) {
            const ret = arg0.buffer;
            return ret;
        },
        __wbg_call_4708e0c13bdc8e95: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_exports_86b4926134c40288: function(arg0) {
            const ret = arg0.exports;
            return ret;
        },
        __wbg_getRandomValues_1c61fac11405ffdc: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_get_b3ed3ad4be2bc8ac: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_instanceof_Global_2db767b6b2347f01: function(arg0) {
            let result;
            try {
                result = arg0 instanceof WebAssembly.Global;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Memory_dc8c61e3f831ee37: function(arg0) {
            let result;
            try {
                result = arg0 instanceof WebAssembly.Memory;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_length_32ed9a279acd054c: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_new_361308b2356cecd0: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_57c27ff3ddf5b62c: function() { return handleError(function (arg0) {
            const ret = new WebAssembly.Module(arg0);
            return ret;
        }, arguments); },
        __wbg_new_a4e3cab1cdd635ba: function() { return handleError(function (arg0, arg1) {
            const ret = new WebAssembly.Instance(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_new_dd2b680c8bf6ae29: function(arg0) {
            const ret = new Uint8Array(arg0);
            return ret;
        },
        __wbg_new_from_slice_a3d2629dc1826784: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_now_1297d7753a3cfbf5: function() {
            const ret = performance.now();
            return ret;
        },
        __wbg_set_cc56eefd2dd91957: function(arg0, arg1, arg2) {
            arg0.set(getArrayU8FromWasm0(arg1, arg2));
        },
        __wbg_subarray_a96e1fef17ed23cb: function(arg0, arg1, arg2) {
            const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_value_ce37a60e576310ff: function(arg0) {
            const ret = arg0.value;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./clause_runtime_bg.js": import0,
    };
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('clause_runtime_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
