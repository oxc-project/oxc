# scripts/fixtures/babel-comments/class_elements-1.js

```js
class C {
  // a
  a() {} // after a
  /* b */ b = 1;
  // end
}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// after a` [26,36]

```diff
+ leading PropertyDefinition[47,53]  // b = 1;
  trailing MethodDefinition[19,25]  // a() {}
```

## `/* b */` [39,46]

```diff
  leading PropertyDefinition[47,53]  // b = 1;
+ trailing MethodDefinition[19,25]  // a() {}
```
