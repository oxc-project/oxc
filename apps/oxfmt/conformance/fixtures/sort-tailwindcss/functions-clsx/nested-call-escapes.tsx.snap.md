# Input

`````tsx
const A = <div className={clsx(
  "flex",
  str.includes("\t") && "p-4",
  str.match(/\s/) && "m-2"
)} />;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = <div className={clsx("flex", str.includes("\t") && "p-4", str.match(/\s/) && "m-2")} />;

`````
