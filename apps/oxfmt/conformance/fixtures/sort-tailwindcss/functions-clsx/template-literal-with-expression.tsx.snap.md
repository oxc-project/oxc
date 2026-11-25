# Input

`````tsx
const A = <div className={clsx(`p-4 flex ${x} m-2 inline`)}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = <div className={clsx(`flex p-4 ${x} m-2 inline`)}>Hello</div>;

`````
