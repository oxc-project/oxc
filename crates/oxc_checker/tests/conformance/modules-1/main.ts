// ok: named import that exists
import { ok_origin, type Point } from "./shapes";
// bad: named export does not exist -> TS2305
import { bad_missingHelper } from "./shapes";
// bad: no default export, but a same-named named export exists -> TS2613
import ok_makePoint from "./shapes";
// bad: no default export at all -> TS1192
import bad_defaultShape from "./shapes";
// bad: module cannot be resolved -> TS2307
import { bad_ghostValue } from "./no-such-module";

const ok_copy: Point = { x: ok_origin.x, y: ok_origin.y };

export const ok_xCoordinate: number = ok_copy.x;
