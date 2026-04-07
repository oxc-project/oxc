# Exit code
1

# stdout
```
  x eslint-js(array-bracket-newline): A linebreak is required before ']'.
   ,-[files/index.js:3:10]
 2 | const a = [
 3 |   1, 2, 3];
   :          ^
 4 | 
   `----

  x eslint-js(no-restricted-syntax): Use `new` keyword when throwing an `Error`.
   ,-[files/index.js:6:7]
 5 | // Violation: no-restricted-syntax (throw Error without `new`)
 6 | throw TypeError("bad");
   :       ^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 1 file with 2 rules using X threads.
```

# stderr
```
```
