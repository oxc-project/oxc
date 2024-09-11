commit: 3bcfee23

Passed: 17/51

# All Passed:
* babel-plugin-transform-nullish-coalescing-operator
* babel-plugin-transform-optional-catch-binding
* babel-preset-typescript
* regexp


# babel-plugin-transform-arrow-functions (1/2)
* with-this-member-expression/input.jsx
x Output mismatch


# babel-plugin-transform-typescript (1/8)
* class-property-definition/input.ts
Unresolved references mismatch:
after transform: ["const"]
rebuilt        : []

* computed-constant-value/input.ts
Missing ReferenceId: Infinity
Missing ReferenceId: Infinity
Missing ReferenceId: Infinity
Missing ReferenceId: Infinity
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
Symbol flags mismatch:
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch:
after transform: SymbolId(6): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch:
after transform: SymbolId(12): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(4): SymbolFlags(FunctionScopedVariable)
Symbol flags mismatch:
after transform: SymbolId(16): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(6): SymbolFlags(FunctionScopedVariable)
Unresolved references mismatch:
after transform: ["Infinity", "NaN"]
rebuilt        : ["Infinity"]
Unresolved reference IDs mismatch for "Infinity":
after transform: [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3)]
rebuilt        : [ReferenceId(2), ReferenceId(5), ReferenceId(8), ReferenceId(12)]

* elimination-declare/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "ReactiveMarkerSymbol"]
rebuilt        : ScopeId(0): []
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1)]
rebuilt        : ScopeId(0): []

* enum-member-reference/input.ts
Missing ReferenceId: Foo
Bindings mismatch:
after transform: ScopeId(1): ["Foo", "a", "b", "c"]
rebuilt        : ScopeId(1): ["Foo"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch:
after transform: SymbolId(1): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(1): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch:
after transform: SymbolId(5): [ReferenceId(3), ReferenceId(4), ReferenceId(5), ReferenceId(6), ReferenceId(7), ReferenceId(8), ReferenceId(9)]
rebuilt        : SymbolId(2): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(4), ReferenceId(5), ReferenceId(6), ReferenceId(8)]

* export-elimination/input.ts
Missing SymbolId: Name
Missing SymbolId: _Name
Missing ReferenceId: _Name
Missing ReferenceId: Name
Missing ReferenceId: Name
Bindings mismatch:
after transform: ScopeId(0): ["Baq", "Bar", "Baz", "Foo", "Func", "Im", "Name", "Ok", "T"]
rebuilt        : ScopeId(0): ["Bar", "Foo", "Func", "Im", "Name", "Ok", "T"]
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2), ScopeId(3), ScopeId(4), ScopeId(5), ScopeId(6), ScopeId(7)]
rebuilt        : ScopeId(0): [ScopeId(1), ScopeId(2), ScopeId(3), ScopeId(4)]
Binding symbols mismatch:
after transform: ScopeId(5): [SymbolId(8), SymbolId(10)]
rebuilt        : ScopeId(3): [SymbolId(6), SymbolId(7)]
Scope flags mismatch:
after transform: ScopeId(5): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Symbol flags mismatch:
after transform: SymbolId(8): SymbolFlags(BlockScopedVariable | ConstVariable | Export)
rebuilt        : SymbolId(7): SymbolFlags(BlockScopedVariable | ConstVariable)
Symbol flags mismatch:
after transform: SymbolId(9): SymbolFlags(FunctionScopedVariable | Export | TypeAlias)
rebuilt        : SymbolId(8): SymbolFlags(FunctionScopedVariable | Export)
Symbol span mismatch:
after transform: SymbolId(9): Span { start: 205, end: 206 }
rebuilt        : SymbolId(8): Span { start: 226, end: 227 }
Symbol reference IDs mismatch:
after transform: SymbolId(9): [ReferenceId(8), ReferenceId(9)]
rebuilt        : SymbolId(8): [ReferenceId(9)]
Symbol redeclarations mismatch:
after transform: SymbolId(9): [Span { start: 226, end: 227 }]
rebuilt        : SymbolId(8): []
Reference symbol mismatch:
after transform: ReferenceId(7): Some("Name")
rebuilt        : ReferenceId(8): Some("Name")

* redeclarations/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): []
Symbol flags mismatch:
after transform: SymbolId(0): SymbolFlags(BlockScopedVariable | ConstVariable | Export | Import)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable | ConstVariable | Export)
Symbol span mismatch:
after transform: SymbolId(0): Span { start: 57, end: 58 }
rebuilt        : SymbolId(0): Span { start: 79, end: 83 }
Symbol reference IDs mismatch:
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1)]
rebuilt        : SymbolId(0): [ReferenceId(0)]
Symbol redeclarations mismatch:
after transform: SymbolId(0): [Span { start: 79, end: 83 }]
rebuilt        : SymbolId(0): []
Symbol flags mismatch:
after transform: SymbolId(1): SymbolFlags(Export | Import | TypeAlias)
rebuilt        : SymbolId(1): SymbolFlags(Export | Import)
Symbol redeclarations mismatch:
after transform: SymbolId(1): [Span { start: 170, end: 171 }]
rebuilt        : SymbolId(1): []
Symbol flags mismatch:
after transform: SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable | Export | Import | TypeAlias)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable | Export)
Symbol span mismatch:
after transform: SymbolId(2): Span { start: 267, end: 268 }
rebuilt        : SymbolId(2): Span { start: 289, end: 293 }
Symbol reference IDs mismatch:
after transform: SymbolId(2): [ReferenceId(3), ReferenceId(4)]
rebuilt        : SymbolId(2): [ReferenceId(2)]
Symbol redeclarations mismatch:
after transform: SymbolId(2): [Span { start: 289, end: 293 }, Span { start: 304, end: 305 }]
rebuilt        : SymbolId(2): []

* ts-declaration-empty-output/input.d.ts
x Output mismatch


# babel-plugin-transform-react-jsx (3/29)
* refresh/can-handle-implicit-arrow-returns/input.jsx
Symbol reference IDs mismatch:
after transform: SymbolId(9): [ReferenceId(23), ReferenceId(24), ReferenceId(25)]
rebuilt        : SymbolId(0): [ReferenceId(6), ReferenceId(7)]
Symbol reference IDs mismatch:
after transform: SymbolId(10): [ReferenceId(26), ReferenceId(27), ReferenceId(29)]
rebuilt        : SymbolId(1): [ReferenceId(10), ReferenceId(13)]
Symbol reference IDs mismatch:
after transform: SymbolId(11): [ReferenceId(30), ReferenceId(31), ReferenceId(32)]
rebuilt        : SymbolId(2): [ReferenceId(18), ReferenceId(19)]
Symbol reference IDs mismatch:
after transform: SymbolId(12): [ReferenceId(33), ReferenceId(34), ReferenceId(36)]
rebuilt        : SymbolId(3): [ReferenceId(22), ReferenceId(25)]
Symbol reference IDs mismatch:
after transform: SymbolId(13): [ReferenceId(37), ReferenceId(38), ReferenceId(39), ReferenceId(40)]
rebuilt        : SymbolId(4): [ReferenceId(29), ReferenceId(32), ReferenceId(33)]
Symbol reference IDs mismatch:
after transform: SymbolId(14): [ReferenceId(41), ReferenceId(42), ReferenceId(44)]
rebuilt        : SymbolId(5): [ReferenceId(38), ReferenceId(41)]
Symbol reference IDs mismatch:
after transform: SymbolId(4): [ReferenceId(14), ReferenceId(45), ReferenceId(46)]
rebuilt        : SymbolId(10): [ReferenceId(15), ReferenceId(46)]
Symbol reference IDs mismatch:
after transform: SymbolId(5): [ReferenceId(16), ReferenceId(47), ReferenceId(48)]
rebuilt        : SymbolId(11): [ReferenceId(27), ReferenceId(48)]
Symbol reference IDs mismatch:
after transform: SymbolId(6): [ReferenceId(18), ReferenceId(49), ReferenceId(50)]
rebuilt        : SymbolId(12): [ReferenceId(31), ReferenceId(50)]
Symbol reference IDs mismatch:
after transform: SymbolId(7): [ReferenceId(19), ReferenceId(51), ReferenceId(52)]
rebuilt        : SymbolId(13): [ReferenceId(36), ReferenceId(52)]
Symbol reference IDs mismatch:
after transform: SymbolId(8): [ReferenceId(21), ReferenceId(53), ReferenceId(54)]
rebuilt        : SymbolId(14): [ReferenceId(43), ReferenceId(54)]
Reference symbol mismatch:
after transform: ReferenceId(23): Some("_s")
rebuilt        : ReferenceId(0): None
Reference symbol mismatch:
after transform: ReferenceId(26): Some("_s2")
rebuilt        : ReferenceId(1): None
Reference symbol mismatch:
after transform: ReferenceId(30): Some("_s3")
rebuilt        : ReferenceId(2): None
Reference symbol mismatch:
after transform: ReferenceId(33): Some("_s4")
rebuilt        : ReferenceId(3): None
Reference symbol mismatch:
after transform: ReferenceId(37): Some("_s5")
rebuilt        : ReferenceId(4): None
Reference symbol mismatch:
after transform: ReferenceId(41): Some("_s6")
rebuilt        : ReferenceId(5): None
Reference flags mismatch:
after transform: ReferenceId(18): ReferenceFlags(Write)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | Write)
Reference symbol mismatch:
after transform: ReferenceId(45): Some("_c")
rebuilt        : ReferenceId(45): None
Reference symbol mismatch:
after transform: ReferenceId(47): Some("_c2")
rebuilt        : ReferenceId(47): None
Reference symbol mismatch:
after transform: ReferenceId(49): Some("_c3")
rebuilt        : ReferenceId(49): None
Reference symbol mismatch:
after transform: ReferenceId(51): Some("_c4")
rebuilt        : ReferenceId(51): None
Reference symbol mismatch:
after transform: ReferenceId(53): Some("_c5")
rebuilt        : ReferenceId(53): None
Unresolved references mismatch:
after transform: ["X", "memo", "module", "useContext"]
rebuilt        : ["$RefreshReg$", "$RefreshSig$", "X", "memo", "module", "useContext"]

* refresh/does-not-consider-require-like-methods-to-be-hocs/input.jsx
x Output mismatch

* refresh/does-not-get-tripped-by-iifes/input.jsx
Bindings mismatch:
after transform: ScopeId(0): []
rebuilt        : ScopeId(0): ["_s"]
Bindings mismatch:
after transform: ScopeId(1): ["_s"]
rebuilt        : ScopeId(1): []
Symbol scope ID mismatch:
after transform: SymbolId(1): ScopeId(1)
rebuilt        : SymbolId(0): ScopeId(0)
Symbol reference IDs mismatch:
after transform: SymbolId(1): [ReferenceId(3), ReferenceId(4), ReferenceId(5)]
rebuilt        : SymbolId(0): [ReferenceId(2), ReferenceId(3)]
Reference symbol mismatch:
after transform: ReferenceId(3): Some("_s")
rebuilt        : ReferenceId(1): None
Unresolved references mismatch:
after transform: ["item", "useFoo"]
rebuilt        : ["$RefreshSig$", "item", "useFoo"]

* refresh/does-not-transform-it-because-it-is-not-used-in-the-AST/input.jsx
x Output mismatch

* refresh/emit-full-signatures-option/input.jsx
x Output mismatch

* refresh/generates-signatures-for-function-declarations-calling-hooks/input.jsx
x Output mismatch

* refresh/generates-signatures-for-function-expressions-calling-hooks/input.jsx
x Output mismatch

* refresh/generates-valid-signature-for-exotic-ways-to-call-hooks/input.jsx
x Output mismatch

* refresh/ignores-complex-definitions/input.jsx
x Output mismatch

* refresh/ignores-hoc-definitions/input.jsx
x Output mismatch

* refresh/includes-custom-hooks-into-the-signatures/input.jsx
x Output mismatch

* refresh/registers-capitalized-identifiers-in-hoc-calls/input.jsx
x Output mismatch

* refresh/registers-identifiers-used-in-jsx-at-definition-site/input.jsx
x Output mismatch

* refresh/registers-identifiers-used-in-react-create-element-at-definition-site/input.jsx
Symbol reference IDs mismatch:
after transform: SymbolId(13): [ReferenceId(33), ReferenceId(47), ReferenceId(48)]
rebuilt        : SymbolId(13): [ReferenceId(2), ReferenceId(48)]
Symbol reference IDs mismatch:
after transform: SymbolId(14): [ReferenceId(35), ReferenceId(49), ReferenceId(50)]
rebuilt        : SymbolId(14): [ReferenceId(5), ReferenceId(50)]
Symbol reference IDs mismatch:
after transform: SymbolId(15): [ReferenceId(37), ReferenceId(51), ReferenceId(52)]
rebuilt        : SymbolId(15): [ReferenceId(8), ReferenceId(52)]
Symbol reference IDs mismatch:
after transform: SymbolId(16): [ReferenceId(39), ReferenceId(53), ReferenceId(54)]
rebuilt        : SymbolId(16): [ReferenceId(12), ReferenceId(54)]
Symbol reference IDs mismatch:
after transform: SymbolId(17): [ReferenceId(41), ReferenceId(55), ReferenceId(56)]
rebuilt        : SymbolId(17): [ReferenceId(35), ReferenceId(56)]
Symbol reference IDs mismatch:
after transform: SymbolId(18): [ReferenceId(43), ReferenceId(57), ReferenceId(58)]
rebuilt        : SymbolId(18): [ReferenceId(41), ReferenceId(58)]
Symbol reference IDs mismatch:
after transform: SymbolId(19): [ReferenceId(45), ReferenceId(59), ReferenceId(60)]
rebuilt        : SymbolId(19): [ReferenceId(45), ReferenceId(60)]
Reference symbol mismatch:
after transform: ReferenceId(47): Some("_c")
rebuilt        : ReferenceId(47): None
Reference symbol mismatch:
after transform: ReferenceId(49): Some("_c2")
rebuilt        : ReferenceId(49): None
Reference symbol mismatch:
after transform: ReferenceId(51): Some("_c3")
rebuilt        : ReferenceId(51): None
Reference symbol mismatch:
after transform: ReferenceId(53): Some("_c4")
rebuilt        : ReferenceId(53): None
Reference symbol mismatch:
after transform: ReferenceId(55): Some("_c5")
rebuilt        : ReferenceId(55): None
Reference symbol mismatch:
after transform: ReferenceId(57): Some("_c6")
rebuilt        : ReferenceId(57): None
Reference symbol mismatch:
after transform: ReferenceId(59): Some("_c7")
rebuilt        : ReferenceId(59): None
Unresolved references mismatch:
after transform: ["React", "funny", "hoc", "jsx", "styled", "wow"]
rebuilt        : ["$RefreshReg$", "React", "funny", "hoc", "jsx", "styled", "wow"]

* refresh/registers-likely-hocs-with-inline-functions-1/input.jsx
x Output mismatch

* refresh/registers-likely-hocs-with-inline-functions-2/input.jsx
x Output mismatch

* refresh/registers-likely-hocs-with-inline-functions-3/input.jsx
x Output mismatch

* refresh/registers-top-level-exported-function-declarations/input.jsx
x Output mismatch

* refresh/registers-top-level-exported-named-arrow-functions/input.jsx
x Output mismatch

* refresh/registers-top-level-function-declarations/input.jsx
x Output mismatch

* refresh/registers-top-level-variable-declarations-with-arrow-functions/input.jsx
x Output mismatch

* refresh/registers-top-level-variable-declarations-with-function-expressions/input.jsx
x Output mismatch

* refresh/supports-typescript-namespace-syntax/input.tsx
x Output mismatch

* refresh/uses-custom-identifiers-for-refresh-reg-and-refresh-sig/input.jsx
x Output mismatch

* refresh/uses-original-function-declaration-if-it-get-reassigned/input.jsx
x Output mismatch

* unicode/input.jsx
x Output mismatch


