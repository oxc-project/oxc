# Exit code
1

# stdout
```
  x js-threads-plugin(no-debugger): Unexpected Debugger Statement
   ,-[files/1.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

  x js-threads-plugin(no-debugger): Unexpected Debugger Statement
   ,-[files/2.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

  x js-threads-plugin(no-debugger): Unexpected Debugger Statement
   ,-[files/3.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
   `----

Found 0 warnings and 3 errors.
Finished in Xms on 3 files with 1 rules using X threads.
```

# stderr
```
Warning: `--js-threads 4` exceeds thread count (2). Using 2 JS threads.
```
