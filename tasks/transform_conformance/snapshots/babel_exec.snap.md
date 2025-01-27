commit: acbc09a8

node: v22.12.0
[?25l
Passed: 318 of 406 (78.33%)

Failures:

./fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js
'eval' and 'arguments' cannot be used as a binding identifier in strict mode

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js:8:19

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js
AssertionError: expected undefined to be 'hello' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js:21:28

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js
Unexpected token `[`. Expected * for generator, private key, identifier or async

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js
AssertionError: expected undefined to be 'hello' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js:22:28

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js
TypeError: Cannot convert undefined or null to object
    at hasOwnProperty (<anonymous>)
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:26)
    at value (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:63:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:44:198
    at j (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:45:6)
    at Function.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:52:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:71:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js
TypeError: Cannot convert undefined or null to object
    at hasOwnProperty (<anonymous>)
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:26)
    at value (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:123:11)
    at Function.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:24:134)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:131:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js
TypeError: Cannot read properties of undefined (reading 'bind')
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:20:59)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:78:12

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js
TypeError: Cannot read properties of undefined (reading 'bind')
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:20:59)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:78:12

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js
TypeError: e.has is not a function
    at _assertClassBrand (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/assertClassBrand.js:2:44)
    at func (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:10:12)
    at Function.method (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:12:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:16:14

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js
AssertionError: expected [Function] to throw error including '@@toPrimitive must return a primitive…' but got 'Cannot convert object to primitive va…'
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@3.0.2/node_modules/@vitest/expect/dist/index.js:1648:21)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@3.0.2/node_modules/@vitest/expect/dist/index.js:1037:17)
    at Proxy.methodWrapper (./node_modules/.pnpm/chai@5.1.2/node_modules/chai/chai.js:1610:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js:37:5

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js:25:5

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js:8:19

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js:9:19

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at getFoo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js:12:17)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js:13:9

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-new-target-exec.test.js
AssertionError: expected [Function Base] to be undefined // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-new-target-exec.test.js:10:29

./fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js
TypeError: Cannot read properties of undefined (reading 'x')
    at m (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:10:16)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:25:63)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:68:12

./fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js
TypeError: Cannot read properties of undefined (reading 'x')
    at m (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:10:16)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:25:63)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:68:12

./fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js
TypeError: Cannot read properties of undefined (reading 'x')
    at m (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:10:16)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:25:63)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:68:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js:10:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js:30:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js:9:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js:21:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js:11:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js:22:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js:9:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js:18:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-reassignment-exec.test.js
AssertionError: expected +0 to be 1 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-reassignment-exec.test.js:17:18

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js:11:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js:23:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js:10:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js:49:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:10:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:30:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:9:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:21:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:11:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:22:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:9:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:18:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:11:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:23:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:10:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:49:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: "#privateFieldValue" is write-only
    at _writeOnlyError (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/writeOnlyError.js:2:9)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:14:18)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:20:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js:15:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js:17:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js:10:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js:24:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js:9:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js:21:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js:34:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js:9:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js:17:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js:18:14

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-reassignment-exec.test.js
AssertionError: expected +0 to be 1 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-reassignment-exec.test.js:17:18

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js
ReferenceError: _Sub_brand is not defined
    at new Sub (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js:16:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js:29:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:15:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:17:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:11:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:25:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:9:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:21:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:34:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:9:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:17:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:18:14

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js
ReferenceError: _Sub_brand is not defined
    at new Sub (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:16:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:29:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:17:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js:20:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js:26:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js:7:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js:14:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js:9:19)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js:17:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js:7:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js:14:23

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js:7:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js:18:18

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Function.check (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js:17:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js:24:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Function.extract (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js:17:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js:27:25

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:20:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:26:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:7:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:14:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:9:19)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:17:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:7:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:14:23

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:7:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:18:18

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Function.check (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:17:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:24:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Function.extract (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:17:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:27:25

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/writeOnlyError.js:2:9)
    at Function.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:7:15)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:14:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:12:9)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:21:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:24:4

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Function.getValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js:10:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js:27:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js:10:5)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js:22:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/writeOnlyError.js:2:9)
    at Function.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js:11:15)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js:22:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Function.testUpdates (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js:24:4)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js:47:5

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:12:9)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:21:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:24:4

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Function.getValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:10:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:27:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:10:5)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:22:2

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/writeOnlyError.js:2:9)
    at Function.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:11:15)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:22:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:44)
    at Function.testUpdates (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:25:4)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:48:5

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./node_modules/.pnpm/@babel+runtime@7.26.7/node_modules/@babel/runtime/helpers/writeOnlyError.js:2:9)
    at Function.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:7:15)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:14:12

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:19:13

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-loose-rhs-not-object-exec.test.js
AssertionError: expected [Function] to throw error including 'right-hand side of \'in\' should be a…' but got '_Class_brand is not defined'
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@3.0.2/node_modules/@vitest/expect/dist/index.js:1648:21)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@3.0.2/node_modules/@vitest/expect/dist/index.js:1037:17)
    at Proxy.methodWrapper (./node_modules/.pnpm/chai@5.1.2/node_modules/chai/chai.js:1610:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-loose-rhs-not-object-exec.test.js:176:5

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-loose-static-shadow-exec.test.js
AssertionError: expected 2 to be 5 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-loose-static-shadow-exec.test.js:18:25

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-static-shadow-exec.test.js
AssertionError: expected 2 to be 5 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-private-static-shadow-exec.test.js:18:25

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-to-native-fields-half-constructed-static-exec.test.js
AssertionError: expected true to be false // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-to-native-fields-half-constructed-static-exec.test.js:29:15

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-to-native-fields-static-shadow-exec.test.js
AssertionError: expected 2 to be 5 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-to-native-fields-static-shadow-exec.test.js:18:25

./fixtures/babel/babel-plugin-transform-react-jsx-source-test-fixtures-react-source-basic-sample-exec.test.js
ReferenceError: transformAsync is not defined
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-react-jsx-source-test-fixtures-react-source-basic-sample-exec.test.js:4:16

./fixtures/babel/babel-plugin-transform-react-jsx-source-test-fixtures-react-source-with-source-exec.test.js
ReferenceError: transformAsync is not defined
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-react-jsx-source-test-fixtures-react-source-with-source-exec.test.js:4:16

./fixtures/babel/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js
AssertionError: expected [Function] to not throw an error but 'ReferenceError: x is not defined' was thrown
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@3.0.2/node_modules/@vitest/expect/dist/index.js:1648:21)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@3.0.2/node_modules/@vitest/expect/dist/index.js:1037:17)
    at Proxy.methodWrapper (./node_modules/.pnpm/chai@5.1.2/node_modules/chai/chai.js:1610:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js:6:9

./fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js
TypeError: Assignment to constant variable.
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6

./fixtures/babel/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js
AssertionError: expected false to be true // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js:10:37
[?25h