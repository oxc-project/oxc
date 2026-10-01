# Exit code
1

# stdout
```
  x globals-env-builtin-plugin(unresolved-globals): env: {"browser":true}, unresolved: Number
   ,-[files/builtin_disabled.js:2:1]
 1 | // `builtin` env is explicitly disabled for this file, so ES builtins should not resolve
 2 | Number;
   : ^^^^^^^^
   `----

  x globals-env-builtin-plugin(unresolved-globals): env: {"browser":true,"builtin":true}, unresolved: none
   ,-[files/index.js:2:1]
 1 | // `env` in config doesn't include `builtin`, but ES builtins should still resolve
 2 | Number;
   : ^^^^^^^^
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 2 files with 1 rules using X threads.
```

# stderr
```
```
