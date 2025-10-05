# Exit code
1

# stdout
```
  x estree-check(check): Visited nodes:
  | * Program
  | * VariableDeclaration: let
  | * VariableDeclarator: (init: ObjectExpression)
  | * Identifier: a
  | * ObjectExpression
  | * Identifier: x
  | * Identifier: y
  | * VariableDeclaration:exit: let
  | * VariableDeclaration: const
  | * VariableDeclarator: (init: BinaryExpression)
  | * Identifier: b
  | * BinaryExpression: * (right: BinaryExpression)
  | * Identifier: x
  | * BinaryExpression: + (right: Literal)
  | * Literal: str
  | * Literal: 123
  | * VariableDeclaration:exit: const
  | * TSTypeAliasDeclaration: (typeAnnotation: TSStringKeyword)
  | * Identifier: T
  | * TSStringKeyword
  | * TSTypeAliasDeclaration:exit: (typeAnnotation: TSStringKeyword)
  | * TSTypeAliasDeclaration: (typeAnnotation: TSUnionType)
  | * Identifier: U
  | * TSUnionType: (types: TSStringKeyword, TSNumberKeyword)
  | * TSStringKeyword
  | * TSNumberKeyword
  | * TSUnionType:exit: (types: TSStringKeyword, TSNumberKeyword)
  | * TSTypeAliasDeclaration:exit: (typeAnnotation: TSUnionType)
  | * Program:exit
    ,-[files/index.ts:5:1]
  4 |     // All `Identifier`s
  5 | ,-> let a = { x: y };
  6 | |   
  7 | |   // No `ParenthesizedExpression`s in AST
  8 | |   const b = (x * ((('str' + ((123))))));
  9 | |   
 10 | |   // TS syntax
 11 | |   type T = string;
 12 | |   
 13 | |   // No `TSParenthesizedType`s in AST
 14 | `-> type U = (((((string)) | ((number)))));
    `----

  x estree-check(check): program:
  | start/end: [59,265]
  | range: [59,265]
  | loc: [{"start":{"line":5,"column":0},"end":{"line":15,"column":0}}]
    ,-[files/index.ts:5:1]
  4 |     // All `Identifier`s
  5 | ,-> let a = { x: y };
  6 | |   
  7 | |   // No `ParenthesizedExpression`s in AST
  8 | |   const b = (x * ((('str' + ((123))))));
  9 | |   
 10 | |   // TS syntax
 11 | |   type T = string;
 12 | |   
 13 | |   // No `TSParenthesizedType`s in AST
 14 | `-> type U = (((((string)) | ((number)))));
    `----

  x estree-check(check): ident "a":
  | start/end: [63,64]
  | range: [63,64]
  | loc: [{"start":{"line":5,"column":4},"end":{"line":5,"column":5}}]
   ,-[files/index.ts:5:5]
 4 | // All `Identifier`s
 5 | let a = { x: y };
   :     ^
 6 | 
   `----

  x estree-check(check): ident "x":
  | start/end: [69,70]
  | range: [69,70]
  | loc: [{"start":{"line":5,"column":10},"end":{"line":5,"column":11}}]
   ,-[files/index.ts:5:11]
 4 | // All `Identifier`s
 5 | let a = { x: y };
   :           ^
 6 | 
   `----

  x estree-check(check): ident "y":
  | start/end: [72,73]
  | range: [72,73]
  | loc: [{"start":{"line":5,"column":13},"end":{"line":5,"column":14}}]
   ,-[files/index.ts:5:14]
 4 | // All `Identifier`s
 5 | let a = { x: y };
   :              ^
 6 | 
   `----

  x estree-check(check): ident "b":
  | start/end: [124,125]
  | range: [124,125]
  | loc: [{"start":{"line":8,"column":6},"end":{"line":8,"column":7}}]
   ,-[files/index.ts:8:7]
 7 | // No `ParenthesizedExpression`s in AST
 8 | const b = (x * ((('str' + ((123))))));
   :       ^
 9 | 
   `----

  x estree-check(check): ident "x":
  | start/end: [129,130]
  | range: [129,130]
  | loc: [{"start":{"line":8,"column":11},"end":{"line":8,"column":12}}]
   ,-[files/index.ts:8:12]
 7 | // No `ParenthesizedExpression`s in AST
 8 | const b = (x * ((('str' + ((123))))));
   :            ^
 9 | 
   `----

  x estree-check(check): ident "T":
  | start/end: [176,177]
  | range: [176,177]
  | loc: [{"start":{"line":11,"column":5},"end":{"line":11,"column":6}}]
    ,-[files/index.ts:11:6]
 10 | // TS syntax
 11 | type T = string;
    :      ^
 12 | 
    `----

  x estree-check(check): ident "U":
  | start/end: [230,231]
  | range: [230,231]
  | loc: [{"start":{"line":14,"column":5},"end":{"line":14,"column":6}}]
    ,-[files/index.ts:14:6]
 13 | // No `TSParenthesizedType`s in AST
 14 | type U = (((((string)) | ((number)))));
    :      ^
    `----

Found 0 warnings and 9 errors.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
