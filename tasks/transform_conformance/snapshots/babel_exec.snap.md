commit: 54a8389f

node: v22.12.0
⎯⎯⎯⎯⎯⎯ Failed Suites 9 ⎯⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js [ fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js ]
Error: 'eval' and 'arguments' cannot be used as a binding identifier in strict mode
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.28.0/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.28.0/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.28.0/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.10.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.10.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[1/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-super-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-super-exec.test.js ]
 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js ]
 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-super-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-super-exec.test.js ]
 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-super-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-super-exec.test.js ]
Error: Invalid access to super
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.28.0/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.28.0/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.28.0/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.10.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.10.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[2/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js ]
Error: Unexpected token `[`. Expected * for generator, private key, identifier or async
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.28.0/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.28.0/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.28.0/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.10.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.10.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[3/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[4/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[5/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js ]
SyntaxError: 'super' keyword unexpected here
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[6/28]⎯

⎯⎯⎯⎯⎯⎯ Failed Tests 19 ⎯⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js > exec
AssertionError: expected '_Class' to be 'Foo' // Object.is equality

Expected: "Foo"
Received: "_Class"

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js:8:19
      6|  expect(Foo.num).toBe(0);
      7|  expect(Foo.num = 1).toBe(1);
      8|  expect(Foo.name).toBe("Foo");
       |                   ^
      9| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[7/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js > exec
AssertionError: expected undefined to be 'hello' // Object.is equality

- Expected: 
"hello"

+ Received: 
undefined

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js:21:28
     19|   }
     20|  }
     21|  expect(new Outer().hello).toBe("hello");
       |                            ^
     22| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[8/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js > exec
AssertionError: expected undefined to be 'hello' // Object.is equality

- Expected: 
"hello"

+ Received: 
undefined

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js:22:28
     20|   }
     21|  }
     22|  expect(new Outer().hello).toBe("hello");
       |                            ^
     23| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[9/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-constructor-collision-exec.test.js > exec
AssertionError: expected undefined to be 'bar' // Object.is equality

- Expected: 
"bar"

+ Received: 
undefined

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-constructor-collision-exec.test.js:18:19
     16|  }
     17|  const f = new Foo();
     18|  expect(f.test()).toBe(foo);
       |                   ^
     19|  expect("bar" in f).toBe(false);
     20| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[10/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js > exec
AssertionError: expected [Function] to throw error matching /attempted to use private field on non…/ but got 'Cannot read properties of undefined (…'

- Expected: 
/attempted to use private field on non-instance/

+ Received: 
"Cannot read properties of undefined (reading '_')"

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js:14:5
     12|   var _ = { _: new _C() };
     13|   var _p = { _: void 0 };
     14|  }).toThrow(/attempted to use private field on non-instance/);
       |     ^
     15|  expect(() => {
     16|   var _C2;

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[11/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-constructor-collision-exec.test.js > exec
AssertionError: expected undefined to be 'bar' // Object.is equality

- Expected: 
"bar"

+ Received: 
undefined

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-constructor-collision-exec.test.js:18:19
     16|  }
     17|  const f = new Foo();
     18|  expect(f.test()).toBe(foo);
       |                   ^
     19|  expect("bar" in f).toBe(false);
     20| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[12/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-extends-computed-exec.test.js > exec
AssertionError: expected [Function] to not throw an error but 'TypeError: Private element is not pre…' was thrown

- Expected: 
undefined

+ Received: 
"TypeError: Private element is not present on this object"

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-extends-computed-exec.test.js:30:9
     28|  expect(() => {
     29|   f.test();
     30|  }).not.toThrow();
       |         ^
     31| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[13/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js > exec
TypeError: e.has is not a function
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:2:44
 ❯ func fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js:10:12
      8|    const func = () => {
      9|     const Test = 3;
     10|     return _assertClassBrand(Test, this, _x)._ + Test;
       |            ^
     11|    };
     12|    return func() + Test;
 ❯ Function.method fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js:12:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js:16:14

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[14/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-extends-computed-exec.test.js > exec
AssertionError: expected [Function] to not throw an error but 'TypeError: Private element is not pre…' was thrown

- Expected: 
undefined

+ Received: 
"TypeError: Private element is not present on this object"

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-extends-computed-exec.test.js:31:9
     29|  expect(() => {
     30|   f.test();
     31|  }).not.toThrow();
       |         ^
     32| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[15/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js > exec
TypeError: e.has is not a function
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:2:44
 ❯ func fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:10:12
      8|    const func = () => {
      9|     const Test = 3;
     10|     return _assertClassBrand(Test, this, _x)._ + Test;
       |            ^
     11|    };
     12|    return func() + Test;
 ❯ Function.method fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:12:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:16:14

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[16/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js > exec
AssertionError: expected [Function] to throw error including '@@toPrimitive must return a primitive…' but got 'Cannot convert object to primitive va…'

Expected: "@@toPrimitive must return a primitive value."
Received: "Cannot convert object to primitive value"

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js:37:5
     35|    return 0;
     36|   }
     37|  }).toThrow("@@toPrimitive must return a primitive value.");
       |     ^
     38|  expect(() => class {
     39|   static get [arrayLike]() {

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[17/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js > exec
AssertionError: expected '_Class' to be 'Foo' // Object.is equality

Expected: "Foo"
Received: "_Class"

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js:8:19
      6|  expect(Foo.num).toBe(0);
      7|  expect(Foo.num = 1).toBe(1);
      8|  expect(Foo.name).toBe("Foo");
       |                   ^
      9| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[18/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js > exec
AssertionError: expected '_Class' to be 'Foo' // Object.is equality

Expected: "Foo"
Received: "_Class"

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js:9:19
      7|  expect(Foo.num).toBe(0);
      8|  expect(Foo.num = 1).toBe(1);
      9|  expect(Foo.name).toBe("Foo");
       |                   ^
     10| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[19/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js > exec
TypeError: Cannot read properties of undefined (reading 'x')
 ❯ m fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:10:16
      8|   }
      9|   m() {
     10|    return this.x;
       |                ^
     11|   }
     12|   getSelf() {
 ❯ Foo.test fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:25:63
 ❯ fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:68:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[20/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js > exec
TypeError: Cannot read properties of undefined (reading 'x')
 ❯ m fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:10:16
      8|   }
      9|   m() {
     10|    return this.x;
       |                ^
     11|   }
     12|   getSelf() {
 ❯ Foo.test fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:25:63
 ❯ fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:68:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[21/28]⎯

 FAIL  fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js > exec
TypeError: Cannot read properties of undefined (reading 'x')
 ❯ m fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:10:16
      8|   }
      9|   m() {
     10|    return this.x;
       |                ^
     11|   }
     12|   getSelf() {
 ❯ Foo.test fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:25:63
 ❯ fixtures/babel/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:68:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[22/28]⎯

 FAIL  fixtures/babel/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js > exec
AssertionError: expected [Function] to not throw an error but 'ReferenceError: x is not defined' was thrown

- Expected: 
undefined

+ Received: 
"ReferenceError: x is not defined"

 ❯ fixtures/babel/babel-preset-env-test-fixtures-plugins-integration-issue-15170-exec.test.js:6:9
      4|  expect(() => {
      5|   x = async (x) => 0;
      6|  }).not.toThrow();
       |         ^
      7| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[23/28]⎯

 FAIL  fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js > exec
TypeError: Assignment to constant variable.
 ❯ fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6
      3| test("exec", () => {
      4|  const one = 123;
      5|  one = 432;
       |      ^
      6| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[24/28]⎯

 FAIL  fixtures/babel/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js > exec
AssertionError: expected false to be true // Object.is equality

- Expected
+ Received

- true
+ false

 ❯ fixtures/babel/babel-preset-env-test-fixtures-sanity-regex-dot-all-exec.test.js:10:37
      8|  expect(/hello.world/.test(input)).toBe(false);
      9|  expect(/hello.world/u.test(input)).toBe(false);
     10|  expect(/hello.world/s.test(input)).toBe(true);
       |                                     ^
     11|  expect(/hello.world/su.test(input)).toBe(true);
     12| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[25/28]⎯

