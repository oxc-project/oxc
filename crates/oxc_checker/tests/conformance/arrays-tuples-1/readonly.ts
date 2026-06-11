export {};

// readonly T[] targets accept fresh array literals.
const ok_ro: readonly number[] = [1, 2];
const bad_ro_elem: readonly number[] = [1, "x"];
const ok_ro_generic: ReadonlyArray<string> = ["a"];
const ok_ro_empty: readonly (string | number)[] = [];

// readonly source cannot flow to a mutable array target.
const ro_source: readonly number[] = [1, 2, 3];
const bad_mut_from_ro: number[] = ro_source;

// Mutable source flows to readonly target.
const mut_source: string[] = ["a", "b"];
const ok_ro_from_mut: readonly string[] = mut_source;
