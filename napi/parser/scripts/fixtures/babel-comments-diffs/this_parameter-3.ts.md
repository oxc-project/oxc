# scripts/fixtures/babel-comments/this_parameter-3.ts

```ts
function f(this: P, // t
 a, // a
 ...b) {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// t` [20,24]

```diff
+ leading Identifier[26,27]  // a
- trailing Identifier[11,18]  // this: P
```

## `// a` [29,33]

```diff
+ leading RestElement[35,39]  // ...b
- trailing Identifier[26,27]  // a
```
