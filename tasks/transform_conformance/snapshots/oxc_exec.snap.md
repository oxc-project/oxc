commit: 41d96516

node: v22.20.0

Passed: 9 of 11 (81.82%)

Failures:

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js
AssertionError: expected [Function] to throw error including 'Receiver must be an instance of class…' but got 'Private element is not present on thi…'
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@3.2.4/node_modules/@vitest/expect/dist/index.js:1420:16)
    at Proxy.<anonymous> (./node_modules/.pnpm/@vitest+expect@3.2.4/node_modules/@vitest/expect/dist/index.js:1029:14)
    at Proxy.methodWrapper (./node_modules/.pnpm/chai@5.3.3/node_modules/chai/index.js:1686:25)
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-private-field-resolve-to-method-in-computed-key-exec.test.js:96:33

./fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js
AssertionError: expected undefined to be [Function C] // Object.is equality
    at ./tasks/transform_conformance/fixtures/oxc/babel-plugin-transform-class-properties-test-fixtures-static-super-tagged-template-exec.test.js:15:17
