# Exit code
1

# stdout
```
  [38;2;244;191;117;1m⚠[0m [38;2;244;191;117;1meslint(no-debugger): `debugger` statement is not allowed[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ debugger;
   · [38;2;246;87;248m─────────[0m
 [2m2[0m │ // £
   ╰────
[38;2;106;159;181m  help: [0mRemove the debugger statement

  [38;2;244;191;117;1m⚠[0m [38;2;244;191;117;1meslint(no-debugger): `debugger` statement is not allowed[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:1]
 [2m2[0m │ // £
 [2m3[0m │ debugger;
   · [38;2;246;87;248m─────────[0m
 [2m4[0m │ // 🤨
   ╰────
[38;2;106;159;181m  help: [0mRemove the debugger statement

  [38;2;244;191;117;1m⚠[0m [38;2;244;191;117;1meslint(no-debugger): `debugger` statement is not allowed[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:6:3]
 [2m5[0m │ {
 [2m6[0m │   debugger;
   · [38;2;246;87;248m  ─────────[0m
 [2m7[0m │ }
   ╰────
[38;2;106;159;181m  help: [0mRemove the debugger statement

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mutf16-plugin(no-debugger): program:
  [38;2;225;80;80;1m│[0m start/end: [0,47]
  [38;2;225;80;80;1m│[0m range: [0,47]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":1,"column":0},"end":{"line":8,"column":0}}][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m debugger;
 [2m2[0m │ [38;2;246;87;248m│[0m   // £
 [2m3[0m │ [38;2;246;87;248m│[0m   debugger;
 [2m4[0m │ [38;2;246;87;248m│[0m   // 🤨
 [2m5[0m │ [38;2;246;87;248m│[0m   {
 [2m6[0m │ [38;2;246;87;248m│[0m     debugger;
 [2m7[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m }
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mutf16-plugin(no-debugger): debugger:
  [38;2;225;80;80;1m│[0m start/end: [0,9]
  [38;2;225;80;80;1m│[0m range: [0,9]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":1,"column":0},"end":{"line":1,"column":9}}][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ debugger;
   · [38;2;246;87;248m─────────[0m
 [2m2[0m │ // £
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mutf16-plugin(no-debugger): debugger:
  [38;2;225;80;80;1m│[0m start/end: [15,24]
  [38;2;225;80;80;1m│[0m range: [15,24]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":3,"column":0},"end":{"line":3,"column":9}}][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:3:1]
 [2m2[0m │ // £
 [2m3[0m │ debugger;
   · [38;2;246;87;248m─────────[0m
 [2m4[0m │ // 🤨
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mutf16-plugin(no-debugger): debugger:
  [38;2;225;80;80;1m│[0m start/end: [35,44]
  [38;2;225;80;80;1m│[0m range: [35,44]
  [38;2;225;80;80;1m│[0m loc: [{"start":{"line":6,"column":2},"end":{"line":6,"column":11}}][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:6:3]
 [2m5[0m │ {
 [2m6[0m │   debugger;
   · [38;2;246;87;248m  ─────────[0m
 [2m7[0m │ }
   ╰────

Found 0 warnings and 4 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
