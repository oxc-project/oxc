# Input

`````tsx
const A = <div className={obj.clsx("p-4 flex")}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["obj"]}}`

`````tsx
const A = <div className={obj.clsx("flex p-4")}>Hello</div>;

`````
