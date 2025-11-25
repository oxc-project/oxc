# Exit code
1

# stdout
```
  x scope-plugin(scope): getDeclaredVariables(): topLevelConstant
   ,-[files/index.cjs:1:1]
 1 | const topLevelConstant = 1;
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^
 2 | let topLevelLet = 2;
   `----

  x scope-plugin(scope): isGlobalReference(topLevelConstant): false
   ,-[files/index.cjs:1:7]
 1 | const topLevelConstant = 1;
   :       ^^^^^^^^^^^^^^^^
 2 | let topLevelLet = 2;
   `----

  x scope-plugin(scope): getDeclaredVariables(): topLevelLet
   ,-[files/index.cjs:2:1]
 1 | const topLevelConstant = 1;
 2 | let topLevelLet = 2;
   : ^^^^^^^^^^^^^^^^^^^^
 3 | var topLevelVar = 3;
   `----

  x scope-plugin(scope): isGlobalReference(topLevelLet): false
   ,-[files/index.cjs:2:5]
 1 | const topLevelConstant = 1;
 2 | let topLevelLet = 2;
   :     ^^^^^^^^^^^
 3 | var topLevelVar = 3;
   `----

  x scope-plugin(scope): getDeclaredVariables(): topLevelVar
   ,-[files/index.cjs:3:1]
 2 | let topLevelLet = 2;
 3 | var topLevelVar = 3;
   : ^^^^^^^^^^^^^^^^^^^^
 4 | 
   `----

  x scope-plugin(scope): isGlobalReference(topLevelVar): false
   ,-[files/index.cjs:3:5]
 2 | let topLevelLet = 2;
 3 | var topLevelVar = 3;
   :     ^^^^^^^^^^^
 4 | 
   `----

  x scope-plugin(scope): getScope(topLevelFunction):
  | type: function
  | isStrict: false
  | variables: [arguments, innerFunction]
  | through: [Object]
  | upper type: global
    ,-[files/index.cjs:5:1]
  4 |     
  5 | ,-> function topLevelFunction() {
  6 | |     function innerFunction() {
  7 | |       function nestedFunction() {
  8 | |         "use strict";
  9 | |       }
 10 | |     }
 11 | |     return Object;
 12 | `-> }
 13 |     
    `----

  x scope-plugin(scope): isGlobalReference(topLevelFunction): false
   ,-[files/index.cjs:5:10]
 4 | 
 5 | function topLevelFunction() {
   :          ^^^^^^^^^^^^^^^^
 6 |   function innerFunction() {
   `----

  x scope-plugin(scope): getScope(innerFunction):
  | type: function
  | isStrict: false
  | variables: [arguments, nestedFunction]
  | through: []
  | upper type: function
    ,-[files/index.cjs:6:3]
  5 |     function topLevelFunction() {
  6 | ,->   function innerFunction() {
  7 | |       function nestedFunction() {
  8 | |         "use strict";
  9 | |       }
 10 | `->   }
 11 |       return Object;
    `----

  x scope-plugin(scope): isGlobalReference(innerFunction): false
   ,-[files/index.cjs:6:12]
 5 | function topLevelFunction() {
 6 |   function innerFunction() {
   :            ^^^^^^^^^^^^^
 7 |     function nestedFunction() {
   `----

  x scope-plugin(scope): getScope(nestedFunction):
  | type: function
  | isStrict: true
  | variables: [arguments]
  | through: []
  | upper type: function
    ,-[files/index.cjs:7:5]
  6 |       function innerFunction() {
  7 | ,->     function nestedFunction() {
  8 | |         "use strict";
  9 | `->     }
 10 |       }
    `----

  x scope-plugin(scope): isGlobalReference(nestedFunction): false
   ,-[files/index.cjs:7:14]
 6 |   function innerFunction() {
 7 |     function nestedFunction() {
   :              ^^^^^^^^^^^^^^
 8 |       "use strict";
   `----

  x scope-plugin(scope): isGlobalReference(Object): true
    ,-[files/index.cjs:11:10]
 10 |   }
 11 |   return Object;
    :          ^^^^^^
 12 | }
    `----

  x scope-plugin(scope): isGlobalReference(module): false
    ,-[files/index.cjs:14:1]
 13 | 
 14 | module.exports = topLevelFunction();
    : ^^^^^^
    `----

  x scope-plugin(scope): isGlobalReference(exports): false
    ,-[files/index.cjs:14:8]
 13 | 
 14 | module.exports = topLevelFunction();
    :        ^^^^^^^
    `----

  x scope-plugin(scope): isGlobalReference(topLevelFunction): false
    ,-[files/index.cjs:14:18]
 13 | 
 14 | module.exports = topLevelFunction();
    :                  ^^^^^^^^^^^^^^^^
    `----

  x scope-plugin(scope): getDeclaredVariables(): topLevelConstant, secondTopLevelConstant
   ,-[files/index.js:1:1]
 1 | ,-> const topLevelConstant = 1,
 2 | `->   secondTopLevelConstant = 2;
 3 |     
   `----

  x scope-plugin(scope): isGlobalReference(topLevelConstant): false
   ,-[files/index.js:1:7]
 1 | const topLevelConstant = 1,
   :       ^^^^^^^^^^^^^^^^
 2 |   secondTopLevelConstant = 2;
   `----

  x scope-plugin(scope): isGlobalReference(secondTopLevelConstant): false
   ,-[files/index.js:2:3]
 1 | const topLevelConstant = 1,
 2 |   secondTopLevelConstant = 2;
   :   ^^^^^^^^^^^^^^^^^^^^^^
 3 | 
   `----

  x scope-plugin(scope): getScope(topLevelFunction):
  | type: function
  | isStrict: true
  | variables: [arguments, param, localConstant]
  | through: [topLevelConstant, Math]
  | upper type: module
    ,-[files/index.js:4:1]
  3 |     
  4 | ,-> function topLevelFunction(param) {
  5 | |     const localConstant = topLevelConstant + param;
  6 | |     return function innerFunction() {
  7 | |       return localConstant + Math.PI;
  8 | |     };
  9 | `-> }
 10 |     
    `----

  x scope-plugin(scope): isGlobalReference(topLevelFunction): false
   ,-[files/index.js:4:10]
 3 | 
 4 | function topLevelFunction(param) {
   :          ^^^^^^^^^^^^^^^^
 5 |   const localConstant = topLevelConstant + param;
   `----

  x scope-plugin(scope): isGlobalReference(param): false
   ,-[files/index.js:4:27]
 3 | 
 4 | function topLevelFunction(param) {
   :                           ^^^^^
 5 |   const localConstant = topLevelConstant + param;
   `----

  x scope-plugin(scope): getDeclaredVariables(): localConstant
   ,-[files/index.js:5:3]
 4 | function topLevelFunction(param) {
 5 |   const localConstant = topLevelConstant + param;
   :   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 6 |   return function innerFunction() {
   `----

  x scope-plugin(scope): isGlobalReference(localConstant): false
   ,-[files/index.js:5:9]
 4 | function topLevelFunction(param) {
 5 |   const localConstant = topLevelConstant + param;
   :         ^^^^^^^^^^^^^
 6 |   return function innerFunction() {
   `----

  x scope-plugin(scope): isGlobalReference(topLevelConstant): false
   ,-[files/index.js:5:25]
 4 | function topLevelFunction(param) {
 5 |   const localConstant = topLevelConstant + param;
   :                         ^^^^^^^^^^^^^^^^
 6 |   return function innerFunction() {
   `----

  x scope-plugin(scope): isGlobalReference(param): false
   ,-[files/index.js:5:44]
 4 | function topLevelFunction(param) {
 5 |   const localConstant = topLevelConstant + param;
   :                                            ^^^^^
 6 |   return function innerFunction() {
   `----

  x scope-plugin(scope): isGlobalReference(innerFunction): false
   ,-[files/index.js:6:19]
 5 |   const localConstant = topLevelConstant + param;
 6 |   return function innerFunction() {
   :                   ^^^^^^^^^^^^^
 7 |     return localConstant + Math.PI;
   `----

  x scope-plugin(scope): isGlobalReference(localConstant): false
   ,-[files/index.js:7:12]
 6 |   return function innerFunction() {
 7 |     return localConstant + Math.PI;
   :            ^^^^^^^^^^^^^
 8 |   };
   `----

  x scope-plugin(scope): isGlobalReference(Math): true
   ,-[files/index.js:7:28]
 6 |   return function innerFunction() {
 7 |     return localConstant + Math.PI;
   :                            ^^^^
 8 |   };
   `----

  x scope-plugin(scope): isGlobalReference(PI): false
   ,-[files/index.js:7:33]
 6 |   return function innerFunction() {
 7 |     return localConstant + Math.PI;
   :                                 ^^
 8 |   };
   `----

  x scope-plugin(scope): getDeclaredVariables(): topLevelExport
    ,-[files/index.js:11:8]
 10 | 
 11 | export const topLevelExport = topLevelFunction(2);
    :        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    `----

  x scope-plugin(scope): isGlobalReference(topLevelExport): false
    ,-[files/index.js:11:14]
 10 | 
 11 | export const topLevelExport = topLevelFunction(2);
    :              ^^^^^^^^^^^^^^
    `----

  x scope-plugin(scope): isGlobalReference(topLevelFunction): false
    ,-[files/index.js:11:31]
 10 | 
 11 | export const topLevelExport = topLevelFunction(2);
    :                               ^^^^^^^^^^^^^^^^
    `----

Found 0 warnings and 33 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
