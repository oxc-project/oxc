# Exit code
1

# stdout
```
  x react(capitalized-calls): Capitalized function called without JSX
    ,-[files/index.jsx:19:16]
 18 | export function NotAllowed() {
 19 |   return <div>{Other()}</div>;
    :                ^^|^^
    :                  `-- `Other` may be a component
 20 | }
    `----
  help: Render `Other` with JSX if it is a component; otherwise rename it to start with a lowercase letter or allowlist it in the compiler configuration
  note: `Other` is treated as a component because it begins with an uppercase letter; React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/
           eslint-plugin-react-hooks/lints/static-components

  x react(capitalized-calls): Capitalized function called without JSX
    ,-[files/index.jsx:24:16]
 23 | export function PatternDoesNotMatch() {
 24 |   return <div>{EventChild()}</div>;
    :                ^^^^^|^^^^
    :                     `-- `EventChild` may be a component
 25 | }
    `----
  help: Render `EventChild` with JSX if it is a component; otherwise rename it to start with a lowercase letter or allowlist it in the compiler configuration
  note: `EventChild` is treated as a component because it begins with an uppercase letter; React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/
           reference/eslint-plugin-react-hooks/lints/static-components

  x react(capitalized-calls): Capitalized function called without JSX
    ,-[files/index.jsx:29:25]
 28 | export function MethodCall() {
 29 |   return <div>{myModule.Child()}</div>;
    :                         ^^|^^
    :                           `-- `Child` may be a component
 30 | }
    `----
  help: Render `Child` with JSX if it is a component; otherwise rename it to start with a lowercase letter or allowlist it in the compiler configuration
  note: `Child` is treated as a component because it begins with an uppercase letter; React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/
           eslint-plugin-react-hooks/lints/static-components

Found 0 warnings and 3 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
