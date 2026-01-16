// Tagged template literals with gql and graphql tags
const query = gql`query GetUser($id:ID!){user(id:$id){name email}}`;

const mutation = graphql`mutation CreatePost($input:PostInput!){createPost(input:$input){id title}}`;
