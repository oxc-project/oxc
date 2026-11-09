# Input

`````tsx
const A = <div className={clsx({[`p-4 flex`]: true, [`m-2 grid`]: false})}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = <div className={clsx({ [`flex p-4`]: true, [`m-2 grid`]: false })}>Hello</div>;

`````
