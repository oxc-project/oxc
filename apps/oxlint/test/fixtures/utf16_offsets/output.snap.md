# Exit code
1

# stdout
```
  ! ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-debugger.html\eslint(no-debugger)]8;;\: `debugger` statement is not allowed
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | // £
   `----
  help: Remove the debugger statement

  x utf16-plugin(no-debugger): debugger:
  | start/end: [0,9]
  | range: [0,9]
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | // £
   `----

  x utf16-plugin(no-debugger): program:
  | start/end: [0,47]
  | range: [0,47]
   ,-[files/index.js:1:1]
 1 | ,-> debugger;
 2 | |   // £
 3 | |   debugger;
 4 | |   // 🤨
 5 | |   {
 6 | |     debugger;
 7 | `-> }
   `----

  ! ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-debugger.html\eslint(no-debugger)]8;;\: `debugger` statement is not allowed
   ,-[files/index.js:3:1]
 2 | // £
 3 | debugger;
   : ^^^^^^^^^
 4 | // 🤨
   `----
  help: Remove the debugger statement

  x utf16-plugin(no-debugger): debugger:
  | start/end: [15,24]
  | range: [15,24]
   ,-[files/index.js:3:1]
 2 | // £
 3 | debugger;
   : ^^^^^^^^^
 4 | // 🤨
   `----

  ! ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-debugger.html\eslint(no-debugger)]8;;\: `debugger` statement is not allowed
   ,-[files/index.js:6:3]
 5 | {
 6 |   debugger;
   :   ^^^^^^^^^
 7 | }
   `----
  help: Remove the debugger statement

  x utf16-plugin(no-debugger): debugger:
  | start/end: [35,44]
  | range: [35,44]
   ,-[files/index.js:6:3]
 5 | {
 6 |   debugger;
   :   ^^^^^^^^^
 7 | }
   `----

Found 3 warnings and 4 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
