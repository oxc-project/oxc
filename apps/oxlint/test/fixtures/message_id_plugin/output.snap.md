# Exit code
1

# stdout
```
  x message-id-plugin(no-var): Unexpected var, use let or const instead.
   ,-[files/index.js:1:1]
 1 | var reportUsingNode = 1;
   : ^^^^^^^^^^^^^^^^^^^^^^^^
 2 | var reportUsingRange = 1;
   `----

  x message-id-plugin(no-var): Unexpected var, use let or const instead.
   ,-[files/index.js:2:1]
 1 | var reportUsingNode = 1;
 2 | var reportUsingRange = 1;
   : ^^^^^^^^^^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
