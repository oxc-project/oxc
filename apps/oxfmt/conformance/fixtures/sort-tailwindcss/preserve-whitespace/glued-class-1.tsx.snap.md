# Input

`````tsx
const A = <div className={`text-white flex bg-${color}`}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"preserveWhitespace":true}}`

`````tsx
const A = <div className={`flex text-white bg-${color}`}>Hello</div>;

`````

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={`flex text-white bg-${color}`}>Hello</div>;

`````
