# Exit code
1

# stdout
```
  x react(set-state-in-render): Cannot call setState during render
   ,-[files/enum.tsx:8:3]
 7 |   }
 8 |   setState(1);
   :   ^^^^|^^^
   :       `-- Found setState() in render
 9 |   return (
   `----
  help: Calling setState during render may trigger an infinite loop.
        * To reset state when other state/props change, store the previous value in state and update conditionally.
        * To derive data from other state/props, compute the derived data during render without using state
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/set-state-in-render

  x react(set-state-in-render): Cannot call setState during render
   ,-[files/jsx-default.jsx:5:3]
 4 |   const [state, setState] = useState(0);
 5 |   setState(1);
   :   ^^^^|^^^
   :       `-- Found setState() in render
 6 |   return (
   `----
  help: Calling setState during render may trigger an infinite loop.
        * To reset state when other state/props change, store the previous value in state and update conditionally.
        * To derive data from other state/props, compute the derived data during render without using state
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/set-state-in-render

Found 0 warnings and 2 errors.
Finished in Xms on 2 files with 1 rules using X threads.
```

# stderr
```
```
