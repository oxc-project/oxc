// At or above the threshold arrays wrap one element per line
const wrapped = [1, 2, 3];
const spreadCounts = [1, 2, ...rest];
// Below the threshold single-line arrays stay flat
const flat = [1, 2];
// Below the threshold multiline arrays are preserved
const preserved = [
  1,
  2,
];
// Nested arrays are evaluated independently
const nested = [[1, 2], [3]];
// Destructuring wraps too
const [a, b, c] = values;
