# Exit code
1

# stdout
```
  x Error running JS plugin.
  | File path: <root>/apps/oxlint/test/fixtures/custom_plugin_lint_fix_error/files/index.js
  | Error: Whoops!
  |     at Object.fix (<root>/apps/oxlint/test/fixtures/custom_plugin_lint_fix_error/plugin.ts:16:23)
  |     at Identifier (<root>/apps/oxlint/test/fixtures/custom_plugin_lint_fix_error/plugin.ts:12:21)

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
