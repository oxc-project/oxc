commit: 54a8389f

node: v22.11.0
⎯⎯⎯⎯⎯⎯ Failed Suites 38 ⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js [ fixtures/babel/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js ]
Error: 'eval' and 'arguments' cannot be used as a binding identifier in strict mode
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[1/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noUninitializedPrivateFieldAccess-static-private-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noUninitializedPrivateFieldAccess-static-private-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[2/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[3/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js ]
Error: Unexpected token `[`. Expected * for generator, private key, identifier or async
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[4/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-access-before-declaration-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-access-before-declaration-exec.test.js ]
SyntaxError: Private field '#p' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[5/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[6/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-2-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-2-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[7/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[8/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[9/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[10/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[11/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-2-exec-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-2-exec-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[12/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[13/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[14/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[15/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js ]
SyntaxError: Private field '#p' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[16/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[17/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-2-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-2-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[18/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[19/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[20/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[21/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[22/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-2-exec-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-2-exec-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[23/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[24/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[25/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[26/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[27/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[28/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[29/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js ]
SyntaxError: Private field '#m' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[30/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[31/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[32/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[33/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-parenthesized-optional-member-call-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-parenthesized-optional-member-call-exec.test.js ]
SyntaxError: Private field '#m' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[34/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js ]
SyntaxError: 'super' keyword unexpected here
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[35/84]⎯

⎯⎯⎯⎯⎯⎯ Failed Tests 46 ⎯⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js:112:10
    110|  var _x = { _: 1 };
    111|  var _m = { _: function() {
    112|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
    113|  } };
    114|  var _self = { _: _Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js:21:46
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js:116:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[36/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-cast-to-boolean-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testIf fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-cast-to-boolean-exec.test.js:7:8
      5|  class C {
      6|   static testIf(o) {
      7|    if (_assertClassBrand(C, o, _a)._.b.c.d) {
       |        ^
      8|     return true;
      9|    }
 ❯ Function.testNullish fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-cast-to-boolean-exec.test.js:89:14
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-cast-to-boolean-exec.test.js:105:4

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[37/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[38/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[39/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[40/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[41/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[42/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[43/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js:111:10
    109|  var _x = { _: 1 };
    110|  var _m = { _: function() {
    111|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
    112|  } };
    113|  var _self = { _: _Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js:20:46
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js:115:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[44/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:32:168

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:115:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[45/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:32:166

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:112:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[46/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-cast-to-boolean-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testIf fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-cast-to-boolean-exec.test.js:7:8
      5|  class C {
      6|   static testIf(o) {
      7|    if (_assertClassBrand(C, o, _a)._.b.c.d) {
       |        ^
      8|     return true;
      9|    }
 ❯ Function.testNullish fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-cast-to-boolean-exec.test.js:89:14
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-cast-to-boolean-exec.test.js:105:4

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[47/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testNull fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-exec.test.js:57:18
     55|     return deep;
     56|    }
     57|    expect(delete _assertClassBrand(Foo, deep?.very.o?.Foo, _self)._.un…
       |                  ^
     58|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.unicorn).toBe…
     59|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.self.unicorn)…
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-exec.test.js:94:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[48/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js > exec
AssertionError: expected function to throw an error, but it didn't
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:42:7

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:160:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[49/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js:44:10
     42|  var _x = { _: 1 };
     43|  var _m = { _: function() {
     44|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
     45|  } };
     46|  var _self = { _: _Foo };
 ❯ f fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js:19:57
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js:34:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js:48:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[50/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ _ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:59:10
     57|  var _x = { _: 1 };
     58|  var _m = { _: function() {
     59|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
     60|  } };
     61|  var _self = { _: _Foo };
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:45:181
 ❯ j fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:46:6
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:53:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:63:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[51/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js:114:10
    112|  var _x = { _: 1 };
    113|  var _m = { _: function() {
    114|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
    115|  } };
    116|  var _self = { _: _Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js:20:17
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js:118:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[52/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:114:10
    112|  var _x = { _: 1 };
    113|  var _m = { _: function() {
    114|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
    115|  } };
    116|  var _self = { _: _Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:23:142
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:118:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[53/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js:114:10
    112|  var _x = { _: 1 };
    113|  var _m = { _: function() {
    114|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
    115|  } };
    116|  var _self = { _: _Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js:20:14
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js:118:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[54/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:35:269

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:118:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[55/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:35:269

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:115:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[56/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[57/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[58/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js:112:10
    110|  var _x = { _: 1 };
    111|  var _m = { _: function() {
    112|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
    113|  } };
    114|  var _self = { _: _Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js:21:46
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js:116:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[59/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-with-transform-exec.test.js:33:168

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-with-transform-exec.test.js:116:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[60/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-with-transform-exec.test.js:33:166

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-with-transform-exec.test.js:113:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[61/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-cast-to-boolean-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testIf fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-cast-to-boolean-exec.test.js:7:8
      5|  class C {
      6|   static testIf(o) {
      7|    if (_assertClassBrand(C, o, _a)._.b.c.d) {
       |        ^
      8|     return true;
      9|    }
 ❯ Function.testNullish fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-cast-to-boolean-exec.test.js:89:14
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-cast-to-boolean-exec.test.js:105:4

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[62/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testNull fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-exec.test.js:58:18
     56|     return deep;
     57|    }
     58|    expect(delete _assertClassBrand(Foo, deep?.very.o?.Foo, _self)._.un…
       |                  ^
     59|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.unicorn).toBe…
     60|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.self.unicorn)…
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-exec.test.js:95:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[63/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js > exec
AssertionError: expected function to throw an error, but it didn't
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:42:7

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:160:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[64/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js:45:10
     43|  var _x = { _: 1 };
     44|  var _m = { _: function() {
     45|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
     46|  } };
     47|  var _self = { _: _Foo };
 ❯ f fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js:20:57
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js:35:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js:49:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[65/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ _ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:60:10
     58|  var _x = { _: 1 };
     59|  var _m = { _: function() {
     60|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
     61|  } };
     62|  var _self = { _: _Foo };
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:46:181
 ❯ j fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:47:6
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:54:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:64:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[66/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js:115:10
    113|  var _x = { _: 1 };
    114|  var _m = { _: function() {
    115|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
    116|  } };
    117|  var _self = { _: _Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js:21:17
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js:119:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[67/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js:115:10
    113|  var _x = { _: 1 };
    114|  var _m = { _: function() {
    115|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
    116|  } };
    117|  var _self = { _: _Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js:24:142
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js:119:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[68/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js:115:10
    113|  var _x = { _: 1 };
    114|  var _m = { _: function() {
    115|   return _assertClassBrand(_Foo, this, _x)._;
       |          ^
    116|  } };
    117|  var _self = { _: _Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js:21:14
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js:119:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[69/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-with-transform-exec.test.js:36:269

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-with-transform-exec.test.js:119:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[70/84]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-with-transform-exec.test.js:36:269

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-with-transform-exec.test.js:116:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[71/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[72/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[73/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[74/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[75/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[76/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[77/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[78/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[79/84]⎯

 FAIL  fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js > exec
TypeError: Assignment to constant variable.
 ❯ fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6
      3| test("exec", () => {
      4|  const one = 123;
      5|  one = 432;
       |      ^
      6| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[80/84]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[81/84]⎯

