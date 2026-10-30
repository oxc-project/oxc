commit: b835f0c6

node: v26.12.1

Passed: 40 of 42 (95.24%)

Failures:

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js
AssertionError: expected [Function] to throw error including 'Receiver must be an instance of class…' but got 'Private element is not present on thi…'
    at Proxy.methodWrapper (./node_modules/chai/index.js:1737:25)
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js:96:33

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js
AssertionError: expected undefined to be [Function C] // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js:15:17
