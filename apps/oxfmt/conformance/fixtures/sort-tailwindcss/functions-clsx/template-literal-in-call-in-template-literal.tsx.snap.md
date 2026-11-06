# Input

`````tsx
const A = <div className={`p-4 flex ${clsx(`m-2 grid`)}`}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = <div className={`flex p-4 ${clsx(`m-2 grid`)}`}>Hello</div>;

`````
