# scripts/fixtures/babel-comments/comment_runs-3.js

```js
foo();
/* x */ // c
bar();
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* x */` [7,14]

```diff
  leading ExpressionStatement[20,26]  // bar();
+ trailing ExpressionStatement[0,6]  // foo();
```

## `// c` [15,19]

```diff
  leading ExpressionStatement[20,26]  // bar();
+ trailing ExpressionStatement[0,6]  // foo();
```
