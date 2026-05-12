# Exit code
1

# stdout
```
  x Error running JS plugin.
  | File path: <fixture>/files/1.js
  | Error: `onCodePathEnd` CFG event handler threw
  |     at onCodePathEnd (<fixture>/plugin.ts:48:15)

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 3 rules using X threads.
```

# stderr
```
filename: <fixture>/files/1.js
events:
[
  "before: tracking",
  "before: throw-in-visit",
  "before: tracking-late",
  "Identifier: tracking",
  "Identifier: throw-in-visit",
  "Identifier: tracking-late",
  "Program:exit: tracking",
  "Program:exit: throw-in-visit",
  "Program:exit: tracking-late",
  "onCodePathEnd: tracking",
  "onCodePathEnd: throw-in-visit",
  "after: tracking",
  "after: throw-in-visit",
  "after: tracking-late"
]
```
