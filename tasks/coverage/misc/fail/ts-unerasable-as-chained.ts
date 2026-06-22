// https://github.com/microsoft/TypeScript/issues/63527
// A chained `as T as U` keeps referring back to the precedence of the `+` operator,
// so the trailing `*` still makes the assertion impossible to erase. Syntax error.
const x = 1 + 1 as any as number * 2;
