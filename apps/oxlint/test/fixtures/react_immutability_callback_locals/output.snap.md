# Exit code
1

# stdout
```
  x react(immutability): Cannot modify local variables after render completes
    ,-[files/hooks.ts:52:5]
 50 |   let count = 0;
 51 |   useEffect(() => {
    :             ^^^|^^^
    :                `-- This function may (indirectly) reassign or modify `count` after render
 52 |     count = 1;
    :     ^^|^^
    :       `-- This modifies `count`
 53 |   });
    `----
  help: This argument is a function which may reassign or mutate `count` after render, which can cause inconsistent behavior on subsequent renders. Consider using state instead
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/immutability

  x react(immutability): Cannot reassign variable after render completes
    ,-[files/hooks.ts:52:5]
 49 | export function useCounter() {
 50 |   let count = 0;
    :       ^^|^^
    :         `-- `count` is declared here
 51 |   useEffect(() => {
 52 |     count = 1;
    :     ^^|^^
    :       `-- Cannot reassign `count` after render completes
 53 |   });
    `----
  help: Reassigning `count` after render has completed can cause inconsistent behavior on subsequent renders. Consider using state instead
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/immutability

Found 0 warnings and 2 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
