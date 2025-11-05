# Exit code
1

# stdout
```
  x Error running JS plugin.
  | File path: <root>/apps/oxlint/test/fixtures/oxc_issue_15325/files/index.js
  | TypeError: Cannot read private member #internal from an object whose class did not declare it
  |     at Object.create (<root>/apps/oxlint/test/fixtures/oxc_issue_15325/plugin.ts:10:33)
  |     at Object.create (<root>/apps/oxlint/test/fixtures/oxc_issue_15325/plugin.ts:36:25)

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
