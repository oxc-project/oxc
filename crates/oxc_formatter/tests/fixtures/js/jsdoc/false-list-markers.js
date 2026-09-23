/**
 * This function handles the case where something wraps and the continuation
 * line starts with a character that looks like a list marker — like `await a
 * + b` or some em-dash — prefix.
 */
const a = 1;

/**
 * This function does something — `await something + other` and returns a value that needs wrapping across lines.
 */
function foo() {}

/**
 * Computes the result of adding values together with a formula:
 * result = first + second + third + fourth + fifth + sixth + more.
 */
function bar() {}

/**
 * @param {string} value - The computed value from applying: result = alpha - beta - gamma - delta - epsilon
 */
function paramMinus(value) {}

/**
 * @param {string} value - The computed value from applying: result = alpha + beta + gamma + delta + epsilon
 */
function paramPlus(value) {}
