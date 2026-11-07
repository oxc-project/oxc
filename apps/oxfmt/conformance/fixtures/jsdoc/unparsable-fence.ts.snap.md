# Input

`````ts
/**
 * ```css
 * .a { color: }}} broken {{{
 * ```
 *
 * ```graphql
 * fragment Broken on T {
 * ```
 */
`````

# Output

## `{"jsdoc":{}}`

`````ts
/**
 * ```css
 * .a { color: }}} broken {{{
 * ```
 *
 * ```graphql
 * fragment Broken on T {
 * ```
 */

`````
