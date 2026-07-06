commit: 1fb0b771

Passed: 246/398

# All Passed:
* babel-plugin-transform-class-static-block
* babel-plugin-transform-private-methods
* babel-plugin-transform-logical-assignment-operators
* babel-plugin-transform-nullish-coalescing-operator
* babel-plugin-transform-optional-chaining
* babel-plugin-transform-optional-catch-binding
* babel-plugin-transform-async-generator-functions
* babel-plugin-transform-object-rest-spread
* babel-plugin-transform-async-to-generator
* babel-plugin-transform-exponentiation-operator
* babel-plugin-transform-arrow-functions
* babel-preset-typescript
* babel-plugin-transform-react-jsx-self
* babel-plugin-transform-react-jsx-source
* regexp
* plugin-tagged-template-transform


# babel-plugin-transform-explicit-resource-management (3/4)
* export-class-name/input.js
Symbol reference IDs mismatch for "C":
after transform: SymbolId(1): [ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(7)]
rebuilt        : SymbolId(2): [ReferenceId(0), ReferenceId(5), ReferenceId(6)]
Symbol reference IDs mismatch for "C":
after transform: SymbolId(3): []
rebuilt        : SymbolId(3): [ReferenceId(4)]
Reference symbol mismatch for "C":
after transform: SymbolId(1) "C"
rebuilt        : SymbolId(3) "C"


# babel-plugin-transform-class-properties (28/33)
* private-field-resolve-to-method/input.js
x Output mismatch

* private-field-resolve-to-method-in-computed-key/input.js
x Output mismatch

* static-super-assignment-target/input.js
x Output mismatch

* static-super-tagged-template/input.js
x Output mismatch

* typescript/declare-computed-keys/input.ts
Symbol reference IDs mismatch for "KEY1":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2)]
rebuilt        : SymbolId(1): []


# babel-plugin-transform-typescript (26/60)
* allow-declare-fields-false/input.ts
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

* computed-constant-value/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "a", "b", "c", "d", "e"]
rebuilt        : ScopeId(1): ["A"]
Bindings mismatch:
after transform: ScopeId(2): ["B", "a", "b", "c", "d", "e"]
rebuilt        : ScopeId(2): ["B"]
Bindings mismatch:
after transform: ScopeId(3): ["C", "a", "b", "c"]
rebuilt        : ScopeId(3): ["C"]
Bindings mismatch:
after transform: ScopeId(4): ["D", "a", "b", "c"]
rebuilt        : ScopeId(4): ["D"]
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

* const-enum-mixed-refs/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Phase", "one", "two"]
rebuilt        : ScopeId(1): ["Phase"]
Symbol flags mismatch for "Phase":
after transform: SymbolId(0): SymbolFlags(ConstEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* const-enum-value-ref-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Phase", "one", "two"]
rebuilt        : ScopeId(1): ["Phase"]
Symbol flags mismatch for "Phase":
after transform: SymbolId(0): SymbolFlags(ConstEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* declare-and-definite-with-initializer/input.ts

  x TS(1263): Declarations with initializers cannot also have definite
  | assignment assertions.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/declare-and-definite-with-initializer/input.ts:8:16]
 7 | class DefiniteExample {
 8 |    readonly bar! = "test";
   :                ^
 9 |    readonly foo! = 1;
   `----


  x TS(1263): Declarations with initializers cannot also have definite
  | assignment assertions.
    ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/declare-and-definite-with-initializer/input.ts:9:16]
  8 |    readonly bar! = "test";
  9 |    readonly foo! = 1;
    :                ^
 10 | }
    `----


* elimination-declare/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "ReactiveMarker", "ReactiveMarkerSymbol"]
rebuilt        : ScopeId(0): []

* enum-member-reference/input.ts
Missing ReferenceId: "Foo"
Missing ReferenceId: "Merge"
Missing ReferenceId: "NestInner"
Bindings mismatch:
after transform: ScopeId(1): ["Foo", "a", "b", "c"]
rebuilt        : ScopeId(1): ["Foo"]
Bindings mismatch:
after transform: ScopeId(2): ["Merge", "x"]
rebuilt        : ScopeId(2): ["Merge"]
Bindings mismatch:
after transform: ScopeId(3): ["Merge", "y"]
rebuilt        : ScopeId(3): ["Merge"]
Bindings mismatch:
after transform: ScopeId(4): ["NestOuter", "a", "b"]
rebuilt        : ScopeId(4): ["NestOuter"]
Bindings mismatch:
after transform: ScopeId(6): ["NestInner", "a", "b"]
rebuilt        : ScopeId(6): ["NestInner"]
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
after transform: SymbolId(5): [Span { start: 70, end: 75 }, Span { start: 103, end: 108 }]
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

* enum-string-alias-member/input.ts
Bindings mismatch:
after transform: ScopeId(2): ["Color", "Green", "Primary", "Red"]
rebuilt        : ScopeId(1): ["Color"]
Symbol flags mismatch for "Color":
after transform: SymbolId(4): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "Color":
after transform: SymbolId(4): [ReferenceId(5), ReferenceId(6), ReferenceId(7), ReferenceId(12)]
rebuilt        : SymbolId(0): [ReferenceId(6), ReferenceId(7), ReferenceId(8)]

* enum-template-literal/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["LARGE", "SMALL", "Size"]
rebuilt        : ScopeId(1): ["Size"]
Bindings mismatch:
after transform: ScopeId(2): ["Animal", "CAT", "DOG"]
rebuilt        : ScopeId(2): ["Animal"]
Bindings mismatch:
after transform: ScopeId(3): ["AnimalSize", "LARGE_DOG", "SMALL_CAT"]
rebuilt        : ScopeId(3): ["AnimalSize"]
Symbol flags mismatch for "Size":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "Size":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(7)]
rebuilt        : SymbolId(0): [ReferenceId(3)]
Symbol flags mismatch for "Animal":
after transform: SymbolId(3): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "Animal":
after transform: SymbolId(3): [ReferenceId(1), ReferenceId(3), ReferenceId(11)]
rebuilt        : SymbolId(2): [ReferenceId(7)]
Symbol flags mismatch for "AnimalSize":
after transform: SymbolId(6): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(4): SymbolFlags(FunctionScopedVariable)

* enum-template-literal-number/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["NUM_1", "NUM_2", "NUM_3", "NUM_4", "NumberEnum"]
rebuilt        : ScopeId(1): ["NumberEnum"]
Bindings mismatch:
after transform: ScopeId(2): ["COMPUTED_1", "COMPUTED_2", "ComputedEnum"]
rebuilt        : ScopeId(2): ["ComputedEnum"]
Symbol flags mismatch for "NumberEnum":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "NumberEnum":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(13)]
rebuilt        : SymbolId(0): [ReferenceId(9)]
Symbol flags mismatch for "ComputedEnum":
after transform: SymbolId(5): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)

* enum-template-literal-trailing-quasi/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "B", "NumberEnum"]
rebuilt        : ScopeId(1): ["NumberEnum"]
Bindings mismatch:
after transform: ScopeId(2): ["C", "ComputedEnum", "D"]
rebuilt        : ScopeId(2): ["ComputedEnum"]
Symbol flags mismatch for "NumberEnum":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "NumberEnum":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(8)]
rebuilt        : SymbolId(0): [ReferenceId(5)]
Symbol flags mismatch for "ComputedEnum":
after transform: SymbolId(3): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)

* export-elimination/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Bar", "Foo", "Func", "Im", "Name", "Ok"]
rebuilt        : ScopeId(0): ["Bar", "Foo", "Func", "Im", "Name", "Ok", "T"]
Symbol flags mismatch for "Name":
after transform: SymbolId(7): SymbolFlags(ValueModule)
rebuilt        : SymbolId(5): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "Name":
after transform: SymbolId(7): Span { start: 116, end: 120 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }
Symbol flags mismatch for "T":
after transform: SymbolId(9): SymbolFlags(Function | TypeAlias)
rebuilt        : SymbolId(8): SymbolFlags(Function)
Symbol span mismatch for "T":
after transform: SymbolId(9): Span { start: 205, end: 206 }
rebuilt        : SymbolId(8): Span { start: 226, end: 227 }
Symbol redeclarations mismatch for "T":
after transform: SymbolId(9): [Span { start: 205, end: 206 }, Span { start: 226, end: 227 }]
rebuilt        : SymbolId(8): []

* namespace/export-import-=/input.ts
Symbol flags mismatch for "N1":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N1":
after transform: SymbolId(1): Span { start: 31, end: 33 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }

* namespace/import-=/input.ts
Symbol reference IDs mismatch for "A":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1)]
rebuilt        : SymbolId(0): [ReferenceId(2)]
Symbol flags mismatch for "N1":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N1":
after transform: SymbolId(1): Span { start: 31, end: 33 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol flags mismatch for "N2":
after transform: SymbolId(4): SymbolFlags(ValueModule)
rebuilt        : SymbolId(4): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N2":
after transform: SymbolId(4): Span { start: 130, end: 132 }
rebuilt        : SymbolId(4): Span { start: 0, end: 0 }

* namespace/preserve-import-=/input.ts
Symbol flags mismatch for "N1":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N1":
after transform: SymbolId(1): Span { start: 34, end: 36 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol flags mismatch for "N2":
after transform: SymbolId(4): SymbolFlags(ValueModule)
rebuilt        : SymbolId(5): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N2":
after transform: SymbolId(4): Span { start: 145, end: 147 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }

* namespace/redeclaration-with-enum/input.ts
Bindings mismatch:
after transform: ScopeId(2): ["x", "y"]
rebuilt        : ScopeId(2): ["x"]
Symbol flags mismatch for "x":
after transform: SymbolId(0): SymbolFlags(RegularEnum | ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "x":
after transform: SymbolId(0): Span { start: 10, end: 11 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol redeclarations mismatch for "x":
after transform: SymbolId(0): [Span { start: 10, end: 11 }, Span { start: 39, end: 40 }]
rebuilt        : SymbolId(0): []
Symbol flags mismatch for "y":
after transform: SymbolId(2): SymbolFlags(RegularEnum | ValueModule)
rebuilt        : SymbolId(3): SymbolFlags(FunctionScopedVariable)
Symbol redeclarations mismatch for "y":
after transform: SymbolId(2): [Span { start: 59, end: 60 }, Span { start: 83, end: 84 }]
rebuilt        : SymbolId(3): []

* namespace/redeclaration-with-interface/input.ts
Bindings mismatch:
after transform: ScopeId(0): []
rebuilt        : ScopeId(0): ["Foo"]
Symbol flags mismatch for "Foo":
after transform: SymbolId(0): SymbolFlags(Interface | ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "Foo":
after transform: SymbolId(0): Span { start: 17, end: 20 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol redeclarations mismatch for "Foo":
after transform: SymbolId(0): [Span { start: 17, end: 20 }, Span { start: 41, end: 44 }]
rebuilt        : SymbolId(0): []

* namespace/redeclaration-with-type-alias/input.ts
Bindings mismatch:
after transform: ScopeId(0): []
rebuilt        : ScopeId(0): ["Foo"]
Symbol flags mismatch for "Foo":
after transform: SymbolId(0): SymbolFlags(TypeAlias | ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "Foo":
after transform: SymbolId(0): Span { start: 12, end: 15 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol redeclarations mismatch for "Foo":
after transform: SymbolId(0): [Span { start: 12, end: 15 }, Span { start: 39, end: 42 }, Span { start: 87, end: 90 }]
rebuilt        : SymbolId(0): []

* namespace/redeclaration-with-type-only-namespace/input.ts
Symbol flags mismatch for "Foo":
after transform: SymbolId(0): SymbolFlags(NamespaceModule | ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "Foo":
after transform: SymbolId(0): Span { start: 17, end: 20 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol redeclarations mismatch for "Foo":
after transform: SymbolId(0): [Span { start: 17, end: 20 }, Span { start: 62, end: 65 }]
rebuilt        : SymbolId(0): []

* optimize-enums/auto-increment-after-string/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "B", "Mixed"]
rebuilt        : ScopeId(1): ["Mixed"]
Symbol flags mismatch for "Mixed":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/exported-not-removed/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Direction", "Down", "Up"]
rebuilt        : ScopeId(1): ["Direction"]
Symbol flags mismatch for "Direction":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)

* optimize-enums/merged-enum/input.ts
Unresolved references mismatch:
after transform: ["A"]
rebuilt        : []

* optimize-enums/non-evaluable-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Runtime", "X", "Y"]
rebuilt        : ScopeId(1): ["Runtime"]
Symbol flags mismatch for "Runtime":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/optional-chain-value-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "B", "Foo"]
rebuilt        : ScopeId(1): ["Foo"]
Symbol flags mismatch for "Foo":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/passed-as-argument-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Active", "Inactive", "Status"]
rebuilt        : ScopeId(1): ["Status"]
Symbol flags mismatch for "Status":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/re-exported-not-removed/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "X"]
rebuilt        : ScopeId(1): ["A"]
Bindings mismatch:
after transform: ScopeId(2): ["B", "Y"]
rebuilt        : ScopeId(2): ["B"]
Symbol flags mismatch for "A":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "B":
after transform: SymbolId(2): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)

* optimize-enums/typeof-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Bar", "X"]
rebuilt        : ScopeId(1): ["Bar"]
Symbol flags mismatch for "Bar":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/value-usage-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "B", "Foo"]
rebuilt        : ScopeId(1): ["Foo"]
Symbol flags mismatch for "Foo":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* redeclarations/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A"]
rebuilt        : ScopeId(0): ["A", "B", "T"]
Symbol flags mismatch for "T":
after transform: SymbolId(1): SymbolFlags(Import | TypeAlias)
rebuilt        : SymbolId(1): SymbolFlags(Import)
Symbol redeclarations mismatch for "T":
after transform: SymbolId(1): [Span { start: 149, end: 150 }, Span { start: 170, end: 171 }]
rebuilt        : SymbolId(1): []
Symbol flags mismatch for "B":
after transform: SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable | TypeAlias)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable)
Symbol redeclarations mismatch for "B":
after transform: SymbolId(2): [Span { start: 289, end: 290 }, Span { start: 304, end: 305 }]
rebuilt        : SymbolId(2): []

* remove-class-properties-without-initializer/input.ts
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

* remove-unused-import-equals/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["D", "a", "b", "bar", "c"]
rebuilt        : ScopeId(0): ["a", "b", "bar", "c"]
Unresolved reference IDs mismatch for "foo":
after transform: [ReferenceId(0), ReferenceId(6)]
rebuilt        : [ReferenceId(0)]

* ts-declaration-empty-output/input.d.ts
x Output mismatch

* use-define-for-class-fields/input.ts
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

* use-define-for-class-fields-without-class-properties/input.ts
Unresolved reference IDs mismatch for "dce":
after transform: [ReferenceId(0), ReferenceId(1), ReferenceId(4), ReferenceId(9), ReferenceId(12), ReferenceId(14), ReferenceId(17)]
rebuilt        : [ReferenceId(5)]


# babel-plugin-transform-react-jsx (51/54)
* refresh/import-after-component/input.js
Missing ScopeId
Missing ReferenceId: "useFoo"
Symbol reference IDs mismatch for "useFoo":
after transform: SymbolId(1): [ReferenceId(1), ReferenceId(7)]
rebuilt        : SymbolId(1): [ReferenceId(6), ReferenceId(11), ReferenceId(12)]

* refresh/react-refresh/includes-custom-hooks-into-the-signatures-when-commonjs-target-is-used/input.jsx
x Output mismatch

* refresh/react-refresh/supports-typescript-namespace-syntax/input.tsx
x Output mismatch


# legacy-decorators (12/106)
* oxc/accessor/input.ts
x Output mismatch

* oxc/accessor-name-collision/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo", "_prop", "_prop2", "_prop3", "prop", "property"]
rebuilt        : ScopeId(0): ["Foo", "_prop", "_prop2", "_prop3", "prop"]
Reference symbol mismatch for "property":
after transform: SymbolId(0) "property"
rebuilt        : <None>
Reference symbol mismatch for "property":
after transform: SymbolId(0) "property"
rebuilt        : <None>
Reference symbol mismatch for "property":
after transform: SymbolId(0) "property"
rebuilt        : <None>
Reference symbol mismatch for "property":
after transform: SymbolId(0) "property"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "property"]

* oxc/accessor-with-class-properties/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "_a", "_a2", "_a_accessor_storage", "_a_computed_accessor_storage", "_b_accessor_storage", "_c_accessor_storage", "a", "dec"]
rebuilt        : ScopeId(0): ["C", "_a", "_a2", "_a_accessor_storage", "_a_computed_accessor_storage", "_b_accessor_storage", "_c_accessor_storage"]
Reference symbol mismatch for "a":
after transform: SymbolId(4) "a"
rebuilt        : <None>
Reference symbol mismatch for "a":
after transform: SymbolId(4) "a"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["WeakMap", "babelHelpers"]
rebuilt        : ["WeakMap", "a", "babelHelpers", "dec"]

* oxc/class-without-name-with-decorated_class/input.ts
Symbol flags mismatch for "_default":
after transform: SymbolId(1): SymbolFlags(Class)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)

* oxc/metadata/abstract-class/input.ts
Symbol span mismatch for "AbstractClass":
after transform: SymbolId(2): Span { start: 69, end: 82 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol span mismatch for "AbstractClass":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(3): Span { start: 69, end: 82 }

* oxc/metadata/ambient-declared-class/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Ambient", "Source", "dec"]
rebuilt        : ScopeId(0): ["Source"]
Reference symbol mismatch for "dec":
after transform: SymbolId(1) "dec"
rebuilt        : <None>
Reference symbol mismatch for "Ambient":
after transform: SymbolId(0) "Ambient"
rebuilt        : <None>
Reference symbol mismatch for "Ambient":
after transform: SymbolId(0) "Ambient"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["Ambient", "Object", "babelHelpers", "dec"]

* oxc/metadata/bound-type-reference/input.ts
Symbol span mismatch for "Example":
after transform: SymbolId(1): Span { start: 87, end: 94 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol span mismatch for "Example":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(2): Span { start: 87, end: 94 }

* oxc/metadata/class-and-method-decorators/input.ts
Symbol span mismatch for "Problem":
after transform: SymbolId(4): Span { start: 90, end: 97 }
rebuilt        : SymbolId(4): Span { start: 0, end: 0 }
Symbol span mismatch for "Problem":
after transform: SymbolId(5): Span { start: 0, end: 0 }
rebuilt        : SymbolId(5): Span { start: 90, end: 97 }

* oxc/metadata/class-expression-via-const/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "Source", "dec"]
rebuilt        : ScopeId(0): ["C", "Source"]
Symbol reference IDs mismatch for "C":
after transform: SymbolId(0): []
rebuilt        : SymbolId(0): [ReferenceId(3), ReferenceId(5)]
Reference symbol mismatch for "dec":
after transform: SymbolId(1) "dec"
rebuilt        : <None>
Reference symbol mismatch for "C":
after transform: <None>
rebuilt        : SymbolId(0) "C"
Reference flags mismatch for "C":
after transform: ReferenceId(2): ReferenceFlags(Read | Type)
rebuilt        : ReferenceId(3): ReferenceFlags(Read)
Reference symbol mismatch for "C":
after transform: <None>
rebuilt        : SymbolId(0) "C"
Reference flags mismatch for "C":
after transform: ReferenceId(3): ReferenceFlags(Read | Type)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Unresolved references mismatch:
after transform: ["C", "Object", "babelHelpers"]
rebuilt        : ["Object", "babelHelpers", "dec"]

* oxc/metadata/constructor-overload/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["MyService", "dec"]
rebuilt        : ScopeId(0): ["MyService"]
Symbol span mismatch for "MyService":
after transform: SymbolId(1): Span { start: 54, end: 63 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "MyService":
after transform: SymbolId(7): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 54, end: 63 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["String", "babelHelpers"]
rebuilt        : ["String", "babelHelpers", "dec"]

* oxc/metadata/cross-file-imported-enum/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Source", "StringEnum", "dec"]
rebuilt        : ScopeId(0): ["Source", "StringEnum"]
Reference symbol mismatch for "dec":
after transform: SymbolId(1) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["Object", "babelHelpers", "dec"]

* oxc/metadata/enum-types/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["StringEnum", "bar", "foo"]
rebuilt        : ScopeId(1): ["StringEnum"]
Bindings mismatch:
after transform: ScopeId(2): ["TemplateStringEnum", "mixed", "template"]
rebuilt        : ScopeId(2): ["TemplateStringEnum"]
Bindings mismatch:
after transform: ScopeId(3): ["NumberEnum", "a", "b"]
rebuilt        : ScopeId(3): ["NumberEnum"]
Bindings mismatch:
after transform: ScopeId(4): ["UnaryEnum", "bitwise", "negative", "positive"]
rebuilt        : ScopeId(4): ["UnaryEnum"]
Bindings mismatch:
after transform: ScopeId(6): ["UnaryOtherEnum", "bitwise", "negative", "positive"]
rebuilt        : ScopeId(6): ["UnaryOtherEnum"]
Bindings mismatch:
after transform: ScopeId(7): ["AutoIncrementEnum", "first", "second", "third"]
rebuilt        : ScopeId(7): ["AutoIncrementEnum"]
Bindings mismatch:
after transform: ScopeId(8): ["MixedEnum", "num", "str"]
rebuilt        : ScopeId(8): ["MixedEnum"]
Bindings mismatch:
after transform: ScopeId(9): ["ComputedEnum", "computed", "expression"]
rebuilt        : ScopeId(9): ["ComputedEnum"]
Symbol flags mismatch for "StringEnum":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "TemplateStringEnum":
after transform: SymbolId(3): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "NumberEnum":
after transform: SymbolId(6): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(4): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "UnaryEnum":
after transform: SymbolId(9): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(6): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "UnaryOtherEnum":
after transform: SymbolId(14): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(9): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "AutoIncrementEnum":
after transform: SymbolId(18): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(11): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "MixedEnum":
after transform: SymbolId(22): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(13): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch for "ComputedEnum":
after transform: SymbolId(25): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(15): SymbolFlags(FunctionScopedVariable)

* oxc/metadata/erased-import-no-type-keyword/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Source", "T", "dec"]
rebuilt        : ScopeId(0): ["Source", "T"]
Reference symbol mismatch for "dec":
after transform: SymbolId(1) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["Object", "babelHelpers", "dec"]

* oxc/metadata/forward-ref-class/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["LaterClass", "Source", "dec"]
rebuilt        : ScopeId(0): ["LaterClass", "Source"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["Object", "babelHelpers", "dec"]

* oxc/metadata/getter-setter-method/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Getter", "Setter", "UntypedGetter", "UntypedSetter", "dec"]
rebuilt        : ScopeId(0): ["Getter", "Setter", "UntypedGetter", "UntypedSetter"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Function", "Number", "Object", "String", "babelHelpers"]
rebuilt        : ["Function", "Number", "Object", "String", "babelHelpers", "dec"]

* oxc/metadata/imports/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Cls", "Foo", "dec"]
rebuilt        : ScopeId(0): ["Cls", "Foo"]
Symbol span mismatch for "Cls":
after transform: SymbolId(7): Span { start: 145, end: 148 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol span mismatch for "Cls":
after transform: SymbolId(12): Span { start: 0, end: 0 }
rebuilt        : SymbolId(2): Span { start: 145, end: 148 }
Reference symbol mismatch for "dec":
after transform: SymbolId(3) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers", "console"]
rebuilt        : ["Object", "babelHelpers", "console", "dec"]

* oxc/metadata/namespace-imported-enum/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["NS", "Source", "dec"]
rebuilt        : ScopeId(0): ["NS", "Source"]
Reference symbol mismatch for "dec":
after transform: SymbolId(1) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["Object", "babelHelpers", "dec"]

* oxc/metadata/nullable-union/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Source", "dec"]
rebuilt        : ScopeId(0): ["Source"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Boolean", "Number", "Object", "String", "babelHelpers"]
rebuilt        : ["Boolean", "Number", "Object", "String", "babelHelpers", "dec"]
Unresolved reference IDs mismatch for "String":
after transform: [ReferenceId(5), ReferenceId(20)]
rebuilt        : [ReferenceId(3)]
Unresolved reference IDs mismatch for "Number":
after transform: [ReferenceId(9), ReferenceId(21)]
rebuilt        : [ReferenceId(8)]

* oxc/metadata/params/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo", "methodDecorator", "paramDecorator"]
rebuilt        : ScopeId(0): ["Foo"]
Symbol span mismatch for "Foo":
after transform: SymbolId(4): Span { start: 107, end: 110 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "Foo":
after transform: SymbolId(11): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 107, end: 110 }
Reference symbol mismatch for "methodDecorator":
after transform: SymbolId(0) "methodDecorator"
rebuilt        : <None>
Reference symbol mismatch for "methodDecorator":
after transform: SymbolId(0) "methodDecorator"
rebuilt        : <None>
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Reference symbol mismatch for "methodDecorator":
after transform: SymbolId(0) "methodDecorator"
rebuilt        : <None>
Reference symbol mismatch for "methodDecorator":
after transform: SymbolId(0) "methodDecorator"
rebuilt        : <None>
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Boolean", "Function", "Number", "String", "babelHelpers"]
rebuilt        : ["Boolean", "Function", "Number", "String", "babelHelpers", "methodDecorator", "paramDecorator"]

* oxc/metadata/private-in-expression-in-decorator/input.ts
Binding symbols mismatch:
after transform: ScopeId(0): [SymbolId(0), SymbolId(1), SymbolId(2)]
rebuilt        : ScopeId(0): [SymbolId(0), SymbolId(1), SymbolId(2)]
Bindings mismatch:
after transform: ScopeId(1): ["Cls"]
rebuilt        : ScopeId(1): []
Bindings mismatch:
after transform: ScopeId(3): ["Cls2"]
rebuilt        : ScopeId(4): []
Symbol reference IDs mismatch for "dec":
after transform: SymbolId(0): [ReferenceId(4), ReferenceId(0), ReferenceId(1), ReferenceId(3)]
rebuilt        : SymbolId(0): [ReferenceId(1), ReferenceId(10)]
Symbol span mismatch for "Cls":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 46, end: 49 }
Symbol scope ID mismatch for "Cls":
after transform: SymbolId(4): ScopeId(1)
rebuilt        : SymbolId(1): ScopeId(0)
Symbol reference IDs mismatch for "Cls":
after transform: SymbolId(4): []
rebuilt        : SymbolId(1): [ReferenceId(2), ReferenceId(7)]
Symbol span mismatch for "Cls2":
after transform: SymbolId(5): Span { start: 0, end: 0 }
rebuilt        : SymbolId(2): Span { start: 116, end: 120 }
Symbol scope ID mismatch for "Cls2":
after transform: SymbolId(5): ScopeId(3)
rebuilt        : SymbolId(2): ScopeId(0)
Symbol reference IDs mismatch for "Cls2":
after transform: SymbolId(5): []
rebuilt        : SymbolId(2): [ReferenceId(11), ReferenceId(17)]
Reference symbol mismatch for "Cls":
after transform: SymbolId(1) "Cls"
rebuilt        : SymbolId(1) "Cls"
Reference symbol mismatch for "Cls":
after transform: SymbolId(1) "Cls"
rebuilt        : SymbolId(1) "Cls"
Reference symbol mismatch for "Cls2":
after transform: SymbolId(2) "Cls2"
rebuilt        : SymbolId(2) "Cls2"
Reference symbol mismatch for "Cls2":
after transform: SymbolId(2) "Cls2"
rebuilt        : SymbolId(2) "Cls2"
Unresolved reference IDs mismatch for "babelHelpers":
after transform: [ReferenceId(7), ReferenceId(8), ReferenceId(9), ReferenceId(11), ReferenceId(13), ReferenceId(17), ReferenceId(18), ReferenceId(19), ReferenceId(20), ReferenceId(22), ReferenceId(24)]
rebuilt        : [ReferenceId(0), ReferenceId(3), ReferenceId(5), ReferenceId(6), ReferenceId(8), ReferenceId(9), ReferenceId(12), ReferenceId(14), ReferenceId(16)]

* oxc/metadata/readonly-array/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Source", "dec"]
rebuilt        : ScopeId(0): ["Source"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Array", "babelHelpers"]
rebuilt        : ["Array", "babelHelpers", "dec"]

* oxc/metadata/readonly-array-interface-shadow/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Source", "dec"]
rebuilt        : ScopeId(0): ["Source"]
Reference symbol mismatch for "dec":
after transform: SymbolId(2) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["Object", "babelHelpers", "dec"]

* oxc/metadata/static-anonymous-class-expression/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "Foo", "dec"]
rebuilt        : ScopeId(0): ["A", "Foo"]
Symbol span mismatch for "Foo":
after transform: SymbolId(2): Span { start: 72, end: 75 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol span mismatch for "Foo":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(2): Span { start: 72, end: 75 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Error", "Object", "babelHelpers"]
rebuilt        : ["Error", "Object", "babelHelpers", "dec"]

* oxc/metadata/this/input.ts
Symbol span mismatch for "Example":
after transform: SymbolId(0): Span { start: 6, end: 13 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "Example":
after transform: SymbolId(2): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 6, end: 13 }

* oxc/metadata/typescript-syntax/input.ts

  x TS(1249): A decorator can only decorate a method implementation, not an
  | overload.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/metadata/typescript-syntax/input.ts:6:3]
 5 | class B {
 6 |   @m
   :   ^^
 7 |   method();
   `----
  help: Move this after all the overloads


* oxc/metadata/unbound-type-reference/input.ts
Symbol span mismatch for "Example":
after transform: SymbolId(0): Span { start: 6, end: 13 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "Example":
after transform: SymbolId(2): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 6, end: 13 }
Reference flags mismatch for "UnboundTypeReference":
after transform: ReferenceId(2): ReferenceFlags(Read | Type)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Reference flags mismatch for "UnboundTypeReference":
after transform: ReferenceId(3): ReferenceFlags(Read | Type)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)

* oxc/metadata/without-decorator/input.ts
Symbol span mismatch for "C":
after transform: SymbolId(2): Span { start: 106, end: 107 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(3): Span { start: 106, end: 107 }

* oxc/static-field/input.ts
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(ClassStaticBlock)
rebuilt        : ScopeId(4): ScopeFlags(StrictMode | ClassStaticBlock)
Scope parent mismatch:
after transform: ScopeId(4): Some(ScopeId(0))
rebuilt        : ScopeId(4): Some(ScopeId(3))
Symbol span mismatch for "Foo":
after transform: SymbolId(2): Span { start: 103, end: 106 }
rebuilt        : SymbolId(3): Span { start: 0, end: 0 }
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(2): [ReferenceId(4), ReferenceId(6), ReferenceId(8)]
rebuilt        : SymbolId(3): [ReferenceId(4), ReferenceId(8)]
Symbol span mismatch for "Foo":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(4): Span { start: 103, end: 106 }

* oxc/static-field-with-class-properties/input.ts
Symbol span mismatch for "Foo":
after transform: SymbolId(2): Span { start: 103, end: 106 }
rebuilt        : SymbolId(3): Span { start: 0, end: 0 }
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(2): [ReferenceId(4), ReferenceId(6), ReferenceId(8), ReferenceId(10)]
rebuilt        : SymbolId(3): [ReferenceId(4), ReferenceId(6), ReferenceId(10)]
Symbol span mismatch for "Foo":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(4): Span { start: 103, end: 106 }

* oxc/with-class-private-properties/input.ts
Symbol span mismatch for "C":
after transform: SymbolId(0): Span { start: 11, end: 12 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(5): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 11, end: 12 }
Symbol span mismatch for "D":
after transform: SymbolId(1): Span { start: 85, end: 86 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol span mismatch for "D":
after transform: SymbolId(6): Span { start: 0, end: 0 }
rebuilt        : SymbolId(3): Span { start: 85, end: 86 }
Symbol span mismatch for "E":
after transform: SymbolId(2): Span { start: 167, end: 168 }
rebuilt        : SymbolId(4): Span { start: 0, end: 0 }
Symbol span mismatch for "E":
after transform: SymbolId(7): Span { start: 0, end: 0 }
rebuilt        : SymbolId(5): Span { start: 167, end: 168 }

* oxc/with-class-private-properties-unnamed-default-export/input.ts
Symbol flags mismatch for "_default":
after transform: SymbolId(0): SymbolFlags(Class)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)

* typescript/accessor/decoratorOnClassAccessor1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor3/input.ts

  x Expected `;` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/accessor/decoratorOnClassAccessor3/input.ts:6:12]
 5 | class C {
 6 |     public @dec get accessor() { return 1; }
   :            |
   :            `-- `;` expected
 7 | }
   `----


* typescript/accessor/decoratorOnClassAccessor4/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor5/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor6/input.ts

  x Expected `;` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/accessor/decoratorOnClassAccessor6/input.ts:6:12]
 5 | class C {
 6 |     public @dec set accessor(value: number) { }
   :            |
   :            `-- `;` expected
 7 | }
   `----


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
Bindings mismatch:
after transform: ScopeId(0): ["A", "B", "C", "dec"]
rebuilt        : ScopeId(0): ["A", "B", "C"]
Symbol span mismatch for "A":
after transform: SymbolId(1): Span { start: 139, end: 140 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "A":
after transform: SymbolId(5): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 139, end: 140 }
Symbol span mismatch for "B":
after transform: SymbolId(2): Span { start: 157, end: 158 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol span mismatch for "B":
after transform: SymbolId(6): Span { start: 0, end: 0 }
rebuilt        : SymbolId(3): Span { start: 157, end: 158 }
Symbol span mismatch for "C":
after transform: SymbolId(4): Span { start: 205, end: 206 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(7): Span { start: 0, end: 0 }
rebuilt        : SymbolId(6): Span { start: 205, end: 206 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Number", "babelHelpers"]
rebuilt        : ["Number", "babelHelpers", "dec"]

* typescript/constructor/parameter/decoratorOnClassConstructorParameter1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Symbol span mismatch for "C":
after transform: SymbolId(4): Span { start: 155, end: 156 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(6): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 155, end: 156 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/constructor/parameter/decoratorOnClassConstructorParameter4/input.ts

  x Expected `,` or `)` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/constructor/parameter/decoratorOnClassConstructorParameter4/input.ts:6:24]
 5 | class C {
 6 |     constructor(public @dec p: number) {}
   :                |       |
   :                |       `-- `,` or `)` expected
   :                `-- Opened here
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
after transform: ScopeId(0): ["Something", "Testing123", "forwardRef"]
rebuilt        : ScopeId(0): ["Testing123"]
Symbol span mismatch for "Testing123":
after transform: SymbolId(3): Span { start: 241, end: 251 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "Testing123":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 241, end: 251 }
Reference symbol mismatch for "Something":
after transform: SymbolId(2) "Something"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["Something", "babelHelpers"]

* typescript/decoratedClassExportsSystem1/input.ts
x Output mismatch

* typescript/decoratedClassExportsSystem2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Something", "Testing123", "forwardRef"]
rebuilt        : ScopeId(0): ["Testing123"]
Symbol span mismatch for "Testing123":
after transform: SymbolId(3): Span { start: 239, end: 249 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "Testing123":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 239, end: 249 }
Reference symbol mismatch for "Something":
after transform: SymbolId(2) "Something"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["Something", "babelHelpers"]

* typescript/decoratorChecksFunctionBodies/input.ts
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(StrictMode | Function | Arrow)
rebuilt        : ScopeId(4): ScopeFlags(Function | Arrow)
Scope parent mismatch:
after transform: ScopeId(3): Some(ScopeId(2))
rebuilt        : ScopeId(4): Some(ScopeId(0))

* typescript/decoratorOnClass1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 99, end: 100 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 99, end: 100 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/decoratorOnClass2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 127, end: 128 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 127, end: 128 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/decoratorOnClass3/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 127, end: 128 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 127, end: 128 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/decoratorOnClass4/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 107, end: 108 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 107, end: 108 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/decoratorOnClass5/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 107, end: 108 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 107, end: 108 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/decoratorOnClass8/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 134, end: 135 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(1): Span { start: 134, end: 135 }
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/decoratorOnClass9/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod10/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod11/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod12/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod13/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod14/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo", "decorator"]
rebuilt        : ScopeId(0): ["Foo"]
Reference symbol mismatch for "decorator":
after transform: SymbolId(0) "decorator"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Function", "babelHelpers"]
rebuilt        : ["Function", "babelHelpers", "decorator"]

* typescript/method/decoratorOnClassMethod15/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo", "decorator"]
rebuilt        : ScopeId(0): ["Foo"]
Reference symbol mismatch for "decorator":
after transform: SymbolId(0) "decorator"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Function", "babelHelpers"]
rebuilt        : ["Function", "babelHelpers", "decorator"]

* typescript/method/decoratorOnClassMethod16/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo", "decorator"]
rebuilt        : ScopeId(0): ["Foo"]
Reference symbol mismatch for "decorator":
after transform: SymbolId(0) "decorator"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Function", "babelHelpers"]
rebuilt        : ["Function", "babelHelpers", "decorator"]

* typescript/method/decoratorOnClassMethod17/input.ts

  x Expected `;` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/decoratorOnClassMethod17/input.ts:7:18]
 6 | class Foo {
 7 |     private prop @decorator
   :                  |
   :                  `-- `;` expected
 8 |     foo() {
   `----


* typescript/method/decoratorOnClassMethod18/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo", "decorator"]
rebuilt        : ScopeId(0): ["Foo"]
Reference symbol mismatch for "decorator":
after transform: SymbolId(0) "decorator"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["Object", "babelHelpers", "decorator"]

* typescript/method/decoratorOnClassMethod19/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod3/input.ts

  x Expected `;` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/decoratorOnClassMethod3/input.ts:6:12]
 5 | class C {
 6 |     public @dec method() {}
   :            |
   :            `-- `;` expected
 7 | }
   `----


* typescript/method/decoratorOnClassMethod4/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod5/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod6/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod7/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod8/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethodOverload1/input.ts

  x TS(1249): A decorator can only decorate a method implementation, not an
  | overload.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/decoratorOnClassMethodOverload1/input.ts:6:5]
 5 | class C {
 6 |     @dec
   :     ^^^^
 7 |     method()
   `----
  help: Move this after all the overloads


* typescript/method/decoratorOnClassMethodOverload2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter3/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["dec", "fn"]
rebuilt        : ScopeId(0): ["fn"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodThisParameter/input.ts

  x Identifier expected. 'this' is a reserved word that cannot be used here.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/parameter/decoratorOnClassMethodThisParameter/input.ts:6:17]
 5 | class C {
 6 |     method(@dec this: C) {}
   :                 ^^^^
 7 | }
   `----


* typescript/property/decoratorOnClassAccessorProperty1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty10/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty11/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty12/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "dec"]
rebuilt        : ScopeId(0): ["A"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["String", "babelHelpers"]
rebuilt        : ["String", "babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty13/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty3/input.ts

  x Expected `;` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/property/decoratorOnClassProperty3/input.ts:6:12]
 5 | class C {
 6 |     public @dec prop;
   :            |
   :            `-- `;` expected
 7 | }
   `----


* typescript/property/decoratorOnClassProperty6/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty7/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]


# plugin-styled-components (25/40)
* minify-comments/input.js
Unresolved references mismatch:
after transform: ["x", "y", "z"]
rebuilt        : ["x", "z"]

* styled-components/add-identifier-with-top-level-import-paths/input.js
x Output mismatch

* styled-components/add-identifier-with-top-level-import-paths-and-named-import/input.js
x Output mismatch

* styled-components/annotate-create-global-style-with-pure-comments/input.js
x Output mismatch

* styled-components/annotate-css-with-pure-comments/input.js
x Output mismatch

* styled-components/annotate-styled-calls-with-pure-comments/input.js
x Output mismatch

* styled-components/css-declared-after-component/input.jsx
x Output mismatch

* styled-components/does-not-replace-native-with-no-tags/input.js
x Output mismatch

* styled-components/pre-transpiled/input.js
x Output mismatch

* styled-components/transformed-imports-with-jsx-member-expressions/input.jsx
x Output mismatch

* styled-components/transpile-css-prop/input.jsx
x Output mismatch

* styled-components/transpile-css-prop-add-import/input.jsx
x Output mismatch

* styled-components/transpile-css-prop-add-require/input.jsx
x Output mismatch

* styled-components/transpile-css-prop-all-options-on/input.jsx
x Output mismatch

* styled-components/transpile-require-default/input.js
x Output mismatch


