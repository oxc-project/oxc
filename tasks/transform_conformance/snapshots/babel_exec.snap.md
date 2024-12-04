commit: 54a8389f

node: v22.11.0
⎯⎯⎯⎯⎯⎯ Failed Suites 32 ⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js [ fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js ]
Error: 'eval' and 'arguments' cannot be used as a binding identifier in strict mode
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[1/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noUninitializedPrivateFieldAccess-static-private-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noUninitializedPrivateFieldAccess-static-private-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[2/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-super-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-super-exec.test.js ]
 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js ]
 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-super-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-super-exec.test.js ]
 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-super-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-super-exec.test.js ]
Error: Invalid access to super
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[3/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js ]
Error: Unexpected token `[`. Expected * for generator, private key, identifier or async
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[4/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-access-before-declaration-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-access-before-declaration-exec.test.js ]
SyntaxError: Private field '#p' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[5/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[6/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-2-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-2-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[7/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[8/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[9/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[10/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[11/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-2-exec-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-2-exec-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[12/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[13/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[14/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[15/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js ]
SyntaxError: Private field '#p' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[16/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[17/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-2-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-2-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[18/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[19/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[20/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[21/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[22/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-2-exec-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-2-exec-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[23/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[24/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[25/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[26/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[27/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[28/50]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js ]
SyntaxError: 'super' keyword unexpected here
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[29/50]⎯

⎯⎯⎯⎯⎯⎯ Failed Tests 18 ⎯⎯⎯⎯⎯⎯⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[30/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[31/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[32/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[33/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[34/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[35/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[36/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[37/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[38/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[39/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[40/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[41/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[42/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[43/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[44/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[45/50]⎯

 FAIL  fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js > exec
TypeError: Assignment to constant variable.
 ❯ fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6
      3| test("exec", () => {
      4|  const one = 123;
      5|  one = 432;
       |      ^
      6| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[46/50]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[47/50]⎯

