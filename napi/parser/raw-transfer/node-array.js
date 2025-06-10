'use strict';

// TODO: Tests

// Unique token which is not exposed publicly.
// Used to prevent user calling class constructors.
const TOKEN = {};

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
// TODO: Currently elements are cached as they're deserialized, to ensure accessing the same element
// twice returns the same object. It'd be better if we had a global node cache, then this wouldn't
// be necessary, and storing `parent` for slices also wouldn't be needed.
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
   * @param {Function} deserialize - Function to deserialize element
   * @param {NodeArray|null} parent - If this is a slice of another `NodeArray`, the parent `NodeArray`
   * @param {number} parentOffset - Index of start of the slice
   * @param {Object} ast - AST object
   * @param {Object} token - Internal token
   * @returns {Proxy<NodeArray>} - `NodeArray` wrapped in a `Proxy`
   */
  constructor(pos, length, stride, deserialize, parent, parentOffset, ast) {
    if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');

    if (parent === null) {
      // Fill array with `length` elements of `undefined`.
      // Array will be used to cache values as they're deserialized.
      // Use `push` to avoid the array being stored as "holey".
      super();
      for (let i = 0; i < length; i++) {
        this.push(void 0);
      }
    } else {
      // This is a slice of another `NodeArray`.
      // No need to give it elements, as all element accesses will go via the parent.
      super(length);
    }

    this.#internal = {
      $pos: pos,
      $ast: ast,
      $stride: stride,
      $deserialize: deserialize,
      $parent: parent,
      $parentOffset: parentOffset,
    };

    const proxy = new Proxy(this, PROXY_HANDLERS);
    nodeArrays.set(proxy, this);
    return proxy;
  }

  // Allow `arr.filter`, `arr.map` etc.
  // TODO: Would be better for some methods e.g. `substr` to return a `NodeArray`.
  static [Symbol.species] = Array;

  // Override `values` method with a more efficient one that avoids repeatedly going via proxy.
  // TODO: Benchmark to check that this is actually faster.
  values() {
    // Get actual `NodeArray`. `this` is a proxy.
    let arr = nodeArrays.get(this),
      internal = arr.#internal;

    let index, endIndex;
    if (internal.$parent === null) {
      // Normal `NodeArray`
      index = 0;
      endIndex = arr.length;
    } else {
      // Slice
      index = internal.$parentOffset;
      endIndex = start + arr.length;
      arr = internal.$parent;
      internal = arr.#internal;
    }

    const { $pos: pos, $deserialize: deserialize, $stride: stride, $ast: ast } = internal;

    // Reuse `ret` object for each turn of iteration
    // TODO: Turn this into a class.
    const ret = { done: false, value: null };
    return {
      next() {
        if (index === endIndex) {
          ret.done = true;
          ret.value = null;
          return ret;
        }

        ret.value = arr[index];
        if (ret.value === void 0) ret.value = arr[index] = deserialize(pos + index * stride, ast);
        index++;
        return ret;
      },

      [Symbol.iterator]() {
        return this;
      },
    };
  }

  // Override `slice` method to return a `NodeArray`.
  //
  // The new `NodeArray` references this `NodeArray` so element accesses on the slice will
  // read/write to this `NodeArray`.
  //
  // TODO: Is logic correct here for all values of `start` and `end`?
  // TODO: Is there any way to avoid the `WeakMap`? Necessary because `this` here is proxy.
  slice(start, end) {
    // Get actual `NodeArray`. `this` is a proxy.
    let arr = nodeArrays.get(this);
    if (arr === void 0) throw new Error('`slice` called on a value which is not a `NodeArray`');

    start = toInt(start);
    if (start < 0) {
      start = arr.length + start;
      if (start < 0) start = 0;
    }
    if (end === void 0) {
      end = arr.length;
    } else {
      end = toInt(end);
      if (end < 0) {
        end = arr.length + end;
        if (end < 0) end = 0;
      }
    }

    if (end > arr.length) end = arr.length;

    let length = end - start;
    if (length <= 0 || start > arr.length) {
      start = 0;
      length = 0;
    }

    const internal = arr.#internal;
    if (internal.$parent !== null) {
      // Slice of a slice
      arr = internal.$parent;
      start += internal.$parentOffset;
    }

    const pos = internal.$pos + start * internal.$stride;
    return new NodeArray(pos, length, internal.$stride, internal.$deserialize, arr, start, internal.$ast);
  }

  // Override output for `console.log`.
  // Otherwise it's confusing that elements which havent been accessed yet appear as `undefined`.
  // TODO: Could deserialize all elements instead and output that?
  [Symbol.for('nodejs.util.inspect.custom')]() {
    return `NodeArray(${this.length}) [ <${this.length} items> ]`;
  }

  static {
    getInternal = arr => arr.#internal;
  }
}

NodeArray.prototype[Symbol.iterator] = NodeArray.prototype.values;

module.exports = { NodeArray, TOKEN };

/**
 * Get element of `NodeArray` at index `index`.
 * `index` must be in bounds (i.e. `< arr.length`).
 *
 * @param {NodeArray} arr - `NodeArray` object
 * @param {number} index - Index of element to get
 * @returns {*} - Element at index `index`
 */
function getElement(arr, index) {
  let internal = getInternal(arr);
  if (internal.$parent !== null) {
    arr = internal.$parent;
    index += internal.$parentOffset;
    internal = getInternal(arr);
  }

  const value = arr[index];
  if (value !== void 0) return value;

  const deserialize = internal.$deserialize;
  return arr[index] = deserialize(internal.$pos + (index * internal.$stride), internal.$ast);
}

/**
 * Convert value to integer.
 * https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number#integer_conversion
 * TODO: Check this is correct.
 *
 * @param {*} value - Value to convert to integer.
 * @returns {number} - Integer
 */
function toInt(value) {
  value = Math.trunc(+value);
  if (value === 0 || Number.isNaN(value)) return 0;
  return value;
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
    if (getInternal(arr).$parent === null) {
      return Reflect.ownKeys(arr);
    }

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
