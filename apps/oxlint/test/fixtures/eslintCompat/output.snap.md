# Exit code
1

# stdout
```
  x eslint-compat-plugin(create): create body:
  | this === rule: true
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x eslint-compat-plugin(create-once): before hook:
  | createOnce call count: 1
  | this === rule: true
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x eslint-compat-plugin(create-once): after hook:
  | identNum: 2
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x eslint-compat-plugin(create-once-after-only): after hook:
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x eslint-compat-plugin(create-once-before-false): before hook:
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x eslint-compat-plugin(create-once-before-only): before hook:
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x eslint-compat-plugin(create-once-selector): after hook:
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^
   `----

  x eslint-compat-plugin(create-once-selector): *:exit visit fn:
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^^^^^^^^^^
   `----

  x eslint-compat-plugin(create-once-selector): Program:exit visit fn:
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^^^^^^^^^^
   `----

  x eslint-compat-plugin(create-once-selector): [body]:exit visit fn:
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^^^^^^^^^^
   `----

  x eslint-compat-plugin(create-once-selector): [body][body][body]:exit visit fn:
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:1]
 1 | let a, b;
   : ^^^^^^^^^^
   `----

  x eslint-compat-plugin(create): ident visit fn "a":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x eslint-compat-plugin(create-once): ident visit fn "a":
  | identNum: 1
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x eslint-compat-plugin(create-once-after-only): ident visit fn "a":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x eslint-compat-plugin(create-once-before-only): ident visit fn "a":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x eslint-compat-plugin(create-once-no-hooks): ident visit fn "a":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:5]
 1 | let a, b;
   :     ^
   `----

  x eslint-compat-plugin(create): ident visit fn "b":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x eslint-compat-plugin(create-once): ident visit fn "b":
  | identNum: 2
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x eslint-compat-plugin(create-once-after-only): ident visit fn "b":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x eslint-compat-plugin(create-once-before-only): ident visit fn "b":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x eslint-compat-plugin(create-once-no-hooks): ident visit fn "b":
  | filename: <fixture>/files/1.js
   ,-[files/1.js:1:8]
 1 | let a, b;
   :        ^
   `----

  x eslint-compat-plugin(create): create body:
  | this === rule: true
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x eslint-compat-plugin(create-once): before hook:
  | createOnce call count: 1
  | this === rule: true
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x eslint-compat-plugin(create-once): after hook:
  | identNum: 2
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x eslint-compat-plugin(create-once-after-only): after hook:
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x eslint-compat-plugin(create-once-before-false): before hook:
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x eslint-compat-plugin(create-once-before-false): after hook:
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x eslint-compat-plugin(create-once-before-only): before hook:
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x eslint-compat-plugin(create-once-selector): after hook:
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^
   `----

  x eslint-compat-plugin(create-once-selector): *:exit visit fn:
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^^^^^^^^^^
   `----

  x eslint-compat-plugin(create-once-selector): Program:exit visit fn:
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^^^^^^^^^^
   `----

  x eslint-compat-plugin(create-once-selector): [body]:exit visit fn:
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^^^^^^^^^^
   `----

  x eslint-compat-plugin(create-once-selector): [body][body][body]:exit visit fn:
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:1]
 1 | let c, d;
   : ^^^^^^^^^^
   `----

  x eslint-compat-plugin(create): ident visit fn "c":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x eslint-compat-plugin(create-once): ident visit fn "c":
  | identNum: 1
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x eslint-compat-plugin(create-once-after-only): ident visit fn "c":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x eslint-compat-plugin(create-once-before-false): ident visit fn "c":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x eslint-compat-plugin(create-once-before-only): ident visit fn "c":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x eslint-compat-plugin(create-once-no-hooks): ident visit fn "c":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:5]
 1 | let c, d;
   :     ^
   `----

  x eslint-compat-plugin(create): ident visit fn "d":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x eslint-compat-plugin(create-once): ident visit fn "d":
  | identNum: 2
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x eslint-compat-plugin(create-once-after-only): ident visit fn "d":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x eslint-compat-plugin(create-once-before-false): ident visit fn "d":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x eslint-compat-plugin(create-once-before-only): ident visit fn "d":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

  x eslint-compat-plugin(create-once-no-hooks): ident visit fn "d":
  | filename: <fixture>/files/2.js
   ,-[files/2.js:1:8]
 1 | let c, d;
   :        ^
   `----

Found 0 warnings and 45 errors.
Finished in Xms on 2 files with 8 rules using X threads.
```

# stderr
```
```
