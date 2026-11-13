# Input

`````js
// Fragment arguments (draft syntax): graphql-js v17 parses them with `experimentalFragmentArguments` (as Prettier does),
// and so does our apollo-parser fork (`experimental_fragment_arguments`), which the original does not.

// Fragment variable definitions
const fragmentVariableDefs = gql`
  fragment    Foo (  $x : Int   =  3 ) on Bar {
    baz( arg : $x )
  }
`;

// Fragment spread arguments
const fragmentSpreadArgs = gql`
  query {
    ...Foo( x : 1 )
  }
`;

`````

# Output

## `{"printWidth":80}`

`````js
// Fragment arguments (draft syntax): graphql-js v17 parses them with `experimentalFragmentArguments` (as Prettier does),
// and so does our apollo-parser fork (`experimental_fragment_arguments`), which the original does not.

// Fragment variable definitions
const fragmentVariableDefs = gql`
  fragment Foo($x: Int = 3) on Bar {
    baz(arg: $x)
  }
`;

// Fragment spread arguments
const fragmentSpreadArgs = gql`
  query {
    ...Foo(x: 1)
  }
`;

`````

## `{"printWidth":100}`

`````js
// Fragment arguments (draft syntax): graphql-js v17 parses them with `experimentalFragmentArguments` (as Prettier does),
// and so does our apollo-parser fork (`experimental_fragment_arguments`), which the original does not.

// Fragment variable definitions
const fragmentVariableDefs = gql`
  fragment Foo($x: Int = 3) on Bar {
    baz(arg: $x)
  }
`;

// Fragment spread arguments
const fragmentSpreadArgs = gql`
  query {
    ...Foo(x: 1)
  }
`;

`````
