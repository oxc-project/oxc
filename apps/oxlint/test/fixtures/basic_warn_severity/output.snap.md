# Exit code
0

# stdout
```
  ! basic-custom-plugin(no-debugger): Unexpected Debugger Statement
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

  ! eslint(no-debugger): `debugger` statement is not allowed
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----
  help: Remove the debugger statement

Found 2 warnings and 0 errors.
Finished in Xms on 1 file with 2 rules using X threads.
```

# stderr
```
```
