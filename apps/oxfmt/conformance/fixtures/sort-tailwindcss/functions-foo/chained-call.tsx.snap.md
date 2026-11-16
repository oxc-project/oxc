# Input

`````tsx
const A = <div className={foo().clsx("p-4 flex")}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["foo"]}}`

`````tsx
const A = <div className={foo().clsx("flex p-4")}>Hello</div>;

`````
