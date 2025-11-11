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
   ,-[files/index.js:2:1]
 1 | // prettier-ignore
 2 | noSpace=1;
   : ^^^^^^^^^
 3 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(leftExtended, right): false
  | isSpaceBetween(right, leftExtended): false
   ,-[files/index.js:2:1]
 1 | // prettier-ignore
 2 | noSpace=1;
   : ^^^^^^^^^
 3 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
   ,-[files/index.js:5:1]
 4 | // prettier-ignore
 5 | singleSpaceBefore =2;
   : ^^^^^^^^^^^^^^^^^^^^
 6 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
   ,-[files/index.js:8:1]
 7 | // prettier-ignore
 8 | singleSpaceAfter= 3;
   : ^^^^^^^^^^^^^^^^^^^
 9 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
    ,-[files/index.js:11:1]
 10 | // prettier-ignore
 11 | multipleSpaces   =   4;
    : ^^^^^^^^^^^^^^^^^^^^^^
 12 | 
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
    ,-[files/index.js:14:1]
 13 |     // prettier-ignore
 14 | ,-> newlineBefore=
 15 | `-> 5;
 16 |     
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
    ,-[files/index.js:18:1]
 17 |     // prettier-ignore
 18 | ,-> newlineAfter
 19 | `-> =6;
 20 |     
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(node, binaryLeft): false
  | isSpaceBetween(binaryLeft, node): false
    ,-[files/index.js:22:1]
 21 | // prettier-ignore
 22 | nested = 7 + 8;
    : ^^^^^^^^^^^^^^
 23 | 
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetween(node, right): false
    ,-[files/index.js:22:1]
 21 | // prettier-ignore
 22 | nested = 7 + 8;
    : ^^^^^^^^^^^^^^
 23 | 
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(beforeString, afterString): true
  | isSpaceBetween(afterString, beforeString): true
    ,-[files/index.js:26:1]
 25 | // prettier-ignore
 26 | beforeString," ",afterString;
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
