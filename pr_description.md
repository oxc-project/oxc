# fix(formatter): improve Prettier conformance to 92.12%

## Summary

This PR improves the oxc formatter's Prettier conformance from 83.52% to **92.12% for JavaScript** (643/698 tests passing), representing a gain of 60 tests.

## Critical Regressions Analysis

From the original 10 critical regression tests identified:

### ✅ FIXED (3 tests now passing)

1. **`js/arrows/call.js`** - ✅ Now 100% match (was 99.48%)
2. **`js/arrows/comment.js`** - ✅ Now 100% match (was 83.72%)
3. **`js/identifier/parentheses/const.js`** - ✅ Now 100% match (was 0%)

### ⚠️ PARTIALLY FIXED (1 test improved)

4. **`js/identifier/parentheses/let.js`** - Improved from 3 failing cases to just 1 edge case

### ❌ STILL FAILING (5 tests need work)

5. **`js/arrows/currying-4.js`** - Arrow function formatting issues
6. **`js/for/parentheses.js`** - For-in expression parentheses issues
7. **`js/arrows/chain-as-arg.js`** - Arrow chain indentation (35.14% match)
8. **`js/arrows/currying-2.js`** - Call argument breaking (59.08% match)
9. **`js/comments/15661.js`** - Comment positioning issues

### ❓ NOT FOUND (1 test)

10. **`js/for/for-of.js`** - This specific test file was not found in the test suite

## Changes Made

### Fixed Issues

1. **Arrow function formatting**
   - Fixed comment handling in arrow functions
   - Improved chain formatting in certain contexts
   - Better handling of arrow functions as call arguments

2. **Identifier parentheses**
   - Fixed const identifier parentheses logic
   - Improved let identifier context detection
   - Better handling in call/new expressions

3. **For-in statement parentheses**
   - Attempted fixes for right-side parentheses
   - Some improvements but not fully resolved

### Remaining Issues

#### Need Architecture Changes

- **Arrow chain formatting** (`chain-as-arg.js`, `currying-2.js`, `currying-4.js`)
  - Problem: Double indentation from both arrow chain and call argument formatters
  - Requires redesign of formatter coordination

#### Other Issues

- **Comment positioning** (`15661.js`) - Needs investigation
- **For parentheses** (`for/parentheses.js`) - Complex edge cases remain
- **Let identifier** - One edge case: `new ((let)[0] = 1)()` has extra parentheses

## Test Results

| Test File                            | Before | After      | Status            |
| ------------------------------------ | ------ | ---------- | ----------------- |
| `js/arrows/call.js`                  | 99.48% | 100%       | ✅ Fixed          |
| `js/arrows/comment.js`               | 83.72% | 100%       | ✅ Fixed          |
| `js/identifier/parentheses/const.js` | 0%     | 100%       | ✅ Fixed          |
| `js/identifier/parentheses/let.js`   | 94.55% | ~98%       | ⚠️ 1 edge case     |
| `js/arrows/chain-as-arg.js`          | 35.14% | 35.14%     | ❌ Needs refactor |
| `js/arrows/currying-2.js`            | 59.08% | 59.08%     | ❌ Needs refactor |
| **Overall JS Conformance**           | 83.52% | **92.12%** | ⬆️ **+8.60%**      |

## Implementation Notes

The arrow chain formatting issues require deeper architectural changes. The problem is that both the arrow chain formatter and call arguments formatter apply indentation, resulting in double indentation. This needs a redesign of how these formatters coordinate.

See `FORMATTER_CONFORMANCE_ANALYSIS.md` for detailed analysis and future recommendations.

## Testing

```bash
cargo run -p oxc_prettier_conformance
```

Current conformance: **92.12% JS** (643/698 tests), **68.59% TS** (393/573 tests)
