# Input

`````ts
/**
 * ```scss
 * .a { .b { color :blue } }
 * ```
 *
 * ```less
 * .mixin() { color: red; }
 * ```
 *
 * ```graphql
 * query Q  { user { id   name } }
 * ```
 */
`````

# Output

## `{"jsdoc":{}}`

`````ts
/**
 * ```scss
 * .a {
 *   .b {
 *     color: blue;
 *   }
 * }
 * ```
 *
 * ```less
 * .mixin() {
 *   color: red;
 * }
 * ```
 *
 * ```graphql
 * query Q  { user { id   name } }
 * ```
 */

`````
