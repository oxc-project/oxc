# Exit code
1

# stdout
```
  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): false
  | isSpaceBetween(right, left): false
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
   ,-[files/index.js:1:1]
 1 | noSpace=1;
   : ^^^^^^^^^
 2 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(leftExtended, right): false
  | isSpaceBetween(right, leftExtended): false
   ,-[files/index.js:1:1]
 1 | noSpace=1;
   : ^^^^^^^^^
 2 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
   ,-[files/index.js:3:1]
 2 | 
 3 | singleSpaceBefore =2;
   : ^^^^^^^^^^^^^^^^^^^^
 4 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
   ,-[files/index.js:5:1]
 4 | 
 5 | singleSpaceAfter= 3;
   : ^^^^^^^^^^^^^^^^^^^
 6 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
   ,-[files/index.js:7:1]
 6 | 
 7 | multipleSpaces   =   4;
   : ^^^^^^^^^^^^^^^^^^^^^^
 8 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
    ,-[files/index.js:9:1]
  8 |     
  9 | ,-> newlineBefore=
 10 | `-> 5;
 11 |     
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
    ,-[files/index.js:12:1]
 11 |     
 12 | ,-> newlineAfter
 13 | `-> =6;
 14 |     
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(node, binaryLeft): false
  | isSpaceBetween(binaryLeft, node): false
    ,-[files/index.js:15:1]
 14 | 
 15 | nested = 7 + 8;
    : ^^^^^^^^^^^^^^
 16 | 
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
    ,-[files/index.js:15:1]
 14 | 
 15 | nested = 7 + 8;
    : ^^^^^^^^^^^^^^
 16 | 
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(beforeString, afterString): true
  | isSpaceBetween(afterString, beforeString): true
    ,-[files/index.js:18:1]
 17 | // We should return `false` for `isSpaceBetween(beforeString, afterString)`, but we currently return `true`
 18 | beforeString," ",afterString;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    `----

Found 0 warnings and 10 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
