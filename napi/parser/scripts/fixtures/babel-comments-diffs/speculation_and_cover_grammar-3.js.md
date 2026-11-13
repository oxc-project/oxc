# scripts/fixtures/babel-comments/speculation_and_cover_grammar-3.js

```js
({ a /* a */ = 1, b: /* b */ c } = d);
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* a */` [5,12]

```diff
  trailing Identifier[3,4]  // a
  trailing Identifier[3,4]  // a
```
