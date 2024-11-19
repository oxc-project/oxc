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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[1/12]⎯

⎯⎯⎯⎯⎯⎯ Failed Tests 11 ⎯⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-objectRestNoSymbols-rest-ignore-symbols-exec.test.js > exec
AssertionError: expected true to be false // Object.is equality

- Expected
+ Received

- false
+ true

 ❯ fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-objectRestNoSymbols-rest-ignore-symbols-exec.test.js:12:19
     10|  expect(a).toBe(1);
     11|  expect(r.b).toBe(2);
     12|  expect(sym in r).toBe(false);
       |                   ^
     13| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[2/12]⎯

 FAIL  fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-pureGetters-rest-remove-unused-excluded-keys-exec.test.js > exec
AssertionError: expected true to be false // Object.is equality

- Expected
+ Received

- false
+ true

 ❯ fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-pureGetters-rest-remove-unused-excluded-keys-exec.test.js:10:17
      8|  let { foo,...rest } = obj;
      9|  expect("foo" in rest).toBe(false);
     10|  expect(called).toBe(false);
       |                 ^
     11| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[3/12]⎯

 FAIL  fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-pureGetters-spread-single-call-exec.test.js > exec
AssertionError: expected { foo: +0, middle: 1, bar: 1 } to deeply equal { foo: +0, middle: +0, bar: 1 }

- Expected
+ Received

  Object {
    "bar": 1,
    "foo": 0,
-   "middle": 0,
+   "middle": 1,
  }

 ❯ fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-pureGetters-spread-single-call-exec.test.js:13:14
     11|  } };
     12|  let res = _objectSpread(_objectSpread(_objectSpread({}, withFoo), { m…
     13|  expect(res).toEqual({
       |              ^
     14|   foo: 0,
     15|   middle: 0,

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[4/12]⎯

 FAIL  fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-setSpreadProperties-no-object-assign-exec-exec.test.js > exec
AssertionError: expected [Function] to throw an error
 ❯ fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-setSpreadProperties-no-object-assign-exec-exec.test.js:14:5
     12|  expect(() => {
     13|   const objSpread = _objectSpread({}, obj);
     14|  }).toThrow();
       |     ^
     15|  const obj2 = { "NOWRITE": 456 };
     16|  expect(() => {

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[5/12]⎯

 FAIL  fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-setSpreadProperties-with-useBuiltIns-no-object-assign-exec-exec.test.js > exec
AssertionError: expected [Function] to throw an error
 ❯ fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-assumption-setSpreadProperties-with-useBuiltIns-no-object-assign-exec-exec.test.js:14:5
     12|  expect(() => {
     13|   const objSpread = _objectSpread({}, obj);
     14|  }).toThrow();
       |     ^
     15|  const obj2 = { "NOWRITE": 456 };
     16|  expect(() => {

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[6/12]⎯

 FAIL  fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-expression-exec.test.js > exec
AssertionError: expected [ 1, 2 ] to deeply equal [ 1 ]

- Expected
+ Received

  Array [
    1,
+   2,
  ]

 ❯ fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-expression-exec.test.js:11:14
      9|   log.push(2);
     10|  } });
     11|  expect(log).toEqual([1]);
       |              ^
     12| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[7/12]⎯

 FAIL  fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-loose-builtins-side-effect-exec.test.js > exec
AssertionError: expected { a: 1, b: 1 } to deeply equal { a: 2, b: 1 }

- Expected
+ Received

  Object {
-   "a": 2,
+   "a": 1,
    "b": 1,
  }

 ❯ fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-loose-builtins-side-effect-exec.test.js:9:12
      7|  };
      8|  var o = Object.assign(Object.assign({ a: 3 }, k), { b: k.a++ });
      9|  expect(o).toEqual({
       |            ^
     10|   a: 2,
     11|   b: 1

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[8/12]⎯

 FAIL  fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-loose-side-effect-exec.test.js > exec
AssertionError: expected { a: 1, b: 1 } to deeply equal { a: 2, b: 1 }

- Expected
+ Received

  Object {
-   "a": 2,
+   "a": 1,
    "b": 1,
  }

 ❯ fixtures/babel-plugin-transform-object-rest-spread-test-fixtures-object-spread-loose-side-effect-exec.test.js:9:12
      7|  };
      8|  var o = Object.assign(Object.assign({ a: 3 }, k), { b: k.a++ });
      9|  expect(o).toEqual({
       |            ^
     10|   a: 2,
     11|   b: 1

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[9/12]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[10/12]⎯

 FAIL  fixtures/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js > exec
TypeError: Assignment to constant variable.
 ❯ fixtures/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6
      3| test("exec", () => {
      4|  const one = 123;
      5|  one = 432;
       |      ^
      6| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[11/12]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[12/12]⎯

