# Exit code
1

# stdout
```
  x react(globals): Cannot reassign variables declared outside of the component/hook
   ,-[files/arrow-hook.js:8:3]
 7 |   const [step] = useState(1);
 8 |   count = count + step;
   :   ^^|^^
   :     `-- `count` cannot be reassigned
 9 |   return count;
   `----
  help: Variable `count` is declared outside of the component/hook. Reassigning this value during render is a side effect which can cause unpredictable behavior. If this variable is used in
        rendering, use useState instead. Otherwise, update it in an effect
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/globals

  x react(globals): Cannot reassign variables declared outside of the component/hook
   ,-[files/module-level-probe.test.tsx:7:3]
 6 | export function Probe() {
 7 |   result = useState(1)[0];
   :   ^^^|^^
   :      `-- `result` cannot be reassigned
 8 |   return null;
   `----
  help: Variable `result` is declared outside of the component/hook. Reassigning this value during render is a side effect which can cause unpredictable behavior. If this variable is used in
        rendering, use useState instead. Otherwise, update it in an effect
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/globals

Found 0 warnings and 2 errors.
Finished in Xms on 4 files with 2 rules using X threads.
```

# stderr
```
```
