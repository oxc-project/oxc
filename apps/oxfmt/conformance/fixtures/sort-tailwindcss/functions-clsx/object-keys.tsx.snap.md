# Input

`````tsx
const A = <div className={clsx({
      'p-4 flex': condition1,
      'm-2 grid': condition2,
      'text-white bg-red-500': condition3
    })}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = (
  <div
    className={clsx({
      "flex p-4": condition1,
      "m-2 grid": condition2,
      "bg-red-500 text-white": condition3,
    })}
  >
    Hello
  </div>
);

`````
