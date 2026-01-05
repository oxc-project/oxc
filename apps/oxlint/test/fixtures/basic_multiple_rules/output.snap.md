# Exit code
1

# stdout
```
  [38;2;244;191;117;1m⚠[0m [38;2;244;191;117;1meslint(no-debugger): `debugger` statement is not allowed[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ debugger;
   · [38;2;246;87;248m─────────[0m
 [2m2[0m │
   ╰────
[38;2;106;159;181m  help: [0mRemove the debugger statement

  [38;2;244;191;117;1m⚠[0m [38;2;244;191;117;1meslint(no-unused-expressions): Expected expression to be used[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:1]
 [2m2[0m │
 [2m3[0m │ foo;
   · [38;2;246;87;248m────[0m
   ╰────
[38;2;106;159;181m  help: [0mConsider using this expression or removing it

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mbasic-custom-plugin(no-debugger): Unexpected Debugger Statement[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ debugger;
   · [38;2;246;87;248m─────────[0m
 [2m2[0m │
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mbasic-custom-plugin(no-debugger-2): Unexpected Debugger Statement[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ debugger;
   · [38;2;246;87;248m─────────[0m
 [2m2[0m │
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mbasic-custom-plugin(no-identifiers-named-foo): Unexpected Identifier named foo[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:1]
 [2m2[0m │
 [2m3[0m │ foo;
   · [38;2;246;87;248m───[0m
   ╰────

Found 2 warnings and 3 errors.
Finished in Xms on 1 file with 4 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
