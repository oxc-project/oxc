import { ID, ORIGIN, Point, VERSION, missing } from "./shapes";
import { Pt } from "./reexport";
import nope from "./does-not-exist";

// Cross-file: VERSION is `2` via isolated declarations inference in shapes.ts.
const bad_version: string = VERSION;
const ok_version: number = VERSION;

// Union target.
const bad_id: ID = true;
const ok_id: ID = "user-1";

// Structural object check, including through a re-export alias.
const bad_point: Point = { x: 1 };
const ok_point: Pt = { x: 1, y: 2 };

// Cross-file value with interface type vs primitive target.
const bad_origin: string = ORIGIN;

// satisfies.
const ok_satisfies = { x: 1, y: 2 } satisfies Point;
const bad_satisfies = { x: 1 } satisfies Point;

// Local (non-exported) alias + literal widening for `let`.
type Flag = "on" | "off";
const bad_flag: Flag = "enabled";
let widened = "on";

// Arrays and tuples.
const bad_numbers: number[] = ["a", "b"];
const ok_numbers: number[] = [1, 2, 3];
const bad_pair: [number, number] = [1, "two"];
const ok_pair: [number, number] = [1, 2];

// Return type checking. Identifier returns stay silent (they may be
// narrowed by control flow, which v0 does not model).
export function describe(id: ID): string {
  if (typeof id === "number") {
    return 42;
  }
  return id;
}

// Silence unused-variable noise for tools reading this fixture.
export { bad_version, ok_version, bad_id, ok_id, bad_point, ok_point, bad_origin, ok_satisfies, bad_satisfies, bad_flag, widened, bad_numbers, ok_numbers, bad_pair, ok_pair, nope };
