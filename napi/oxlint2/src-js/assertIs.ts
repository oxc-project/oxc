/**
 * Assert a value is of a certain type.
 *
 * Has no runtime effect - only for guiding the type-checker.
 * Minification removes this function and all calls to it, so it has zero runtime cost.
 *
 * @param value - Value
 */
// oxlint-disable-next-line no-unused-vars
export default function assertIs<T>(value: unknown): asserts value is T {}
