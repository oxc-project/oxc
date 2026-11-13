# scripts/fixtures/babel-comments/speculation_and_cover_grammar-2.ts

```ts
let f = <T,>(/* a */ a: T) => a; x = a < /* b */ b > c;
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* a */` [13,20]

Babel does not attach this comment to any node.

```diff
- leading Identifier[21,25]  // a: T
```
