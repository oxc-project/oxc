# Exit code
1

# stdout
```
  x react(purity): Cannot call impure function during render
    ,-[files/index.jsx:32:32]
 31 | export function RenderMap(items) {
 32 |   return <div>{items.map(() => Date.now())}</div>;
    :                                ^^^^^|^^^^
    :                                     `-- Cannot call impure function
 33 | }
    `----
  help: `Date.now` is an impure function. Calling an impure function can produce unstable results that update unpredictably when the component re-renders
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/purity

  x react(purity): Cannot call impure function during render
    ,-[files/index.jsx:35:35]
 34 | export function RenderFilter(items) {
 35 |   return <div>{items.filter(() => new Date()).length}</div>;
    :                                   ^^^^^|^^^^
    :                                        `-- Cannot call impure function
 36 | }
    `----
  help: `Date` is an impure function. Calling an impure function can produce unstable results that update unpredictably when the component re-renders
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/purity

  x react(purity): Cannot call impure function during render
    ,-[files/index.jsx:38:25]
 37 | export function NestedRenderMap(items) {
 38 |   const getTime = () => new Date();
    :                         ^^^^^|^^^^
    :                              `-- Cannot call impure function
 39 |   return <div>{items.map(() => getTime())}</div>;
    `----
  help: `Date` is an impure function. Calling an impure function can produce unstable results that update unpredictably when the component re-renders
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/purity

  x react(purity): Cannot call impure function during render
    ,-[files/index.jsx:42:25]
 41 | export function DirectCall() {
 42 |   const getTime = () => Date.now();
    :                         ^^^^^|^^^^
    :                              `-- Cannot call impure function
 43 |   return <div>{getTime()}</div>;
    `----
  help: `Date.now` is an impure function. Calling an impure function can produce unstable results that update unpredictably when the component re-renders
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/purity

Found 0 warnings and 4 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
