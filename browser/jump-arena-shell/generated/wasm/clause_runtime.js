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
 * @param {number} byte
 * @returns {number}
 */
export function clause_session_v1_request_push(byte) {
    const ret = wasm.clause_session_v1_request_push(byte);
    return ret >>> 0;
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_getRandomValues_1c61fac11405ffdc: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
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

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
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
