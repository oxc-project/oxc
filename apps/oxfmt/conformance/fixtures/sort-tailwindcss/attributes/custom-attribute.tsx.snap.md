# Input

`````tsx
const A = <div myClassProp="p-4 flex">Hello</div>;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div myClassProp="p-4 flex">Hello</div>;

`````

## `{"sortTailwindcss":{"attributes":["myClassProp"]}}`

`````tsx
const A = <div myClassProp="flex p-4">Hello</div>;

`````

## `{"sortTailwindcss":{"attributes":["data-classes"]}}`

`````tsx
const A = <div myClassProp="p-4 flex">Hello</div>;

`````
