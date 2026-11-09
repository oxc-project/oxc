# Input

`````js
const a = html`<style>a { ${b}: ${c} }</style>`;
const d = html`<style>${e} { color: red }</style><p>${f}</p>`;

`````

# Output

## `{"printWidth":80}`

`````js
const a = html`<style>
  a {
    ${b}: ${c};
  }
</style>`;
const d = html`<style>
    ${e} {
      color: red;
    }
  </style>
  <p>${f}</p>`;

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}`

`````js
const a = html`
  <style>
    a {
      ${b}: ${c};
    }
  </style>
`;
const d = html`
  <style>
    ${e} {
      color: red;
    }
  </style>
  <p>${f}</p>
`;

`````
