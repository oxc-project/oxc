# Exit code
1

# stdout
```
  x react(immutability): This value cannot be modified
   ,-[files/index.tsx:6:5]
 3 | export function useLatest<T>(value: T) {
 4 |   const latest = useRef(value);
   :         ^^^|^^
   :            `-- `latest` originates here
 5 |   useEffect(() => {
 6 |     latest.current = value;
   :     ^^^|^^
   :        `-- `latest` cannot be modified
 7 |   });
   `----
  help: Modifying a value returned from a hook is not allowed. Consider moving the modification into the hook where the value is constructed
        
        Hint: If this value is a Ref (value returned by `useRef()`), rename the variable to end in "Ref".
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/immutability

  x react(immutability): This value cannot be modified
    ,-[files/index.tsx:16:9]
 11 | export function Probe() {
 12 |   const clicks = useRef(0);
    :         ^^^|^^
    :            `-- `clicks` originates here
 13 |   return (
 14 |     <button
 15 |       onClick={() => {
 16 |         clicks.current += 1;
    :         ^^^|^^
    :            `-- `clicks` cannot be modified
 17 |       }}
    `----
  help: Modifying a value returned from a hook is not allowed. Consider moving the modification into the hook where the value is constructed
        
        Hint: If this value is a Ref (value returned by `useRef()`), rename the variable to end in "Ref".
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/immutability

  x react(immutability): This value cannot be modified
    ,-[files/index.tsx:42:3]
 41 | export function OtherProperty({ value }: { value: { other: boolean } }) {
 42 |   value.other = true;
    :   ^^|^^
    :     `-- value cannot be modified
 43 |   return <div />;
    `----
  help: Do not mutate component props or hook arguments. If the value should change, update it where it is owned and pass an update callback, or use local state
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/immutability

Found 0 warnings and 3 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
