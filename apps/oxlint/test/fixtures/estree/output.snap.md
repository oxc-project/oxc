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
    ,-[files/index.ts:4:1]
  3 |     // All `Identifier`s
  4 | ,-> let a = { x: y };
  5 | |   
  6 | |   // No `ParenthesizedExpression`s in AST
  7 | |   const b = (x * ((('str' + ((123))))));
  8 | |   
  9 | |   // TS syntax
 10 | |   type T = string;
 11 | |   
 12 | |   // No `TSParenthesizedType`s in AST
 13 | `-> type U = (((((string)) | ((number)))));
    `----

  x estree-check(check): program:
  | start/end: [37,243]
  | range: [37,243]
  | loc: [{"start":{"line":4,"column":0},"end":{"line":14,"column":0}}]
    ,-[files/index.ts:4:1]
  3 |     // All `Identifier`s
  4 | ,-> let a = { x: y };
  5 | |   
  6 | |   // No `ParenthesizedExpression`s in AST
  7 | |   const b = (x * ((('str' + ((123))))));
  8 | |   
  9 | |   // TS syntax
 10 | |   type T = string;
 11 | |   
 12 | |   // No `TSParenthesizedType`s in AST
 13 | `-> type U = (((((string)) | ((number)))));
    `----

  x estree-check(check): ident "a":
  | start/end: [41,42]
  | range: [41,42]
  | loc: [{"start":{"line":4,"column":4},"end":{"line":4,"column":5}}]
   ,-[files/index.ts:4:5]
 3 | // All `Identifier`s
 4 | let a = { x: y };
   :     ^
 5 | 
   `----

  x estree-check(check): ident "x":
  | start/end: [47,48]
  | range: [47,48]
  | loc: [{"start":{"line":4,"column":10},"end":{"line":4,"column":11}}]
   ,-[files/index.ts:4:11]
 3 | // All `Identifier`s
 4 | let a = { x: y };
   :           ^
 5 | 
   `----

  x estree-check(check): ident "y":
  | start/end: [50,51]
  | range: [50,51]
  | loc: [{"start":{"line":4,"column":13},"end":{"line":4,"column":14}}]
   ,-[files/index.ts:4:14]
 3 | // All `Identifier`s
 4 | let a = { x: y };
   :              ^
 5 | 
   `----

  x estree-check(check): ident "b":
  | start/end: [102,103]
  | range: [102,103]
  | loc: [{"start":{"line":7,"column":6},"end":{"line":7,"column":7}}]
   ,-[files/index.ts:7:7]
 6 | // No `ParenthesizedExpression`s in AST
 7 | const b = (x * ((('str' + ((123))))));
   :       ^
 8 | 
   `----

  x estree-check(check): ident "x":
  | start/end: [107,108]
  | range: [107,108]
  | loc: [{"start":{"line":7,"column":11},"end":{"line":7,"column":12}}]
   ,-[files/index.ts:7:12]
 6 | // No `ParenthesizedExpression`s in AST
 7 | const b = (x * ((('str' + ((123))))));
   :            ^
 8 | 
   `----

  x estree-check(check): ident "T":
  | start/end: [154,155]
  | range: [154,155]
  | loc: [{"start":{"line":10,"column":5},"end":{"line":10,"column":6}}]
    ,-[files/index.ts:10:6]
  9 | // TS syntax
 10 | type T = string;
    :      ^
 11 | 
    `----

  x estree-check(check): ident "U":
  | start/end: [208,209]
  | range: [208,209]
  | loc: [{"start":{"line":13,"column":5},"end":{"line":13,"column":6}}]
    ,-[files/index.ts:13:6]
 12 | // No `TSParenthesizedType`s in AST
 13 | type U = (((((string)) | ((number)))));
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
