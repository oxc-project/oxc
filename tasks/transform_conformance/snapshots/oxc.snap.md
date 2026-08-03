commit: c86e9e4b

Passed: 240/398

# All Passed:
* babel-plugin-transform-private-methods
* babel-plugin-transform-logical-assignment-operators
* babel-plugin-transform-nullish-coalescing-operator
* babel-plugin-transform-optional-chaining
* babel-plugin-transform-optional-catch-binding
* babel-plugin-transform-async-generator-functions
* babel-plugin-transform-exponentiation-operator
* babel-plugin-transform-arrow-functions
* babel-preset-typescript
* babel-plugin-transform-react-jsx-self
* babel-plugin-transform-react-jsx-source
* regexp
* plugin-tagged-template-transform


# babel-plugin-transform-explicit-resource-management (2/4)
* export-class-name/input.js

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-explicit-resource-management/test/fixtures/export-class-name/input.js:3:1]
 2 | 
 3 | export class C {
   : ^^^^^^
 4 |   static getSelf() { return C; }
   `----


* try-catch/input.js

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-explicit-resource-management/test/fixtures/try-catch/input.js:1:1]
 1 | export class WorkspaceResolver {
   : ^^^^^^
 2 |     async invite() {
   `----



# babel-plugin-transform-class-properties (28/33)
* private-field-resolve-to-method/input.js
x Output mismatch

* private-field-resolve-to-method-in-computed-key/input.js
x Output mismatch

* static-super-assignment-target/input.js
x Output mismatch

* static-super-tagged-template/input.js
x Output mismatch

* typescript/class-fields-with-computed-key/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-class-properties/test/fixtures/typescript/class-fields-with-computed-key/input.ts:3:1]
 2 | 
 3 | export class Obj {
   : ^^^^^^
 4 |   public readonly [Collection.identifier] = true;
   `----



# babel-plugin-transform-class-static-block (4/5)
* properties-and-methods/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-class-static-block/test/fixtures/properties-and-methods/input.ts:3:1]
 2 | 
 3 | export class C {
   : ^^^^^^
 4 |   // Private properties and methods use up prop names for static block
   `----



# babel-plugin-transform-object-rest-spread (7/8)
* object-rest/export/input.js

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-object-rest-spread/test/fixtures/object-rest/export/input.js:1:1]
 1 | export let { ...a0 } = foo;
   : ^^^^^^
 2 | export let [{...b0}] = z
   `----


  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-object-rest-spread/test/fixtures/object-rest/export/input.js:2:1]
 1 | export let { ...a0 } = foo;
 2 | export let [{...b0}] = z
   : ^^^^^^
   `----



# babel-plugin-transform-async-to-generator (25/28)
* function/export/default-with-name/input.js

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-async-to-generator/test/fixtures/function/export/default-with-name/input.js:1:1]
 1 | export default async function D(a, b = 0) {
   : ^^^^^^
 2 |   await Promise.resolve();
   `----


* function/export/default-without-name/input.js

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-async-to-generator/test/fixtures/function/export/default-without-name/input.js:1:1]
 1 | export default async function (a, b = 0) {
   : ^^^^^^
 2 |   await Promise.resolve();
   `----


* function/export/named/input.js

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-async-to-generator/test/fixtures/function/export/named/input.js:1:1]
 1 | export async function named(...args) {
   : ^^^^^^
 2 |   await Promise.resolve();
   `----



# babel-plugin-transform-typescript (29/60)
* allow-declare-fields-false/input.ts
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

* class-constructor-arguments-with-declared-fields/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/class-constructor-arguments-with-declared-fields/input.ts:6:1]
 5 | // purely defensive.)
 6 | export class WithStaticSameName {
   : ^^^^^^
 7 |   static x = 0;
   `----


  x Unexpected export.
    ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/class-constructor-arguments-with-declared-fields/input.ts:11:1]
 10 | 
 11 | export class WithPrivateSameName {
    : ^^^^^^
 12 |   #x = 0;
    `----


* computed-constant-value/input.ts
Unresolved references mismatch:
after transform: ["Infinity", "NaN"]
rebuilt        : ["Infinity"]
Unresolved reference IDs mismatch for "Infinity":
after transform: [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(8), ReferenceId(11), ReferenceId(14), ReferenceId(18)]
rebuilt        : [ReferenceId(2), ReferenceId(5), ReferenceId(8), ReferenceId(12)]

* computed-static-property-with-constructor/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/computed-static-property-with-constructor/input.ts:1:1]
 1 | export class SampleClass {
   : ^^^^^^
 2 |     static [Symbol.toPrimitive] = "test";
   `----


* const-enum-value-ref-kept/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/const-enum-value-ref-kept/input.ts:9:1]
 8 | 
 9 | export default Phase;
   : ^^^^^^
   `----


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

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/elimination-declare/input.ts:3:1]
 2 | 
 3 | export declare class ReactiveMarker {
   : ^^^^^^
 4 |   private [ReactiveMarkerSymbol]?: void
   `----


  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/elimination-declare/input.ts:7:1]
 6 | 
 7 | export declare const A = 1
   : ^^^^^^
   `----


* elimination-empty-export-named/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/elimination-empty-export-named/input.ts:1:1]
 1 | export {} from 'mod';
   : ^^^^^^
 2 | export {} from './app.ts';
   `----


  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/elimination-empty-export-named/input.ts:2:1]
 1 | export {} from 'mod';
 2 | export {} from './app.ts';
   : ^^^^^^
 3 | export {}
   `----


  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/elimination-empty-export-named/input.ts:3:1]
 2 | export {} from './app.ts';
 3 | export {}
   : ^^^^^^
   `----


* enum-member-reference/input.ts
Missing ReferenceId: "Foo"
Missing ReferenceId: "Merge"
Missing ReferenceId: "NestInner"
Symbol reference IDs mismatch for "x":
after transform: SymbolId(0): [ReferenceId(2), ReferenceId(4)]
rebuilt        : SymbolId(0): [ReferenceId(7)]
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(14): [ReferenceId(8), ReferenceId(9), ReferenceId(10), ReferenceId(11), ReferenceId(12), ReferenceId(13), ReferenceId(14)]
rebuilt        : SymbolId(2): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(4), ReferenceId(5), ReferenceId(6), ReferenceId(8)]
Symbol redeclarations mismatch for "Merge":
after transform: SymbolId(5): [Span { start: 70, end: 75 }, Span { start: 103, end: 108 }]
rebuilt        : SymbolId(3): []
Symbol reference IDs mismatch for "Merge":
after transform: SymbolId(16): [ReferenceId(20), ReferenceId(21), ReferenceId(22)]
rebuilt        : SymbolId(5): [ReferenceId(16), ReferenceId(17), ReferenceId(18), ReferenceId(19)]
Symbol reference IDs mismatch for "NestInner":
after transform: SymbolId(18): [ReferenceId(31), ReferenceId(32), ReferenceId(33), ReferenceId(34), ReferenceId(35)]
rebuilt        : SymbolId(9): [ReferenceId(25), ReferenceId(26), ReferenceId(28), ReferenceId(29), ReferenceId(30), ReferenceId(31)]

* enum-string-alias-member/input.ts
Symbol reference IDs mismatch for "Color":
after transform: SymbolId(4): [ReferenceId(5), ReferenceId(6), ReferenceId(7), ReferenceId(12)]
rebuilt        : SymbolId(0): [ReferenceId(6), ReferenceId(7), ReferenceId(8)]

* enum-template-literal/input.ts
Symbol reference IDs mismatch for "Size":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(2), ReferenceId(7)]
rebuilt        : SymbolId(0): [ReferenceId(3)]
Symbol reference IDs mismatch for "Animal":
after transform: SymbolId(3): [ReferenceId(1), ReferenceId(3), ReferenceId(11)]
rebuilt        : SymbolId(2): [ReferenceId(7)]

* enum-template-literal-number/input.ts
Symbol reference IDs mismatch for "NumberEnum":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(3), ReferenceId(13)]
rebuilt        : SymbolId(0): [ReferenceId(9)]

* enum-template-literal-trailing-quasi/input.ts
Symbol reference IDs mismatch for "NumberEnum":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1), ReferenceId(2), ReferenceId(8)]
rebuilt        : SymbolId(0): [ReferenceId(5)]

* export-elimination/input.ts

  x Unexpected export.
    ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/export-elimination/input.ts:11:1]
 10 | 
 11 | export { Im, Ok, Foo, Bar, Func, Baz, Baq, Name };
    : ^^^^^^
 12 | 
    `----


  x Unexpected export.
    ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/export-elimination/input.ts:17:1]
 16 | }
 17 | export { T }
    : ^^^^^^
    `----


* exports/type-and-non-type/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/exports/type-and-non-type/input.ts:4:1]
 3 | 
 4 | export { type ToastProps, ToastViewport };
   : ^^^^^^
   `----


* jsx/issue-10956/input.tsx

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/jsx/issue-10956/input.tsx:3:1]
 2 | /** @jsxRuntime classic */
 3 | export const foo = <div></div>
   : ^^^^^^
   `----


* namespace/import-=/input.ts
Symbol reference IDs mismatch for "A":
after transform: SymbolId(0): [ReferenceId(0), ReferenceId(1)]
rebuilt        : SymbolId(0): [ReferenceId(2)]

* namespace/redeclaration-with-enum/input.ts
Symbol redeclarations mismatch for "x":
after transform: SymbolId(0): [Span { start: 10, end: 11 }, Span { start: 39, end: 40 }]
rebuilt        : SymbolId(0): []
Symbol redeclarations mismatch for "y":
after transform: SymbolId(2): [Span { start: 59, end: 60 }, Span { start: 83, end: 84 }]
rebuilt        : SymbolId(3): []

* namespace/redeclaration-with-interface/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/namespace/redeclaration-with-interface/input.ts:1:1]
 1 | export interface Foo {}
   : ^^^^^^
 2 | export namespace Foo {
   `----


  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/namespace/redeclaration-with-interface/input.ts:2:1]
 1 | export interface Foo {}
 2 | export namespace Foo {
   : ^^^^^^
 3 |   export const Bar = 1;
   `----


* namespace/redeclaration-with-type-alias/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/namespace/redeclaration-with-type-alias/input.ts:1:1]
 1 | export type Foo = {};
   : ^^^^^^
 2 | export namespace Foo {
   `----


  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/namespace/redeclaration-with-type-alias/input.ts:2:1]
 1 | export type Foo = {};
 2 | export namespace Foo {
   : ^^^^^^
 3 |     export const Bar = 0;
   `----


  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/namespace/redeclaration-with-type-alias/input.ts:5:1]
 4 | }
 5 | export namespace Foo {
   : ^^^^^^
 6 |     export const Zoo = 1;
   `----


* namespace/redeclaration-with-type-only-namespace/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/namespace/redeclaration-with-type-only-namespace/input.ts:1:1]
 1 | export namespace Foo {
   : ^^^^^^
 2 |     export type T = 0;
   `----


  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/namespace/redeclaration-with-type-only-namespace/input.ts:4:1]
 3 | }
 4 | export namespace Foo {
   : ^^^^^^
 5 |     export const Bar = 1;
   `----


* optimize-enums/exported-not-removed/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/optimize-enums/exported-not-removed/input.ts:1:1]
 1 | export enum Direction {
   : ^^^^^^
 2 |   Up,
   `----


* optimize-enums/merged-enum/input.ts
Unresolved references mismatch:
after transform: ["A"]
rebuilt        : []

* optimize-enums/re-exported-not-removed/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/optimize-enums/re-exported-not-removed/input.ts:5:1]
 4 | enum B { Y = "hello" }
 5 | export { A, B }
   : ^^^^^^
 6 | 
   `----


* redeclarations/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/redeclarations/input.ts:4:1]
 3 | const A: A = 0;
 4 | export {A};
   : ^^^^^^
 5 | 
   `----


  x Unexpected export.
    ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/redeclarations/input.ts:9:1]
  8 | type T = number;
  9 | export { T }
    : ^^^^^^
 10 | 
    `----


  x Unexpected export.
    ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/redeclarations/input.ts:15:1]
 14 | type B = number;
 15 | export { B }
    : ^^^^^^
    `----


* remove-class-properties-without-initializer/input.ts
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

* remove-unused-import-equals/input.ts

  x Unexpected export.
    ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/remove-unused-import-equals/input.ts:17:1]
 16 | 
 17 | export let bar = c
    : ^^^^^^
    `----


* ts-declaration-empty-output/input.d.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/ts-declaration-empty-output/input.d.ts:1:1]
 1 | export interface Things<P, T> {
   : ^^^^^^
 2 |     p: P;
   `----


  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/ts-declaration-empty-output/input.d.ts:6:1]
 5 | 
 6 | export interface Props {
   : ^^^^^^
 7 | }
   `----


  x Unexpected export.
    ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/ts-declaration-empty-output/input.d.ts:9:1]
  8 | 
  9 | export default class MyComponent {
    : ^^^^^^
 10 |     props: Props;
    `----


  x Unexpected export.
    ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/ts-declaration-empty-output/input.d.ts:12:1]
 11 | }
 12 | export namespace Something {
    : ^^^^^^
 13 |     export const foo = 123;
    `----


* ts-private-field-with-remove-class-fields-without-initializer/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-typescript/test/fixtures/ts-private-field-with-remove-class-fields-without-initializer/input.ts:1:1]
 1 | export class ArrayBufferViewTransferable implements Transferable {
   : ^^^^^^
 2 |   #view: ArrayBufferView;
   `----


* use-define-for-class-fields/input.ts
Unresolved references mismatch:
after transform: ["dce"]
rebuilt        : []

* use-define-for-class-fields-without-class-properties/input.ts
Unresolved reference IDs mismatch for "dce":
after transform: [ReferenceId(0), ReferenceId(1), ReferenceId(4), ReferenceId(9), ReferenceId(12), ReferenceId(14), ReferenceId(17)]
rebuilt        : [ReferenceId(5)]


# babel-plugin-transform-react-jsx (50/54)
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

* spread-props-classic/input.jsx

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/babel-plugin-transform-react-jsx/test/fixtures/spread-props-classic/input.jsx:1:1]
 1 | export function Foo(props) {
   : ^^^^^^
 2 |   return (
   `----



# legacy-decorators (13/106)
* oxc/accessor/input.ts
x Output mismatch

* oxc/accessor-name-collision/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/accessor-name-collision/input.ts:6:1]
 5 | 
 6 | export class Foo {
   : ^^^^^^
 7 |   @property()
   `----


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

* oxc/class-without-name-with-decorated-static-element/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/class-without-name-with-decorated-static-element/input.ts:3:1]
 2 | 
 3 | export default class {
   : ^^^^^^
 4 |   @dec
   `----


* oxc/class-without-name-with-decorated_class/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/class-without-name-with-decorated_class/input.ts:4:1]
 3 | @dec
 4 | export default class {
   : ^^^^^^
 5 |   @dec
   `----


* oxc/class-without-name-with-decorated_element/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/class-without-name-with-decorated_element/input.ts:3:1]
 2 | 
 3 | export default class {
   : ^^^^^^
 4 |   @dec
   `----


* oxc/export-class-method-decorated/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/export-class-method-decorated/input.ts:1:1]
 1 | export class T {
   : ^^^^^^
 2 |   @first() method(@first() test) {
   `----


* oxc/metadata/abstract-class/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/metadata/abstract-class/input.ts:4:1]
 3 | @dce()
 4 | export abstract class AbstractClass {
   : ^^^^^^
 5 |     constructor(public dependency: Dependency) {}
   `----


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

* oxc/metadata/class-and-method-decorators/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/metadata/class-and-method-decorators/input.ts:5:1]
 4 | @singleton()
 5 | export class Problem extends C {
   : ^^^^^^
 6 |   @deco()
   `----


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

  x Unexpected export.
    ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/metadata/enum-types/input.ts:48:1]
 47 | 
 48 | export class Foo {
    : ^^^^^^
 49 |   @decorate
    `----


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

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/metadata/params/input.ts:4:1]
 3 | 
 4 | export class Foo {
   : ^^^^^^
 5 |   @methodDecorator(1)
   `----


* oxc/metadata/private-in-expression-in-decorator/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/metadata/private-in-expression-in-decorator/input.ts:4:1]
 3 | @dec
 4 | export class Cls {
   : ^^^^^^
 5 |   #zoo = 0;
   `----


  x Unexpected export.
    ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/metadata/private-in-expression-in-decorator/input.ts:11:1]
 10 | @dec
 11 | export class Cls2 {
    : ^^^^^^
 12 |   #zoo = 0;
    `----


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

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/metadata/static-anonymous-class-expression/input.ts:5:1]
 4 | @dec()
 5 | export class Foo {
   : ^^^^^^
 6 |   static Error1 = class extends Error {};
   `----


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
Reference flags mismatch for "UnboundTypeReference":
after transform: ReferenceId(2): ReferenceFlags(Read | Type)
rebuilt        : ReferenceId(5): ReferenceFlags(Read)
Reference flags mismatch for "UnboundTypeReference":
after transform: ReferenceId(3): ReferenceFlags(Read | Type)
rebuilt        : ReferenceId(7): ReferenceFlags(Read)

* oxc/static-field/input.ts
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(ClassStaticBlock)
rebuilt        : ScopeId(4): ScopeFlags(StrictMode | ClassStaticBlock)
Scope parent mismatch:
after transform: ScopeId(4): Some(ScopeId(0))
rebuilt        : ScopeId(4): Some(ScopeId(3))
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(2): [ReferenceId(4), ReferenceId(6), ReferenceId(8)]
rebuilt        : SymbolId(3): [ReferenceId(4), ReferenceId(8)]

* oxc/static-field-with-class-properties/input.ts
Symbol reference IDs mismatch for "Foo":
after transform: SymbolId(2): [ReferenceId(4), ReferenceId(6), ReferenceId(8), ReferenceId(10)]
rebuilt        : SymbolId(3): [ReferenceId(4), ReferenceId(6), ReferenceId(10)]

* oxc/with-class-private-properties/input.ts

  x Unexpected export.
    ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/with-class-private-properties/input.ts:10:1]
  9 | @dec
 10 | export class D {
    : ^^^^^^
 11 |   prop = 0;
    `----


  x Unexpected export.
    ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/with-class-private-properties/input.ts:18:1]
 17 | @dec
 18 | export default class E {
    : ^^^^^^
 19 |   prop = 0;
    `----


  x Unexpected export.
    ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/with-class-private-properties/input.ts:30:1]
 29 | 
 30 | export class G {
    : ^^^^^^
 31 |   @dec
    `----


* oxc/with-class-private-properties-unnamed-default-export/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/oxc/with-class-private-properties-unnamed-default-export/input.ts:2:1]
 1 | @dec
 2 | export default class {
   : ^^^^^^
 3 |   prop = 0;
   `----


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

* typescript/constructor/decoratorOnClassConstructor1/input.ts
x Output mismatch

* typescript/constructor/decoratorOnClassConstructor4/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["A", "B", "C", "dec"]
rebuilt        : ScopeId(0): ["A", "B", "C"]
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

  x Unexpected export.
    ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/decoratedClassExportsCommonJS1/input.ts:10:1]
  9 | @Something({ v: () => Testing123 })
 10 | export class Testing123 {
    : ^^^^^^
 11 |     static prop0: string;
    `----


* typescript/decoratedClassExportsCommonJS2/input.ts

  x Unexpected export.
    ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/decoratedClassExportsCommonJS2/input.ts:10:1]
  9 | @Something({ v: () => Testing123 })
 10 | export class Testing123 { }
    : ^^^^^^
    `----


* typescript/decoratedClassExportsSystem1/input.ts

  x Unexpected export.
    ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/decoratedClassExportsSystem1/input.ts:10:1]
  9 | @Something({ v: () => Testing123 })
 10 | export class Testing123 {
    : ^^^^^^
 11 |     static prop0: string;
    `----


* typescript/decoratedClassExportsSystem2/input.ts

  x Unexpected export.
    ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/decoratedClassExportsSystem2/input.ts:10:1]
  9 | @Something({ v: () => Testing123 })
 10 | export class Testing123 { }
    : ^^^^^^
    `----


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
Reference symbol mismatch for "dec":
after transform: SymbolId(0) "dec"
rebuilt        : <None>
Unresolved references mismatch:
after transform: ["babelHelpers"]
rebuilt        : ["babelHelpers", "dec"]

* typescript/decoratorOnClass2/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/decoratorOnClass2/input.ts:7:1]
 6 | @dec
 7 | export class C {
   : ^^^^^^
 8 | }
   `----


* typescript/decoratorOnClass3/input.ts

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/decoratorOnClass3/input.ts:6:1]
 5 | 
 6 | export
   : ^^^^^^
 7 | @dec
   `----


* typescript/decoratorOnClass4/input.ts
Bindings mismatch:
after transform: ScopeId(0): ["C", "dec"]
rebuilt        : ScopeId(0): ["C"]
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


# plugin-styled-components (22/40)
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

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/plugin-styled-components/test/fixtures/styled-components/css-declared-after-component/input.jsx:4:1]
 3 | 
 4 | export default function Example() {
   : ^^^^^^
 5 |   return <div css={someCss}>oops</div>
   `----


* styled-components/does-not-replace-native-with-no-tags/input.js
x Output mismatch

* styled-components/pre-transpiled/input.js
x Output mismatch

* styled-components/transformed-imports-with-jsx-member-expressions/input.jsx
x Output mismatch

* styled-components/transpile-css-prop/input.jsx
x Output mismatch

* styled-components/transpile-css-prop-add-import/input.jsx

  x Flow is not supported
   ,-[tasks/transform_conformance/tests/plugin-styled-components/test/fixtures/styled-components/transpile-css-prop-add-import/input.jsx:1:1]
 1 | // @flow
   : ^^^^^^^^
 2 | import React from 'react'
   `----


* styled-components/transpile-css-prop-add-require/input.jsx

  x Flow is not supported
   ,-[tasks/transform_conformance/tests/plugin-styled-components/test/fixtures/styled-components/transpile-css-prop-add-require/input.jsx:1:1]
 1 | // @flow
   : ^^^^^^^^
 2 | import React from 'react'
   `----


* styled-components/transpile-css-prop-all-options-on/input.jsx
x Output mismatch

* styled-components/transpile-require-default/input.js
x Output mismatch

* styled-components/use-directory-name/input.js

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/plugin-styled-components/test/fixtures/styled-components/use-directory-name/input.js:6:1]
 5 | styled.div``;
 6 | export default styled.button``;
   : ^^^^^^
   `----


* styled-components/use-file-name/input.js

  x Unexpected export.
   ,-[tasks/transform_conformance/tests/plugin-styled-components/test/fixtures/styled-components/use-file-name/input.js:6:1]
 5 | styled.div``;
 6 | export default styled.button``;
   : ^^^^^^
   `----


* styled-components/use-namespace/input.js

  x Unexpected export.
    ,-[tasks/transform_conformance/tests/plugin-styled-components/test/fixtures/styled-components/use-namespace/input.js:23:1]
 22 | 
 23 | export default styled.default.button``
    : ^^^^^^
    `----



