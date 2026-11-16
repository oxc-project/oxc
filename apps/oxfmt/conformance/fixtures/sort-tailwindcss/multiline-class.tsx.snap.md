# Input

`````tsx
const A = (
  <div
    className="p-4
      flex"
  >
    Hello
  </div>
);
const B = (
  <div
    id="b"
    className="p-4
      flex"
  >
    Hello
  </div>
);

`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className="flex p-4">Hello</div>;
const B = (
  <div id="b" className="flex p-4">
    Hello
  </div>
);

`````
