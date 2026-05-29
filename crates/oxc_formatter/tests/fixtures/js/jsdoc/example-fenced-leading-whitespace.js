// Fenced code blocks with leading whitespace on opening/closing fences
// should still be detected and formatted properly.

/**
 * Opening fence has leading whitespace.
 *
 * @example
 *   ```js
 *   const value = createLongLabel("alpha", "beta", "gamma", "delta", "epsilon");
 *   console.log(value);
 *   ```
 */
function openingFenceIndented() {}

/**
 * Closing fence has leading whitespace.
 *
 * @example
 * ```js
 * const label = `hello ${name}`;
 * console.log(label);
 *   ```
 */
function closingFenceIndented(name) {}

/**
 * Both opening and closing fences have leading whitespace.
 *
 * @example
 *     ```js
 *     const x = 1;
 *     const y = 2;
 *     ```
 */
function bothFencesIndented() {}
