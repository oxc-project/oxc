# scripts/fixtures/babel-comments/decorators_before_export-2.ts

```ts
foo(); // after foo
@dec export class C {}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// after foo` [7,19]

```diff
+ leading ExportNamedDeclaration[25,42]  // export class C {}
  trailing ExpressionStatement[0,6]  // foo();
```
