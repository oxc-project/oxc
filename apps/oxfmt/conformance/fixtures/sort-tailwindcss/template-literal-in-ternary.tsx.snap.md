# Input

`````tsx
const A = <div className={condition ? `p-4 flex` : `m-2 grid`} />;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={condition ? `flex p-4` : `m-2 grid`} />;

`````
