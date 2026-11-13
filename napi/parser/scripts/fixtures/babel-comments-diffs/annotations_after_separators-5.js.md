# scripts/fixtures/babel-comments/annotations_after_separators-5.js

```js
let a = 1, /* @__PURE__ */
  b = f();
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* @__PURE__ */` [11,26]

```diff
+ leading VariableDeclarator[29,36]  // b = f()
- trailing VariableDeclarator[4,9]  // a = 1
```
