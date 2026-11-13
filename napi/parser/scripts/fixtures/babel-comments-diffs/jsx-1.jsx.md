# scripts/fixtures/babel-comments/jsx-1.jsx

```jsx
<div>{/* child */}<a /* attr */ b={c} /></div>
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `/* attr */` [21,31]

```diff
  leading JSXAttribute[32,37]  // b={c}
+ trailing JSXIdentifier[19,20]  // a
```
