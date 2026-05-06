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

Oops! Something went wrong! :(

ESLint: 10.2.1

Error: `after` hook threw
Occurred while linting <fixture>/files/1.js
Rule: "eslint-compat-plugin/tracking-late"
    at after (<fixture>/plugin.ts:51:15)
```
