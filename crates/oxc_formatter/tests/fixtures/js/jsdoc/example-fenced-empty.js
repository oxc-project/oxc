// Empty fenced code blocks (opening + closing fence only, no inner code)
// should be preserved even with leading whitespace.

/**
 * Empty fence with language tag and leading whitespace.
 *
 * @example
 *   ```js
 *   ```
 */
function emptyFenceWithLanguage() {}

/**
 * Empty fence without language tag and leading whitespace.
 *
 * @example
 *   ```
 *   ```
 */
function emptyFenceWithoutLanguage() {}

/**
 * Empty fence with no leading whitespace.
 *
 * @example
 * ```js
 * ```
 */
function emptyFenceNoWhitespace() {}
