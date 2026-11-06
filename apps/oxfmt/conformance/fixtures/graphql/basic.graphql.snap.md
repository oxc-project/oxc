# Input

`````graphql
query { user(id:1){ name  email } }
`````

# Output

## `{}`

`````graphql
query {
  user(id: 1) {
    name
    email
  }
}

`````
