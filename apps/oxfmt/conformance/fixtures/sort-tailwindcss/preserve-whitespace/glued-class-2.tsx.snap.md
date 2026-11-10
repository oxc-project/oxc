# Input

`````tsx
const A = <div className={`mb-4 flex items-center rounded-full bg-${color}/10`}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"preserveWhitespace":true}}`

`````tsx
const A = <div className={`mb-4 flex items-center rounded-full bg-${color}/10`}>Hello</div>;

`````

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={`mb-4 flex items-center rounded-full bg-${color}/10`}>Hello</div>;

`````
