# Exit code
1

# stdout
```
  x context-plugin(log-context):
  | this === rule: true
  | id: context-plugin/log-context
  | filename: <fixture>/files/1.js
  | getFilename(): <fixture>/files/1.js
  | physicalFilename: <fixture>/files/1.js
  | getPhysicalFilename(): <fixture>/files/1.js
  | cwd: <fixture>
  | getCwd(): <fixture>
  | parserPath: undefined
   ,-[files/1.js:1:1]
 1 | let x;
   : ^
   `----

  x context-plugin(log-context):
  | this === undefined: true
   ,-[files/1.js:1:1]
 1 | let x;
   : ^^^^^^
   `----

  x context-plugin(log-context):
  | this === rule: true
  | id: context-plugin/log-context
  | filename: <fixture>/files/2.js
  | getFilename(): <fixture>/files/2.js
  | physicalFilename: <fixture>/files/2.js
  | getPhysicalFilename(): <fixture>/files/2.js
  | cwd: <fixture>
  | getCwd(): <fixture>
  | parserPath: undefined
   ,-[files/2.js:1:1]
 1 | let y;
   : ^
   `----

  x context-plugin(log-context):
  | this === undefined: true
   ,-[files/2.js:1:1]
 1 | let y;
   : ^^^^^^
   `----

Found 0 warnings and 4 errors.
Finished in Xms on 2 files with 1 rules using X threads.
```

# stderr
```
```
