# scripts/fixtures/babel-comments/statements-3.js

```js
'use strict' /* d */;
// next
foo
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// next` [22,29]

```diff
  leading ExpressionStatement[30,33]  // foo
+ trailing ExpressionStatement[0,21]  // 'use strict' /* d */;
```
