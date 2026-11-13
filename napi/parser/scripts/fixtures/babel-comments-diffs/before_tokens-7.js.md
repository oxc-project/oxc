# scripts/fixtures/babel-comments/before_tokens-7.js

```js
[a /* a */, , /* e */ , b]; ({ a: 1 /* b */, c });
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* e */` [14,21]

Oxc does not expose this comment on any node.

```diff
+ dangling ArrayExpression[0,26]  // [a /* a */, , /* e */ , b]
```
