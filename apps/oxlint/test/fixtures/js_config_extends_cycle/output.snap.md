# Exit code
1

# stdout
```
Failed to parse oxlint configuration file.

  x Failed to load config: <fixture>/oxlint.config.ts
  | 
  | Error: `extends` contains a circular reference.
  | 
  | <root>.extends[0].extends[0].extends[0] points back to <root>.extends[0]
  | Cycle: <root>.extends[0] -> <root>.extends[0].extends[0] -> <root>.extends[0]
```

# stderr
```
```
