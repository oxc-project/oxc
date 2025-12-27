# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mError running JS plugin.
  [38;2;225;80;80;1m│[0m File path: <fixture>/files/2.js
  [38;2;225;80;80;1m│[0m AssertionError [ERR_ASSERTION]: The expression evaluated to a falsy value:
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m   assert(isFirstFile === isFirstFileLinted)
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m     at Object.create (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/test/fixtures/lint_visit_cfg_error/plugin.ts:39:5)
  [38;2;225;80;80;1m│[0m     at lintFileImpl (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:21352:99)
  [38;2;225;80;80;1m│[0m     at lintFile (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:21328:3)
  [38;2;225;80;80;1m│[0m     at lintFileWrapper (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/cli.js:11:41)[0m

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mError running JS plugin.
  [38;2;225;80;80;1m│[0m File path: <fixture>/files/1.js
  [38;2;225;80;80;1m│[0m AssertionError [ERR_ASSERTION]: The expression evaluated to a falsy value:
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m   assert(isFirstFile === isFirstFileLinted)
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m     at Object.create (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/test/fixtures/lint_visit_cfg_error/plugin.ts:39:5)
  [38;2;225;80;80;1m│[0m     at lintFileImpl (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:21352:99)
  [38;2;225;80;80;1m│[0m     at lintFile (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:21328:3)
  [38;2;225;80;80;1m│[0m     at lintFileWrapper (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/cli.js:11:41)[0m

Found 0 warnings and 2 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
