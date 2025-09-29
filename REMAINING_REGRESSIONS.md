# Remaining Prettier Conformance Regressions

**Updated**: 2025-09-29
**Current Status**: 21 regressions remaining (2 fixed out of 23 original)

## Summary by Priority

### 🔴 Priority 1: Near-Perfect Match (>95%)

These are closest to passing and should be easiest to fix:

1. **`js/strings/template-literals.js`** - 98.43% ⚠️ _Attempted fix didn't work_
2. **`js/require/require.js`** - 93.51%

### 🟡 Priority 2: High Match (90-95%)

1. **`typescript/arrow/16067.ts`** - 93.88%
2. **`js/decorators/member-expression.js`** - 92.42%
3. **`js/arrows/curried.js`** - 91.95% (increased from 82.70%)
4. **`js/comments-closure-typecast/comment-in-the-middle.js`** - 90.91%
5. **`js/comments/empty-statements.js`** - 90.91%
6. **`js/comments/function-declaration.js`** - 92.80%

### 🟠 Priority 3: Moderate Match (75-90%)

1. **`js/arrows/currying-4.js`** - 87.61% (decreased from 94.50%)
2. **`js/functional-composition/pipe-function-calls.js`** - 87.80%
3. **`js/new-expression/new_expression.js`** - 88.89%
4. **`typescript/decorators-ts/typeorm.ts`** - 88.37%
5. **`typescript/functional-composition/pipe-function-calls.ts`** - 82.76%
6. **`js/test-declarations/angular_async.js`** - 86.21%
7. **`js/test-declarations/angular_fakeAsync.js`** - 75.86%
8. **`js/test-declarations/angular_waitForAsync.js`** - 75.86%
9. **`js/decorators/parens.js`** - 75.00%
10. **`js/new-expression/call.js`** - 75.00%

### 🔵 Priority 4: Low Match (<75%)

These need significant work:

1. **`js/method-chain/print-width-120/constructor.js`** - 71.43%
2. **`typescript/comments/type-parameters.ts`** - 73.33%
3. **`js/test-declarations/angularjs_inject.js`** - 69.84%
4. **`js/arrows/currying-2.js`** - 59.08%
5. **`typescript/satisfies-operators/nested-await-and-satisfies.ts`** - 42.86%
6. **`typescript/as/nested-await-and-as.ts`** - 42.86%
7. **`js/arrows/chain-as-arg.js`** - 35.14%

## Breakdown by Category

### JavaScript Arrow Functions (4 remaining)

- ❌ `js/arrows/chain-as-arg.js` - 35.14%
- ❌ `js/arrows/curried.js` - 91.95%
- ❌ `js/arrows/currying-2.js` - 59.08%
- ❌ `js/arrows/currying-4.js` - 87.61%

### JavaScript Decorators (2 remaining)

- ❌ `js/decorators/member-expression.js` - 92.42%
- ❌ `js/decorators/parens.js` - 75.00%

### JavaScript New Expressions (2 remaining)

- ❌ `js/new-expression/call.js` - 75.00%
- ❌ `js/new-expression/new_expression.js` - 88.89%

### JavaScript Test Declarations (4 remaining)

- ❌ `js/test-declarations/angular_async.js` - 86.21%
- ❌ `js/test-declarations/angular_fakeAsync.js` - 75.86%
- ❌ `js/test-declarations/angular_waitForAsync.js` - 75.86%
- ❌ `js/test-declarations/angularjs_inject.js` - 69.84%

### JavaScript Comments (3 remaining)

- ❌ `js/comments/empty-statements.js` - 90.91%
- ❌ `js/comments/function-declaration.js` - 92.80%
- ❌ `js/comments-closure-typecast/comment-in-the-middle.js` - 90.91%

### JavaScript Other (4 remaining)

- ❌ `js/strings/template-literals.js` - 98.43% ⚠️
- ❌ `js/require/require.js` - 93.51%
- ❌ `js/functional-composition/pipe-function-calls.js` - 87.80%
- ❌ `js/method-chain/print-width-120/constructor.js` - 71.43%

### TypeScript (5 remaining)

- ❌ `typescript/arrow/16067.ts` - 93.88%
- ❌ `typescript/as/nested-await-and-as.ts` - 42.86%
- ❌ `typescript/comments/type-parameters.ts` - 73.33%
- ❌ `typescript/decorators-ts/typeorm.ts` - 88.37%
- ❌ `typescript/functional-composition/pipe-function-calls.ts` - 82.76%
- ❌ `typescript/satisfies-operators/nested-await-and-satisfies.ts` - 42.86%

## Pattern Groups

### Group A: Arrow Function Currying (4 tests)

All related to curried arrow function formatting. Should be fixed together.

### Group B: Decorators (3 tests)

2 JS + 1 TS decorator tests. Likely same root cause.

### Group C: Angular Test Patterns (4 tests)

All Angular-specific test declaration patterns. May share formatting logic.

### Group D: New Expression Calls (2 tests)

Both about `new` operator with function calls.

### Group E: TypeScript Operators (3 tests)

`as`, `satisfies`, and type parameter formatting.

### Group F: Comments (3 tests)

Comment positioning and preservation issues.

## Recommended Fix Order

1. **Fix template-literals properly** (98.43% - so close!)
2. **Phase 2 tests** (93%+ match rate)
3. **Comment-related tests** (90%+ match, likely related)
4. **Decorator pattern** (group fix possible)
5. **Arrow currying patterns** (complex but related)
6. **Angular patterns** (specific framework patterns)
7. **Low-match tests** (require deeper investigation)

## Notes

- Template literals fix was attempted but didn't work - needs different approach
- Arrow currying tests show varying match rates, suggesting partial fixes may have affected them
- TypeScript operator tests have very low match rates, indicating fundamental formatting differences

---

_This document tracks the 21 remaining regressions that need to be fixed to reach main branch parity._
