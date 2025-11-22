/**
 * Assert a value is of a certain type.
 *
 * Has no runtime effect - only for guiding the type-checker.
 * Minification removes this function and all calls to it, so it has zero runtime cost.
 *
 * @param value - Value
 */
// oxlint-disable-next-line no-unused-vars
export function typeAssertIs<T>(value: unknown): asserts value is T {}

/**
 * Assert a value is not `null` or `undefined`.
 *
 * In release builds, is a no-op. Only does runtime checks in debug builds.
 * Minification removes this function and all calls to it in release builds, so it has zero runtime cost.
 *
 * @param value - Value
 */
export function debugAssertIsNonNull<T>(value: T | null | undefined): asserts value is T {
  if (!DEBUG) return;

  if (value === null || value === undefined) {
    // oxlint-disable-next-line typescript/restrict-template-expressions
    throw new Error(`Expected non-null value, got ${value}`);
  }
}
