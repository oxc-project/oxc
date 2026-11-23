# Input

`````tsx
const A = <div className={clsx(
  "flex p-4",
  arr.filter(x => x.startsWith("prefix")).join(" ")
)} />;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = <div className={clsx("flex p-4", arr.filter((x) => x.startsWith("prefix")).join(" "))} />;

`````
