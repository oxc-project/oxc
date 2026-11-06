# Input

`````ts
const HTML_WITH_CONST = /* HTML */ `
<div>
<h1>foo</h1>
  <p>foo</p>
</div>
` as const;

`````

# Output

## `{"printWidth":80}`

`````ts
const HTML_WITH_CONST = /* HTML */ `
  <div>
    <h1>foo</h1>
    <p>foo</p>
  </div>
` as const;

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}`

`````ts
const HTML_WITH_CONST = /* HTML */ `
  <div>
    <h1>foo</h1>
    <p>foo</p>
  </div>
` as const;

`````
