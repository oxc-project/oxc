# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mError running JS plugin.
  [38;2;225;80;80;1m│[0m File path: <fixture>/files/index.js
  [38;2;225;80;80;1m│[0m Error: Whoops!
  [38;2;225;80;80;1m│[0m     at Object.fix (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/test/fixtures/lint_fix_error/plugin.ts:16:23)
  [38;2;225;80;80;1m│[0m     at getFixes (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:12251:18)
  [38;2;225;80;80;1m│[0m     at report (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:12307:10)
  [38;2;225;80;80;1m│[0m     at Object.report (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:12497:4)
  [38;2;225;80;80;1m│[0m     at Identifier (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/test/fixtures/lint_fix_error/plugin.ts:12:21)
  [38;2;225;80;80;1m│[0m     at walkIdentifier (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:19007:70)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18392:4)
  [38;2;225;80;80;1m│[0m     at walkVariableDeclarator (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:19219:2)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18497:4)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18218:33)[0m

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
