# Exit code
1

# stdout
```
  x test-plugin(no-var): Use let or const instead of var
   ,-[files/index.js:2:1]
 1 | // oxlint-disable-next-line test-plugin/no-var
 2 | var shouldError = 1;
   : ^^^^^^^^^^^^^^^^^^^^
 3 | 
   `----

  x test-plugin(no-var): Use let or const instead of var
   ,-[files/index.js:5:1]
 4 | // eslint-disable-next-line test-plugin/no-var
 5 | var shouldAlsoError = 2;
   : ^^^^^^^^^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
