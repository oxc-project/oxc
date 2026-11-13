# scripts/fixtures/babel-comments/top_level_await_reparse-2.js

```js
await /x/u; y = /* @__PURE__ */ f().x; import 'z';
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* @__PURE__ */` [16,31]

```diff
- leading CallExpression[32,35]  // f()
+ leading MemberExpression[32,37]  // f().x
```
