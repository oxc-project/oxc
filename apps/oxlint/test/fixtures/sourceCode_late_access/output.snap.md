# Exit code
1

# stdout
```
  x source-code-plugin(create): program:
  | text: "let foo, bar;\n"
  | getText(): "let foo, bar;\n"
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^
   `----

  x source-code-plugin(create-once): after:
  | source: "let foo, bar;\n"
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^
   `----

  x source-code-plugin(create-once): program:
  | text: "let foo, bar;\n"
  | getText(): "let foo, bar;\n"
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^
   `----

  x source-code-plugin(create): var decl:
  | source: "let foo, bar;"
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^^^^^^^^^^^^^
   `----

  x source-code-plugin(create-once): var decl:
  | source: "let foo, bar;"
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^^^^^^^^^^^^^
   `----

  x source-code-plugin(create): ident "foo":
  | source: "foo"
  | source with before: "t foo"
  | source with after: "foo,"
  | source with both: "t foo,"
   ,-[files/1.js:1:5]
 1 | let foo, bar;
   :     ^^^
   `----

  x source-code-plugin(create-once): ident "foo":
  | source: "foo"
  | source with before: "t foo"
  | source with after: "foo,"
  | source with both: "t foo,"
   ,-[files/1.js:1:5]
 1 | let foo, bar;
   :     ^^^
   `----

  x source-code-plugin(create): ident "bar":
  | source: "bar"
  | source with before: ", bar"
  | source with after: "bar;"
  | source with both: ", bar;"
   ,-[files/1.js:1:10]
 1 | let foo, bar;
   :          ^^^
   `----

  x source-code-plugin(create-once): ident "bar":
  | source: "bar"
  | source with before: ", bar"
  | source with after: "bar;"
  | source with both: ", bar;"
   ,-[files/1.js:1:10]
 1 | let foo, bar;
   :          ^^^
   `----

  x source-code-plugin(create): program:
  | text: "let qux;\n"
  | getText(): "let qux;\n"
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

  x source-code-plugin(create-once): program:
  | text: "let qux;\n"
  | getText(): "let qux;\n"
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
   ,-[files/2.js:1:5]
 1 | let qux;
   :     ^^^
   `----

  x source-code-plugin(create-once): ident "qux":
  | source: "qux"
  | source with before: "t qux"
  | source with after: "qux;"
  | source with both: "t qux;"
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
