# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): program:
  [38;2;225;80;80;1m│[0m text: "let foo, bar;\n"
  [38;2;225;80;80;1m│[0m getText(): "let foo, bar;\n"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): program:
  [38;2;225;80;80;1m│[0m text: "let foo, bar;\n"
  [38;2;225;80;80;1m│[0m getText(): "let foo, bar;\n"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): var decl:
  [38;2;225;80;80;1m│[0m source: "let foo, bar;"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m─────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): var decl:
  [38;2;225;80;80;1m│[0m source: "let foo, bar;"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m─────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): ident "foo":
  [38;2;225;80;80;1m│[0m source: "foo"
  [38;2;225;80;80;1m│[0m source with before: "t foo"
  [38;2;225;80;80;1m│[0m source with after: "foo,"
  [38;2;225;80;80;1m│[0m source with both: "t foo,"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:5]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m    ───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): ident "foo":
  [38;2;225;80;80;1m│[0m source: "foo"
  [38;2;225;80;80;1m│[0m source with before: "t foo"
  [38;2;225;80;80;1m│[0m source with after: "foo,"
  [38;2;225;80;80;1m│[0m source with both: "t foo,"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:5]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m    ───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): ident "bar":
  [38;2;225;80;80;1m│[0m source: "bar"
  [38;2;225;80;80;1m│[0m source with before: ", bar"
  [38;2;225;80;80;1m│[0m source with after: "bar;"
  [38;2;225;80;80;1m│[0m source with both: ", bar;"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:10]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m         ───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): ident "bar":
  [38;2;225;80;80;1m│[0m source: "bar"
  [38;2;225;80;80;1m│[0m source with before: ", bar"
  [38;2;225;80;80;1m│[0m source with after: "bar;"
  [38;2;225;80;80;1m│[0m source with both: ", bar;"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:10]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m         ───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): after:
  [38;2;225;80;80;1m│[0m source: "let foo, bar;\n"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): program:
  [38;2;225;80;80;1m│[0m text: "let qux;\n"
  [38;2;225;80;80;1m│[0m getText(): "let qux;\n"[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let qux;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): program:
  [38;2;225;80;80;1m│[0m text: "let qux;\n"
  [38;2;225;80;80;1m│[0m getText(): "let qux;\n"[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let qux;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): var decl:
  [38;2;225;80;80;1m│[0m source: "let qux;"[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let qux;
   · [38;2;246;87;248m────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): var decl:
  [38;2;225;80;80;1m│[0m source: "let qux;"[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let qux;
   · [38;2;246;87;248m────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): ident "qux":
  [38;2;225;80;80;1m│[0m source: "qux"
  [38;2;225;80;80;1m│[0m source with before: "t qux"
  [38;2;225;80;80;1m│[0m source with after: "qux;"
  [38;2;225;80;80;1m│[0m source with both: "t qux;"[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let qux;
   · [38;2;246;87;248m    ───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): ident "qux":
  [38;2;225;80;80;1m│[0m source: "qux"
  [38;2;225;80;80;1m│[0m source with before: "t qux"
  [38;2;225;80;80;1m│[0m source with after: "qux;"
  [38;2;225;80;80;1m│[0m source with both: "t qux;"[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let qux;
   · [38;2;246;87;248m    ───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): after:
  [38;2;225;80;80;1m│[0m source: "let qux;\n"[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let qux;
   · [38;2;246;87;248m▲[0m
   ╰────

Found 0 warnings and 16 errors.
Finished in Xms on 2 files with 2 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
