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

  x options-plugin(empty-default-options):
  | options: [
  |   "from-config",
  |   42
  | ]
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

  x options-plugin(merge-options):
  | options: [
  |   {
  |     "fromConfig": 11,
  |     "overrideDefault": 12,
  |     "nested": {
  |       "fromConfig": 13,
  |       "overrideDefault": 14,
  |       "fromConfigObject": {
  |         "objectKey": 17
  |       },
  |       "fromDefault": 3
  |     },
  |     "fromDefault": 1
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

  x options-plugin(schema-and-default-options):
  | options: [
  |   {
  |     "fromConfig": 72,
  |     "overrideSchemaByConfig": 32,
  |     "overrideDefaultOptionsByConfig": 62,
  |     "overrideBothByConfig": 42,
  |     "fromDefaultOptions": 51,
  |     "overrideSchemaByDefaultOptions": 21,
  |     "fromSchema": 10
  |   }
  | ]
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

  x options-plugin(schema-and-empty-default-options):
  | options: [
  |   {
  |     "fromConfig": 72,
  |     "overrideSchemaByConfig": 32,
  |     "overrideDefaultOptionsByConfig": 62,
  |     "overrideBothByConfig": 42,
  |     "fromSchema": 10,
  |     "overrideSchemaByDefaultOptions": 20
  |   }
  | ]
  | isDeepFrozen: true
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^
   `----

  x options-plugin(schema-defaults):
  | options: []
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

  x options-plugin(empty-default-options):
  | options: []
  | isDeepFrozen: true
   ,-[files/nested/index.js:1:1]
 1 | let x;
   : ^
   `----

  x options-plugin(merge-options):
  | options: [
  |   {
  |     "fromConfig": 21,
  |     "fromDefault": 1,
  |     "overrideDefault": 2,
  |     "nested": {
  |       "fromDefault": 3,
  |       "overrideDefault": 4
  |     }
  |   },
  |   {
  |     "fromConfig": 22,
  |     "fromDefault": 5
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

  x options-plugin(schema-and-default-options):
  | options: [
  |   {
  |     "fromDefaultOptions": 51,
  |     "overrideDefaultOptionsByConfig": 61,
  |     "overrideSchemaByDefaultOptions": 21,
  |     "overrideBothByConfig": 41,
  |     "fromSchema": 10,
  |     "overrideSchemaByConfig": 30
  |   }
  | ]
  | isDeepFrozen: true
   ,-[files/nested/index.js:1:1]
 1 | let x;
   : ^
   `----

  x options-plugin(schema-and-empty-default-options):
  | options: []
  | isDeepFrozen: true
   ,-[files/nested/index.js:1:1]
 1 | let x;
   : ^
   `----

  x options-plugin(schema-defaults):
  | options: [
  |   {
  |     "overrideSchema": 21,
  |     "fromConfig": 31,
  |     "fromSchema": 10
  |   },
  |   "config-string"
  | ]
  | isDeepFrozen: true
   ,-[files/nested/index.js:1:1]
 1 | let x;
   : ^
   `----

Found 0 warnings and 16 errors.
Finished in Xms on 2 files with 8 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
