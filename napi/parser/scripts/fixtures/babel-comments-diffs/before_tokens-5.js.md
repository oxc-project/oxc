# scripts/fixtures/babel-comments/before_tokens-5.js

```js
x = (/* a */ y /* b */); z = (/* c */ y, w /* d */);
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* d */` [43,50]

```diff
+ trailing Identifier[41,42]  // w
- trailing SequenceExpression[38,42]  // y, w
```
