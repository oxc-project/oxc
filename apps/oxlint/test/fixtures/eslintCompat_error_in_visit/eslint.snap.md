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
  "after: tracking",
  "after: throw-in-visit",
  "after: tracking-late"
]

Oops! Something went wrong! :(

ESLint: 9.39.4

Error: `Identifier` visit function threw
Occurred while linting <fixture>/files/1.js:1
Rule: "eslint-compat-plugin/throw-in-visit"
    at Identifier (<fixture>/plugin.ts:42:15)
```
