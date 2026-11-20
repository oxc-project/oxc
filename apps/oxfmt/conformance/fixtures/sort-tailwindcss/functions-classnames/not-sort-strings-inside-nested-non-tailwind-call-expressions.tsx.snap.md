# Input

`````tsx
const A = <div className={classNames(
  "bg-red-500",
  value.includes("\n") ? "pt-2" : "pt-1"
)} />;
`````

# Output

## `{"sortTailwindcss":{"functions":["classNames"]}}`

`````tsx
const A = <div className={classNames("bg-red-500", value.includes("\n") ? "pt-2" : "pt-1")} />;

`````
