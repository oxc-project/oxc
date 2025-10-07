# Exit code
1

# stdout
```
  x Error running JS plugin.
  | File path: <root>/apps/oxlint/test/fixtures/custom_plugin_lint_after_hook_error/files/index.js
  | Error: Whoops!
  |     at after (<root>/apps/oxlint/test/fixtures/custom_plugin_lint_after_hook_error/plugin.ts:13:19)

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
