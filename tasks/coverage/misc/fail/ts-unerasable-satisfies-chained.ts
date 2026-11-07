// https://github.com/microsoft/TypeScript/issues/63527
// Same rule as `as`: a chained `satisfies T satisfies U` keeps referring back to the
// precedence of the `+` operator, so the trailing `*` still makes the assertion impossible
// to erase. Syntax error.
const x = 1 + 1 satisfies any satisfies number * 2;
