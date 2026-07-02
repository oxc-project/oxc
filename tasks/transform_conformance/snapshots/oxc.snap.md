commit: 1fb0b771

Passed: 210/397

# All Passed:
* babel-plugin-transform-class-static-block
* babel-plugin-transform-private-methods
* babel-plugin-transform-optional-catch-binding
* babel-plugin-transform-arrow-functions
* babel-preset-typescript
* babel-plugin-transform-react-jsx-self
* babel-plugin-transform-react-jsx-source
* regexp
* plugin-tagged-template-transform


# babel-plugin-transform-explicit-resource-management (1/4)
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

* function-with-scopes-in-params/input.js
Bindings mismatch:
after transform: ScopeId(6): ["x", "y"]
rebuilt        : ScopeId(6): []
Bindings mismatch:
after transform: ScopeId(9): []
rebuilt        : ScopeId(7): ["x", "y"]
Scope parent mismatch:
after transform: ScopeId(9): Some(ScopeId(1))
rebuilt        : ScopeId(7): Some(ScopeId(6))
Scope parent mismatch:
after transform: ScopeId(10): Some(ScopeId(1))
rebuilt        : ScopeId(10): Some(ScopeId(6))
Scope parent mismatch:
after transform: ScopeId(12): Some(ScopeId(1))
rebuilt        : ScopeId(12): Some(ScopeId(6))
Symbol scope ID mismatch for "x":
after transform: SymbolId(3): ScopeId(6)
rebuilt        : SymbolId(4): ScopeId(7)
Symbol scope ID mismatch for "y":
after transform: SymbolId(4): ScopeId(6)
rebuilt        : SymbolId(5): ScopeId(7)

* try-catch/input.js
Bindings mismatch:
after transform: ScopeId(2): []
rebuilt        : ScopeId(2): ["_usingCtx"]
Bindings mismatch:
after transform: ScopeId(3): ["_usingCtx"]
rebuilt        : ScopeId(3): []
Symbol scope ID mismatch for "_usingCtx":
after transform: SymbolId(2): ScopeId(3)
rebuilt        : SymbolId(1): ScopeId(2)


# babel-plugin-transform-class-properties (16/33)
* instance-prop-initializer-var-clash/input.js
Scope parent mismatch:
after transform: ScopeId(2): Some(ScopeId(6))
rebuilt        : ScopeId(4): Some(ScopeId(3))
Scope parent mismatch:
after transform: ScopeId(4): Some(ScopeId(6))
rebuilt        : ScopeId(6): Some(ScopeId(3))

* interaction-with-other-transforms/input.js
Bindings mismatch:
after transform: ScopeId(7): []
rebuilt        : ScopeId(2): ["_c"]
Bindings mismatch:
after transform: ScopeId(8): ["_c"]
rebuilt        : ScopeId(3): []
Bindings mismatch:
after transform: ScopeId(10): ["_args"]
rebuilt        : ScopeId(9): ["_args", "_k"]
Scope parent mismatch:
after transform: ScopeId(10): Some(ScopeId(4))
rebuilt        : ScopeId(9): Some(ScopeId(8))
Bindings mismatch:
after transform: ScopeId(11): ["_k"]
rebuilt        : ScopeId(10): []
Symbol scope ID mismatch for "_c":
after transform: SymbolId(3): ScopeId(8)
rebuilt        : SymbolId(6): ScopeId(2)
Symbol scope ID mismatch for "_k":
after transform: SymbolId(11): ScopeId(11)
rebuilt        : SymbolId(11): ScopeId(9)

* private-field-resolve-to-method/input.js
x Output mismatch

* private-field-resolve-to-method-in-computed-key/input.js
x Output mismatch

* private-logical-assignment/input.js
Bindings mismatch:
after transform: ScopeId(4): []
rebuilt        : ScopeId(6): ["_this$self"]
Bindings mismatch:
after transform: ScopeId(5): ["_this$self"]
rebuilt        : ScopeId(7): []
Symbol scope ID mismatch for "_this$self":
after transform: SymbolId(4): ScopeId(5)
rebuilt        : SymbolId(4): ScopeId(6)

* private-optional-member-with-sequence/input.js
Bindings mismatch:
after transform: ScopeId(2): []
rebuilt        : ScopeId(4): ["_ref"]
Bindings mismatch:
after transform: ScopeId(3): ["_ref"]
rebuilt        : ScopeId(5): []
Symbol scope ID mismatch for "_ref":
after transform: SymbolId(2): ScopeId(3)
rebuilt        : SymbolId(1): ScopeId(4)

* static-async-super/input.js
Scope parent mismatch:
after transform: ScopeId(6): Some(ScopeId(4))
rebuilt        : ScopeId(4): Some(ScopeId(3))

* static-super-assignment-target/input.js
x Output mismatch

* static-super-tagged-template/input.js
x Output mismatch

* super-in-constructor-missing/input.js
Scope parent mismatch:
after transform: ScopeId(4): Some(ScopeId(2))
rebuilt        : ScopeId(4): Some(ScopeId(3))

* super-in-constructor-nested/input.js
Scope parent mismatch:
after transform: ScopeId(15): Some(ScopeId(11))
rebuilt        : ScopeId(13): Some(ScopeId(12))

* super-in-constructor-nested-super/input.js
Scope parent mismatch:
after transform: ScopeId(5): Some(ScopeId(3))
rebuilt        : ScopeId(5): Some(ScopeId(4))

* super-in-constructor-strict/input.js
Bindings mismatch:
after transform: ScopeId(1): ["_super"]
rebuilt        : ScopeId(1): []
Bindings mismatch:
after transform: ScopeId(2): ["C"]
rebuilt        : ScopeId(2): ["C", "_super"]
Scope parent mismatch:
after transform: ScopeId(11): Some(ScopeId(1))
rebuilt        : ScopeId(3): Some(ScopeId(2))
Bindings mismatch:
after transform: ScopeId(6): ["_super2"]
rebuilt        : ScopeId(8): []
Bindings mismatch:
after transform: ScopeId(7): ["C"]
rebuilt        : ScopeId(9): ["C", "_super2"]
Scope parent mismatch:
after transform: ScopeId(13): Some(ScopeId(6))
rebuilt        : ScopeId(10): Some(ScopeId(9))
Symbol scope ID mismatch for "_super":
after transform: SymbolId(6): ScopeId(1)
rebuilt        : SymbolId(1): ScopeId(2)
Symbol scope ID mismatch for "_super2":
after transform: SymbolId(7): ScopeId(6)
rebuilt        : SymbolId(5): ScopeId(9)

* this-in-computed-key/input.js
Bindings mismatch:
after transform: ScopeId(1): ["_ref", "_this"]
rebuilt        : ScopeId(1): []
Bindings mismatch:
after transform: ScopeId(2): ["C"]
rebuilt        : ScopeId(2): ["C", "_ref", "_this"]
Bindings mismatch:
after transform: ScopeId(4): ["_ref2", "_this2"]
rebuilt        : ScopeId(6): []
Bindings mismatch:
after transform: ScopeId(5): []
rebuilt        : ScopeId(7): ["_ref2", "_this2"]
Symbol scope ID mismatch for "_this":
after transform: SymbolId(5): ScopeId(1)
rebuilt        : SymbolId(1): ScopeId(2)
Symbol scope ID mismatch for "_ref":
after transform: SymbolId(6): ScopeId(1)
rebuilt        : SymbolId(2): ScopeId(2)
Symbol scope ID mismatch for "_this2":
after transform: SymbolId(7): ScopeId(4)
rebuilt        : SymbolId(5): ScopeId(7)
Symbol scope ID mismatch for "_ref2":
after transform: SymbolId(8): ScopeId(4)
rebuilt        : SymbolId(6): ScopeId(7)

* typescript/declare-computed-keys/input.ts
Symbol reference IDs mismatch for "KEY1":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2)]
rebuilt        : SymbolId(1): []

* typescript/optional-call/input.ts
Bindings mismatch:
after transform: ScopeId(4): []
rebuilt        : ScopeId(2): ["_o$X", "_o$X2", "_o$X3"]
Bindings mismatch:
after transform: ScopeId(5): ["_o$X", "_o$X2", "_o$X3", "o"]
rebuilt        : ScopeId(3): ["o"]
Symbol reference IDs mismatch for "X":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(6), ReferenceId(11), ReferenceId(16)]
rebuilt        : SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(8), ReferenceId(14)]
Symbol scope ID mismatch for "_o$X":
after transform: SymbolId(3): ScopeId(5)
rebuilt        : SymbolId(1): ScopeId(2)
Symbol scope ID mismatch for "_o$X2":
after transform: SymbolId(4): ScopeId(5)
rebuilt        : SymbolId(2): ScopeId(2)
Symbol scope ID mismatch for "_o$X3":
after transform: SymbolId(5): ScopeId(5)
rebuilt        : SymbolId(3): ScopeId(2)

* typescript/optional-member/input.ts
Symbol reference IDs mismatch for "X":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(6), ReferenceId(9), ReferenceId(12)]
rebuilt        : SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(6), ReferenceId(10)]


# babel-plugin-transform-logical-assignment-operators (5/6)
* super-prop-computed/input.js
Bindings mismatch:
after transform: ScopeId(2): []
rebuilt        : ScopeId(2): ["_mutatedProp", "_unboundProp"]
Bindings mismatch:
after transform: ScopeId(3): ["_mutatedProp", "_unboundProp"]
rebuilt        : ScopeId(3): []
Symbol scope ID mismatch for "_unboundProp":
after transform: SymbolId(3): ScopeId(3)
rebuilt        : SymbolId(3): ScopeId(2)
Symbol scope ID mismatch for "_mutatedProp":
after transform: SymbolId(4): ScopeId(3)
rebuilt        : SymbolId(4): ScopeId(2)


# babel-plugin-transform-nullish-coalescing-operator (2/3)
* transform-in-arrow-function-expression/input.js
Bindings mismatch:
after transform: ScopeId(1): []
rebuilt        : ScopeId(1): ["_a"]
Bindings mismatch:
after transform: ScopeId(2): ["_a"]
rebuilt        : ScopeId(2): []
Symbol scope ID mismatch for "_a":
after transform: SymbolId(0): ScopeId(2)
rebuilt        : SymbolId(0): ScopeId(1)


# babel-plugin-transform-optional-chaining (2/3)
* oxc/keep-this/input.ts
Bindings mismatch:
after transform: ScopeId(2): []
rebuilt        : ScopeId(2): ["_f", "_ref", "_this$f", "_this$x", "_this$x$f", "_this$x$f2", "_this$x$y", "_this$x$y$f", "_this$x$y2", "_this$x$y2$f", "_this$x2", "_this$x2$f", "_this$x3", "_this$x3$y", "_this$x3$y$f", "_this$x4", "_this$x4$f", "_this$x5"]
Bindings mismatch:
after transform: ScopeId(3): ["_f", "_ref", "_this$f", "_this$x", "_this$x$f", "_this$x$f2", "_this$x$y", "_this$x$y$f", "_this$x$y2", "_this$x$y2$f", "_this$x2", "_this$x2$f", "_this$x3", "_this$x3$y", "_this$x3$y$f", "_this$x4", "_this$x4$f", "_this$x5"]
rebuilt        : ScopeId(3): []
Symbol scope ID mismatch for "_this$f":
after transform: SymbolId(2): ScopeId(3)
rebuilt        : SymbolId(19): ScopeId(2)
Symbol scope ID mismatch for "_this$x$f":
after transform: SymbolId(3): ScopeId(3)
rebuilt        : SymbolId(20): ScopeId(2)
Symbol scope ID mismatch for "_this$x":
after transform: SymbolId(4): ScopeId(3)
rebuilt        : SymbolId(21): ScopeId(2)
Symbol scope ID mismatch for "_this$x$y$f":
after transform: SymbolId(5): ScopeId(3)
rebuilt        : SymbolId(22): ScopeId(2)
Symbol scope ID mismatch for "_this$x$y":
after transform: SymbolId(6): ScopeId(3)
rebuilt        : SymbolId(23): ScopeId(2)
Symbol scope ID mismatch for "_this$x2":
after transform: SymbolId(7): ScopeId(3)
rebuilt        : SymbolId(24): ScopeId(2)
Symbol scope ID mismatch for "_this$x2$f":
after transform: SymbolId(8): ScopeId(3)
rebuilt        : SymbolId(25): ScopeId(2)
Symbol scope ID mismatch for "_this$x3":
after transform: SymbolId(9): ScopeId(3)
rebuilt        : SymbolId(26): ScopeId(2)
Symbol scope ID mismatch for "_this$x3$y":
after transform: SymbolId(10): ScopeId(3)
rebuilt        : SymbolId(27): ScopeId(2)
Symbol scope ID mismatch for "_this$x3$y$f":
after transform: SymbolId(11): ScopeId(3)
rebuilt        : SymbolId(28): ScopeId(2)
Symbol scope ID mismatch for "_this$x$y2":
after transform: SymbolId(12): ScopeId(3)
rebuilt        : SymbolId(29): ScopeId(2)
Symbol scope ID mismatch for "_this$x$y2$f":
after transform: SymbolId(13): ScopeId(3)
rebuilt        : SymbolId(30): ScopeId(2)
Symbol scope ID mismatch for "_this$x4":
after transform: SymbolId(14): ScopeId(3)
rebuilt        : SymbolId(31): ScopeId(2)
Symbol scope ID mismatch for "_this$x4$f":
after transform: SymbolId(15): ScopeId(3)
rebuilt        : SymbolId(32): ScopeId(2)
Symbol scope ID mismatch for "_this$x$f2":
after transform: SymbolId(16): ScopeId(3)
rebuilt        : SymbolId(33): ScopeId(2)
Symbol scope ID mismatch for "_this$x5":
after transform: SymbolId(17): ScopeId(3)
rebuilt        : SymbolId(34): ScopeId(2)
Symbol scope ID mismatch for "_f":
after transform: SymbolId(18): ScopeId(3)
rebuilt        : SymbolId(35): ScopeId(2)
Symbol scope ID mismatch for "_ref":
after transform: SymbolId(19): ScopeId(3)
rebuilt        : SymbolId(36): ScopeId(2)


# babel-plugin-transform-async-generator-functions (1/4)
* for-await/single-statement-body/input.js
Bindings mismatch:
after transform: ScopeId(1): ["asyncIterable"]
rebuilt        : ScopeId(5): ["_didIteratorError", "_iterator", "_iteratorAbruptCompletion", "_iteratorError", "_step", "asyncIterable"]
Bindings mismatch:
after transform: ScopeId(2): ["_didIteratorError", "_iterator", "_iteratorAbruptCompletion", "_iteratorError", "_step"]
rebuilt        : ScopeId(6): []
Symbol scope ID mismatch for "_iteratorAbruptCompletion":
after transform: SymbolId(5): ScopeId(2)
rebuilt        : SymbolId(4): ScopeId(5)
Symbol scope ID mismatch for "_didIteratorError":
after transform: SymbolId(4): ScopeId(2)
rebuilt        : SymbolId(5): ScopeId(5)
Symbol scope ID mismatch for "_iteratorError":
after transform: SymbolId(6): ScopeId(2)
rebuilt        : SymbolId(6): ScopeId(5)
Symbol scope ID mismatch for "_iterator":
after transform: SymbolId(7): ScopeId(2)
rebuilt        : SymbolId(7): ScopeId(5)
Symbol scope ID mismatch for "_step":
after transform: SymbolId(3): ScopeId(2)
rebuilt        : SymbolId(8): ScopeId(5)

* for-await/with-if-statement/input.js
Bindings mismatch:
after transform: ScopeId(1): ["asyncIterable"]
rebuilt        : ScopeId(5): ["_didIteratorError", "_iterator", "_iteratorAbruptCompletion", "_iteratorError", "_step", "asyncIterable"]
Bindings mismatch:
after transform: ScopeId(2): ["_didIteratorError", "_iterator", "_iteratorAbruptCompletion", "_iteratorError", "_step"]
rebuilt        : ScopeId(6): []
Symbol scope ID mismatch for "_iteratorAbruptCompletion":
after transform: SymbolId(5): ScopeId(2)
rebuilt        : SymbolId(4): ScopeId(5)
Symbol scope ID mismatch for "_didIteratorError":
after transform: SymbolId(4): ScopeId(2)
rebuilt        : SymbolId(5): ScopeId(5)
Symbol scope ID mismatch for "_iteratorError":
after transform: SymbolId(6): ScopeId(2)
rebuilt        : SymbolId(6): ScopeId(5)
Symbol scope ID mismatch for "_iterator":
after transform: SymbolId(7): ScopeId(2)
rebuilt        : SymbolId(7): ScopeId(5)
Symbol scope ID mismatch for "_step":
after transform: SymbolId(3): ScopeId(2)
rebuilt        : SymbolId(8): ScopeId(5)

* for-await/with-labeled-statement/input.js
Bindings mismatch:
after transform: ScopeId(1): ["asyncIterable"]
rebuilt        : ScopeId(5): ["_didIteratorError", "_iterator", "_iteratorAbruptCompletion", "_iteratorError", "_step", "asyncIterable"]
Bindings mismatch:
after transform: ScopeId(2): ["_didIteratorError", "_iterator", "_iteratorAbruptCompletion", "_iteratorError", "_step"]
rebuilt        : ScopeId(6): []
Symbol scope ID mismatch for "_iteratorAbruptCompletion":
after transform: SymbolId(5): ScopeId(2)
rebuilt        : SymbolId(4): ScopeId(5)
Symbol scope ID mismatch for "_didIteratorError":
after transform: SymbolId(4): ScopeId(2)
rebuilt        : SymbolId(5): ScopeId(5)
Symbol scope ID mismatch for "_iteratorError":
after transform: SymbolId(6): ScopeId(2)
rebuilt        : SymbolId(6): ScopeId(5)
Symbol scope ID mismatch for "_iterator":
after transform: SymbolId(7): ScopeId(2)
rebuilt        : SymbolId(7): ScopeId(5)
Symbol scope ID mismatch for "_step":
after transform: SymbolId(3): ScopeId(2)
rebuilt        : SymbolId(8): ScopeId(5)


# babel-plugin-transform-object-rest-spread (7/8)
* object-rest/with-arrow-function-expression/input.js
Bindings mismatch:
after transform: ScopeId(1): ["_ref", "args"]
rebuilt        : ScopeId(1): ["_ref"]
Bindings mismatch:
after transform: ScopeId(2): []
rebuilt        : ScopeId(2): ["args"]
Symbol scope ID mismatch for "args":
after transform: SymbolId(1): ScopeId(1)
rebuilt        : SymbolId(2): ScopeId(2)


# babel-plugin-transform-async-to-generator (22/28)
* class/property-definition/input.js
Scope parent mismatch:
after transform: ScopeId(16): Some(ScopeId(14))
rebuilt        : ScopeId(4): Some(ScopeId(3))
Scope parent mismatch:
after transform: ScopeId(20): Some(ScopeId(18))
rebuilt        : ScopeId(10): Some(ScopeId(9))
Scope parent mismatch:
after transform: ScopeId(22): Some(ScopeId(6))
rebuilt        : ScopeId(16): Some(ScopeId(15))
Scope parent mismatch:
after transform: ScopeId(24): Some(ScopeId(10))
rebuilt        : ScopeId(22): Some(ScopeId(21))

* super/assign/input.js
Scope parent mismatch:
after transform: ScopeId(7): Some(ScopeId(5))
rebuilt        : ScopeId(3): Some(ScopeId(2))
Scope parent mismatch:
after transform: ScopeId(9): Some(ScopeId(5))
rebuilt        : ScopeId(5): Some(ScopeId(2))
Scope parent mismatch:
after transform: ScopeId(11): Some(ScopeId(5))
rebuilt        : ScopeId(7): Some(ScopeId(2))

* super/computed-member/input.js
Scope parent mismatch:
after transform: ScopeId(8): Some(ScopeId(6))
rebuilt        : ScopeId(5): Some(ScopeId(4))

* super/nested/input.js
Scope parent mismatch:
after transform: ScopeId(19): Some(ScopeId(17))
rebuilt        : ScopeId(3): Some(ScopeId(2))
Scope parent mismatch:
after transform: ScopeId(15): Some(ScopeId(13))
rebuilt        : ScopeId(13): Some(ScopeId(12))

* super/nested-class/input.js
Scope parent mismatch:
after transform: ScopeId(22): Some(ScopeId(20))
rebuilt        : ScopeId(5): Some(ScopeId(4))
Scope parent mismatch:
after transform: ScopeId(18): Some(ScopeId(16))
rebuilt        : ScopeId(16): Some(ScopeId(15))

* super/property/input.js
Scope parent mismatch:
after transform: ScopeId(8): Some(ScopeId(6))
rebuilt        : ScopeId(4): Some(ScopeId(3))
Scope parent mismatch:
after transform: ScopeId(12): Some(ScopeId(10))
rebuilt        : ScopeId(10): Some(ScopeId(9))


# babel-plugin-transform-exponentiation-operator (3/5)
* assign-to-member-expression/input.js
Bindings mismatch:
after transform: ScopeId(1): []
rebuilt        : ScopeId(1): ["_fn4$foo$bar$qux2", "_this$foo$bar2", "_this4", "_this5", "_this6"]
Bindings mismatch:
after transform: ScopeId(2): ["_fn4$foo$bar$qux2", "_this$foo$bar2", "_this4", "_this5", "_this6"]
rebuilt        : ScopeId(2): []
Symbol scope ID mismatch for "_this4":
after transform: SymbolId(46): ScopeId(2)
rebuilt        : SymbolId(48): ScopeId(1)
Symbol scope ID mismatch for "_this$foo$bar2":
after transform: SymbolId(47): ScopeId(2)
rebuilt        : SymbolId(49): ScopeId(1)
Symbol scope ID mismatch for "_this5":
after transform: SymbolId(48): ScopeId(2)
rebuilt        : SymbolId(50): ScopeId(1)
Symbol scope ID mismatch for "_this6":
after transform: SymbolId(49): ScopeId(2)
rebuilt        : SymbolId(51): ScopeId(1)
Symbol scope ID mismatch for "_fn4$foo$bar$qux2":
after transform: SymbolId(50): ScopeId(2)
rebuilt        : SymbolId(52): ScopeId(1)

* private-properties/input.js
Bindings mismatch:
after transform: ScopeId(2): ["obj"]
rebuilt        : ScopeId(2): ["_fn$x$y$z", "_obj$x$y$z", "_this", "_this$x$y$z", "obj"]
Bindings mismatch:
after transform: ScopeId(3): ["_fn$x$y$z", "_obj$x$y$z", "_this", "_this$x$y$z"]
rebuilt        : ScopeId(3): []
Symbol scope ID mismatch for "_this":
after transform: SymbolId(2): ScopeId(3)
rebuilt        : SymbolId(2): ScopeId(2)
Symbol scope ID mismatch for "_this$x$y$z":
after transform: SymbolId(3): ScopeId(3)
rebuilt        : SymbolId(3): ScopeId(2)
Symbol scope ID mismatch for "_obj$x$y$z":
after transform: SymbolId(4): ScopeId(3)
rebuilt        : SymbolId(4): ScopeId(2)
Symbol scope ID mismatch for "_fn$x$y$z":
after transform: SymbolId(5): ScopeId(3)
rebuilt        : SymbolId(5): ScopeId(2)


# babel-plugin-transform-typescript (23/60)
* allow-declare-fields-false/input.ts
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

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
rebuilt        : ScopeId(3): ["B"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["C", "a", "b", "c"]
rebuilt        : ScopeId(5): ["C"]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(5): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(4): ["D", "a", "b", "c"]
rebuilt        : ScopeId(7): ["D"]
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(0x0)
rebuilt        : ScopeId(7): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "Phase":
after transform: SymbolId(0): SymbolFlags(ConstEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* const-enum-value-ref-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Phase", "one", "two"]
rebuilt        : ScopeId(1): ["Phase"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["Merge", "x"]
rebuilt        : ScopeId(3): ["Merge"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["Merge", "y"]
rebuilt        : ScopeId(5): ["Merge"]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(5): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(4): ["NestOuter", "a", "b"]
rebuilt        : ScopeId(7): ["NestOuter"]
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(0x0)
rebuilt        : ScopeId(7): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(7): ["NestInner", "a", "b"]
rebuilt        : ScopeId(11): ["NestInner"]
Scope flags mismatch:
after transform: ScopeId(7): ScopeFlags(0x0)
rebuilt        : ScopeId(11): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["Animal", "CAT", "DOG"]
rebuilt        : ScopeId(3): ["Animal"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["AnimalSize", "LARGE_DOG", "SMALL_CAT"]
rebuilt        : ScopeId(5): ["AnimalSize"]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(5): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["COMPUTED_1", "COMPUTED_2", "ComputedEnum"]
rebuilt        : ScopeId(3): ["ComputedEnum"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["C", "ComputedEnum", "D"]
rebuilt        : ScopeId(3): ["ComputedEnum"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(6): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(4): ScopeFlags(Function)
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
Symbol reference IDs mismatch for "T":
after transform: SymbolId(9): [ReferenceId(8), ReferenceId(9)]
rebuilt        : SymbolId(8): [ReferenceId(9)]
Symbol redeclarations mismatch for "T":
after transform: SymbolId(9): [Span { start: 205, end: 206 }, Span { start: 226, end: 227 }]
rebuilt        : SymbolId(8): []

* namespace/export-import-=/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["_N"]
rebuilt        : ScopeId(1): ["X", "_N"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["X"]
rebuilt        : ScopeId(2): []
Symbol flags mismatch for "N1":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N1":
after transform: SymbolId(1): Span { start: 31, end: 33 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol scope ID mismatch for "X":
after transform: SymbolId(2): ScopeId(2)
rebuilt        : SymbolId(3): ScopeId(1)

* namespace/import-=/input.ts
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["_N2"]
rebuilt        : ScopeId(3): ["X", "_N2"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(4): ["V", "X"]
rebuilt        : ScopeId(4): ["V"]
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
Symbol scope ID mismatch for "X":
after transform: SymbolId(5): ScopeId(4)
rebuilt        : SymbolId(6): ScopeId(3)

* namespace/preserve-import-=/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["_N"]
rebuilt        : ScopeId(1): ["Foo", "_N"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["Foo", "foo"]
rebuilt        : ScopeId(2): ["foo"]
Bindings mismatch:
after transform: ScopeId(2): ["_N2"]
rebuilt        : ScopeId(3): ["Foo", "_N2"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(4): ["Foo", "foo"]
rebuilt        : ScopeId(4): ["foo"]
Symbol flags mismatch for "N1":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N1":
after transform: SymbolId(1): Span { start: 34, end: 36 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol scope ID mismatch for "Foo":
after transform: SymbolId(2): ScopeId(3)
rebuilt        : SymbolId(3): ScopeId(1)
Symbol flags mismatch for "N2":
after transform: SymbolId(4): SymbolFlags(ValueModule)
rebuilt        : SymbolId(5): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N2":
after transform: SymbolId(4): Span { start: 145, end: 147 }
rebuilt        : SymbolId(5): Span { start: 0, end: 0 }
Symbol scope ID mismatch for "Foo":
after transform: SymbolId(5): ScopeId(4)
rebuilt        : SymbolId(7): ScopeId(3)
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(5): [ReferenceId(2)]
rebuilt        : SymbolId(7): []

* namespace/redeclaration-with-enum/input.ts
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["x", "y"]
rebuilt        : ScopeId(3): ["x"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(5): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(7): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(3): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "Mixed":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/exported-not-removed/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Direction", "Down", "Up"]
rebuilt        : ScopeId(1): ["Direction"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "Runtime":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/optional-chain-value-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "B", "Foo"]
rebuilt        : ScopeId(1): ["Foo"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "Foo":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/passed-as-argument-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["Active", "Inactive", "Status"]
rebuilt        : ScopeId(1): ["Status"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "Status":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/re-exported-not-removed/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "X"]
rebuilt        : ScopeId(1): ["A"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["B", "Y"]
rebuilt        : ScopeId(3): ["B"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
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
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "Bar":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* optimize-enums/value-usage-kept/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "B", "Foo"]
rebuilt        : ScopeId(1): ["Foo"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "Foo":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)

* preserve-import-=/input.js
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(1): [ReferenceId(1)]
rebuilt        : SymbolId(1): []

* redeclarations/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A"]
rebuilt        : ScopeId(0): ["A", "B", "T"]
Symbol flags mismatch for "A":
after transform: SymbolId(0): SymbolFlags(BlockScopedVariable | ConstVariable | Import)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable | ConstVariable)
Symbol span mismatch for "A":
after transform: SymbolId(0): Span { start: 57, end: 58 }
rebuilt        : SymbolId(0): Span { start: 79, end: 80 }
Symbol reference IDs mismatch for "A":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1)]
rebuilt        : SymbolId(0): [ReferenceId(0)]
Symbol redeclarations mismatch for "A":
after transform: SymbolId(0): [Span { start: 57, end: 58 }, Span { start: 79, end: 80 }]
rebuilt        : SymbolId(0): []
Symbol flags mismatch for "T":
after transform: SymbolId(1): SymbolFlags(Import | TypeAlias)
rebuilt        : SymbolId(1): SymbolFlags(Import)
Symbol redeclarations mismatch for "T":
after transform: SymbolId(1): [Span { start: 149, end: 150 }, Span { start: 170, end: 171 }]
rebuilt        : SymbolId(1): []
Symbol flags mismatch for "B":
after transform: SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable | Import | TypeAlias)
rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable | ConstVariable)
Symbol span mismatch for "B":
after transform: SymbolId(2): Span { start: 267, end: 268 }
rebuilt        : SymbolId(2): Span { start: 289, end: 290 }
Symbol reference IDs mismatch for "B":
after transform: SymbolId(2): [ReferenceId(3), ReferenceId(4)]
rebuilt        : SymbolId(2): [ReferenceId(2)]
Symbol redeclarations mismatch for "B":
after transform: SymbolId(2): [Span { start: 267, end: 268 }, Span { start: 289, end: 290 }, Span { start: 304, end: 305 }]
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
after transform: [ReferenceId(0), ReferenceId(3), ReferenceId(6)]
rebuilt        : [ReferenceId(0)]

* ts-declaration-empty-output/input.d.ts
x Output mismatch

* ts-private-field-with-remove-class-fields-without-initializer/input.ts
Unresolved references mismatch:
after transform: ["ArrayBufferView", "Transferable", "WeakMap", "babelHelpers", "kTransferable", "kValue"]
rebuilt        : ["WeakMap", "babelHelpers", "kTransferable", "kValue"]

* use-define-for-class-fields/input.ts
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

* use-define-for-class-fields-without-class-properties/input.ts
Unresolved reference IDs mismatch for "dce":
after transform: [ReferenceId(0), ReferenceId(1), ReferenceId(4), ReferenceId(9), ReferenceId(12), ReferenceId(14), ReferenceId(17)]
rebuilt        : [ReferenceId(5)]


# babel-plugin-transform-react-jsx (49/54)
* refresh/import-after-component/input.js
Missing ScopeId
Missing ScopeId
Missing ReferenceId: "useFoo"
Symbol reference IDs mismatch for "useFoo":
after transform: SymbolId(1): [ReferenceId(1), ReferenceId(7)]
rebuilt        : SymbolId(1): [ReferenceId(6), ReferenceId(11), ReferenceId(12)]

* refresh/react-refresh/generates-signatures-for-function-expressions-calling-hooks/input.jsx
Bindings mismatch:
after transform: ScopeId(9): []
rebuilt        : ScopeId(9): ["_s3"]
Bindings mismatch:
after transform: ScopeId(10): ["_s3"]
rebuilt        : ScopeId(10): []
Symbol scope ID mismatch for "_s3":
after transform: SymbolId(24): ScopeId(10)
rebuilt        : SymbolId(14): ScopeId(9)

* refresh/react-refresh/generates-valid-signature-for-exotic-ways-to-call-hooks/input.jsx
Bindings mismatch:
after transform: ScopeId(1): []
rebuilt        : ScopeId(1): ["_s"]
Bindings mismatch:
after transform: ScopeId(2): ["_s", "bar", "baz", "useFancyState"]
rebuilt        : ScopeId(2): ["bar", "baz", "useFancyState"]
Symbol scope ID mismatch for "_s":
after transform: SymbolId(8): ScopeId(2)
rebuilt        : SymbolId(4): ScopeId(1)

* refresh/react-refresh/includes-custom-hooks-into-the-signatures-when-commonjs-target-is-used/input.jsx
x Output mismatch

* refresh/react-refresh/supports-typescript-namespace-syntax/input.tsx
x Output mismatch


# legacy-decorators (10/105)
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
after transform: ["PropertyDescriptor", "babelHelpers"]
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
after transform: ["PropertyDescriptor", "WeakMap", "babelHelpers"]
rebuilt        : ["WeakMap", "a", "babelHelpers", "dec"]

* oxc/class-without-name-with-decorated_class/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["dec"]
rebuilt        : ScopeId(0): ["_default", "dec"]
Bindings mismatch:
after transform: ScopeId(1): ["_default"]
rebuilt        : ScopeId(1): []
Symbol flags mismatch for "_default":
after transform: SymbolId(1): SymbolFlags(Class)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol scope ID mismatch for "_default":
after transform: SymbolId(1): ScopeId(1)
rebuilt        : SymbolId(1): ScopeId(0)

* oxc/class-without-name-with-decorated_element/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["dec"]
rebuilt        : ScopeId(0): ["_default", "dec"]
Bindings mismatch:
after transform: ScopeId(1): ["_default"]
rebuilt        : ScopeId(1): []
Symbol scope ID mismatch for "_default":
after transform: SymbolId(1): ScopeId(1)
rebuilt        : SymbolId(1): ScopeId(0)

* oxc/metadata/abstract-class/input.ts
Symbol reference IDs mismatch for "Dependency":
after transform: SymbolId(1): [ReferenceId(1), ReferenceId(2), ReferenceId(3)]
rebuilt        : SymbolId(1): [ReferenceId(5), ReferenceId(7)]
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
Symbol reference IDs mismatch for "BoundTypeReference":
after transform: SymbolId(0): [ReferenceId(3), ReferenceId(1), ReferenceId(4), ReferenceId(5), ReferenceId(6)]
rebuilt        : SymbolId(0): [ReferenceId(1), ReferenceId(7), ReferenceId(9)]
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
after transform: ["ClassDecorator", "String", "babelHelpers"]
rebuilt        : ["String", "babelHelpers", "dec"]

* oxc/metadata/cross-file-imported-enum/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Source", "StringEnum", "dec"]
rebuilt        : ScopeId(0): ["Source", "StringEnum"]
Symbol reference IDs mismatch for "StringEnum":
after transform: SymbolId(0): [ReferenceId(1), ReferenceId(2), ReferenceId(3)]
rebuilt        : SymbolId(0): [ReferenceId(3), ReferenceId(5)]
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
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["TemplateStringEnum", "mixed", "template"]
rebuilt        : ScopeId(3): ["TemplateStringEnum"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["NumberEnum", "a", "b"]
rebuilt        : ScopeId(5): ["NumberEnum"]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(5): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(4): ["UnaryEnum", "bitwise", "negative", "positive"]
rebuilt        : ScopeId(7): ["UnaryEnum"]
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(0x0)
rebuilt        : ScopeId(7): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(7): ["UnaryOtherEnum", "bitwise", "negative", "positive"]
rebuilt        : ScopeId(11): ["UnaryOtherEnum"]
Scope flags mismatch:
after transform: ScopeId(7): ScopeFlags(0x0)
rebuilt        : ScopeId(11): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(8): ["AutoIncrementEnum", "first", "second", "third"]
rebuilt        : ScopeId(13): ["AutoIncrementEnum"]
Scope flags mismatch:
after transform: ScopeId(8): ScopeFlags(0x0)
rebuilt        : ScopeId(13): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(9): ["MixedEnum", "num", "str"]
rebuilt        : ScopeId(15): ["MixedEnum"]
Scope flags mismatch:
after transform: ScopeId(9): ScopeFlags(0x0)
rebuilt        : ScopeId(15): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(10): ["ComputedEnum", "computed", "expression"]
rebuilt        : ScopeId(17): ["ComputedEnum"]
Scope flags mismatch:
after transform: ScopeId(10): ScopeFlags(0x0)
rebuilt        : ScopeId(17): ScopeFlags(Function)
Symbol flags mismatch for "StringEnum":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "StringEnum":
after transform: SymbolId(0): [ReferenceId(21), ReferenceId(5), ReferenceId(27)]
rebuilt        : SymbolId(0): [ReferenceId(3)]
Symbol flags mismatch for "TemplateStringEnum":
after transform: SymbolId(3): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "TemplateStringEnum":
after transform: SymbolId(3): [ReferenceId(7), ReferenceId(31)]
rebuilt        : SymbolId(2): [ReferenceId(7)]
Symbol flags mismatch for "NumberEnum":
after transform: SymbolId(6): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(4): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "NumberEnum":
after transform: SymbolId(6): [ReferenceId(22), ReferenceId(9), ReferenceId(23), ReferenceId(37)]
rebuilt        : SymbolId(4): [ReferenceId(13), ReferenceId(53)]
Symbol flags mismatch for "UnaryEnum":
after transform: SymbolId(9): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(6): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "UnaryEnum":
after transform: SymbolId(9): [ReferenceId(11), ReferenceId(45)]
rebuilt        : SymbolId(6): [ReferenceId(21)]
Symbol flags mismatch for "UnaryOtherEnum":
after transform: SymbolId(14): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(9): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "UnaryOtherEnum":
after transform: SymbolId(14): [ReferenceId(13), ReferenceId(53)]
rebuilt        : SymbolId(9): [ReferenceId(32)]
Symbol flags mismatch for "AutoIncrementEnum":
after transform: SymbolId(18): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(11): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "AutoIncrementEnum":
after transform: SymbolId(18): [ReferenceId(15), ReferenceId(61)]
rebuilt        : SymbolId(11): [ReferenceId(40)]
Symbol flags mismatch for "MixedEnum":
after transform: SymbolId(22): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(13): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "MixedEnum":
after transform: SymbolId(22): [ReferenceId(17), ReferenceId(66)]
rebuilt        : SymbolId(13): [ReferenceId(45)]
Symbol flags mismatch for "ComputedEnum":
after transform: SymbolId(25): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(15): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "ComputedEnum":
after transform: SymbolId(25): [ReferenceId(19), ReferenceId(72)]
rebuilt        : SymbolId(15): [ReferenceId(52)]

* oxc/metadata/erased-import-no-type-keyword/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Source", "T", "dec"]
rebuilt        : ScopeId(0): ["Source", "T"]
Symbol reference IDs mismatch for "T":
after transform: SymbolId(0): [ReferenceId(1), ReferenceId(2), ReferenceId(3)]
rebuilt        : SymbolId(0): [ReferenceId(3), ReferenceId(5)]
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
Symbol reference IDs mismatch for "LaterClass":
after transform: SymbolId(5): [ReferenceId(2), ReferenceId(3), ReferenceId(4)]
rebuilt        : SymbolId(1): [ReferenceId(3), ReferenceId(5)]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "PropertyDescriptor", "babelHelpers"]
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
after transform: ["Function", "Number", "Object", "PropertyDescriptor", "String", "babelHelpers"]
rebuilt        : ["Function", "Number", "Object", "String", "babelHelpers", "dec"]

* oxc/metadata/imports/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Bar", "Cls", "Foo", "Zoo", "dec"]
rebuilt        : ScopeId(0): ["Cls", "Foo"]
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(0): [ReferenceId(2), ReferenceId(3), ReferenceId(12), ReferenceId(13)]
rebuilt        : SymbolId(0): [ReferenceId(10), ReferenceId(12)]
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
after transform: ["Object", "PropertyDescriptor", "babelHelpers", "console"]
rebuilt        : ["Object", "babelHelpers", "console", "dec"]

* oxc/metadata/namespace-imported-enum/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["NS", "Source", "dec"]
rebuilt        : ScopeId(0): ["NS", "Source"]
Symbol reference IDs mismatch for "NS":
after transform: SymbolId(0): [ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(4)]
rebuilt        : SymbolId(0): [ReferenceId(3), ReferenceId(4), ReferenceId(6)]
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
after transform: ScopeId(4): ["Cls2"]
rebuilt        : ScopeId(5): []
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
after transform: SymbolId(5): ScopeId(4)
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
after transform: ["Array", "ReadonlyArray", "babelHelpers"]
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
Symbol reference IDs mismatch for "A":
after transform: SymbolId(1): [ReferenceId(4), ReferenceId(5), ReferenceId(6)]
rebuilt        : SymbolId(0): [ReferenceId(6), ReferenceId(8)]
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
after transform: ["ClassDecorator", "Error", "Object", "babelHelpers"]
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
Unresolved reference IDs mismatch for "UnboundTypeReference":
after transform: [ReferenceId(1), ReferenceId(2), ReferenceId(3)]
rebuilt        : [ReferenceId(5), ReferenceId(7)]

* oxc/metadata/without-decorator/input.ts
Symbol span mismatch for "C":
after transform: SymbolId(2): Span { start: 106, end: 107 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(3): Span { start: 106, end: 107 }

* oxc/static-field/input.ts
Scope flags mismatch:
after transform: ScopeId(6): ScopeFlags(ClassStaticBlock)
rebuilt        : ScopeId(6): ScopeFlags(StrictMode | ClassStaticBlock)
Scope parent mismatch:
after transform: ScopeId(6): Some(ScopeId(0))
rebuilt        : ScopeId(6): Some(ScopeId(5))
Symbol span mismatch for "Foo":
after transform: SymbolId(2): Span { start: 103, end: 106 }
rebuilt        : SymbolId(3): Span { start: 0, end: 0 }
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(2): [ReferenceId(4), ReferenceId(6), ReferenceId(8)]
rebuilt        : SymbolId(3): [ReferenceId(4), ReferenceId(8)]
Symbol span mismatch for "Foo":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(4): Span { start: 103, end: 106 }
Unresolved references mismatch:
after transform: ["ClassDecorator", "babelHelpers", "console"]
rebuilt        : ["babelHelpers", "console"]

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
Unresolved references mismatch:
after transform: ["ClassDecorator", "babelHelpers", "console"]
rebuilt        : ["babelHelpers", "console"]

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
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
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
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor5/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
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
after transform: ["Function", "babelHelpers"]
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
after transform: ScopeId(4): ScopeFlags(StrictMode | Function | Arrow)
rebuilt        : ScopeId(6): ScopeFlags(Function | Arrow)
Scope parent mismatch:
after transform: ScopeId(4): Some(ScopeId(3))
rebuilt        : ScopeId(6): Some(ScopeId(0))
Scope flags mismatch:
after transform: ScopeId(5): ScopeFlags(StrictMode | FunctionBody)
rebuilt        : ScopeId(7): ScopeFlags(FunctionBody)

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
after transform: ["Function", "babelHelpers"]
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
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod10/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Function", "babelHelpers"]
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
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
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
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
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
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod5/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod6/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod7/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
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
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Symbol reference IDs mismatch for "C":
after transform: SymbolId(4): [ReferenceId(1), ReferenceId(4)]
rebuilt        : SymbolId(0): [ReferenceId(3)]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter3/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["dec", "fn"]
rebuilt        : ScopeId(0): ["fn"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Promise", "babelHelpers"]
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
after transform: ["PropertyDescriptor", "babelHelpers"]
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
after transform: ["PropertyDescriptor", "babelHelpers"]
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
after transform: ["Function", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty7/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["Function", "babelHelpers"]
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


