# Exit code
1

# stdout
```
  x react(immutability): Cannot modify local variables after render completes
   ,-[files/non-imported.jsx:5:34]
 4 |   const sharedVal = useSharedValue(0);
 5 |   return <Button onPress={() => (sharedVal.value = Math.random())} title="Randomize" />;
   :                           ^|     ^^^^|^^^^
   :                            |         `-- This modifies `sharedVal`
   :                            `-- This function may (indirectly) reassign or modify `sharedVal` after render
 6 | }
   `----
  help: This argument is a function which may reassign or mutate `sharedVal` after render, which can cause inconsistent behavior on subsequent renders. Consider using state instead
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/immutability

  x react(immutability): This value cannot be modified
   ,-[files/non-imported.jsx:5:34]
 3 | export function SomeComponent() {
 4 |   const sharedVal = useSharedValue(0);
   :         ^^^^|^^^^
   :             `-- `sharedVal` originates here
 5 |   return <Button onPress={() => (sharedVal.value = Math.random())} title="Randomize" />;
   :                                  ^^^^|^^^^
   :                                      `-- `sharedVal` cannot be modified
 6 | }
   `----
  help: Modifying a value returned from a hook is not allowed. Consider moving the modification into the hook where the value is constructed
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/immutability

Found 0 warnings and 2 errors.
Finished in Xms on 2 files with 2 rules using X threads.
```

# stderr
```
```
