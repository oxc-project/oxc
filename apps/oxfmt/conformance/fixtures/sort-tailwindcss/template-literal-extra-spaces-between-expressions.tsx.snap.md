# Input

`````tsx
const A = <div className={`${a}   ${b}   ${c}`}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={`${a} ${b} ${c}`}>Hello</div>;

`````
