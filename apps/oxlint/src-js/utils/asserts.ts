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
 * Use this for testing conditions which would indicate a bug in the code.
 * Do NOT use this for validating user input.
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

/**
 * Assert a condition.
 *
 * In release builds, is a no-op. Only does runtime checks in debug builds.
 * Minification removes this function and all calls to it in release builds, so it has zero runtime cost.
 *
 * Use this for testing conditions which would indicate a bug in the code.
 * Do NOT use this for validating user input.
 *
 * If creating the error message is expensive, or potentially creating the message itself can result in an error
 * when the assertion passes, pass a function which returns the message.
 *
 * ```ts
 * debugAssert(condition, () => `Condition failed: ${getErrorMessage()}`);
 * ```
 *
 * @param condition - Condition which is expected to be `true`
 * @param message - Message to include in error if condition is `false`,
 *   or a function which returns the message to include in error if condition is `false` (optional).
 */
export function debugAssert(
  condition: boolean,
  message?: string | (() => string),
): asserts condition {
  if (!DEBUG) return;

  if (!condition) {
    if (typeof message === "function") message = message();
    throw new Error(message ?? "Assertion failed");
  }
}
