// Type cast comments on rest elements should stay after the comma

// ArrayPattern
const [a, /** @type {string[]} */ ...rest1] = arr;

// ObjectPattern
const { a: x, /** @type {object} */ ...rest2 } = obj;

// ArrayAssignmentTarget
[a, /** @type {string[]} */ ...rest3] = arr;

// ObjectAssignmentTarget
({ a: x, /** @type {object} */ ...rest4 } = obj);

// Nested patterns
const [{ a, /** @type {number} */ ...inner }, /** @type {any[]} */ ...outer] = nested;

// FormalParameters (issue #18589)
(/** @type {string} */ param, /** @type {string[]} */ ...rest) => {};
