# scripts/fixtures/babel-comments/empty_containers-1.js

```js
function f(/* params */) { /* body */ }
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* params */` [11,23]

Oxc does not expose this comment on any node.

```diff
+ dangling FunctionDeclaration[0,39]  // function f(/* params */) { /* body */ }
```
