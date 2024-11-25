commit: 54a8389f

node: v22.11.0
⎯⎯⎯⎯⎯⎯ Failed Suites 57 ⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js [ fixtures/babel-plugin-transform-arrow-functions-test-fixtures-arrow-functions-implicit-var-arguments-exec.test.js ]
Error: 'eval' and 'arguments' cannot be used as a binding identifier in strict mode
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[1/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-before-member-call-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[2/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-noUninitializedPrivateFieldAccess-static-private-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-noUninitializedPrivateFieldAccess-static-private-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[3/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-super-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-super-exec.test.js ]
 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-delete-super-property-exec.test.js ]
 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-super-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-super-exec.test.js ]
 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-static-super-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-static-super-exec.test.js ]
Error: Invalid access to super
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[4/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-accessor-key-exec.test.js ]
Error: Unexpected token `[`. Expected * for generator, private key, identifier or async
 ❯ getRollupError ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:396:41
 ❯ convertProgram ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:1084:26
 ❯ parseAstAsync ../../node_modules/.pnpm/rollup@4.27.3/node_modules/rollup/dist/es/shared/parseAst.js:2070:106
 ❯ ssrTransformScript ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:52381:11
 ❯ loadAndTransform ../../node_modules/.pnpm/vite@5.4.11_@types+node@22.9.1/node_modules/vite/dist/node/chunks/dep-CB_7IfJ-.js:51979:72

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[5/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-access-before-declaration-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-access-before-declaration-exec.test.js ]
SyntaxError: Private field '#p' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[6/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-1-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[7/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-2-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-2-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[8/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-3-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[9/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[10/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-static-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-array-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[11/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-1-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[12/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-2-exec-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-2-exec-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[13/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-3-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[14/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[15/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-static-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-destructuring-object-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[16/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-access-before-declaration-exec.test.js ]
SyntaxError: Private field '#p' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[17/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-1-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[18/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-2-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-2-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[19/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-3-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[20/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[21/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-static-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-array-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[22/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-1-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-1-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[23/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-2-exec-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-2-exec-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[24/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-3-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-3-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[25/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[26/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-static-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-destructuring-object-pattern-static-exec.test.js ]
SyntaxError: Private field '#client' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[27/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[28/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[29/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-member-call-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[30/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[31/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[32/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-in-function-param-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[33/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[34/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-member-optional-call-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[35/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[36/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-member-call-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[37/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[38/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-exec.test.js ]
SyntaxError: Private field '#m' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[39/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-parenthesized-optional-member-call-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[40/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-computed-redeclared-exec.test.js ]
SyntaxError: Private field '#foo' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[41/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[42/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-member-call-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[43/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[44/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[45/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-in-function-param-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[46/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[47/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-member-optional-call-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[48/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[49/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-member-call-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[50/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[51/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-parenthesized-optional-member-call-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-parenthesized-optional-member-call-exec.test.js ]
SyntaxError: Private field '#m' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[52/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-parenthesized-optional-member-call-with-transform-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-parenthesized-optional-member-call-with-transform-exec.test.js ]
SyntaxError: Private field '#x' must be declared in an enclosing class
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[53/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js [ fixtures/babel-plugin-transform-class-properties-test-fixtures-regression-7371-exec.test.js ]
SyntaxError: 'super' keyword unexpected here
⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[54/98]⎯

⎯⎯⎯⎯⎯⎯ Failed Tests 41 ⎯⎯⎯⎯⎯⎯⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-cast-to-boolean-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testIf fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-cast-to-boolean-exec.test.js:7:8
      5|  class C {
      6|   static testIf(o) {
      7|    if (_assertClassBrand(C, o, _a)._.b.c.d) {
       |        ^
      8|     return true;
      9|    }
 ❯ Function.testNullish fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-cast-to-boolean-exec.test.js:89:14
 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-noDocumentAll-optional-chain-cast-to-boolean-exec.test.js:105:4

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[55/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-class-binding-exec.test.js:10:22
      8|  A = null;
      9|  expect(oldA.self).toBe(oldA);
     10|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     11| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[56/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js > exec
AssertionError: expected '_Class' to be 'Foo' // Object.is equality

Expected: "Foo"
Received: "_Class"

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-infer-name-exec.test.js:8:19
      6|  expect(Foo.num).toBe(0);
      7|  expect(Foo.num = 1).toBe(1);
      8|  expect(Foo.name).toBe("Foo");
       |                   ^
      9| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[57/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-assumption-setPublicClassFields-static-this-exec.test.js:13:22
     11|  A = null;
     12|  expect(oldA.self).toBe(oldA);
     13|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     14| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[58/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js > exec
AssertionError: expected undefined to be 'hello' // Object.is equality

- Expected: 
"hello"

+ Received: 
undefined

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-call-in-decorator-exec.test.js:21:28
     19|   }
     20|  }
     21|  expect(new Outer().hello).toBe("hello");
       |                            ^
     22| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[59/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js > exec
AssertionError: expected undefined to be 'hello' // Object.is equality

- Expected: 
"hello"

+ Received: 
undefined

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-nested-class-super-property-in-decorator-exec.test.js:22:28
     20|   }
     21|  }
     22|  expect(new Outer().hello).toBe("hello");
       |                            ^
     23| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[60/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-constructor-collision-exec.test.js > exec
AssertionError: expected undefined to be 'bar' // Object.is equality

- Expected: 
"bar"

+ Received: 
undefined

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-constructor-collision-exec.test.js:18:19
     16|  }
     17|  const f = new Foo();
     18|  expect(f.test()).toBe(foo);
       |                   ^
     19|  expect("bar" in f).toBe(false);
     20| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[61/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-constructor-collision-exec.test.js > exec
AssertionError: expected undefined to be 'bar' // Object.is equality

- Expected: 
"bar"

+ Received: 
undefined

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-constructor-collision-exec.test.js:18:19
     16|  }
     17|  const f = new Foo();
     18|  expect(f.test()).toBe(foo);
       |                   ^
     19|  expect("bar" in f).toBe(false);
     20| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[62/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-extends-computed-exec.test.js > exec
AssertionError: expected [Function] to not throw an error but 'TypeError: Private element is not pre…' was thrown

- Expected: 
undefined

+ Received: 
"TypeError: Private element is not present on this object"

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-nested-class-extends-computed-exec.test.js:30:9
     28|  expect(() => {
     29|   f.test();
     30|  }).not.toThrow();
       |         ^
     31| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[63/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:31:166

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-before-property-with-transform-exec.test.js:110:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[64/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-cast-to-boolean-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testIf fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-cast-to-boolean-exec.test.js:7:8
      5|  class C {
      6|   static testIf(o) {
      7|    if (_assertClassBrand(C, o, _a)._.b.c.d) {
       |        ^
      8|     return true;
      9|    }
 ❯ Function.testNullish fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-cast-to-boolean-exec.test.js:89:14
 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-cast-to-boolean-exec.test.js:105:4

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[65/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testNull fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-exec.test.js:56:18
     54|     return deep;
     55|    }
     56|    expect(delete _assertClassBrand(Foo, deep?.very.o?.Foo, _self)._.un…
       |                  ^
     57|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.unicorn).toBe…
     58|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.self.unicorn)…
 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-exec.test.js:92:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[66/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js > exec
AssertionError: expected function to throw an error, but it didn't
 ❯ Function.test fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:41:7

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-delete-property-with-transform-exec.test.js:158:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[67/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:34:269

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-optional-chain-optional-property-with-transform-exec.test.js:113:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[68/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-class-binding-exec.test.js:17:32
     15|  A = null;
     16|  expect(oldA.extract().self).toBe(oldA);
     17|  expect(oldA.extract().getA()).toBe(oldA);
       |                                ^
     18| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[69/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js > exec
TypeError: e.has is not a function
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:2:44
 ❯ func fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js:10:12
      8|    const func = () => {
      9|     const Test = 3;
     10|     return _assertClassBrand(Test, this, _x)._ + Test;
       |            ^
     11|    };
     12|    return func() + Test;
 ❯ Function.method fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js:12:11
 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-shadow-exec.test.js:16:14

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[70/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-loose-static-this-exec.test.js:20:32
     18|  A = null;
     19|  expect(oldA.extract().self).toBe(oldA);
     20|  expect(oldA.extract().getA()).toBe(oldA);
       |                                ^
     21| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[71/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-extends-computed-exec.test.js > exec
AssertionError: expected [Function] to not throw an error but 'TypeError: Private element is not pre…' was thrown

- Expected: 
undefined

+ Received: 
"TypeError: Private element is not present on this object"

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-nested-class-extends-computed-exec.test.js:31:9
     29|  expect(() => {
     30|   f.test();
     31|  }).not.toThrow();
       |         ^
     32| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[72/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-with-transform-exec.test.js:32:166

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-before-property-with-transform-exec.test.js:111:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[73/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-cast-to-boolean-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testIf fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-cast-to-boolean-exec.test.js:7:8
      5|  class C {
      6|   static testIf(o) {
      7|    if (_assertClassBrand(C, o, _a)._.b.c.d) {
       |        ^
      8|     return true;
      9|    }
 ❯ Function.testNullish fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-cast-to-boolean-exec.test.js:89:14
 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-cast-to-boolean-exec.test.js:105:4

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[74/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.testNull fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-exec.test.js:57:18
     55|     return deep;
     56|    }
     57|    expect(delete _assertClassBrand(Foo, deep?.very.o?.Foo, _self)._.un…
       |                  ^
     58|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.unicorn).toBe…
     59|    expect(delete _assertClassBrand(Foo, o?.Foo, _self)._.self.unicorn)…
 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-exec.test.js:93:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[75/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js > exec
AssertionError: expected function to throw an error, but it didn't
 ❯ Function.test fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:41:7

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-delete-property-with-transform-exec.test.js:158:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[76/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-with-transform-exec.test.js > exec
TypeError: Private element is not present on this object
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:3:9
 ❯ Function.test fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-with-transform-exec.test.js:35:269

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-optional-chain-optional-property-with-transform-exec.test.js:114:6

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[77/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-class-binding-exec.test.js:17:32
     15|  A = null;
     16|  expect(oldA.extract().self).toBe(oldA);
     17|  expect(oldA.extract().getA()).toBe(oldA);
       |                                ^
     18| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[78/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-self-field-exec.test.js > exec
ReferenceError: Foo is not defined
 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-self-field-exec.test.js:14:15
     12|    };
     13|   }
     14|  }, _x = { _: Foo }, _defineProperty(_Foo, "y", Foo), _Foo);
       |               ^
     15|  const { x, y } = f.extract();
     16|  expect(x).toBe(f);

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[79/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js > exec
TypeError: e.has is not a function
 ❯ _assertClassBrand ../../node_modules/.pnpm/@babel+runtime@7.26.0/node_modules/@babel/runtime/helpers/assertClassBrand.js:2:44
 ❯ func fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:10:12
      8|    const func = () => {
      9|     const Test = 3;
     10|     return _assertClassBrand(Test, this, _x)._ + Test;
       |            ^
     11|    };
     12|    return func() + Test;
 ❯ Function.method fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:12:11
 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-shadow-exec.test.js:16:14

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[80/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-static-this-exec.test.js:20:32
     18|  A = null;
     19|  expect(oldA.extract().self).toBe(oldA);
     20|  expect(oldA.extract().getA()).toBe(oldA);
       |                                ^
     21| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[81/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-private-tagged-template-exec.test.js > exec
AssertionError: expected undefined to be Foo{} // Object.is equality

- Expected: 
Foo {}

+ Received: 
undefined

 ❯ new Foo fixtures/babel-plugin-transform-class-properties-test-fixtures-private-tagged-template-exec.test.js:18:22
     16|    expect(receiver).toBe(this);
     17|    const receiver2 = _classPrivateFieldGet(_tag, this)`tagged template…
     18|    expect(receiver2).toBe(this);
       |                      ^
     19|   }
     20|  }
 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-private-tagged-template-exec.test.js:21:2

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[82/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js > exec
AssertionError: expected [Function] to throw error including '@@toPrimitive must return a primitive…' but got 'Cannot convert object to primitive va…'

Expected: "@@toPrimitive must return a primitive value."
Received: "Cannot convert object to primitive value"

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-computed-toPrimitive-exec.test.js:37:5
     35|    return 0;
     36|   }
     37|  }).toThrow("@@toPrimitive must return a primitive value.");
       |     ^
     38|  expect(() => class {
     39|   static get [arrayLike]() {

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[83/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-class-binding-exec.test.js:10:22
      8|  A = null;
      9|  expect(oldA.self).toBe(oldA);
     10|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     11| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[84/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js > exec
AssertionError: expected '_Class' to be 'Foo' // Object.is equality

Expected: "Foo"
Received: "_Class"

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-infer-name-exec.test.js:8:19
      6|  expect(Foo.num).toBe(0);
      7|  expect(Foo.num = 1).toBe(1);
      8|  expect(Foo.name).toBe("Foo");
       |                   ^
      9| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[85/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-loose-static-this-exec.test.js:13:22
     11|  A = null;
     12|  expect(oldA.self).toBe(oldA);
     13|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     14| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[86/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-static-class-binding-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-static-class-binding-exec.test.js:11:22
      9|  A = null;
     10|  expect(oldA.self).toBe(oldA);
     11|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     12| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[87/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js > exec
AssertionError: expected '_Class' to be 'Foo' // Object.is equality

Expected: "Foo"
Received: "_Class"

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-static-infer-name-exec.test.js:9:19
      7|  expect(Foo.num).toBe(0);
      8|  expect(Foo.num = 1).toBe(1);
      9|  expect(Foo.name).toBe("Foo");
       |                   ^
     10| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[88/98]⎯

 FAIL  fixtures/babel-plugin-transform-class-properties-test-fixtures-public-static-this-exec.test.js > exec
AssertionError: expected null to be [Function A] // Object.is equality

- Expected: 
[Function A]

+ Received: 
null

 ❯ fixtures/babel-plugin-transform-class-properties-test-fixtures-public-static-this-exec.test.js:14:22
     12|  A = null;
     13|  expect(oldA.self).toBe(oldA);
     14|  expect(oldA.getA()).toBe(oldA);
       |                      ^
     15| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[89/98]⎯

 FAIL  fixtures/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js > exec
TypeError: Cannot read properties of undefined (reading 'x')
 ❯ m fixtures/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:10:16
      8|   }
      9|   m() {
     10|    return this.x;
       |                ^
     11|   }
     12|   getSelf() {
 ❯ Foo.test fixtures/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:25:63
 ❯ fixtures/babel-plugin-transform-optional-chaining-test-fixtures-assumption-noDocumentAll-parenthesized-expression-member-call-exec.test.js:68:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[90/98]⎯

 FAIL  fixtures/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js > exec
TypeError: Cannot read properties of undefined (reading 'x')
 ❯ m fixtures/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:10:16
      8|   }
      9|   m() {
     10|    return this.x;
       |                ^
     11|   }
     12|   getSelf() {
 ❯ Foo.test fixtures/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:25:63
 ❯ fixtures/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-exec.test.js:68:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[91/98]⎯

 FAIL  fixtures/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js > exec
TypeError: Cannot read properties of undefined (reading 'x')
 ❯ m fixtures/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:10:16
      8|   }
      9|   m() {
     10|    return this.x;
       |                ^
     11|   }
     12|   getSelf() {
 ❯ Foo.test fixtures/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:25:63
 ❯ fixtures/babel-plugin-transform-optional-chaining-test-fixtures-general-parenthesized-expression-member-call-loose-exec.test.js:68:12

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[92/98]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[93/98]⎯

 FAIL  fixtures/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js > exec
TypeError: Assignment to constant variable.
 ❯ fixtures/babel-preset-env-test-fixtures-sanity-check-es2015-constants-exec.test.js:5:6
      3| test("exec", () => {
      4|  const one = 123;
      5|  one = 432;
       |      ^
      6| })

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[94/98]⎯

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

⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[95/98]⎯

