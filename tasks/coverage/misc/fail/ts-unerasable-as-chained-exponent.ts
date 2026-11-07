// https://github.com/microsoft/TypeScript/issues/63661
// A chained `as T as U` keeps referring back to the `**` operator, so the trailing `**`
// still makes the assertion impossible to erase. Syntax error.
const x = 2 ** 3 as any as number ** 2;
