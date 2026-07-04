# Exit code
1

# stdout
```
  x fix-plugin(no-var): Unexpected var, use let or const instead
   ,-[files/test.custom:1:1]
 1 | var foo = 1;
   : ^^^^^^^^^^^^
 2 | var bar = 2;
   `----

  ! eslint(no-unused-vars): Variable 'foo' is declared but never used. Unused variables should start with a '_'.
   ,-[files/test.custom:1:5]
 1 | var foo = 1;
   :     ^|^
   :      `-- 'foo' is declared here
 2 | var bar = 2;
   `----
  help: Consider removing this declaration.

  x fix-plugin(no-var): Unexpected var, use let or const instead
   ,-[files/test.custom:2:1]
 1 | var foo = 1;
 2 | var bar = 2;
   : ^^^^^^^^^^^^
 3 | const baz = 3;
   `----

  ! eslint(no-unused-vars): Variable 'bar' is declared but never used. Unused variables should start with a '_'.
   ,-[files/test.custom:2:5]
 1 | var foo = 1;
 2 | var bar = 2;
   :     ^|^
   :      `-- 'bar' is declared here
 3 | const baz = 3;
   `----
  help: Consider removing this declaration.

  ! eslint(no-unused-vars): Variable 'baz' is declared but never used. Unused variables should start with a '_'.
   ,-[files/test.custom:3:7]
 2 | var bar = 2;
 3 | const baz = 3;
   :       ^|^
   :        `-- 'baz' is declared here
   `----
  help: Consider removing this declaration.

Found 3 warnings and 2 errors.
Finished in Xms on 1 file with 96 rules using X threads.
```

# stderr
```
```
