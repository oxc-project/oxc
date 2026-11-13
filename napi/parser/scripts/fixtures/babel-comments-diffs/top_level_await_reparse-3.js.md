# scripts/fixtures/babel-comments/top_level_await_reparse-3.js

```js
await /* @__PURE__ */ f().x; import 'z';
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* @__PURE__ */` [6,21]

```diff
- leading CallExpression[22,25]  // f()
+ leading MemberExpression[22,27]  // f().x
```
