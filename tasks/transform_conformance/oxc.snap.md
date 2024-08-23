commit: 12619ffe

Passed: 9/35

# All Passed:
* babel-plugin-transform-optional-catch-binding


# babel-plugin-transform-typescript (2/7)
* computed-constant-value/input.ts
  x Semantic Collector failed after transform

  x Missing ReferenceId: Infinity
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/computed-constant-value/input.ts:1:1]
 1 | enum A {
   : ^
 2 |   a = Infinity,
   `----

  x Missing ReferenceId: Infinity
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/computed-constant-value/input.ts:1:1]
 1 | enum A {
   : ^
 2 |   a = Infinity,
   `----

  x Missing ReferenceId: Infinity
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/computed-constant-value/input.ts:1:1]
 1 | enum A {
   : ^
 2 |   a = Infinity,
   `----

  x Missing ReferenceId: Infinity
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/computed-constant-value/input.ts:1:1]
 1 | enum A {
   : ^
 2 |   a = Infinity,
   `----


* elimination-declare/input.ts
  x Bindings mismatch:
  | after transform: ScopeId(0): ["A", "ReactiveMarkerSymbol"]
  | rebuilt        : ScopeId(0): []

  x Scope children mismatch:
  | after transform: ScopeId(0): [ScopeId(1)]
  | rebuilt        : ScopeId(0): []


* enum-member-reference/input.ts
  x Semantic Collector failed after transform

  x Missing ReferenceId: Foo
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/enum-member-reference/input.ts:1:1]
 1 | var x = 10;
   : ^
 2 | 
   `----


* export-elimination/input.ts
  x Semantic Collector failed after transform

  x Missing SymbolId: Name
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/export-elimination/input.ts:1:1]
 1 | import Im, {Ok} from 'a';
   : ^
 2 | class Foo {}
   `----

  x Missing SymbolId: _Name
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/export-elimination/input.ts:1:1]
 1 | import Im, {Ok} from 'a';
   : ^
 2 | class Foo {}
   `----

  x Missing ReferenceId: _Name
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/export-elimination/input.ts:1:1]
 1 | import Im, {Ok} from 'a';
   : ^
 2 | class Foo {}
   `----

  x Missing ReferenceId: Name
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/export-elimination/input.ts:1:1]
 1 | import Im, {Ok} from 'a';
   : ^
 2 | class Foo {}
   `----

  x Missing ReferenceId: Name
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/export-elimination/input.ts:1:1]
 1 | import Im, {Ok} from 'a';
   : ^
 2 | class Foo {}
   `----


* redeclarations/input.ts
  x Scope children mismatch:
  | after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
  | rebuilt        : ScopeId(0): []

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

  x Unresolved references mismatch:
  | after transform: ["X", "memo", "module", "useContext"]
  | rebuilt        : ["$RefreshReg$", "$RefreshSig$", "X", "memo", "module",
  | "useContext"]


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

  x Unresolved references mismatch:
  | after transform: ["foo", "gk", "require", "requireCond"]
  | rebuilt        : ["$RefreshReg$", "foo", "gk", "require", "requireCond"]


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

  x Unresolved references mismatch:
  | after transform: ["item", "useFoo"]
  | rebuilt        : ["$RefreshSig$", "item", "useFoo"]


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

  x Unresolved references mismatch:
  | after transform: ["React", "useState"]
  | rebuilt        : ["$RefreshReg$", "$RefreshSig$", "React", "useState"]


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

  x Unresolved references mismatch:
  | after transform: ["React", "ref", "useState"]
  | rebuilt        : ["$RefreshReg$", "$RefreshSig$", "React", "ref",
  | "useState"]


* refresh/generates-valid-signature-for-exotic-ways-to-call-hooks/input.jsx
  x Scope children mismatch:
  | after transform: ScopeId(0): [ScopeId(1)]
  | rebuilt        : ScopeId(0): [ScopeId(1), ScopeId(3)]

  x Bindings mismatch:
  | after transform: No scope
  | rebuilt        : ScopeId(3): []

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

  x Unresolved references mismatch:
  | after transform: ["React", "useFancyEffect", "useThePlatform"]
  | rebuilt        : ["$RefreshReg$", "$RefreshSig$", "React",
  | "useFancyEffect", "useThePlatform"]


* refresh/includes-custom-hooks-into-the-signatures/input.jsx
  x Scope children mismatch:
  | after transform: ScopeId(0): [ScopeId(1), ScopeId(2), ScopeId(4)]
  | rebuilt        : ScopeId(0): [ScopeId(1), ScopeId(2), ScopeId(3),
  | ScopeId(5), ScopeId(6)]

  x Bindings mismatch:
  | after transform: No scope
  | rebuilt        : ScopeId(2): []

  x Bindings mismatch:
  | after transform: No scope
  | rebuilt        : ScopeId(6): []

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

  x Unresolved references mismatch:
  | after transform: ["React"]
  | rebuilt        : ["$RefreshReg$", "$RefreshSig$", "React"]


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

  x Unresolved references mismatch:
  | after transform: ["hoc"]
  | rebuilt        : ["$RefreshReg$", "hoc"]


* refresh/registers-identifiers-used-in-jsx-at-definition-site/input.jsx
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

  x Unresolved references mismatch:
  | after transform: ["funny", "hoc", "styled", "wow"]
  | rebuilt        : ["$RefreshReg$", "funny", "hoc", "styled", "wow"]


* refresh/registers-identifiers-used-in-react-create-element-at-definition-site/input.jsx
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

  x Unresolved references mismatch:
  | after transform: ["React", "funny", "hoc", "jsx", "styled", "wow"]
  | rebuilt        : ["$RefreshReg$", "React", "funny", "hoc", "jsx",
  | "styled", "wow"]


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

  x Unresolved references mismatch:
  | after transform: ["React", "forwardRef", "memo"]
  | rebuilt        : ["$RefreshReg$", "React", "forwardRef", "memo"]


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

  x Unresolved references mismatch:
  | after transform: ["React", "forwardRef"]
  | rebuilt        : ["$RefreshReg$", "React", "forwardRef"]


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

  x Unresolved references mismatch:
  | after transform: ["React", "forwardRef"]
  | rebuilt        : ["$RefreshReg$", "React", "forwardRef"]


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

  x Unresolved references mismatch:
  | after transform: []
  | rebuilt        : ["$RefreshReg$"]


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

  x Unresolved references mismatch:
  | after transform: []
  | rebuilt        : ["$RefreshReg$"]


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

  x Unresolved references mismatch:
  | after transform: []
  | rebuilt        : ["$RefreshReg$"]


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

  x Unresolved references mismatch:
  | after transform: []
  | rebuilt        : ["$RefreshReg$"]


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

  x Unresolved references mismatch:
  | after transform: []
  | rebuilt        : ["$RefreshReg$"]


* refresh/supports-typescript-namespace-syntax/input.tsx
  x Semantic Collector failed after transform

  x Missing SymbolId: Foo
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing SymbolId: _Foo
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing SymbolId: Bar
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing SymbolId: _Bar
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: _Bar
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: _Bar
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: Bar
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: Bar
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: _Foo
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: _Foo
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: _Foo
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: _Foo
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: D
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing SymbolId: NotExported
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing SymbolId: _NotExported
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: _NotExported
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: NotExported
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: NotExported
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: Foo
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----

  x Missing ReferenceId: Foo
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/refresh/supports-typescript-namespace-syntax/input.tsx:1:1]
 1 | namespace Foo {
   : ^
 2 |   export namespace Bar {
   `----


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

  x Unresolved references mismatch:
  | after transform: ["Foo", "X", "useContext"]
  | rebuilt        : ["Foo", "X", "import.meta.refreshReg",
  | "import.meta.refreshSig", "useContext"]

  x Unresolved reference IDs mismatch for "Foo":
  | after transform: [ReferenceId(2), ReferenceId(5)]
  | rebuilt        : [ReferenceId(5)]


* refresh/uses-original-function-declaration-if-it-get-reassigned/input.jsx
  x Symbol reference IDs mismatch:
  | after transform: SymbolId(1): [ReferenceId(3), ReferenceId(6),
  | ReferenceId(7)]
  | rebuilt        : SymbolId(2): [ReferenceId(1), ReferenceId(7)]

  x Reference symbol mismatch:
  | after transform: ReferenceId(6): Some("_c")
  | rebuilt        : ReferenceId(6): None

  x Unresolved references mismatch:
  | after transform: ["connect"]
  | rebuilt        : ["$RefreshReg$", "connect"]



