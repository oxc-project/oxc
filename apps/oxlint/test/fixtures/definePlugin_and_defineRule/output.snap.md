# Exit code
1

# stdout
```
  x define-plugin-and-rule-plugin(create): create body:
  | this === rule: true
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once): before hook:
  | createOnce call count: 1
  | this === rule: true
  | filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once): after hook:
  | identNum: 2
  | filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once-after-only): after hook:
  | filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-false): before hook:
  | filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-only): before hook:
  | filename: files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x define-plugin-and-rule-plugin(create): ident visit fn "a":
  | filename: files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create-once): ident visit fn "a":
  | identNum: 1
  | filename: files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create-once-after-only): ident visit fn "a":
  | filename: files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-only): ident visit fn "a":
  | filename: files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create-once-no-hooks): ident visit fn "a":
  | filename: files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create): ident visit fn "b":
  | filename: files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create-once): ident visit fn "b":
  | identNum: 2
  | filename: files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create-once-after-only): ident visit fn "b":
  | filename: files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-only): ident visit fn "b":
  | filename: files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create-once-no-hooks): ident visit fn "b":
  | filename: files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create): create body:
  | this === rule: true
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once): before hook:
  | createOnce call count: 1
  | this === rule: true
  | filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once): after hook:
  | identNum: 2
  | filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once-after-only): after hook:
  | filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-false): before hook:
  | filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-false): after hook:
  | filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-only): before hook:
  | filename: files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x define-plugin-and-rule-plugin(create): ident visit fn "c":
  | filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create-once): ident visit fn "c":
  | identNum: 1
  | filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create-once-after-only): ident visit fn "c":
  | filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-false): ident visit fn "c":
  | filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-only): ident visit fn "c":
  | filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create-once-no-hooks): ident visit fn "c":
  | filename: files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x define-plugin-and-rule-plugin(create): ident visit fn "d":
  | filename: files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create-once): ident visit fn "d":
  | identNum: 2
  | filename: files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create-once-after-only): ident visit fn "d":
  | filename: files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-false): ident visit fn "d":
  | filename: files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create-once-before-only): ident visit fn "d":
  | filename: files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x define-plugin-and-rule-plugin(create-once-no-hooks): ident visit fn "d":
  | filename: files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

Found 0 warnings and 35 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
