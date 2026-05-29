// Fenced code blocks with keepUnparsableExampleIndent should not accumulate
// extra indentation on each format pass (idempotency).

/**
 * Non-JS fenced block with indented content.
 *
 * @example
 * ```html
 *     <div class="card">
 *       <span>Hello</span>
 *
 *       <span>World</span>
 *     </div>
 * ```
 */
function nonJsIndentedFence() {}

/**
 * JS fenced block where inner code has uniform leading whitespace.
 *
 * @example
 * ```js
 *     const x = 1;
 *     const y = 2;
 *     console.log(x + y);
 * ```
 */
function jsIndentedFence() {}

/**
 * Non-JS fenced block with mixed relative indentation.
 *
 * @example
 * ```yaml
 *   root:
 *     child:
 *       value: 42
 * ```
 */
function mixedRelativeIndent() {}
