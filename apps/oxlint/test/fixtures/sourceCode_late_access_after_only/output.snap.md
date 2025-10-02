# Exit code
1

# stdout
```
  x source-code-plugin(create-once): after:
  | text: "let foo, bar;\n"
  | getText(): "let foo, bar;\n"
  | ast: "foo"
   ,-[files/1.js:1:1]
 1 | let foo, bar;
   : ^
   `----

  x source-code-plugin(create-once): after:
  | text: "let qux;\n"
  | getText(): "let qux;\n"
  | ast: "qux"
   ,-[files/2.js:1:1]
 1 | let qux;
   : ^
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 2 files using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
