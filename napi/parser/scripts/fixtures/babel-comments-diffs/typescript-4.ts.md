# scripts/fixtures/babel-comments/typescript-4.ts

```ts
class A extends B/* a */<T>/* b */ {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* a */` [17,24]

```diff
  leading TSTypeParameterInstantiation[24,27]  // <T>
+ trailing Identifier[16,17]  // B
```

## `/* b */` [27,34]

```diff
  leading ClassBody[35,37]  // {}
+ trailing TSTypeParameterInstantiation[24,27]  // <T>
```
