export {};

// readonly tuples.
const ok_ro_tuple: readonly [number, number] = [1, 2];
const bad_ro_tuple_arity: readonly [number, number] = [1, 2, 3];
const ro_tuple_source: readonly [string, number] = ["a", 1];
const bad_mut_tuple_from_ro: [string, number] = ro_tuple_source;

// Tuples widen to arrays, but arrays do not narrow to tuples.
const tuple_source: [number, number] = [1, 2];
const ok_array_from_tuple: number[] = tuple_source;
const array_source: number[] = [1, 2];
const bad_tuple_from_array: [number, number] = array_source;

// Tuples inside array literals.
const ok_entries: [string, number][] = [
  ["a", 1],
  ["b", 2],
];
const bad_entry_arity: [string, number][] = [["a", 1], ["b"]];
