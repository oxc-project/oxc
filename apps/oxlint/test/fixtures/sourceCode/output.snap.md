# Exit code
1

# stdout
```
  x source-code-plugin(create): create:
  | text: "let foo, bar;\n\n// x\n// y\n"
  | getText(): "let foo, bar;\n\n// x\n// y\n"
  | lines: ["let foo, bar;","","// x","// y",""]
  | lineStartIndices: [0,14,15,20,25]
  | locs:
  |   0 => { line: 1, column: 0 }("l")
  |   1 => { line: 1, column: 1 }("e")
  |   2 => { line: 1, column: 2 }("t")
  |   3 => { line: 1, column: 3 }(" ")
  |   4 => { line: 1, column: 4 }("f")
  |   5 => { line: 1, column: 5 }("o")
  |   6 => { line: 1, column: 6 }("o")
  |   7 => { line: 1, column: 7 }(",")
  |   8 => { line: 1, column: 8 }(" ")
  |   9 => { line: 1, column: 9 }("b")
  |   10 => { line: 1, column: 10 }("a")
  |   11 => { line: 1, column: 11 }("r")
  |   12 => { line: 1, column: 12 }(";")
  |   13 => { line: 1, column: 13 }("\n")
  |   14 => { line: 2, column: 0 }("\n")
  |   15 => { line: 3, column: 0 }("/")
  |   16 => { line: 3, column: 1 }("/")
  |   17 => { line: 3, column: 2 }(" ")
  |   18 => { line: 3, column: 3 }("x")
  |   19 => { line: 3, column: 4 }("\n")
  |   20 => { line: 4, column: 0 }("/")
  |   21 => { line: 4, column: 1 }("/")
  |   22 => { line: 4, column: 2 }(" ")
  |   23 => { line: 4, column: 3 }("y")
  |   24 => { line: 4, column: 4 }("\n")
  |   25 => { line: 5, column: 0 }("<EOF>")
  | ast: "foo"
  | visitorKeys: left, right
  | isESTree: true
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^
 2 | 
   `----

  x source-code-plugin(create-once): after:
  | source: "let foo, bar;\n\n// x\n// y\n"
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^
 2 | 
   `----

  x source-code-plugin(create-once): before:
  | text: "let foo, bar;\n\n// x\n// y\n"
  | getText(): "let foo, bar;\n\n// x\n// y\n"
  | lines: ["let foo, bar;","","// x","// y",""]
  | lineStartIndices: [0,14,15,20,25]
  | locs:
  |   0 => { line: 1, column: 0 }("l")
  |   1 => { line: 1, column: 1 }("e")
  |   2 => { line: 1, column: 2 }("t")
  |   3 => { line: 1, column: 3 }(" ")
  |   4 => { line: 1, column: 4 }("f")
  |   5 => { line: 1, column: 5 }("o")
  |   6 => { line: 1, column: 6 }("o")
  |   7 => { line: 1, column: 7 }(",")
  |   8 => { line: 1, column: 8 }(" ")
  |   9 => { line: 1, column: 9 }("b")
  |   10 => { line: 1, column: 10 }("a")
  |   11 => { line: 1, column: 11 }("r")
  |   12 => { line: 1, column: 12 }(";")
  |   13 => { line: 1, column: 13 }("\n")
  |   14 => { line: 2, column: 0 }("\n")
  |   15 => { line: 3, column: 0 }("/")
  |   16 => { line: 3, column: 1 }("/")
  |   17 => { line: 3, column: 2 }(" ")
  |   18 => { line: 3, column: 3 }("x")
  |   19 => { line: 3, column: 4 }("\n")
  |   20 => { line: 4, column: 0 }("/")
  |   21 => { line: 4, column: 1 }("/")
  |   22 => { line: 4, column: 2 }(" ")
  |   23 => { line: 4, column: 3 }("y")
  |   24 => { line: 4, column: 4 }("\n")
  |   25 => { line: 5, column: 0 }("<EOF>")
  | ast: "foo"
  | visitorKeys: left, right
  | isESTree: true
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^
 2 | 
   `----

  x source-code-plugin(create): var decl:
  | source: "let foo, bar;"
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^^^^^^^^^^^^^
 2 | 
   `----

  x source-code-plugin(create-once): var decl:
  | source: "let foo, bar;"
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^^^^^^^^^^^^^
 2 | 
   `----

  x source-code-plugin(create): ident "foo":
  | source: "foo"
  | source with before: "t foo"
  | source with after: "foo,"
  | source with both: "t foo,"
  | start loc: {"line":1,"column":4}
  | end loc: {"line":1,"column":7}
   ,-[files/1.js:1:5]
 1 | let foo, bar;
   :     ^^^
 2 | 
   `----

  x source-code-plugin(create-once): ident "foo":
  | source: "foo"
  | source with before: "t foo"
  | source with after: "foo,"
  | source with both: "t foo,"
  | start loc: {"line":1,"column":4}
  | end loc: {"line":1,"column":7}
   ,-[files/1.js:1:5]
 1 | let foo, bar;
   :     ^^^
 2 | 
   `----

  x source-code-plugin(create): ident "bar":
  | source: "bar"
  | source with before: ", bar"
  | source with after: "bar;"
  | source with both: ", bar;"
  | start loc: {"line":1,"column":9}
  | end loc: {"line":1,"column":12}
   ,-[files/1.js:1:10]
 1 | let foo, bar;
   :          ^^^
 2 | 
   `----

  x source-code-plugin(create-once): ident "bar":
  | source: "bar"
  | source with before: ", bar"
  | source with after: "bar;"
  | source with both: ", bar;"
  | start loc: {"line":1,"column":9}
  | end loc: {"line":1,"column":12}
   ,-[files/1.js:1:10]
 1 | let foo, bar;
   :          ^^^
 2 | 
   `----

  x source-code-plugin(create): create:
  | text: "let qux;\n"
  | getText(): "let qux;\n"
  | lines: ["let qux;",""]
  | lineStartIndices: [0,9]
  | locs:
  |   0 => { line: 1, column: 0 }("l")
  |   1 => { line: 1, column: 1 }("e")
  |   2 => { line: 1, column: 2 }("t")
  |   3 => { line: 1, column: 3 }(" ")
  |   4 => { line: 1, column: 4 }("q")
  |   5 => { line: 1, column: 5 }("u")
  |   6 => { line: 1, column: 6 }("x")
  |   7 => { line: 1, column: 7 }(";")
  |   8 => { line: 1, column: 8 }("\n")
  |   9 => { line: 2, column: 0 }("<EOF>")
  | ast: "qux"
  | visitorKeys: left, right
  | isESTree: true
   ,-[files/2.js:1:1]
 1 | let qux;
   : ^
   `----

  x source-code-plugin(create-once): after:
  | source: "let qux;\n"
   ,-[files/2.js:1:1]
 1 | let qux;
   : ^
   `----

  x source-code-plugin(create-once): before:
  | text: "let qux;\n"
  | getText(): "let qux;\n"
  | lines: ["let qux;",""]
  | lineStartIndices: [0,9]
  | locs:
  |   0 => { line: 1, column: 0 }("l")
  |   1 => { line: 1, column: 1 }("e")
  |   2 => { line: 1, column: 2 }("t")
  |   3 => { line: 1, column: 3 }(" ")
  |   4 => { line: 1, column: 4 }("q")
  |   5 => { line: 1, column: 5 }("u")
  |   6 => { line: 1, column: 6 }("x")
  |   7 => { line: 1, column: 7 }(";")
  |   8 => { line: 1, column: 8 }("\n")
  |   9 => { line: 2, column: 0 }("<EOF>")
  | ast: "qux"
  | visitorKeys: left, right
  | isESTree: true
   ,-[files/2.js:1:1]
 1 | let qux;
   : ^
   `----

  x source-code-plugin(create): var decl:
  | source: "let qux;"
   ,-[files/2.js:1:1]
 1 | let qux;
   : ^^^^^^^^
   `----

  x source-code-plugin(create-once): var decl:
  | source: "let qux;"
   ,-[files/2.js:1:1]
 1 | let qux;
   : ^^^^^^^^
   `----

  x source-code-plugin(create): ident "qux":
  | source: "qux"
  | source with before: "t qux"
  | source with after: "qux;"
  | source with both: "t qux;"
  | start loc: {"line":1,"column":4}
  | end loc: {"line":1,"column":7}
   ,-[files/2.js:1:5]
 1 | let qux;
   :     ^^^
   `----

  x source-code-plugin(create-once): ident "qux":
  | source: "qux"
  | source with before: "t qux"
  | source with after: "qux;"
  | source with both: "t qux;"
  | start loc: {"line":1,"column":4}
  | end loc: {"line":1,"column":7}
   ,-[files/2.js:1:5]
 1 | let qux;
   :     ^^^
   `----

Found 0 warnings and 16 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
