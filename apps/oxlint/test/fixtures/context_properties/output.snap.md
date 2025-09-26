# Exit code
1

# stdout
```
  x context-plugin(log-context): id: context-plugin/log-context
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x context-plugin(log-context): filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x context-plugin(log-context): physicalFilename: files/1.js
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x context-plugin(log-context): id: context-plugin/log-context
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x context-plugin(log-context): filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x context-plugin(log-context): physicalFilename: files/2.js
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

Found 0 warnings and 6 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
