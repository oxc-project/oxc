// https://github.com/microsoft/TypeScript/issues/63527
// `as`/`satisfies` cannot be erased here without changing the meaning, because the
// trailing `*` binds tighter than the leading `+`. This is a syntax error.
const x = 1 + 1 as number * 2;
