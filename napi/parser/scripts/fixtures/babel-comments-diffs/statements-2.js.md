# scripts/fixtures/babel-comments/statements-2.js

```js
// a
foo(); // b
/* c */ bar();
// d
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// b` [12,16]

```diff
+ leading ExpressionStatement[25,31]  // bar();
  trailing ExpressionStatement[5,11]  // foo();
```

## `/* c */` [17,24]

```diff
  leading ExpressionStatement[25,31]  // bar();
+ trailing ExpressionStatement[5,11]  // foo();
```
