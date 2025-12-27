# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): Program:
  [38;2;225;80;80;1m│[0m parent: undefined
  [38;2;225;80;80;1m│[0m ancestors: [  ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m─────────────────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): VariableDeclaration:
  [38;2;225;80;80;1m│[0m parent: Program
  [38;2;225;80;80;1m│[0m ancestors: [ Program ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m────────────────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): VariableDeclarator:
  [38;2;225;80;80;1m│[0m parent: VariableDeclaration
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:7]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m      ─────────────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): Identifier:
  [38;2;225;80;80;1m│[0m parent: VariableDeclarator
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration, VariableDeclarator ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:7]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m      ───[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): ObjectExpression:
  [38;2;225;80;80;1m│[0m parent: VariableDeclarator
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration, VariableDeclarator ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:13]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m            ───────────────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): Property:
  [38;2;225;80;80;1m│[0m parent: ObjectExpression
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:15]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m              ─────────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): Identifier:
  [38;2;225;80;80;1m│[0m parent: Property
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, Property ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:15]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m              ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): ArrayExpression:
  [38;2;225;80;80;1m│[0m parent: Property
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, Property ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:18]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m                 ──────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): Identifier:
  [38;2;225;80;80;1m│[0m parent: ArrayExpression
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, Property, ArrayExpression ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:19]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m                  ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): Identifier:
  [38;2;225;80;80;1m│[0m parent: ArrayExpression
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, Property, ArrayExpression ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:22]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m                     ─[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): SpreadElement:
  [38;2;225;80;80;1m│[0m parent: ObjectExpression
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:26]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m                         ────[0m
   ╰────

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mparents(check): Identifier:
  [38;2;225;80;80;1m│[0m parent: SpreadElement
  [38;2;225;80;80;1m│[0m ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, SpreadElement ][0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:29]
 [2m1[0m │ const obj = { a: [b, c], ...d };
   · [38;2;246;87;248m                            ─[0m
   ╰────

Found 0 warnings and 12 errors.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
