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

// Functions to get internal properties of a `NodeArray`. Initialized in class static block below.
let getInternalFromProxy, getLength, getElement;

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
    return new NodeArrayValuesIterator(this);
  }

  // Override `keys` method with a more efficient one that avoids going via proxy for every iteration.
  // TODO: Benchmark to check that this is actually faster.
  keys() {
    return new NodeArrayKeysIterator(this);
  }

  // Override `entries` method with a more efficient one that avoids going via proxy for every iteration.
  // TODO: Benchmark to check that this is actually faster.
  entries() {
    return new NodeArrayEntriesIterator(this);
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
     * Get internal properties of `NodeArray`, given a proxy wrapping a `NodeArray`.
     * @param {Proxy} proxy - Proxy wrapping `NodeArray` object
     * @returns {Object} - Internal properties object
     */
    getInternalFromProxy = proxy => proxy[ARRAY].#internal;

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

  constructor(proxy) {
    const internal = getInternalFromProxy(proxy),
      { pos, stride } = internal;

    this.#internal = {
      pos,
      endPos: pos + internal.length * stride,
      ast: internal.ast,
      construct: internal.construct,
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

  constructor(proxy) {
    const internal = getInternalFromProxy(proxy);
    this.#internal = { index: 0, length: internal.length };
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

  constructor(proxy) {
    const internal = getInternalFromProxy(proxy);

    this.#internal = {
      index: 0,
      length: internal.length,
      pos: internal.pos,
      ast: internal.ast,
      construct: internal.construct,
      stride: internal.stride,
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
    const index = toIndex(key);
    if (index !== null) return index < getLength(arr);
    return Reflect.has(arr, key);
  },

  // Get elements and length.
  get(arr, key) {
    // Methods of `NodeArray` are called with `this` being the proxy, rather than the `NodeArray` itself.
    // They can "unwrap" the proxy by getting `this[ARRAY]`.
    if (key === ARRAY) return arr;
    if (key === 'length') return getLength(arr);
    const index = toIndex(key);
    if (index !== null) return getElement(arr, index);

    return Reflect.get(arr, key);
  },

  // Get descriptors for elements and length.
  getOwnPropertyDescriptor(arr, key) {
    if (key === 'length') {
      // Cannot return `writable: false` unfortunately
      return { value: getLength(arr), writable: true, enumerable: false, configurable: false };
    }

    const index = toIndex(key);
    if (index !== null) {
      const value = getElement(arr, index);
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
    if (key === 'length' || toIndex(key) !== null) return false;
    return Reflect.defineProperty(arr, key, descriptor);
  },

  // Prevent deleting entries.
  deleteProperty(arr, key) {
    // Note: `Reflect.deleteProperty(arr, 'length')` already returns `false`
    if (toIndex(key) !== null) return false;
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
 * Convert key to array index, if it is a valid array index.
 *
 * Only strings comprising a plain integer are valid indexes.
 * e.g. `"-1"`, `"01"`, `"0xFF"`, `"1e1"`, `"1 "` are not valid indexes.
 * Integers >= 4294967295 are not valid indexes.
 *
 * @param {string|Symbol} - Key used for property lookup.
 * @returns {number|null} - `key` converted to integer, if it's a valid array index, otherwise `null`.
 */
function toIndex(key) {
  if (typeof key === 'string') {
    if (key === '0') return 0;
    if (INDEX_REGEX.test(key)) {
      const index = +key;
      if (index < 4294967295) return index;
    }
  }
  return null;
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
