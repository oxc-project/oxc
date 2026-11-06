# Input

`````js
foo(html` <div>a</div> `);
foo(html`<div class="aaaaaaaaaaaaaaaa"><span>bbbbbbbbbbbbbbbbbbbbbbbb</span><span>cccccccccccccccccc</span></div> `);
foo(html`<div class="aaaaaaaaaaaaaaaa"><span>bbbbbbbbbbbbbbbbbbbbbbbb</span><span>cccccccccccccccccc</span></div>
`);
foo(
  html`
    <div>a</div>
  `
);
const a = () => html` <div>a</div> `;
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = foo(html` <div>a</div> `);

`````

# Output

## `{"printWidth":80}`

`````js
foo(html` <div>a</div> `);
foo(
  html`<div class="aaaaaaaaaaaaaaaa">
    <span>bbbbbbbbbbbbbbbbbbbbbbbb</span><span>cccccccccccccccccc</span>
  </div> `,
);
foo(
  html`<div class="aaaaaaaaaaaaaaaa">
    <span>bbbbbbbbbbbbbbbbbbbbbbbb</span><span>cccccccccccccccccc</span>
  </div> `,
);
foo(html` <div>a</div> `);
const a = () => html` <div>a</div> `;
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
  foo(html` <div>a</div> `);

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}`

`````js
foo(html`
  <div>a</div>
`);
foo(html`
  <div class="aaaaaaaaaaaaaaaa">
    <span>bbbbbbbbbbbbbbbbbbbbbbbb</span>
    <span>cccccccccccccccccc</span>
  </div>
`);
foo(html`
  <div class="aaaaaaaaaaaaaaaa">
    <span>bbbbbbbbbbbbbbbbbbbbbbbb</span>
    <span>cccccccccccccccccc</span>
  </div>
`);
foo(html`
  <div>a</div>
`);
const a = () => html`
  <div>a</div>
`;
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = foo(html`
  <div>a</div>
`);

`````
