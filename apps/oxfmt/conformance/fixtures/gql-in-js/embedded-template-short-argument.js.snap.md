# Input

`````js
const schema = graphql(`query{users{x}}`);

`````

# Output

## `{"printWidth":80}`

`````js
const schema = graphql(`
  query {
    users {
      x
    }
  }
`);

`````

## `{"printWidth":100}`

`````js
const schema = graphql(`
  query {
    users {
      x
    }
  }
`);

`````
