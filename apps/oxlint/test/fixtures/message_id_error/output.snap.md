# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mError running JS plugin.
  [38;2;225;80;80;1m│[0m File path: <fixture>/files/index.js
  [38;2;225;80;80;1m│[0m Error: Unknown messageId 'unknownMessage'. Available `messageIds`: 'validMessage'
  [38;2;225;80;80;1m│[0m     at resolveMessageFromMessageId (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:12331:48)
  [38;2;225;80;80;1m│[0m     at getMessage (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:12314:13)
  [38;2;225;80;80;1m│[0m     at report (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:12284:31)
  [38;2;225;80;80;1m│[0m     at Object.report (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:12497:4)
  [38;2;225;80;80;1m│[0m     at DebuggerStatement (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/test/fixtures/message_id_error/plugin.ts:18:21)
  [38;2;225;80;80;1m│[0m     at walkDebuggerStatement (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18719:20)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18221:4)
  [38;2;225;80;80;1m│[0m     at walkNode (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:18218:33)
  [38;2;225;80;80;1m│[0m     at walkProgram (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:19111:2)
  [38;2;225;80;80;1m│[0m     at lintFileImpl (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:21361:230)[0m

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
