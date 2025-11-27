# Exit code
1

# stdout
```
  x options-plugin(default-options):
  | options: [
  |   "string",
  |   123,
  |   true,
  |   {
  |     "toBe": false,
  |     "notToBe": true
  |   },
  |   {
  |     "deep": [
  |       {
  |         "deeper": {
  |           "evenDeeper": [
  |             {
  |               "soDeep": {
  |                 "soSoDeep": true
  |               }
  |             }
  |           ]
  |         }
  |       }
  |     ]
  |   }
  | ]
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

  x options-plugin(options):
  | options: []
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
