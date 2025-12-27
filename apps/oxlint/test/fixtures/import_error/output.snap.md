# Exit code
1

# stdout
```
Failed to parse oxlint configuration file.

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mFailed to load JS plugin: ./plugin.ts
  [38;2;225;80;80;1m│[0m   Error: whoops!
  [38;2;225;80;80;1m│[0m     at file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/test/fixtures/import_error/plugin.ts:1:7
  [38;2;225;80;80;1m│[0m     at ModuleJob.run (node:internal/modules/esm/module_job:413:25)
  [38;2;225;80;80;1m│[0m     at async onImport.tracePromise.__proto__ (node:internal/modules/esm/loader:660:26)
  [38;2;225;80;80;1m│[0m     at async loadPlugin (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:15329:17)[0m
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
