# Input

`````js
const markup = html`<div z="last" a="first"></div>`;
const interpolated = html`<div z=${last()} a=${first()}></div>`;

`````

# Output

## `{"sortAttributes":true}`

`````js
const markup = html`<div a="first" z="last"></div>`;
const interpolated = html`<div z=${last()} a=${first()}></div>`;

`````
