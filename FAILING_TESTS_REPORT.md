# Prettier Conformance: Specific Failing Tests Report

## 🚨 Critical Summary
**Branch**: `temp/fix-newly-failing-conformance-tests`
**Status**: **23 REGRESSIONS** compared to main branch

| Metric | Development Branch | Main Branch | Difference |
|--------|-------------------|-------------|------------|
| **JavaScript** | 647/699 (92.56%) | 663/699 (94.85%) | **-16 regressions** |
| **TypeScript** | 526/573 (91.80%) | 533/573 (93.02%) | **-7 regressions** |

## Specific Test Regressions

### JavaScript Regressions (16 tests failing in dev but passing in main)

#### Arrow Functions (5 tests) - **CRITICAL AREA**
- `js/arrows/call.js` - 99.35% match
- `js/arrows/chain-as-arg.js` - 35.14% match
- `js/arrows/curried.js` - 82.70% match
- `js/arrows/currying-2.js` - 59.08% match
- `js/arrows/currying-4.js` - 94.50% match

#### Decorators (2 tests)
- `js/decorators/member-expression.js` - 92.42% match
- `js/decorators/parens.js` - 75.00% match

#### New Expressions (2 tests)
- `js/new-expression/call.js` - 75.00% match
- `js/new-expression/new_expression.js` - 88.89% match

#### Test Declarations (3 tests)
- `js/test-declarations/angular_async.js` - 86.21% match
- `js/test-declarations/angular_fakeAsync.js` - 75.86% match
- `js/test-declarations/angular_waitForAsync.js` - 75.86% match

#### Other Regressions
- `js/functional-composition/pipe-function-calls.js` - 87.80% match
- `js/method-chain/print-width-120/constructor.js` - 71.43% match
- `js/require/require.js` - 93.51% match
- `js/strings/template-literals.js` - 98.43% match

### TypeScript Regressions (7 tests failing in dev but passing in main)

- `typescript/arrow/16067.ts` - 93.88% match
- `typescript/as/nested-await-and-as.ts` - 42.86% match
- `typescript/cast/generic-cast.ts` - 97.84% match
- `typescript/comments/type-parameters.ts` - 73.33% match
- `typescript/decorators-ts/typeorm.ts` - 88.37% match
- `typescript/functional-composition/pipe-function-calls.ts` - 82.76% match
- `typescript/satisfies-operators/nested-await-and-satisfies.ts` - 42.86% match

## Pattern Analysis

### Most Affected Areas:
1. **Arrow Functions** - 5 regressions (highest concentration)
2. **Decorators** - 2 JS + 1 TS = 3 total
3. **Test Declarations** - 3 regressions (Angular-specific)
4. **Type Operators** - as/satisfies/cast operators in TS

### High Match Ratio Regressions (>90%)
These are almost passing and should be easiest to fix:
1. `js/arrows/call.js` - 99.35%
2. `js/strings/template-literals.js` - 98.43%
3. `typescript/cast/generic-cast.ts` - 97.84%
4. `js/arrows/currying-4.js` - 94.50%
5. `typescript/arrow/16067.ts` - 93.88%
6. `js/require/require.js` - 93.51%

### Low Match Ratio Regressions (<50%)
These need significant work:
1. `js/arrows/chain-as-arg.js` - 35.14%
2. `typescript/as/nested-await-and-as.ts` - 42.86%
3. `typescript/satisfies-operators/nested-await-and-satisfies.ts` - 42.86%

## Recommended Fix Priority

### Priority 1: Quick Wins (>95% match)
Fix these first as they're almost passing:
1. `js/arrows/call.js`
2. `js/strings/template-literals.js`
3. `typescript/cast/generic-cast.ts`

### Priority 2: Arrow Function Pattern
Fix the arrow function formatting issues as a group:
1. All 5 arrow function tests
2. Related functional composition tests

### Priority 3: TypeScript Operators
Fix TypeScript-specific operator formatting:
1. as/satisfies/cast operators
2. Type parameter comments

## Root Cause Hypothesis

Based on the regression patterns:
1. **Arrow Functions**: Something changed in how arrow functions are formatted, especially with currying and chaining
2. **Parentheses**: Several tests involve parentheses handling (decorators/parens, new expressions)
3. **Type Operators**: TypeScript type assertion and satisfies operators formatting changed
4. **Angular Pattern**: Specific Angular test declaration patterns affected

## Next Steps

1. **Immediate Action**: Run specific failing tests to see actual vs expected output
   ```bash
   cargo run -p oxc_prettier_conformance -- --filter "js/arrows/call.js"
   ```

2. **Check Recent Changes**: Review commits that modified:
   - Arrow function formatting
   - Parentheses handling
   - TypeScript operator formatting

3. **Fix High-Match Tests First**: Start with tests >95% matching as they'll be quickest to resolve

## Test Commands

To test specific regressions:
```bash
# Test a specific file
cargo run -p oxc_prettier_conformance -- --filter "js/arrows/call.js"

# Test all arrow functions
cargo run -p oxc_prettier_conformance -- --filter "js/arrows"

# Test all TypeScript regressions
cargo run -p oxc_prettier_conformance -- --filter "typescript/arrow/16067.ts"
cargo run -p oxc_prettier_conformance -- --filter "typescript/as/nested-await-and-as.ts"
# ... etc
```