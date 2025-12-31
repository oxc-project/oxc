# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcontext-plugin(log-context):
  [38;2;225;80;80;1m│[0m this === rule: true
  [38;2;225;80;80;1m│[0m id: context-plugin/log-context
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/1.js
  [38;2;225;80;80;1m│[0m getFilename(): <fixture>/files/1.js
  [38;2;225;80;80;1m│[0m physicalFilename: <fixture>/files/1.js
  [38;2;225;80;80;1m│[0m getPhysicalFilename(): <fixture>/files/1.js
  [38;2;225;80;80;1m│[0m cwd: <fixture>
  [38;2;225;80;80;1m│[0m getCwd(): <fixtures>/context_properties[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let x;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcontext-plugin(log-context):
  [38;2;225;80;80;1m│[0m this === undefined: true[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let x;
   · [38;2;246;87;248m──────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcontext-plugin(log-context):
  [38;2;225;80;80;1m│[0m this === rule: true
  [38;2;225;80;80;1m│[0m id: context-plugin/log-context
  [38;2;225;80;80;1m│[0m filename: <fixture>/files/2.js
  [38;2;225;80;80;1m│[0m getFilename(): <fixture>/files/2.js
  [38;2;225;80;80;1m│[0m physicalFilename: <fixture>/files/2.js
  [38;2;225;80;80;1m│[0m getPhysicalFilename(): <fixture>/files/2.js
  [38;2;225;80;80;1m│[0m cwd: <fixture>
  [38;2;225;80;80;1m│[0m getCwd(): <fixtures>/context_properties[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let y;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcontext-plugin(log-context):
  [38;2;225;80;80;1m│[0m this === undefined: true[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let y;
   · [38;2;246;87;248m──────[0m
   ╰────

Found 0 warnings and 4 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
