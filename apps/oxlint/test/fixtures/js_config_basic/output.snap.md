# Exit code
1

# stdout
```
  x eslint(no-debugger): `debugger` statement is not allowed
   ,-[files/test.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | if (x == 1) {
   `----
  help: Remove the debugger statement

  ! eslint(eqeqeq): Expected === and instead saw ==
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
