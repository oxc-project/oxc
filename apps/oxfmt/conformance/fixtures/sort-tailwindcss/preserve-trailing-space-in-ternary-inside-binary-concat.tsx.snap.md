# Input

`````tsx
const A = <div className={"h-fit m-1 w-full " + (flag1 ? "block " : "hidden ") + (flag2 ? "p-2" : "p-4")} />;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = (
  <div className={"m-1 h-fit w-full " + (flag1 ? "block " : "hidden ") + (flag2 ? "p-2" : "p-4")} />
);

`````
