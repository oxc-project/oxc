# scripts/fixtures/babel-comments/comma_separated_lists-1.js

```js
f(
  a, // c
  b,
);
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [8,12]

```diff
+ leading Identifier[15,16]  // b
- trailing Identifier[5,6]  // a
```
