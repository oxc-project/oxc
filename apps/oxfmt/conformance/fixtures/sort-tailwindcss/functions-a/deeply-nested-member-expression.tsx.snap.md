# Input

`````tsx
const A = <div className={a.b.c.clsx("p-4 flex")}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["a"]}}`

`````tsx
const A = <div className={a.b.c.clsx("flex p-4")}>Hello</div>;

`````
