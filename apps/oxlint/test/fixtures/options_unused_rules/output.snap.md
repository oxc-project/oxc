# Exit code
1

# stdout
```
  x options-unused-rules-plugin(config-options):
  | options: [
  |   {
  |     "fromConfig": 1,
  |     "fromDefault": 20,
  |     "fromSchema": 10
  |   }
  | ]
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

  x options-unused-rules-plugin(default-options):
  | options: [
  |   {
  |     "fromDefault": 20,
  |     "fromSchema": 10
  |   }
  | ]
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 1 file with 2 rules using X threads.
```

# stderr
```
```
