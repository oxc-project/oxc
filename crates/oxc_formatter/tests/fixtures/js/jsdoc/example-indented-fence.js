// Bug: indented closing fence gets semicolon appended
// because rfind("\n```") fails when closing ``` has leading whitespace
/**
 * Exposes and documents a given getter.
 *
 * @example
 * ```typescript
 *   const x: number = 1;
 *  ```
 */
function foo() {}

// Also test with more indentation on closing fence
/**
 * @example
 * ```js
 *   const x = 1;
 *    ```
 */
function bar() {}

// Tab-indented closing fence
/**
 * @example
 * ```js
 *   const y = 2;
 *	```
 */
function baz() {}

// Closing fence with trailing whitespace
/**
 * @example
 * ```js
 *   const z = 3;
 *   ```
 */
function qux() {}

// Empty fenced block with indented close
/**
 * @example
 * ```js
 *   ```
 */
function empty() {}

// Caption + indented closing fence
/**
 * @example <caption>My caption</caption>
 * ```js
 *   const c = 42;
 *   ```
 */
function caption() {}

// Negative case: nested fence-like opening line should NOT be treated as close
/**
 * @example
 * ```md
 *   ```js
 *   const a = 1;
 * ```
 */
function nested() {}

// Negative case: indented opening fence (```js) must NOT be treated as a closer
// when there is no real closing fence -- the block is unclosed
/**
 * @example
 * ```md
 *   some text
 *   ```js
 */
function falseClose() {}
