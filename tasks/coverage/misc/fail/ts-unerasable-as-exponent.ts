// https://github.com/microsoft/TypeScript/issues/63661
// `**` is right-associative, so erasing `as number` would turn `(2 ** 3) as number ** 2`
// into `2 ** (3 ** 2)`. The assertion cannot be erased without changing the meaning.
// This is a syntax error.
const x = 2 ** 3 as number ** 2;
