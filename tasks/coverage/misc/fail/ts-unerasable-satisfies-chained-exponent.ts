// https://github.com/microsoft/TypeScript/issues/63661
// Same rule as `as`: for a chained `satisfies T satisfies U`, the check still compares
// against the `**` before the first assertion, so the trailing `**` makes the assertions
// impossible to erase.
// This is a syntax error.
const x = 2 ** 3 satisfies any satisfies number ** 2;
