# Exit code
1

# stdout
```
  x react(exhaustive-effect-dependencies): Found missing effect dependencies
   ,-[files/index.jsx:6:6]
 4 |   useEffect(() => {
 5 |     log(value);
   :         ^^|^^
   :           `-- Missing dependency `value`
 6 |   }, []);
   :      ^|
   :       `-- This dependency list is missing values used by the callback
 7 |   // oxlint-disable-next-line react/purity
   `----
  help: Missing dependencies can cause an effect to fire less often than it should
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/exhaustive-deps

  x react(purity): Cannot call impure function during render
   ,-[files/index.jsx:8:16]
 7 |   // oxlint-disable-next-line react/purity
 8 |   const rand = Math.random();
   :                ^^^^^^|^^^^^^
   :                      `-- Cannot call impure function
 9 |   return <div>{rand}</div>;
   `----
  help: `Math.random` is an impure function. Calling an impure function can produce unstable results that update unpredictably when the component re-renders
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/purity

  x react(exhaustive-effect-dependencies): Found missing effect dependencies
    ,-[files/index.jsx:15:6]
 13 |   useEffect(() => {
 14 |     log(value);
    :         ^^|^^
    :           `-- Missing dependency `value`
 15 |   }, []);
    :      ^|
    :       `-- This dependency list is missing values used by the callback
 16 |   return null;
    `----
  help: Missing dependencies can cause an effect to fire less often than it should
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/exhaustive-deps

Found 0 warnings and 3 errors.
Finished in Xms on 1 file with 3 rules using X threads.
```

# stderr
```
```
