# Input

`````ts
/**
 * ```json
 * { "argOne": "value1", "argTwo": "value2", "argThree": "value3", "four": 4444, "five": 555555 }
 * ```
 *
 * ```json
 * { "argOne": "value1", "argTwo": "value2", "argThree": "value3", "four": 4444, "five": 55555 }
 * ```
 */
`````

# Output

## `{"jsdoc":{}}`

`````ts
/**
 * ```json
 * {
 *   "argOne": "value1",
 *   "argTwo": "value2",
 *   "argThree": "value3",
 *   "four": 4444,
 *   "five": 555555
 * }
 * ```
 *
 * ```json
 * { "argOne": "value1", "argTwo": "value2", "argThree": "value3", "four": 4444, "five": 55555 }
 * ```
 */

`````
