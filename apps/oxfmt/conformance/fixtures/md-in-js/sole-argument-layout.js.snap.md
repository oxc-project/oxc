# Input

`````js
foo(md`
# hi
`);
foo(
  md`
# hi
`
);
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = foo(markdown`
# hi
`);

`````

# Output

## `{"printWidth":80}`

`````js
foo(md`
# hi
`);
foo(md`
# hi
`);
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
  foo(markdown`
# hi
  `);

`````

## `{"printWidth":100,"proseWrap":"always"}`

`````js
foo(md`
# hi
`);
foo(md`
# hi
`);
const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = foo(
  markdown`
# hi
  `,
);

`````
