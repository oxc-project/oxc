# Input

`````tsx
const A = (
  <div
    className="flex
      p-4"
  >
    Hello
  </div>
);

`````

# Output

## `{"sortTailwindcss":{"preserveWhitespace":true}}`

`````tsx
const A = (
  <div
    className="flex
      p-4"
  >
    Hello
  </div>
);

`````

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className="flex p-4">Hello</div>;

`````
