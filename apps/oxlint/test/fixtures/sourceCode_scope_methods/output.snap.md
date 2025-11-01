# Exit code
1

# stdout
```
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

  x scope-plugin(scope): getScope(topLevelFunction): type: function
  | isStrict: true
  | vars: [arguments, param, localConstant]
  | through: [topLevelConstant, Math]
  | upper: module
  | 
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

Found 0 warnings and 17 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
