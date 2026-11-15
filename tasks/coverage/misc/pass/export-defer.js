export defer { a, b as c, "d" as e, f as "g" } from "x";
export defer * as ns from "x";
export defer * as "string ns" from "x";
export defer { a as h } from "x" with { type: "json" };
export defer {} from "x";
export defer
{ i } from "x";
import defer from "y";
export { defer };
export { defer as j } from "y";
