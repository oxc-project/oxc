# scripts/fixtures/babel-comments/decorators_before_export-5.ts

```ts
@a /* between */ @b export class C {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* between */` [3,16]

```diff
+ leading Decorator[17,19]  // @b
- leading ExportNamedDeclaration[20,37]  // export class C {}
+ trailing Decorator[0,2]  // @a
```
