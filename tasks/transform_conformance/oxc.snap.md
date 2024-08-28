commit: 12619ffe

Passed: 11/37

# All Passed:
* babel-plugin-transform-optional-catch-binding
* babel-plugin-transform-arrow-functions


# babel-plugin-transform-typescript (3/8)
* computed-constant-value/input.ts
  x Missing ReferenceId: Infinity

  x Missing ReferenceId: Infinity

  x Missing ReferenceId: Infinity

  x Missing ReferenceId: Infinity

  x Bindings mismatch:
  | after transform: ScopeId(1): ["A", "a", "b", "c", "d", "e"]
  | rebuilt        : ScopeId(1): ["A"]

  x Scope flags mismatch:
  | after transform: ScopeId(1): ScopeFlags(StrictMode)
  | rebuilt        : ScopeId(1): ScopeFlags(StrictMode | Function)

  x Bindings mismatch:
  | after transform: ScopeId(2): ["B", "a", "b", "c", "d", "e"]
  | rebuilt        : ScopeId(2): ["B"]

  x Scope flags mismatch:
  | after transform: ScopeId(2): ScopeFlags(StrictMode)
  | rebuilt        : ScopeId(2): ScopeFlags(StrictMode | Function)

  x Bindings mismatch:
  | after transform: ScopeId(3): ["C", "a", "b", "c"]
  | rebuilt        : ScopeId(3): ["C"]

  x Scope flags mismatch:
  | after transform: ScopeId(3): ScopeFlags(StrictMode)
  | rebuilt        : ScopeId(3): ScopeFlags(StrictMode | Function)

  x Bindings mismatch:
  | after transform: ScopeId(4): ["D", "a", "b", "c"]
  | rebuilt        : ScopeId(4): ["D"]

  x Scope flags mismatch:
  | after transform: ScopeId(4): ScopeFlags(StrictMode)
  | rebuilt        : ScopeId(4): ScopeFlags(StrictMode | Function)

  x Symbol flags mismatch:
  | after transform: SymbolId(0): SymbolFlags(RegularEnum)
  | rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

  x Symbol flags mismatch:
  | after transform: SymbolId(6): SymbolFlags(RegularEnum)
  | rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)

  x Symbol flags mismatch:
  | after transform: SymbolId(12): SymbolFlags(RegularEnum)
  | rebuilt        : SymbolId(4): SymbolFlags(FunctionScopedVariable)

  x Symbol flags mismatch:
  | after transform: SymbolId(16): SymbolFlags(RegularEnum)
  | rebuilt        : SymbolId(6): SymbolFlags(FunctionScopedVariable)


* elimination-declare/input.ts
  x Bindings mismatch:
  | after transform: ScopeId(0): ["A", "ReactiveMarkerSymbol"]
  | rebuilt        : ScopeId(0): []


* enum-member-reference/input.ts
  x Missing ReferenceId: Foo

  x Bindings mismatch:
  | after transform: ScopeId(1): ["Foo", "a", "b", "c"]
  | rebuilt        : ScopeId(1): ["Foo"]

  x Scope flags mismatch:
  | after transform: ScopeId(1): ScopeFlags(StrictMode)
  | rebuilt        : ScopeId(1): ScopeFlags(StrictMode | Function)

  x Symbol flags mismatch:
  | after transform: SymbolId(1): SymbolFlags(RegularEnum)
  | rebuilt        : SymbolId(1): SymbolFlags(FunctionScopedVariable)

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(3), ReferenceId(4),
  | ReferenceId(5), ReferenceId(6), ReferenceId(7), ReferenceId(8),
  | ReferenceId(9)]
  | rebuilt        : SymbolId(2): [ReferenceId(0), ReferenceId(1),
  | ReferenceId(2), ReferenceId(3), ReferenceId(4), ReferenceId(5),
  | ReferenceId(6), ReferenceId(8)]


* export-elimination/input.ts
  x Missing SymbolId: Name

  x Missing SymbolId: _Name

  x Missing ReferenceId: _Name

  x Missing ReferenceId: Name

  x Missing ReferenceId: Name

  x Bindings mismatch:
  | after transform: ScopeId(0): ["Baq", "Bar", "Baz", "Foo", "Func", "Im",
  | "Name", "Ok", "T"]
  | rebuilt        : ScopeId(0): ["Bar", "Foo", "Func", "Im", "Name", "Ok",
  | "T"]

  x Binding symbols mismatch:
  | after transform: ScopeId(5): [SymbolId(8), SymbolId(10)]
  | rebuilt        : ScopeId(3): [SymbolId(6), SymbolId(7)]

  x Symbol flags mismatch:
  | after transform: SymbolId(8): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export)
  | rebuilt        : SymbolId(7): SymbolFlags(BlockScopedVariable |
  | ConstVariable)

  x Symbol flags mismatch:
  | after transform: SymbolId(9): SymbolFlags(BlockScopedVariable | Export |
  | Function | TypeAlias)
  | rebuilt        : SymbolId(8): SymbolFlags(BlockScopedVariable | Export
  | | Function)

  x Symbol span mismatch:
  | after transform: SymbolId(9): Span { start: 205, end: 206 }
  | rebuilt        : SymbolId(8): Span { start: 226, end: 227 }

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(9): [ReferenceId(8), ReferenceId(9)]
  | rebuilt        : SymbolId(8): [ReferenceId(9)]

  x Symbol redeclarations mismatch:
  | after transform: SymbolId(9): [Span { start: 226, end: 227 }]
  | rebuilt        : SymbolId(8): []

  x Reference symbol mismatch:
  | after transform: ReferenceId(7): Some("Name")
  | rebuilt        : ReferenceId(8): Some("Name")


* redeclarations/input.ts
  x Symbol flags mismatch:
  | after transform: SymbolId(0): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export | Import)
  | rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export)

  x Symbol span mismatch:
  | after transform: SymbolId(0): Span { start: 57, end: 58 }
  | rebuilt        : SymbolId(0): Span { start: 79, end: 83 }

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1)]
  | rebuilt        : SymbolId(0): [ReferenceId(0)]

  x Symbol redeclarations mismatch:
  | after transform: SymbolId(0): [Span { start: 79, end: 83 }]
  | rebuilt        : SymbolId(0): []

  x Symbol flags mismatch:
  | after transform: SymbolId(1): SymbolFlags(Export | Import | TypeAlias)
  | rebuilt        : SymbolId(1): SymbolFlags(Export | Import)

  x Symbol redeclarations mismatch:
  | after transform: SymbolId(1): [Span { start: 170, end: 171 }]
  | rebuilt        : SymbolId(1): []

  x Symbol flags mismatch:
  | after transform: SymbolId(2): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export | Import | TypeAlias)
  | rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export)

  x Symbol span mismatch:
  | after transform: SymbolId(2): Span { start: 267, end: 268 }
  | rebuilt        : SymbolId(2): Span { start: 289, end: 293 }

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(2): [ReferenceId(3), ReferenceId(4)]
  | rebuilt        : SymbolId(2): [ReferenceId(2)]

  x Symbol redeclarations mismatch:
  | after transform: SymbolId(2): [Span { start: 289, end: 293 }, Span
  | { start: 304, end: 305 }]
  | rebuilt        : SymbolId(2): []



# babel-plugin-transform-react-jsx (6/27)
* refresh/can-handle-implicit-arrow-returns/input.jsx
  x Output mismatch
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(9): [ReferenceId(23), ReferenceId(24),
  | ReferenceId(25)]
  | rebuilt        : SymbolId(0): [ReferenceId(6), ReferenceId(7)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(10): [ReferenceId(26), ReferenceId(27),
  | ReferenceId(28)]
  | rebuilt        : SymbolId(1): [ReferenceId(18), ReferenceId(19)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(11): [ReferenceId(29), ReferenceId(30),
  | ReferenceId(31), ReferenceId(32)]
  | rebuilt        : SymbolId(2): [ReferenceId(29), ReferenceId(32),
  | ReferenceId(33)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(12): [ReferenceId(33), ReferenceId(34),
  | ReferenceId(36)]
  | rebuilt        : SymbolId(3): [ReferenceId(10), ReferenceId(13)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(13): [ReferenceId(37), ReferenceId(38),
  | ReferenceId(40)]
  | rebuilt        : SymbolId(4): [ReferenceId(22), ReferenceId(25)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(14): [ReferenceId(41), ReferenceId(42),
  | ReferenceId(44)]
  | rebuilt        : SymbolId(5): [ReferenceId(38), ReferenceId(41)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(4): [ReferenceId(14), ReferenceId(45),
  | ReferenceId(46)]
  | rebuilt        : SymbolId(10): [ReferenceId(15), ReferenceId(46)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(16), ReferenceId(47),
  | ReferenceId(48)]
  | rebuilt        : SymbolId(11): [ReferenceId(27), ReferenceId(48)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(6): [ReferenceId(18), ReferenceId(49),
  | ReferenceId(50)]
  | rebuilt        : SymbolId(12): [ReferenceId(31), ReferenceId(50)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(7): [ReferenceId(19), ReferenceId(51),
  | ReferenceId(52)]
  | rebuilt        : SymbolId(13): [ReferenceId(36), ReferenceId(52)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(8): [ReferenceId(21), ReferenceId(53),
  | ReferenceId(54)]
  | rebuilt        : SymbolId(14): [ReferenceId(43), ReferenceId(54)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(23): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(26): Some("_s2")
  | rebuilt        : ReferenceId(1): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(29): Some("_s3")
  | rebuilt        : ReferenceId(2): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(33): Some("_s4")
  | rebuilt        : ReferenceId(3): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(37): Some("_s5")
  | rebuilt        : ReferenceId(4): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(41): Some("_s6")
  | rebuilt        : ReferenceId(5): None

  x Reference flags mismatch:
  | after transform: ReferenceId(18): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(31): ReferenceFlags(Read | Write)

  x Reference symbol mismatch:
  | after transform: ReferenceId(45): Some("_c")
  | rebuilt        : ReferenceId(45): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(47): Some("_c2")
  | rebuilt        : ReferenceId(47): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(49): Some("_c3")
  | rebuilt        : ReferenceId(49): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(51): Some("_c4")
  | rebuilt        : ReferenceId(51): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(53): Some("_c5")
  | rebuilt        : ReferenceId(53): None


* refresh/does-not-consider-require-like-methods-to-be-hocs/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(0): [ReferenceId(6), ReferenceId(12)]
  | rebuilt        : SymbolId(2): [ReferenceId(8)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(1): [ReferenceId(7), ReferenceId(14)]
  | rebuilt        : SymbolId(3): [ReferenceId(10)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(2): [ReferenceId(8), ReferenceId(16)]
  | rebuilt        : SymbolId(4): [ReferenceId(12)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(3): [ReferenceId(9), ReferenceId(18)]
  | rebuilt        : SymbolId(5): [ReferenceId(14)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(10), ReferenceId(21),
  | ReferenceId(22)]
  | rebuilt        : SymbolId(7): [ReferenceId(15), ReferenceId(18)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(21): Some("_c")
  | rebuilt        : ReferenceId(17): None


* refresh/does-not-get-tripped-by-iifes/input.jsx
  x Bindings mismatch:
  | after transform: ScopeId(0): []
  | rebuilt        : ScopeId(0): ["_s"]

  x Bindings mismatch:
  | after transform: ScopeId(1): ["_s"]
  | rebuilt        : ScopeId(1): []

  x Symbol scope ID mismatch:
  | after transform: SymbolId(1): ScopeId(1)
  | rebuilt        : SymbolId(0): ScopeId(0)

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(1): [ReferenceId(3), ReferenceId(4),
  | ReferenceId(5)]
  | rebuilt        : SymbolId(0): [ReferenceId(2), ReferenceId(3)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(3): Some("_s")
  | rebuilt        : ReferenceId(1): None


* refresh/generates-signatures-for-function-declarations-calling-hooks/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(6), ReferenceId(7),
  | ReferenceId(9)]
  | rebuilt        : SymbolId(1): [ReferenceId(1), ReferenceId(6)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(3): [ReferenceId(3), ReferenceId(10),
  | ReferenceId(11)]
  | rebuilt        : SymbolId(5): [ReferenceId(8), ReferenceId(11)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(6): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(10): Some("_c")
  | rebuilt        : ReferenceId(10): None


* refresh/generates-signatures-for-function-expressions-calling-hooks/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(22): [ReferenceId(26), ReferenceId(27),
  | ReferenceId(28), ReferenceId(29), ReferenceId(30)]
  | rebuilt        : SymbolId(1): [ReferenceId(2), ReferenceId(5),
  | ReferenceId(8), ReferenceId(9)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(23): [ReferenceId(32), ReferenceId(33),
  | ReferenceId(34), ReferenceId(35), ReferenceId(36)]
  | rebuilt        : SymbolId(2): [ReferenceId(17), ReferenceId(20),
  | ReferenceId(23), ReferenceId(24)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(24): [ReferenceId(38), ReferenceId(39),
  | ReferenceId(40)]
  | rebuilt        : SymbolId(14): [ReferenceId(33), ReferenceId(34)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(15): [ReferenceId(17), ReferenceId(41),
  | ReferenceId(42)]
  | rebuilt        : SymbolId(19): [ReferenceId(7), ReferenceId(42)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(16): [ReferenceId(18), ReferenceId(43),
  | ReferenceId(44)]
  | rebuilt        : SymbolId(20): [ReferenceId(4), ReferenceId(44)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(17): [ReferenceId(19), ReferenceId(45),
  | ReferenceId(46)]
  | rebuilt        : SymbolId(21): [ReferenceId(15), ReferenceId(46)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(18): [ReferenceId(21), ReferenceId(47),
  | ReferenceId(48)]
  | rebuilt        : SymbolId(22): [ReferenceId(22), ReferenceId(48)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(19): [ReferenceId(22), ReferenceId(49),
  | ReferenceId(50)]
  | rebuilt        : SymbolId(23): [ReferenceId(19), ReferenceId(50)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(20): [ReferenceId(23), ReferenceId(51),
  | ReferenceId(52)]
  | rebuilt        : SymbolId(24): [ReferenceId(30), ReferenceId(52)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(26): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(32): Some("_s2")
  | rebuilt        : ReferenceId(1): None

  x Reference flags mismatch:
  | after transform: ReferenceId(18): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(4): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(17): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(7): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(22): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(19): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(21): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(22): ReferenceFlags(Read | Write)

  x Reference symbol mismatch:
  | after transform: ReferenceId(38): Some("_s3")
  | rebuilt        : ReferenceId(32): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(41): Some("_c")
  | rebuilt        : ReferenceId(41): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(43): Some("_c2")
  | rebuilt        : ReferenceId(43): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(45): Some("_c3")
  | rebuilt        : ReferenceId(45): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(47): Some("_c4")
  | rebuilt        : ReferenceId(47): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(49): Some("_c5")
  | rebuilt        : ReferenceId(49): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(51): Some("_c6")
  | rebuilt        : ReferenceId(51): None


* refresh/generates-valid-signature-for-exotic-ways-to-call-hooks/input.jsx
  x Missing ScopeId

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(10): [ReferenceId(17), ReferenceId(18),
  | ReferenceId(20)]
  | rebuilt        : SymbolId(0): [ReferenceId(1), ReferenceId(16)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(9): [ReferenceId(12), ReferenceId(13),
  | ReferenceId(15)]
  | rebuilt        : SymbolId(4): [ReferenceId(3), ReferenceId(7)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(7): [ReferenceId(9), ReferenceId(21),
  | ReferenceId(22)]
  | rebuilt        : SymbolId(10): [ReferenceId(19), ReferenceId(22)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(17): Some("_s2")
  | rebuilt        : ReferenceId(0): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(12): Some("_s")
  | rebuilt        : ReferenceId(2): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(21): Some("_c")
  | rebuilt        : ReferenceId(21): None


* refresh/includes-custom-hooks-into-the-signatures/input.jsx
  x Missing ScopeId

  x Missing ScopeId

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(8): [ReferenceId(10), ReferenceId(11),
  | ReferenceId(13)]
  | rebuilt        : SymbolId(1): [ReferenceId(3), ReferenceId(7)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(9): [ReferenceId(14), ReferenceId(15),
  | ReferenceId(17)]
  | rebuilt        : SymbolId(2): [ReferenceId(10), ReferenceId(12)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(10): [ReferenceId(19), ReferenceId(20),
  | ReferenceId(22)]
  | rebuilt        : SymbolId(3): [ReferenceId(14), ReferenceId(18)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(6): [ReferenceId(6), ReferenceId(23),
  | ReferenceId(24)]
  | rebuilt        : SymbolId(10): [ReferenceId(21), ReferenceId(24)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(10): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(14): Some("_s2")
  | rebuilt        : ReferenceId(1): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(19): Some("_s3")
  | rebuilt        : ReferenceId(2): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(23): Some("_c")
  | rebuilt        : ReferenceId(23): None


* refresh/registers-capitalized-identifiers-in-hoc-calls/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(3): [ReferenceId(6), ReferenceId(14),
  | ReferenceId(15)]
  | rebuilt        : SymbolId(4): [ReferenceId(1), ReferenceId(15)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(4): [ReferenceId(8), ReferenceId(16),
  | ReferenceId(17)]
  | rebuilt        : SymbolId(5): [ReferenceId(3), ReferenceId(17)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(9), ReferenceId(18),
  | ReferenceId(19)]
  | rebuilt        : SymbolId(6): [ReferenceId(8), ReferenceId(19)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(6): [ReferenceId(11), ReferenceId(20),
  | ReferenceId(21)]
  | rebuilt        : SymbolId(7): [ReferenceId(12), ReferenceId(21)]

  x Reference flags mismatch:
  | after transform: ReferenceId(8): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(3): ReferenceFlags(Read | Write)

  x Reference symbol mismatch:
  | after transform: ReferenceId(14): Some("_c")
  | rebuilt        : ReferenceId(14): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(16): Some("_c2")
  | rebuilt        : ReferenceId(16): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(18): Some("_c3")
  | rebuilt        : ReferenceId(18): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(20): Some("_c4")
  | rebuilt        : ReferenceId(20): None


* refresh/registers-identifiers-used-in-jsx-at-definition-site/input.jsx
  x Output mismatch
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(0): [ReferenceId(5), ReferenceId(7),
  | ReferenceId(8), ReferenceId(9), ReferenceId(19), ReferenceId(21),
  | ReferenceId(34)]
  | rebuilt        : SymbolId(0): [ReferenceId(9), ReferenceId(13),
  | ReferenceId(14), ReferenceId(17), ReferenceId(37), ReferenceId(41)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(2): [ReferenceId(16), ReferenceId(23),
  | ReferenceId(48)]
  | rebuilt        : SymbolId(4): [ReferenceId(3), ReferenceId(31)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(3): [ReferenceId(11), ReferenceId(25),
  | ReferenceId(38)]
  | rebuilt        : SymbolId(5): [ReferenceId(6), ReferenceId(21)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(4): [ReferenceId(12), ReferenceId(40)]
  | rebuilt        : SymbolId(6): [ReferenceId(23)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(13), ReferenceId(42)]
  | rebuilt        : SymbolId(7): [ReferenceId(25)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(7): [ReferenceId(14), ReferenceId(44)]
  | rebuilt        : SymbolId(9): [ReferenceId(27)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(8): [ReferenceId(15), ReferenceId(46)]
  | rebuilt        : SymbolId(10): [ReferenceId(29)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(9): [ReferenceId(17), ReferenceId(50)]
  | rebuilt        : SymbolId(11): [ReferenceId(33)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(11): [ReferenceId(10), ReferenceId(31),
  | ReferenceId(36)]
  | rebuilt        : SymbolId(13): [ReferenceId(19), ReferenceId(39)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(13): [ReferenceId(22), ReferenceId(53),
  | ReferenceId(54)]
  | rebuilt        : SymbolId(15): [ReferenceId(2), ReferenceId(45)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(14): [ReferenceId(24), ReferenceId(55),
  | ReferenceId(56)]
  | rebuilt        : SymbolId(16): [ReferenceId(5), ReferenceId(47)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(15): [ReferenceId(26), ReferenceId(57),
  | ReferenceId(58)]
  | rebuilt        : SymbolId(17): [ReferenceId(11), ReferenceId(49)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(16): [ReferenceId(28), ReferenceId(59),
  | ReferenceId(60)]
  | rebuilt        : SymbolId(18): [ReferenceId(34), ReferenceId(51)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(17): [ReferenceId(30), ReferenceId(61),
  | ReferenceId(62)]
  | rebuilt        : SymbolId(19): [ReferenceId(38), ReferenceId(53)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(18): [ReferenceId(32), ReferenceId(63),
  | ReferenceId(64)]
  | rebuilt        : SymbolId(20): [ReferenceId(42), ReferenceId(55)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(53): Some("_c")
  | rebuilt        : ReferenceId(44): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(55): Some("_c2")
  | rebuilt        : ReferenceId(46): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(57): Some("_c3")
  | rebuilt        : ReferenceId(48): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(59): Some("_c4")
  | rebuilt        : ReferenceId(50): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(61): Some("_c5")
  | rebuilt        : ReferenceId(52): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(63): Some("_c6")
  | rebuilt        : ReferenceId(54): None


* refresh/registers-identifiers-used-in-react-create-element-at-definition-site/input.jsx
  x Output mismatch
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(13): [ReferenceId(33), ReferenceId(45),
  | ReferenceId(46)]
  | rebuilt        : SymbolId(13): [ReferenceId(2), ReferenceId(46)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(14): [ReferenceId(35), ReferenceId(47),
  | ReferenceId(48)]
  | rebuilt        : SymbolId(14): [ReferenceId(5), ReferenceId(48)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(15): [ReferenceId(37), ReferenceId(49),
  | ReferenceId(50)]
  | rebuilt        : SymbolId(15): [ReferenceId(11), ReferenceId(50)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(16): [ReferenceId(39), ReferenceId(51),
  | ReferenceId(52)]
  | rebuilt        : SymbolId(16): [ReferenceId(33), ReferenceId(52)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(17): [ReferenceId(41), ReferenceId(53),
  | ReferenceId(54)]
  | rebuilt        : SymbolId(17): [ReferenceId(39), ReferenceId(54)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(18): [ReferenceId(43), ReferenceId(55),
  | ReferenceId(56)]
  | rebuilt        : SymbolId(18): [ReferenceId(43), ReferenceId(56)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(45): Some("_c")
  | rebuilt        : ReferenceId(45): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(47): Some("_c2")
  | rebuilt        : ReferenceId(47): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(49): Some("_c3")
  | rebuilt        : ReferenceId(49): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(51): Some("_c4")
  | rebuilt        : ReferenceId(51): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(53): Some("_c5")
  | rebuilt        : ReferenceId(53): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(55): Some("_c6")
  | rebuilt        : ReferenceId(55): None


* refresh/registers-likely-hocs-with-inline-functions-1/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(4): [ReferenceId(5), ReferenceId(18),
  | ReferenceId(19)]
  | rebuilt        : SymbolId(5): [ReferenceId(1), ReferenceId(19)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(6), ReferenceId(20),
  | ReferenceId(21)]
  | rebuilt        : SymbolId(6): [ReferenceId(3), ReferenceId(21)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(6): [ReferenceId(8), ReferenceId(22),
  | ReferenceId(23)]
  | rebuilt        : SymbolId(7): [ReferenceId(8), ReferenceId(23)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(7): [ReferenceId(9), ReferenceId(24),
  | ReferenceId(25)]
  | rebuilt        : SymbolId(8): [ReferenceId(6), ReferenceId(25)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(8): [ReferenceId(10), ReferenceId(26),
  | ReferenceId(27)]
  | rebuilt        : SymbolId(9): [ReferenceId(10), ReferenceId(27)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(9): [ReferenceId(12), ReferenceId(28),
  | ReferenceId(29)]
  | rebuilt        : SymbolId(10): [ReferenceId(16), ReferenceId(29)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(10): [ReferenceId(13), ReferenceId(30),
  | ReferenceId(31)]
  | rebuilt        : SymbolId(11): [ReferenceId(14), ReferenceId(31)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(11): [ReferenceId(14), ReferenceId(32),
  | ReferenceId(33)]
  | rebuilt        : SymbolId(12): [ReferenceId(12), ReferenceId(33)]

  x Reference flags mismatch:
  | after transform: ReferenceId(5): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(1): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(9): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(6): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(8): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(8): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(14): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(12): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(13): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(14): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(12): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(16): ReferenceFlags(Read | Write)

  x Reference symbol mismatch:
  | after transform: ReferenceId(18): Some("_c")
  | rebuilt        : ReferenceId(18): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(20): Some("_c2")
  | rebuilt        : ReferenceId(20): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(22): Some("_c3")
  | rebuilt        : ReferenceId(22): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(24): Some("_c4")
  | rebuilt        : ReferenceId(24): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(26): Some("_c5")
  | rebuilt        : ReferenceId(26): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(28): Some("_c6")
  | rebuilt        : ReferenceId(28): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(30): Some("_c7")
  | rebuilt        : ReferenceId(30): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(32): Some("_c8")
  | rebuilt        : ReferenceId(32): None


* refresh/registers-likely-hocs-with-inline-functions-2/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(2): [ReferenceId(2), ReferenceId(6),
  | ReferenceId(7)]
  | rebuilt        : SymbolId(3): [ReferenceId(4), ReferenceId(7)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(3): [ReferenceId(3), ReferenceId(8),
  | ReferenceId(9)]
  | rebuilt        : SymbolId(4): [ReferenceId(2), ReferenceId(9)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(4): [ReferenceId(4), ReferenceId(10),
  | ReferenceId(11)]
  | rebuilt        : SymbolId(5): [ReferenceId(0), ReferenceId(11)]

  x Reference flags mismatch:
  | after transform: ReferenceId(4): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(0): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(3): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(2): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(2): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(4): ReferenceFlags(Read | Write)

  x Reference symbol mismatch:
  | after transform: ReferenceId(6): Some("_c")
  | rebuilt        : ReferenceId(6): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(8): Some("_c2")
  | rebuilt        : ReferenceId(8): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(10): Some("_c3")
  | rebuilt        : ReferenceId(10): None


* refresh/registers-likely-hocs-with-inline-functions-3/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(3): [ReferenceId(2), ReferenceId(6),
  | ReferenceId(7)]
  | rebuilt        : SymbolId(4): [ReferenceId(4), ReferenceId(7)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(4): [ReferenceId(3), ReferenceId(8),
  | ReferenceId(9)]
  | rebuilt        : SymbolId(5): [ReferenceId(2), ReferenceId(9)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(4), ReferenceId(10),
  | ReferenceId(11)]
  | rebuilt        : SymbolId(6): [ReferenceId(0), ReferenceId(11)]

  x Reference flags mismatch:
  | after transform: ReferenceId(4): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(0): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(3): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(2): ReferenceFlags(Read | Write)

  x Reference flags mismatch:
  | after transform: ReferenceId(2): ReferenceFlags(Write)
  | rebuilt        : ReferenceId(4): ReferenceFlags(Read | Write)

  x Reference symbol mismatch:
  | after transform: ReferenceId(6): Some("_c")
  | rebuilt        : ReferenceId(6): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(8): Some("_c2")
  | rebuilt        : ReferenceId(8): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(10): Some("_c3")
  | rebuilt        : ReferenceId(10): None


* refresh/registers-top-level-exported-function-declarations/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(0): [ReferenceId(1), ReferenceId(5),
  | ReferenceId(11)]
  | rebuilt        : SymbolId(1): [ReferenceId(3), ReferenceId(5)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(7): [ReferenceId(4), ReferenceId(14),
  | ReferenceId(15)]
  | rebuilt        : SymbolId(8): [ReferenceId(2), ReferenceId(14)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(8): [ReferenceId(6), ReferenceId(16),
  | ReferenceId(17)]
  | rebuilt        : SymbolId(9): [ReferenceId(6), ReferenceId(16)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(9): [ReferenceId(8), ReferenceId(18),
  | ReferenceId(19)]
  | rebuilt        : SymbolId(10): [ReferenceId(9), ReferenceId(18)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(14): Some("_c")
  | rebuilt        : ReferenceId(13): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(16): Some("_c2")
  | rebuilt        : ReferenceId(15): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(18): Some("_c3")
  | rebuilt        : ReferenceId(17): None


* refresh/registers-top-level-exported-named-arrow-functions/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(0): [ReferenceId(1), ReferenceId(2),
  | ReferenceId(4), ReferenceId(8), ReferenceId(10)]
  | rebuilt        : SymbolId(1): [ReferenceId(3), ReferenceId(5),
  | ReferenceId(9)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(4): [ReferenceId(3), ReferenceId(12),
  | ReferenceId(13)]
  | rebuilt        : SymbolId(5): [ReferenceId(2), ReferenceId(11)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(5), ReferenceId(14),
  | ReferenceId(15)]
  | rebuilt        : SymbolId(6): [ReferenceId(6), ReferenceId(13)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(12): Some("_c")
  | rebuilt        : ReferenceId(10): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(14): Some("_c2")
  | rebuilt        : ReferenceId(12): None


* refresh/registers-top-level-function-declarations/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(0): [ReferenceId(1), ReferenceId(3),
  | ReferenceId(7)]
  | rebuilt        : SymbolId(1): [ReferenceId(3), ReferenceId(5)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(3): [ReferenceId(2), ReferenceId(9),
  | ReferenceId(10)]
  | rebuilt        : SymbolId(4): [ReferenceId(2), ReferenceId(9)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(4): [ReferenceId(4), ReferenceId(11),
  | ReferenceId(12)]
  | rebuilt        : SymbolId(5): [ReferenceId(6), ReferenceId(11)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(9): Some("_c")
  | rebuilt        : ReferenceId(8): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(11): Some("_c2")
  | rebuilt        : ReferenceId(10): None


* refresh/registers-top-level-variable-declarations-with-arrow-functions/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(0): [ReferenceId(1), ReferenceId(3),
  | ReferenceId(9)]
  | rebuilt        : SymbolId(1): [ReferenceId(3), ReferenceId(5)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(5): [ReferenceId(2), ReferenceId(12),
  | ReferenceId(13)]
  | rebuilt        : SymbolId(6): [ReferenceId(2), ReferenceId(12)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(6): [ReferenceId(4), ReferenceId(14),
  | ReferenceId(15)]
  | rebuilt        : SymbolId(7): [ReferenceId(6), ReferenceId(14)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(7): [ReferenceId(6), ReferenceId(16),
  | ReferenceId(17)]
  | rebuilt        : SymbolId(8): [ReferenceId(9), ReferenceId(16)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(12): Some("_c")
  | rebuilt        : ReferenceId(11): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(14): Some("_c2")
  | rebuilt        : ReferenceId(13): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(16): Some("_c3")
  | rebuilt        : ReferenceId(15): None


* refresh/registers-top-level-variable-declarations-with-function-expressions/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(0): [ReferenceId(1), ReferenceId(3),
  | ReferenceId(7)]
  | rebuilt        : SymbolId(1): [ReferenceId(3), ReferenceId(5)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(7): [ReferenceId(2), ReferenceId(9),
  | ReferenceId(10)]
  | rebuilt        : SymbolId(8): [ReferenceId(2), ReferenceId(9)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(8): [ReferenceId(4), ReferenceId(11),
  | ReferenceId(12)]
  | rebuilt        : SymbolId(9): [ReferenceId(6), ReferenceId(11)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(9): Some("_c")
  | rebuilt        : ReferenceId(8): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(11): Some("_c2")
  | rebuilt        : ReferenceId(10): None


* refresh/supports-typescript-namespace-syntax/input.tsx
  x Output mismatch
  x Missing SymbolId: Foo

  x Missing SymbolId: _Foo

  x Missing SymbolId: Bar

  x Missing SymbolId: _Bar

  x Missing ReferenceId: _Bar

  x Missing ReferenceId: _Bar

  x Missing ReferenceId: Bar

  x Missing ReferenceId: Bar

  x Missing ReferenceId: _Foo

  x Missing ReferenceId: _Foo

  x Missing ReferenceId: _Foo

  x Missing ReferenceId: _Foo

  x Missing ReferenceId: D

  x Missing SymbolId: NotExported

  x Missing SymbolId: _NotExported

  x Missing ReferenceId: _NotExported

  x Missing ReferenceId: NotExported

  x Missing ReferenceId: NotExported

  x Missing ReferenceId: Foo

  x Missing ReferenceId: Foo

  x Binding symbols mismatch:
  | after transform: ScopeId(0): [SymbolId(0)]
  | rebuilt        : ScopeId(0): [SymbolId(0)]

  x Binding symbols mismatch:
  | after transform: ScopeId(1): [SymbolId(1), SymbolId(5), SymbolId(6),
  | SymbolId(7), SymbolId(9)]
  | rebuilt        : ScopeId(1): [SymbolId(1), SymbolId(2), SymbolId(7),
  | SymbolId(8), SymbolId(9)]

  x Binding symbols mismatch:
  | after transform: ScopeId(2): [SymbolId(2), SymbolId(3), SymbolId(4),
  | SymbolId(10)]
  | rebuilt        : ScopeId(2): [SymbolId(3), SymbolId(4), SymbolId(5),
  | SymbolId(6)]

  x Binding symbols mismatch:
  | after transform: ScopeId(7): [SymbolId(8), SymbolId(11)]
  | rebuilt        : ScopeId(7): [SymbolId(10), SymbolId(11)]

  x Symbol flags mismatch:
  | after transform: SymbolId(2): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export | ArrowFunction)
  | rebuilt        : SymbolId(4): SymbolFlags(BlockScopedVariable |
  | ConstVariable)

  x Symbol flags mismatch:
  | after transform: SymbolId(3): SymbolFlags(BlockScopedVariable | Function)
  | rebuilt        : SymbolId(5): SymbolFlags(FunctionScopedVariable)

  x Symbol flags mismatch:
  | after transform: SymbolId(4): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export)
  | rebuilt        : SymbolId(6): SymbolFlags(BlockScopedVariable |
  | ConstVariable)

  x Symbol flags mismatch:
  | after transform: SymbolId(5): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export | ArrowFunction)
  | rebuilt        : SymbolId(7): SymbolFlags(BlockScopedVariable |
  | ConstVariable)

  x Symbol flags mismatch:
  | after transform: SymbolId(6): SymbolFlags(BlockScopedVariable | Export
  | | Function)
  | rebuilt        : SymbolId(8): SymbolFlags(FunctionScopedVariable)

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(6): []
  | rebuilt        : SymbolId(8): [ReferenceId(9)]

  x Symbol flags mismatch:
  | after transform: SymbolId(8): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export | ArrowFunction)
  | rebuilt        : SymbolId(11): SymbolFlags(BlockScopedVariable |
  | ConstVariable)


* refresh/uses-custom-identifiers-for-refresh-reg-and-refresh-sig/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(3): [ReferenceId(7), ReferenceId(8),
  | ReferenceId(10)]
  | rebuilt        : SymbolId(1): [ReferenceId(1), ReferenceId(6)]

  x Symbol reference IDs mismatch:
  | after transform: SymbolId(1): [ReferenceId(3), ReferenceId(11),
  | ReferenceId(12)]
  | rebuilt        : SymbolId(3): [ReferenceId(8), ReferenceId(11)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(7): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference symbol mismatch:
  | after transform: ReferenceId(11): Some("_c")
  | rebuilt        : ReferenceId(10): None


* refresh/uses-original-function-declaration-if-it-get-reassigned/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(1): [ReferenceId(3), ReferenceId(6),
  | ReferenceId(7)]
  | rebuilt        : SymbolId(2): [ReferenceId(1), ReferenceId(7)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(6): Some("_c")
  | rebuilt        : ReferenceId(6): None



