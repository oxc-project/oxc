# Input

`````tsx
const A = <div className="p-4 flex" />;
/**
 * ```js
 * foo( 1 )
 * ```
 */
export const B = <div className="text-sm grid" />;
`````

# Output

## `{"sortTailwindcss":{},"jsdoc":{}}`

`````tsx
const A = <div className="flex p-4" />;
/**
 * ```js
 * foo(1);
 * ```
 */
export const B = <div className="grid text-sm" />;

`````
