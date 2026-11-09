# Input

`````tsx
const A = <div className={`p-4 flex ${a ? `m-2 grid ${b ? `inline-block` : ""}` : ""}`}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={`flex p-4 ${a ? `m-2 grid ${b ? `inline-block` : ""}` : ""}`}>Hello</div>;

`````
