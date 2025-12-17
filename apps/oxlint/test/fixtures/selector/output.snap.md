# Exit code
1

# stdout
```
  x selectors(check):
  | *: Program
  | :not(Identifier): Program
  | *: VariableDeclaration
  | :not(Identifier): VariableDeclaration
  | *: VariableDeclarator
  | :not(Identifier): VariableDeclarator
  | *: Identifier(obj)
  | Identifier: Identifier(obj)
  | :matches(Identifier, FunctionDeclaration): Identifier(obj)
  | *:exit: Identifier(obj)
  | *: ObjectExpression
  | :not(Identifier): ObjectExpression
  | *: Property
  | :not(Identifier): Property
  | *: Identifier(a)
  | Identifier: Identifier(a)
  | :matches(Identifier, FunctionDeclaration): Identifier(a)
  | ObjectExpression Identifier: Identifier(a)
  | Property > Identifier: Identifier(a)
  | ObjectExpression > Property > Identifier: Identifier(a)
  | Identifier[name=a]: Identifier(a)
  | ObjectExpression > Property > Identifier[name=a]: Identifier(a)
  | :matches(Identifier[name=a], FunctionDeclaration[id.name=foo]): Identifier(a)
  | *:exit: Identifier(a)
  | *: ArrayExpression
  | :not(Identifier): ArrayExpression
  | ObjectExpression ArrayExpression: ArrayExpression
  | *: Identifier(b)
  | Identifier: Identifier(b)
  | :matches(Identifier, FunctionDeclaration): Identifier(b)
  | ArrayExpression > Identifier: Identifier(b)
  | ArrayExpression Identifier: Identifier(b)
  | ObjectExpression Identifier: Identifier(b)
  | Program > VariableDeclaration > VariableDeclarator > ObjectExpression > Property > ArrayExpression > Identifier: Identifier(b)
  | ArrayExpression Identifier[name=b]: Identifier(b)
  | *:exit: Identifier(b)
  | *: Identifier(c)
  | Identifier: Identifier(c)
  | :matches(Identifier, FunctionDeclaration): Identifier(c)
  | ArrayExpression > Identifier: Identifier(c)
  | ArrayExpression Identifier: Identifier(c)
  | Identifier ~ Identifier: Identifier(c)
  | ObjectExpression Identifier: Identifier(c)
  | Program > VariableDeclaration > VariableDeclarator > ObjectExpression > Property > ArrayExpression > Identifier: Identifier(c)
  | ArrayExpression > Identifier[name=c]: Identifier(c)
  | *:exit: Identifier(c)
  | *:exit: ArrayExpression
  | *:exit: Property
  | *: SpreadElement
  | :not(Identifier): SpreadElement
  | :matches(ObjectExpression > SpreadElement, FunctionDeclaration): SpreadElement
  | Property ~ [type]: SpreadElement
  | :matches(ObjectExpression > SpreadElement, FunctionDeclaration[id.name=bar]): SpreadElement
  | *: Identifier(d)
  | Identifier: Identifier(d)
  | :matches(Identifier, FunctionDeclaration): Identifier(d)
  | ObjectExpression Identifier: Identifier(d)
  | Identifier[name=d]: Identifier(d)
  | *:exit: Identifier(d)
  | *:exit: SpreadElement
  | *:exit: ObjectExpression
  | *:exit: VariableDeclarator
  | *:exit: VariableDeclaration
  | *: FunctionDeclaration(foo)
  | :function: FunctionDeclaration(foo)
  | :not(Identifier): FunctionDeclaration(foo)
  | :matches(Identifier, FunctionDeclaration): FunctionDeclaration(foo)
  | Program > FunctionDeclaration: FunctionDeclaration(foo)
  | VariableDeclaration + FunctionDeclaration: FunctionDeclaration(foo)
  | VariableDeclaration ~ FunctionDeclaration: FunctionDeclaration(foo)
  | :matches(ObjectExpression > SpreadElement, FunctionDeclaration): FunctionDeclaration(foo)
  | :matches(Identifier[name=a], FunctionDeclaration[id.name=foo]): FunctionDeclaration(foo)
  | *: Identifier(foo)
  | Identifier: Identifier(foo)
  | :matches(Identifier, FunctionDeclaration): Identifier(foo)
  | *:exit: Identifier(foo)
  | *: BlockStatement
  | :not(Identifier): BlockStatement
  | *:exit: BlockStatement
  | *:exit: FunctionDeclaration(foo)
  | *: FunctionDeclaration(bar)
  | :function: FunctionDeclaration(bar)
  | :not(Identifier): FunctionDeclaration(bar)
  | :matches(Identifier, FunctionDeclaration): FunctionDeclaration(bar)
  | Program > FunctionDeclaration: FunctionDeclaration(bar)
  | VariableDeclaration ~ FunctionDeclaration: FunctionDeclaration(bar)
  | :matches(ObjectExpression > SpreadElement, FunctionDeclaration): FunctionDeclaration(bar)
  | :matches(ObjectExpression > SpreadElement, FunctionDeclaration[id.name=bar]): FunctionDeclaration(bar)
  | *: Identifier(bar)
  | Identifier: Identifier(bar)
  | :matches(Identifier, FunctionDeclaration): Identifier(bar)
  | *:exit: Identifier(bar)
  | *: BlockStatement
  | :not(Identifier): BlockStatement
  | *:exit: BlockStatement
  | *:exit: FunctionDeclaration(bar)
  | *: ExpressionStatement
  | :not(Identifier): ExpressionStatement
  | *: ArrowFunctionExpression
  | :function: ArrowFunctionExpression
  | :not(Identifier): ArrowFunctionExpression
  | *: BlockStatement
  | :not(Identifier): BlockStatement
  | *:exit: BlockStatement
  | *:exit: ArrowFunctionExpression
  | *:exit: ExpressionStatement
  | *:exit: Program
   ,-[files/index.js:1:1]
 1 | ,-> const obj = { a: [b, c], ...d };
 2 | |   
 3 | |   function foo() {}
 4 | |   function bar() {}
 5 | |   
 6 | `-> () => {};
   `----

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
