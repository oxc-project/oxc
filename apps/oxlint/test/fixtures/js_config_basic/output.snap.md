# Exit code
1

# stdout
```
  x ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-debugger.html\eslint(no-debugger)]8;;\: `debugger` statement is not allowed
   ,-[files/test.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | if (x == 1) {
   `----
  help: Remove the debugger statement

  ! ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/eqeqeq.html\eslint(eqeqeq)]8;;\: Expected === and instead saw ==
   ,-[files/test.js:2:7]
 1 | debugger;
 2 | if (x == 1) {
   :       ^^
 3 | }
   `----
  help: Prefer === operator

Found 1 warning and 1 error.
Finished in Xms on 1 file with 90 rules using X threads.
```

# stderr
```
```
