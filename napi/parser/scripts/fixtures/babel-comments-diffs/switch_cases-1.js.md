# scripts/fixtures/babel-comments/switch_cases-1.js

```js
switch (x) {
  // a
  case 1:
    foo();
    // falls through
  case 2: // two
    bar();
  // end
}
```

`-` lines are Oxc only, `+` lines are Babel only, unprefixed lines are shared.

## `// falls through` [45,61]

```diff
  leading SwitchCase[64,89]  // case 2: // two\n    bar();
+ trailing SwitchCase[22,40]  // case 1:\n    foo();
```
