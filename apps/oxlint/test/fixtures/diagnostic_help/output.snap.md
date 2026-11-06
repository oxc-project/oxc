# Exit code
1

# stdout
```
  x help-plugin(no-debugger): Null help.
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

  x help-plugin(no-debugger): Omitted help.
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

  x help-plugin(no-debugger): Undefined help.
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

  x help-plugin(no-debugger): Unexpected debugger statement.
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----
  help: Remove the debugger statement before committing your code.

  x Error running JS plugin.
  | File path: <fixture>/files/invalid.js
  | TypeError: `help` must be a string if provided
  |     at DebuggerStatement (<fixture>/plugin.ts:18:23)

Found 0 warnings and 5 errors.
Finished in Xms on 2 files with 1 rules using X threads.
```

# stderr
```
```
