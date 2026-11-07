// https://github.com/microsoft/TypeScript/issues/63661
// Same rule as `as`: erasing `satisfies number` would let the trailing `**` re-associate.
// This is a syntax error.
const x = 2 ** 3 satisfies number ** 2;
