commit: 12619ffe

Passed: 9/35

# All Passed:



# babel-plugin-transform-optional-catch-binding (0/1)
* try-catch-shadow/input.js
  x Scopes mismatch after transform



# babel-plugin-transform-typescript (3/7)
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
  x Bindings Mismatch:
  | previous scope ScopeId(0): ["A", "ReactiveMarkerSymbol"]
  | current  scope ScopeId(0): []


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



# babel-plugin-transform-react-jsx (6/27)
* refresh/can-handle-implicit-arrow-returns/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(23): Some("_s")
  | current  reference ReferenceId(0): None

  x reference Mismatch:
  | previous reference ReferenceId(26): Some("_s2")
  | current  reference ReferenceId(1): None

  x reference Mismatch:
  | previous reference ReferenceId(29): Some("_s3")
  | current  reference ReferenceId(2): None

  x reference Mismatch:
  | previous reference ReferenceId(33): Some("_s4")
  | current  reference ReferenceId(3): None

  x reference Mismatch:
  | previous reference ReferenceId(37): Some("_s5")
  | current  reference ReferenceId(4): None

  x reference Mismatch:
  | previous reference ReferenceId(41): Some("_s6")
  | current  reference ReferenceId(5): None

  x reference Mismatch:
  | previous reference ReferenceId(45): Some("_c")
  | current  reference ReferenceId(45): None

  x reference Mismatch:
  | previous reference ReferenceId(47): Some("_c2")
  | current  reference ReferenceId(47): None

  x reference Mismatch:
  | previous reference ReferenceId(49): Some("_c3")
  | current  reference ReferenceId(49): None

  x reference Mismatch:
  | previous reference ReferenceId(51): Some("_c4")
  | current  reference ReferenceId(51): None

  x reference Mismatch:
  | previous reference ReferenceId(53): Some("_c5")
  | current  reference ReferenceId(53): None


* refresh/does-not-consider-require-like-methods-to-be-hocs/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(21): Some("_c")
  | current  reference ReferenceId(17): None


* refresh/does-not-get-tripped-by-iifes/input.jsx
  x Bindings Mismatch:
  | previous scope ScopeId(0): []
  | current  scope ScopeId(0): ["_s"]

  x Bindings Mismatch:
  | previous scope ScopeId(1): ["_s"]
  | current  scope ScopeId(1): []

  x reference Mismatch:
  | previous reference ReferenceId(3): Some("_s")
  | current  reference ReferenceId(1): None


* refresh/generates-signatures-for-function-declarations-calling-hooks/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(6): Some("_s")
  | current  reference ReferenceId(0): None

  x reference Mismatch:
  | previous reference ReferenceId(10): Some("_c")
  | current  reference ReferenceId(10): None


* refresh/generates-signatures-for-function-expressions-calling-hooks/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(26): Some("_s")
  | current  reference ReferenceId(0): None

  x reference Mismatch:
  | previous reference ReferenceId(32): Some("_s2")
  | current  reference ReferenceId(1): None

  x reference Mismatch:
  | previous reference ReferenceId(38): Some("_s3")
  | current  reference ReferenceId(32): None

  x reference Mismatch:
  | previous reference ReferenceId(41): Some("_c")
  | current  reference ReferenceId(41): None

  x reference Mismatch:
  | previous reference ReferenceId(43): Some("_c2")
  | current  reference ReferenceId(43): None

  x reference Mismatch:
  | previous reference ReferenceId(45): Some("_c3")
  | current  reference ReferenceId(45): None

  x reference Mismatch:
  | previous reference ReferenceId(47): Some("_c4")
  | current  reference ReferenceId(47): None

  x reference Mismatch:
  | previous reference ReferenceId(49): Some("_c5")
  | current  reference ReferenceId(49): None

  x reference Mismatch:
  | previous reference ReferenceId(51): Some("_c6")
  | current  reference ReferenceId(51): None


* refresh/generates-valid-signature-for-exotic-ways-to-call-hooks/input.jsx
  x Scopes mismatch after transform

  x reference Mismatch:
  | previous reference ReferenceId(17): Some("_s2")
  | current  reference ReferenceId(0): None

  x reference Mismatch:
  | previous reference ReferenceId(12): Some("_s")
  | current  reference ReferenceId(2): None

  x reference Mismatch:
  | previous reference ReferenceId(21): Some("_c")
  | current  reference ReferenceId(21): None


* refresh/includes-custom-hooks-into-the-signatures/input.jsx
  x Scopes mismatch after transform

  x reference Mismatch:
  | previous reference ReferenceId(10): Some("_s")
  | current  reference ReferenceId(0): None

  x reference Mismatch:
  | previous reference ReferenceId(14): Some("_s2")
  | current  reference ReferenceId(1): None

  x reference Mismatch:
  | previous reference ReferenceId(19): Some("_s3")
  | current  reference ReferenceId(2): None

  x reference Mismatch:
  | previous reference ReferenceId(23): Some("_c")
  | current  reference ReferenceId(23): None


* refresh/registers-capitalized-identifiers-in-hoc-calls/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(14): Some("_c")
  | current  reference ReferenceId(14): None

  x reference Mismatch:
  | previous reference ReferenceId(16): Some("_c2")
  | current  reference ReferenceId(16): None

  x reference Mismatch:
  | previous reference ReferenceId(18): Some("_c3")
  | current  reference ReferenceId(18): None

  x reference Mismatch:
  | previous reference ReferenceId(20): Some("_c4")
  | current  reference ReferenceId(20): None


* refresh/registers-identifiers-used-in-jsx-at-definition-site/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(53): Some("_c")
  | current  reference ReferenceId(44): None

  x reference Mismatch:
  | previous reference ReferenceId(55): Some("_c2")
  | current  reference ReferenceId(46): None

  x reference Mismatch:
  | previous reference ReferenceId(57): Some("_c3")
  | current  reference ReferenceId(48): None

  x reference Mismatch:
  | previous reference ReferenceId(59): Some("_c4")
  | current  reference ReferenceId(50): None

  x reference Mismatch:
  | previous reference ReferenceId(61): Some("_c5")
  | current  reference ReferenceId(52): None

  x reference Mismatch:
  | previous reference ReferenceId(63): Some("_c6")
  | current  reference ReferenceId(54): None


* refresh/registers-identifiers-used-in-react-create-element-at-definition-site/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(45): Some("_c")
  | current  reference ReferenceId(45): None

  x reference Mismatch:
  | previous reference ReferenceId(47): Some("_c2")
  | current  reference ReferenceId(47): None

  x reference Mismatch:
  | previous reference ReferenceId(49): Some("_c3")
  | current  reference ReferenceId(49): None

  x reference Mismatch:
  | previous reference ReferenceId(51): Some("_c4")
  | current  reference ReferenceId(51): None

  x reference Mismatch:
  | previous reference ReferenceId(53): Some("_c5")
  | current  reference ReferenceId(53): None

  x reference Mismatch:
  | previous reference ReferenceId(55): Some("_c6")
  | current  reference ReferenceId(55): None


* refresh/registers-likely-hocs-with-inline-functions-1/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(18): Some("_c")
  | current  reference ReferenceId(18): None

  x reference Mismatch:
  | previous reference ReferenceId(20): Some("_c2")
  | current  reference ReferenceId(20): None

  x reference Mismatch:
  | previous reference ReferenceId(22): Some("_c3")
  | current  reference ReferenceId(22): None

  x reference Mismatch:
  | previous reference ReferenceId(24): Some("_c4")
  | current  reference ReferenceId(24): None

  x reference Mismatch:
  | previous reference ReferenceId(26): Some("_c5")
  | current  reference ReferenceId(26): None

  x reference Mismatch:
  | previous reference ReferenceId(28): Some("_c6")
  | current  reference ReferenceId(28): None

  x reference Mismatch:
  | previous reference ReferenceId(30): Some("_c7")
  | current  reference ReferenceId(30): None

  x reference Mismatch:
  | previous reference ReferenceId(32): Some("_c8")
  | current  reference ReferenceId(32): None


* refresh/registers-likely-hocs-with-inline-functions-2/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(6): Some("_c")
  | current  reference ReferenceId(6): None

  x reference Mismatch:
  | previous reference ReferenceId(8): Some("_c2")
  | current  reference ReferenceId(8): None

  x reference Mismatch:
  | previous reference ReferenceId(10): Some("_c3")
  | current  reference ReferenceId(10): None


* refresh/registers-likely-hocs-with-inline-functions-3/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(6): Some("_c")
  | current  reference ReferenceId(6): None

  x reference Mismatch:
  | previous reference ReferenceId(8): Some("_c2")
  | current  reference ReferenceId(8): None

  x reference Mismatch:
  | previous reference ReferenceId(10): Some("_c3")
  | current  reference ReferenceId(10): None


* refresh/registers-top-level-exported-function-declarations/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(14): Some("_c")
  | current  reference ReferenceId(13): None

  x reference Mismatch:
  | previous reference ReferenceId(16): Some("_c2")
  | current  reference ReferenceId(15): None

  x reference Mismatch:
  | previous reference ReferenceId(18): Some("_c3")
  | current  reference ReferenceId(17): None


* refresh/registers-top-level-exported-named-arrow-functions/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(12): Some("_c")
  | current  reference ReferenceId(10): None

  x reference Mismatch:
  | previous reference ReferenceId(14): Some("_c2")
  | current  reference ReferenceId(12): None


* refresh/registers-top-level-function-declarations/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(9): Some("_c")
  | current  reference ReferenceId(8): None

  x reference Mismatch:
  | previous reference ReferenceId(11): Some("_c2")
  | current  reference ReferenceId(10): None


* refresh/registers-top-level-variable-declarations-with-arrow-functions/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(12): Some("_c")
  | current  reference ReferenceId(11): None

  x reference Mismatch:
  | previous reference ReferenceId(14): Some("_c2")
  | current  reference ReferenceId(13): None

  x reference Mismatch:
  | previous reference ReferenceId(16): Some("_c3")
  | current  reference ReferenceId(15): None


* refresh/registers-top-level-variable-declarations-with-function-expressions/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(9): Some("_c")
  | current  reference ReferenceId(8): None

  x reference Mismatch:
  | previous reference ReferenceId(11): Some("_c2")
  | current  reference ReferenceId(10): None


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
  x reference Mismatch:
  | previous reference ReferenceId(7): Some("_s")
  | current  reference ReferenceId(0): None

  x reference Mismatch:
  | previous reference ReferenceId(11): Some("_c")
  | current  reference ReferenceId(10): None


* refresh/uses-original-function-declaration-if-it-get-reassigned/input.jsx
  x reference Mismatch:
  | previous reference ReferenceId(6): Some("_c")
  | current  reference ReferenceId(6): None



