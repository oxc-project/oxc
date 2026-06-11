// Nested object mismatches against type-literal targets.
type Inner = { value: number; tag: string };
type Outer = { name: string; inner: Inner };

const ok_outer: Outer = { name: "a", inner: { value: 1, tag: "t" } };

const bad_nested_missing: Outer = { name: "b", inner: { value: 1 } };

const bad_nested_wrong_type: Outer = { name: "c", inner: { value: "no", tag: "t" } };

const bad_nested_extra: Outer = { name: "d", inner: { value: 1, tag: "t", bonus: true } };

// Deeply nested target, fully satisfied: silent.
type Deep = { level_one: { level_two: { leaf: boolean } } };
const ok_deep: Deep = { level_one: { level_two: { leaf: true } } };

export {};
