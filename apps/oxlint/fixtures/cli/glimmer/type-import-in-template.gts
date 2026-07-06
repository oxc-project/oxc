// `Button` is used as a value in the template below (which the partial loader
// blanks out) and only in a *type* position in the surrounding module, so
// `consistent-type-imports` would wrongly report it as type-only and its autofix
// would rewrite the value import to `import type`, breaking the runtime use. The
// rule must be skipped for loader-derived Glimmer TS sources. The `debugger`
// statement proves other native rules still run.
import Button from "./button";

export type ButtonRef = Button;

debugger;

export default <template><Button /></template>;
