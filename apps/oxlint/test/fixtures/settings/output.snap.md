# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcontext-settings-plugin(log-settings): setting is-nested: "yes"[0m
   ╭─[[38;2;92;157;255;1mfiles/nested/index.js[0m:1:1]
 [2m1[0m │ "nested source file that should be linted with nested config's settings";
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcontext-settings-plugin(log-settings): setting nestedSetting: {"key":"nestedValue","number":42}[0m
   ╭─[[38;2;92;157;255;1mfiles/test.js[0m:1:1]
 [2m1[0m │ console.log("test file");
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcontext-settings-plugin(log-settings): setting arraySetting: ["item1","item2"][0m
   ╭─[[38;2;92;157;255;1mfiles/test.js[0m:1:1]
 [2m1[0m │ console.log("test file");
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mcontext-settings-plugin(log-settings): setting jsdoc: {"ignorePrivate":false}[0m
   ╭─[[38;2;92;157;255;1mfiles/test.js[0m:1:1]
 [2m1[0m │ console.log("test file");
   · [38;2;246;87;248m▲[0m
   ╰────

Found 0 warnings and 4 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
