# scripts/fixtures/babel-comments/comma_separated_lists-3.js

```js
x = {
  a: 1, // c
  b: 2,
};
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [14,18]

```diff
+ leading Property[21,25]  // b: 2
- trailing Property[8,12]  // a: 1
```
