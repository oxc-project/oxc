# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mError running JS plugin.
  [38;2;225;80;80;1m│[0m File path: <fixture>/files/1.js
  [38;2;225;80;80;1m│[0m Error: Identifier in 1st file: x
  [38;2;225;80;80;1m│[0m     at Identifier (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/test/fixtures/lint_visit_error/plugin.ts:41:32)
  [38;2;225;80;80;1m│[0m     at walkIdentifier (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:19007:70)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18392:4)
  [38;2;225;80;80;1m│[0m     at walkVariableDeclarator (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:19219:2)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18497:4)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18218:33)
  [38;2;225;80;80;1m│[0m     at walkVariableDeclaration (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:19213:2)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18494:4)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18218:33)
  [38;2;225;80;80;1m│[0m     at walkProgram (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:19111:2)[0m

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1merror-plugin(error): Identifier in 2nd file: y[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let y;
   · [38;2;246;87;248m    ─[0m
   ╰────

Found 0 warnings and 2 errors.
Finished in Xms on 2 files with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
