commit: 16a2d671

node: v26.12.1

Passed: 496 of 592 (83.78%)

Failures:

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-decorator-initializer-this-exec.test.js
AssertionError: expected [Function replaced] to be [Function Foo] // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-decorator-initializer-this-exec.test.js:59:20
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-decorator-initializer-this-exec.test.js:27:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-installed-on-correct-class-exec.test.js
AssertionError: expected false to be true // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-installed-on-correct-class-exec.test.js:54:19
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-installed-on-correct-class-exec.test.js:29:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js
TypeError: Private element is not present on this object
    at _assertClassBrand (./npm/runtime/src/helpers/esm/assertClassBrand.js:3:9)
    at B.iter (./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js:52:28)
    at iter.next (<anonymous>)
    at [Symbol.iterator] (./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js:67:24)
    at [Symbol.iterator].next (<anonymous>)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js:70:14
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-environment-super-exec.test.js:31:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:13:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-exec.test.js
TypeError: Private element is not present on this object
    at _assertClassBrand (./npm/runtime/src/helpers/esm/assertClassBrand.js:3:9)
    at Foo.method (./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-exec.test.js:35:5)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-exec.test.js:81:15
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-method-private-instance-element-super-exec.test.js:37:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-access-exec.test.js
TypeError: Private element is not present on this object
    at _assertClassBrand (./npm/runtime/src/helpers/esm/assertClassBrand.js:3:9)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-access-exec.test.js:208:3
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-access-exec.test.js:35:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-destructuring-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-destructuring-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes--to-es2015-replacement-static-private-instance-element-destructuring-exec.test.js:13:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js
TypeError: Private element is not present on this object
    at _assertClassBrand (./npm/runtime/src/helpers/esm/assertClassBrand.js:3:9)
    at B.iter (./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js:138:18)
    at iter.next (<anonymous>)
    at [Symbol.iterator] (./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js:156:24)
    at [Symbol.iterator].next (<anonymous>)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js:159:14
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-elements-super-this-exec.test.js:37:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js
TypeError: Private element is not present on this object
    at _assertClassBrand (./npm/runtime/src/helpers/esm/assertClassBrand.js:3:9)
    at B.iter (./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js:30:24)
    at iter.next (<anonymous>)
    at [Symbol.iterator] (./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js:57:24)
    at [Symbol.iterator].next (<anonymous>)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js:60:14
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-environment-super-exec.test.js:31:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-assumption-ignoreFunctionLength-exec.test.js:13:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-exec.test.js
TypeError: Private element is not present on this object
    at _assertClassBrand (./npm/runtime/src/helpers/esm/assertClassBrand.js:3:9)
    at Foo.method (./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-exec.test.js:35:5)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-exec.test.js:81:15
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-method-private-instance-element-super-exec.test.js:37:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-access-exec.test.js
TypeError: Private element is not present on this object
    at _assertClassBrand (./npm/runtime/src/helpers/esm/assertClassBrand.js:3:9)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-access-exec.test.js:208:3
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-access-exec.test.js:35:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-destructuring-exec.test.js
Error: Transform error
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-destructuring-exec.test.js:2:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-classes-replacement-static-private-instance-element-destructuring-exec.test.js:13:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-value-exec.test.js
AssertionError: expected [Function] to throw an error
    at Proxy.methodWrapper (./node_modules/chai/index.js:1737:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-value-exec.test.js:48:4
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-computed-keys-same-value-exec.test.js:25:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-methods-with-same-key-exec.test.js
AssertionError: expected [Function] to throw an error
    at Proxy.methodWrapper (./node_modules/chai/index.js:1737:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-methods-with-same-key-exec.test.js:42:4
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-duplicated-keys--to-es2015-methods-with-same-key-exec.test.js:23:7

./fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-outer-class-binding-mutated-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-outer-class-binding-mutated-exec.test.js:50:18
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-proposal-decorators-test-fixtures-2023-11-misc--to-es2015-outer-class-binding-mutated-exec.test.js:25:7

./fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js
AssertionError: expected undefined to strictly equal 1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js:7:15
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js
TypeError: Cannot destructure property 'b' of 'undefined' as it is undefined.
    at Object.foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js:8:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-async-to-generator-test-fixtures-async-to-generator-async-complex-params-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js:6:18
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:40:262)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:126:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:39:260)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:119:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:67:8)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:167:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at value (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:61:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:42:163
    at j (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:43:7)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:50:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:69:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at value (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:121:12)
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:17:40)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:129:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:39:253)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:129:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js
AssertionError: expected undefined to deeply equal 1
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:38:251)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:122:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js
TypeError: Cannot read properties of undefined (reading 'bind')
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:12:21)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:76:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js
TypeError: Cannot read properties of undefined (reading 'bind')
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:12:21)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:76:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at Foo.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:67:8)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:167:5
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js
AssertionError: expected [Function] to throw error including '@@toPrimitive must return a primitive…' but got 'Cannot convert object to primitive va…'
    at Proxy.methodWrapper (./node_modules/chai/index.js:1737:25)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js:35:4
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js
AssertionError: expected function to throw an error, but it didn't
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js:18:2
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js:6:18
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js:13:7

./fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js
AssertionError: expected '_Class' to be 'Foo' // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js:7:18
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at getFoo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js:6:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js:7:2
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-class-static-block-test-fixtures-integration-loose-private-methods-access-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-assignment-shadowed-block-scoped-bindings-exec.test.js
ReferenceError: Cannot access 'a' before initialization
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-assignment-shadowed-block-scoped-bindings-exec.test.js:7:20
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-assignment-shadowed-block-scoped-bindings-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-declaration-shadowed-block-scoped-bindings-exec.test.js
ReferenceError: Cannot access 'a' before initialization
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-declaration-shadowed-block-scoped-bindings-exec.test.js:6:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-object-rest-spread-test-fixtures-object-rest-for-x-declaration-shadowed-block-scoped-bindings-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js:28:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-basic-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js:14:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-class-binding-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js:13:2
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-get-only-setter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js:11:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-helper-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-reassignment-exec.test.js
AssertionError: expected +0 to be 1 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-reassignment-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-reassignment-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js:14:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-set-only-getter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js:47:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-loose-updates-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:28:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-basic-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:14:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-class-binding-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:13:2
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:11:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-helper-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:14:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-set-only-getter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:47:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsProperties-updates-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: "#privateFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:12:19)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:11:9
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js:8:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-assignment-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-async-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js:8:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js:16:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-before-fields-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js:14:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-class-binding-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js:32:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-context-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js:10:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-exfiltrated-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js:11:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-generator-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-reassignment-exec.test.js
AssertionError: expected +0 to be 1 // Object.is equality
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-reassignment-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-reassignment-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js
ReferenceError: _Sub_brand is not defined
    at new Sub (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js:8:33)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js:27:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-loose-super-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:8:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-assignment-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-async-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:9:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:16:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-before-fields-exec.test.js:21:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js
ReferenceError: _A_brand is not defined
    at new A (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:14:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-class-binding-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:32:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-context-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:7:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:10:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-exfiltrated-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:11:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-generator-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js
ReferenceError: _Sub_brand is not defined
    at new Sub (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:8:33)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:27:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsProperties-super-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js
ReferenceError: _Cl_brand is not defined
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:10:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-method-privateFieldsAsSymbols-async-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js:14:24)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js:24:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-basic-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js:8:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-class-check-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js:7:21)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js:11:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-exfiltrated-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js:8:21
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-generator-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js:12:15
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-scopable-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Sub.check (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js:17:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-super-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at B.extract (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js:25:24
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-loose-this-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:14:24)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:24:8
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-basic-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:8:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-class-check-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new Cl (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:7:21)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:11:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-exfiltrated-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.test (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:8:21
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-generator-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.publicMethod (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:5:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:12:15
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-scopable-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Sub.check (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:17:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-super-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at B.extract (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:10:7)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:25:24
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-private-static-method-privateFieldsAsProperties-this-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:5:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:8:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-get-only-setter-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:10:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:14:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:17:3
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-access-in-static-field-initializer-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.getValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js:8:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js:25:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-basic-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js:8:6)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js:15:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-destructure-set-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js:9:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js:14:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-get-only-setter-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.testUpdates (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js:17:3)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js:45:4
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-loose-updates-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:10:11)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:14:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:17:3
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-access-in-static-field-initializer-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.getValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:8:12)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:25:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-basic-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at new C (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:8:6)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:15:1
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-destructure-set-exec.test.js:17:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:9:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:14:11
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-get-only-setter-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js
TypeError: attempted to use private field on non-instance
    at _classPrivateFieldBase (./npm/runtime/src/helpers/esm/classPrivateFieldLooseBase.js:2:44)
    at Cl.testUpdates (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:17:5)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:46:4
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsProperties-updates-exec.test.js:19:7

./fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js
TypeError: "#privateStaticFieldValue" is write-only
    at _writeOnlyError (./npm/runtime/src/helpers/esm/writeOnlyError.js:2:9)
    at Cl.getPrivateStaticFieldValue (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:5:16)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:8:7
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-methods-test-fixtures-static-accessors-privateFieldsAsSymbols-get-only-setter-exec.test.js:15:7

./fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js
ReferenceError: _Foo_brand is not defined
    at new Foo (./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:6:39)
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:12:12
    at ./tasks/transform_conformance/fixtures/babel/babel-plugin-transform-private-property-in-object-test-fixtures-assumption-privateFieldsAsProperties-method-exec.test.js:17:7

./fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js
TypeError: Assignment to constant variable.
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:3:1
    at ./tasks/transform_conformance/fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:13:7
