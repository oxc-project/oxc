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
  "before: throw-in-before",
  "after: tracking"
]

Oops! Something went wrong! :(

ESLint: 10.2.1

Error: Error while loading rule 'test-plugin/throw-in-before': `before` hook threw
Occurred while linting <fixture>/files/1.js
    at before (<fixture>/plugin.ts:44:15)
```
