# Exit code
1

# stdout
```
  x react(immutability): Cannot reassign variable after render completes
    ,-[files/hooks.ts:28:7]
 22 | export function useRenderLocal(read: () => number) {
 23 |   let renderLocal: number | undefined;
    :       ^^^^^|^^^^^
    :            `-- `renderLocal` is declared here
 24 |   const getSnapshot = useMemo(() => {
 25 |     let cached: number | undefined;
 26 |     return () => {
 27 |       cached ??= read();
 28 |       renderLocal = cached;
    :       ^^^^^|^^^^^
    :            `-- Cannot reassign `renderLocal` after render completes
 29 |       return cached;
    `----
  help: Reassigning `renderLocal` after render has completed can cause inconsistent behavior on subsequent renders. Consider using state instead
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/immutability

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
