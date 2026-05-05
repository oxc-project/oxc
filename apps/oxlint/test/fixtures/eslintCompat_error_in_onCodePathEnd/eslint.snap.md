# Exit code
2

# stdout
```
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

Oops! Something went wrong! :(

ESLint: 10.2.1

Error: `onCodePathEnd` CFG event handler threw
Occurred while linting <fixture>/files/1.js
Rule: "eslint-compat-plugin/throw-in-visit"
    at onCodePathEnd (<fixture>/plugin.ts:48:15)
```
