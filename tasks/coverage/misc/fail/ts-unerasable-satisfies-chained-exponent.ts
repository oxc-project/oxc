// https://github.com/microsoft/TypeScript/issues/63661
// Same rule as `as`: a chained `satisfies T satisfies U` keeps referring back to the `**`
// operator, so the trailing `**` still makes the assertion impossible to erase. Syntax error.
const x = 2 ** 3 satisfies any satisfies number ** 2;
