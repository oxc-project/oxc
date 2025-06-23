'use strict';

const { TOKEN, constructorError } = require('./lazy-common.js');

// Internal symbol to get `NodeArray` from a proxy wrapping a `NodeArray`.
//
// Methods of `NodeArray` are called with `this` being the proxy, rather than the `NodeArray` itself.
// They can "unwrap" the proxy by getting `this[ARRAY]`, and the `get` proxy trap will return
// the actual `NodeArray`.
//
// This symbol is not exported, and it is not actually defined on `NodeArray`s, so user cannot obtain it
// via `Object.getOwnPropertySymbols` or `Reflect.ownKeys`. Therefore user code cannot unwrap the proxy.
const ARRAY = Symbol();

// Functions to get length of and get element from an array. Initialized in class static block below.
let getLength, getElement;

/**
 * An array of AST nodes where elements are deserialized lazily upon access.
 *
 * Extends `Array` to make `Array.isArray` return `true` for a `NodeArray`.
 *
 * TODO: Other methods could maybe be more optimal, avoiding going via proxy multiple times
 * e.g. `some`, `indexOf`.
 */
class NodeArray extends Array {
  #internal;

  /**
   * Create a `NodeArray`.
   *
   * Constructor does not actually return a `NodeArray`, but one wrapped in a `Proxy`.
   * The proxy intercepts accesses to elements and lazily deserializes them,
   * and blocks mutation of elements or `length` property.
   *
   * @constructor
   * @param {number} pos - Buffer position of first element
   * @param {number} length - Number of elements
   * @param {number} stride - Element size in bytes
   * @param {Function} construct - Function to deserialize element
   * @param {Object} ast - AST object
   * @returns {Proxy<NodeArray>} - `NodeArray` wrapped in a `Proxy`
   */
  constructor(pos, length, stride, construct, ast) {
    if (ast?.token !== TOKEN) constructorError();

    super();
    this.#internal = { pos, length, ast, stride, construct };
    return new Proxy(this, PROXY_HANDLERS);
  }

  // Allow `arr.filter`, `arr.map` etc.
  static [Symbol.species] = Array;

  // Override `values` method with a more efficient one that avoids going via proxy for every iteration.
  // TODO: Benchmark to check that this is actually faster.
  values() {
    return new NodeArrayValuesIterator(this[ARRAY].#internal);
  }

  // Override `keys` method with a more efficient one that avoids going via proxy for every iteration.
  // TODO: Benchmark to check that this is actually faster.
  keys() {
    return new NodeArrayKeysIterator(this[ARRAY].#internal.length);
  }

  // Override `entries` method with a more efficient one that avoids going via proxy for every iteration.
  // TODO: Benchmark to check that this is actually faster.
  entries() {
    return new NodeArrayEntriesIterator(this[ARRAY].#internal);
  }

  // This method is overwritten with reference to `values` method below.
  // Defining dummy method here to prevent the later assignment altering the shape of class prototype.
  [Symbol.iterator]() {}

  /**
   * Override `slice` method to return a `NodeArray`.
   *
   * @this {NodeArray}
   * @param {*} start - Start of slice
   * @param {*} end - End of slice
   * @returns {NodeArray} - `NodeArray` containing slice of this one
   */
  slice(start, end) {
    const internal = this[ARRAY].#internal,
      { length } = internal;

    start = toInt(start);
    if (start < 0) {
      start = length + start;
      if (start < 0) start = 0;
    }

    if (end === void 0) {
      end = length;
    } else {
      end = toInt(end);
      if (end < 0) {
        end += length;
        if (end < 0) end = 0;
      } else if (end > length) {
        end = length;
      }
    }

    let sliceLength = end - start;
    if (sliceLength <= 0 || start >= length) {
      start = 0;
      sliceLength = 0;
    }

    const { stride } = internal;
    return new NodeArray(internal.pos + start * stride, sliceLength, stride, internal.construct, internal.ast);
  }

  // Make `console.log` deserialize all elements.
  [Symbol.for('nodejs.util.inspect.custom')]() {
    const values = [...this.values()];
    Object.setPrototypeOf(values, DebugNodeArray.prototype);
    return values;
  }

  static {
    /**
     * Get length of `NodeArray`.
     * @param {NodeArray} arr - `NodeArray` object
     * @returns {number} - Array length
     */
    getLength = arr => arr.#internal.length;

    /**
     * Get element of `NodeArray` at index `index`.
     *
     * @param {NodeArray} arr - `NodeArray` object
     * @param {number} index - Index of element to get
     * @returns {*|undefined} - Element at index `index`, or `undefined` if out of bounds
     */
    getElement = (arr, index) => {
      const internal = arr.#internal;
      if (index >= internal.length) return void 0;
      return (0, internal.construct)(internal.pos + index * internal.stride, internal.ast);
    };
  }
}

NodeArray.prototype[Symbol.iterator] = NodeArray.prototype.values;

module.exports = NodeArray;

/**
 * Iterator over values of a `NodeArray`.
 * Returned by `values` method, and also used as iterator for `for (const node of nodeArray) {}`.
 */
class NodeArrayValuesIterator {
  #internal;

  constructor(arrInternal) {
    const { ast, pos, stride } = arrInternal || {};
    if (ast?.token !== TOKEN) constructorError();

    this.#internal = {
      pos,
      endPos: pos + arrInternal.length * stride,
      ast,
      construct: arrInternal.construct,
      stride,
    };
  }

  next() {
    const internal = this.#internal,
      { pos } = internal;
    if (pos === internal.endPos) return { done: true, value: null };
    internal.pos = pos + internal.stride;
    return { done: false, value: (0, internal.construct)(pos, internal.ast) };
  }

  [Symbol.iterator]() {
    return this;
  }
}

/**
 * Iterator over keys of a `NodeArray`. Returned by `keys` method.
 */
class NodeArrayKeysIterator {
  #internal;

  constructor(length) {
    // Don't bother gating constructor with `TOKEN` check.
    // This iterator doesn't access the buffer, so is harmless.
    this.#internal = { index: 0, length };
  }

  next() {
    const internal = this.#internal,
      { index } = internal;
    if (index === internal.length) return { done: true, value: null };
    internal.index = index + 1;
    return { done: false, value: index };
  }

  [Symbol.iterator]() {
    return this;
  }
}

/**
 * Iterator over values of a `NodeArray`. Returned by `entries` method.
 */
class NodeArrayEntriesIterator {
  #internal;

  constructor(arrInternal) {
    const { ast } = arrInternal || {};
    if (ast?.token !== TOKEN) constructorError();

    this.#internal = {
      index: 0,
      length: arrInternal.length,
      pos: arrInternal.pos,
      ast,
      construct: arrInternal.construct,
      stride: arrInternal.stride,
    };
  }

  next() {
    const internal = this.#internal,
      { index } = internal;
    if (index === internal.length) return { done: true, value: null };
    internal.index = index + 1;
    return {
      done: false,
      value: [index, (0, internal.construct)(internal.pos + index * internal.stride, internal.ast)],
    };
  }

  [Symbol.iterator]() {
    return this;
  }
}

// Class used for `[Symbol.for('nodejs.util.inspect.custom')]` method (for `console.log`).
const DebugNodeArray = class NodeArray extends Array {};

// Proxy handlers.
//
// Every `NodeArray` returned to user is wrapped in a `Proxy`, using these handlers.
// They lazily deserialize array elements upon access, and block mutation of array elements / `length`.
const PROXY_HANDLERS = {
  // Return `true` for indexes which are in bounds.
  // e.g. `'0' in arr`.
  has(arr, key) {
    if (isIndex(key)) return key * 1 < getLength(arr);
    return Reflect.has(arr, key);
  },

  // Get elements and length.
  get(arr, key) {
    // Methods of `NodeArray` are called with `this` being the proxy, rather than the `NodeArray` itself.
    // They can "unwrap" the proxy by getting `this[ARRAY]`.
    if (key === ARRAY) return arr;
    if (key === 'length') return getLength(arr);
    if (isIndex(key)) return getElement(arr, key * 1);

    return Reflect.get(arr, key);
  },

  // Get descriptors for elements and length.
  getOwnPropertyDescriptor(arr, key) {
    if (key === 'length') {
      // Cannot return `writable: false` unfortunately
      return { value: getLength(arr), writable: true, enumerable: false, configurable: false };
    }

    if (isIndex(key)) {
      const value = getElement(arr, key * 1);
      if (value === void 0) return void 0;
      // Cannot return `configurable: false` unfortunately
      return { value, writable: false, enumerable: true, configurable: true };
    }

    return Reflect.getOwnPropertyDescriptor(arr, key);
  },

  // Prevent setting `length` or entries.
  // Catches:
  // * `Object.defineProperty(arr, 0, {value: null})`.
  // * `arr[1] = null`.
  // * `arr.length = 0`.
  // * `Object.defineProperty(arr, 'length', {value: 0})`.
  // * Other operations which mutate entries e.g. `arr.push(123)`.
  defineProperty(arr, key, descriptor) {
    if (key === 'length' || isIndex(key)) return false;
    return Reflect.defineProperty(arr, key, descriptor);
  },

  // Prevent deleting entries.
  deleteProperty(arr, key) {
    // Note: `Reflect.deleteProperty(arr, 'length')` already returns `false`
    if (isIndex(key)) return false;
    return Reflect.deleteProperty(arr, key);
  },

  // Get keys, including element indexes.
  ownKeys(arr) {
    const keys = [],
      length = getLength(arr);
    for (let i = 0; i < length; i++) {
      keys.push(i + '');
    }
    keys.push(...Reflect.ownKeys(arr));
    return keys;
  },
};

/**
 * Check if a key is a valid array index.
 * Only strings comprising a plain integer are valid indexes.
 * e.g. `"-1"`, `"01"`, `"0xFF"`, `"1e1"`, `"1 "` are not valid indexes.
 *
 * @param {*} - Key used for property lookup.
 * @returns {boolean} - `true` if `key` is a valid array index.
 */
function isIndex(key) {
  // TODO: Any way to do this without a regex?
  return typeof key === 'string' && (key === '0' || INDEX_REGEX.test(key));
}

const INDEX_REGEX = /^[1-9]\d*$/;

/**
 * Convert value to integer.
 * https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number#integer_conversion
 *
 * @param {*} value - Value to convert to integer.
 * @returns {number} - Integer
 */
function toInt(value) {
  value = Math.trunc(+value);
  // `value === 0` check is to convert -0 to 0
  if (value === 0 || Number.isNaN(value)) return 0;
  return value;
}
