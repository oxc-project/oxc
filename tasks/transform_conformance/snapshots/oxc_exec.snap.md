commit: 99dcba5e

node: v24.10.0

Passed: 10 of 12 (83.33%)

Failures:

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js
AssertionError: expected [Function] to throw error including 'Receiver must be an instance of class…' but got 'Private element is not present on thi…'
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@4.0.10/node_modules/@vitest/expect/dist/index.js:1485:16)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@4.0.10/node_modules/@vitest/expect/dist/index.js:1090:14)
    at Proxy.methodWrapper (./node_modules/.pnpm/chai@6.2.1/node_modules/chai/index.js:1700:25)
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js:96:33

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js
AssertionError: expected undefined to be [Function C] // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js:15:17
