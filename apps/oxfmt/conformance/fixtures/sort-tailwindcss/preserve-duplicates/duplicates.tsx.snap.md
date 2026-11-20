# Input

`````tsx
const A = <div className="flex p-4 flex p-4">Hello</div>;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className="flex p-4">Hello</div>;

`````

## `{"sortTailwindcss":{"preserveDuplicates":true}}`

`````tsx
const A = <div className="flex flex p-4 p-4">Hello</div>;

`````
