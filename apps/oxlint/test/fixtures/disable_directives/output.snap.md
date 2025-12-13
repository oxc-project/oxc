# Exit code
1

# stdout
```
  x test-plugin(no-var): Use let or const instead of var
   ,-[files/index.js:1:1]
 1 | var shouldError = 1;
   : ^^^^^^^^^^^^^^^^^^^^
 2 | 
   `----

  x ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-debugger.html\eslint(no-debugger)]8;;\: `debugger` statement is not allowed
    ,-[files/index.js:10:1]
  9 | // should trigger an error
 10 | debugger;
    : ^^^^^^^^^
 11 | 
    `----
  help: Remove the debugger statement

  x test-plugin(no-var): Use let or const instead of var
    ,-[files/index.js:16:1]
 15 | /* oxlint-disable-next-line test-plugin */ // `test-plugin` should be `test-plugin/no-var`
 16 | var incorrectlyDisabled = 4;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 17 | 
    `----

  x test-plugin(no-var): Use let or const instead of var
    ,-[files/index.js:19:1]
 18 | /* oxlint-disable-next-line no-var */ // `no-var` should be `test-plugin/no-var`
 19 | var anotherIncorrectlyDisabled = 4;
    : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 20 | 
    `----

  x test-plugin(no-var): Use let or const instead of var
    ,-[files/index.js:22:1]
 21 | // This var should trigger an error again
 22 | var shouldErrorAgain = 3;
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
