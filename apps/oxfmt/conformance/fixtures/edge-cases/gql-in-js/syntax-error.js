// Invalid GraphQL: neither Rust nor Prettier can parse it.
// Both sides must leave the template content as-is.
const broken = gql`
  query {
    user(   id:
  }
`;

// Valid template after a broken one, to ensure the failure does not leak.
const ok = gql`
  query {   user( id : 5 ) { name }  }
`;
