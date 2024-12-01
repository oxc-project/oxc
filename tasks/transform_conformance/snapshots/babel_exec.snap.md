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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[1/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noUninitializedPrivateFieldAccess-static-private-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noUninitializedPrivateFieldAccess-static-private-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[2/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[3/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js ]
Error: Unexpected token `[`. Expected * for generator, private key, identifier or async
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[4/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-access-before-declaration-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-access-before-declaration-exec.test.js ]
SyntaxError: Private field '#p' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[5/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[6/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-2-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-2-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[7/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[8/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[9/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[10/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[11/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-2-exec-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-2-exec-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[12/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[13/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[14/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[15/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js ]
SyntaxError: Private field '#p' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[16/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[17/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-2-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-2-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[18/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[19/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[20/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[21/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-1-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[22/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-2-exec-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-2-exec-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[23/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-3-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[24/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[25/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-static-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[26/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[27/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[28/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[29/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js ]
SyntaxError: Private field '#m' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[30/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[31/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[32/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[33/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-parenthesized-optional-member-call-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-parenthesized-optional-member-call-exec.test.js ]
SyntaxError: Private field '#m' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[34/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js [ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js ]
SyntaxError: 'super' keyword unexpected here
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[35/95]⎯

⎯⎯⎯⎯⎯⎯ Failed Tests 57 ⎯⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js:110:10
    108|  var _x = { _: 1 };
    109|  var _m = { _: function() {
    110|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
    111|  } };
    112|  var _self = { _: Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js:20:46
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js:114:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[36/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[37/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-class-binding-exec.test.js:10:22
      8|  A = null;
      9|  expect(oldA.self).toBe(oldA);
     10|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     11| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[38/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[39/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-this-exec.test.js:13:22
     11|  A = null;
     12|  expect(oldA.self).toBe(oldA);
     13|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     14| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[40/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[41/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[42/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[43/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[44/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[45/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js:109:10
    107|  var _x = { _: 1 };
    108|  var _m = { _: function() {
    109|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
    110|  } };
    111|  var _self = { _: Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js:19:46
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js:113:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[46/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:31:168

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js:113:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[47/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:31:166

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:110:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[48/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[49/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testNull fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-exec.test.js:56:18
     54|     return deep;
     55|    }
     56|    expect(delete _assertClassBrand(Foo, deep?.very.o?.Foo, _self)._.un…
       |                  ^
     57|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.unicorn).toBe…
     58|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.self.unicorn)…
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-exec.test.js:92:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[50/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js > exec
AssertionError: expected function to throw an error, but it didn't
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:41:7

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:158:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[51/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js:42:10
     40|  var _x = { _: 1 };
     41|  var _m = { _: function() {
     42|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
     43|  } };
     44|  var _self = { _: Foo };
 ❯ f fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js:18:57
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js:33:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js:46:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[52/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ _ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:57:10
     55|  var _x = { _: 1 };
     56|  var _m = { _: function() {
     57|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
     58|  } };
     59|  var _self = { _: Foo };
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:44:181
 ❯ j fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:45:6
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:52:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js:61:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[53/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js:112:10
    110|  var _x = { _: 1 };
    111|  var _m = { _: function() {
    112|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
    113|  } };
    114|  var _self = { _: Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js:19:17
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js:116:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[54/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:112:10
    110|  var _x = { _: 1 };
    111|  var _m = { _: function() {
    112|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
    113|  } };
    114|  var _self = { _: Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:22:142
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js:116:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[55/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js:112:10
    110|  var _x = { _: 1 };
    111|  var _m = { _: function() {
    112|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
    113|  } };
    114|  var _self = { _: Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js:19:14
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js:116:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[56/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:34:269

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js:116:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[57/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:34:269

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:113:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[58/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-class-binding-exec.test.js:17:32
     15|  A = null;
     16|  expect(oldA.extract().self).toBe(oldA);
     17|  expect(oldA.extract().getA()).toBe(oldA);
       |                                ^
     18| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[59/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[60/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-this-exec.test.js:20:32
     18|  A = null;
     19|  expect(oldA.extract().self).toBe(oldA);
     20|  expect(oldA.extract().getA()).toBe(oldA);
       |                                ^
     21| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[61/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[62/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js:110:10
    108|  var _x = { _: 1 };
    109|  var _m = { _: function() {
    110|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
    111|  } };
    112|  var _self = { _: Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js:20:46
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js:114:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[63/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-with-transform-exec.test.js:32:168

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-with-transform-exec.test.js:114:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[64/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-with-transform-exec.test.js:32:166

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-with-transform-exec.test.js:111:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[65/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[66/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testNull fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-exec.test.js:57:18
     55|     return deep;
     56|    }
     57|    expect(delete _assertClassBrand(Foo, deep?.very.o?.Foo, _self)._.un…
       |                  ^
     58|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.unicorn).toBe…
     59|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.self.unicorn)…
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-exec.test.js:93:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[67/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js > exec
AssertionError: expected function to throw an error, but it didn't
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:41:7

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:158:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[68/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js:43:10
     41|  var _x = { _: 1 };
     42|  var _m = { _: function() {
     43|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
     44|  } };
     45|  var _self = { _: Foo };
 ❯ f fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js:19:57
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js:34:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js:47:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[69/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ _ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:58:10
     56|  var _x = { _: 1 };
     57|  var _m = { _: function() {
     58|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
     59|  } };
     60|  var _self = { _: Foo };
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:45:181
 ❯ j fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:46:6
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:53:11
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js:62:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[70/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js:113:10
    111|  var _x = { _: 1 };
    112|  var _m = { _: function() {
    113|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
    114|  } };
    115|  var _self = { _: Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js:20:17
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js:117:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[71/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js:113:10
    111|  var _x = { _: 1 };
    112|  var _m = { _: function() {
    113|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
    114|  } };
    115|  var _self = { _: Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js:23:142
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js:117:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[72/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Object._ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js:113:10
    111|  var _x = { _: 1 };
    112|  var _m = { _: function() {
    113|   return _assertClassBrand(Foo, this, _x)._;
       |          ^
    114|  } };
    115|  var _self = { _: Foo };
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js:20:14
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js:117:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[73/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-with-transform-exec.test.js:35:269

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-with-transform-exec.test.js:117:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[74/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-with-transform-exec.test.js:35:269

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-with-transform-exec.test.js:114:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[75/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-class-binding-exec.test.js:17:32
     15|  A = null;
     16|  expect(oldA.extract().self).toBe(oldA);
     17|  expect(oldA.extract().getA()).toBe(oldA);
       |                                ^
     18| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[76/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-self-field-exec.test.js > exec
ReferenceError: Foo is not defined
 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-self-field-exec.test.js:14:15
     12|    };
     13|   }
     14|  }, _x = { _: Foo }, _defineProperty(_Foo, "y", Foo), _Foo);
       |               ^
     15|  const { x, y } = f.extract();
     16|  expect(x).toBe(f);

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[77/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[78/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-private-static-this-exec.test.js:20:32
     18|  A = null;
     19|  expect(oldA.extract().self).toBe(oldA);
     20|  expect(oldA.extract().getA()).toBe(oldA);
       |                                ^
     21| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[79/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[80/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-class-binding-exec.test.js:10:22
      8|  A = null;
      9|  expect(oldA.self).toBe(oldA);
     10|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     11| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[81/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[82/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-this-exec.test.js:13:22
     11|  A = null;
     12|  expect(oldA.self).toBe(oldA);
     13|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     14| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[83/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-class-binding-exec.test.js:11:22
      9|  A = null;
     10|  expect(oldA.self).toBe(oldA);
     11|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     12| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[84/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[85/95]⎯

 FAIL  fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel/babel-plugin-transform-class-properties-test-fixtures-public-static-this-exec.test.js:14:22
     12|  A = null;
     13|  expect(oldA.self).toBe(oldA);
     14|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     15| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[86/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[87/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[88/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[89/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[90/95]⎯

 FAIL  fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js > exec
TypeError: Assignment to constant variable.
 ❯ fixtures/babel/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6
      3| test("exec", () => {
      4|  const one = 123;
      5|  one = 432;
       |      ^
      6| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[91/95]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[92/95]⎯

