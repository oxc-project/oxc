import { manhattan, ok_default_pairs, type Pair, type Point } from "./pairs";

// Clean fixture: everything here type-checks with zero errors.
const ok_origin: Point = [0, 0];
const ok_dist: number = manhattan(ok_origin, [3, 4]);

const ok_more_pairs: Pair[] = ok_default_pairs;
const ok_first: Pair = ["c", ok_dist];

const ok_grid: ReadonlyArray<readonly number[]> = [[1], [], [2, 3]];
const ok_names: Array<string> = [];
const ok_mixed: Array<string | number> = ["a", 1, "b", 2];
const ok_nested: (string[] | number[])[] = [["a"], [1, 2], []];

const ok_optional_tail: [string, number?] = ["only"];
const ok_rest_tuple: [boolean, ...string[]] = [true, "x", "y"];
const ok_ro_tuple: readonly [Point, Point] = [
  [0, 0],
  [1, 1],
];
