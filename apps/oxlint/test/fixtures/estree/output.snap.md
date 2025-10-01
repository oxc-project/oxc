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

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
