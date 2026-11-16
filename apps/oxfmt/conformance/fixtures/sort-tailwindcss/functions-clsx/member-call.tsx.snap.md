# Input

`````tsx
const A = <div className={clsx.foo("p-4 flex")}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = <div className={clsx.foo("flex p-4")}>Hello</div>;

`````
