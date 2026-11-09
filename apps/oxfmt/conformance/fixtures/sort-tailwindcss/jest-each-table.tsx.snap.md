# Input

`````tsx
test.each`
  a | b
  ${<div className="p-4 flex" />} | ${1}
`('x', () => {});
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
test.each`
  a                               | b
  ${<div className="p-4 flex" />} | ${1}
`("x", () => {});

`````
