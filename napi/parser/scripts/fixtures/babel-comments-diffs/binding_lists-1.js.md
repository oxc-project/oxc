# scripts/fixtures/babel-comments/binding_lists-1.js

```js
const {
  a, // c
  b,
} = x;
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [13,17]

```diff
+ leading Property[20,21]  // b
- trailing Property[10,11]  // a
```
