# Prettier Conformance Test Regression Analysis

## Summary
- JavaScript: 52 failing (dev) vs 36 failing (main)
- TypeScript: 47 failing (dev) vs 40 failing (main)
- **Total Regressions: 23**

## JavaScript Regressions (16 tests)
Tests that are failing in dev branch but passing in main:

- `js/arrows/call.js`
- `js/arrows/chain-as-arg.js`
- `js/arrows/curried.js`
- `js/arrows/currying-2.js`
- `js/arrows/currying-4.js`
- `js/decorators/member-expression.js`
- `js/decorators/parens.js`
- `js/functional-composition/pipe-function-calls.js`
- `js/method-chain/print-width-120/constructor.js`
- `js/new-expression/call.js`
- `js/new-expression/new_expression.js`
- `js/require/require.js`
- `js/strings/template-literals.js`
- `js/test-declarations/angular_async.js`
- `js/test-declarations/angular_fakeAsync.js`
- `js/test-declarations/angular_waitForAsync.js`

## TypeScript Regressions (7 tests)
Tests that are failing in dev branch but passing in main:

- `typescript/arrow/16067.ts`
- `typescript/as/nested-await-and-as.ts`
- `typescript/cast/generic-cast.ts`
- `typescript/comments/type-parameters.ts`
- `typescript/decorators-ts/typeorm.ts`
- `typescript/functional-composition/pipe-function-calls.ts`
- `typescript/satisfies-operators/nested-await-and-satisfies.ts`

## Analysis by Category

### Regressions by Test Category:

**arrows** (5 tests):
  - `js/arrows/call.js`
  - `js/arrows/chain-as-arg.js`
  - `js/arrows/curried.js`
  - `js/arrows/currying-2.js`
  - `js/arrows/currying-4.js`

**decorators** (2 tests):
  - `js/decorators/member-expression.js`
  - `js/decorators/parens.js`

**functional-composition** (1 tests):
  - `js/functional-composition/pipe-function-calls.js`

**method-chain** (1 tests):
  - `js/method-chain/print-width-120/constructor.js`

**new-expression** (2 tests):
  - `js/new-expression/call.js`
  - `js/new-expression/new_expression.js`

**require** (1 tests):
  - `js/require/require.js`

**strings** (1 tests):
  - `js/strings/template-literals.js`

**test-declarations** (3 tests):
  - `js/test-declarations/angular_async.js`
  - `js/test-declarations/angular_fakeAsync.js`
  - `js/test-declarations/angular_waitForAsync.js`

**typescript** (7 tests):
  - `typescript/arrow/16067.ts`
  - `typescript/as/nested-await-and-as.ts`
  - `typescript/cast/generic-cast.ts`
  - `typescript/comments/type-parameters.ts`
  - `typescript/decorators-ts/typeorm.ts`
  - `typescript/functional-composition/pipe-function-calls.ts`
  - `typescript/satisfies-operators/nested-await-and-satisfies.ts`
