# Exit code
1

# stdout
```
  x error-plugin(error): Visited nodes:
  | * onCodePathStart                     Program
  | * onCodePathSegmentStart              Program
  | * onCodePathSegmentEnd                Literal
  | * onCodePathSegmentStart              Literal
  | * onCodePathSegmentEnd                BlockStatement
  | * onCodePathSegmentStart              BlockStatement
  | * onCodePathSegmentLoop               WhileStatement
  | * onCodePathSegmentEnd                WhileStatement
  | * onUnreachableCodePathSegmentStart   WhileStatement
  | * onUnreachableCodePathSegmentEnd     Program
  | * onCodePathEnd                       Program
   ,-[files/index.js:1:1]
 1 | while (true) {}
   : ^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
