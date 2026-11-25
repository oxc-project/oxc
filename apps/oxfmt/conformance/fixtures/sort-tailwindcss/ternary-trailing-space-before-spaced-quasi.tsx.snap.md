# Input

`````tsx
const A = <div className={`${condition ? 'active ' : ''} suffix`} />;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={`${condition ? "active" : ""} suffix`} />;

`````
