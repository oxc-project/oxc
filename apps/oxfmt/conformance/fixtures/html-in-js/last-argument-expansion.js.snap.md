# Input

`````js
// Tagged: html`...`
foo(html`<div><p>bar</p>foo</div>`);
foo(html` <div><p>bar</p>foo</div> `);
const a = b => html`<div><p>bar</p>foo</div>`;
const c = b => html` <div><p>bar</p>foo</div> `;

// Comment: /* HTML */ `...`
foo(/* HTML */ `<div><p>bar</p>foo</div>`);
foo(/* HTML */ ` <div><p>bar</p>foo</div> `);
const e = b => /* HTML */ `<div><p>bar</p>foo</div>`;
const g = b => /* HTML */ ` <div><p>bar</p>foo</div> `;

`````

# Output

## `{"printWidth":80}`

`````js
// Tagged: html`...`
foo(
  html`<div>
    <p>bar</p>
    foo
  </div>`,
);
foo(html`
  <div>
    <p>bar</p>
    foo
  </div>
`);
const a = (b) =>
  html`<div>
    <p>bar</p>
    foo
  </div>`;
const c = (b) => html`
  <div>
    <p>bar</p>
    foo
  </div>
`;

// Comment: /* HTML */ `...`
foo(
  /* HTML */ `<div>
    <p>bar</p>
    foo
  </div>`,
);
foo(/* HTML */ `
  <div>
    <p>bar</p>
    foo
  </div>
`);
const e = (b) =>
  /* HTML */ `<div>
    <p>bar</p>
    foo
  </div>`;
const g = (b) => /* HTML */ `
  <div>
    <p>bar</p>
    foo
  </div>
`;

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}`

`````js
// Tagged: html`...`
foo(html`
  <div>
    <p>bar</p>
    foo
  </div>
`);
foo(html`
  <div>
    <p>bar</p>
    foo
  </div>
`);
const a = (b) => html`
  <div>
    <p>bar</p>
    foo
  </div>
`;
const c = (b) => html`
  <div>
    <p>bar</p>
    foo
  </div>
`;

// Comment: /* HTML */ `...`
foo(/* HTML */ `
  <div>
    <p>bar</p>
    foo
  </div>
`);
foo(/* HTML */ `
  <div>
    <p>bar</p>
    foo
  </div>
`);
const e = (b) => /* HTML */ `
  <div>
    <p>bar</p>
    foo
  </div>
`;
const g = (b) => /* HTML */ `
  <div>
    <p>bar</p>
    foo
  </div>
`;

`````
