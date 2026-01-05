# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): create:
  [38;2;225;80;80;1m│[0m text: "let foo, bar;\n\n// x\n// y\n"
  [38;2;225;80;80;1m│[0m getText(): "let foo, bar;\n\n// x\n// y\n"
  [38;2;225;80;80;1m│[0m lines: ["let foo, bar;","","// x","// y",""]
  [38;2;225;80;80;1m│[0m lineStartIndices: [0,14,15,20,25]
  [38;2;225;80;80;1m│[0m locs:
  [38;2;225;80;80;1m│[0m   0 => { line: 1, column: 0 }("l")
  [38;2;225;80;80;1m│[0m   1 => { line: 1, column: 1 }("e")
  [38;2;225;80;80;1m│[0m   2 => { line: 1, column: 2 }("t")
  [38;2;225;80;80;1m│[0m   3 => { line: 1, column: 3 }(" ")
  [38;2;225;80;80;1m│[0m   4 => { line: 1, column: 4 }("f")
  [38;2;225;80;80;1m│[0m   5 => { line: 1, column: 5 }("o")
  [38;2;225;80;80;1m│[0m   6 => { line: 1, column: 6 }("o")
  [38;2;225;80;80;1m│[0m   7 => { line: 1, column: 7 }(",")
  [38;2;225;80;80;1m│[0m   8 => { line: 1, column: 8 }(" ")
  [38;2;225;80;80;1m│[0m   9 => { line: 1, column: 9 }("b")
  [38;2;225;80;80;1m│[0m   10 => { line: 1, column: 10 }("a")
  [38;2;225;80;80;1m│[0m   11 => { line: 1, column: 11 }("r")
  [38;2;225;80;80;1m│[0m   12 => { line: 1, column: 12 }(";")
  [38;2;225;80;80;1m│[0m   13 => { line: 1, column: 13 }("\n")
  [38;2;225;80;80;1m│[0m   14 => { line: 2, column: 0 }("\n")
  [38;2;225;80;80;1m│[0m   15 => { line: 3, column: 0 }("/")
  [38;2;225;80;80;1m│[0m   16 => { line: 3, column: 1 }("/")
  [38;2;225;80;80;1m│[0m   17 => { line: 3, column: 2 }(" ")
  [38;2;225;80;80;1m│[0m   18 => { line: 3, column: 3 }("x")
  [38;2;225;80;80;1m│[0m   19 => { line: 3, column: 4 }("\n")
  [38;2;225;80;80;1m│[0m   20 => { line: 4, column: 0 }("/")
  [38;2;225;80;80;1m│[0m   21 => { line: 4, column: 1 }("/")
  [38;2;225;80;80;1m│[0m   22 => { line: 4, column: 2 }(" ")
  [38;2;225;80;80;1m│[0m   23 => { line: 4, column: 3 }("y")
  [38;2;225;80;80;1m│[0m   24 => { line: 4, column: 4 }("\n")
  [38;2;225;80;80;1m│[0m   25 => { line: 5, column: 0 }("<EOF>")
  [38;2;225;80;80;1m│[0m ast: "foo"
  [38;2;225;80;80;1m│[0m visitorKeys: left, right
  [38;2;225;80;80;1m│[0m isESTree: true[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m▲[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): before:
  [38;2;225;80;80;1m│[0m text: "let foo, bar;\n\n// x\n// y\n"
  [38;2;225;80;80;1m│[0m getText(): "let foo, bar;\n\n// x\n// y\n"
  [38;2;225;80;80;1m│[0m lines: ["let foo, bar;","","// x","// y",""]
  [38;2;225;80;80;1m│[0m lineStartIndices: [0,14,15,20,25]
  [38;2;225;80;80;1m│[0m locs:
  [38;2;225;80;80;1m│[0m   0 => { line: 1, column: 0 }("l")
  [38;2;225;80;80;1m│[0m   1 => { line: 1, column: 1 }("e")
  [38;2;225;80;80;1m│[0m   2 => { line: 1, column: 2 }("t")
  [38;2;225;80;80;1m│[0m   3 => { line: 1, column: 3 }(" ")
  [38;2;225;80;80;1m│[0m   4 => { line: 1, column: 4 }("f")
  [38;2;225;80;80;1m│[0m   5 => { line: 1, column: 5 }("o")
  [38;2;225;80;80;1m│[0m   6 => { line: 1, column: 6 }("o")
  [38;2;225;80;80;1m│[0m   7 => { line: 1, column: 7 }(",")
  [38;2;225;80;80;1m│[0m   8 => { line: 1, column: 8 }(" ")
  [38;2;225;80;80;1m│[0m   9 => { line: 1, column: 9 }("b")
  [38;2;225;80;80;1m│[0m   10 => { line: 1, column: 10 }("a")
  [38;2;225;80;80;1m│[0m   11 => { line: 1, column: 11 }("r")
  [38;2;225;80;80;1m│[0m   12 => { line: 1, column: 12 }(";")
  [38;2;225;80;80;1m│[0m   13 => { line: 1, column: 13 }("\n")
  [38;2;225;80;80;1m│[0m   14 => { line: 2, column: 0 }("\n")
  [38;2;225;80;80;1m│[0m   15 => { line: 3, column: 0 }("/")
  [38;2;225;80;80;1m│[0m   16 => { line: 3, column: 1 }("/")
  [38;2;225;80;80;1m│[0m   17 => { line: 3, column: 2 }(" ")
  [38;2;225;80;80;1m│[0m   18 => { line: 3, column: 3 }("x")
  [38;2;225;80;80;1m│[0m   19 => { line: 3, column: 4 }("\n")
  [38;2;225;80;80;1m│[0m   20 => { line: 4, column: 0 }("/")
  [38;2;225;80;80;1m│[0m   21 => { line: 4, column: 1 }("/")
  [38;2;225;80;80;1m│[0m   22 => { line: 4, column: 2 }(" ")
  [38;2;225;80;80;1m│[0m   23 => { line: 4, column: 3 }("y")
  [38;2;225;80;80;1m│[0m   24 => { line: 4, column: 4 }("\n")
  [38;2;225;80;80;1m│[0m   25 => { line: 5, column: 0 }("<EOF>")
  [38;2;225;80;80;1m│[0m ast: "foo"
  [38;2;225;80;80;1m│[0m visitorKeys: left, right
  [38;2;225;80;80;1m│[0m isESTree: true[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m▲[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): var decl:
  [38;2;225;80;80;1m│[0m source: "let foo, bar;"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m─────────────[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): var decl:
  [38;2;225;80;80;1m│[0m source: "let foo, bar;"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m─────────────[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): ident "foo":
  [38;2;225;80;80;1m│[0m source: "foo"
  [38;2;225;80;80;1m│[0m source with before: "t foo"
  [38;2;225;80;80;1m│[0m source with after: "foo,"
  [38;2;225;80;80;1m│[0m source with both: "t foo,"
  [38;2;225;80;80;1m│[0m range: [4,7]
  [38;2;225;80;80;1m│[0m loc: {"start":{"line":1,"column":4},"end":{"line":1,"column":7}}
  [38;2;225;80;80;1m│[0m start loc: {"line":1,"column":4}
  [38;2;225;80;80;1m│[0m end loc: {"line":1,"column":7}[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:5]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m    ───[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): ident "foo":
  [38;2;225;80;80;1m│[0m source: "foo"
  [38;2;225;80;80;1m│[0m source with before: "t foo"
  [38;2;225;80;80;1m│[0m source with after: "foo,"
  [38;2;225;80;80;1m│[0m source with both: "t foo,"
  [38;2;225;80;80;1m│[0m start loc: {"line":1,"column":4}
  [38;2;225;80;80;1m│[0m end loc: {"line":1,"column":7}[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:5]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m    ───[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): ident "bar":
  [38;2;225;80;80;1m│[0m source: "bar"
  [38;2;225;80;80;1m│[0m source with before: ", bar"
  [38;2;225;80;80;1m│[0m source with after: "bar;"
  [38;2;225;80;80;1m│[0m source with both: ", bar;"
  [38;2;225;80;80;1m│[0m range: [9,12]
  [38;2;225;80;80;1m│[0m loc: {"start":{"line":1,"column":9},"end":{"line":1,"column":12}}
  [38;2;225;80;80;1m│[0m start loc: {"line":1,"column":9}
  [38;2;225;80;80;1m│[0m end loc: {"line":1,"column":12}[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:10]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m         ───[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): ident "bar":
  [38;2;225;80;80;1m│[0m source: "bar"
  [38;2;225;80;80;1m│[0m source with before: ", bar"
  [38;2;225;80;80;1m│[0m source with after: "bar;"
  [38;2;225;80;80;1m│[0m source with both: ", bar;"
  [38;2;225;80;80;1m│[0m start loc: {"line":1,"column":9}
  [38;2;225;80;80;1m│[0m end loc: {"line":1,"column":12}[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:10]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m         ───[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): after:
  [38;2;225;80;80;1m│[0m source: "let foo, bar;\n\n// x\n// y\n"[0m
   ╭─[[38;2;92;157;255;1mfiles/1.js[0m:1:1]
 [2m1[0m │ let foo, bar;
   · [38;2;246;87;248m▲[0m
 [2m2[0m │ 
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create): create:
  [38;2;225;80;80;1m│[0m text: "let qux;\n"
  [38;2;225;80;80;1m│[0m getText(): "let qux;\n"
  [38;2;225;80;80;1m│[0m lines: ["let qux;",""]
  [38;2;225;80;80;1m│[0m lineStartIndices: [0,9]
  [38;2;225;80;80;1m│[0m locs:
  [38;2;225;80;80;1m│[0m   0 => { line: 1, column: 0 }("l")
  [38;2;225;80;80;1m│[0m   1 => { line: 1, column: 1 }("e")
  [38;2;225;80;80;1m│[0m   2 => { line: 1, column: 2 }("t")
  [38;2;225;80;80;1m│[0m   3 => { line: 1, column: 3 }(" ")
  [38;2;225;80;80;1m│[0m   4 => { line: 1, column: 4 }("q")
  [38;2;225;80;80;1m│[0m   5 => { line: 1, column: 5 }("u")
  [38;2;225;80;80;1m│[0m   6 => { line: 1, column: 6 }("x")
  [38;2;225;80;80;1m│[0m   7 => { line: 1, column: 7 }(";")
  [38;2;225;80;80;1m│[0m   8 => { line: 1, column: 8 }("\n")
  [38;2;225;80;80;1m│[0m   9 => { line: 2, column: 0 }("<EOF>")
  [38;2;225;80;80;1m│[0m ast: "qux"
  [38;2;225;80;80;1m│[0m visitorKeys: left, right
  [38;2;225;80;80;1m│[0m isESTree: true[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:1]
 [2m1[0m │ let qux;
   · [38;2;246;87;248m▲[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): before:
  [38;2;225;80;80;1m│[0m text: "let qux;\n"
  [38;2;225;80;80;1m│[0m getText(): "let qux;\n"
  [38;2;225;80;80;1m│[0m lines: ["let qux;",""]
  [38;2;225;80;80;1m│[0m lineStartIndices: [0,9]
  [38;2;225;80;80;1m│[0m locs:
  [38;2;225;80;80;1m│[0m   0 => { line: 1, column: 0 }("l")
  [38;2;225;80;80;1m│[0m   1 => { line: 1, column: 1 }("e")
  [38;2;225;80;80;1m│[0m   2 => { line: 1, column: 2 }("t")
  [38;2;225;80;80;1m│[0m   3 => { line: 1, column: 3 }(" ")
  [38;2;225;80;80;1m│[0m   4 => { line: 1, column: 4 }("q")
  [38;2;225;80;80;1m│[0m   5 => { line: 1, column: 5 }("u")
  [38;2;225;80;80;1m│[0m   6 => { line: 1, column: 6 }("x")
  [38;2;225;80;80;1m│[0m   7 => { line: 1, column: 7 }(";")
  [38;2;225;80;80;1m│[0m   8 => { line: 1, column: 8 }("\n")
  [38;2;225;80;80;1m│[0m   9 => { line: 2, column: 0 }("<EOF>")
  [38;2;225;80;80;1m│[0m ast: "qux"
  [38;2;225;80;80;1m│[0m visitorKeys: left, right
  [38;2;225;80;80;1m│[0m isESTree: true[0m
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
  [38;2;225;80;80;1m│[0m source with both: "t qux;"
  [38;2;225;80;80;1m│[0m range: [4,7]
  [38;2;225;80;80;1m│[0m loc: {"start":{"line":1,"column":4},"end":{"line":1,"column":7}}
  [38;2;225;80;80;1m│[0m start loc: {"line":1,"column":4}
  [38;2;225;80;80;1m│[0m end loc: {"line":1,"column":7}[0m
   ╭─[[38;2;92;157;255;1mfiles/2.js[0m:1:5]
 [2m1[0m │ let qux;
   · [38;2;246;87;248m    ───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1msource-code-plugin(create-once): ident "qux":
  [38;2;225;80;80;1m│[0m source: "qux"
  [38;2;225;80;80;1m│[0m source with before: "t qux"
  [38;2;225;80;80;1m│[0m source with after: "qux;"
  [38;2;225;80;80;1m│[0m source with both: "t qux;"
  [38;2;225;80;80;1m│[0m start loc: {"line":1,"column":4}
  [38;2;225;80;80;1m│[0m end loc: {"line":1,"column":7}[0m
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
