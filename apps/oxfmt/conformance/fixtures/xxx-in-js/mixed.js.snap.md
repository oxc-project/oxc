# Input

`````js
// Multiple embedded languages in one file
const mixedStyles = css`.button{color:red;}`;

const mixedStyled = styled.button`padding:10px;`;

const mixedQuery = gql`query{users{name}}`;

const mixedTemplate = html`<div><h1>Title</h1></div>`;

const mixedDocs = md`#Documentation
This is **important**.`;

// Multi-line with blank lines - should preserve blank lines without trailing whitespace
const multilineCSS = css`
  .foo {
    color: red;
  }

  .bar {
    color: blue;
  }
`;

const multilineGQL = gql`
  type Foo {
    name: String!
  }

  type Bar {
    value: Int!
  }
`;

// Empty - Regular template literals retain newlines and spaces, but embedded ones are condensed
const empty = css``;
const empty2 = styled`
`;
const empty3 = styled.div` `;
const empty4 = gql`   `;
const empty5 = html`

`;
const empty6 = md`

`;

`````

# Output

## `{}`

`````js
// Multiple embedded languages in one file
const mixedStyles = css`
  .button {
    color: red;
  }
`;

const mixedStyled = styled.button`
  padding: 10px;
`;

const mixedQuery = gql`
  query {
    users {
      name
    }
  }
`;

const mixedTemplate = html`<div><h1>Title</h1></div>`;

const mixedDocs = md`
#Documentation
This is **important**.
`;

// Multi-line with blank lines - should preserve blank lines without trailing whitespace
const multilineCSS = css`
  .foo {
    color: red;
  }

  .bar {
    color: blue;
  }
`;

const multilineGQL = gql`
  type Foo {
    name: String!
  }

  type Bar {
    value: Int!
  }
`;

// Empty - Regular template literals retain newlines and spaces, but embedded ones are condensed
const empty = css``;
const empty2 = styled``;
const empty3 = styled.div``;
const empty4 = gql``;
const empty5 = html``;
const empty6 = md``;

`````

## `{"embeddedLanguageFormatting":"off"}`

`````js
// Multiple embedded languages in one file
const mixedStyles = css`.button{color:red;}`;

const mixedStyled = styled.button`padding:10px;`;

const mixedQuery = gql`query{users{name}}`;

const mixedTemplate = html`<div><h1>Title</h1></div>`;

const mixedDocs = md`#Documentation
This is **important**.`;

// Multi-line with blank lines - should preserve blank lines without trailing whitespace
const multilineCSS = css`
  .foo {
    color: red;
  }

  .bar {
    color: blue;
  }
`;

const multilineGQL = gql`
  type Foo {
    name: String!
  }

  type Bar {
    value: Int!
  }
`;

// Empty - Regular template literals retain newlines and spaces, but embedded ones are condensed
const empty = css``;
const empty2 = styled`
`;
const empty3 = styled.div` `;
const empty4 = gql`   `;
const empty5 = html`

`;
const empty6 = md`

`;

`````

## `{"endOfLine":"crlf"}`

`````js
// Multiple embedded languages in one file<CRLF>
const mixedStyles = css`<CRLF>
  .button {<CRLF>
    color: red;<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
const mixedStyled = styled.button`<CRLF>
  padding: 10px;<CRLF>
`;<CRLF>
<CRLF>
const mixedQuery = gql`<CRLF>
  query {<CRLF>
    users {<CRLF>
      name<CRLF>
    }<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
const mixedTemplate = html`<div><h1>Title</h1></div>`;<CRLF>
<CRLF>
const mixedDocs = md`<CRLF>
#Documentation<CRLF>
This is **important**.<CRLF>
`;<CRLF>
<CRLF>
// Multi-line with blank lines - should preserve blank lines without trailing whitespace<CRLF>
const multilineCSS = css`<CRLF>
  .foo {<CRLF>
    color: red;<CRLF>
  }<CRLF>
<CRLF>
  .bar {<CRLF>
    color: blue;<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
const multilineGQL = gql`<CRLF>
  type Foo {<CRLF>
    name: String!<CRLF>
  }<CRLF>
<CRLF>
  type Bar {<CRLF>
    value: Int!<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
// Empty - Regular template literals retain newlines and spaces, but embedded ones are condensed<CRLF>
const empty = css``;<CRLF>
const empty2 = styled``;<CRLF>
const empty3 = styled.div``;<CRLF>
const empty4 = gql``;<CRLF>
const empty5 = html``;<CRLF>
const empty6 = md``;<CRLF>

`````
