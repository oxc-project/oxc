# Input

`````ts
/**
 * ```css
 * .c { @apply p-4 flex; }
 * ```
 */
export const C = 1;
`````

# Output

## `{"sortTailwindcss":{},"jsdoc":{}}`

`````ts
/**
 * ```css
 * .c {
 *   @apply flex p-4;
 * }
 * ```
 */
export const C = 1;

`````
