# Input

`````js
_ = gql`

                  ${
                    a
                    // comment
                    + b}

`;

`````

# Output

## `{"printWidth":80}`

`````js
_ = gql`
  ${
    a +
    // comment
    b
  }
`;

`````

## `{"printWidth":100}`

`````js
_ = gql`
  ${
    a +
    // comment
    b
  }
`;

`````
