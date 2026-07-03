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

  x fix-plugin(no-var): Unexpected var, use let or const instead
   ,-[files/test.custom:2:1]
 1 | var foo = 1;
 2 | var bar = 2;
   : ^^^^^^^^^^^^
 3 | const baz = 3;
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 1 file with 96 rules using X threads.
```

# stderr
```
```
