# Input

`````tsx
const A = <div className={obj[key]("p-4 flex")}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["obj"]}}`

`````tsx
const A = <div className={obj[key]("flex p-4")}>Hello</div>;

`````
