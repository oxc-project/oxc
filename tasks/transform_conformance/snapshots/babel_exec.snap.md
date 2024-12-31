commit: 54a8389f

node: v22.12.0

Passed: 330 of 362 (91.16%)

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
Private field '#foo' must be declared in an enclosing class

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

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js
TypeError: '#privateFieldValue' was defined without a getter
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js:16:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js:19:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
TypeError: '#privateFieldValue' was defined without a getter
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:16:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:19:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: '#privateFieldValue' was defined without a getter
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:13:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:16:13

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js
TypeError: '#privateStaticFieldValue' was defined without a getter
    at Function.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:9:14)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:13:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js
TypeError: '#privateStaticFieldValue' was defined without a getter
    at Function.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js:12:14)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js:19:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
TypeError: '#privateStaticFieldValue' was defined without a getter
    at Function.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:12:14)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:19:12

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: '#privateStaticFieldValue' was defined without a getter
    at Function.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:9:14)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:13:12

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
