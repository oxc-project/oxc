# Exit code
1

# stdout
```
  x eslint(no-unused-vars): Variable 'trulyUnusedÜ' is declared but never used. Unused variables should start with a '_'.
   ,-[files/test.custom:8:7]
 7 | const café = 42;
 8 | const trulyUnusedÜ = 99;
   :       ^^^^^^|^^^^^
   :             `-- 'trulyUnusedÜ' is declared here
 9 | 
   `----
  help: Consider removing this declaration.

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 96 rules using X threads.
```

# stderr
```
```
