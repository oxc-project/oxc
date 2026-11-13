# scripts/fixtures/babel-comments/comma_separated_lists-4.js

```js
function f(
  a, // c
  b,
) {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [17,21]

```diff
+ leading Identifier[24,25]  // b
- trailing Identifier[14,15]  // a
```
