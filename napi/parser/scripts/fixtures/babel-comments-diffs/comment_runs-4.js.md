# scripts/fixtures/babel-comments/comment_runs-4.js

```js
foo(); /* x */ bar();
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* x */` [7,14]

```diff
  leading ExpressionStatement[15,21]  // bar();
+ trailing ExpressionStatement[0,6]  // foo();
```
