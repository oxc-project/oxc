# scripts/fixtures/babel-comments/comma_separated_lists-7.js

```js
f(
  a, /* x */ // c
  b,
);
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* x */` [8,15]

```diff
+ leading Identifier[23,24]  // b
- trailing Identifier[5,6]  // a
```

## `// c` [16,20]

```diff
+ leading Identifier[23,24]  // b
- trailing Identifier[5,6]  // a
```
