# Exit code
1

# stdout
```
  x dupe(no-debugger): Unexpected Debugger Statement
   ,-[files/pkg-a/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

  x dupe(no-debugger): Unexpected Debugger Statement
   ,-[files/pkg-b/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 4 files using X threads.
```

# stderr
```
```
