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

  x options-plugin(merge-options):
  | options: [
  |   {
  |     "fromDefault": 1,
  |     "overrideDefault": 12,
  |     "nested": {
  |       "fromDefault": 3,
  |       "overrideDefault": 14,
  |       "fromConfig": 13
  |     },
  |     "fromConfig": 11
  |   },
  |   15,
  |   true,
  |   [],
  |   16
  | ]
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

  x options-plugin(no-options):
  | options: []
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

  x options-plugin(options):
  | options: [
  |   false,
  |   {
  |     "array": [
  |       {
  |         "deep": true,
  |         "str": "hello"
  |       },
  |       456
  |     ],
  |     "not": null
  |   }
  | ]
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

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
   ,-[files/nested/index.js:1:1]
 1 | let x;
   : ^
   `----

  x options-plugin(merge-options):
  | options: [
  |   {
  |     "fromDefault": 1,
  |     "overrideDefault": 2,
  |     "nested": {
  |       "fromDefault": 3,
  |       "overrideDefault": 4
  |     },
  |     "fromConfig": 21
  |   },
  |   {
  |     "fromDefault": 5,
  |     "fromConfig": 22
  |   },
  |   {
  |     "fromDefault": 6
  |   },
  |   7
  | ]
  | isDeepFrozen: true
   ,-[files/nested/index.js:1:1]
 1 | let x;
   : ^
   `----

  x options-plugin(no-options):
  | options: []
  | isDeepFrozen: true
   ,-[files/nested/index.js:1:1]
 1 | let x;
   : ^
   `----

  x options-plugin(options):
  | options: [
  |   {
  |     "somethingElse": true
  |   }
  | ]
  | isDeepFrozen: true
   ,-[files/nested/index.js:1:1]
 1 | let x;
   : ^
   `----

Found 0 warnings and 8 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
