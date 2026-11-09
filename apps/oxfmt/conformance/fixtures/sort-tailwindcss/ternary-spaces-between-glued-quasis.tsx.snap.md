# Input

`````tsx
const A = <div className={`prefix${condition ? ' middle ' : ''}suffix`} />;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={`prefix${condition ? " middle " : ""}suffix`} />;

`````
