# Input

`````tsx
const A = <div className={`MuiApi-item-root${isExtendable ? ' MuiApi-item-header-extendable' : ''}`} />;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = (
  <div className={`MuiApi-item-root${isExtendable ? " MuiApi-item-header-extendable" : ""}`} />
);

`````
