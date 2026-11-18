# Exit code
1

# stdout
```
  x react(purity): Cannot call impure function during render
   ,-[files/index.jsx:4:13]
 3 | export function A() {
 4 |   const a = Date.now();
   :             ^^^^^|^^^^
   :                  `-- Cannot call impure function
 5 |   return <div>{a}</div>;
   `----
  help: `Date.now` is an impure function. Calling an impure function can produce unstable results that update unpredictably when the component re-renders
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/purity

  x react(purity): Cannot call impure function during render
    ,-[files/index.jsx:11:13]
 10 |   const filler = 0;
 11 |   const b = Math.random() + filler;
    :             ^^^^^^|^^^^^^
    :                   `-- Cannot call impure function
 12 |   return <div>{b}</div>;
    `----
  help: `Math.random` is an impure function. Calling an impure function can produce unstable results that update unpredictably when the component re-renders
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/purity

  x react(set-state-in-effect): Calling setState synchronously within an effect can trigger cascading renders
    ,-[files/index.jsx:19:5]
 16 |   const [name, setName] = useState("");
 17 |   useEffect(() => {
    :   ^^^^|^^^^
    :       `-- This is the containing effect
 18 |     if (!item) return;
 19 |     setName(item.name);
    :     ^^^|^^^
    :        `-- Avoid calling setState() directly within an effect
 20 |     // eslint-disable-next-line react-hooks/exhaustive-deps
    `----
  help: Effects should synchronize React with external systems. Calling setState synchronously inside an effect starts another render and is usually unnecessary. Derive the value during render,
        initialize state directly, or update it from the event that caused the change. Use an effect only when synchronizing with an external system.
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/set-state-in-effect

Found 0 warnings and 3 errors.
Finished in Xms on 1 file with 3 rules using X threads.
```

# stderr
```
```
