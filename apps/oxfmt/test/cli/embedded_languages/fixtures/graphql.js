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
