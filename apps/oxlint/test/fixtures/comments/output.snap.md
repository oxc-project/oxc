# Exit code
1

# stdout
```
  x test-comments(test-comments): VariableDeclaration(topLevelVariable1):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 1 comment
  |   [0] Line: " Line comment 1" at [29, 46]
   ,-[files/test.js:1:1]
 1 | const topLevelVariable1 = 1;
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | // Line comment 1
   `----

  x test-comments(test-comments): getAllComments: 10 comments
  |   [0] Line: " Line comment 1" at [29, 46]
  |   [1] Block: " Block comment 1 " at [76, 97]
  |   [2] Block: "*\n * JSDoc comment\n " at [99, 123]
  |   [3] Line: " Line comment 2" at [163, 180]
  |   [4] Block: " Block comment 2 " at [183, 204]
  |   [5] Block: "*\n   * JSDoc comment 2\n   " at [256, 286]
  |   [6] Line: " Line comment 3" at [321, 338]
  |   [7] Line: " Line comment 4" at [405, 422]
  |   [8] Block: " Block comment 3 " at [426, 447]
  |   [9] Block: " Block comment 4 " at [474, 495]
    ,-[files/test.js:1:1]
  1 | ,-> const topLevelVariable1 = 1;
  2 | |   // Line comment 1
  3 | |   const topLevelVariable2 = 1; /* Block comment 1 */
  4 | |   
  5 | |   /**
  6 | |    * JSDoc comment
  7 | |    */
  8 | |   export function topLevelFunction() {
  9 | |     // Line comment 2
 10 | |     /* Block comment 2 */
 11 | |     let functionScopedVariable = topLevelVariable;
 12 | |     /**
 13 | |      * JSDoc comment 2
 14 | |      */
 15 | |     function nestedFunction() {
 16 | |       // Line comment 3
 17 | |       return functionScopedVariable;
 18 | |     }
 19 | |     return nestedFunction(); // Line comment 4
 20 | |   }
 21 | |   
 22 | |   /* Block comment 3 */
 23 | `-> const topLevelVariable3 = /* Block comment 4 */ 2;
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable2):
  | getCommentsBefore: 1 comment
  |   [0] Line: " Line comment 1" at [29, 46]
  | getCommentsInside: 0 comments
  | getCommentsAfter: 2 comments
  |   [0] Block: " Block comment 1 " at [76, 97]
  |   [1] Block: "*\n * JSDoc comment\n " at [99, 123]
   ,-[files/test.js:3:1]
 2 | // Line comment 1
 3 | const topLevelVariable2 = 1; /* Block comment 1 */
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 4 | 
   `----

  x test-comments(test-comments): commentsExistBetween(topLevelVariable, topLevelFunction): true
   ,-[files/test.js:3:1]
 2 | // Line comment 1
 3 | const topLevelVariable2 = 1; /* Block comment 1 */
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 4 | 
   `----

  x test-comments(test-comments): FunctionDeclaration(topLevelFunction):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 5 comments
  |   [0] Line: " Line comment 2" at [163, 180]
  |   [1] Block: " Block comment 2 " at [183, 204]
  |   [2] Block: "*\n   * JSDoc comment 2\n   " at [256, 286]
  |   [3] Line: " Line comment 3" at [321, 338]
  |   [4] Line: " Line comment 4" at [405, 422]
  | getCommentsAfter: 1 comment
  |   [0] Block: " Block comment 3 " at [426, 447]
    ,-[files/test.js:8:8]
  7 |      */
  8 | ,-> export function topLevelFunction() {
  9 | |     // Line comment 2
 10 | |     /* Block comment 2 */
 11 | |     let functionScopedVariable = topLevelVariable;
 12 | |     /**
 13 | |      * JSDoc comment 2
 14 | |      */
 15 | |     function nestedFunction() {
 16 | |       // Line comment 3
 17 | |       return functionScopedVariable;
 18 | |     }
 19 | |     return nestedFunction(); // Line comment 4
 20 | `-> }
 21 |     
    `----

  x test-comments(test-comments): VariableDeclaration(functionScopedVariable):
  | getCommentsBefore: 2 comments
  |   [0] Line: " Line comment 2" at [163, 180]
  |   [1] Block: " Block comment 2 " at [183, 204]
  | getCommentsInside: 0 comments
  | getCommentsAfter: 1 comment
  |   [0] Block: "*\n   * JSDoc comment 2\n   " at [256, 286]
    ,-[files/test.js:11:3]
 10 |   /* Block comment 2 */
 11 |   let functionScopedVariable = topLevelVariable;
    :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 12 |   /**
    `----

  x test-comments(test-comments): FunctionDeclaration(nestedFunction):
  | getCommentsBefore: 1 comment
  |   [0] Block: "*\n   * JSDoc comment 2\n   " at [256, 286]
  | getCommentsInside: 1 comment
  |   [0] Line: " Line comment 3" at [321, 338]
  | getCommentsAfter: 0 comments
    ,-[files/test.js:15:3]
 14 |        */
 15 | ,->   function nestedFunction() {
 16 | |       // Line comment 3
 17 | |       return functionScopedVariable;
 18 | `->   }
 19 |       return nestedFunction(); // Line comment 4
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable3):
  | getCommentsBefore: 1 comment
  |   [0] Block: " Block comment 3 " at [426, 447]
  | getCommentsInside: 1 comment
  |   [0] Block: " Block comment 4 " at [474, 495]
  | getCommentsAfter: 0 comments
    ,-[files/test.js:23:1]
 22 | /* Block comment 3 */
 23 | const topLevelVariable3 = /* Block comment 4 */ 2;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    `----

Found 0 warnings and 8 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
