commit: d20b314c

Passed: 72/100

# All Passed:
* babel-plugin-transform-optional-catch-binding
* babel-plugin-transform-async-to-generator
* babel-plugin-transform-arrow-functions
* babel-preset-typescript
* babel-plugin-transform-react-jsx-source
* regexp


# babel-plugin-transform-class-static-block (3/5)
* contains-assignment/input.js
Reference flags mismatch for "a":
after transform: ReferenceId(0): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(0): ReferenceFlags(Write)
Reference flags mismatch for "b":
after transform: ReferenceId(1): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(1): ReferenceFlags(Write)
Reference flags mismatch for "c":
after transform: ReferenceId(2): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(2): ReferenceFlags(Write)
Reference flags mismatch for "i":
after transform: ReferenceId(6): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(6): ReferenceFlags(Write)

* nested-scopes/input.js
Reference flags mismatch for "x":
after transform: ReferenceId(0): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(0): ReferenceFlags(Write)


# babel-plugin-transform-nullish-coalescing-operator (0/2)
* invalid-variable-name/input.js
Reference flags mismatch for "_out$head$fooBarQux":
after transform: ReferenceId(2): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(0): ReferenceFlags(Write)

* transform-in-arrow-function-expression/input.js
Reference flags mismatch for "_a":
after transform: ReferenceId(2): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(0): ReferenceFlags(Write)


# babel-plugin-transform-async-generator-functions (1/3)
* for-await/with-if-statement/input.js
Reference flags mismatch for "_iteratorAbruptCompletion":
after transform: ReferenceId(4): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(6): ReferenceFlags(Write)
Reference flags mismatch for "_step":
after transform: ReferenceId(5): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(7): ReferenceFlags(Write)
Reference flags mismatch for "_iteratorAbruptCompletion":
after transform: ReferenceId(7): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(10): ReferenceFlags(Write)

* for-await/with-labeled-statement/input.js
Reference flags mismatch for "_iteratorAbruptCompletion":
after transform: ReferenceId(4): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(6): ReferenceFlags(Write)
Reference flags mismatch for "_step":
after transform: ReferenceId(5): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(7): ReferenceFlags(Write)
Reference flags mismatch for "_iteratorAbruptCompletion":
after transform: ReferenceId(7): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(10): ReferenceFlags(Write)


# babel-plugin-transform-object-rest-spread (3/5)
* object-rest/assignment-expression/input.js
Reference flags mismatch for "_c":
after transform: ReferenceId(29): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(2): ReferenceFlags(Write)
Reference flags mismatch for "b2":
after transform: ReferenceId(3): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(6): ReferenceFlags(Write)
Reference flags mismatch for "_c2":
after transform: ReferenceId(34): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(11): ReferenceFlags(Write)
Reference flags mismatch for "b2":
after transform: ReferenceId(7): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(15): ReferenceFlags(Write)
Reference flags mismatch for "_c3":
after transform: ReferenceId(39): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(20): ReferenceFlags(Write)
Reference flags mismatch for "b2":
after transform: ReferenceId(11): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(24): ReferenceFlags(Write)
Reference flags mismatch for "_c4":
after transform: ReferenceId(44): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(30): ReferenceFlags(Write)
Reference flags mismatch for "b3":
after transform: ReferenceId(16): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(34): ReferenceFlags(Write)
Reference flags mismatch for "_c5":
after transform: ReferenceId(49): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(40): ReferenceFlags(Write)
Reference flags mismatch for "b3":
after transform: ReferenceId(21): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(44): ReferenceFlags(Write)
Reference flags mismatch for "_c6":
after transform: ReferenceId(54): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(50): ReferenceFlags(Write)
Reference flags mismatch for "b3":
after transform: ReferenceId(26): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(54): ReferenceFlags(Write)

* object-rest/export/input.js
Symbol flags mismatch for "b0":
after transform: SymbolId(1): SymbolFlags(BlockScopedVariable | Export)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable)


# babel-plugin-transform-exponentiation-operator (1/5)
* assign-to-identifier/input.js
Reference flags mismatch for "_y":
after transform: ReferenceId(11): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(3): ReferenceFlags(Write)
Reference flags mismatch for "y":
after transform: ReferenceId(1): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(5): ReferenceFlags(Write)
Reference flags mismatch for "_z":
after transform: ReferenceId(15): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(8): ReferenceFlags(Write)
Reference flags mismatch for "z":
after transform: ReferenceId(2): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(10): ReferenceFlags(Write)
Reference flags mismatch for "_q":
after transform: ReferenceId(19): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(14): ReferenceFlags(Write)
Reference flags mismatch for "q":
after transform: ReferenceId(4): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(16): ReferenceFlags(Write)
Reference flags mismatch for "_unbound":
after transform: ReferenceId(25): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(23): ReferenceFlags(Write)
Reference flags mismatch for "___unbound":
after transform: ReferenceId(7): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(25): ReferenceFlags(Write)

* assign-to-member-expression/input.js
Reference flags mismatch for "_obj$foo$bar":
after transform: ReferenceId(48): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(6): ReferenceFlags(Write)
Reference flags mismatch for "_boundPropName":
after transform: ReferenceId(53): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(11): ReferenceFlags(Write)
Reference flags mismatch for "_unboundPropName":
after transform: ReferenceId(58): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(18): ReferenceFlags(Write)
Reference flags mismatch for "_obj$foo2$bar":
after transform: ReferenceId(62): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(25): ReferenceFlags(Write)
Reference flags mismatch for "_boundPropName2":
after transform: ReferenceId(65): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(27): ReferenceFlags(Write)
Reference flags mismatch for "_obj$foo3$bar":
after transform: ReferenceId(69): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(34): ReferenceFlags(Write)
Reference flags mismatch for "_unboundPropName2":
after transform: ReferenceId(72): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(36): ReferenceFlags(Write)
Reference flags mismatch for "_boundPropObj$foo$bar":
after transform: ReferenceId(77): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(43): ReferenceFlags(Write)
Reference flags mismatch for "_unboundPropObj$foo$b":
after transform: ReferenceId(82): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(50): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj":
after transform: ReferenceId(86): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(57): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj2":
after transform: ReferenceId(90): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(62): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj$foo$bar":
after transform: ReferenceId(94): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(67): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj3":
after transform: ReferenceId(98): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(72): ReferenceFlags(Write)
Reference flags mismatch for "_boundPropName3":
after transform: ReferenceId(101): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(74): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj4":
after transform: ReferenceId(105): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(81): ReferenceFlags(Write)
Reference flags mismatch for "_unboundPropName3":
after transform: ReferenceId(108): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(83): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj$foo2$bar":
after transform: ReferenceId(112): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(90): ReferenceFlags(Write)
Reference flags mismatch for "_boundPropName4":
after transform: ReferenceId(115): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(92): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj$foo3$bar":
after transform: ReferenceId(119): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(99): ReferenceFlags(Write)
Reference flags mismatch for "_unboundPropName4":
after transform: ReferenceId(122): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(101): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj5":
after transform: ReferenceId(126): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(108): ReferenceFlags(Write)
Reference flags mismatch for "_boundPropObj2$foo$ba":
after transform: ReferenceId(129): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(110): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj6":
after transform: ReferenceId(133): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(117): ReferenceFlags(Write)
Reference flags mismatch for "_unboundPropObj2$foo$":
after transform: ReferenceId(136): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(119): ReferenceFlags(Write)
Reference flags mismatch for "_fn":
after transform: ReferenceId(140): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(126): ReferenceFlags(Write)
Reference flags mismatch for "_fn$foo$bar":
after transform: ReferenceId(144): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(131): ReferenceFlags(Write)
Reference flags mismatch for "_fn$prop":
after transform: ReferenceId(148): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(136): ReferenceFlags(Write)
Reference flags mismatch for "_fn2":
after transform: ReferenceId(151): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(138): ReferenceFlags(Write)
Reference flags mismatch for "_fn$prop2":
after transform: ReferenceId(155): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(145): ReferenceFlags(Write)
Reference flags mismatch for "_ref":
after transform: ReferenceId(158): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(147): ReferenceFlags(Write)
Reference flags mismatch for "_this":
after transform: ReferenceId(162): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(154): ReferenceFlags(Write)
Reference flags mismatch for "_this$foo$bar":
after transform: ReferenceId(166): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(158): ReferenceFlags(Write)
Reference flags mismatch for "_this2":
after transform: ReferenceId(170): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(162): ReferenceFlags(Write)
Reference flags mismatch for "_this3":
after transform: ReferenceId(174): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(166): ReferenceFlags(Write)
Reference flags mismatch for "_fn4$foo$bar$qux":
after transform: ReferenceId(177): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(167): ReferenceFlags(Write)
Reference flags mismatch for "_this4":
after transform: ReferenceId(181): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(174): ReferenceFlags(Write)
Reference flags mismatch for "_this$foo$bar2":
after transform: ReferenceId(185): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(178): ReferenceFlags(Write)
Reference flags mismatch for "_this5":
after transform: ReferenceId(189): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(182): ReferenceFlags(Write)
Reference flags mismatch for "_this6":
after transform: ReferenceId(193): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(186): ReferenceFlags(Write)
Reference flags mismatch for "_fn4$foo$bar$qux2":
after transform: ReferenceId(196): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(187): ReferenceFlags(Write)
Reference flags mismatch for "_unbound":
after transform: ReferenceId(202): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(197): ReferenceFlags(Write)
Reference flags mismatch for "_bound":
after transform: ReferenceId(207): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(202): ReferenceFlags(Write)
Reference flags mismatch for "_unbound2":
after transform: ReferenceId(212): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(209): ReferenceFlags(Write)

* assign-used-result/input.js
Reference flags mismatch for "bound":
after transform: ReferenceId(1): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(1): ReferenceFlags(Write)
Reference flags mismatch for "_unbound":
after transform: ReferenceId(27): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(5): ReferenceFlags(Write)
Reference flags mismatch for "unbound":
after transform: ReferenceId(3): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(7): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj":
after transform: ReferenceId(32): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(15): ReferenceFlags(Write)
Reference flags mismatch for "_boundObj$foo$bar":
after transform: ReferenceId(36): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(21): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj$foo$bar":
after transform: ReferenceId(40): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(27): ReferenceFlags(Write)
Reference flags mismatch for "_boundProp":
after transform: ReferenceId(45): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(33): ReferenceFlags(Write)
Reference flags mismatch for "_unboundProp":
after transform: ReferenceId(50): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(41): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj2":
after transform: ReferenceId(54): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(49): ReferenceFlags(Write)
Reference flags mismatch for "_boundProp2":
after transform: ReferenceId(57): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(51): ReferenceFlags(Write)
Reference flags mismatch for "_unboundObj3":
after transform: ReferenceId(61): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(59): ReferenceFlags(Write)
Reference flags mismatch for "_unboundProp2":
after transform: ReferenceId(64): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(61): ReferenceFlags(Write)

* private-properties/input.js
Reference flags mismatch for "_this":
after transform: ReferenceId(3): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(0): ReferenceFlags(Write)
Reference flags mismatch for "_this$x$y$z":
after transform: ReferenceId(9): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(7): ReferenceFlags(Write)
Reference flags mismatch for "_obj$x$y$z":
after transform: ReferenceId(13): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(11): ReferenceFlags(Write)
Reference flags mismatch for "_fn$x$y$z":
after transform: ReferenceId(17): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(16): ReferenceFlags(Write)


# babel-plugin-transform-typescript (2/9)
* class-property-definition/input.ts
Unresolved references mismatch:
after transform: ["const"]
rebuilt        : []

* computed-constant-value/input.ts
Missing ReferenceId: "Infinity"
Missing ReferenceId: "Infinity"
Missing ReferenceId: "Infinity"
Missing ReferenceId: "Infinity"
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
Missing ReferenceId: "Foo"
Bindings mismatch:
after transform: ScopeId(1): ["Foo", "a", "b", "c"]
rebuilt        : ScopeId(1): ["Foo"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "Foo":
after transform: SymbolId(1): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(1): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(5): [ReferenceId(3), ReferenceId(4), ReferenceId(5), ReferenceId(6), ReferenceId(7), ReferenceId(8), ReferenceId(9)]
rebuilt        : SymbolId(2): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(4), ReferenceId(5), ReferenceId(6), ReferenceId(8)]

* export-elimination/input.ts
Missing SymbolId: "Name"
Missing SymbolId: "_Name"
Missing ReferenceId: "_Name"
Missing ReferenceId: "Name"
Missing ReferenceId: "Name"
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
Symbol flags mismatch for "Q":
after transform: SymbolId(8): SymbolFlags(BlockScopedVariable | ConstVariable | Export)
rebuilt        : SymbolId(7): SymbolFlags(BlockScopedVariable | ConstVariable)
Symbol flags mismatch for "T":
after transform: SymbolId(9): SymbolFlags(FunctionScopedVariable | Export | TypeAlias)
rebuilt        : SymbolId(8): SymbolFlags(FunctionScopedVariable | Export)
Symbol span mismatch for "T":
after transform: SymbolId(9): Span { start: 205, end: 206 }
rebuilt        : SymbolId(8): Span { start: 226, end: 227 }
Symbol reference IDs mismatch for "T":
after transform: SymbolId(9): [ReferenceId(8), ReferenceId(9)]
rebuilt        : SymbolId(8): [ReferenceId(9)]
Symbol redeclarations mismatch for "T":
after transform: SymbolId(9): [Span { start: 226, end: 227 }]
rebuilt        : SymbolId(8): []
Reference symbol mismatch for "Name":
after transform: SymbolId(7) "Name"
rebuilt        : SymbolId(5) "Name"

* redeclarations/input.ts
Scope children mismatch:
after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
rebuilt        : ScopeId(0): []
Symbol flags mismatch for "A":
after transform: SymbolId(0): SymbolFlags(BlockScopedVariable | ConstVariable | Export | Import)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable | ConstVariable | Export)
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
after transform: SymbolId(1): SymbolFlags(Export | Import | TypeAlias)
rebuilt        : SymbolId(1): SymbolFlags(Export | Import)
Symbol redeclarations mismatch for "T":
after transform: SymbolId(1): [Span { start: 170, end: 171 }]
rebuilt        : SymbolId(1): []
Symbol flags mismatch for "B":
after transform: SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable | Export | Import | TypeAlias)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable | Export)
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


# babel-plugin-transform-react-jsx (25/34)
* refresh/does-not-transform-it-because-it-is-not-used-in-the-AST/input.jsx
x Output mismatch

* refresh/react-refresh/can-handle-implicit-arrow-returns/input.jsx
Reference flags mismatch for "_c3":
after transform: ReferenceId(18): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(31): ReferenceFlags(Write)

* refresh/react-refresh/generates-signatures-for-function-expressions-calling-hooks/input.jsx
Reference flags mismatch for "_c2":
after transform: ReferenceId(18): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(4): ReferenceFlags(Write)
Reference flags mismatch for "_c":
after transform: ReferenceId(17): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(7): ReferenceFlags(Write)
Reference flags mismatch for "_c5":
after transform: ReferenceId(22): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(19): ReferenceFlags(Write)
Reference flags mismatch for "_c4":
after transform: ReferenceId(21): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(22): ReferenceFlags(Write)

* refresh/react-refresh/includes-custom-hooks-into-the-signatures-when-commonjs-target-is-used/input.jsx
x Output mismatch

* refresh/react-refresh/registers-capitalized-identifiers-in-hoc-calls/input.jsx
Reference flags mismatch for "_c2":
after transform: ReferenceId(8): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(3): ReferenceFlags(Write)

* refresh/react-refresh/registers-likely-hocs-with-inline-functions-1/input.jsx
Reference flags mismatch for "_c":
after transform: ReferenceId(5): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(1): ReferenceFlags(Write)
Reference flags mismatch for "_c4":
after transform: ReferenceId(9): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(6): ReferenceFlags(Write)
Reference flags mismatch for "_c3":
after transform: ReferenceId(8): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(8): ReferenceFlags(Write)
Reference flags mismatch for "_c8":
after transform: ReferenceId(14): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(12): ReferenceFlags(Write)
Reference flags mismatch for "_c7":
after transform: ReferenceId(13): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(14): ReferenceFlags(Write)
Reference flags mismatch for "_c6":
after transform: ReferenceId(12): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(16): ReferenceFlags(Write)

* refresh/react-refresh/registers-likely-hocs-with-inline-functions-2/input.jsx
Reference flags mismatch for "_c3":
after transform: ReferenceId(4): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(0): ReferenceFlags(Write)
Reference flags mismatch for "_c2":
after transform: ReferenceId(3): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(2): ReferenceFlags(Write)
Reference flags mismatch for "_c":
after transform: ReferenceId(2): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(4): ReferenceFlags(Write)

* refresh/react-refresh/registers-likely-hocs-with-inline-functions-3/input.jsx
Reference flags mismatch for "_c3":
after transform: ReferenceId(4): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(0): ReferenceFlags(Write)
Reference flags mismatch for "_c2":
after transform: ReferenceId(3): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(2): ReferenceFlags(Write)
Reference flags mismatch for "_c":
after transform: ReferenceId(2): ReferenceFlags(Read | Write)
rebuilt        : ReferenceId(4): ReferenceFlags(Write)

* refresh/react-refresh/supports-typescript-namespace-syntax/input.tsx
x Output mismatch


