# Exit code
1

# stdout
```
Failed to parse oxlint configuration file.

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mFailed to load JS plugin: ./plugin.ts
  [38;2;225;80;80;1m│[0m   Error: Whoops!
  [38;2;225;80;80;1m│[0m     at Object.createOnce (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/test/fixtures/lint_createOnce_error/plugin.ts:10:15)
  [38;2;225;80;80;1m│[0m     at registerPlugin (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:15382:32)
  [38;2;225;80;80;1m│[0m     at loadPlugin (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:15330:35)[0m
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
