# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1minterpolation-test(no-var): Variables should not use var[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ var testWithNoData = {};
   · [38;2;246;87;248m────────────────────────[0m
 [2m2[0m │ var testWithName = {};
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1minterpolation-test(no-var): Variable `testWithName` should not use var[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:2:1]
 [2m1[0m │ var testWithNoData = {};
 [2m2[0m │ var testWithName = {};
   · [38;2;246;87;248m──────────────────────[0m
 [2m3[0m │ var testWithNameNoData = {};
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1minterpolation-test(no-var): Variable `{{name}}` should not use var[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:1]
 [2m2[0m │ var testWithName = {};
 [2m3[0m │ var testWithNameNoData = {};
   · [38;2;246;87;248m────────────────────────────[0m
 [2m4[0m │ var testWithMultiple = {};
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1minterpolation-test(no-var): Variable `testWithMultiple` of type `string` should not use var[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:4:1]
 [2m3[0m │ var testWithNameNoData = {};
 [2m4[0m │ var testWithMultiple = {};
   · [38;2;246;87;248m──────────────────────────[0m
 [2m5[0m │ var testWithMultipleNoData = {};
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1minterpolation-test(no-var): Variable `{{name}}` of type `{{type}}` should not use var[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:5:1]
 [2m4[0m │ var testWithMultiple = {};
 [2m5[0m │ var testWithMultipleNoData = {};
   · [38;2;246;87;248m────────────────────────────────[0m
 [2m6[0m │ var testWithMissingData = {};
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1minterpolation-test(no-var): Value is `example` and name is `{{name}}`[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:6:1]
 [2m5[0m │ var testWithMultipleNoData = {};
 [2m6[0m │ var testWithMissingData = {};
   · [38;2;246;87;248m─────────────────────────────[0m
 [2m7[0m │ var testWithSpaces = {};
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1minterpolation-test(no-var): Value with spaces is `hello` and name is `world`[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:7:1]
 [2m6[0m │ var testWithMissingData = {};
 [2m7[0m │ var testWithSpaces = {};
   · [38;2;246;87;248m────────────────────────[0m
   ╰────

Found 0 warnings and 7 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
