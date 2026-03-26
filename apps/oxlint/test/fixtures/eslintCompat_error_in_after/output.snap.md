# Exit code
1

# stdout
```
  x Error running JS plugin.
  | File path: <fixture>/files/1.js
  | Error: `after` hook threw
  |     at after (<fixture>/plugin.ts:51:15)

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 3 rules using X threads.
```

# stderr
```
filename: <fixture>/files/1.js
events:
[
  "before: tracking",
  "before: throw-in-after",
  "before: tracking-late",
  "Identifier: tracking",
  "Identifier: throw-in-after",
  "Identifier: tracking-late",
  "Program:exit: tracking",
  "Program:exit: throw-in-after",
  "Program:exit: tracking-late",
  "onCodePathEnd: tracking",
  "onCodePathEnd: throw-in-after",
  "onCodePathEnd: tracking-late",
  "after: tracking",
  "after: throw-in-after",
  "after: tracking-late"
]
```
