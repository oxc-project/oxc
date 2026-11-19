# Input

`````tsx
const A = <div className="  p-4 flex  ">Hello</div>;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className="flex p-4">Hello</div>;

`````

## `{"sortTailwindcss":{"preserveWhitespace":true}}`

`````tsx
const A = <div className="  flex p-4  ">Hello</div>;

`````

## `{"sortTailwindcss":{},"sortImports":{}}`

`````tsx
const A = <div className="flex p-4">Hello</div>;

`````

## `{"sortTailwindcss":{"preserveWhitespace":true},"sortImports":{}}`

`````tsx
const A = <div className="  flex p-4  ">Hello</div>;

`````
