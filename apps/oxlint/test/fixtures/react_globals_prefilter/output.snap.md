# Exit code
1

# stdout
```
  x react(globals): Cannot reassign variables declared outside of the component/hook
   ,-[files/memo.jsx:6:3]
 5 | export const Component = React.memo(function (props) {
 6 |   someGlobal = true;
   :   ^^^^^|^^^^
   :        `-- `someGlobal` cannot be reassigned
 7 |   return <div>{props.text}</div>;
   `----
  help: Variable `someGlobal` is declared outside of the component/hook. Reassigning this value during render is a side effect which can cause unpredictable behavior. If this variable is used in
        rendering, use useState instead. Otherwise, update it in an effect
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/globals

  x react(globals): Cannot reassign variables declared outside of the component/hook
   ,-[files/provider.ts:8:3]
 7 |   const [value] = useState(0);
 8 |   someGlobal = true;
   :   ^^^^^|^^^^
   :        `-- `someGlobal` cannot be reassigned
 9 |   return createElement("div", null, value, props.children);
   `----
  help: Variable `someGlobal` is declared outside of the component/hook. Reassigning this value during render is a side effect which can cause unpredictable behavior. If this variable is used in
        rendering, use useState instead. Otherwise, update it in an effect
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/globals

  x react(globals): Cannot reassign variables declared outside of the component/hook
   ,-[files/use-foo.js:7:3]
 6 | export function useFoo() {
 7 |   someGlobal = true;
   :   ^^^^^|^^^^
   :        `-- `someGlobal` cannot be reassigned
 8 |   return useState(0);
   `----
  help: Variable `someGlobal` is declared outside of the component/hook. Reassigning this value during render is a side effect which can cause unpredictable behavior. If this variable is used in
        rendering, use useState instead. Otherwise, update it in an effect
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/globals

Found 0 warnings and 3 errors.
Finished in Xms on 4 files with 1 rules using X threads.
```

# stderr
```
```
