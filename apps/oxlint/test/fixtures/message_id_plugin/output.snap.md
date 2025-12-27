# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mmessage-id-plugin(no-var): Unexpected var, use let or const instead.[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ var reportUsingNode = 1;
   · [38;2;246;87;248m────────────────────────[0m
 [2m2[0m │ var reportUsingRange = 1;
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mmessage-id-plugin(no-var): Unexpected var, use let or const instead.[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:2:1]
 [2m1[0m │ var reportUsingNode = 1;
 [2m2[0m │ var reportUsingRange = 1;
   · [38;2;246;87;248m─────────────────────────[0m
   ╰────

Found 0 warnings and 2 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
