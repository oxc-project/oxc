commit: 1fb0b771

Passed: 341/397

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


# babel-plugin-transform-class-properties (29/33)
* private-field-resolve-to-method/input.js
x Output mismatch

* private-field-resolve-to-method-in-computed-key/input.js
x Output mismatch

* static-super-assignment-target/input.js
x Output mismatch

* static-super-tagged-template/input.js
x Output mismatch


# babel-plugin-transform-typescript (53/60)
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


* enum-member-reference/input.ts
x Output mismatch

* namespace/redeclaration-with-enum/input.ts
Symbol redeclarations mismatch for "y":
after transform: SymbolId(2): [Span { start: 59, end: 60 }, Span { start: 83, end: 84 }]
rebuilt        : SymbolId(3): []

* optimize-enums/auto-increment-after-string/input.ts
x Output mismatch

* optimize-enums/exported-not-removed/input.ts
x Output mismatch

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

* ts-declaration-empty-output/input.d.ts
x Output mismatch


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


# legacy-decorators (78/105)
* oxc/accessor/input.ts
x Output mismatch

* oxc/class-without-name-with-decorated_class/input.ts
Symbol flags mismatch for "_default":
after transform: SymbolId(1): SymbolFlags(Class)
rebuilt        : SymbolId(1): SymbolFlags(BlockScopedVariable)

* oxc/metadata/class-expression-via-const/input.ts
Symbol reference IDs mismatch for "C":
after transform: SymbolId(0): []
rebuilt        : SymbolId(0): [ReferenceId(3), ReferenceId(5)]
Reference symbol mismatch for "C":
after transform: <None>
rebuilt        : SymbolId(0) "C"
Reference symbol mismatch for "C":
after transform: <None>
rebuilt        : SymbolId(0) "C"
Unresolved references mismatch:
after transform: ["C", "Object", "babelHelpers", "dec"]
rebuilt        : ["Object", "babelHelpers", "dec"]

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
Symbol scope ID mismatch for "Cls":
after transform: SymbolId(4): ScopeId(1)
rebuilt        : SymbolId(1): ScopeId(0)
Symbol reference IDs mismatch for "Cls":
after transform: SymbolId(4): []
rebuilt        : SymbolId(1): [ReferenceId(2), ReferenceId(7)]
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


* oxc/static-field/input.ts
Scope flags mismatch:
after transform: ScopeId(4): ScopeFlags(ClassStaticBlock)
rebuilt        : ScopeId(4): ScopeFlags(StrictMode | ClassStaticBlock)
Scope parent mismatch:
after transform: ScopeId(4): Some(ScopeId(0))
rebuilt        : ScopeId(4): Some(ScopeId(3))

* typescript/accessor/decoratorOnClassAccessor3/input.ts

  x Expected `;` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/accessor/decoratorOnClassAccessor3/input.ts:6:12]
 5 | class C {
 6 |     public @dec get accessor() { return 1; }
   :            |
   :            `-- `;` expected
 7 | }
   `----


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

* typescript/decoratedClassExportsSystem1/input.ts
x Output mismatch

* typescript/decoratorOnClass9/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod11/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod12/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod17/input.ts

  x Expected `;` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/decoratorOnClassMethod17/input.ts:7:18]
 6 | class Foo {
 7 |     private prop @decorator
   :                  |
   :                  `-- `;` expected
 8 |     foo() {
   `----


* typescript/method/decoratorOnClassMethod19/input.ts
x Output mismatch

* typescript/method/decoratorOnClassMethod3/input.ts

  x Expected `;` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/decoratorOnClassMethod3/input.ts:6:12]
 5 | class C {
 6 |     public @dec method() {}
   :            |
   :            `-- `;` expected
 7 | }
   `----


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


* typescript/method/parameter/decoratorOnClassMethodThisParameter/input.ts

  x Identifier expected. 'this' is a reserved word that cannot be used here.
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/method/parameter/decoratorOnClassMethodThisParameter/input.ts:6:17]
 5 | class C {
 6 |     method(@dec this: C) {}
   :                 ^^^^
 7 | }
   `----


* typescript/property/decoratorOnClassProperty3/input.ts

  x Expected `;` but found `@`
   ,-[tasks/transform_conformance/tests/legacy-decorators/test/fixtures/typescript/property/decoratorOnClassProperty3/input.ts:6:12]
 5 | class C {
 6 |     public @dec prop;
   :            |
   :            `-- `;` expected
 7 | }
   `----



# plugin-styled-components (26/40)
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


