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

  x loc-plugin(no-bugger): Misaligned location
   ,-[files/index.js:1:4]
 1 | debugger;
   :    ^
 2 | debugger;
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

  x Error running JS plugin.
  | File path: <fixture>/files/negative-loc.js
  | RangeError: Line/column pair translates to an out of range offset
  |     at Program (<fixture>/plugin.ts:14:23)

  x Error running JS plugin.
  | File path: <fixture>/files/negative-node.js
  | TypeError: `node.range[0]` and `node.range[1]` must be non-negative integers
  |     at Program (<fixture>/plugin.ts:22:23)

  x loc-plugin(no-bugger): Out-of-range node
   ,-[files/out-of-range-node.js:3:1]
 2 | debugger;
   `----

Found 0 warnings and 8 errors.
Finished in Xms on 4 files with 1 rules using X threads.
```

# stderr
```
```
