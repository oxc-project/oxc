# Exit code
1

# stdout
```
  x utf16-plugin(no-debugger): debugger:
  | start/end: [0,9]
  | range: [0,9]
  | loc: [{"start":{"line":1,"column":0},"end":{"line":1,"column":9}}]
   ,-[files/index.js:1:1]
 1 | debugger;
   : ^^^^^^^^^
 2 | // £
   `----

  x utf16-plugin(no-debugger): program:
  | start/end: [0,47]
  | range: [0,47]
  | loc: [{"start":{"line":1,"column":0},"end":{"line":8,"column":0}}]
   ,-[files/index.js:1:1]
 1 | ,-> debugger;
 2 | |   // £
 3 | |   debugger;
 4 | |   // 🤨
 5 | |   {
 6 | |     debugger;
 7 | `-> }
   `----

  x utf16-plugin(no-debugger): debugger:
  | start/end: [15,24]
  | range: [15,24]
  | loc: [{"start":{"line":3,"column":0},"end":{"line":3,"column":9}}]
   ,-[files/index.js:3:1]
 2 | // £
 3 | debugger;
   : ^^^^^^^^^
 4 | // 🤨
   `----

  x utf16-plugin(no-debugger): debugger:
  | start/end: [35,44]
  | range: [35,44]
  | loc: [{"start":{"line":6,"column":2},"end":{"line":6,"column":11}}]
   ,-[files/index.js:6:3]
 5 | {
 6 |   debugger;
   :   ^^^^^^^^^
 7 | }
   `----

Found 0 warnings and 4 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
