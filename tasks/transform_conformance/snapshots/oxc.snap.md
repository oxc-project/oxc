commit: acbc09a8

Passed: 135/219

# All Passed:
* babel-plugin-transform-class-static-block
* babel-plugin-transform-private-methods
* babel-plugin-transform-logical-assignment-operators
* babel-plugin-transform-nullish-coalescing-operator
* babel-plugin-transform-optional-catch-binding
* babel-plugin-transform-async-generator-functions
* babel-plugin-transform-object-rest-spread
* babel-plugin-transform-async-to-generator
* babel-plugin-transform-exponentiation-operator
* babel-plugin-transform-arrow-functions
* babel-preset-typescript
* babel-plugin-transform-react-jsx-source
* regexp


# babel-plugin-transform-class-properties (20/27)
* private-field-resolve-to-method/input.js
x Output mismatch

* private-field-resolve-to-method-in-computed-key/input.js
x Output mismatch

* static-block-this-and-class-name/input.js
Symbol flags mismatch for "inner":
after transform: SymbolId(8): SymbolFlags(BlockScopedVariable | Function)
rebuilt        : SymbolId(14): SymbolFlags(FunctionScopedVariable)

* static-super-assignment-target/input.js
x Output mismatch

* static-super-tagged-template/input.js
x Output mismatch

* typescript/optional-call/input.ts
Symbol reference IDs mismatch for "X":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(6), ReferenceId(11), ReferenceId(16)]
rebuilt        : SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(8), ReferenceId(14)]

* typescript/optional-member/input.ts
Symbol reference IDs mismatch for "X":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(6), ReferenceId(9), ReferenceId(12)]
rebuilt        : SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(6), ReferenceId(10)]


# babel-plugin-transform-typescript (2/14)
* class-property-definition/input.ts
Unresolved references mismatch:
after transform: ["const"]
rebuilt        : []

* computed-constant-value/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "a", "b", "c", "d", "e"]
rebuilt        : ScopeId(1): ["A"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["B", "a", "b", "c", "d", "e"]
rebuilt        : ScopeId(2): ["B"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["C", "a", "b", "c"]
rebuilt        : ScopeId(3): ["C"]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(4): ["D", "a", "b", "c"]
rebuilt        : ScopeId(4): ["D"]
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(0x0)
rebuilt        : ScopeId(4): ScopeFlags(Function)
Symbol flags mismatch for "A":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "B":
after transform: SymbolId(6): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "C":
after transform: SymbolId(12): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(4): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "D":
after transform: SymbolId(16): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(6): SymbolFlags(FunctionScopedVariable)
Unresolved references mismatch:
after transform: ["Infinity", "NaN"]
rebuilt        : ["Infinity"]
Unresolved reference IDs mismatch for "Infinity":
after transform: [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(8), ReferenceId(11), ReferenceId(14), ReferenceId(18)]
rebuilt        : [ReferenceId(2), ReferenceId(5), ReferenceId(8), ReferenceId(12)]

* elimination-declare/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "ReactiveMarkerSymbol"]
rebuilt        : ScopeId(0): []
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1)]
rebuilt        : ScopeId(0): []

* enum-member-reference/input.ts
Missing ReferenceId: "Foo"
Missing ReferenceId: "Merge"
Missing ReferenceId: "NestInner"
Bindings mismatch:
after transform: ScopeId(1): ["Foo", "a", "b", "c"]
rebuilt        : ScopeId(1): ["Foo"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["Merge", "x"]
rebuilt        : ScopeId(2): ["Merge"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["Merge", "y"]
rebuilt        : ScopeId(3): ["Merge"]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(4): ["NestOuter", "a", "b"]
rebuilt        : ScopeId(4): ["NestOuter"]
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(0x0)
rebuilt        : ScopeId(4): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(6): ["NestInner", "a", "b"]
rebuilt        : ScopeId(6): ["NestInner"]
Scope flags mismatch:
after transform: ScopeId(6): ScopeFlags(0x0)
rebuilt        : ScopeId(6): ScopeFlags(Function)
Symbol reference IDs mismatch for "x":
after transform: SymbolId(0): [ReferenceId(2), ReferenceId(4)]
rebuilt        : SymbolId(0): [ReferenceId(7)]
Symbol flags mismatch for "Foo":
after transform: SymbolId(1): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(1): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(14): [ReferenceId(8), ReferenceId(9), ReferenceId(10), ReferenceId(11), ReferenceId(12), ReferenceId(13), ReferenceId(14)]
rebuilt        : SymbolId(2): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(4), ReferenceId(5), ReferenceId(6), ReferenceId(8)]
Symbol flags mismatch for "Merge":
after transform: SymbolId(5): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(3): SymbolFlags(FunctionScopedVariable)
Symbol redeclarations mismatch for "Merge":
after transform: SymbolId(5): [Span { start: 103, end: 108 }]
rebuilt        : SymbolId(3): []
Symbol reference IDs mismatch for "Merge":
after transform: SymbolId(16): [ReferenceId(20), ReferenceId(21), ReferenceId(22)]
rebuilt        : SymbolId(5): [ReferenceId(16), ReferenceId(17), ReferenceId(18), ReferenceId(19)]
Symbol flags mismatch for "NestOuter":
after transform: SymbolId(8): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(6): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "NestInner":
after transform: SymbolId(11): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(8): SymbolFlags(BlockScopedVariable)
Symbol reference IDs mismatch for "NestInner":
after transform: SymbolId(18): [ReferenceId(31), ReferenceId(32), ReferenceId(33), ReferenceId(34), ReferenceId(35)]
rebuilt        : SymbolId(9): [ReferenceId(25), ReferenceId(26), ReferenceId(28), ReferenceId(29), ReferenceId(30), ReferenceId(31)]

* export-elimination/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Bar", "Foo", "Func", "Im", "Name", "Ok"]
rebuilt        : ScopeId(0): ["Bar", "Foo", "Func", "Im", "Name", "Ok", "T"]
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2), ScopeId(3), ScopeId(4), ScopeId(5), ScopeId(6), ScopeId(7)]
rebuilt        : ScopeId(0): [ScopeId(1), ScopeId(2), ScopeId(3), ScopeId(4)]
Scope flags mismatch:
after transform: ScopeId(5): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Symbol flags mismatch for "Name":
after transform: SymbolId(7): SymbolFlags(NameSpaceModule | ValueModule)
rebuilt        : SymbolId(5): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "Name":
after transform: SymbolId(7): Span { start: 116, end: 120 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }
Symbol flags mismatch for "T":
after transform: SymbolId(9): SymbolFlags(FunctionScopedVariable | TypeAlias)
rebuilt        : SymbolId(8): SymbolFlags(FunctionScopedVariable)
Symbol span mismatch for "T":
after transform: SymbolId(9): Span { start: 205, end: 206 }
rebuilt        : SymbolId(8): Span { start: 226, end: 227 }
Symbol reference IDs mismatch for "T":
after transform: SymbolId(9): [ReferenceId(8), ReferenceId(9)]
rebuilt        : SymbolId(8): [ReferenceId(9)]
Symbol redeclarations mismatch for "T":
after transform: SymbolId(9): [Span { start: 226, end: 227 }]
rebuilt        : SymbolId(8): []

* exports/type-and-non-type/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1)]
rebuilt        : ScopeId(0): []

* namespace/export-import-=/input.ts
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "N1":
after transform: SymbolId(1): SymbolFlags(NameSpaceModule)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N1":
after transform: SymbolId(1): Span { start: 31, end: 33 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }

* namespace/import-=/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["V", "X", "_N"]
rebuilt        : ScopeId(1): ["V", "_N"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Symbol reference IDs mismatch for "A":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1)]
rebuilt        : SymbolId(0): [ReferenceId(2)]
Symbol flags mismatch for "N1":
after transform: SymbolId(1): SymbolFlags(NameSpaceModule | ValueModule)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N1":
after transform: SymbolId(1): Span { start: 31, end: 33 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol flags mismatch for "N2":
after transform: SymbolId(4): SymbolFlags(NameSpaceModule | ValueModule)
rebuilt        : SymbolId(4): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N2":
after transform: SymbolId(4): Span { start: 130, end: 132 }
rebuilt        : SymbolId(4): Span { start: 0, end: 0 }

* namespace/preserve-import-=/input.ts
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Symbol flags mismatch for "N1":
after transform: SymbolId(1): SymbolFlags(NameSpaceModule | ValueModule)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N1":
after transform: SymbolId(1): Span { start: 34, end: 36 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol flags mismatch for "N2":
after transform: SymbolId(4): SymbolFlags(NameSpaceModule | ValueModule)
rebuilt        : SymbolId(5): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N2":
after transform: SymbolId(4): Span { start: 145, end: 147 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(5): [ReferenceId(2)]
rebuilt        : SymbolId(7): []

* preserve-import-=/input.js
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(1): [ReferenceId(1)]
rebuilt        : SymbolId(1): []

* redeclarations/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A"]
rebuilt        : ScopeId(0): ["A", "B", "T"]
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): []
Symbol flags mismatch for "A":
after transform: SymbolId(0): SymbolFlags(BlockScopedVariable | ConstVariable | Import)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable | ConstVariable)
Symbol span mismatch for "A":
after transform: SymbolId(0): Span { start: 57, end: 58 }
rebuilt        : SymbolId(0): Span { start: 79, end: 83 }
Symbol reference IDs mismatch for "A":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1)]
rebuilt        : SymbolId(0): [ReferenceId(0)]
Symbol redeclarations mismatch for "A":
after transform: SymbolId(0): [Span { start: 79, end: 83 }]
rebuilt        : SymbolId(0): []
Symbol flags mismatch for "T":
after transform: SymbolId(1): SymbolFlags(Import | TypeAlias)
rebuilt        : SymbolId(1): SymbolFlags(Import)
Symbol redeclarations mismatch for "T":
after transform: SymbolId(1): [Span { start: 170, end: 171 }]
rebuilt        : SymbolId(1): []
Symbol flags mismatch for "B":
after transform: SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable | Import | TypeAlias)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable)
Symbol span mismatch for "B":
after transform: SymbolId(2): Span { start: 267, end: 268 }
rebuilt        : SymbolId(2): Span { start: 289, end: 293 }
Symbol reference IDs mismatch for "B":
after transform: SymbolId(2): [ReferenceId(3), ReferenceId(4)]
rebuilt        : SymbolId(2): [ReferenceId(2)]
Symbol redeclarations mismatch for "B":
after transform: SymbolId(2): [Span { start: 289, end: 293 }, Span { start: 304, end: 305 }]
rebuilt        : SymbolId(2): []

* ts-declaration-empty-output/input.d.ts
x Output mismatch


# babel-plugin-transform-react-jsx (37/40)
* refresh/does-not-transform-it-because-it-is-not-used-in-the-AST/input.jsx
x Output mismatch

* refresh/react-refresh/includes-custom-hooks-into-the-signatures-when-commonjs-target-is-used/input.jsx
x Output mismatch

* refresh/react-refresh/supports-typescript-namespace-syntax/input.tsx
x Output mismatch


# legacy-decorators (0/62)
* typescript/accessor/decoratorOnClassAccessor1/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor2/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor3/input.ts

  x Expected a semicolon or an implicit semicolon after a statement, but found
  | none
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/accessor/decoratorOnClassAccessor3/input.ts:6:11]
 5 | class C {
 6 |     public @dec get accessor() { return 1; }
   :           ^
 7 | }
   `----
  help: Try insert a semicolon here


* typescript/accessor/decoratorOnClassAccessor4/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor5/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor6/input.ts

  x Expected a semicolon or an implicit semicolon after a statement, but found
  | none
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/accessor/decoratorOnClassAccessor6/input.ts:6:11]
 5 | class C {
 6 |     public @dec set accessor(value: number) { }
   :           ^
 7 | }
   `----
  help: Try insert a semicolon here


* typescript/accessor/decoratorOnClassAccessor7/input.ts
x Output mismatch

* typescript/accessor/decoratorOnClassAccessor8/input.ts
x Output mismatch

* typescript/constructableDecoratorOnClass01/input.ts
Symbol span mismatch for "C":
after transform: SymbolId(1): Span { start: 74, end: 75 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(2): Span { start: 0, end: 0 }
rebuilt        : SymbolId(2): Span { start: 74, end: 75 }

* typescript/constructor/decoratorOnClassConstructor1/input.ts
x Output mismatch

* typescript/constructor/decoratorOnClassConstructor4/input.ts
x Output mismatch

* typescript/constructor/parameter/decoratorOnClassConstructorParameter1/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 155, end: 156 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(5): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 155, end: 156 }
Unresolved references mismatch:
after transform: ["Function", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/constructor/parameter/decoratorOnClassConstructorParameter4/input.ts

  x Expected `,` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/constructor/parameter/decoratorOnClassConstructorParameter4/input.ts:6:24]
 5 | class C {
 6 |     constructor(public @dec p: number) {}
   :                        |
   :                        `-- `,` expected
 7 | }
   `----


* typescript/constructor/parameter/decoratorOnClassConstructorParameter5/input.ts
x Output mismatch

* typescript/decoratedBlockScopedClass1/input.ts
x Output mismatch

* typescript/decoratedBlockScopedClass2/input.ts
x Output mismatch

* typescript/decoratedBlockScopedClass3/input.ts
x Output mismatch

* typescript/decoratedClassExportsCommonJS1/input.ts
x Output mismatch

* typescript/decoratedClassExportsCommonJS2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Something", "Testing123"]
rebuilt        : ScopeId(0): ["Testing123"]
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2), ScopeId(3)]
rebuilt        : ScopeId(0): [ScopeId(1), ScopeId(2)]
Symbol span mismatch for "Testing123":
after transform: SymbolId(2): Span { start: 241, end: 251 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "Testing123":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 241, end: 251 }
Reference symbol mismatch for "Something":
after transform: SymbolId(1) "Something"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["Something", "babelHelpers"]

* typescript/decoratedClassExportsSystem1/input.ts
x Output mismatch

* typescript/decoratedClassExportsSystem2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Something", "Testing123"]
rebuilt        : ScopeId(0): ["Testing123"]
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2), ScopeId(3)]
rebuilt        : ScopeId(0): [ScopeId(1), ScopeId(2)]
Symbol span mismatch for "Testing123":
after transform: SymbolId(2): Span { start: 239, end: 249 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "Testing123":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 239, end: 249 }
Reference symbol mismatch for "Something":
after transform: SymbolId(1) "Something"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["Something", "babelHelpers"]

* typescript/decoratorChecksFunctionBodies/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1), ScopeId(2), ScopeId(4)]
Scope children mismatch:
after transform: ScopeId(2): [ScopeId(3), ScopeId(4)]
rebuilt        : ScopeId(2): [ScopeId(3)]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(StrictMode | Function | Arrow)
rebuilt        : ScopeId(4): ScopeFlags(Function | Arrow)
Scope parent mismatch:
after transform: ScopeId(3): Some(ScopeId(2))
rebuilt        : ScopeId(4): Some(ScopeId(0))

* typescript/decoratorOnClass1/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Symbol span mismatch for "C":
after transform: SymbolId(2): Span { start: 99, end: 100 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 99, end: 100 }

* typescript/decoratorOnClass2/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Symbol span mismatch for "C":
after transform: SymbolId(2): Span { start: 127, end: 128 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 127, end: 128 }

* typescript/decoratorOnClass3/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Symbol span mismatch for "C":
after transform: SymbolId(2): Span { start: 127, end: 128 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 127, end: 128 }

* typescript/decoratorOnClass4/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Symbol span mismatch for "C":
after transform: SymbolId(1): Span { start: 107, end: 108 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(2): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 107, end: 108 }

* typescript/decoratorOnClass5/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Symbol span mismatch for "C":
after transform: SymbolId(1): Span { start: 107, end: 108 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(2): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 107, end: 108 }

* typescript/decoratorOnClass8/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Symbol span mismatch for "C":
after transform: SymbolId(0): Span { start: 134, end: 135 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(1): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 134, end: 135 }
Unresolved references mismatch:
after transform: ["Function", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/decoratorOnClass9/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod1/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod10/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["Function", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod11/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod12/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod13/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod14/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod15/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod16/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod17/input.ts

  x Expected a semicolon or an implicit semicolon after a statement, but found
  | none
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/decoratorOnClassMethod17/input.ts:7:17]
 6 | class Foo {
 7 |     private prop @decorator
   :                 ^
 8 |     foo() {
   `----
  help: Try insert a semicolon here


* typescript/method/decoratorOnClassMethod18/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod19/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod2/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod3/input.ts

  x Expected a semicolon or an implicit semicolon after a statement, but found
  | none
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/decoratorOnClassMethod3/input.ts:6:11]
 5 | class C {
 6 |     public @dec method() {}
   :           ^
 7 | }
   `----
  help: Try insert a semicolon here


* typescript/method/decoratorOnClassMethod4/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod5/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod6/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod7/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod8/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]

* typescript/method/decoratorOnClassMethodOverload1/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Scope children mismatch:
after transform: ScopeId(2): [ScopeId(3), ScopeId(4)]
rebuilt        : ScopeId(1): [ScopeId(2)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "dec"]
rebuilt        : []

* typescript/method/decoratorOnClassMethodOverload2/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Scope children mismatch:
after transform: ScopeId(2): [ScopeId(3), ScopeId(4)]
rebuilt        : ScopeId(1): [ScopeId(2)]
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter1/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["Object", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter2/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Symbol reference IDs mismatch for "C":
after transform: SymbolId(3): [ReferenceId(1), ReferenceId(4)]
rebuilt        : SymbolId(0): [ReferenceId(3)]
Unresolved references mismatch:
after transform: ["Object", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter3/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["Promise", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodThisParameter/input.ts

  x Identifier expected. 'this' is a reserved word that cannot be used here.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/parameter/decoratorOnClassMethodThisParameter/input.ts:6:17]
 5 | class C {
 6 |     method(@dec this: C) {}
   :                 ^^^^
 7 | }
   `----


* typescript/property/decoratorOnClassProperty1/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]

* typescript/property/decoratorOnClassProperty10/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]

* typescript/property/decoratorOnClassProperty11/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]

* typescript/property/decoratorOnClassProperty12/input.ts
x Output mismatch

* typescript/property/decoratorOnClassProperty13/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["PropertyDescriptor", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty2/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]

* typescript/property/decoratorOnClassProperty3/input.ts

  x Expected a semicolon or an implicit semicolon after a statement, but found
  | none
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/property/decoratorOnClassProperty3/input.ts:6:11]
 5 | class C {
 6 |     public @dec prop;
   :           ^
 7 | }
   `----
  help: Try insert a semicolon here


* typescript/property/decoratorOnClassProperty6/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["Function", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty7/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): [ScopeId(1)]
Unresolved references mismatch:
after transform: ["Function", "babelHelpers", "dec"]
rebuilt        : ["babelHelpers", "dec"]


