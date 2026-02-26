// Single-line type tag
/** @type {string} */
const a = "hello";

// Multi-line to single-line conversion
/**
 * @type {number}
 */
const b = 42;

// Empty JSDoc removal
/**
 *
 */
const c = true;

// Description only
/**
 * this is a description.
 */
function foo() {}

// Tag normalization: @return → @returns
/**
 * @return {string} the result
 */
function bar() { return ""; }

// Tag normalization: @arg → @param
/**
 * @arg {number} x the value
 */
function baz(x) {}

// Tag normalization: @yield → @yields
/**
 * @yield {number}
 */
function* gen() {}

// Capitalization
/**
 * @param {string} name - the user's name
 */
function greet(name) {}

// Type whitespace normalization
/**
 * @type {  string  |  number  }
 */
const d = "hello";

// Multiple params
/**
 * @param {string} a - first param
 * @param {number} b - second param
 * @param {boolean} c - third param
 */
function multi(a, b, c) {}
