# Prettier Conformance Fix Implementation Plan

## 🚨 ZERO-REGRESSION POLICY
**Every change MUST maintain or improve test pass rates. NO exceptions.**

## Phase 0: Investigation & Setup (Read-Only)
1. **Examine actual vs expected output** for each failing test
   - Run with --filter to see specific differences
   - Document the formatting discrepancies
2. **Identify common patterns** across failures
3. **Check recent commits** that might have caused regressions
4. **Set up verification scripts** for continuous testing

## Phase 1: Ultra-High Match Fixes (>97% match)
**Goal**: Fix tests that are 1-2 characters away from passing

### ✅ Step 1.1: Template Literals (98.43% match) - ATTEMPTED
- Test: `js/strings/template-literals.js`
- **Status**: ❌ Fix attempted but didn't work (still at 98.43%)
- **Attempted**: Added parentheses logic for object literals in templates
- **Result**: No regression but issue not resolved
- **TODO**: Needs different approach

### ✅ Step 1.2: TypeScript Cast (97.84% match) - COMPLETED
- Test: `typescript/cast/generic-cast.ts`
- **Status**: ✅ FIXED (now 100% match)
- **Solution**: Added length-based logic for line breaking in TSTypeAssertion
- **Commit**: `9e7d8ea55`
- **Success**: 527/573 TS tests passing (up from 526)

### ✅ Step 1.3: Arrow Call (99.35% match) - COMPLETED
- Test: `js/arrows/call.js`
- **Status**: ✅ FIXED (now 100% match)
- **Solution**: Detect long curried arrow arguments for proper expansion
- **Commit**: `9e7d8ea55`
- **Success**: 648/699 JS tests passing (up from 647)

## Phase 2: High Match Individual Fixes (93-95%)
**Goal**: Fix nearly-passing tests one at a time

### Step 2.1: Simple Arrow Function
- Test: `js/arrows/currying-4.js` (94.50%)
- Fix currying parentheses/spacing
- **Verify**: All arrow tests still pass/fail as before
- **Success Criteria**: 650/699 JS tests passing

### Step 2.2: TypeScript Arrow
- Test: `typescript/arrow/16067.ts` (93.88%)
- Fix TS-specific arrow formatting
- **Verify**: No other TS tests regressed
- **Success Criteria**: 528/573 TS tests passing

### Step 2.3: Require Statement
- Test: `js/require/require.js` (93.51%)
- Fix require() call formatting
- **Verify**: Full JS suite
- **Success Criteria**: 651/699 JS tests passing

## Phase 3: Pattern-Based Fixes (Related Tests)
**Goal**: Fix groups of related issues together

### Step 3.1: Decorator Pattern Fix
- Tests: `js/decorators/member-expression.js` (92.42%), `js/decorators/parens.js` (75.00%)
- Identify common decorator formatting issue
- Apply fix to decorator formatting logic
- **Verify**: Both tests + full suite
- **Success Criteria**: 653/699 JS tests passing

### Step 3.2: New Expression Pattern
- Tests: `js/new-expression/call.js` (75%), `js/new-expression/new_expression.js` (88.89%)
- Fix `new` operator parentheses handling
- **Verify**: Both tests + full suite
- **Success Criteria**: 655/699 JS tests passing

### Step 3.3: Angular Test Declarations
- Tests: All 3 angular test declarations (75-86% match)
- Identify Angular-specific pattern issue
- Apply targeted fix
- **Verify**: All 3 tests + full suite
- **Success Criteria**: 658/699 JS tests passing

## Phase 4: Complex Arrow Functions
**Goal**: Fix remaining arrow function issues

### Step 4.1: Medium Complexity Arrows
- Test: `js/arrows/curried.js` (82.70%)
- Fix currying formatting
- **Verify**: All arrow tests
- **Success Criteria**: 659/699 JS tests passing

### Step 4.2: Complex Arrows
- Tests: `js/arrows/currying-2.js` (59%), `js/arrows/chain-as-arg.js` (35%)
- These require deeper formatting changes
- Fix chaining and complex currying
- **Verify**: Full arrow test suite
- **Success Criteria**: 661/699 JS tests passing

## Phase 5: TypeScript Operators
**Goal**: Fix TS-specific operator formatting

### Step 5.1: High-Match TS Fixes
- Tests: `typescript/decorators-ts/typeorm.ts` (88%), `typescript/functional-composition/pipe-function-calls.ts` (82%)
- Fix decorator and pipe operator formatting
- **Verify**: Full TS suite
- **Success Criteria**: 530/573 TS tests passing

### Step 5.2: Type Operators
- Tests: `typescript/as/nested-await-and-as.ts` (42%), `typescript/satisfies-operators/nested-await-and-satisfies.ts` (42%)
- Fix as/satisfies operator formatting
- **Verify**: All TS operator tests
- **Success Criteria**: 532/573 TS tests passing

### Step 5.3: Type Parameters
- Test: `typescript/comments/type-parameters.ts` (73%)
- Fix type parameter comment positioning
- **Verify**: Full TS suite
- **Success Criteria**: 533/573 TS tests passing

## Phase 6: Remaining Fixes
**Goal**: Fix any remaining regressions

### Step 6.1: Functional Composition
- Test: `js/functional-composition/pipe-function-calls.js` (87%)
- Fix pipe operator formatting
- **Success Criteria**: 662/699 JS tests passing

### Step 6.2: Method Chaining
- Test: `js/method-chain/print-width-120/constructor.js` (71%)
- Fix print width handling in method chains
- **Success Criteria**: 663/699 JS tests passing (BACK TO MAIN LEVEL)

## Verification Protocol

### After EVERY change:
1. Run specific test: `cargo run -p oxc_prettier_conformance -- --filter "test-name"`
2. Run full suite: `cargo run -p oxc_prettier_conformance`
3. Compare numbers to previous run
4. If ANY regression: `git stash` immediately

### Before EVERY commit:
1. Full conformance test must pass at same or better level
2. Run `just ready` for all checks
3. Document exact test count improvements

## Rollback Procedures
- Keep git stash of each successful step
- If regression detected: immediate `git stash` or `git reset --hard HEAD`
- Never commit until verification complete
- Tag each successful phase: `git tag phase-X-complete`

## Success Metrics

### Current Progress
- **Completed**: 2/23 tests fixed (8.7%)
- **Current State**: 648/699 JS, 527/573 TS
- **Remaining**: 21 tests to fix

### Original Targets
- **Phase 1**: ~~3~~ 2 tests fixed, ~~650~~ 648/699 JS, 527/573 TS ✅ (Partial)
- **Phase 2**: 3 tests to fix, target 651/699 JS, 528/573 TS
- **Phase 3**: 7 tests to fix, target 658/699 JS, 528/573 TS
- **Phase 4**: 2 tests to fix, target 660/699 JS, 528/573 TS
- **Phase 5**: 5 tests to fix, target 660/699 JS, 533/573 TS
- **Phase 6**: 2 tests to fix, target 662/699 JS, 533/573 TS
- **FINAL**: Back to main branch performance levels (663/699 JS, 533/573 TS)

## Total: 21 regressions remaining (2 completed, 1 attempted but failed)

## Command Reference

### Testing Commands
```bash
# Test specific file
cargo run -p oxc_prettier_conformance -- --filter "js/arrows/call.js"

# Test category
cargo run -p oxc_prettier_conformance -- --filter "js/arrows"

# Full conformance test
cargo run -p oxc_prettier_conformance

# Full validation before commit
just ready
```

### Git Safety Commands
```bash
# Save work before risky change
git stash push -m "Before fixing [test-name]"

# Tag successful phase
git tag phase-1-complete -m "Fixed 3 ultra-high match tests"

# Emergency rollback
git reset --hard HEAD
git stash pop  # if you want to recover stashed work
```

## Risk Mitigation

1. **Never skip verification** - Every change must be tested
2. **Work on one test at a time** in early phases
3. **Group fixes only when** patterns are confirmed identical
4. **Document every change** with test results
5. **Keep detailed notes** of what fixed each test
6. **If stuck on a test** for >30 mins, skip and return later
7. **Priority is zero regression**, not speed

## Expected Timeline

- Phase 0: 30-60 minutes (investigation)
- Phase 1: 30 minutes (3 simple fixes)
- Phase 2: 45 minutes (3 individual fixes)
- Phase 3: 90 minutes (pattern-based fixes)
- Phase 4: 60 minutes (complex arrows)
- Phase 5: 90 minutes (TypeScript operators)
- Phase 6: 30 minutes (final fixes)

**Total estimated time**: 6-7 hours of focused work

## Notes

- The branch name "fix-newly-failing-conformance-tests" suggests these tests were recently broken
- Check git history between main and this branch for potential causes
- Some fixes might resolve multiple tests if they share root causes
- Low match ratios (<50%) indicate fundamental formatting differences that need careful analysis