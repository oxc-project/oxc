# Exit code
1

# stdout
```
  x context-settings-plugin(log-settings): setting arraySetting: ["item1","item2"]
   ,-[files/test.js:1:1]
 1 | console.log('test file');
   : ^
   `----

  x context-settings-plugin(log-settings): setting nestedSetting: {"key":"nestedValue","number":42}
   ,-[files/test.js:1:1]
 1 | console.log('test file');
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
