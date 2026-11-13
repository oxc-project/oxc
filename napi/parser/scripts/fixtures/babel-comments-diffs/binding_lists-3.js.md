# scripts/fixtures/babel-comments/binding_lists-3.js

```js
function f(
  a, /* x */ // c
  b,
) {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* x */` [17,24]

```diff
+ leading Identifier[32,33]  // b
- trailing Identifier[14,15]  // a
```

## `// c` [25,29]

```diff
+ leading Identifier[32,33]  // b
- trailing Identifier[14,15]  // a
```
