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
  7 | |   // prettier-ignore
  8 | |   const b = (x * ((('str' + ((123))))));
  9 | |   
 10 | |   // TS syntax
 11 | |   type T = string;
 12 | |   
 13 | |   // No `TSParenthesizedType`s in AST
 14 | |   // prettier-ignore
 15 | `-> type U = (((((string)) | ((number)))));
    `----

  x estree-check(check): program:
  | start/end: [37,281]
  | range: [37,281]
  | loc: [{"start":{"line":4,"column":0},"end":{"line":16,"column":0}}]
    ,-[files/index.ts:4:1]
  3 |     // All `Identifier`s
  4 | ,-> let a = { x: y };
  5 | |   
  6 | |   // No `ParenthesizedExpression`s in AST
  7 | |   // prettier-ignore
  8 | |   const b = (x * ((('str' + ((123))))));
  9 | |   
 10 | |   // TS syntax
 11 | |   type T = string;
 12 | |   
 13 | |   // No `TSParenthesizedType`s in AST
 14 | |   // prettier-ignore
 15 | `-> type U = (((((string)) | ((number)))));
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
  | start/end: [121,122]
  | range: [121,122]
  | loc: [{"start":{"line":8,"column":6},"end":{"line":8,"column":7}}]
   ,-[files/index.ts:8:7]
 7 | // prettier-ignore
 8 | const b = (x * ((('str' + ((123))))));
   :       ^
 9 | 
   `----

  x estree-check(check): ident "x":
  | start/end: [126,127]
  | range: [126,127]
  | loc: [{"start":{"line":8,"column":11},"end":{"line":8,"column":12}}]
   ,-[files/index.ts:8:12]
 7 | // prettier-ignore
 8 | const b = (x * ((('str' + ((123))))));
   :            ^
 9 | 
   `----

  x estree-check(check): ident "T":
  | start/end: [173,174]
  | range: [173,174]
  | loc: [{"start":{"line":11,"column":5},"end":{"line":11,"column":6}}]
    ,-[files/index.ts:11:6]
 10 | // TS syntax
 11 | type T = string;
    :      ^
 12 | 
    `----

  x estree-check(check): ident "U":
  | start/end: [246,247]
  | range: [246,247]
  | loc: [{"start":{"line":15,"column":5},"end":{"line":15,"column":6}}]
    ,-[files/index.ts:15:6]
 14 | // prettier-ignore
 15 | type U = (((((string)) | ((number)))));
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
