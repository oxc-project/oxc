# scripts/fixtures/babel-comments/list_boundaries-3.js

```js
function f() { return /* c */ (a); }
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* c */` [22,29]

```diff
- dangling ReturnStatement[15,34]  // return /* c */ (a);
+ leading Identifier[31,32]  // a
```
