/* tslint:disable */
import * as wasm from './robohome_crypto_bg';

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory;
}

function getArrayU8FromWasm(ptr, len) {
    return getUint8Memory().subarray(ptr / 1, ptr / 1 + len);
}

let cachedGlobalArgumentPtr = null;
function globalArgumentPtr() {
    if (cachedGlobalArgumentPtr === null) {
        cachedGlobalArgumentPtr = wasm.__wbindgen_global_argument_ptr();
    }
    return cachedGlobalArgumentPtr;
}

let cachegetUint32Memory = null;
function getUint32Memory() {
    if (cachegetUint32Memory === null || cachegetUint32Memory.buffer !== wasm.memory.buffer) {
        cachegetUint32Memory = new Uint32Array(wasm.memory.buffer);
    }
    return cachegetUint32Memory;
}
/**
* @returns {KeyPair}
*/
export function gen_pair() {
    return KeyPair.__wrap(wasm.gen_pair());
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm(arg) {
    const ptr = wasm.__wbindgen_malloc(arg.length * 1);
    getUint8Memory().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
/**
* @param {Uint8Array} arg0
* @param {Uint8Array} arg1
* @returns {Uint8Array}
*/
export function gen_shared_secret(arg0, arg1) {
    const ptr0 = passArray8ToWasm(arg0);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm(arg1);
    const len1 = WASM_VECTOR_LEN;
    const retptr = globalArgumentPtr();
    try {
        wasm.gen_shared_secret(retptr, ptr0, len0, ptr1, len1);
        const mem = getUint32Memory();
        const rustptr = mem[retptr / 4];
        const rustlen = mem[retptr / 4 + 1];
        if (rustptr === 0) return;
        const realRet = getArrayU8FromWasm(rustptr, rustlen).slice();
        wasm.__wbindgen_free(rustptr, rustlen * 1);
        return realRet;


    } finally {
        wasm.__wbindgen_free(ptr0, len0 * 1);
        wasm.__wbindgen_free(ptr1, len1 * 1);

    }

}

const lTextDecoder = typeof TextDecoder === 'undefined' ? require('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8');

function getStringFromWasm(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

const heap = new Array(32);

heap.fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

export function __wbg_new_baf10398b0d0c64d(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    return addHeapObject(new Function(varg0));
}

function getObject(idx) { return heap[idx]; }

export function __wbg_call_173f04c850a68d5f(arg0, arg1) {
    return addHeapObject(getObject(arg0).call(getObject(arg1)));
}

export function __wbg_self_58232ab37cbe6608(arg0) {
    return addHeapObject(getObject(arg0).self);
}

export function __wbg_crypto_329b714d7e7d321d(arg0) {
    return addHeapObject(getObject(arg0).crypto);
}

export function __wbg_getRandomValues_2f960218fce3a102(arg0) {
    return addHeapObject(getObject(arg0).getRandomValues);
}

export function __wbg_getRandomValues_5581e85fc6616df6(arg0, arg1, arg2) {
    let varg1 = getArrayU8FromWasm(arg1, arg2);
    getObject(arg0).getRandomValues(varg1);
}

export function __wbg_require_4a70cbfd3adc73a8(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    return addHeapObject(require(varg0));
}

export function __wbg_randomFillSync_355c3fcfa754fa4e(arg0, arg1, arg2) {
    let varg1 = getArrayU8FromWasm(arg1, arg2);
    getObject(arg0).randomFillSync(varg1);
}

function freeKeyPair(ptr) {

    wasm.__wbg_keypair_free(ptr);
}

/**
 */
export class KeyPair {

    static __wrap(ptr) {
        const obj = Object.create(KeyPair.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;
        freeKeyPair(ptr);
    }

    /**
    * @returns {Uint8Array}
    */
    public() {
        const retptr = globalArgumentPtr();
        wasm.keypair_public(retptr, this.ptr);
        const mem = getUint32Memory();
        const rustptr = mem[retptr / 4];
        const rustlen = mem[retptr / 4 + 1];

        const realRet = getArrayU8FromWasm(rustptr, rustlen).slice();
        wasm.__wbindgen_free(rustptr, rustlen * 1);
        return realRet;

    }
    /**
    * @returns {Uint8Array}
    */
    private() {
        const retptr = globalArgumentPtr();
        wasm.keypair_private(retptr, this.ptr);
        const mem = getUint32Memory();
        const rustptr = mem[retptr / 4];
        const rustlen = mem[retptr / 4 + 1];

        const realRet = getArrayU8FromWasm(rustptr, rustlen).slice();
        wasm.__wbindgen_free(rustptr, rustlen * 1);
        return realRet;

    }
}

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

export function __wbindgen_object_drop_ref(i) { dropObject(i); }

export function __wbindgen_is_undefined(idx) {
    return getObject(idx) === undefined ? 1 : 0;
}

export function __wbindgen_jsval_eq(a, b) {
    return getObject(a) === getObject(b) ? 1 : 0;
}

export function __wbindgen_throw(ptr, len) {
    throw new Error(getStringFromWasm(ptr, len));
}

