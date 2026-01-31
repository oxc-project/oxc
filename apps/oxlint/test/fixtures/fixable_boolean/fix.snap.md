# Exit code
1

# stdout
```
  x fixable-boolean-plugin(no-console-false): Console with fixable: false
   ,-[files/index.js:2:1]
 1 | debugger;
 2 | console.log("test");
   : ^^^^^^^^^^^^^^^^^^^
   `----

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 2 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```

# File altered: files/index.js
```

console.log("test");

```
