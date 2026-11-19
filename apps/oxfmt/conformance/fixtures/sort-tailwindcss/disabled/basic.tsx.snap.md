# Input

`````tsx
const A = <div className="p-4 flex bg-red-500 text-white">Hello</div>;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className="flex bg-red-500 p-4 text-white">Hello</div>;

`````

## `{}`

`````tsx
const A = <div className="p-4 flex bg-red-500 text-white">Hello</div>;

`````

## `{"sortTailwindcss":false}`

`````tsx
const A = <div className="p-4 flex bg-red-500 text-white">Hello</div>;

`````
