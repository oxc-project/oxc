// https://github.com/microsoft/TypeScript/issues/63527
// Same rule as `as`: erasing `satisfies number` would let `*` re-associate, so this
// is a syntax error.
const x = 1 + 1 satisfies number * 2;
