# scripts/fixtures/babel-comments/decorators_before_export-3.ts

```ts
foo();
/** doc */
@dec export class C {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/** doc */` [7,17]

```diff
  leading ExportNamedDeclaration[23,40]  // export class C {}
+ trailing ExpressionStatement[0,6]  // foo();
```
