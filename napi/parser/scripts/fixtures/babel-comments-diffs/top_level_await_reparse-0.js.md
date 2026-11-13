# scripts/fixtures/babel-comments/top_level_await_reparse-0.js

```js
await /x/u; f(a, /* c */ b); // t
await /x/u; f(a, /* c */ b); // t
await /x/u; f(a, /* c */ b); // t
export {};
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// t` [29,33]

```diff
+ leading ExpressionStatement[34,45]  // await /x/u;
  trailing ExpressionStatement[12,28]  // f(a, /* c */ b);
```

## `// t` [63,67]

```diff
+ leading ExpressionStatement[68,79]  // await /x/u;
  trailing ExpressionStatement[46,62]  // f(a, /* c */ b);
```

## `// t` [97,101]

```diff
+ leading ExportNamedDeclaration[102,112]  // export {};
  trailing ExpressionStatement[80,96]  // f(a, /* c */ b);
```
