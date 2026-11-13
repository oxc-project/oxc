# scripts/fixtures/babel-comments/comma_separated_lists-5.js

```js
let a = 1, // c
  b = 2;
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [11,15]

```diff
+ leading VariableDeclarator[18,23]  // b = 2
- trailing VariableDeclarator[4,9]  // a = 1
```
