# Exit code
1

# stdout
```
  x react(rule-suppression): React rule suppression prevents optimization
   ,-[files/index.jsx:3:3]
 2 | export function Component() {
 3 |   // eslint-disable-next-line react-hooks/exhaustive-deps
   :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^|^^^^^^^^^^^^^^^^^^^^^^^^^^^
   :                              `-- Found React rule suppression
 4 |   const filler = 0;
   `----
  help: Remove the suppression and address the reported React rule violation
  note: React Compiler has skipped optimizing this component because one or more React ESLint rules were disabled. React Compiler only works when your components follow all the rules of React,
           disabling them may result in unexpected or incorrect behavior. Found suppression `eslint-disable-next-line react-hooks/exhaustive-deps`. Additional guidance: https://react.dev/
           reference/eslint-plugin-react-hooks

  x react(purity): Cannot call impure function during render
   ,-[files/index.jsx:5:16]
 4 |   const filler = 0;
 5 |   const rand = Math.random() + filler;
   :                ^^^^^^|^^^^^^
   :                      `-- Cannot call impure function
 6 |   return <div>{rand}</div>;
   `----
  help: `Math.random` is an impure function. Calling an impure function can produce unstable results that update unpredictably when the component re-renders
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/purity

  x react(set-state-in-effect): Calling setState synchronously within an effect can trigger cascading renders
    ,-[files/index.jsx:12:5]
  9 |   const [name, setName] = useState("");
 10 |   useEffect(() => {
    :   ^^^^|^^^^
    :       `-- This is the containing effect
 11 |     if (!item) return;
 12 |     setName(item.name);
    :     ^^^|^^^
    :        `-- Avoid calling setState() directly within an effect
 13 |     // eslint-disable-next-line react-hooks/exhaustive-deps
    `----
  help: Effects should synchronize React with external systems. Calling setState synchronously inside an effect starts another render and is usually unnecessary. Derive the value during render,
        initialize state directly, or update it from the event that caused the change. Use an effect only when synchronizing with an external system.
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/set-state-in-effect

  x react(rule-suppression): React rule suppression prevents optimization
    ,-[files/index.jsx:13:5]
 12 |     setName(item.name);
 13 |     // eslint-disable-next-line react-hooks/exhaustive-deps
    :     ^^^^^^^^^^^^^^^^^^^^^^^^^^^|^^^^^^^^^^^^^^^^^^^^^^^^^^^
    :                                `-- Found React rule suppression
 14 |   }, [item?.id]);
    `----
  help: Remove the suppression and address the reported React rule violation
  note: React Compiler has skipped optimizing this component because one or more React ESLint rules were disabled. React Compiler only works when your components follow all the rules of React,
           disabling them may result in unexpected or incorrect behavior. Found suppression `eslint-disable-next-line react-hooks/exhaustive-deps`. Additional guidance: https://react.dev/
           reference/eslint-plugin-react-hooks

Found 0 warnings and 4 errors.
Finished in Xms on 1 file with 3 rules using X threads.
```

# stderr
```
```
