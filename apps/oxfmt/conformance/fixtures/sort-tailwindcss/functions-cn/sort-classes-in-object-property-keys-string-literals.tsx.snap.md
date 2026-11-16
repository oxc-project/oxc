# Input

`````tsx
const A = <div className={cn({ 'p-[2px] elevation-elevated-selected': true })}>Hello</div>;
`````

# Output

## `{"sortTailwindcss":{"functions":["cn"]}}`

`````tsx
const A = <div className={cn({ "elevation-elevated-selected p-[2px]": true })}>Hello</div>;

`````
