# Input

`````tsx
const A = <div className={clsx("p-4 flex", input.split(" ").map(x => x))} />;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = (
  <div
    className={clsx(
      "flex p-4",
      input.split(" ").map((x) => x),
    )}
  />
);

`````
