# Exit code
1

# stdout
```
  x test-comments(test-comments): VariableDeclaration(topLevelVariable1):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 1 comment
  |   [0] Line: " Line comment 1" at [29, 46]
  | commentsExistBetween(id, init): false
   ,-[files/comments.js:1:1]
 1 | const topLevelVariable1 = 1;
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | // Line comment 1
   `----

  x test-comments(test-comments): getAllComments: 12 comments
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
  |   [10] Line: " Line comment 5" at [559, 576]
  |   [11] Line: " Line comment 6" at [577, 594]
    ,-[files/comments.js:1:1]
  1 | ,-> const topLevelVariable1 = 1;
  2 | |   // Line comment 1
  3 | |   const topLevelVariable2 = 2; /* Block comment 1 */
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
 23 | |   const topLevelVariable3 = /* Block comment 4 */ 3;
 24 | |   
 25 | |   const topLevelVariable4 = 4;
 26 | |   const topLevelVariable5 = 5;
 27 | |   
 28 | |   // Line comment 5
 29 | |   // Line comment 6
 30 | |   
 31 | |   const topLevelVariable6 = 6;
 32 | `-> const topLevelVariable7 = 7;
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable2):
  | getCommentsBefore: 1 comment
  |   [0] Line: " Line comment 1" at [29, 46]
  | getCommentsInside: 0 comments
  | getCommentsAfter: 2 comments
  |   [0] Block: " Block comment 1 " at [76, 97]
  |   [1] Block: "*\n * JSDoc comment\n " at [99, 123]
  | commentsExistBetween(id, init): false
   ,-[files/comments.js:3:1]
 2 | // Line comment 1
 3 | const topLevelVariable2 = 2; /* Block comment 1 */
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 4 | 
   `----

  x test-comments(test-comments): commentsExistBetween(topLevelVariable2, topLevelFunction): true
   ,-[files/comments.js:3:1]
 2 | // Line comment 1
 3 | const topLevelVariable2 = 2; /* Block comment 1 */
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 4 | 
   `----

  x test-comments(test-comments): commentsExistBetween(topLevelFunction, topLevelVariable2): false
    ,-[files/comments.js:8:8]
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
    ,-[files/comments.js:8:8]
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
  | commentsExistBetween(id, init): false
    ,-[files/comments.js:11:3]
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
    ,-[files/comments.js:15:3]
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
  | commentsExistBetween(id, init): true
    ,-[files/comments.js:23:1]
 22 | /* Block comment 3 */
 23 | const topLevelVariable3 = /* Block comment 4 */ 3;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 24 | 
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable4):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
    ,-[files/comments.js:25:1]
 24 | 
 25 | const topLevelVariable4 = 4;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 26 | const topLevelVariable5 = 5;
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable5):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 2 comments
  |   [0] Line: " Line comment 5" at [559, 576]
  |   [1] Line: " Line comment 6" at [577, 594]
  | commentsExistBetween(id, init): false
    ,-[files/comments.js:26:1]
 25 | const topLevelVariable4 = 4;
 26 | const topLevelVariable5 = 5;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 27 | 
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable6):
  | getCommentsBefore: 2 comments
  |   [0] Line: " Line comment 5" at [559, 576]
  |   [1] Line: " Line comment 6" at [577, 594]
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
    ,-[files/comments.js:31:1]
 30 | 
 31 | const topLevelVariable6 = 6;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 32 | const topLevelVariable7 = 7;
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable7):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
    ,-[files/comments.js:32:1]
 31 | const topLevelVariable6 = 6;
 32 | const topLevelVariable7 = 7;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable1):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
   ,-[files/no_comments.js:1:1]
 1 | const topLevelVariable1 = 1;
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | const topLevelVariable2 = 2;
   `----

  x test-comments(test-comments): getAllComments: 0 comments
    ,-[files/no_comments.js:1:1]
  1 | ,-> const topLevelVariable1 = 1;
  2 | |   const topLevelVariable2 = 2;
  3 | |   
  4 | |   export function topLevelFunction() {
  5 | |     let functionScopedVariable = topLevelVariable;
  6 | |     function nestedFunction() {
  7 | |       return functionScopedVariable;
  8 | |     }
  9 | |     return nestedFunction();
 10 | |   }
 11 | |   
 12 | |   const topLevelVariable3 = 3;
 13 | |   const topLevelVariable4 = 4;
 14 | |   const topLevelVariable5 = 5;
 15 | |   const topLevelVariable6 = 6;
 16 | `-> const topLevelVariable7 = 7;
    `----

  x test-comments(test-comments): commentsExistBetween(topLevelVariable2, topLevelFunction): false
   ,-[files/no_comments.js:2:1]
 1 | const topLevelVariable1 = 1;
 2 | const topLevelVariable2 = 2;
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 3 | 
   `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable2):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
   ,-[files/no_comments.js:2:1]
 1 | const topLevelVariable1 = 1;
 2 | const topLevelVariable2 = 2;
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 3 | 
   `----

  x test-comments(test-comments): commentsExistBetween(topLevelFunction, topLevelVariable2): false
    ,-[files/no_comments.js:4:8]
  3 |     
  4 | ,-> export function topLevelFunction() {
  5 | |     let functionScopedVariable = topLevelVariable;
  6 | |     function nestedFunction() {
  7 | |       return functionScopedVariable;
  8 | |     }
  9 | |     return nestedFunction();
 10 | `-> }
 11 |     
    `----

  x test-comments(test-comments): FunctionDeclaration(topLevelFunction):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
    ,-[files/no_comments.js:4:8]
  3 |     
  4 | ,-> export function topLevelFunction() {
  5 | |     let functionScopedVariable = topLevelVariable;
  6 | |     function nestedFunction() {
  7 | |       return functionScopedVariable;
  8 | |     }
  9 | |     return nestedFunction();
 10 | `-> }
 11 |     
    `----

  x test-comments(test-comments): VariableDeclaration(functionScopedVariable):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
   ,-[files/no_comments.js:5:3]
 4 | export function topLevelFunction() {
 5 |   let functionScopedVariable = topLevelVariable;
   :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 6 |   function nestedFunction() {
   `----

  x test-comments(test-comments): FunctionDeclaration(nestedFunction):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
   ,-[files/no_comments.js:6:3]
 5 |       let functionScopedVariable = topLevelVariable;
 6 | ,->   function nestedFunction() {
 7 | |       return functionScopedVariable;
 8 | `->   }
 9 |       return nestedFunction();
   `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable3):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
    ,-[files/no_comments.js:12:1]
 11 | 
 12 | const topLevelVariable3 = 3;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 13 | const topLevelVariable4 = 4;
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable4):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
    ,-[files/no_comments.js:13:1]
 12 | const topLevelVariable3 = 3;
 13 | const topLevelVariable4 = 4;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 14 | const topLevelVariable5 = 5;
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable5):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
    ,-[files/no_comments.js:14:1]
 13 | const topLevelVariable4 = 4;
 14 | const topLevelVariable5 = 5;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 15 | const topLevelVariable6 = 6;
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable6):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
    ,-[files/no_comments.js:15:1]
 14 | const topLevelVariable5 = 5;
 15 | const topLevelVariable6 = 6;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 16 | const topLevelVariable7 = 7;
    `----

  x test-comments(test-comments): VariableDeclaration(topLevelVariable7):
  | getCommentsBefore: 0 comments
  | getCommentsInside: 0 comments
  | getCommentsAfter: 0 comments
  | commentsExistBetween(id, init): false
    ,-[files/no_comments.js:16:1]
 15 | const topLevelVariable6 = 6;
 16 | const topLevelVariable7 = 7;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    `----

Found 0 warnings and 26 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
