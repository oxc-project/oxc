# Exit code
1

# stdout
```
  x basic-custom-plugin(no-debugger): Unexpected Debugger Statement
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | 
   `----

  x basic-custom-plugin(no-debugger-2): Unexpected Debugger Statement
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | 
   `----

  ! ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-debugger.html\eslint(no-debugger)]8;;\: `debugger` statement is not allowed
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | 
   `----
  help: Remove the debugger statement

  x basic-custom-plugin(no-identifiers-named-foo): Unexpected Identifier named foo
   ,-[files/index.js:3:1]
 2 | 
 3 | foo;
   : ^^^
   `----

  ! ]8;;https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-unused-expressions.html\eslint(no-unused-expressions)]8;;\: Expected expression to be used
   ,-[files/index.js:3:1]
 2 | 
 3 | foo;
   : ^^^^
   `----
  help: Consider using this expression or removing it

Found 2 warnings and 3 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
