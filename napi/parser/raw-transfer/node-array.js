'use strict';

const { TOKEN, constructorError } = require('./lazy-common.js');

// Mapping from a proxy to the `NodeArray` that it wraps.
// Used by `slice` method.
const nodeArrays = new WeakMap();

// Function to get `#internal` property of a `NodeArray`.
// Initialized in static block in `NodeArray` class.
let getInternal;

// An array of AST nodes where elements are deserialized lazily upon access.
//
// Extends `Array` to make `Array.isArray` return `true` for a `NodeArray`.
//
// TODO: Other methods could maybe be more optimal, avoiding going via proxy multiple times.
// e.g. `some`, `indexOf`
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

    super(length);
    this.#internal = { pos, ast, stride, construct };

    const proxy = new Proxy(this, PROXY_HANDLERS);
    nodeArrays.set(proxy, this);
    return proxy;
  }

  // Allow `arr.filter`, `arr.map` etc.
  // TODO: Would be better for `slice` method to return a `NodeArray`.
  static [Symbol.species] = Array;

  // Override `values` method with a more efficient one that avoids going via proxy for every iteration.
  // TODO: Benchmark to check that this is actually faster.
  values() {
    // Get actual `NodeArray`. `this` is a proxy.
    const arr = nodeArrays.get(this);
    return new NodeArrayValuesIterator(arr.#internal, arr.length);
  }

  // Override `keys` method with a more efficient one that avoids going via proxy for every iteration.
  // TODO: Benchmark to check that this is actually faster.
  keys() {
    // Get actual `NodeArray`. `this` is a proxy.
    const arr = nodeArrays.get(this);
    return new NodeArrayKeysIterator(arr.length);
  }

  // Override `entries` method with a more efficient one that avoids going via proxy for every iteration.
  // TODO: Benchmark to check that this is actually faster.
  entries() {
    // Get actual `NodeArray`. `this` is a proxy.
    const arr = nodeArrays.get(this);
    return new NodeArrayEntriesIterator(arr.#internal, arr.length);
  }

  // Make `console.log` deserialize all elements.
  [Symbol.for('nodejs.util.inspect.custom')]() {
    const values = [...this.values()];
    Object.setPrototypeOf(values, DebugNodeArray.prototype);
    return values;
  }

  static {
    getInternal = arr => arr.#internal;
  }
}

NodeArray.prototype[Symbol.iterator] = NodeArray.prototype.values;

module.exports = NodeArray;

// Iterator over values of a `NodeArray`.
// Returned by `values` method, and also used as iterator for `for (const value of nodeArray) {}`.
class NodeArrayValuesIterator {
  #internal;

  constructor(arrInternal, length) {
    const { ast, pos, stride } = arrInternal || {};
    if (ast?.token !== TOKEN) constructorError();

    this.#internal = {
      pos,
      endPos: pos + length * stride,
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

// Iterator over keys of a `NodeArray`. Returned by `keys` method.
class NodeArrayKeysIterator {
  #internal;

  constructor(length) {
    // Don't bother gating constructor with `TOKEN` since this iterator doesn't access buffer,
    // and so is harmless
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

// Iterator over values of a `NodeArray`. Returned by `entries` method.
class NodeArrayEntriesIterator {
  #internal;

  constructor(arrInternal, length) {
    const { ast } = arrInternal || {};
    if (ast?.token !== TOKEN) constructorError();

    this.#internal = {
      index: 0,
      length,
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

// Class used for `[Symbol.for('nodejs.util.inspect.custom')]` method (`console.log`).
const DebugNodeArray = class NodeArray extends Array {};

/**
 * Get element of `NodeArray` at index `index`.
 * `index` must be in bounds (i.e. `< arr.length`).
 *
 * @param {NodeArray} arr - `NodeArray` object
 * @param {number} index - Index of element to get
 * @returns {*} - Element at index `index`
 */
function getElement(arr, index) {
  const internal = getInternal(arr);
  return (0, internal.construct)(internal.pos + index * internal.stride, internal.ast);
}

// Proxy handlers.
//
// Every `NodeArray` returned to user is wrapped in a `Proxy`, using these handlers.
// They lazily deserialize array elements upon access, and block mutation of array elements / `length`.
const PROXY_HANDLERS = {
  // Return `true` for indexes which are in bounds.
  // e.g. `'0' in arr`.
  has(arr, key) {
    if (isIndex(key)) return key * 1 < arr.length;
    return Reflect.has(arr, key);
  },

  // Get entries which are in bounds.
  get(arr, key) {
    if (isIndex(key)) {
      key *= 1;
      if (key >= arr.length) return void 0;
      return getElement(arr, key);
    }
    return Reflect.get(arr, key);
  },

  // Get descriptors which are in bounds.
  getOwnPropertyDescriptor(arr, key) {
    if (isIndex(key)) {
      key *= 1;
      if (key >= arr.length) return void 0;
      // Cannot return `configurable: false` unfortunately
      return { value: getElement(arr, key), writable: false, enumerable: true, configurable: true };
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
    // `NodeArray`s which are slices don't have their own elements. Act as if they do.
    const keys = [];
    for (let i = 0; i < arr.length; i++) {
      keys.push(i + '');
    }
    keys.push(...Reflect.ownKeys(arr));
    return keys;
  },
};

/**
 * Check if a key is a valid array index.
 * Only strings comprising of plain integers are valid indexes.
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
