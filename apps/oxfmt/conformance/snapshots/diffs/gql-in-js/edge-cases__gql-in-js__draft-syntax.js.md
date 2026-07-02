# edge-cases/gql-in-js/draft-syntax.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -10,7 +10,7 @@
 // Both sides must leave the template content as-is.
 // NOTE: graphql-js v17 which used by Prettier 3.9.x accepts this, need to follow once released
 const fragmentSpreadArgs = gql`
   query {
-    ...Foo(x: 1)
+    ...Foo( x : 1 )
   }
 `;

`````

### Actual (oxfmt)

`````js
// Fragment variable definitions: Prettier's graphql-js v16 accepts them (with experimental options)
// This is not supported by original apollo-parser, but supported by our fork.
const fragmentVariableDefs = gql`
  fragment Foo($x: Int = 3) on Bar {
    baz(arg: $x)
  }
`;

// Fragment spread arguments: rejected by BOTH apollo-parser fork and graphql-js v16.
// Both sides must leave the template content as-is.
// NOTE: graphql-js v17 which used by Prettier 3.9.x accepts this, need to follow once released
const fragmentSpreadArgs = gql`
  query {
    ...Foo( x : 1 )
  }
`;

`````

### Expected (prettier)

`````js
// Fragment variable definitions: Prettier's graphql-js v16 accepts them (with experimental options)
// This is not supported by original apollo-parser, but supported by our fork.
const fragmentVariableDefs = gql`
  fragment Foo($x: Int = 3) on Bar {
    baz(arg: $x)
  }
`;

// Fragment spread arguments: rejected by BOTH apollo-parser fork and graphql-js v16.
// Both sides must leave the template content as-is.
// NOTE: graphql-js v17 which used by Prettier 3.9.x accepts this, need to follow once released
const fragmentSpreadArgs = gql`
  query {
    ...Foo(x: 1)
  }
`;

`````

## Option 2

`````json
{"printWidth":100}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -10,7 +10,7 @@
 // Both sides must leave the template content as-is.
 // NOTE: graphql-js v17 which used by Prettier 3.9.x accepts this, need to follow once released
 const fragmentSpreadArgs = gql`
   query {
-    ...Foo(x: 1)
+    ...Foo( x : 1 )
   }
 `;

`````

### Actual (oxfmt)

`````js
// Fragment variable definitions: Prettier's graphql-js v16 accepts them (with experimental options)
// This is not supported by original apollo-parser, but supported by our fork.
const fragmentVariableDefs = gql`
  fragment Foo($x: Int = 3) on Bar {
    baz(arg: $x)
  }
`;

// Fragment spread arguments: rejected by BOTH apollo-parser fork and graphql-js v16.
// Both sides must leave the template content as-is.
// NOTE: graphql-js v17 which used by Prettier 3.9.x accepts this, need to follow once released
const fragmentSpreadArgs = gql`
  query {
    ...Foo( x : 1 )
  }
`;

`````

### Expected (prettier)

`````js
// Fragment variable definitions: Prettier's graphql-js v16 accepts them (with experimental options)
// This is not supported by original apollo-parser, but supported by our fork.
const fragmentVariableDefs = gql`
  fragment Foo($x: Int = 3) on Bar {
    baz(arg: $x)
  }
`;

// Fragment spread arguments: rejected by BOTH apollo-parser fork and graphql-js v16.
// Both sides must leave the template content as-is.
// NOTE: graphql-js v17 which used by Prettier 3.9.x accepts this, need to follow once released
const fragmentSpreadArgs = gql`
  query {
    ...Foo(x: 1)
  }
`;

`````
