# Input

`````tsx
const A = (
  <div className={clsx("p-4 flex")}>
    <span className={cva("p-2 inline")}>Title</span>
  </div>
);
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx","cva"]}}`

`````tsx
const A = (
  <div className={clsx("flex p-4")}>
    <span className={cva("inline p-2")}>Title</span>
  </div>
);

`````
