# Input

`````js
foo(/* GraphQL */ `query Q { x }`);
foo(
  /* GraphQL */ `
    query Q { x }
  `
);
foo(/* HTML */ ` <div>a</div> `);
const a = () => /* GraphQL */ `query Q { x }`;
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = foo(/* GraphQL */ `query Q { x }`);

`````

# Output

## `{"printWidth":80}`

`````js
foo(/* GraphQL */ `
  query Q {
    x
  }
`);
foo(/* GraphQL */ `
  query Q {
    x
  }
`);
foo(/* HTML */ ` <div>a</div> `);
const a = () => /* GraphQL */ `
  query Q {
    x
  }
`;
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
  foo(/* GraphQL */ `
    query Q {
      x
    }
  `);

`````

## `{"printWidth":100}`

`````js
foo(/* GraphQL */ `
  query Q {
    x
  }
`);
foo(/* GraphQL */ `
  query Q {
    x
  }
`);
foo(/* HTML */ ` <div>a</div> `);
const a = () => /* GraphQL */ `
  query Q {
    x
  }
`;
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = foo(
  /* GraphQL */ `
    query Q {
      x
    }
  `,
);

`````
