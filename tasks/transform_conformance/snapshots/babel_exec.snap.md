commit: 54a8389f

node: v22.12.0

Passed: 187 of 215 (86.98%)

Failures:

./fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js
'eval' and 'arguments' cannot be used as a binding identifier in strict mode

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js:8:19

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js
AssertionError: expected undefined to be 'hello' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js:21:28

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js
Unexpected token `[`. Expected * for generator, private key, identifier or async

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js
AssertionError: expected undefined to be 'hello' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js:22:28

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-constructor-collision-exec.test.js
AssertionError: expected undefined to be 'bar' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-constructor-collision-exec.test.js:18:19

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js
AssertionError: expected [Function] to throw error matching /attempted to use private field on non…/ but got 'Cannot read properties of undefined (…'
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:1438:21)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:923:17)
    at Proxy.methodWrapper (./node_modules/.pnpm/chai@5.1.2/node_modules/chai/chai.js:1610:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js:14:5

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-constructor-collision-exec.test.js
AssertionError: expected undefined to be 'bar' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-constructor-collision-exec.test.js:18:19

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-extends-computed-exec.test.js
AssertionError: expected [Function] to not throw an error but 'TypeError: Private element is not pre…' was thrown
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:1438:21)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:923:17)
    at Proxy.methodWrapper (./node_modules/.pnpm/chai@5.1.2/node_modules/chai/chai.js:1610:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-extends-computed-exec.test.js:30:9

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js
TypeError: e.has is not a function
    at _assertClassBrand (./node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:2:44)
    at func (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js:10:12)
    at Function.method (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js:12:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js:16:14

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js
Private field '#foo' must be declared in an enclosing class

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-extends-computed-exec.test.js
AssertionError: expected [Function] to not throw an error but 'TypeError: Private element is not pre…' was thrown
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:1438:21)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@2.1.2/node_modules/@vitest/expect/dist/index.js:923:17)
    at Proxy.methodWrapper (./node_modules/.pnpm/chai@5.1.2/node_modules/chai/chai.js:1610:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-extends-computed-exec.test.js:31:9

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
Invalid access to super

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js:8:19

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js:9:19

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-super-exec.test.js
Invalid access to super

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js
'super' keyword unexpected here

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
