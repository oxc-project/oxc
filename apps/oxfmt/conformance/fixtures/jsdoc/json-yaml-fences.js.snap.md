# Input

`````js
/**
 * ```json
 * {"a":1,   "b":[2,3]}
 * ```
 *
 * ```json5
 * {a:1,}
 * ```
 *
 * ```yaml
 * key:   value
 * list:
 *   -   1
 * ```
 */
`````

# Output

## `{"jsdoc":{}}`

`````js
/**
 * ```json
 * { "a": 1, "b": [2, 3] }
 * ```
 *
 * ```json5
 * {a:1,}
 * ```
 *
 * ```yaml
 * key: value
 * list:
 *   - 1
 * ```
 */

`````
