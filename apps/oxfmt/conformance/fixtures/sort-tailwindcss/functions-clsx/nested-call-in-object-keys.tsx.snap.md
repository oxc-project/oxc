# Input

`````tsx
const A = <div className={clsx({
      'p-4 flex': value.includes(" "),
      'error-class': true
    })}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["clsx"]}}`

`````tsx
const A = (
  <div
    className={clsx({
      "flex p-4": value.includes(" "),
      "error-class": true,
    })}
  >
    Hello
  </div>
);

`````
