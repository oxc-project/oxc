// Draft-spec syntax not supported by apollo-parser.
// Fragment variable definitions: Prettier's graphql-js accepts them
// (experimental), so this must be formatted via the Prettier fallback.
const fragmentVariableDefs = gql`
  fragment    Foo (  $x : Int   =  3 ) on Bar {
    baz( arg : $x )
  }
`;

// Fragment spread arguments: rejected by BOTH apollo-parser and graphql-js.
// Both sides must leave the template content as-is.
const fragmentSpreadArgs = gql`
  query {
    ...Foo( x : 1 )
  }
`;
