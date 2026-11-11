# Input

`````js
_ = css`
  a{
    color:
                  ${
                    a
                    // comment
                    + b}
    ;
  }
`;

`````

# Output

## `{"printWidth":80}`

`````js
_ = css`
  a {
    color: ${
      a +
      // comment
      b
    };
  }
`;

`````

## `{"printWidth":100}`

`````js
_ = css`
  a {
    color: ${
      a +
      // comment
      b
    };
  }
`;

`````
