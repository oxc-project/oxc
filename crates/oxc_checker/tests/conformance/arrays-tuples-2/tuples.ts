export {};

// Tuple arity mismatches.
const ok_pair: [string, number] = ["id", 1];
const bad_arity_short: [string, number] = ["id"];
const bad_arity_long: [string, number] = ["id", 1, true];

// Tuple element type mismatches (one error per bad element).
const bad_elems_swapped: [string, number] = [1, "id"];
const ok_triple: [boolean, string, number] = [true, "x", 0];
const bad_middle_elem: [boolean, string, number] = [true, 2, 0];

// Optional tuple elements.
const ok_optional_omitted: [number, string?] = [1];
const ok_optional_present: [number, string?] = [1, "a"];
const bad_optional_type: [number, string?] = [1, 2];

// Rest elements in tuples.
const ok_rest: [string, ...number[]] = ["a", 1, 2, 3];
const ok_rest_empty: [string, ...number[]] = ["a"];
const bad_rest_elem: [string, ...number[]] = ["a", 1, "b"];
