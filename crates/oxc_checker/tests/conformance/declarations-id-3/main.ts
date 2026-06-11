import { Grid, ok_distance_sq, ok_origin } from "./api";
import type { Mode, Point } from "./api";

export const ok_grid: Grid = new Grid(10);
export const ok_zero: number = ok_distance_sq(ok_origin, ok_origin);
export const ok_corner: Point = { x: 9, y: 9 };
export const ok_default_mode: Mode = "slow";

// Clean: default export of an annotated identifier.
export default ok_grid;
