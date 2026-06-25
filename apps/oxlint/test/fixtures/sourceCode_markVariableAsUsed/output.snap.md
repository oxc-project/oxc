# Exit code
1

# stdout
```
  x mark-used-plugin(mark-used): [1] mark `unusedTopLevel` from Program:
  | before: false
  | result: true
  | after: true
   ,-[files/index.cjs:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [2] mark `nonExistent` from Program:
  | result: false
   ,-[files/index.cjs:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [3] mark `shadowedName` (no refNode):
  | before: false
  | result: true
  | after: true
   ,-[files/index.cjs:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [4] mark `nestedVar` from outer:
  | before: false
  | result: true
  | after: true
   ,-[files/index.cjs:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [5] mark `unusedTopLevel2` from outer:
  | before: false
  | result: true
  | after: true
   ,-[files/index.cjs:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [6] mark `nestedVar2` from inner:
  | before: false
  | result: true
  | after: true
   ,-[files/index.cjs:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [7] mark `doesNotExist` from inner:
  | result: false
   ,-[files/index.cjs:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [1] mark `unusedTopLevel` from Program:
  | before: false
  | result: true
  | after: true
   ,-[files/index.js:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [2] mark `nonExistent` from Program:
  | result: false
   ,-[files/index.js:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [3] mark `shadowedName` (no refNode):
  | before: false
  | result: true
  | after: true
   ,-[files/index.js:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [4] mark `nestedVar` from outer:
  | before: false
  | result: true
  | after: true
   ,-[files/index.js:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [5] mark `unusedTopLevel2` from outer:
  | before: false
  | result: true
  | after: true
   ,-[files/index.js:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [6] mark `nestedVar2` from inner:
  | before: false
  | result: true
  | after: true
   ,-[files/index.js:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

  x mark-used-plugin(mark-used): [7] mark `doesNotExist` from inner:
  | result: false
   ,-[files/index.js:1:1]
 1 | // `unusedTopLevel` is declared but never referenced in code,
   : ^
 2 | // so `eslintUsed` should start as `false`.
   `----

Found 0 warnings and 14 errors.
Finished in Xms on 2 files with 1 rules using X threads.
```

# stderr
```
```
