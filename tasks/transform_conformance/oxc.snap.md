commit: 12619ffe

Passed: 8/35

# All Passed:



# babel-plugin-transform-optional-catch-binding (0/1)
* try-catch-shadow/input.js
  x Bindings mismatch:
  | after transform: ScopeId(0): ["_unused", "_unused2"]
  | rebuilt        : ScopeId(0): ["_unused"]

  x Scope children mismatch:
  | after transform: ScopeId(0): [ScopeId(1), ScopeId(2)]
  | rebuilt        : ScopeId(0): [ScopeId(1), ScopeId(2)]

  x Bindings mismatch:
  | after transform: No scope
  | rebuilt        : ScopeId(2): []

  x Bindings mismatch:
  | after transform: ScopeId(2): []
  | rebuilt        : ScopeId(3): ["_unused2"]

  x Scope parent mismatch:
  | after transform: ScopeId(2): Some(ScopeId(0))
  | rebuilt        : ScopeId(3): Some(ScopeId(2))

  x Symbol flags mismatch:
  | after transform: SymbolId(1): SymbolFlags(CatchVariable)
  | rebuilt        : SymbolId(1): SymbolFlags(FunctionScopedVariable |
  | CatchVariable)



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

  x Symbol flags mismatch:
  | after transform: SymbolId(1): SymbolFlags(Export | Import | TypeAlias)
  | rebuilt        : SymbolId(1): SymbolFlags(Export | Import)

  x Symbol flags mismatch:
  | after transform: SymbolId(2): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export | Import | TypeAlias)
  | rebuilt        : SymbolId(2): SymbolFlags(BlockScopedVariable |
  | ConstVariable | Export)

  x Symbol span mismatch:
  | after transform: SymbolId(2): Span { start: 267, end: 268 }
  | rebuilt        : SymbolId(2): Span { start: 289, end: 293 }



# babel-plugin-transform-react-jsx (6/27)
* refresh/can-handle-implicit-arrow-returns/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(23): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference mismatch:
  | after transform: ReferenceId(26): Some("_s2")
  | rebuilt        : ReferenceId(1): None

  x Reference mismatch:
  | after transform: ReferenceId(29): Some("_s3")
  | rebuilt        : ReferenceId(2): None

  x Reference mismatch:
  | after transform: ReferenceId(33): Some("_s4")
  | rebuilt        : ReferenceId(3): None

  x Reference mismatch:
  | after transform: ReferenceId(37): Some("_s5")
  | rebuilt        : ReferenceId(4): None

  x Reference mismatch:
  | after transform: ReferenceId(41): Some("_s6")
  | rebuilt        : ReferenceId(5): None

  x Reference mismatch:
  | after transform: ReferenceId(45): Some("_c")
  | rebuilt        : ReferenceId(45): None

  x Reference mismatch:
  | after transform: ReferenceId(47): Some("_c2")
  | rebuilt        : ReferenceId(47): None

  x Reference mismatch:
  | after transform: ReferenceId(49): Some("_c3")
  | rebuilt        : ReferenceId(49): None

  x Reference mismatch:
  | after transform: ReferenceId(51): Some("_c4")
  | rebuilt        : ReferenceId(51): None

  x Reference mismatch:
  | after transform: ReferenceId(53): Some("_c5")
  | rebuilt        : ReferenceId(53): None


* refresh/does-not-consider-require-like-methods-to-be-hocs/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(21): Some("_c")
  | rebuilt        : ReferenceId(17): None


* refresh/does-not-get-tripped-by-iifes/input.jsx
  x Bindings mismatch:
  | after transform: ScopeId(0): []
  | rebuilt        : ScopeId(0): ["_s"]

  x Bindings mismatch:
  | after transform: ScopeId(1): ["_s"]
  | rebuilt        : ScopeId(1): []

  x Reference mismatch:
  | after transform: ReferenceId(3): Some("_s")
  | rebuilt        : ReferenceId(1): None


* refresh/generates-signatures-for-function-declarations-calling-hooks/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(6): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference mismatch:
  | after transform: ReferenceId(10): Some("_c")
  | rebuilt        : ReferenceId(10): None


* refresh/generates-signatures-for-function-expressions-calling-hooks/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(26): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference mismatch:
  | after transform: ReferenceId(32): Some("_s2")
  | rebuilt        : ReferenceId(1): None

  x Reference mismatch:
  | after transform: ReferenceId(38): Some("_s3")
  | rebuilt        : ReferenceId(32): None

  x Reference mismatch:
  | after transform: ReferenceId(41): Some("_c")
  | rebuilt        : ReferenceId(41): None

  x Reference mismatch:
  | after transform: ReferenceId(43): Some("_c2")
  | rebuilt        : ReferenceId(43): None

  x Reference mismatch:
  | after transform: ReferenceId(45): Some("_c3")
  | rebuilt        : ReferenceId(45): None

  x Reference mismatch:
  | after transform: ReferenceId(47): Some("_c4")
  | rebuilt        : ReferenceId(47): None

  x Reference mismatch:
  | after transform: ReferenceId(49): Some("_c5")
  | rebuilt        : ReferenceId(49): None

  x Reference mismatch:
  | after transform: ReferenceId(51): Some("_c6")
  | rebuilt        : ReferenceId(51): None


* refresh/generates-valid-signature-for-exotic-ways-to-call-hooks/input.jsx
  x Scope children mismatch:
  | after transform: ScopeId(0): [ScopeId(1)]
  | rebuilt        : ScopeId(0): [ScopeId(1), ScopeId(3)]

  x Bindings mismatch:
  | after transform: No scope
  | rebuilt        : ScopeId(3): []

  x Reference mismatch:
  | after transform: ReferenceId(17): Some("_s2")
  | rebuilt        : ReferenceId(0): None

  x Reference mismatch:
  | after transform: ReferenceId(12): Some("_s")
  | rebuilt        : ReferenceId(2): None

  x Reference mismatch:
  | after transform: ReferenceId(21): Some("_c")
  | rebuilt        : ReferenceId(21): None


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

  x Reference mismatch:
  | after transform: ReferenceId(10): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference mismatch:
  | after transform: ReferenceId(14): Some("_s2")
  | rebuilt        : ReferenceId(1): None

  x Reference mismatch:
  | after transform: ReferenceId(19): Some("_s3")
  | rebuilt        : ReferenceId(2): None

  x Reference mismatch:
  | after transform: ReferenceId(23): Some("_c")
  | rebuilt        : ReferenceId(23): None


* refresh/registers-capitalized-identifiers-in-hoc-calls/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(14): Some("_c")
  | rebuilt        : ReferenceId(14): None

  x Reference mismatch:
  | after transform: ReferenceId(16): Some("_c2")
  | rebuilt        : ReferenceId(16): None

  x Reference mismatch:
  | after transform: ReferenceId(18): Some("_c3")
  | rebuilt        : ReferenceId(18): None

  x Reference mismatch:
  | after transform: ReferenceId(20): Some("_c4")
  | rebuilt        : ReferenceId(20): None


* refresh/registers-identifiers-used-in-jsx-at-definition-site/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(53): Some("_c")
  | rebuilt        : ReferenceId(44): None

  x Reference mismatch:
  | after transform: ReferenceId(55): Some("_c2")
  | rebuilt        : ReferenceId(46): None

  x Reference mismatch:
  | after transform: ReferenceId(57): Some("_c3")
  | rebuilt        : ReferenceId(48): None

  x Reference mismatch:
  | after transform: ReferenceId(59): Some("_c4")
  | rebuilt        : ReferenceId(50): None

  x Reference mismatch:
  | after transform: ReferenceId(61): Some("_c5")
  | rebuilt        : ReferenceId(52): None

  x Reference mismatch:
  | after transform: ReferenceId(63): Some("_c6")
  | rebuilt        : ReferenceId(54): None


* refresh/registers-identifiers-used-in-react-create-element-at-definition-site/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(45): Some("_c")
  | rebuilt        : ReferenceId(45): None

  x Reference mismatch:
  | after transform: ReferenceId(47): Some("_c2")
  | rebuilt        : ReferenceId(47): None

  x Reference mismatch:
  | after transform: ReferenceId(49): Some("_c3")
  | rebuilt        : ReferenceId(49): None

  x Reference mismatch:
  | after transform: ReferenceId(51): Some("_c4")
  | rebuilt        : ReferenceId(51): None

  x Reference mismatch:
  | after transform: ReferenceId(53): Some("_c5")
  | rebuilt        : ReferenceId(53): None

  x Reference mismatch:
  | after transform: ReferenceId(55): Some("_c6")
  | rebuilt        : ReferenceId(55): None


* refresh/registers-likely-hocs-with-inline-functions-1/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(18): Some("_c")
  | rebuilt        : ReferenceId(18): None

  x Reference mismatch:
  | after transform: ReferenceId(20): Some("_c2")
  | rebuilt        : ReferenceId(20): None

  x Reference mismatch:
  | after transform: ReferenceId(22): Some("_c3")
  | rebuilt        : ReferenceId(22): None

  x Reference mismatch:
  | after transform: ReferenceId(24): Some("_c4")
  | rebuilt        : ReferenceId(24): None

  x Reference mismatch:
  | after transform: ReferenceId(26): Some("_c5")
  | rebuilt        : ReferenceId(26): None

  x Reference mismatch:
  | after transform: ReferenceId(28): Some("_c6")
  | rebuilt        : ReferenceId(28): None

  x Reference mismatch:
  | after transform: ReferenceId(30): Some("_c7")
  | rebuilt        : ReferenceId(30): None

  x Reference mismatch:
  | after transform: ReferenceId(32): Some("_c8")
  | rebuilt        : ReferenceId(32): None


* refresh/registers-likely-hocs-with-inline-functions-2/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(6): Some("_c")
  | rebuilt        : ReferenceId(6): None

  x Reference mismatch:
  | after transform: ReferenceId(8): Some("_c2")
  | rebuilt        : ReferenceId(8): None

  x Reference mismatch:
  | after transform: ReferenceId(10): Some("_c3")
  | rebuilt        : ReferenceId(10): None


* refresh/registers-likely-hocs-with-inline-functions-3/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(6): Some("_c")
  | rebuilt        : ReferenceId(6): None

  x Reference mismatch:
  | after transform: ReferenceId(8): Some("_c2")
  | rebuilt        : ReferenceId(8): None

  x Reference mismatch:
  | after transform: ReferenceId(10): Some("_c3")
  | rebuilt        : ReferenceId(10): None


* refresh/registers-top-level-exported-function-declarations/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(14): Some("_c")
  | rebuilt        : ReferenceId(13): None

  x Reference mismatch:
  | after transform: ReferenceId(16): Some("_c2")
  | rebuilt        : ReferenceId(15): None

  x Reference mismatch:
  | after transform: ReferenceId(18): Some("_c3")
  | rebuilt        : ReferenceId(17): None


* refresh/registers-top-level-exported-named-arrow-functions/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(12): Some("_c")
  | rebuilt        : ReferenceId(10): None

  x Reference mismatch:
  | after transform: ReferenceId(14): Some("_c2")
  | rebuilt        : ReferenceId(12): None


* refresh/registers-top-level-function-declarations/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(9): Some("_c")
  | rebuilt        : ReferenceId(8): None

  x Reference mismatch:
  | after transform: ReferenceId(11): Some("_c2")
  | rebuilt        : ReferenceId(10): None


* refresh/registers-top-level-variable-declarations-with-arrow-functions/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(12): Some("_c")
  | rebuilt        : ReferenceId(11): None

  x Reference mismatch:
  | after transform: ReferenceId(14): Some("_c2")
  | rebuilt        : ReferenceId(13): None

  x Reference mismatch:
  | after transform: ReferenceId(16): Some("_c3")
  | rebuilt        : ReferenceId(15): None


* refresh/registers-top-level-variable-declarations-with-function-expressions/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(9): Some("_c")
  | rebuilt        : ReferenceId(8): None

  x Reference mismatch:
  | after transform: ReferenceId(11): Some("_c2")
  | rebuilt        : ReferenceId(10): None


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
  x Reference mismatch:
  | after transform: ReferenceId(7): Some("_s")
  | rebuilt        : ReferenceId(0): None

  x Reference mismatch:
  | after transform: ReferenceId(11): Some("_c")
  | rebuilt        : ReferenceId(10): None


* refresh/uses-original-function-declaration-if-it-get-reassigned/input.jsx
  x Reference mismatch:
  | after transform: ReferenceId(6): Some("_c")
  | rebuilt        : ReferenceId(6): None



