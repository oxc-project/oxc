# Exit code
1

# stdout
```
  x Error running JS plugin.
  | File path: <fixture>/files/1.js
  | Error: Identifier in 1st file: x
  |     at Identifier (<fixture>/plugin.ts:43:32)

  x error-plugin(error): onCodePathStart: Program
   ,-[files/2.js:1:1]
 1 | let y;
   : ^^^^^^^
   `----

  x error-plugin(error): Identifier in 2nd file: y
   ,-[files/2.js:1:5]
 1 | let y;
   :     ^
   `----

Found 0 warnings and 3 errors.
Finished in Xms on 2 files with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
