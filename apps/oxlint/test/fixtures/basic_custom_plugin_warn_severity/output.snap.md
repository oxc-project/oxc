# Exit code
0

# stdout
```
  ! basic-custom-plugin(no-debugger): Unexpected Debugger Statement
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

  ! ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-debugger.html\eslint(no-debugger)]8;;\: `debugger` statement is not allowed
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----
  help: Remove the debugger statement

Found 2 warnings and 0 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
