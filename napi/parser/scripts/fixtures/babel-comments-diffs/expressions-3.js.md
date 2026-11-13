# scripts/fixtures/babel-comments/expressions-3.js

```js
`a${/* in */ b}c`
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* in */` [4,12]

```diff
  leading Identifier[13,14]  // b
+ trailing TemplateElement[1,2]  // a
```
