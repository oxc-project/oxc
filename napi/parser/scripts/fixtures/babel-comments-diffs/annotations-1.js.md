# scripts/fixtures/babel-comments/annotations-1.js

```js
/* @__PURE__ */ foo(); /* #__NO_SIDE_EFFECTS__ */ function g() {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* @__PURE__ */` [0,15]

```diff
- leading CallExpression[16,21]  // foo()
+ leading ExpressionStatement[16,22]  // foo();
```

## `/* #__NO_SIDE_EFFECTS__ */` [23,49]

```diff
  leading FunctionDeclaration[50,65]  // function g() {}
+ trailing ExpressionStatement[16,22]  // foo();
```
