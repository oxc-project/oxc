# scripts/fixtures/babel-comments/decorators_before_export-4.ts

```ts
foo();
/** doc */
@dec export default class C {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/** doc */` [7,17]

```diff
  leading ExportDefaultDeclaration[23,48]  // export default class C {}
+ trailing ExpressionStatement[0,6]  // foo();
```
