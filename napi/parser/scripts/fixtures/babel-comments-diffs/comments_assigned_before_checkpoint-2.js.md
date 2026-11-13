# scripts/fixtures/babel-comments/comments_assigned_before_checkpoint-2.js

```js
f(a, // c
  async (b, // e
  d));
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// c` [5,9]

```diff
+ leading CallExpression[12,31]  // async (b, // e\n  d)
- trailing Identifier[2,3]  // a
```

## `// e` [22,26]

```diff
+ leading Identifier[29,30]  // d
- trailing Identifier[19,20]  // b
```
