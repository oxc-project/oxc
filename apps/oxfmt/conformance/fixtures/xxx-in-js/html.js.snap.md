# Input

`````js
// Tagged template literals with html tag
const template = html`<div class="container"><h1>Hello</h1><p>World</p></div>`;

const component = html`<button type="button" onclick="handleClick()">Click</button>`;

`````

# Output

## `{}`

`````js
// Tagged template literals with html tag
const template = html`<div class="container">
  <h1>Hello</h1>
  <p>World</p>
</div>`;

const component = html`<button type="button" onclick="handleClick()">Click</button>`;

`````

## `{"embeddedLanguageFormatting":"off"}`

`````js
// Tagged template literals with html tag
const template = html`<div class="container"><h1>Hello</h1><p>World</p></div>`;

const component = html`<button type="button" onclick="handleClick()">Click</button>`;

`````

## `{"endOfLine":"crlf"}`

`````js
// Tagged template literals with html tag<CRLF>
const template = html`<div class="container"><CRLF>
  <h1>Hello</h1><CRLF>
  <p>World</p><CRLF>
</div>`;<CRLF>
<CRLF>
const component = html`<button type="button" onclick="handleClick()">Click</button>`;<CRLF>

`````
