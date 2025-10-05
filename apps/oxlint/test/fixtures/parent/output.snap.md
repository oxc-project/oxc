# Exit code
1

# stdout
```
  x parents(check): VariableDeclaration:
  | parent: Program
  | ancestors: [ Program ]
   ,-[files/index.js:1:1]
 1 | const obj = { a: [b, c], ...d };
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x parents(check): Program:
  | parent: undefined
  | ancestors: [  ]
   ,-[files/index.js:1:1]
 1 | const obj = { a: [b, c], ...d };
   : ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x parents(check): Identifier:
  | parent: VariableDeclarator
  | ancestors: [ Program, VariableDeclaration, VariableDeclarator ]
   ,-[files/index.js:1:7]
 1 | const obj = { a: [b, c], ...d };
   :       ^^^
   `----

  x parents(check): VariableDeclarator:
  | parent: VariableDeclaration
  | ancestors: [ Program, VariableDeclaration ]
   ,-[files/index.js:1:7]
 1 | const obj = { a: [b, c], ...d };
   :       ^^^^^^^^^^^^^^^^^^^^^^^^^
   `----

  x parents(check): ObjectExpression:
  | parent: VariableDeclarator
  | ancestors: [ Program, VariableDeclaration, VariableDeclarator ]
   ,-[files/index.js:1:13]
 1 | const obj = { a: [b, c], ...d };
   :             ^^^^^^^^^^^^^^^^^^^
   `----

  x parents(check): Identifier:
  | parent: Property
  | ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, Property ]
   ,-[files/index.js:1:15]
 1 | const obj = { a: [b, c], ...d };
   :               ^
   `----

  x parents(check): Property:
  | parent: ObjectExpression
  | ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression ]
   ,-[files/index.js:1:15]
 1 | const obj = { a: [b, c], ...d };
   :               ^^^^^^^^^
   `----

  x parents(check): ArrayExpression:
  | parent: Property
  | ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, Property ]
   ,-[files/index.js:1:18]
 1 | const obj = { a: [b, c], ...d };
   :                  ^^^^^^
   `----

  x parents(check): Identifier:
  | parent: ArrayExpression
  | ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, Property, ArrayExpression ]
   ,-[files/index.js:1:19]
 1 | const obj = { a: [b, c], ...d };
   :                   ^
   `----

  x parents(check): Identifier:
  | parent: ArrayExpression
  | ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, Property, ArrayExpression ]
   ,-[files/index.js:1:22]
 1 | const obj = { a: [b, c], ...d };
   :                      ^
   `----

  x parents(check): SpreadElement:
  | parent: ObjectExpression
  | ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression ]
   ,-[files/index.js:1:26]
 1 | const obj = { a: [b, c], ...d };
   :                          ^^^^
   `----

  x parents(check): Identifier:
  | parent: SpreadElement
  | ancestors: [ Program, VariableDeclaration, VariableDeclarator, ObjectExpression, SpreadElement ]
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
