// @returns with type and description
/**
 * @returns {string} the formatted value
 */
function format() { return ""; }

// @returns type only
/**
 * @returns {void}
 */
function doSomething() {}

// @throws
/**
 * @throws {Error} when input is invalid
 */
function validate(x) {}

// @deprecated
/**
 * @deprecated use newFoo instead
 */
function oldFoo() {}

// @internal (tag without body)
/**
 * @internal
 */
function secret() {}

// @template
/**
 * @template T
 */
function identity(x) { return x; }

// @typedef
/**
 * @typedef {Object} Options
 */

// Description with tags
/**
 * performs a complex operation.
 * @param {string} input - the raw input
 * @returns {number} the computed result
 */
function compute(input) { return 0; }

// @example preservation
/**
 * @example
 * const x = foo(1);
 * console.log(x);
 */
function foo(n) { return n; }

// @prop → @property normalization
/**
 * @prop {string} name the name
 */
const config = {};

// Param without type
/**
 * @param name - the name
 */
function noType(name) {}

// Param without description
/**
 * @param {string} name
 */
function noDesc(name) {}
