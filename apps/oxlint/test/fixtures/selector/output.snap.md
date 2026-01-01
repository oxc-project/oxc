# Exit code
1

# stdout
```
  x selectors(check):
  | *: Program
  | :not(Identifier): Program
  | *: VariableDeclaration
  | :Declaration: VariableDeclaration
  | :statement: VariableDeclaration
  | :not(Identifier): VariableDeclaration
  | *: VariableDeclarator
  | :not(Identifier): VariableDeclarator
  | *: Identifier(obj)
  | :expression: Identifier(obj)
  | :paTTern: Identifier(obj)
  | Identifier: Identifier(obj)
  | :matches(Identifier, FunctionDeclaration): Identifier(obj)
  | *:exit: Identifier(obj)
  | *: ObjectExpression
  | :expression: ObjectExpression
  | :paTTern: ObjectExpression
  | :not(Identifier): ObjectExpression
  | *: Property
  | :not(Identifier): Property
  | *: Identifier(a)
  | :expression: Identifier(a)
  | :paTTern: Identifier(a)
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
  | :expression: ArrayExpression
  | :paTTern: ArrayExpression
  | :not(Identifier): ArrayExpression
  | ObjectExpression ArrayExpression: ArrayExpression
  | *: Identifier(b)
  | :expression: Identifier(b)
  | :paTTern: Identifier(b)
  | Identifier: Identifier(b)
  | :matches(Identifier, FunctionDeclaration): Identifier(b)
  | ArrayExpression > Identifier: Identifier(b)
  | ArrayExpression Identifier: Identifier(b)
  | ObjectExpression Identifier: Identifier(b)
  | Program > VariableDeclaration > VariableDeclarator > ObjectExpression > Property > ArrayExpression > Identifier: Identifier(b)
  | ArrayExpression Identifier[name=b]: Identifier(b)
  | *:exit: Identifier(b)
  | *: Identifier(c)
  | :expression: Identifier(c)
  | :paTTern: Identifier(c)
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
  | :expression: Identifier(d)
  | :paTTern: Identifier(d)
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
  | :Declaration: FunctionDeclaration(foo)
  | :FUNCTION: FunctionDeclaration(foo)
  | :statement: FunctionDeclaration(foo)
  | :not(Identifier): FunctionDeclaration(foo)
  | :matches(Identifier, FunctionDeclaration): FunctionDeclaration(foo)
  | Program > FunctionDeclaration: FunctionDeclaration(foo)
  | VariableDeclaration + FunctionDeclaration: FunctionDeclaration(foo)
  | VariableDeclaration ~ FunctionDeclaration: FunctionDeclaration(foo)
  | :matches(ObjectExpression > SpreadElement, FunctionDeclaration): FunctionDeclaration(foo)
  | :matches(Identifier[name=a], FunctionDeclaration[id.name=foo]): FunctionDeclaration(foo)
  | *: Identifier(foo)
  | :expression: Identifier(foo)
  | :paTTern: Identifier(foo)
  | :function > Identifier: Identifier(foo)
  | Identifier: Identifier(foo)
  | :matches(Identifier, FunctionDeclaration): Identifier(foo)
  | *:exit: Identifier(foo)
  | *: BlockStatement
  | :statement: BlockStatement
  | :not(Identifier): BlockStatement
  | *:exit: BlockStatement
  | *:exit: FunctionDeclaration(foo)
  | *: FunctionDeclaration(bar)
  | :Declaration: FunctionDeclaration(bar)
  | :FUNCTION: FunctionDeclaration(bar)
  | :statement: FunctionDeclaration(bar)
  | :not(Identifier): FunctionDeclaration(bar)
  | :matches(Identifier, FunctionDeclaration): FunctionDeclaration(bar)
  | Program > FunctionDeclaration: FunctionDeclaration(bar)
  | VariableDeclaration ~ FunctionDeclaration: FunctionDeclaration(bar)
  | :matches(ObjectExpression > SpreadElement, FunctionDeclaration): FunctionDeclaration(bar)
  | :matches(ObjectExpression > SpreadElement, FunctionDeclaration[id.name=bar]): FunctionDeclaration(bar)
  | *: Identifier(bar)
  | :expression: Identifier(bar)
  | :paTTern: Identifier(bar)
  | :function > Identifier: Identifier(bar)
  | Identifier: Identifier(bar)
  | :matches(Identifier, FunctionDeclaration): Identifier(bar)
  | *:exit: Identifier(bar)
  | *: BlockStatement
  | :statement: BlockStatement
  | :not(Identifier): BlockStatement
  | *:exit: BlockStatement
  | *:exit: FunctionDeclaration(bar)
  | *: ExpressionStatement
  | :statement: ExpressionStatement
  | :not(Identifier): ExpressionStatement
  | *: ArrowFunctionExpression
  | :FUNCTION: ArrowFunctionExpression
  | :expression: ArrowFunctionExpression
  | :paTTern: ArrowFunctionExpression
  | :not(Identifier): ArrowFunctionExpression
  | *: ObjectPattern
  | :paTTern: ObjectPattern
  | :not(Identifier): ObjectPattern
  | *: Property
  | :not(Identifier): Property
  | *: Identifier(e)
  | :expression: Identifier(e)
  | :paTTern: Identifier(e)
  | Identifier: Identifier(e)
  | :matches(Identifier, FunctionDeclaration): Identifier(e)
  | Property > Identifier: Identifier(e)
  | *:exit: Identifier(e)
  | *: Identifier(f)
  | :expression: Identifier(f)
  | :paTTern: Identifier(f)
  | Identifier: Identifier(f)
  | :matches(Identifier, FunctionDeclaration): Identifier(f)
  | Property > Identifier: Identifier(f)
  | *:exit: Identifier(f)
  | *:exit: Property
  | *:exit: ObjectPattern
  | *: BlockStatement
  | :statement: BlockStatement
  | :not(Identifier): BlockStatement
  | *:exit: BlockStatement
  | *:exit: ArrowFunctionExpression
  | *:exit: ExpressionStatement
  | *: ExpressionStatement
  | :statement: ExpressionStatement
  | :not(Identifier): ExpressionStatement
  | *: MetaProperty
  | :expression: MetaProperty
  | :paTTern: MetaProperty
  | :not(Identifier): MetaProperty
  | *: Identifier(import)
  | Identifier: Identifier(import)
  | :matches(Identifier, FunctionDeclaration): Identifier(import)
  | *:exit: Identifier(import)
  | *: Identifier(meta)
  | Identifier: Identifier(meta)
  | :matches(Identifier, FunctionDeclaration): Identifier(meta)
  | *:exit: Identifier(meta)
  | *:exit: MetaProperty
  | *:exit: ExpressionStatement
  | *:exit: Program
   ,-[files/index.js:1:1]
 1 | ,-> const obj = { a: [b, c], ...d };
 2 | |   
 3 | |   function foo() {}
 4 | |   function bar() {}
 5 | |   
 6 | |   ({ e: f }) => {};
 7 | |   
 8 | `-> import.meta;
   `----

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
