# Input

`````js
foo(css`color: red;`);
foo(css`
  color: red;
`);
foo(
  css`
    color: red;
  `
);
this.foo.bar(css`color: red;`);
const a = () => css`color: red;`;
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = foo(css`color: red;`);
foo(css // comment
`color: red;`);

`````

# Output

## `{"printWidth":80}`

`````js
foo(css`
  color: red;
`);
foo(css`
  color: red;
`);
foo(css`
  color: red;
`);
this.foo.bar(css`
  color: red;
`);
const a = () => css`
  color: red;
`;
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
  foo(css`
    color: red;
  `);
foo(
  css // comment
  `
    color: red;
  `,
);

`````

## `{"printWidth":100}`

`````js
foo(css`
  color: red;
`);
foo(css`
  color: red;
`);
foo(css`
  color: red;
`);
this.foo.bar(css`
  color: red;
`);
const a = () => css`
  color: red;
`;
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = foo(css`
  color: red;
`);
foo(
  css // comment
  `
    color: red;
  `,
);

`````
