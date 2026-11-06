# Input

`````ts
/**
 * ```js
 * let   a
 * ```
 *
 * ```jsx
 * let   a
 * ```
 *
 * ```css
 * .body {color:red;
 * }
 * ```
 *
 * ```html
 * <div class="body"  >   </div>
 * ```
 */
`````

# Output

## `{"jsdoc":{}}`

`````ts
/**
 * ```js
 * let a;
 * ```
 *
 * ```jsx
 * let a;
 * ```
 *
 * ```css
 * .body {
 *   color: red;
 * }
 * ```
 *
 * ```html
 * <div class="body"></div>
 * ```
 */

`````
