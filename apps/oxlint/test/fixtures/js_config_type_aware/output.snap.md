# Exit code
1

# stdout
```
  x typescript-eslint(no-floating-promises): Promises must be awaited, add void operator to ignore.
   ,-[files/test.js:3:1]
 2 | 
 3 | floating;
   : ^^^^^^^^^
   `----
  help: The promise must end with a call to .catch, or end with a call to .then with a rejection handler, or be explicitly marked as ignored with the `void` operator.

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
