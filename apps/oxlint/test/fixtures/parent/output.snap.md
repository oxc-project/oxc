# Exit code
1

# stdout
```
  x parents(check): VariableDeclaration -> Program
   ,-[files/index.js:1:1]
 1 | const obj = { a: [b, c], ...d };
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x parents(check): Program -> null
   ,-[files/index.js:1:1]
 1 | const obj = { a: [b, c], ...d };
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x parents(check): Identifier -> VariableDeclarator
   ,-[files/index.js:1:7]
 1 | const obj = { a: [b, c], ...d };
   :       ^^^
   `----

  x parents(check): VariableDeclarator -> VariableDeclaration
   ,-[files/index.js:1:7]
 1 | const obj = { a: [b, c], ...d };
   :       ^^^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x parents(check): ObjectExpression -> VariableDeclarator
   ,-[files/index.js:1:13]
 1 | const obj = { a: [b, c], ...d };
   :             ^^^^^^^^^^^^^^^^^^^
   `----

  x parents(check): Identifier -> Property
   ,-[files/index.js:1:15]
 1 | const obj = { a: [b, c], ...d };
   :               ^
   `----

  x parents(check): Property -> ObjectExpression
   ,-[files/index.js:1:15]
 1 | const obj = { a: [b, c], ...d };
   :               ^^^^^^^^^
   `----

  x parents(check): ArrayExpression -> Property
   ,-[files/index.js:1:18]
 1 | const obj = { a: [b, c], ...d };
   :                  ^^^^^^
   `----

  x parents(check): Identifier -> ArrayExpression
   ,-[files/index.js:1:19]
 1 | const obj = { a: [b, c], ...d };
   :                   ^
   `----

  x parents(check): Identifier -> ArrayExpression
   ,-[files/index.js:1:22]
 1 | const obj = { a: [b, c], ...d };
   :                      ^
   `----

  x parents(check): SpreadElement -> ObjectExpression
   ,-[files/index.js:1:26]
 1 | const obj = { a: [b, c], ...d };
   :                          ^^^^
   `----

  x parents(check): Identifier -> SpreadElement
   ,-[files/index.js:1:29]
 1 | const obj = { a: [b, c], ...d };
   :                             ^
   `----

Found 0 warnings and 12 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
