# scripts/fixtures/babel-comments/top_level_await_reparse-1.js

```js
'use strict'; // c
await /x/u;
import 'z';
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [14,18]

```diff
+ leading ExpressionStatement[19,30]  // await /x/u;
  trailing ExpressionStatement[0,13]  // 'use strict';
```
