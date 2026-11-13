# scripts/fixtures/babel-comments/annotations_among_leading_comments-1.js

```js
// c
/* @__PURE__ */ foo();
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* @__PURE__ */` [5,20]

```diff
- leading CallExpression[21,26]  // foo()
+ leading ExpressionStatement[21,27]  // foo();
```
