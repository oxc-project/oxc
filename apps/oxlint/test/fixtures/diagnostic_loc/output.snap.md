# Exit code
1

# stdout
```
  x loc-plugin(no-bugger): Bugger!
   ,-[files/index.js:1:3]
 1 | debugger;
   :   ^^^^^^
 2 | debugger;
   `----

  x loc-plugin(no-bugger): Bugger debugger debug!
   ,-[files/index.js:1:3]
 1 | ,-> debugger;
 2 | |   debugger;
 3 | `-> debugger;
   `----

  x loc-plugin(no-bugger): Bugger!
   ,-[files/index.js:2:3]
 1 | debugger;
 2 | debugger;
   :   ^^^^^^
 3 | debugger;
   `----

  x loc-plugin(no-bugger): Bugger!
   ,-[files/index.js:3:3]
 2 | debugger;
 3 | debugger;
   :   ^^^^^^
   `----

Found 0 warnings and 4 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
