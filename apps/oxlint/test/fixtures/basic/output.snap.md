# Exit code
1

# stdout
```
  x basic-custom-plugin(no-debugger): Unexpected Debugger Statement
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

Found 1 warning and 1 error.
Finished in Xms on 1 file with 91 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
