# Input

`````tsx
const A = <div className={condition ? `p-4 flex ${x} m-2 inline` : `grid`} />;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={condition ? `flex p-4 ${x} m-2 inline` : `grid`} />;

`````
