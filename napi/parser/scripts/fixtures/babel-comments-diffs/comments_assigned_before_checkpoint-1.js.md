# scripts/fixtures/babel-comments/comments_assigned_before_checkpoint-1.js

```js
f(a, // c
  (b, // e
  d));
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [5,9]

```diff
+ leading SequenceExpression[13,24]  // b, // e\n  d
- trailing Identifier[2,3]  // a
```

## `// e` [16,20]

```diff
+ leading Identifier[23,24]  // d
- trailing Identifier[13,14]  // b
```
