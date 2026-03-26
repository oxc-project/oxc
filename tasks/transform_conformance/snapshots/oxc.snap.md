commit: 0124e7c7

Passed: 101/345

# All Passed:
* babel-plugin-transform-class-static-block
* babel-plugin-transform-nullish-coalescing-operator
* babel-plugin-transform-optional-catch-binding
* babel-plugin-transform-arrow-functions
* babel-preset-typescript
* regexp


# babel-plugin-transform-explicit-resource-management (0/4)
* export-class-name/input.js
x Output mismatch

* for-of-no-block/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* function-with-scopes-in-params/input.js
Bindings mismatch:
after transform: ScopeId(1): ["_usingCtx", "a", "b", "x", "y"]
rebuilt        : ScopeId(1): ["_usingCtx", "a", "b"]
Bindings mismatch:
after transform: ScopeId(5): []
rebuilt        : ScopeId(4): ["x", "y"]
Symbol scope ID mismatch for "x":
after transform: SymbolId(3): ScopeId(1)
rebuilt        : SymbolId(4): ScopeId(4)
Symbol scope ID mismatch for "y":
after transform: SymbolId(4): ScopeId(1)
rebuilt        : SymbolId(5): ScopeId(4)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)

* try-catch/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_usingCtx":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-class-properties (1/32)
* instance-prop-initializer-no-existing-constructor/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* instance-prop-initializer-var-clash/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* interaction-with-other-transforms/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)

* private-field-resolve-to-method/input.js
x Output mismatch

* private-field-resolve-to-method-in-computed-key/input.js
x Output mismatch

* private-logical-assignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)

* private-loose-logical-assignment/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_nullish":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)

* private-loose-tagged-template/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-loose-tagged-template-static/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Object":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* private-optional-call-with-non-optional-callee/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-optional-member-with-sequence/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "undefined":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(2): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* public-static-super-call/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* static-async-super/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* static-block-this-and-class-name/input.js
Reference flags mismatch for "_C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_C3":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_C4":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_C5":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_C5":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Nested":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Nested":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Nested":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(22): ReferenceFlags(Read)

* static-prop-initializer-strict-mode/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* static-super-assignment-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(124): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(59): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "bound":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(33): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(66): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(73): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(77): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(80): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(84): ReferenceFlags(Read)
rebuilt        : ReferenceId(59): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(87): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(90): ReferenceFlags(Read)
rebuilt        : ReferenceId(67): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "unbound":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(69): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(98): ReferenceFlags(Read)
rebuilt        : ReferenceId(71): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(95): ReferenceFlags(Read)
rebuilt        : ReferenceId(75): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(106): ReferenceFlags(Read)
rebuilt        : ReferenceId(80): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(103): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(89): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(114): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(119): ReferenceFlags(Read)
rebuilt        : ReferenceId(98): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(122): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)

* static-super-assignment-target/input.js
x Output mismatch

* static-super-tagged-template/input.js
x Output mismatch

* static-super-update-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(151): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(72): ReferenceFlags(Read)
rebuilt        : ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(65): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(84): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(77): ReferenceFlags(Read)
rebuilt        : ReferenceId(59): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(94): ReferenceFlags(Read)
rebuilt        : ReferenceId(68): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(89): ReferenceFlags(Read)
rebuilt        : ReferenceId(72): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(103): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(98): ReferenceFlags(Read)
rebuilt        : ReferenceId(82): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(115): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(108): ReferenceFlags(Read)
rebuilt        : ReferenceId(93): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(128): ReferenceFlags(Read)
rebuilt        : ReferenceId(102): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(121): ReferenceFlags(Read)
rebuilt        : ReferenceId(107): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(139): ReferenceFlags(Read)
rebuilt        : ReferenceId(116): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(134): ReferenceFlags(Read)
rebuilt        : ReferenceId(121): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(149): ReferenceFlags(Read)
rebuilt        : ReferenceId(127): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(144): ReferenceFlags(Read)
rebuilt        : ReferenceId(132): ReferenceFlags(Read | MemberWriteTarget)

* super-in-constructor-missing/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* super-in-constructor-nested/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* super-in-constructor-nested-super/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* super-in-constructor-strict/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_super":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_super2":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* super-in-static-block/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "prop":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "prop":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(14): ReferenceFlags(Read)

* super-in-static-prop-initializer/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "prop":
after transform: ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "prop":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(16): ReferenceFlags(Read)

* this-in-computed-key/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)

* typescript/class-fields-with-computed-key/input.ts
Reference flags mismatch for "_Collection$identifie":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Obj":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* typescript/declare-computed-keys/input.ts
Symbol reference IDs mismatch for "KEY1":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2)]
rebuilt        : SymbolId(1): []
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* typescript/declare-fields/input.ts
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* typescript/optional-call/input.ts
Symbol reference IDs mismatch for "X":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(6), ReferenceId(11), ReferenceId(16)]
rebuilt        : SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(8), ReferenceId(14)]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* typescript/optional-member/input.ts
Symbol reference IDs mismatch for "X":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(6), ReferenceId(9), ReferenceId(12)]
rebuilt        : SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(6), ReferenceId(10)]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-private-methods (0/1)
* unused-methods/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-logical-assignment-operators (1/6)
* computed-prop-identifier/input.js
Reference flags mismatch for "boundObj":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj2":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj3":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_mutatedObj":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_mutatedObj2":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_mutatedObj3":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_boundObj$prop":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(53): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_boundObj$prop2":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(59): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(62): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_boundObj$prop3":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(65): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundProp":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(67): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundProp":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(68): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundProp5":
after transform: ReferenceId(72): ReferenceFlags(Read)
rebuilt        : ReferenceId(71): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_mutatedProp5":
after transform: ReferenceId(74): ReferenceFlags(Read)
rebuilt        : ReferenceId(74): ReferenceFlags(Read | MemberWriteTarget)

* computed-prop-literal/input.js
Reference flags mismatch for "boundObj":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj2":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj3":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj4":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj5":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj6":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)

* computed-prop-template-literal/input.js
Reference flags mismatch for "boundObj":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj2":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)

* literal-member-expression/input.js
Reference flags mismatch for "boundProp":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundProp":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundProp":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* super-prop-computed/input.js
Reference flags mismatch for "boundProp":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundProp":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundProp":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_mutatedProp":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-optional-chaining (1/2)
* oxc/keep-this/input.ts
Reference flags mismatch for "_this$f":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$f":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$f":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$f":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$y$f":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$y$f":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x2":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x2":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x2$f":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x3":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x3":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x3$y$f":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$y2":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$y2":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$y2$f":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x4":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x4":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x4":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x4$f":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$f2":
after transform: ReferenceId(57): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$f2":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_f":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(53): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$f":
after transform: ReferenceId(64): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x$f":
after transform: ReferenceId(69): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x$y$f":
after transform: ReferenceId(74): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x2":
after transform: ReferenceId(78): ReferenceFlags(Read)
rebuilt        : ReferenceId(76): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x2$f":
after transform: ReferenceId(81): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x3":
after transform: ReferenceId(85): ReferenceFlags(Read)
rebuilt        : ReferenceId(85): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x3$y$f":
after transform: ReferenceId(89): ReferenceFlags(Read)
rebuilt        : ReferenceId(87): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x$y2":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(93): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x$y2$f":
after transform: ReferenceId(96): ReferenceFlags(Read)
rebuilt        : ReferenceId(95): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x4":
after transform: ReferenceId(100): ReferenceFlags(Read)
rebuilt        : ReferenceId(101): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x4":
after transform: ReferenceId(103): ReferenceFlags(Read)
rebuilt        : ReferenceId(104): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x4$f":
after transform: ReferenceId(106): ReferenceFlags(Read)
rebuilt        : ReferenceId(106): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_repro$x$f2":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(112): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "repro":
after transform: ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(116): ReferenceFlags(Read)
Reference flags mismatch for "_f2":
after transform: ReferenceId(116): ReferenceFlags(Read)
rebuilt        : ReferenceId(118): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-async-generator-functions (1/3)
* for-await/with-if-statement/input.js
Reference flags mismatch for "_handleAsyncIterables":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_handleAsyncIterables":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)

* for-await/with-labeled-statement/input.js
Reference flags mismatch for "_handleAsyncIterable":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_step":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_iterator":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_handleAsyncIterable":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-object-rest-spread (0/6)
* object-rest/assignment-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/complex/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(44): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(50): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/export/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/keys/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/simple/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(59): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(57): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(62): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(72): ReferenceFlags(Read)
rebuilt        : ReferenceId(68): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_key":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(71): ReferenceFlags(Read)
rebuilt        : ReferenceId(71): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(78): ReferenceFlags(Read)
rebuilt        : ReferenceId(76): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_key2":
after transform: ReferenceId(75): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(77): ReferenceFlags(Read)
rebuilt        : ReferenceId(79): ReferenceFlags(Read | MemberWriteTarget)

* object-rest/with-arrow-function-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-async-to-generator (0/25)
* arguments/assign/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* arguments/async-method/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* arguments/nested-block/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* arrow/basic/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* arrow/without-params/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* class/method-definition/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* class/method-parameters-error/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* class/property-definition/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* class/static-block/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* class/this-after-super/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "S":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)

* class/this-after-super-in-super/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* class/this-after-super-nested/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Outer2":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(12): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)

* class/this-after-super-with-async-arrow-function/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* function/export/default-with-name/input.js
Reference flags mismatch for "_D":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_D":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* function/export/default-without-name/input.js
Reference flags mismatch for "_ref":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* function/export/named/input.js
Reference flags mismatch for "_named":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_named":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* function/expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* object/method/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* object/property-with-function/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref2":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref3":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref4":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref5":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref6":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref7":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)

* super/assign/input.js
Reference flags mismatch for "_value":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_prop":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_superprop_getObject":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* super/computed-member/input.js
Reference flags mismatch for "_prop":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_superprop_get":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_superprop_get":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* super/nested/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* super/nested-class/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Outer":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(3): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* super/outer-super-in-nested-class/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_ref":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* super/property/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-exponentiation-operator (0/5)
* assign-to-identifier/input.js
Reference flags mismatch for "Math":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)

* assign-to-member-expression/input.js
Reference flags mismatch for "obj":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$foo$bar":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$foo$bar":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(61): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(57): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$foo2$bar":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(68): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$foo2$bar":
after transform: ReferenceId(64): ReferenceFlags(Read)
rebuilt        : ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$foo3$bar":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(75): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$foo3$bar":
after transform: ReferenceId(71): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(80): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(52): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(85): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(81): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj":
after transform: ReferenceId(87): ReferenceFlags(Read)
rebuilt        : ReferenceId(59): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(89): ReferenceFlags(Read)
rebuilt        : ReferenceId(60): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj":
after transform: ReferenceId(88): ReferenceFlags(Read)
rebuilt        : ReferenceId(61): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj2":
after transform: ReferenceId(91): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(93): ReferenceFlags(Read)
rebuilt        : ReferenceId(65): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj2":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "unboundObj":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(68): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj$foo$bar":
after transform: ReferenceId(95): ReferenceFlags(Read)
rebuilt        : ReferenceId(69): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(97): ReferenceFlags(Read)
rebuilt        : ReferenceId(70): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj$foo$bar":
after transform: ReferenceId(96): ReferenceFlags(Read)
rebuilt        : ReferenceId(71): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj3":
after transform: ReferenceId(99): ReferenceFlags(Read)
rebuilt        : ReferenceId(76): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(104): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj3":
after transform: ReferenceId(100): ReferenceFlags(Read)
rebuilt        : ReferenceId(79): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj4":
after transform: ReferenceId(106): ReferenceFlags(Read)
rebuilt        : ReferenceId(85): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(87): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj4":
after transform: ReferenceId(107): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "unboundObj":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(91): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj$foo2$bar":
after transform: ReferenceId(113): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(118): ReferenceFlags(Read)
rebuilt        : ReferenceId(96): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj$foo2$bar":
after transform: ReferenceId(114): ReferenceFlags(Read)
rebuilt        : ReferenceId(97): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "unboundObj":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(100): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj$foo3$bar":
after transform: ReferenceId(120): ReferenceFlags(Read)
rebuilt        : ReferenceId(103): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(125): ReferenceFlags(Read)
rebuilt        : ReferenceId(105): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj$foo3$bar":
after transform: ReferenceId(121): ReferenceFlags(Read)
rebuilt        : ReferenceId(106): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj5":
after transform: ReferenceId(127): ReferenceFlags(Read)
rebuilt        : ReferenceId(112): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(132): ReferenceFlags(Read)
rebuilt        : ReferenceId(114): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj5":
after transform: ReferenceId(128): ReferenceFlags(Read)
rebuilt        : ReferenceId(115): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj6":
after transform: ReferenceId(134): ReferenceFlags(Read)
rebuilt        : ReferenceId(121): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(139): ReferenceFlags(Read)
rebuilt        : ReferenceId(123): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj6":
after transform: ReferenceId(135): ReferenceFlags(Read)
rebuilt        : ReferenceId(124): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(141): ReferenceFlags(Read)
rebuilt        : ReferenceId(128): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(143): ReferenceFlags(Read)
rebuilt        : ReferenceId(129): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn":
after transform: ReferenceId(142): ReferenceFlags(Read)
rebuilt        : ReferenceId(130): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(132): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$foo$bar":
after transform: ReferenceId(145): ReferenceFlags(Read)
rebuilt        : ReferenceId(133): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(147): ReferenceFlags(Read)
rebuilt        : ReferenceId(134): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$foo$bar":
after transform: ReferenceId(146): ReferenceFlags(Read)
rebuilt        : ReferenceId(135): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(137): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$prop":
after transform: ReferenceId(149): ReferenceFlags(Read)
rebuilt        : ReferenceId(140): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(154): ReferenceFlags(Read)
rebuilt        : ReferenceId(142): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$prop":
after transform: ReferenceId(150): ReferenceFlags(Read)
rebuilt        : ReferenceId(143): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(146): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$prop2":
after transform: ReferenceId(156): ReferenceFlags(Read)
rebuilt        : ReferenceId(149): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(161): ReferenceFlags(Read)
rebuilt        : ReferenceId(151): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$prop2":
after transform: ReferenceId(157): ReferenceFlags(Read)
rebuilt        : ReferenceId(152): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this":
after transform: ReferenceId(163): ReferenceFlags(Read)
rebuilt        : ReferenceId(155): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(165): ReferenceFlags(Read)
rebuilt        : ReferenceId(156): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this":
after transform: ReferenceId(164): ReferenceFlags(Read)
rebuilt        : ReferenceId(157): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$foo$bar":
after transform: ReferenceId(167): ReferenceFlags(Read)
rebuilt        : ReferenceId(159): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(169): ReferenceFlags(Read)
rebuilt        : ReferenceId(160): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$foo$bar":
after transform: ReferenceId(168): ReferenceFlags(Read)
rebuilt        : ReferenceId(161): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this2":
after transform: ReferenceId(171): ReferenceFlags(Read)
rebuilt        : ReferenceId(163): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(173): ReferenceFlags(Read)
rebuilt        : ReferenceId(164): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this2":
after transform: ReferenceId(172): ReferenceFlags(Read)
rebuilt        : ReferenceId(165): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn4":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(168): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this3":
after transform: ReferenceId(175): ReferenceFlags(Read)
rebuilt        : ReferenceId(169): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(180): ReferenceFlags(Read)
rebuilt        : ReferenceId(171): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this3":
after transform: ReferenceId(176): ReferenceFlags(Read)
rebuilt        : ReferenceId(172): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this4":
after transform: ReferenceId(182): ReferenceFlags(Read)
rebuilt        : ReferenceId(175): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(184): ReferenceFlags(Read)
rebuilt        : ReferenceId(176): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this4":
after transform: ReferenceId(183): ReferenceFlags(Read)
rebuilt        : ReferenceId(177): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$foo$bar2":
after transform: ReferenceId(186): ReferenceFlags(Read)
rebuilt        : ReferenceId(179): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(188): ReferenceFlags(Read)
rebuilt        : ReferenceId(180): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$foo$bar2":
after transform: ReferenceId(187): ReferenceFlags(Read)
rebuilt        : ReferenceId(181): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this5":
after transform: ReferenceId(190): ReferenceFlags(Read)
rebuilt        : ReferenceId(183): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(192): ReferenceFlags(Read)
rebuilt        : ReferenceId(184): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this5":
after transform: ReferenceId(191): ReferenceFlags(Read)
rebuilt        : ReferenceId(185): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn4":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(188): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this6":
after transform: ReferenceId(194): ReferenceFlags(Read)
rebuilt        : ReferenceId(189): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(199): ReferenceFlags(Read)
rebuilt        : ReferenceId(191): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this6":
after transform: ReferenceId(195): ReferenceFlags(Read)
rebuilt        : ReferenceId(192): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "___bound":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(194): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(201): ReferenceFlags(Read)
rebuilt        : ReferenceId(195): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "___bound":
after transform: ReferenceId(200): ReferenceFlags(Read)
rebuilt        : ReferenceId(196): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "___unbound2":
after transform: ReferenceId(203): ReferenceFlags(Read)
rebuilt        : ReferenceId(199): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(205): ReferenceFlags(Read)
rebuilt        : ReferenceId(200): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "___unbound2":
after transform: ReferenceId(204): ReferenceFlags(Read)
rebuilt        : ReferenceId(201): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(204): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(210): ReferenceFlags(Read)
rebuilt        : ReferenceId(206): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(206): ReferenceFlags(Read)
rebuilt        : ReferenceId(207): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(211): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(215): ReferenceFlags(Read)
rebuilt        : ReferenceId(213): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(211): ReferenceFlags(Read)
rebuilt        : ReferenceId(214): ReferenceFlags(Read | MemberWriteTarget)

* assign-used-result/input.js
Reference flags mismatch for "Math":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_boundObj$foo$bar":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_boundObj$foo$bar":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "unboundObj":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj$foo$bar":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(30): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj$foo$bar":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(53): ReferenceFlags(Read)
rebuilt        : ReferenceId(45): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boundObj":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj2":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(53): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(60): ReferenceFlags(Read)
rebuilt        : ReferenceId(55): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj2":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj3":
after transform: ReferenceId(62): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(65): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_unboundObj3":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)

* bail-bigint/input.js
Reference flags mismatch for "Math":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* private-properties/input.js
Reference flags mismatch for "_this":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$y$z":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_this$x$y$z":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "obj":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$x$y$z":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_obj$x$y$z":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "fn":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$x$y$z":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Math":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_fn$x$y$z":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-typescript (6/31)
* allow-declare-fields-false/input.ts
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

* class-constructor-arguments/input.ts
Reference flags mismatch for "foo":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "bar":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "zoo":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "bang":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "bar":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "zoo":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "bang":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boom":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "foo":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "bar":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "zoo":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "bang":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "boom":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "A":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "C":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "C":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "C":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "D":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "D":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "D":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "D":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "D":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Infinity", "NaN"]
rebuilt        : ["Infinity"]
Unresolved reference IDs mismatch for "Infinity":
after transform: [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(8), ReferenceId(11), ReferenceId(14), ReferenceId(18)]
rebuilt        : [ReferenceId(2), ReferenceId(5), ReferenceId(8), ReferenceId(12)]

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
Reference flags mismatch for "Foo":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Merge":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Merge":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Merge":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Merge":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NestOuter":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NestOuter":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NestOuter":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NestOuter":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NestInner":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NestInner":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NestInner":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NestInner":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)

* enum-template-literal/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["LARGE", "SMALL", "Size"]
rebuilt        : ScopeId(1): ["Size"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["Animal", "CAT", "DOG"]
rebuilt        : ScopeId(2): ["Animal"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["AnimalSize", "LARGE_DOG", "SMALL_CAT"]
rebuilt        : ScopeId(3): ["AnimalSize"]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
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
Reference flags mismatch for "Size":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Size":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animal":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Animal":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "AnimalSize":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "AnimalSize":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* enum-template-literal-number/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["NUM_1", "NUM_2", "NUM_3", "NUM_4", "NumberEnum"]
rebuilt        : ScopeId(1): ["NumberEnum"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["COMPUTED_1", "COMPUTED_2", "ComputedEnum"]
rebuilt        : ScopeId(2): ["ComputedEnum"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Symbol flags mismatch for "NumberEnum":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "NumberEnum":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(13)]
rebuilt        : SymbolId(0): [ReferenceId(9)]
Symbol flags mismatch for "ComputedEnum":
after transform: SymbolId(5): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "ComputedEnum":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "ComputedEnum":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)

* enum-template-literal-trailing-quasi/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["A", "B", "NumberEnum"]
rebuilt        : ScopeId(1): ["NumberEnum"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["C", "ComputedEnum", "D"]
rebuilt        : ScopeId(2): ["ComputedEnum"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Symbol flags mismatch for "NumberEnum":
after transform: SymbolId(0): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(0): SymbolFlags(FunctionScopedVariable)
Symbol reference IDs mismatch for "NumberEnum":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(8)]
rebuilt        : SymbolId(0): [ReferenceId(5)]
Symbol flags mismatch for "ComputedEnum":
after transform: SymbolId(3): SymbolFlags(RegularEnum)
rebuilt        : SymbolId(2): SymbolFlags(FunctionScopedVariable)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "ComputedEnum":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "ComputedEnum":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* export-elimination/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Bar", "Foo", "Func", "Im", "Name", "Ok"]
rebuilt        : ScopeId(0): ["Bar", "Foo", "Func", "Im", "Name", "Ok", "T"]
Scope flags mismatch:
after transform: ScopeId(5): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(3): ScopeFlags(Function)
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
Reference flags mismatch for "_Name":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* namespace/export-import-=/input.ts
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Symbol flags mismatch for "N1":
after transform: SymbolId(1): SymbolFlags(ValueModule)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "N1":
after transform: SymbolId(1): Span { start: 31, end: 33 }
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Reference flags mismatch for "A":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_N":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* namespace/import-=/input.ts
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
Reference flags mismatch for "A":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* namespace/preserve-import-=/input.ts
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(2): ScopeFlags(Function)
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
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(5): [ReferenceId(2)]
rebuilt        : SymbolId(7): []
Reference flags mismatch for "nsa":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "nsa":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* namespace/redeclaration-with-enum/input.ts
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["x", "y"]
rebuilt        : ScopeId(2): ["x"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(4): ScopeFlags(Function)
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
Reference flags mismatch for "x":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "x":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "y":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "y":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "_Foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* namespace/redeclaration-with-type-alias/input.ts
Bindings mismatch:
after transform: ScopeId(0): []
rebuilt        : ScopeId(0): ["Foo"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(StrictMode | Function)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Symbol flags mismatch for "Foo":
after transform: SymbolId(0): SymbolFlags(TypeAlias | ValueModule)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Symbol span mismatch for "Foo":
after transform: SymbolId(0): Span { start: 12, end: 15 }
rebuilt        : SymbolId(0): Span { start: 0, end: 0 }
Symbol redeclarations mismatch for "Foo":
after transform: SymbolId(0): [Span { start: 12, end: 15 }, Span { start: 39, end: 42 }, Span { start: 87, end: 90 }]
rebuilt        : SymbolId(0): []
Reference flags mismatch for "_Foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo2":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "_Foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* preserve-import-=/input.js
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(1): [ReferenceId(1)]
rebuilt        : SymbolId(1): []
Reference flags mismatch for "nsa":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "foo":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "a":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "b":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved reference IDs mismatch for "foo":
after transform: [ReferenceId(0), ReferenceId(3), ReferenceId(6)]
rebuilt        : [ReferenceId(0)]

* ts-declaration-empty-output/input.d.ts
x Output mismatch

* ts-private-field-with-remove-class-fields-without-initializer/input.ts
Reference flags mismatch for "kValue":
after transform: ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(4): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "view":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(9): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["ArrayBufferView", "Transferable", "WeakMap", "babelHelpers", "kTransferable", "kValue"]
rebuilt        : ["WeakMap", "babelHelpers", "kTransferable", "kValue"]

* use-define-for-class-fields/input.ts
Reference flags mismatch for "StaticCls":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

* use-define-for-class-fields-without-class-properties/input.ts
Scope parent mismatch:
after transform: ScopeId(12): Some(ScopeId(0))
rebuilt        : ScopeId(2): Some(ScopeId(1))
Scope parent mismatch:
after transform: ScopeId(11): Some(ScopeId(0))
rebuilt        : ScopeId(3): Some(ScopeId(1))
Scope parent mismatch:
after transform: ScopeId(13): Some(ScopeId(0))
rebuilt        : ScopeId(5): Some(ScopeId(4))
Scope parent mismatch:
after transform: ScopeId(16): Some(ScopeId(0))
rebuilt        : ScopeId(8): Some(ScopeId(7))
Scope parent mismatch:
after transform: ScopeId(14): Some(ScopeId(0))
rebuilt        : ScopeId(9): Some(ScopeId(7))
Scope parent mismatch:
after transform: ScopeId(15): Some(ScopeId(0))
rebuilt        : ScopeId(10): Some(ScopeId(7))
Scope parent mismatch:
after transform: ScopeId(17): Some(ScopeId(0))
rebuilt        : ScopeId(12): Some(ScopeId(11))
Scope parent mismatch:
after transform: ScopeId(18): Some(ScopeId(0))
rebuilt        : ScopeId(17): Some(ScopeId(15))
Scope parent mismatch:
after transform: ScopeId(19): Some(ScopeId(0))
rebuilt        : ScopeId(19): Some(ScopeId(15))
Scope parent mismatch:
after transform: ScopeId(20): Some(ScopeId(0))
rebuilt        : ScopeId(20): Some(ScopeId(15))
Reference flags mismatch for "_y":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_y2":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_y3":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_y4":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_c":
after transform: ReferenceId(44): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_y5":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_a2":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_c2":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Unresolved reference IDs mismatch for "dce":
after transform: [ReferenceId(0), ReferenceId(1), ReferenceId(4), ReferenceId(9), ReferenceId(12), ReferenceId(14), ReferenceId(17)]
rebuilt        : [ReferenceId(5)]


# babel-plugin-transform-react-jsx (39/51)
* misc/arbitrary_length_member_expr/input.jsx
Reference flags mismatch for "a":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)

* refresh/import-after-component/input.js
Missing ScopeId
Missing ReferenceId: "useFoo"
Symbol reference IDs mismatch for "useFoo":
after transform: SymbolId(1): [ReferenceId(1), ReferenceId(7)]
rebuilt        : SymbolId(1): [ReferenceId(6), ReferenceId(11), ReferenceId(12)]

* refresh/react-refresh/generates-valid-signature-for-exotic-ways-to-call-hooks/input.jsx
Reference flags mismatch for "FancyHook":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)

* refresh/react-refresh/includes-custom-hooks-into-the-signatures-when-commonjs-target-is-used/input.jsx
x Output mismatch

* refresh/react-refresh/registers-identifiers-used-in-jsx-at-definition-site/input.jsx
Reference flags mismatch for "Dict":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)

* refresh/react-refresh/supports-typescript-namespace-syntax/input.tsx
x Output mismatch

* refresh/react-refresh/uses-custom-identifiers-for-refresh-reg-and-refresh-sig/input.jsx
Reference flags mismatch for "_s":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_c":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* spread-children-automatic/input.jsx
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* spread-children-classic/input.jsx
Reference flags mismatch for "React":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* spread-children-mixed-automatic/input.jsx
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* spread-children-multiple-automatic/input.jsx
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

* spread-props-classic/input.jsx
Reference flags mismatch for "React":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "React":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-react-jsx-self (0/1)
* react-source/duplicate-self-prop/input.jsx
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(0): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)


# babel-plugin-transform-react-jsx-source (1/2)
* react-source/duplicate-source-prop/input.jsx
Reference flags mismatch for "_reactJsxRuntime":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)


# legacy-decorators (0/93)
* oxc/accessor/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "_a", "_foo$bar", "_foo$bar2", "a", "dec", "foo"]
rebuilt        : ScopeId(0): ["C", "_a", "_foo$bar", "_foo$bar2"]
Reference flags mismatch for "value":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "C":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "C":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "a":
after transform: SymbolId(4) "a"
rebuilt        : <None>
Reference symbol mismatch for "a":
after transform: SymbolId(4) "a"
rebuilt        : <None>
Reference flags mismatch for "a":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "foo":
after transform: SymbolId(5) "foo"
rebuilt        : <None>
Reference flags mismatch for "_foo$bar":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["PropertyDescriptor", "babelHelpers"]
rebuilt        : ["a", "babelHelpers", "dec", "foo"]

* oxc/accessor-with-class-properties/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "_a", "_a2", "_a_accessor_storage", "_a_computed_accessor_storage", "_b_accessor_storage", "_c_accessor_storage", "a", "dec"]
rebuilt        : ScopeId(0): ["C", "_a", "_a2", "_a_accessor_storage", "_a_computed_accessor_storage", "_b_accessor_storage", "_c_accessor_storage"]
Reference symbol mismatch for "a":
after transform: SymbolId(4) "a"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_b_accessor_storage":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_b_accessor_storage":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "a":
after transform: SymbolId(4) "a"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(32): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(40): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_default":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_default":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* oxc/computed-key-property-decorator/input.ts
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "MyModel":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* oxc/computed-key-property-decorator-with-initializer/input.ts
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "MyModel":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* oxc/export-class-method-decorated/input.ts
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "T":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* oxc/fields-with-declare-modifier/input.ts
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "DeclareFields":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "DeclareFields":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)

* oxc/metadata/abstract-class/input.ts
Symbol reference IDs mismatch for "Dependency":
after transform: SymbolId(1): [ReferenceId(1), ReferenceId(2), ReferenceId(3)]
rebuilt        : SymbolId(1): [ReferenceId(6), ReferenceId(7)]
Symbol span mismatch for "AbstractClass":
after transform: SymbolId(2): Span { start: 69, end: 82 }
rebuilt        : SymbolId(3): Span { start: 0, end: 0 }
Symbol span mismatch for "AbstractClass":
after transform: SymbolId(5): Span { start: 0, end: 0 }
rebuilt        : SymbolId(4): Span { start: 69, end: 82 }
Reference flags mismatch for "dependency":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

* oxc/metadata/bound-type-reference/input.ts
Symbol reference IDs mismatch for "BoundTypeReference":
after transform: SymbolId(0): [ReferenceId(3), ReferenceId(1), ReferenceId(4), ReferenceId(5), ReferenceId(6)]
rebuilt        : SymbolId(0): [ReferenceId(1), ReferenceId(8), ReferenceId(9)]
Symbol span mismatch for "Example":
after transform: SymbolId(1): Span { start: 87, end: 94 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol span mismatch for "Example":
after transform: SymbolId(4): Span { start: 0, end: 0 }
rebuilt        : SymbolId(3): Span { start: 87, end: 94 }
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* oxc/metadata/class-and-method-decorators/input.ts
Symbol span mismatch for "Problem":
after transform: SymbolId(4): Span { start: 90, end: 97 }
rebuilt        : SymbolId(4): Span { start: 0, end: 0 }
Symbol span mismatch for "Problem":
after transform: SymbolId(5): Span { start: 0, end: 0 }
rebuilt        : SymbolId(5): Span { start: 90, end: 97 }
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Problem":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["ClassDecorator", "String", "babelHelpers"]
rebuilt        : ["String", "babelHelpers", "dec"]

* oxc/metadata/enum-types/input.ts
Bindings mismatch:
after transform: ScopeId(1): ["StringEnum", "bar", "foo"]
rebuilt        : ScopeId(1): ["StringEnum"]
Scope flags mismatch:
after transform: ScopeId(1): ScopeFlags(0x0)
rebuilt        : ScopeId(1): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(2): ["TemplateStringEnum", "mixed", "template"]
rebuilt        : ScopeId(2): ["TemplateStringEnum"]
Scope flags mismatch:
after transform: ScopeId(2): ScopeFlags(0x0)
rebuilt        : ScopeId(2): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(3): ["NumberEnum", "a", "b"]
rebuilt        : ScopeId(3): ["NumberEnum"]
Scope flags mismatch:
after transform: ScopeId(3): ScopeFlags(0x0)
rebuilt        : ScopeId(3): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(4): ["UnaryEnum", "bitwise", "negative", "positive"]
rebuilt        : ScopeId(4): ["UnaryEnum"]
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(0x0)
rebuilt        : ScopeId(4): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(6): ["UnaryOtherEnum", "bitwise", "negative", "positive"]
rebuilt        : ScopeId(6): ["UnaryOtherEnum"]
Scope flags mismatch:
after transform: ScopeId(6): ScopeFlags(0x0)
rebuilt        : ScopeId(6): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(7): ["AutoIncrementEnum", "first", "second", "third"]
rebuilt        : ScopeId(7): ["AutoIncrementEnum"]
Scope flags mismatch:
after transform: ScopeId(7): ScopeFlags(0x0)
rebuilt        : ScopeId(7): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(8): ["MixedEnum", "num", "str"]
rebuilt        : ScopeId(8): ["MixedEnum"]
Scope flags mismatch:
after transform: ScopeId(8): ScopeFlags(0x0)
rebuilt        : ScopeId(8): ScopeFlags(Function)
Bindings mismatch:
after transform: ScopeId(9): ["ComputedEnum", "computed", "expression"]
rebuilt        : ScopeId(9): ["ComputedEnum"]
Scope flags mismatch:
after transform: ScopeId(9): ScopeFlags(0x0)
rebuilt        : ScopeId(9): ScopeFlags(Function)
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
Reference flags mismatch for "StringEnum":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "StringEnum":
after transform: ReferenceId(25): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "TemplateStringEnum":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "TemplateStringEnum":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "NumberEnum":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryEnum":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryEnum":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryEnum":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryEnum":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryEnum":
after transform: ReferenceId(43): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryEnum":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryOtherEnum":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryOtherEnum":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryOtherEnum":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryOtherEnum":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryOtherEnum":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(28): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnaryOtherEnum":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "AutoIncrementEnum":
after transform: ReferenceId(55): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "AutoIncrementEnum":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "AutoIncrementEnum":
after transform: ReferenceId(57): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "AutoIncrementEnum":
after transform: ReferenceId(56): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "AutoIncrementEnum":
after transform: ReferenceId(59): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "AutoIncrementEnum":
after transform: ReferenceId(58): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "MixedEnum":
after transform: ReferenceId(62): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "MixedEnum":
after transform: ReferenceId(64): ReferenceFlags(Read)
rebuilt        : ReferenceId(42): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "MixedEnum":
after transform: ReferenceId(63): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "ComputedEnum":
after transform: ReferenceId(68): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "ComputedEnum":
after transform: ReferenceId(67): ReferenceFlags(Read)
rebuilt        : ReferenceId(47): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "ComputedEnum":
after transform: ReferenceId(70): ReferenceFlags(Read)
rebuilt        : ReferenceId(49): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "ComputedEnum":
after transform: ReferenceId(69): ReferenceFlags(Read)
rebuilt        : ReferenceId(50): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(76): ReferenceFlags(Read)
rebuilt        : ReferenceId(54): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(74): ReferenceFlags(Read)
rebuilt        : ReferenceId(56): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(75): ReferenceFlags(Read)
rebuilt        : ReferenceId(58): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(80): ReferenceFlags(Read)
rebuilt        : ReferenceId(59): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(78): ReferenceFlags(Read)
rebuilt        : ReferenceId(61): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(79): ReferenceFlags(Read)
rebuilt        : ReferenceId(63): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(84): ReferenceFlags(Read)
rebuilt        : ReferenceId(64): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(82): ReferenceFlags(Read)
rebuilt        : ReferenceId(66): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(83): ReferenceFlags(Read)
rebuilt        : ReferenceId(68): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(88): ReferenceFlags(Read)
rebuilt        : ReferenceId(69): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(86): ReferenceFlags(Read)
rebuilt        : ReferenceId(71): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(87): ReferenceFlags(Read)
rebuilt        : ReferenceId(73): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(92): ReferenceFlags(Read)
rebuilt        : ReferenceId(74): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(90): ReferenceFlags(Read)
rebuilt        : ReferenceId(76): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(91): ReferenceFlags(Read)
rebuilt        : ReferenceId(78): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(96): ReferenceFlags(Read)
rebuilt        : ReferenceId(79): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(94): ReferenceFlags(Read)
rebuilt        : ReferenceId(81): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(95): ReferenceFlags(Read)
rebuilt        : ReferenceId(83): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(100): ReferenceFlags(Read)
rebuilt        : ReferenceId(84): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(98): ReferenceFlags(Read)
rebuilt        : ReferenceId(86): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(99): ReferenceFlags(Read)
rebuilt        : ReferenceId(88): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(104): ReferenceFlags(Read)
rebuilt        : ReferenceId(89): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(102): ReferenceFlags(Read)
rebuilt        : ReferenceId(91): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(103): ReferenceFlags(Read)
rebuilt        : ReferenceId(93): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(112): ReferenceFlags(Read)
rebuilt        : ReferenceId(94): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(108): ReferenceFlags(Read)
rebuilt        : ReferenceId(96): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(109): ReferenceFlags(Read)
rebuilt        : ReferenceId(98): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(110): ReferenceFlags(Read)
rebuilt        : ReferenceId(100): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(111): ReferenceFlags(Read)
rebuilt        : ReferenceId(102): ReferenceFlags(Read | MemberWriteTarget)

* oxc/metadata/fields-with-declare-modifier/input.ts
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "DeclareFields":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "DeclareFields":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* oxc/metadata/getter-setter-method/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Getter", "Setter", "UntypedGetter", "UntypedSetter", "dec"]
rebuilt        : ScopeId(0): ["Getter", "Setter", "UntypedGetter", "UntypedSetter"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Getter":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(15): ReferenceFlags(Read)
rebuilt        : ReferenceId(10): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Getter":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UntypedGetter":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(26): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UntypedSetter":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(32): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(33): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Setter":
after transform: ReferenceId(34): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(42): ReferenceFlags(Read)
rebuilt        : ReferenceId(34): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(38): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Setter":
after transform: ReferenceId(41): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Function", "Number", "Object", "PropertyDescriptor", "String", "babelHelpers"]
rebuilt        : ["Function", "Number", "Object", "String", "babelHelpers", "dec"]

* oxc/metadata/imports/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Bar", "Cls", "Foo", "Zoo", "_ref", "dec"]
rebuilt        : ScopeId(0): ["Cls", "Foo", "_ref"]
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(0): [ReferenceId(2), ReferenceId(3), ReferenceId(12), ReferenceId(13)]
rebuilt        : SymbolId(0): [ReferenceId(11), ReferenceId(12)]
Symbol span mismatch for "Cls":
after transform: SymbolId(7): Span { start: 145, end: 148 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol span mismatch for "Cls":
after transform: SymbolId(13): Span { start: 0, end: 0 }
rebuilt        : SymbolId(3): Span { start: 145, end: 148 }
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(3) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Object", "PropertyDescriptor", "babelHelpers", "console"]
rebuilt        : ["Object", "babelHelpers", "console", "dec"]

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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "methodDecorator":
after transform: SymbolId(0) "methodDecorator"
rebuilt        : <None>
Reference symbol mismatch for "methodDecorator":
after transform: SymbolId(0) "methodDecorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(15): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(31): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "methodDecorator":
after transform: SymbolId(0) "methodDecorator"
rebuilt        : <None>
Reference symbol mismatch for "methodDecorator":
after transform: SymbolId(0) "methodDecorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(27): ReferenceFlags(Read)
rebuilt        : ReferenceId(19): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(28): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(29): ReferenceFlags(Read)
rebuilt        : ReferenceId(23): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(30): ReferenceFlags(Read)
rebuilt        : ReferenceId(25): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(40): ReferenceFlags(Read)
rebuilt        : ReferenceId(26): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(38): ReferenceFlags(Read)
rebuilt        : ReferenceId(27): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(35): ReferenceFlags(Read)
rebuilt        : ReferenceId(29): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(36): ReferenceFlags(Read)
rebuilt        : ReferenceId(31): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(37): ReferenceFlags(Read)
rebuilt        : ReferenceId(33): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(39): ReferenceFlags(Read)
rebuilt        : ReferenceId(35): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(51): ReferenceFlags(Read)
rebuilt        : ReferenceId(36): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(48): ReferenceFlags(Read)
rebuilt        : ReferenceId(37): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(49): ReferenceFlags(Read)
rebuilt        : ReferenceId(39): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(45): ReferenceFlags(Read)
rebuilt        : ReferenceId(41): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(46): ReferenceFlags(Read)
rebuilt        : ReferenceId(43): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(47): ReferenceFlags(Read)
rebuilt        : ReferenceId(46): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(50): ReferenceFlags(Read)
rebuilt        : ReferenceId(48): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(54): ReferenceFlags(Read)
rebuilt        : ReferenceId(50): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(52): ReferenceFlags(Read)
rebuilt        : ReferenceId(51): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "paramDecorator":
after transform: SymbolId(2) "paramDecorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(53): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "Cls":
after transform: SymbolId(1) "Cls"
rebuilt        : SymbolId(1) "Cls"
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "Cls":
after transform: SymbolId(1) "Cls"
rebuilt        : SymbolId(1) "Cls"
Reference flags mismatch for "Cls":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "Cls2":
after transform: SymbolId(2) "Cls2"
rebuilt        : SymbolId(2) "Cls2"
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(14): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(16): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "Cls2":
after transform: SymbolId(2) "Cls2"
rebuilt        : SymbolId(2) "Cls2"
Reference flags mismatch for "Cls2":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Unresolved reference IDs mismatch for "babelHelpers":
after transform: [ReferenceId(7), ReferenceId(8), ReferenceId(9), ReferenceId(11), ReferenceId(13), ReferenceId(17), ReferenceId(18), ReferenceId(19), ReferenceId(20), ReferenceId(22), ReferenceId(24)]
rebuilt        : [ReferenceId(0), ReferenceId(3), ReferenceId(5), ReferenceId(6), ReferenceId(8), ReferenceId(9), ReferenceId(12), ReferenceId(14), ReferenceId(16)]

* oxc/metadata/properties/input.ts
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Example":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(18): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Example":
after transform: ReferenceId(17): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* oxc/metadata/static-anonymous-class-expression/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "Foo", "_ref", "dec"]
rebuilt        : ScopeId(0): ["A", "Foo", "_ref"]
Symbol reference IDs mismatch for "A":
after transform: SymbolId(1): [ReferenceId(4), ReferenceId(5), ReferenceId(6)]
rebuilt        : SymbolId(1): [ReferenceId(7), ReferenceId(8)]
Symbol span mismatch for "Foo":
after transform: SymbolId(2): Span { start: 72, end: 75 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol span mismatch for "Foo":
after transform: SymbolId(5): Span { start: 0, end: 0 }
rebuilt        : SymbolId(3): Span { start: 72, end: 75 }
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

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
rebuilt        : SymbolId(1): Span { start: 0, end: 0 }
Symbol span mismatch for "Example":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(2): Span { start: 6, end: 13 }
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "UnboundTypeReference":
after transform: ReferenceId(2): ReferenceFlags(Read | Type)
rebuilt        : ReferenceId(6): ReferenceFlags(Read)
Reference flags mismatch for "UnboundTypeReference":
after transform: ReferenceId(3): ReferenceFlags(Read | Type)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)
Unresolved reference IDs mismatch for "UnboundTypeReference":
after transform: [ReferenceId(1), ReferenceId(2), ReferenceId(3)]
rebuilt        : [ReferenceId(6), ReferenceId(7)]

* oxc/metadata/without-decorator/input.ts
Symbol span mismatch for "C":
after transform: SymbolId(2): Span { start: 106, end: 107 }
rebuilt        : SymbolId(2): Span { start: 0, end: 0 }
Symbol span mismatch for "C":
after transform: SymbolId(3): Span { start: 0, end: 0 }
rebuilt        : SymbolId(3): Span { start: 106, end: 107 }
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "B":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "_Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "_Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["ClassDecorator", "babelHelpers", "console"]
rebuilt        : ["babelHelpers", "console"]

* oxc/use-define-for-class-fields/input.ts
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cls":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "StaticCls":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(11): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "dec":
after transform: ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(8): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(16): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "dec":
after transform: ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(14): ReferenceFlags(Read)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(19): ReferenceFlags(Read)
rebuilt        : ReferenceId(17): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(21): ReferenceFlags(Read)
rebuilt        : ReferenceId(18): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "dec":
after transform: ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
rebuilt        : ReferenceId(19): ReferenceFlags(Read)
Reference flags mismatch for "F":
after transform: ReferenceId(20): ReferenceFlags(Read)
rebuilt        : ReferenceId(20): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(22): ReferenceFlags(Read)
rebuilt        : ReferenceId(21): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(24): ReferenceFlags(Read)
rebuilt        : ReferenceId(22): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "G":
after transform: ReferenceId(23): ReferenceFlags(Read)
rebuilt        : ReferenceId(24): ReferenceFlags(Read | MemberWriteTarget)

* oxc/with-class-private-properties-unnamed-default-export/input.ts
Symbol flags mismatch for "_default":
after transform: SymbolId(0): SymbolFlags(Class)
rebuilt        : SymbolId(0): SymbolFlags(BlockScopedVariable)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* oxc/with-typescript-remove-class-properties-without-initializer/input.ts
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Cls":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)

* typescript/accessor/decoratorOnClassAccessor1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/accessor/decoratorOnClassAccessor5/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(12): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)

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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod10/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod14/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo", "decorator"]
rebuilt        : ScopeId(0): ["Foo"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "decorator":
after transform: SymbolId(0) "decorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Function", "babelHelpers"]
rebuilt        : ["Function", "babelHelpers", "decorator"]

* typescript/method/decoratorOnClassMethod15/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo", "decorator"]
rebuilt        : ScopeId(0): ["Foo"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "decorator":
after transform: SymbolId(0) "decorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Function", "babelHelpers"]
rebuilt        : ["Function", "babelHelpers", "decorator"]

* typescript/method/decoratorOnClassMethod16/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["Foo", "decorator"]
rebuilt        : ScopeId(0): ["Foo"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "decorator":
after transform: SymbolId(0) "decorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "decorator":
after transform: SymbolId(0) "decorator"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "Foo":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["Object", "babelHelpers", "decorator"]

* typescript/method/decoratorOnClassMethod19/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod5/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod6/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod7/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/decoratorOnClassMethod8/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["TypedPropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Object", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/method/parameter/decoratorOnClassMethodParameter3/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["dec", "fn"]
rebuilt        : ScopeId(0): ["fn"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "Class":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "value":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "C":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "C":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "value":
after transform: ReferenceId(8): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(10): ReferenceFlags(Read)
rebuilt        : ReferenceId(5): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(14): ReferenceFlags(Read)
rebuilt        : ReferenceId(11): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(13): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["PropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty1/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty10/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty11/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty12/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "dec"]
rebuilt        : ScopeId(0): ["A"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "A":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["String", "babelHelpers"]
rebuilt        : ["String", "babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty13/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "value":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(4): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["PropertyDescriptor", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty2/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
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
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Function", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/property/decoratorOnClassProperty7/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(0): ReferenceFlags(Read | MemberWriteTarget)
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Reference flags mismatch for "C":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(2): ReferenceFlags(Read | MemberWriteTarget)
Unresolved references mismatch:
after transform: ["Function", "babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]


# plugin-styled-components (19/40)
* minify-comments/input.js
Unresolved references mismatch:
after transform: ["x", "y", "z"]
rebuilt        : ["x", "z"]

* styled-components/add-display-names/input.js
Reference flags mismatch for "styled":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(1): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "styled":
after transform: ReferenceId(7): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "ClassComponent":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

* styled-components/add-identifier/input.js
Reference flags mismatch for "styled":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* styled-components/add-identifier-and-display-name/input.js
Reference flags mismatch for "styled":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "styled":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(9): ReferenceFlags(Read | MemberWriteTarget)

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

* styled-components/named-styled-import/input.js
Reference flags mismatch for "styled":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

* styled-components/pre-transpiled/input.js
x Output mismatch

* styled-components/track-the-imported-variable/input.js
Reference flags mismatch for "s":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(6): ReferenceFlags(Read | MemberWriteTarget)

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

* styled-components/transpile-template-literals-with-config/input.js
Reference flags mismatch for "styled":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)


# plugin-tagged-template-transform (1/12)
* basic/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* escape-sequence/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* invalid-escape/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* mixed-case/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* multiple/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(5): ReferenceFlags(Read)
rebuilt        : ReferenceId(7): ReferenceFlags(Read | MemberWriteTarget)

* multiple-expressions/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(3): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* nested-scope/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* opening-and-closing/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* string-raw/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(6): ReferenceFlags(Read)
rebuilt        : ReferenceId(4): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(9): ReferenceFlags(Read)
rebuilt        : ReferenceId(8): ReferenceFlags(Read | MemberWriteTarget)
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(12): ReferenceFlags(Read)
rebuilt        : ReferenceId(13): ReferenceFlags(Read | MemberWriteTarget)

* uppercase/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(1): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)

* with-expression/input.js
Reference flags mismatch for "babelHelpers":
after transform: ReferenceId(2): ReferenceFlags(Read)
rebuilt        : ReferenceId(3): ReferenceFlags(Read | MemberWriteTarget)


