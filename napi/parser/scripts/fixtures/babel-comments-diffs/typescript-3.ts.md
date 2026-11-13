# scripts/fixtures/babel-comments/typescript-3.ts

```ts
new Foo /* a */ <T> /* b */ ();
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* a */` [8,15]

```diff
  leading TSTypeParameterInstantiation[16,19]  // <T>
+ trailing Identifier[4,7]  // Foo
```

## `/* b */` [20,27]

Babel does not attach this comment to any node.

```diff
- trailing TSTypeParameterInstantiation[16,19]  // <T>
```
