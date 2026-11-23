# Input

`````js
// Tagged template literals with gql and graphql tags
const query = gql`query GetUser($id:ID!){user(id:$id){name email}}`;

const mutation = graphql`mutation CreatePost($input:PostInput!){createPost(input:$input){id title}}`;

// graphql() function call - single argument (hugging layout)
const schema = graphql(`query{users{name email}}`);

// graphql() function call - multiple arguments
graphql(schema, `mutation MarkReadNotificationMutation($input:MarkReadNotificationData!){markReadNotification(data:$input){notification{seenState}}}`)

// graphql() function call - empty
graphql(``);

// Non-target: gql() is NOT recognized as a function call pattern
gql(`query{users{name}}`);

// Non-target: other function names
someFunction(`query{users{name}}`);

`````

# Output

## `{}`

`````js
// Tagged template literals with gql and graphql tags
const query = gql`
  query GetUser($id: ID!) {
    user(id: $id) {
      name
      email
    }
  }
`;

const mutation = graphql`
  mutation CreatePost($input: PostInput!) {
    createPost(input: $input) {
      id
      title
    }
  }
`;

// graphql() function call - single argument (hugging layout)
const schema = graphql(`
  query {
    users {
      name
      email
    }
  }
`);

// graphql() function call - multiple arguments
graphql(
  schema,
  `
    mutation MarkReadNotificationMutation($input: MarkReadNotificationData!) {
      markReadNotification(data: $input) {
        notification {
          seenState
        }
      }
    }
  `,
);

// graphql() function call - empty
graphql(``);

// Non-target: gql() is NOT recognized as a function call pattern
gql(`query{users{name}}`);

// Non-target: other function names
someFunction(`query{users{name}}`);

`````

## `{"embeddedLanguageFormatting":"off"}`

`````js
// Tagged template literals with gql and graphql tags
const query = gql`query GetUser($id:ID!){user(id:$id){name email}}`;

const mutation = graphql`mutation CreatePost($input:PostInput!){createPost(input:$input){id title}}`;

// graphql() function call - single argument (hugging layout)
const schema = graphql(`query{users{name email}}`);

// graphql() function call - multiple arguments
graphql(
  schema,
  `mutation MarkReadNotificationMutation($input:MarkReadNotificationData!){markReadNotification(data:$input){notification{seenState}}}`,
);

// graphql() function call - empty
graphql(``);

// Non-target: gql() is NOT recognized as a function call pattern
gql(`query{users{name}}`);

// Non-target: other function names
someFunction(`query{users{name}}`);

`````

## `{"endOfLine":"crlf"}`

`````js
// Tagged template literals with gql and graphql tags<CRLF>
const query = gql`<CRLF>
  query GetUser($id: ID!) {<CRLF>
    user(id: $id) {<CRLF>
      name<CRLF>
      email<CRLF>
    }<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
const mutation = graphql`<CRLF>
  mutation CreatePost($input: PostInput!) {<CRLF>
    createPost(input: $input) {<CRLF>
      id<CRLF>
      title<CRLF>
    }<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
// graphql() function call - single argument (hugging layout)<CRLF>
const schema = graphql(`<CRLF>
  query {<CRLF>
    users {<CRLF>
      name<CRLF>
      email<CRLF>
    }<CRLF>
  }<CRLF>
`);<CRLF>
<CRLF>
// graphql() function call - multiple arguments<CRLF>
graphql(<CRLF>
  schema,<CRLF>
  `<CRLF>
    mutation MarkReadNotificationMutation($input: MarkReadNotificationData!) {<CRLF>
      markReadNotification(data: $input) {<CRLF>
        notification {<CRLF>
          seenState<CRLF>
        }<CRLF>
      }<CRLF>
    }<CRLF>
  `,<CRLF>
);<CRLF>
<CRLF>
// graphql() function call - empty<CRLF>
graphql(``);<CRLF>
<CRLF>
// Non-target: gql() is NOT recognized as a function call pattern<CRLF>
gql(`query{users{name}}`);<CRLF>
<CRLF>
// Non-target: other function names<CRLF>
someFunction(`query{users{name}}`);<CRLF>

`````
