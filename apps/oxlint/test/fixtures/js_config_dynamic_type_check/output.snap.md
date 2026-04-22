# Exit code
1

# stdout
```
  x typescript-eslint(no-floating-promises): Promises must be awaited, add void operator to ignore.
   ,-[files/test.ts:2:1]
 1 | const floating = Promise.resolve("ok");
 2 | floating;
   : ^^^^^^^^^
 3 | 
   `----
  help: The promise must end with a call to .catch, or end with a call to .then with a rejection handler, or be explicitly marked as ignored with the `void` operator.

  x typescript(TS2322): Type 'string' is not assignable to type 'number'.
   ,-[files/test.ts:4:7]
 3 | 
 4 | const value: number = "42";
   :       ^^^^^
 5 | void value;
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
```
