// Single-line arrays that fit stay single-line
const flat = [1, 2, 3];
// A line break after `[` keeps the array expanded, one element per line
const expanded = [
  1, 2, 3];
// Already one-per-line stays as-is
const stable = [
  1,
  2,
  3,
];
// Applies to destructuring and assignment targets, including leading holes
const [
  a,
  b,
] = values;
const [
  ,
  c,
] = values;
[
  d,
  e,
] = values;
// Holes are skipped when detecting the leading newline
const holes = [,
  f,
];
// Prettier's expand heuristic for arrays of objects still applies
const objects = [{ a: 1, b: 2 }, { a: 3, b: 4 }];
