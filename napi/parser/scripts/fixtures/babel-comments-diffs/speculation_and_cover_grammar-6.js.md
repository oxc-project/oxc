# scripts/fixtures/babel-comments/speculation_and_cover_grammar-6.js

```js
({ a = /* @__PURE__ */ f().x } = x);
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* @__PURE__ */` [7,22]

```diff
- leading CallExpression[23,26]  // f()
+ leading MemberExpression[23,28]  // f().x
```
