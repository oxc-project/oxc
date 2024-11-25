let wasm;

const cachedTextDecoder = typeof TextDecoder !== 'undefined'
  ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true })
  : {
    decode: () => {
      throw Error('TextDecoder not available');
    },
  };

if (typeof TextDecoder !== 'undefined') cachedTextDecoder.decode();

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
  if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
  if (heap_next === heap.length) heap.push(heap.length + 1);
  const idx = heap_next;
  heap_next = heap[idx];

  if (typeof heap_next !== 'number') throw new Error('corrupt heap');

  heap[idx] = obj;
  return idx;
}

function getObject(idx) {
  return heap[idx];
}

function _assertBoolean(n) {
  if (typeof n !== 'boolean') {
    throw new Error(`expected a boolean argument, found ${typeof n}`);
  }
}

function _assertNum(n) {
  if (typeof n !== 'number') throw new Error(`expected a number argument, found ${typeof n}`);
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : {
  encode: () => {
    throw Error('TextEncoder not available');
  },
};

const encodeString = typeof cachedTextEncoder.encodeInto === 'function'
  ? function(arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
  }
  : function(arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
      read: arg.length,
      written: buf.length,
    };
  };

function passStringToWasm0(arg, malloc, realloc) {
  if (typeof arg !== 'string') throw new Error(`expected a string argument, found ${typeof arg}`);

  if (realloc === undefined) {
    const buf = cachedTextEncoder.encode(arg);
    const ptr = malloc(buf.length, 1) >>> 0;
    getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
  }

  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;

  const mem = getUint8ArrayMemory0();

  let offset = 0;

  for (; offset < len; offset++) {
    const code = arg.charCodeAt(offset);
    if (code > 0x7F) break;
    mem[ptr + offset] = code;
  }

  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset);
    }
    ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
    const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
    const ret = encodeString(arg, view);
    if (ret.read !== arg.length) throw new Error('failed to pass whole string');
    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }

  WASM_VECTOR_LEN = offset;
  return ptr;
}

function isLikeNone(x) {
  return x === undefined || x === null;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
  if (
    cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true ||
    (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)
  ) {
    cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
  }
  return cachedDataViewMemory0;
}

function debugString(val) {
  // primitive types
  const type = typeof val;
  if (type == 'number' || type == 'boolean' || val == null) {
    return `${val}`;
  }
  if (type == 'string') {
    return `"${val}"`;
  }
  if (type == 'symbol') {
    const description = val.description;
    if (description == null) {
      return 'Symbol';
    } else {
      return `Symbol(${description})`;
    }
  }
  if (type == 'function') {
    const name = val.name;
    if (typeof name == 'string' && name.length > 0) {
      return `Function(${name})`;
    } else {
      return 'Function';
    }
  }
  // objects
  if (Array.isArray(val)) {
    const length = val.length;
    let debug = '[';
    if (length > 0) {
      debug += debugString(val[0]);
    }
    for (let i = 1; i < length; i++) {
      debug += ', ' + debugString(val[i]);
    }
    debug += ']';
    return debug;
  }
  // Test for built-in
  const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
  let className;
  if (builtInMatches.length > 1) {
    className = builtInMatches[1];
  } else {
    // Failed to match the standard '[object ClassName]'
    return toString.call(val);
  }
  if (className == 'Object') {
    // we're a user defined class or Object
    // JSON.stringify avoids problems with cycles, and is generally much
    // easier than looping through ownProperties of `val`.
    try {
      return 'Object(' + JSON.stringify(val) + ')';
    } catch (_) {
      return 'Object';
    }
  }
  // errors
  if (val instanceof Error) {
    return `${val.name}: ${val.message}\n${val.stack}`;
  }
  // TODO we could test for more things here, like `Set`s and `Map`s.
  return className;
}

function dropObject(idx) {
  if (idx < 132) return;
  heap[idx] = heap_next;
  heap_next = idx;
}

function takeObject(idx) {
  const ret = getObject(idx);
  dropObject(idx);
  return ret;
}

function getArrayJsValueFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  const mem = getDataViewMemory0();
  const result = [];
  for (let i = ptr; i < ptr + 4 * len; i += 4) {
    result.push(takeObject(mem.getUint32(i, true)));
  }
  return result;
}

function logError(f, args) {
  try {
    return f.apply(this, args);
  } catch (e) {
    let error = function() {
      try {
        return e instanceof Error ? `${e.message}\n\nStack:\n${e.stack}` : e.toString();
      } catch (_) {
        return '<failed to stringify thrown value>';
      }
    }();
    console.error('wasm-bindgen: imported JS function that was not marked as `catch` threw an error:', error);
    throw e;
  }
}
/**
 * @param {string} query
 * @param {any} opts
 * @returns {any}
 */
export function browserslist(query, opts) {
  try {
    const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.browserslist(retptr, ptr0, len0, addHeapObject(opts));
    var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
    var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
    var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
    if (r2) {
      throw takeObject(r1);
    }
    return takeObject(r0);
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
}

function handleError(f, args) {
  try {
    return f.apply(this, args);
  } catch (e) {
    wasm.__wbindgen_exn_store(addHeapObject(e));
  }
}

const OxcFinalization = (typeof FinalizationRegistry === 'undefined')
  ? { register: () => {}, unregister: () => {} }
  : new FinalizationRegistry(ptr => wasm.__wbg_oxc_free(ptr >>> 0, 1));

export class Oxc {
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    OxcFinalization.unregister(this);
    return ptr;
  }

  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_oxc_free(ptr, 0);
  }
  /**
   * @returns {any}
   */
  get ast() {
    if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
    _assertNum(this.__wbg_ptr);
    const ret = wasm.__wbg_get_oxc_ast(this.__wbg_ptr);
    return takeObject(ret);
  }
  /**
   * @returns {string}
   */
  get ir() {
    let deferred1_0;
    let deferred1_1;
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      wasm.__wbg_get_oxc_ir(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      deferred1_0 = r0;
      deferred1_1 = r1;
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
  }
  /**
   * @returns {string}
   */
  get controlFlowGraph() {
    let deferred1_0;
    let deferred1_1;
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      wasm.__wbg_get_oxc_controlFlowGraph(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      deferred1_0 = r0;
      deferred1_1 = r1;
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
  }
  /**
   * @returns {any}
   */
  get symbols() {
    if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
    _assertNum(this.__wbg_ptr);
    const ret = wasm.__wbg_get_oxc_symbols(this.__wbg_ptr);
    return takeObject(ret);
  }
  /**
   * @returns {string}
   */
  get scopeText() {
    let deferred1_0;
    let deferred1_1;
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      wasm.__wbg_get_oxc_scopeText(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      deferred1_0 = r0;
      deferred1_1 = r1;
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
  }
  /**
   * @returns {string}
   */
  get codegenText() {
    let deferred1_0;
    let deferred1_1;
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      wasm.__wbg_get_oxc_codegenText(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      deferred1_0 = r0;
      deferred1_1 = r1;
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
  }
  /**
   * @returns {string}
   */
  get formattedText() {
    let deferred1_0;
    let deferred1_1;
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      wasm.__wbg_get_oxc_formattedText(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      deferred1_0 = r0;
      deferred1_1 = r1;
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
  }
  /**
   * @returns {string}
   */
  get prettierFormattedText() {
    let deferred1_0;
    let deferred1_1;
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      wasm.__wbg_get_oxc_prettierFormattedText(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      deferred1_0 = r0;
      deferred1_1 = r1;
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
  }
  /**
   * @returns {string}
   */
  get prettierIrText() {
    let deferred1_0;
    let deferred1_1;
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      wasm.__wbg_get_oxc_prettierIrText(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      deferred1_0 = r0;
      deferred1_1 = r1;
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
  }
  constructor() {
    const ret = wasm.oxc_new();
    this.__wbg_ptr = ret >>> 0;
    OxcFinalization.register(this, this.__wbg_ptr, this);
    return this;
  }
  /**
   * Returns Array of String
   * # Errors
   * # Panics
   * @returns {any[]}
   */
  getDiagnostics() {
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      wasm.oxc_getDiagnostics(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
      var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
      if (r3) {
        throw takeObject(r2);
      }
      var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
      wasm.__wbindgen_free(r0, r1 * 4, 4);
      return v1;
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
    }
  }
  /**
   * Returns comments
   * # Errors
   * @returns {any[]}
   */
  getComments() {
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      wasm.oxc_getComments(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
      var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
      if (r3) {
        throw takeObject(r2);
      }
      var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
      wasm.__wbindgen_free(r0, r1 * 4, 4);
      return v1;
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
    }
  }
  /**
   * # Errors
   * Serde serialization error
   * @param {string} source_text
   * @param {OxcOptions} options
   */
  run(source_text, options) {
    try {
      if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      _assertNum(this.__wbg_ptr);
      const ptr0 = passStringToWasm0(source_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.oxc_run(retptr, this.__wbg_ptr, ptr0, len0, addHeapObject(options));
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      if (r1) {
        throw takeObject(r0);
      }
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
    }
  }
}

async function __wbg_load(module, imports) {
  if (typeof Response === 'function' && module instanceof Response) {
    if (typeof WebAssembly.instantiateStreaming === 'function') {
      try {
        return await WebAssembly.instantiateStreaming(module, imports);
      } catch (e) {
        if (module.headers.get('Content-Type') != 'application/wasm') {
          console.warn(
            '`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n',
            e,
          );
        } else {
          throw e;
        }
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
}

function __wbg_get_imports() {
  const imports = {};
  imports.wbg = {};
  imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_is_undefined = function(arg0) {
    const ret = getObject(arg0) === undefined;
    _assertBoolean(ret);
    return ret;
  };
  imports.wbg.__wbindgen_as_number = function(arg0) {
    const ret = +getObject(arg0);
    return ret;
  };
  imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_error_f851667af71bcfc6 = function() {
    return logError(function(arg0, arg1) {
      let deferred0_0;
      let deferred0_1;
      try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
      } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
      }
    }, arguments);
  };
  imports.wbg.__wbg_new_abda76e883ba8a5f = function() {
    return logError(function() {
      const ret = new Error();
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_stack_658279fe44541cf6 = function() {
    return logError(function(arg0, arg1) {
      const ret = getObject(arg1).stack;
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments);
  };
  imports.wbg.__wbindgen_in = function(arg0, arg1) {
    const ret = getObject(arg0) in getObject(arg1);
    _assertBoolean(ret);
    return ret;
  };
  imports.wbg.__wbindgen_boolean_get = function(arg0) {
    const v = getObject(arg0);
    const ret = typeof v === 'boolean' ? (v ? 1 : 0) : 2;
    _assertNum(ret);
    return ret;
  };
  imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof obj === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbindgen_is_object = function(arg0) {
    const val = getObject(arg0);
    const ret = typeof val === 'object' && val !== null;
    _assertBoolean(ret);
    return ret;
  };
  imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof obj === 'number' ? obj : undefined;
    if (!isLikeNone(ret)) {
      _assertNum(ret);
    }
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbindgen_number_new = function(arg0) {
    const ret = arg0;
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
    const ret = getObject(arg0) == getObject(arg1);
    _assertBoolean(ret);
    return ret;
  };
  imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
    const ret = BigInt.asUintN(64, arg0);
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_getwithrefkey_edc2c8960f0f1191 = function() {
    return logError(function(arg0, arg1) {
      const ret = getObject(arg0)[getObject(arg1)];
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_set_f975102236d3c502 = function() {
    return logError(function(arg0, arg1, arg2) {
      getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
    }, arguments);
  };
  imports.wbg.__wbg_new_034f913e7636e987 = function() {
    return logError(function() {
      const ret = new Array();
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_set_425e70f7c64ac962 = function() {
    return logError(function(arg0, arg1, arg2) {
      getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
    }, arguments);
  };
  imports.wbg.__wbg_instanceof_ArrayBuffer_74945570b4a62ec7 = function() {
    return logError(function(arg0) {
      let result;
      try {
        result = getObject(arg0) instanceof ArrayBuffer;
      } catch (_) {
        result = false;
      }
      const ret = result;
      _assertBoolean(ret);
      return ret;
    }, arguments);
  };
  imports.wbg.__wbg_newnoargs_1ede4bf2ebbaaf43 = function() {
    return logError(function(arg0, arg1) {
      const ret = new Function(getStringFromWasm0(arg0, arg1));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_call_a9ef466721e824f2 = function() {
    return handleError(function(arg0, arg1) {
      const ret = getObject(arg0).call(getObject(arg1));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_new_7a87a0376e40533b = function() {
    return logError(function() {
      const ret = new Map();
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_set_277a63e77c89279f = function() {
    return logError(function(arg0, arg1, arg2) {
      const ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_getTime_41225036a0393d63 = function() {
    return logError(function(arg0) {
      const ret = getObject(arg0).getTime();
      return ret;
    }, arguments);
  };
  imports.wbg.__wbg_new0_218ada33b570be35 = function() {
    return logError(function() {
      const ret = new Date();
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_new_e69b5f66fda8f13c = function() {
    return logError(function() {
      const ret = new Object();
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_globalThis_05c129bf37fcf1be = function() {
    return handleError(function() {
      const ret = globalThis.globalThis;
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_self_bf91bf94d9e04084 = function() {
    return handleError(function() {
      const ret = self.self;
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_window_52dd9f07d03fd5f8 = function() {
    return handleError(function() {
      const ret = window.window;
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_global_3eca19bb09e9c484 = function() {
    return handleError(function() {
      const ret = global.global;
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_instanceof_Uint8Array_df0761410414ef36 = function() {
    return logError(function(arg0) {
      let result;
      try {
        result = getObject(arg0) instanceof Uint8Array;
      } catch (_) {
        result = false;
      }
      const ret = result;
      _assertBoolean(ret);
      return ret;
    }, arguments);
  };
  imports.wbg.__wbg_new_fec2611eb9180f95 = function() {
    return logError(function(arg0) {
      const ret = new Uint8Array(getObject(arg0));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_length_9254c4bd3b9f23c4 = function() {
    return logError(function(arg0) {
      const ret = getObject(arg0).length;
      _assertNum(ret);
      return ret;
    }, arguments);
  };
  imports.wbg.__wbg_set_ec2fcf81bc573fd9 = function() {
    return logError(function(arg0, arg1, arg2) {
      getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    }, arguments);
  };
  imports.wbg.__wbg_buffer_ccaed51a635d8a2d = function() {
    return logError(function(arg0) {
      const ret = getObject(arg0).buffer;
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbindgen_is_string = function(arg0) {
    const ret = typeof (getObject(arg0)) === 'string';
    _assertBoolean(ret);
    return ret;
  };
  imports.wbg.__wbg_stringify_eead5648c09faaf8 = function() {
    return handleError(function(arg0) {
      const ret = JSON.stringify(getObject(arg0));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_get_ef828680c64da212 = function() {
    return handleError(function(arg0, arg1) {
      const ret = Reflect.get(getObject(arg0), getObject(arg1));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
    takeObject(arg0);
  };
  imports.wbg.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
  };
  imports.wbg.__wbindgen_memory = function() {
    const ret = wasm.memory;
    return addHeapObject(ret);
  };

  return imports;
}

function __wbg_init_memory(imports, memory) {
}

function __wbg_finalize_init(instance, module) {
  wasm = instance.exports;
  __wbg_init.__wbindgen_wasm_module = module;
  cachedDataViewMemory0 = null;
  cachedUint8ArrayMemory0 = null;

  return wasm;
}

function initSync(module) {
  if (wasm !== undefined) return wasm;

  if (typeof module !== 'undefined') {
    if (Object.getPrototypeOf(module) === Object.prototype) {
      ({ module } = module);
    } else {
      console.warn('using deprecated parameters for `initSync()`; pass a single object instead');
    }
  }

  const imports = __wbg_get_imports();

  __wbg_init_memory(imports);

  if (!(module instanceof WebAssembly.Module)) {
    module = new WebAssembly.Module(module);
  }

  const instance = new WebAssembly.Instance(module, imports);

  return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
  if (wasm !== undefined) return wasm;

  if (typeof module_or_path !== 'undefined') {
    if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
      ({ module_or_path } = module_or_path);
    } else {
      console.warn('using deprecated parameters for the initialization function; pass a single object instead');
    }
  }

  if (typeof module_or_path === 'undefined') {
    module_or_path = new URL('oxc_wasm_bg.wasm', import.meta.url);
  }
  const imports = __wbg_get_imports();

  if (
    typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) ||
    (typeof URL === 'function' && module_or_path instanceof URL)
  ) {
    module_or_path = fetch(module_or_path);
  }

  __wbg_init_memory(imports);

  const { instance, module } = await __wbg_load(await module_or_path, imports);

  return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
