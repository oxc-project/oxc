# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1meslint(no-debugger): `debugger` statement is not allowed[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:10:1]
 [2m 9[0m │ // should trigger an error
 [2m10[0m │ debugger;
    · [38;2;246;87;248m─────────[0m
 [2m11[0m │ 
    ╰────
[38;2;106;159;181m  help: [0mRemove the debugger statement

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(no-var): Use let or const instead of var[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ var shouldError = 1;
   · [38;2;246;87;248m────────────────────[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(no-var): Use let or const instead of var[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:16:1]
 [2m15[0m │ /* oxlint-disable-next-line test-plugin */ // `test-plugin` should be `test-plugin/no-var`
 [2m16[0m │ var incorrectlyDisabled = 4;
    · [38;2;246;87;248m────────────────────────────[0m
 [2m17[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(no-var): Use let or const instead of var[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:19:1]
 [2m18[0m │ /* oxlint-disable-next-line no-var */ // `no-var` should be `test-plugin/no-var`
 [2m19[0m │ var anotherIncorrectlyDisabled = 4;
    · [38;2;246;87;248m───────────────────────────────────[0m
 [2m20[0m │ 
    ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mtest-plugin(no-var): Use let or const instead of var[0m
    ╭─[[38;2;92;157;255;1mfiles/index.js[0m:22:1]
 [2m21[0m │ // This var should trigger an error again
 [2m22[0m │ var shouldErrorAgain = 3;
    · [38;2;246;87;248m─────────────────────────[0m
    ╰────

Found 0 warnings and 5 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
