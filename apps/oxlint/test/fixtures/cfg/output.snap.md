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

  x cfg-plugin(cfg): Visited nodes:
  | * onCodePathStart                     Program
  | * onCodePathSegmentStart              Program
  | * onCodePathSegmentEnd                Program
  | * onCodePathEnd                       Program
   ,-[files/modules.ts:1:1]
 1 | declare module "virtual:foo";
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x cfg-plugin(cfg): Visited nodes:
  | * onCodePathStart                     Program
  | * onCodePathSegmentStart              Program
  | * onCodePathSegmentEnd                Program
  | * onCodePathEnd                       Program
   ,-[files/shims.d.ts:1:1]
 1 | declare module "*.css";
   : ^^^^^^^^^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 3 errors.
Finished in Xms on 3 files with 2 rules using X threads.
```

# stderr
```
```
