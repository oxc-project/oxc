/**
 * Get error message from an error.
 *
 * `err` is expected to be an `Error` object, but can be anything.
 *
 * * If it's an `Error`, the error message and stack trace is returned.
 * * If it's another object with a string `message` property, `message` is returned.
 * * Otherwise, a generic "Unknown error" message is returned.
 *
 * This function will never throw, and always returns a non-empty string, even if:
 *
 * * `err` is `null` or `undefined`.
 * * `err` is an object with a getter for `message` property which throws.
 * * `err` has a getter for `stack` or `message` property which returns a different value each time it's accessed.
 *
 * @param err - Error
 * @returns Error message
 */
export function getErrorMessage(err: unknown): string {
  try {
    if (err instanceof Error) {
      // Note: `stack` includes the error message
      const { stack } = err;
      if (typeof stack === 'string' && stack !== '') return stack;
    }

    const { message } = err as { message?: unknown };
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
