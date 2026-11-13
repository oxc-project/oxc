# scripts/fixtures/babel-comments/annotations_after_separators-3.js

```js
f(a, /* @__PURE__ */
  b);
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* @__PURE__ */` [5,20]

```diff
+ leading Identifier[23,24]  // b
- trailing Identifier[2,3]  // a
```
