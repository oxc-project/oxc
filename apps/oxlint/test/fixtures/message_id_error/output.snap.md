# Exit code
1

# stdout
```
  x Error running JS plugin.
  | File path: <root>/apps/oxlint/test/fixtures/message_id_error/files/index.js
  | Error: Unknown messageId 'unknownMessage'. Available `messageIds`: 'validMessage'
  |     at DebuggerStatement (<root>/apps/oxlint/test/fixtures/message_id_error/plugin.ts:18:21)

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
