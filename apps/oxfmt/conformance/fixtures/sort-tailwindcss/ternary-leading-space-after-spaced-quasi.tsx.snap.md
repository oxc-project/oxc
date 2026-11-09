# Input

`````tsx
const A = <div className={`header ${isExtendable ? ' active' : ''}`} />;
`````

# Output

## `{"sortTailwindcss":{}}`

`````tsx
const A = <div className={`header ${isExtendable ? "active" : ""}`} />;

`````
