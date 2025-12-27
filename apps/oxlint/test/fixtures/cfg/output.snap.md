# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1merror-plugin(error): Visited nodes:
  [38;2;225;80;80;1m│[0m * onCodePathStart                     Program
  [38;2;225;80;80;1m│[0m * onCodePathSegmentStart              Program
  [38;2;225;80;80;1m│[0m * onCodePathSegmentEnd                Literal
  [38;2;225;80;80;1m│[0m * onCodePathSegmentStart              Literal
  [38;2;225;80;80;1m│[0m * onCodePathSegmentEnd                BlockStatement
  [38;2;225;80;80;1m│[0m * onCodePathSegmentStart              BlockStatement
  [38;2;225;80;80;1m│[0m * onCodePathSegmentLoop               WhileStatement
  [38;2;225;80;80;1m│[0m * onCodePathSegmentEnd                WhileStatement
  [38;2;225;80;80;1m│[0m * onUnreachableCodePathSegmentStart   WhileStatement
  [38;2;225;80;80;1m│[0m * onUnreachableCodePathSegmentEnd     Program
  [38;2;225;80;80;1m│[0m * onCodePathEnd                       Program[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ while (true) {}
   · [38;2;246;87;248m────────────────[0m
   ╰────

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
