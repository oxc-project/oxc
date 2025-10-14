# Exit code
1

# stdout
```
  x test-getAllComments(test-getAllComments): getAllComments() returned 6 comments:
  |   [0] Line: " Line comment 1" at [0, 17]
  |   [1] Block: " Block comment 1 " at [31, 52]
  |   [2] Block: "*\n * JSDoc comment\n " at [54, 78]
  |   [3] Line: " Line comment 2" at [105, 122]
  |   [4] Line: " Line comment 3" at [135, 152]
  |   [5] Block: " Block comment 2 " at [156, 177]
    ,-[files/test.js:2:1]
  1 |     // Line comment 1
  2 | ,-> const x = 1; /* Block comment 1 */
  3 | |   
  4 | |   /**
  5 | |    * JSDoc comment
  6 | |    */
  7 | |   export function foo() {
  8 | |     // Line comment 2
  9 | |     return x; // Line comment 3
 10 | |   }
 11 | |   
 12 | `-> /* Block comment 2 */
    `----

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
