# Exit code
1

# stdout
```
  x test-plugin(no-var): Use let or const instead of var
   ,-[files/index.js:3:1]
 2 | 
 3 | var shouldError = 1;
   : ^^^^^^^^^^^^^^^^^^^^
 4 | 
   `----

  x ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-debugger.html\eslint(no-debugger)]8;;\: `debugger` statement is not allowed
    ,-[files/index.js:12:1]
 11 | // should trigger an error
 12 | debugger;
    : ^^^^^^^^^
 13 | 
    `----
  help: Remove the debugger statement

  x test-plugin(no-var): Use let or const instead of var
    ,-[files/index.js:18:1]
 17 | /* oxlint-disable-next-line test-plugin */ // `test-plugin` should be `test-plugin/no-var`
 18 | var incorrectlyDisabled = 4;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 19 | 
    `----

  x test-plugin(no-var): Use let or const instead of var
    ,-[files/index.js:21:1]
 20 | /* oxlint-disable-next-line no-var */ // `no-var` should be `test-plugin/no-var`
 21 | var anotherIncorrectlyDisabled = 4;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 22 | 
    `----

  x test-plugin(no-var): Use let or const instead of var
    ,-[files/index.js:24:1]
 23 | // This var should trigger an error again
 24 | var shouldErrorAgain = 3;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^
    `----

Found 0 warnings and 5 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
