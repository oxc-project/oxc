# scripts/fixtures/babel-comments/typescript-2.ts

```ts
class A { /* before */ @dec /* after dec */ method() {} }
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* after dec */` [28,43]

```diff
  leading Identifier[44,50]  // method
+ trailing Decorator[23,27]  // @dec
```
