# Exit code
1

# stdout
```
  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): false
  | isSpaceBetweenTokens(left, right): false
  | isSpaceBetween(right, left): false
  | isSpaceBetweenTokens(right, left): false
  | isSpaceBetween(left, node): false
  | isSpaceBetweenTokens(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetweenTokens(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetweenTokens(right, node): false
  | isSpaceBetween(node, right): false
  | isSpaceBetweenTokens(node, right): false
   ,-[files/index.js:2:1]
 1 | // prettier-ignore
 2 | noSpace=1;
   : ^^^^^^^^^
 3 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(leftExtended, right): false
  | isSpaceBetweenTokens(leftExtended, right): false
  | isSpaceBetween(right, leftExtended): false
  | isSpaceBetweenTokens(right, leftExtended): false
   ,-[files/index.js:2:1]
 1 | // prettier-ignore
 2 | noSpace=1;
   : ^^^^^^^^^
 3 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetweenTokens(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetweenTokens(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetweenTokens(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetweenTokens(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetweenTokens(right, node): false
  | isSpaceBetween(node, right): false
  | isSpaceBetweenTokens(node, right): false
   ,-[files/index.js:5:1]
 4 | // prettier-ignore
 5 | singleSpaceBefore =2;
   : ^^^^^^^^^^^^^^^^^^^^
 6 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetweenTokens(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetweenTokens(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetweenTokens(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetweenTokens(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetweenTokens(right, node): false
  | isSpaceBetween(node, right): false
  | isSpaceBetweenTokens(node, right): false
   ,-[files/index.js:8:1]
 7 | // prettier-ignore
 8 | singleSpaceAfter= 3;
   : ^^^^^^^^^^^^^^^^^^^
 9 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetweenTokens(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetweenTokens(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetweenTokens(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetweenTokens(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetweenTokens(right, node): false
  | isSpaceBetween(node, right): false
  | isSpaceBetweenTokens(node, right): false
    ,-[files/index.js:11:1]
 10 | // prettier-ignore
 11 | multipleSpaces   =   4;
    : ^^^^^^^^^^^^^^^^^^^^^^
 12 | 
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetweenTokens(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetweenTokens(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetweenTokens(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetweenTokens(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetweenTokens(right, node): false
  | isSpaceBetween(node, right): false
  | isSpaceBetweenTokens(node, right): false
    ,-[files/index.js:14:1]
 13 |     // prettier-ignore
 14 | ,-> newlineBefore=
 15 | `-> 5;
 16 |     
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetweenTokens(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetweenTokens(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetweenTokens(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetweenTokens(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetweenTokens(right, node): false
  | isSpaceBetween(node, right): false
  | isSpaceBetweenTokens(node, right): false
    ,-[files/index.js:18:1]
 17 |     // prettier-ignore
 18 | ,-> newlineAfter
 19 | `-> =6;
 20 |     
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(node, binaryLeft): false
  | isSpaceBetweenTokens(node, binaryLeft): false
  | isSpaceBetween(binaryLeft, node): false
  | isSpaceBetweenTokens(binaryLeft, node): false
    ,-[files/index.js:22:1]
 21 | // prettier-ignore
 22 | nested = 7 + 8;
    : ^^^^^^^^^^^^^^
 23 | 
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(left, right): true
  | isSpaceBetweenTokens(left, right): true
  | isSpaceBetween(right, left): true
  | isSpaceBetweenTokens(right, left): true
  | isSpaceBetween(left, node): false
  | isSpaceBetweenTokens(left, node): false
  | isSpaceBetween(node, left): false
  | isSpaceBetweenTokens(node, left): false
  | isSpaceBetween(right, node): false
  | isSpaceBetweenTokens(right, node): false
  | isSpaceBetween(node, right): false
  | isSpaceBetweenTokens(node, right): false
    ,-[files/index.js:22:1]
 21 | // prettier-ignore
 22 | nested = 7 + 8;
    : ^^^^^^^^^^^^^^
 23 | 
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(beforeString, afterString): false
  | isSpaceBetweenTokens(beforeString, afterString): false
  | isSpaceBetween(afterString, beforeString): false
  | isSpaceBetweenTokens(afterString, beforeString): false
    ,-[files/index.js:25:1]
 24 | // prettier-ignore
 25 | beforeString," ",afterString;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    `----

  x test-plugin(is-space-between):
  | isSpaceBetween(openingElement, closingElement): false
  | isSpaceBetweenTokens(openingElement, closingElement): false
  | isSpaceBetween(closingElement, openingElement): false
  | isSpaceBetweenTokens(closingElement, openingElement): false
   ,-[files/index.jsx:1:1]
 1 | <Foo>aaa</Foo>;
   : ^^^^^^^^^^^^^^
 2 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(openingElement, closingElement): false
  | isSpaceBetweenTokens(openingElement, closingElement): true
  | isSpaceBetween(closingElement, openingElement): false
  | isSpaceBetweenTokens(closingElement, openingElement): true
   ,-[files/index.jsx:3:1]
 2 | 
 3 | <Bar>b c</Bar>;
   : ^^^^^^^^^^^^^^
 4 | 
   `----

  x test-plugin(is-space-between):
  | isSpaceBetween(openingElement, closingElement): false
  | isSpaceBetweenTokens(openingElement, closingElement): true
  | isSpaceBetween(closingElement, openingElement): false
  | isSpaceBetweenTokens(closingElement, openingElement): true
   ,-[files/index.jsx:6:1]
 5 |     // prettier-ignore
 6 | ,-> <Qux>d
 7 | `-> e</Qux>;
   `----

Found 0 warnings and 13 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
