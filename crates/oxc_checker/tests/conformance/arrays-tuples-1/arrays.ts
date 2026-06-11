export {};

// Element type mismatches inside array literals.
const ok_numbers: number[] = [1, 2, 3];
const bad_mixed: number[] = [1, "two", 3];
const ok_strings: string[] = ["a", "b"];
const bad_bool_elem: boolean[] = [true, 0, false];

// Array<T> generic syntax.
const ok_generic: Array<string> = ["a", "b"];
const bad_generic: Array<string> = ["a", 1];

// Empty array literal to typed targets is always fine.
const ok_empty: number[] = [];
const ok_empty_generic: Array<boolean> = [];
const ok_empty_union: (string | number)[] = [];

// Union element types.
const ok_union_elems: (string | number)[] = [1, "a", 2];
const bad_union_elem: (string | number)[] = [1, true];
