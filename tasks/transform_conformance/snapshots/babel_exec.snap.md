commit: 54a8389f

node: v22.12.0

Passed: 225 of 362 (62.15%)

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
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:26)
    at value (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:63:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:44:198
    at j (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:45:6)
    at Function.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:52:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:71:6

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js
TypeError: Cannot convert undefined or null to object
    at hasOwnProperty (<anonymous>)
    at _classPrivateFieldBase (./node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/classPrivateFieldLooseBase.js:2:26)
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

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-self-method-exec.test.js
Private field '#bar' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js
TypeError: e.has is not a function
    at _assertClassBrand (./node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:2:44)
    at func (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:10:12)
    at Function.method (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:12:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:16:14

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js
AssertionError: expected [Function] to throw error including '@@toPrimitive must return a primitive…' but got 'Cannot convert object to primitive va…'
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:1438:21)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:923:17)
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

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-in-exec.test.js
Private field '#bar' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-new-target-exec.test.js
AssertionError: expected [Function Base] to be undefined // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-new-target-exec.test.js:10:29

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-private-in-exec.test.js
Private field '#bar' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-private-methods-access-exec.test.js
ReferenceError: _Foo_brand is not defined
    at getFoo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-private-methods-access-exec.test.js:17:35)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-private-methods-access-exec.test.js:18:9

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

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-arguments-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-basic-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-class-binding-exec.test.js
Private field '#getA' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-get-only-setter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-helper-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js
Private field '#getA' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-reassignment-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js
Private field '#getA' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-basic-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-class-binding-exec.test.js
Private field '#getA' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-helper-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-set-only-getter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-updates-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-reassignment-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-set-only-getter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-updates-bigint-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-updates-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-class-binding-exec.test.js
AssertionError: expected null to be [Function A] // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-class-binding-exec.test.js:20:28

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-context-exec.test.js
Private field '#getStatus' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-exfiltrated-exec.test.js
Private field '#privateMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js
Private field '#privateMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js
Private field '#method' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js
Private field '#getA' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js
Private field '#getStatus' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js
Private field '#privateMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-reassignment-exec.test.js
Private field '#privateMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js
Private field '#privateMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js
Private field '#method' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js
Private field '#getA' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js
Private field '#getStatus' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js
Private field '#privateMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-class-binding-exec.test.js
AssertionError: expected null to be [Function A] // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-class-binding-exec.test.js:20:28

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-context-exec.test.js
Private field '#getStatus' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-exfiltrated-exec.test.js
Private field '#privateMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-read-only-exec.test.js
Private field '#method' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-reassignment-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-basic-exec.test.js:21:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-basic-exec.test.js:28:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-class-check-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-class-check-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-class-check-exec.test.js:17:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-exfiltrated-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-generator-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-generator-exec.test.js:18:14

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-reassignment-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js
Private field '#privateMethodA' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-reassignment-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js
Private field '#privateMethodA' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-basic-exec.test.js:21:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-basic-exec.test.js:28:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-class-check-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-class-check-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-class-check-exec.test.js:17:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-exfiltrated-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-generator-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-generator-exec.test.js:18:14

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-reassignment-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-scopable-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-scopable-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-scopable-exec.test.js:22:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsSymbols-this-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-read-only-exec.test.js
Private field '#method' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-reassignment-exec.test.js
Private field '#privateStaticMethod' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-scopable-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-scopable-exec.test.js:8:38)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-scopable-exec.test.js:22:9

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-tagged-template-exec.test.js
ReferenceError: _Foo_brand is not defined
    at Function.getReceiver (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-tagged-template-exec.test.js:11:29)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-tagged-template-exec.test.js:17:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-this-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-access-in-static-field-initializer-exec.test.js
Private field '#p' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-basic-exec.test.js
Private field '#privateStaticFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-destructure-set-exec.test.js
Private field '#p' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js
Private field '#privateStaticFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js
Private field '#p' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js
Private field '#privateStaticFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js
Private field '#p' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js
Private field '#privateStaticFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-set-only-getter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js
Private field '#p' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js
Private field '#privateStaticFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js
Private field '#p' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
Private field '#privateStaticFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-access-in-static-field-initializer-exec.test.js
Private field '#p' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-basic-exec.test.js
Private field '#privateStaticFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-destructure-set-exec.test.js
Private field '#p' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
Private field '#privateStaticFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-set-only-getter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-updates-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-set-only-getter-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-updates-exec.test.js
Private field '#privateFieldValue' must be declared in an enclosing class

./fixtures/babel/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js
AssertionError: expected [Function] to not throw an error but 'ReferenceError: x is not defined' was thrown
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:1438:21)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:923:17)
    at Proxy.methodWrapper (./node_modules/.pnpm/chai@5.1.2/node_modules/chai/chai.js:1610:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js:6:9

./fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js
TypeError: Assignment to constant variable.
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6

./fixtures/babel/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js
AssertionError: expected false to be true // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js:10:37
