# Input

`````js
foo(
  gql`
    query {{{
  `
);

`````

# Output

## `{"printWidth":80}`

`````js
foo(gql`
    query {{{
  `);

`````

## `{"printWidth":100}`

`````js
foo(gql`
    query {{{
  `);

`````
