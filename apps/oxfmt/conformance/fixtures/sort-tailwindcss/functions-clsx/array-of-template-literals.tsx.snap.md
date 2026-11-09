# Input

`````tsx
const A = <div className={clsx([`p-4 flex`, `m-2 grid`])}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = <div className={clsx([`flex p-4`, `m-2 grid`])}>Hello</div>;

`````
