# scripts/fixtures/babel-comments/annotations_after_separators-4.js

```js
function h(a, /* @__PURE__ */
  b) {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* @__PURE__ */` [14,29]

```diff
+ leading Identifier[32,33]  // b
- trailing Identifier[11,12]  // a
```
