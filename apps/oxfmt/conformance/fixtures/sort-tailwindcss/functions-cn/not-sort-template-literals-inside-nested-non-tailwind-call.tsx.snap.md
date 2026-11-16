# Input

`````tsx
const a = cn(x.includes(`a  b`) && "p-4  flex");
const b = cn(x.includes(`${"a  b"}  c`));
const C = <div className={x.split(`a  b`).length ? "p-4  flex" : ""} />;
const D = <div className={foo(`p-4
  flex`)} />;

`````

# Output

## `{"sortTailwindcss":{"functions":["cn"]}}`

`````tsx
const a = cn(x.includes(`a  b`) && "flex p-4");
const b = cn(x.includes(`${"a  b"}  c`));
const C = <div className={x.split(`a  b`).length ? "flex p-4" : ""} />;
const D = (
  <div
    className={foo(`p-4
  flex`)}
  />
);

`````
