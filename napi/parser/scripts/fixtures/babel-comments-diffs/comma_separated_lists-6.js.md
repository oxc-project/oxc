# scripts/fixtures/babel-comments/comma_separated_lists-6.js

```js
[
  a, // c
  b,
] = x;
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [7,11]

```diff
+ leading Identifier[14,15]  // b
- trailing Identifier[4,5]  // a
```
