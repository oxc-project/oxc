# Exit code
1

# stdout
```
  x react(todo): [PruneHoistedContexts] Rewrite hoisted function references
   ,-[files/index.jsx:3:12]
 2 |   function foo() {
 3 |     return bar();
   :            ^^^
 4 |   }
   `----
  help: Rewrite the highlighted code using syntax supported by React Compiler
  note: React Compiler skipped optimizing this component or hook. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/unsupported-syntax

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
