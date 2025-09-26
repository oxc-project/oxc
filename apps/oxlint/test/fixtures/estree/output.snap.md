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

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
