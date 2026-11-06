# Input

`````ts
const GQL_QUERY_WITH_CONST = /* GraphQL */ `
  query S { shop }
` as const;

`````

# Output

## `{"printWidth":80}`

`````ts
const GQL_QUERY_WITH_CONST = /* GraphQL */ `
  query S {
    shop
  }
` as const;

`````

## `{"printWidth":100}`

`````ts
const GQL_QUERY_WITH_CONST = /* GraphQL */ `
  query S {
    shop
  }
` as const;

`````
