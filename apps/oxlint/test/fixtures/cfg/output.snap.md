# Exit code
1

# stdout
```
  x cfg-plugin(cfg): Visited nodes:
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
Finished in Xms on 1 file with 2 rules using X threads.
```

# stderr
```
```
