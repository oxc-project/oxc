# Exit code
1

# stdout
```
  x globals-env-builtin-plugin(unresolved-globals): Unresolved global: Number
   ,-[files/builtin_disabled.js:2:1]
 1 | // `builtin` env is explicitly disabled for this file, so ES builtins should not resolve
 2 | Number.isFinite(1);
   : ^^^^^^
 3 | 
   `----

  x globals-env-builtin-plugin(unresolved-globals): env: {"browser":true}
   ,-[files/builtin_disabled.js:2:1]
 1 |     // `builtin` env is explicitly disabled for this file, so ES builtins should not resolve
 2 | ,-> Number.isFinite(1);
 3 | |   
 4 | |   // Enabled by `browser` env
 5 | `-> window.alert("hello");
   `----

  x globals-env-builtin-plugin(unresolved-globals): env: {"browser":true,"builtin":true}
   ,-[files/index.js:2:1]
 1 |     // ES builtins should resolve, even though `env` in config does not include `builtin`
 2 | ,-> Number.isFinite(1);
 3 | |   
 4 | |   // Enabled by `browser` env
 5 | |   window.alert("hello");
 6 | |   
 7 | |   // Not defined anywhere
 8 | `-> notDefinedAnywhere;
   `----

  x globals-env-builtin-plugin(unresolved-globals): Unresolved global: notDefinedAnywhere
   ,-[files/index.js:8:1]
 7 | // Not defined anywhere
 8 | notDefinedAnywhere;
   : ^^^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 4 errors.
Finished in Xms on 2 files with 1 rules using X threads.
```

# stderr
```
```
