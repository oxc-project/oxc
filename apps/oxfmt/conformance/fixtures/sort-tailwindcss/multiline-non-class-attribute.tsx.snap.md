# Input

`````tsx
const A = (
  <div
    title="first
      second"
    className="p-4 flex"
  >
    Hello
  </div>
);

`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = (
  <div
    title="first
      second"
    className="flex p-4"
  >
    Hello
  </div>
);

`````
