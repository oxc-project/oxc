export {};

// Nested array literals.
const ok_matrix: number[][] = [
  [1, 2],
  [3, 4],
];
const bad_matrix: number[][] = [
  [1, 2],
  ["x", 4],
];
const ok_nested_empty: string[][] = [[], []];
const bad_inner_generic: Array<Array<string>> = [["a"], [1]];
const ok_deep: number[][][] = [[[1]], [[]], []];
