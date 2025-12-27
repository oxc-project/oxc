# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): getAllComments: 0 comments[0m
    ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:1:1]
 [2m 1[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m const topLevelVariable1 = 1;
 [2m 2[0m │ [38;2;246;87;248m│[0m   const topLevelVariable2 = 2;
 [2m 3[0m │ [38;2;246;87;248m│[0m   
 [2m 4[0m │ [38;2;246;87;248m│[0m   export function topLevelFunction() {
 [2m 5[0m │ [38;2;246;87;248m│[0m     let functionScopedVariable = topLevelVariable;
 [2m 6[0m │ [38;2;246;87;248m│[0m     function nestedFunction() {
 [2m 7[0m │ [38;2;246;87;248m│[0m       return functionScopedVariable;
 [2m 8[0m │ [38;2;246;87;248m│[0m     }
 [2m 9[0m │ [38;2;246;87;248m│[0m     return nestedFunction();
 [2m10[0m │ [38;2;246;87;248m│[0m   }
 [2m11[0m │ [38;2;246;87;248m│[0m   
 [2m12[0m │ [38;2;246;87;248m│[0m   const topLevelVariable3 = 3;
 [2m13[0m │ [38;2;246;87;248m│[0m   const topLevelVariable4 = 4;
 [2m14[0m │ [38;2;246;87;248m│[0m   const topLevelVariable5 = 5;
 [2m15[0m │ [38;2;246;87;248m│[0m   const topLevelVariable6 = 6;
 [2m16[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m const topLevelVariable7 = 7;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): commentsExistBetween(topLevelVariable2, topLevelFunction): false[0m
   ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:2:1]
 [2m1[0m │ const topLevelVariable1 = 1;
 [2m2[0m │ const topLevelVariable2 = 2;
   · [38;2;246;87;248m────────────────────────────[0m
 [2m3[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): commentsExistBetween(topLevelFunction, topLevelVariable2): false[0m
    ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:4:8]
 [2m 3[0m │     
 [2m 4[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m export function topLevelFunction() {
 [2m 5[0m │ [38;2;246;87;248m│[0m     let functionScopedVariable = topLevelVariable;
 [2m 6[0m │ [38;2;246;87;248m│[0m     function nestedFunction() {
 [2m 7[0m │ [38;2;246;87;248m│[0m       return functionScopedVariable;
 [2m 8[0m │ [38;2;246;87;248m│[0m     }
 [2m 9[0m │ [38;2;246;87;248m│[0m     return nestedFunction();
 [2m10[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m11[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable1):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
   ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:1:1]
 [2m1[0m │ const topLevelVariable1 = 1;
   · [38;2;246;87;248m────────────────────────────[0m
 [2m2[0m │ const topLevelVariable2 = 2;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable2):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
   ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:2:1]
 [2m1[0m │ const topLevelVariable1 = 1;
 [2m2[0m │ const topLevelVariable2 = 2;
   · [38;2;246;87;248m────────────────────────────[0m
 [2m3[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): FunctionDeclaration(topLevelFunction):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments[0m
    ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:4:8]
 [2m 3[0m │     
 [2m 4[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m export function topLevelFunction() {
 [2m 5[0m │ [38;2;246;87;248m│[0m     let functionScopedVariable = topLevelVariable;
 [2m 6[0m │ [38;2;246;87;248m│[0m     function nestedFunction() {
 [2m 7[0m │ [38;2;246;87;248m│[0m       return functionScopedVariable;
 [2m 8[0m │ [38;2;246;87;248m│[0m     }
 [2m 9[0m │ [38;2;246;87;248m│[0m     return nestedFunction();
 [2m10[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m11[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(functionScopedVariable):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
   ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:5:3]
 [2m4[0m │ export function topLevelFunction() {
 [2m5[0m │   let functionScopedVariable = topLevelVariable;
   · [38;2;246;87;248m  ──────────────────────────────────────────────[0m
 [2m6[0m │   function nestedFunction() {
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): FunctionDeclaration(nestedFunction):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments[0m
   ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:6:3]
 [2m5[0m │       let functionScopedVariable = topLevelVariable;
 [2m6[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m   function nestedFunction() {
 [2m7[0m │ [38;2;246;87;248m│[0m       return functionScopedVariable;
 [2m8[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m   }
 [2m9[0m │       return nestedFunction();
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable3):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:12:1]
 [2m11[0m │ 
 [2m12[0m │ const topLevelVariable3 = 3;
    · [38;2;246;87;248m────────────────────────────[0m
 [2m13[0m │ const topLevelVariable4 = 4;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable4):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:13:1]
 [2m12[0m │ const topLevelVariable3 = 3;
 [2m13[0m │ const topLevelVariable4 = 4;
    · [38;2;246;87;248m────────────────────────────[0m
 [2m14[0m │ const topLevelVariable5 = 5;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable5):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:14:1]
 [2m13[0m │ const topLevelVariable4 = 4;
 [2m14[0m │ const topLevelVariable5 = 5;
    · [38;2;246;87;248m────────────────────────────[0m
 [2m15[0m │ const topLevelVariable6 = 6;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable6):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:15:1]
 [2m14[0m │ const topLevelVariable5 = 5;
 [2m15[0m │ const topLevelVariable6 = 6;
    · [38;2;246;87;248m────────────────────────────[0m
 [2m16[0m │ const topLevelVariable7 = 7;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable7):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/no_comments.js[0m:16:1]
 [2m15[0m │ const topLevelVariable6 = 6;
 [2m16[0m │ const topLevelVariable7 = 7;
    · [38;2;246;87;248m────────────────────────────[0m
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): getAllComments: 12 comments
  [38;2;225;80;80;1m│[0m   [0] Line: " Line comment 1" at [29, 46]
  [38;2;225;80;80;1m│[0m   [1] Block: " Block comment 1 " at [76, 97]
  [38;2;225;80;80;1m│[0m   [2] Block: "*\n * JSDoc comment\n " at [99, 123]
  [38;2;225;80;80;1m│[0m   [3] Line: " Line comment 2" at [163, 180]
  [38;2;225;80;80;1m│[0m   [4] Block: " Block comment 2 " at [183, 204]
  [38;2;225;80;80;1m│[0m   [5] Block: "*\n   * JSDoc comment 2\n   " at [256, 286]
  [38;2;225;80;80;1m│[0m   [6] Line: " Line comment 3" at [321, 338]
  [38;2;225;80;80;1m│[0m   [7] Line: " Line comment 4" at [405, 422]
  [38;2;225;80;80;1m│[0m   [8] Block: " Block comment 3 " at [426, 447]
  [38;2;225;80;80;1m│[0m   [9] Block: " Block comment 4 " at [474, 495]
  [38;2;225;80;80;1m│[0m   [10] Line: " Line comment 5" at [559, 576]
  [38;2;225;80;80;1m│[0m   [11] Line: " Line comment 6" at [577, 594][0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:1:1]
 [2m 1[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m const topLevelVariable1 = 1;
 [2m 2[0m │ [38;2;246;87;248m│[0m   // Line comment 1
 [2m 3[0m │ [38;2;246;87;248m│[0m   const topLevelVariable2 = 2; /* Block comment 1 */
 [2m 4[0m │ [38;2;246;87;248m│[0m   
 [2m 5[0m │ [38;2;246;87;248m│[0m   /**
 [2m 6[0m │ [38;2;246;87;248m│[0m    * JSDoc comment
 [2m 7[0m │ [38;2;246;87;248m│[0m    */
 [2m 8[0m │ [38;2;246;87;248m│[0m   export function topLevelFunction() {
 [2m 9[0m │ [38;2;246;87;248m│[0m     // Line comment 2
 [2m10[0m │ [38;2;246;87;248m│[0m     /* Block comment 2 */
 [2m11[0m │ [38;2;246;87;248m│[0m     let functionScopedVariable = topLevelVariable;
 [2m12[0m │ [38;2;246;87;248m│[0m     /**
 [2m13[0m │ [38;2;246;87;248m│[0m      * JSDoc comment 2
 [2m14[0m │ [38;2;246;87;248m│[0m      */
 [2m15[0m │ [38;2;246;87;248m│[0m     function nestedFunction() {
 [2m16[0m │ [38;2;246;87;248m│[0m       // Line comment 3
 [2m17[0m │ [38;2;246;87;248m│[0m       return functionScopedVariable;
 [2m18[0m │ [38;2;246;87;248m│[0m     }
 [2m19[0m │ [38;2;246;87;248m│[0m     return nestedFunction(); // Line comment 4
 [2m20[0m │ [38;2;246;87;248m│[0m   }
 [2m21[0m │ [38;2;246;87;248m│[0m   
 [2m22[0m │ [38;2;246;87;248m│[0m   /* Block comment 3 */
 [2m23[0m │ [38;2;246;87;248m│[0m   const topLevelVariable3 = /* Block comment 4 */ 3;
 [2m24[0m │ [38;2;246;87;248m│[0m   
 [2m25[0m │ [38;2;246;87;248m│[0m   const topLevelVariable4 = 4;
 [2m26[0m │ [38;2;246;87;248m│[0m   const topLevelVariable5 = 5;
 [2m27[0m │ [38;2;246;87;248m│[0m   
 [2m28[0m │ [38;2;246;87;248m│[0m   // Line comment 5
 [2m29[0m │ [38;2;246;87;248m│[0m   // Line comment 6
 [2m30[0m │ [38;2;246;87;248m│[0m   
 [2m31[0m │ [38;2;246;87;248m│[0m   const topLevelVariable6 = 6;
 [2m32[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m const topLevelVariable7 = 7;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): commentsExistBetween(topLevelVariable2, topLevelFunction): true[0m
   ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:3:1]
 [2m2[0m │ // Line comment 1
 [2m3[0m │ const topLevelVariable2 = 2; /* Block comment 1 */
   · [38;2;246;87;248m────────────────────────────[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): commentsExistBetween(topLevelFunction, topLevelVariable2): false[0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:8:8]
 [2m 7[0m │      */
 [2m 8[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m export function topLevelFunction() {
 [2m 9[0m │ [38;2;246;87;248m│[0m     // Line comment 2
 [2m10[0m │ [38;2;246;87;248m│[0m     /* Block comment 2 */
 [2m11[0m │ [38;2;246;87;248m│[0m     let functionScopedVariable = topLevelVariable;
 [2m12[0m │ [38;2;246;87;248m│[0m     /**
 [2m13[0m │ [38;2;246;87;248m│[0m      * JSDoc comment 2
 [2m14[0m │ [38;2;246;87;248m│[0m      */
 [2m15[0m │ [38;2;246;87;248m│[0m     function nestedFunction() {
 [2m16[0m │ [38;2;246;87;248m│[0m       // Line comment 3
 [2m17[0m │ [38;2;246;87;248m│[0m       return functionScopedVariable;
 [2m18[0m │ [38;2;246;87;248m│[0m     }
 [2m19[0m │ [38;2;246;87;248m│[0m     return nestedFunction(); // Line comment 4
 [2m20[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m21[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable1):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 1 comment
  [38;2;225;80;80;1m│[0m   [0] Line: " Line comment 1" at [29, 46]
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
   ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:1:1]
 [2m1[0m │ const topLevelVariable1 = 1;
   · [38;2;246;87;248m────────────────────────────[0m
 [2m2[0m │ // Line comment 1
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable2):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 1 comment
  [38;2;225;80;80;1m│[0m   [0] Line: " Line comment 1" at [29, 46]
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 2 comments
  [38;2;225;80;80;1m│[0m   [0] Block: " Block comment 1 " at [76, 97]
  [38;2;225;80;80;1m│[0m   [1] Block: "*\n * JSDoc comment\n " at [99, 123]
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
   ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:3:1]
 [2m2[0m │ // Line comment 1
 [2m3[0m │ const topLevelVariable2 = 2; /* Block comment 1 */
   · [38;2;246;87;248m────────────────────────────[0m
 [2m4[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): FunctionDeclaration(topLevelFunction):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 5 comments
  [38;2;225;80;80;1m│[0m   [0] Line: " Line comment 2" at [163, 180]
  [38;2;225;80;80;1m│[0m   [1] Block: " Block comment 2 " at [183, 204]
  [38;2;225;80;80;1m│[0m   [2] Block: "*\n   * JSDoc comment 2\n   " at [256, 286]
  [38;2;225;80;80;1m│[0m   [3] Line: " Line comment 3" at [321, 338]
  [38;2;225;80;80;1m│[0m   [4] Line: " Line comment 4" at [405, 422]
  [38;2;225;80;80;1m│[0m getCommentsAfter: 1 comment
  [38;2;225;80;80;1m│[0m   [0] Block: " Block comment 3 " at [426, 447][0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:8:8]
 [2m 7[0m │      */
 [2m 8[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m export function topLevelFunction() {
 [2m 9[0m │ [38;2;246;87;248m│[0m     // Line comment 2
 [2m10[0m │ [38;2;246;87;248m│[0m     /* Block comment 2 */
 [2m11[0m │ [38;2;246;87;248m│[0m     let functionScopedVariable = topLevelVariable;
 [2m12[0m │ [38;2;246;87;248m│[0m     /**
 [2m13[0m │ [38;2;246;87;248m│[0m      * JSDoc comment 2
 [2m14[0m │ [38;2;246;87;248m│[0m      */
 [2m15[0m │ [38;2;246;87;248m│[0m     function nestedFunction() {
 [2m16[0m │ [38;2;246;87;248m│[0m       // Line comment 3
 [2m17[0m │ [38;2;246;87;248m│[0m       return functionScopedVariable;
 [2m18[0m │ [38;2;246;87;248m│[0m     }
 [2m19[0m │ [38;2;246;87;248m│[0m     return nestedFunction(); // Line comment 4
 [2m20[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
 [2m21[0m │     
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(functionScopedVariable):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 2 comments
  [38;2;225;80;80;1m│[0m   [0] Line: " Line comment 2" at [163, 180]
  [38;2;225;80;80;1m│[0m   [1] Block: " Block comment 2 " at [183, 204]
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 1 comment
  [38;2;225;80;80;1m│[0m   [0] Block: "*\n   * JSDoc comment 2\n   " at [256, 286]
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:11:3]
 [2m10[0m │   /* Block comment 2 */
 [2m11[0m │   let functionScopedVariable = topLevelVariable;
    · [38;2;246;87;248m  ──────────────────────────────────────────────[0m
 [2m12[0m │   /**
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): FunctionDeclaration(nestedFunction):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 1 comment
  [38;2;225;80;80;1m│[0m   [0] Block: "*\n   * JSDoc comment 2\n   " at [256, 286]
  [38;2;225;80;80;1m│[0m getCommentsInside: 1 comment
  [38;2;225;80;80;1m│[0m   [0] Line: " Line comment 3" at [321, 338]
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments[0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:15:3]
 [2m14[0m │        */
 [2m15[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m   function nestedFunction() {
 [2m16[0m │ [38;2;246;87;248m│[0m       // Line comment 3
 [2m17[0m │ [38;2;246;87;248m│[0m       return functionScopedVariable;
 [2m18[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m   }
 [2m19[0m │       return nestedFunction(); // Line comment 4
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable3):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 1 comment
  [38;2;225;80;80;1m│[0m   [0] Block: " Block comment 3 " at [426, 447]
  [38;2;225;80;80;1m│[0m getCommentsInside: 1 comment
  [38;2;225;80;80;1m│[0m   [0] Block: " Block comment 4 " at [474, 495]
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): true[0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:23:1]
 [2m22[0m │ /* Block comment 3 */
 [2m23[0m │ const topLevelVariable3 = /* Block comment 4 */ 3;
    · [38;2;246;87;248m──────────────────────────────────────────────────[0m
 [2m24[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable4):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:25:1]
 [2m24[0m │ 
 [2m25[0m │ const topLevelVariable4 = 4;
    · [38;2;246;87;248m────────────────────────────[0m
 [2m26[0m │ const topLevelVariable5 = 5;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable5):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 2 comments
  [38;2;225;80;80;1m│[0m   [0] Line: " Line comment 5" at [559, 576]
  [38;2;225;80;80;1m│[0m   [1] Line: " Line comment 6" at [577, 594]
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:26:1]
 [2m25[0m │ const topLevelVariable4 = 4;
 [2m26[0m │ const topLevelVariable5 = 5;
    · [38;2;246;87;248m────────────────────────────[0m
 [2m27[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable6):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 2 comments
  [38;2;225;80;80;1m│[0m   [0] Line: " Line comment 5" at [559, 576]
  [38;2;225;80;80;1m│[0m   [1] Line: " Line comment 6" at [577, 594]
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:31:1]
 [2m30[0m │ 
 [2m31[0m │ const topLevelVariable6 = 6;
    · [38;2;246;87;248m────────────────────────────[0m
 [2m32[0m │ const topLevelVariable7 = 7;
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable7):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
    ╭─[[38;2;92;157;255;1mfiles/comments.js[0m:32:1]
 [2m31[0m │ const topLevelVariable6 = 6;
 [2m32[0m │ const topLevelVariable7 = 7;
    · [38;2;246;87;248m────────────────────────────[0m
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): getAllComments: 3 comments
  [38;2;225;80;80;1m│[0m   [0] Shebang: "/usr/bin/env node" at [0, 19]
  [38;2;225;80;80;1m│[0m   [1] Line: " Line comment after hashbang" at [20, 50]
  [38;2;225;80;80;1m│[0m   [2] Block: " Block comment after hashbang " at [51, 85][0m
   ╭─[[38;2;92;157;255;1mfiles/hashbang.js[0m:4:1]
 [2m3[0m │     /* Block comment after hashbang */
 [2m4[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m const topLevelVariable1 = 1;
 [2m5[0m │ [38;2;246;87;248m│[0m   const topLevelVariable2 = 2;
 [2m6[0m │ [38;2;246;87;248m│[0m   
 [2m7[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m export function topLevelFunction() {}
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): commentsExistBetween(topLevelVariable2, topLevelFunction): false[0m
   ╭─[[38;2;92;157;255;1mfiles/hashbang.js[0m:5:1]
 [2m4[0m │ const topLevelVariable1 = 1;
 [2m5[0m │ const topLevelVariable2 = 2;
   · [38;2;246;87;248m────────────────────────────[0m
 [2m6[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): commentsExistBetween(topLevelFunction, topLevelVariable2): false[0m
   ╭─[[38;2;92;157;255;1mfiles/hashbang.js[0m:7:8]
 [2m6[0m │ 
 [2m7[0m │ export function topLevelFunction() {}
   · [38;2;246;87;248m       ──────────────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable1):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 3 comments
  [38;2;225;80;80;1m│[0m   [0] Shebang: "/usr/bin/env node" at [0, 19]
  [38;2;225;80;80;1m│[0m   [1] Line: " Line comment after hashbang" at [20, 50]
  [38;2;225;80;80;1m│[0m   [2] Block: " Block comment after hashbang " at [51, 85]
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
   ╭─[[38;2;92;157;255;1mfiles/hashbang.js[0m:4:1]
 [2m3[0m │ /* Block comment after hashbang */
 [2m4[0m │ const topLevelVariable1 = 1;
   · [38;2;246;87;248m────────────────────────────[0m
 [2m5[0m │ const topLevelVariable2 = 2;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): VariableDeclaration(topLevelVariable2):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments
  [38;2;225;80;80;1m│[0m commentsExistBetween(id, init): false[0m
   ╭─[[38;2;92;157;255;1mfiles/hashbang.js[0m:5:1]
 [2m4[0m │ const topLevelVariable1 = 1;
 [2m5[0m │ const topLevelVariable2 = 2;
   · [38;2;246;87;248m────────────────────────────[0m
 [2m6[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-comments(test-comments): FunctionDeclaration(topLevelFunction):
  [38;2;225;80;80;1m│[0m getCommentsBefore: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsInside: 0 comments
  [38;2;225;80;80;1m│[0m getCommentsAfter: 0 comments[0m
   ╭─[[38;2;92;157;255;1mfiles/hashbang.js[0m:7:8]
 [2m6[0m │ 
 [2m7[0m │ export function topLevelFunction() {}
   · [38;2;246;87;248m       ──────────────────────────────[0m
   ╰────

Found 0 warnings and 32 errors.
Finished in Xms on 3 files with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
