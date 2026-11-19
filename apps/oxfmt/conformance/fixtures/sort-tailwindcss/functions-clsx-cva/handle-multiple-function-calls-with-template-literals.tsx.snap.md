# Input

`````tsx
const A = <div className={clsx(`p-4 flex`, cva(`m-2 grid`))}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx","cva"]}}`

`````tsx
const A = <div className={clsx(`flex p-4`, cva(`m-2 grid`))}>Hello</div>;

`````
