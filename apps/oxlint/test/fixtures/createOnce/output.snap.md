# Exit code
1

# stdout
```
  x create-once-plugin(after-only): after hook: filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(after-only): after hook: id: create-once-plugin/after-only
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: call count: 1
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: filename: Cannot access `context.filename` in `createOnce`
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): before hook: filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): after hook: filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: id: Cannot access `context.id` in `createOnce`
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): before hook: id: create-once-plugin/always-run
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): after hook: id: create-once-plugin/always-run
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: options: Cannot access `context.options` in `createOnce`
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: physicalFilename: Cannot access `context.physicalFilename` in `createOnce`
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: report: Cannot report errors in `createOnce`
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: sourceCode: Cannot access `context.sourceCode` in `createOnce`
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: this === rule: true
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(before-only): before hook: filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(before-only): before hook: id: create-once-plugin/before-only
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(skip-run): before hook: filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(skip-run): before hook: id: create-once-plugin/skip-run
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x create-once-plugin(after-only): ident visit fn "x": filename: files/1.js
   ,-[files/1.js:1:5]
 1 | let x;
   :     ^
   `----

  x create-once-plugin(always-run): ident visit fn "x": filename: files/1.js
   ,-[files/1.js:1:5]
 1 | let x;
   :     ^
   `----

  x create-once-plugin(before-only): ident visit fn "x": filename: files/1.js
   ,-[files/1.js:1:5]
 1 | let x;
   :     ^
   `----

  x create-once-plugin(no-hooks): ident visit fn "x": filename: files/1.js
   ,-[files/1.js:1:5]
 1 | let x;
   :     ^
   `----

  x create-once-plugin(after-only): after hook: filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(after-only): after hook: id: create-once-plugin/after-only
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: call count: 1
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: filename: Cannot access `context.filename` in `createOnce`
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): before hook: filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): after hook: filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: id: Cannot access `context.id` in `createOnce`
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): before hook: id: create-once-plugin/always-run
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): after hook: id: create-once-plugin/always-run
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: options: Cannot access `context.options` in `createOnce`
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: physicalFilename: Cannot access `context.physicalFilename` in `createOnce`
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: report: Cannot report errors in `createOnce`
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: sourceCode: Cannot access `context.sourceCode` in `createOnce`
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(always-run): createOnce: this === rule: true
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(before-only): before hook: filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(before-only): before hook: id: create-once-plugin/before-only
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(skip-run): before hook: filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(skip-run): before hook: id: create-once-plugin/skip-run
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x create-once-plugin(after-only): ident visit fn "y": filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let y;
   :     ^
   `----

  x create-once-plugin(always-run): ident visit fn "y": filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let y;
   :     ^
   `----

  x create-once-plugin(before-only): ident visit fn "y": filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let y;
   :     ^
   `----

  x create-once-plugin(no-hooks): ident visit fn "y": filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let y;
   :     ^
   `----

Found 0 warnings and 44 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
