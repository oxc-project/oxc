// https://github.com/microsoft/TypeScript/issues/63661
// For a chained `as T as U`, the check still compares against the `**` before the first
// assertion, so the trailing `**` makes the assertions impossible to erase.
// This is a syntax error.
const x = 2 ** 3 as any as number ** 2;
