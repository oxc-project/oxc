# Input

`````tsx
const A = <div className={`
      p-4 flex
      ${x}
      m-2 inline
    `}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"preserveWhitespace":false,"preserveDuplicates":false}}`

`````tsx
const A = <div className={`flex p-4 ${x} m-2 inline`}>Hello</div>;

`````

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={`flex p-4 ${x} m-2 inline`}>Hello</div>;

`````

## `{"sortTailwindcss":{"preserveWhitespace":false}}`

`````tsx
const A = <div className={`flex p-4 ${x} m-2 inline`}>Hello</div>;

`````
