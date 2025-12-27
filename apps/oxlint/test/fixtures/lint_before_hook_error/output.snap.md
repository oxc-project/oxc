# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mError running JS plugin.
  [38;2;225;80;80;1m│[0m File path: <fixture>/files/index.js
  [38;2;225;80;80;1m│[0m Error: Whoops!
  [38;2;225;80;80;1m│[0m     at before (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/test/fixtures/lint_before_hook_error/plugin.ts:12:19)
  [38;2;225;80;80;1m│[0m     at lintFileImpl (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:21355:31)
  [38;2;225;80;80;1m│[0m     at lintFile (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:21328:3)
  [38;2;225;80;80;1m│[0m     at lintFileWrapper (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/cli.js:11:41)[0m

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
