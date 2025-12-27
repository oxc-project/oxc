# Exit code
1

# stdout
```
  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mselectors(check):
  [38;2;225;80;80;1m│[0m *: Program
  [38;2;225;80;80;1m│[0m :not(Identifier): Program
  [38;2;225;80;80;1m│[0m *: VariableDeclaration
  [38;2;225;80;80;1m│[0m :Declaration: VariableDeclaration
  [38;2;225;80;80;1m│[0m :statement: VariableDeclaration
  [38;2;225;80;80;1m│[0m :not(Identifier): VariableDeclaration
  [38;2;225;80;80;1m│[0m *: VariableDeclarator
  [38;2;225;80;80;1m│[0m :not(Identifier): VariableDeclarator
  [38;2;225;80;80;1m│[0m *: Identifier(obj)
  [38;2;225;80;80;1m│[0m :expression: Identifier(obj)
  [38;2;225;80;80;1m│[0m :paTTern: Identifier(obj)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(obj)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(obj)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(obj)
  [38;2;225;80;80;1m│[0m *: ObjectExpression
  [38;2;225;80;80;1m│[0m :expression: ObjectExpression
  [38;2;225;80;80;1m│[0m :paTTern: ObjectExpression
  [38;2;225;80;80;1m│[0m :not(Identifier): ObjectExpression
  [38;2;225;80;80;1m│[0m *: Property
  [38;2;225;80;80;1m│[0m :not(Identifier): Property
  [38;2;225;80;80;1m│[0m *: Identifier(a)
  [38;2;225;80;80;1m│[0m :expression: Identifier(a)
  [38;2;225;80;80;1m│[0m :paTTern: Identifier(a)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(a)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(a)
  [38;2;225;80;80;1m│[0m ObjectExpression Identifier: Identifier(a)
  [38;2;225;80;80;1m│[0m Property > Identifier: Identifier(a)
  [38;2;225;80;80;1m│[0m ObjectExpression > Property > Identifier: Identifier(a)
  [38;2;225;80;80;1m│[0m Identifier[name=a]: Identifier(a)
  [38;2;225;80;80;1m│[0m ObjectExpression > Property > Identifier[name=a]: Identifier(a)
  [38;2;225;80;80;1m│[0m :matches(Identifier[name=a], FunctionDeclaration[id.name=foo]): Identifier(a)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(a)
  [38;2;225;80;80;1m│[0m *: ArrayExpression
  [38;2;225;80;80;1m│[0m :expression: ArrayExpression
  [38;2;225;80;80;1m│[0m :paTTern: ArrayExpression
  [38;2;225;80;80;1m│[0m :not(Identifier): ArrayExpression
  [38;2;225;80;80;1m│[0m ObjectExpression ArrayExpression: ArrayExpression
  [38;2;225;80;80;1m│[0m *: Identifier(b)
  [38;2;225;80;80;1m│[0m :expression: Identifier(b)
  [38;2;225;80;80;1m│[0m :paTTern: Identifier(b)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(b)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(b)
  [38;2;225;80;80;1m│[0m ArrayExpression > Identifier: Identifier(b)
  [38;2;225;80;80;1m│[0m ArrayExpression Identifier: Identifier(b)
  [38;2;225;80;80;1m│[0m ObjectExpression Identifier: Identifier(b)
  [38;2;225;80;80;1m│[0m Program > VariableDeclaration > VariableDeclarator > ObjectExpression > Property > ArrayExpression > Identifier: Identifier(b)
  [38;2;225;80;80;1m│[0m ArrayExpression Identifier[name=b]: Identifier(b)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(b)
  [38;2;225;80;80;1m│[0m *: Identifier(c)
  [38;2;225;80;80;1m│[0m :expression: Identifier(c)
  [38;2;225;80;80;1m│[0m :paTTern: Identifier(c)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(c)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(c)
  [38;2;225;80;80;1m│[0m ArrayExpression > Identifier: Identifier(c)
  [38;2;225;80;80;1m│[0m ArrayExpression Identifier: Identifier(c)
  [38;2;225;80;80;1m│[0m Identifier ~ Identifier: Identifier(c)
  [38;2;225;80;80;1m│[0m ObjectExpression Identifier: Identifier(c)
  [38;2;225;80;80;1m│[0m Program > VariableDeclaration > VariableDeclarator > ObjectExpression > Property > ArrayExpression > Identifier: Identifier(c)
  [38;2;225;80;80;1m│[0m ArrayExpression > Identifier[name=c]: Identifier(c)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(c)
  [38;2;225;80;80;1m│[0m *:exit: ArrayExpression
  [38;2;225;80;80;1m│[0m *:exit: Property
  [38;2;225;80;80;1m│[0m *: SpreadElement
  [38;2;225;80;80;1m│[0m :not(Identifier): SpreadElement
  [38;2;225;80;80;1m│[0m :matches(ObjectExpression > SpreadElement, FunctionDeclaration): SpreadElement
  [38;2;225;80;80;1m│[0m Property ~ [type]: SpreadElement
  [38;2;225;80;80;1m│[0m :matches(ObjectExpression > SpreadElement, FunctionDeclaration[id.name=bar]): SpreadElement
  [38;2;225;80;80;1m│[0m *: Identifier(d)
  [38;2;225;80;80;1m│[0m :expression: Identifier(d)
  [38;2;225;80;80;1m│[0m :paTTern: Identifier(d)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(d)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(d)
  [38;2;225;80;80;1m│[0m ObjectExpression Identifier: Identifier(d)
  [38;2;225;80;80;1m│[0m Identifier[name=d]: Identifier(d)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(d)
  [38;2;225;80;80;1m│[0m *:exit: SpreadElement
  [38;2;225;80;80;1m│[0m *:exit: ObjectExpression
  [38;2;225;80;80;1m│[0m *:exit: VariableDeclarator
  [38;2;225;80;80;1m│[0m *:exit: VariableDeclaration
  [38;2;225;80;80;1m│[0m *: FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m :Declaration: FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m :FUNCTION: FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m :statement: FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m :not(Identifier): FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m Program > FunctionDeclaration: FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m VariableDeclaration + FunctionDeclaration: FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m VariableDeclaration ~ FunctionDeclaration: FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m :matches(ObjectExpression > SpreadElement, FunctionDeclaration): FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m :matches(Identifier[name=a], FunctionDeclaration[id.name=foo]): FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m *: Identifier(foo)
  [38;2;225;80;80;1m│[0m :expression: Identifier(foo)
  [38;2;225;80;80;1m│[0m :paTTern: Identifier(foo)
  [38;2;225;80;80;1m│[0m :function > Identifier: Identifier(foo)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(foo)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(foo)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(foo)
  [38;2;225;80;80;1m│[0m *: BlockStatement
  [38;2;225;80;80;1m│[0m :statement: BlockStatement
  [38;2;225;80;80;1m│[0m :not(Identifier): BlockStatement
  [38;2;225;80;80;1m│[0m *:exit: BlockStatement
  [38;2;225;80;80;1m│[0m *:exit: FunctionDeclaration(foo)
  [38;2;225;80;80;1m│[0m *: FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m :Declaration: FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m :FUNCTION: FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m :statement: FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m :not(Identifier): FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m Program > FunctionDeclaration: FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m VariableDeclaration ~ FunctionDeclaration: FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m :matches(ObjectExpression > SpreadElement, FunctionDeclaration): FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m :matches(ObjectExpression > SpreadElement, FunctionDeclaration[id.name=bar]): FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m *: Identifier(bar)
  [38;2;225;80;80;1m│[0m :expression: Identifier(bar)
  [38;2;225;80;80;1m│[0m :paTTern: Identifier(bar)
  [38;2;225;80;80;1m│[0m :function > Identifier: Identifier(bar)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(bar)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(bar)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(bar)
  [38;2;225;80;80;1m│[0m *: BlockStatement
  [38;2;225;80;80;1m│[0m :statement: BlockStatement
  [38;2;225;80;80;1m│[0m :not(Identifier): BlockStatement
  [38;2;225;80;80;1m│[0m *:exit: BlockStatement
  [38;2;225;80;80;1m│[0m *:exit: FunctionDeclaration(bar)
  [38;2;225;80;80;1m│[0m *: ExpressionStatement
  [38;2;225;80;80;1m│[0m :statement: ExpressionStatement
  [38;2;225;80;80;1m│[0m :not(Identifier): ExpressionStatement
  [38;2;225;80;80;1m│[0m *: ArrowFunctionExpression
  [38;2;225;80;80;1m│[0m :FUNCTION: ArrowFunctionExpression
  [38;2;225;80;80;1m│[0m :expression: ArrowFunctionExpression
  [38;2;225;80;80;1m│[0m :paTTern: ArrowFunctionExpression
  [38;2;225;80;80;1m│[0m :not(Identifier): ArrowFunctionExpression
  [38;2;225;80;80;1m│[0m *: ObjectPattern
  [38;2;225;80;80;1m│[0m :paTTern: ObjectPattern
  [38;2;225;80;80;1m│[0m :not(Identifier): ObjectPattern
  [38;2;225;80;80;1m│[0m *: Property
  [38;2;225;80;80;1m│[0m :not(Identifier): Property
  [38;2;225;80;80;1m│[0m *: Identifier(e)
  [38;2;225;80;80;1m│[0m :expression: Identifier(e)
  [38;2;225;80;80;1m│[0m :paTTern: Identifier(e)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(e)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(e)
  [38;2;225;80;80;1m│[0m Property > Identifier: Identifier(e)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(e)
  [38;2;225;80;80;1m│[0m *: Identifier(f)
  [38;2;225;80;80;1m│[0m :expression: Identifier(f)
  [38;2;225;80;80;1m│[0m :paTTern: Identifier(f)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(f)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(f)
  [38;2;225;80;80;1m│[0m Property > Identifier: Identifier(f)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(f)
  [38;2;225;80;80;1m│[0m *:exit: Property
  [38;2;225;80;80;1m│[0m *:exit: ObjectPattern
  [38;2;225;80;80;1m│[0m *: BlockStatement
  [38;2;225;80;80;1m│[0m :statement: BlockStatement
  [38;2;225;80;80;1m│[0m :not(Identifier): BlockStatement
  [38;2;225;80;80;1m│[0m *:exit: BlockStatement
  [38;2;225;80;80;1m│[0m *:exit: ArrowFunctionExpression
  [38;2;225;80;80;1m│[0m *:exit: ExpressionStatement
  [38;2;225;80;80;1m│[0m *: ExpressionStatement
  [38;2;225;80;80;1m│[0m :statement: ExpressionStatement
  [38;2;225;80;80;1m│[0m :not(Identifier): ExpressionStatement
  [38;2;225;80;80;1m│[0m *: MetaProperty
  [38;2;225;80;80;1m│[0m :expression: MetaProperty
  [38;2;225;80;80;1m│[0m :paTTern: MetaProperty
  [38;2;225;80;80;1m│[0m :not(Identifier): MetaProperty
  [38;2;225;80;80;1m│[0m *: Identifier(import)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(import)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(import)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(import)
  [38;2;225;80;80;1m│[0m *: Identifier(meta)
  [38;2;225;80;80;1m│[0m Identifier: Identifier(meta)
  [38;2;225;80;80;1m│[0m :matches(Identifier, FunctionDeclaration): Identifier(meta)
  [38;2;225;80;80;1m│[0m *:exit: Identifier(meta)
  [38;2;225;80;80;1m│[0m *:exit: MetaProperty
  [38;2;225;80;80;1m│[0m *:exit: ExpressionStatement
  [38;2;225;80;80;1m│[0m *:exit: Program[0m
   ╭─[[38;2;92;157;255;1mfiles/index.js[0m:1:1]
 [2m1[0m │ [38;2;246;87;248m╭[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m const obj = { a: [b, c], ...d };
 [2m2[0m │ [38;2;246;87;248m│[0m   
 [2m3[0m │ [38;2;246;87;248m│[0m   function foo() {}
 [2m4[0m │ [38;2;246;87;248m│[0m   function bar() {}
 [2m5[0m │ [38;2;246;87;248m│[0m   
 [2m6[0m │ [38;2;246;87;248m│[0m   ({ e: f }) => {};
 [2m7[0m │ [38;2;246;87;248m│[0m   
 [2m8[0m │ [38;2;246;87;248m╰[0m[38;2;246;87;248m─[0m[38;2;246;87;248m▶[0m import.meta;
   ╰────

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 1 rules using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
