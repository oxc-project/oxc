# Input

`````js
// Regular JavaScript - No embedded languages (should not be affected)
function greet(name) {
  return `Hello, ${name}!`;
}

const message = `This is a regular template string`;

class Formatter {
  format(text) {
    return text.trim();
  }
}

// prettier-ignore - Should not format
// prettier-ignore
const unformattedCss = css`.button{color:red;background:blue;}`;

const formattedCss = css`.container{display:flex;align-items:center;}`;

// prettier-ignore
const unformattedGql = gql`query GetUser($id:ID!){user(id:$id){name email}}`;

const formattedGql = gql`query GetPosts{posts{title author}}`;

// Unsupported tags - Should not format
const unknown = customTag`This won't be formatted`;

const sqlQuery = sql`SELECT * FROM users WHERE id = 1`;

// Invalid syntax in supported tag - Should not format
const invalidCss = css`
  repeating-linear-gradient(
    0deg,
var(--color),
    transparent 3px
  );
`;

`````

# Output

## `{}`

`````js
// Regular JavaScript - No embedded languages (should not be affected)
function greet(name) {
  return `Hello, ${name}!`;
}

const message = `This is a regular template string`;

class Formatter {
  format(text) {
    return text.trim();
  }
}

// prettier-ignore - Should not format
// prettier-ignore
const unformattedCss = css`.button{color:red;background:blue;}`;

const formattedCss = css`
  .container {
    display: flex;
    align-items: center;
  }
`;

// prettier-ignore
const unformattedGql = gql`query GetUser($id:ID!){user(id:$id){name email}}`;

const formattedGql = gql`
  query GetPosts {
    posts {
      title
      author
    }
  }
`;

// Unsupported tags - Should not format
const unknown = customTag`This won't be formatted`;

const sqlQuery = sql`SELECT * FROM users WHERE id = 1`;

// Invalid syntax in supported tag - Should not format
const invalidCss = css`
  repeating-linear-gradient(
    0deg,
var(--color),
    transparent 3px
  );
`;

`````

## `{"embeddedLanguageFormatting":"off"}`

`````js
// Regular JavaScript - No embedded languages (should not be affected)
function greet(name) {
  return `Hello, ${name}!`;
}

const message = `This is a regular template string`;

class Formatter {
  format(text) {
    return text.trim();
  }
}

// prettier-ignore - Should not format
// prettier-ignore
const unformattedCss = css`.button{color:red;background:blue;}`;

const formattedCss = css`.container{display:flex;align-items:center;}`;

// prettier-ignore
const unformattedGql = gql`query GetUser($id:ID!){user(id:$id){name email}}`;

const formattedGql = gql`query GetPosts{posts{title author}}`;

// Unsupported tags - Should not format
const unknown = customTag`This won't be formatted`;

const sqlQuery = sql`SELECT * FROM users WHERE id = 1`;

// Invalid syntax in supported tag - Should not format
const invalidCss = css`
  repeating-linear-gradient(
    0deg,
var(--color),
    transparent 3px
  );
`;

`````

## `{"endOfLine":"crlf"}`

`````js
// Regular JavaScript - No embedded languages (should not be affected)<CRLF>
function greet(name) {<CRLF>
  return `Hello, ${name}!`;<CRLF>
}<CRLF>
<CRLF>
const message = `This is a regular template string`;<CRLF>
<CRLF>
class Formatter {<CRLF>
  format(text) {<CRLF>
    return text.trim();<CRLF>
  }<CRLF>
}<CRLF>
<CRLF>
// prettier-ignore - Should not format<CRLF>
// prettier-ignore<CRLF>
const unformattedCss = css`.button{color:red;background:blue;}`;<CRLF>
<CRLF>
const formattedCss = css`<CRLF>
  .container {<CRLF>
    display: flex;<CRLF>
    align-items: center;<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
// prettier-ignore<CRLF>
const unformattedGql = gql`query GetUser($id:ID!){user(id:$id){name email}}`;<CRLF>
<CRLF>
const formattedGql = gql`<CRLF>
  query GetPosts {<CRLF>
    posts {<CRLF>
      title<CRLF>
      author<CRLF>
    }<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
// Unsupported tags - Should not format<CRLF>
const unknown = customTag`This won't be formatted`;<CRLF>
<CRLF>
const sqlQuery = sql`SELECT * FROM users WHERE id = 1`;<CRLF>
<CRLF>
// Invalid syntax in supported tag - Should not format<CRLF>
const invalidCss = css`<CRLF>
  repeating-linear-gradient(<CRLF>
    0deg,<CRLF>
var(--color),<CRLF>
    transparent 3px<CRLF>
  );<CRLF>
`;<CRLF>

`````
