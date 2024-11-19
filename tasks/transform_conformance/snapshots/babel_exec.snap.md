commit: d20b314c

node: v22.11.0
⎯⎯⎯⎯⎯⎯ Failed Suites 1 ⎯⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js [ fixtures/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js ]
Error: 'eval' and 'arguments' cannot be used as a binding identifier in strict mode
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.24.0/node_modules/rollup/dist/es/shared/parseAst.js:395:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.24.0/node_modules/rollup/dist/es/shared/parseAst.js:1083:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.24.0/node_modules/rollup/dist/es/shared/parseAst.js:2069:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.8_@types+node@22.9.0/node_modules/vite/dist/node/chunks/dep-CDnG8rE7.js:52319:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.8_@types+node@22.9.0/node_modules/vite/dist/node/chunks/dep-CDnG8rE7.js:51917:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[1/4]⎯

⎯⎯⎯⎯⎯⎯⎯ Failed Tests 3 ⎯⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js > exec
AssertionError: expected [Function] to not throw an error but 'ReferenceError: x is not defined' was thrown

- Expected: 
undefined

+ Received: 
"ReferenceError: x is not defined"

 ❯ fixtures/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js:6:9
      4|  expect(() => {
      5|   x = async (x) => 0;
      6|  }).not.toThrow();
       |         ^
      7| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[2/4]⎯

 FAIL  fixtures/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js > exec
TypeError: Assignment to constant variable.
 ❯ fixtures/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6
      3| test("exec", () => {
      4|  const one = 123;
      5|  one = 432;
       |      ^
      6| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[3/4]⎯

 FAIL  fixtures/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js > exec
AssertionError: expected false to be true // Object.is equality

- Expected
+ Received

- true
+ false

 ❯ fixtures/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js:10:37
      8|  expect(/hello.world/.test(input)).toBe(false);
      9|  expect(/hello.world/u.test(input)).toBe(false);
     10|  expect(/hello.world/s.test(input)).toBe(true);
       |                                     ^
     11|  expect(/hello.world/su.test(input)).toBe(true);
     12| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[4/4]⎯

