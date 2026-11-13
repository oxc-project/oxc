# scripts/fixtures/babel-comments/comma_separated_lists-2.js

```js
x = [
  a, // c
  b,
];
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [11,15]

```diff
+ leading Identifier[18,19]  // b
- trailing Identifier[8,9]  // a
```
