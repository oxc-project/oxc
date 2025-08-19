/**
 * Get error message from an error.
 *
 * `err` is expected to be an `Error` object, but can be anything.
 *
 * This function will never throw, and always returns a string, even if:
 *
 * * `err` is `null` or `undefined`.
 * * `err` is an object with a getter for `message` property which throws.
 * * `err` has a getter for `message` property which returns a different value each time it's accessed.
 *
 * @param err - Error
 * @returns Error message
 */
export function getErrorMessage(err: unknown): string {
  try {
    const { message } = err as undefined | { message: string };
    if (typeof message === 'string' && message !== '') return message;
  } catch {}

  return 'Unknown error';
}

/**
 * Assert a value is of a certain type.
 *
 * Has no runtime effect - only for guiding the type-checker.
 * Minification removes this function and all calls to it, so it has zero runtime cost.
 *
 * @param value - Value
 */
// oxlint-disable-next-line no-unused-vars
export function assertIs<T>(value: unknown): asserts value is T {}
