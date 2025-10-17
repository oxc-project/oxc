# Bisect Analysis for Issue #14732

## Problem with Automated Bisect

The automated bisect workflow (`.github/workflows/bisect-auto-14732.yml`) completed but identified an **incorrect commit**:

- **Identified:** `28cfae0ae` - "refactor(oxlint): use vitest's built in file snapshot comparison"
- **Why it's wrong:** Only touches `apps/oxlint/test/utils.ts` (test utilities), not NAPI parser

### Root Cause of False Positive

The bisect script marks commits as "bad" when:
1. Build fails
2. Test times out (60 seconds)
3. Workers don't all complete

**Issue:** The script doesn't distinguish between:
- Actual heap corruption crashes (the real bug)
- Build failures (many commits had cargo feature changes)
- Slow builds + 16 workers exceeding timeout
- Worker hangs (not crashes)

## Key Observations

### Suspect Commits

Based on commit analysis, the most likely culprits are:

1. **`7e4d04fad` - "feat(napi/parser): add option to add `parent` prop to AST nodes"**
   - Date: Sun Oct 5 2025
   - Added massive generated deserializer files (6000-7000 lines each)
   - Added 4 new deserializer variants: `js_parent.js`, `js_range_parent.js`, `ts_parent.js`, `ts_range_parent.js`
   - These files create circular references (parent pointers)
   - **High probability:** Circular references + concurrent worker threads = heap corruption on Windows

2. **`e75d42d50` - "perf(napi/parser, linter/plugins): remove runtime preserveParens option"**
   - Date: Sat Oct 4 2025
   - Added new `ts_range_no_parens.js` deserializer (5856 lines)
   - Changed serialization logic

3. **Other raw transfer commits:**
   - `68c0252ff` - Refactored deserializer code generation
   - `4f301de9f` - Improved formatting of generated code
   - `f6d890af1` - Re-ran ast_tools codegen

### Commits Between Good and Bad

- Good: `1b3f43746891a3fabdd0d2528595a1bdb4c0f26f` (v0.92.0, Sep 18 2025)
- Bad: `454ee94ff30d8423520bba9488bed0e3f8d1c77b` (v0.95.0, Oct 17 2025)
- Window: ~30 days, many commits

### Related Issues

- Commit `674510d13` fixed thread stack overflow in `tasks/coverage` (not NAPI)
- Multiple test262/babel conformance changes
- Windows CI for NAPI tests added in `0ec084708`

## Recommended Next Steps

### 1. Manual Testing of Key Commits

Test these specific commits manually on Windows (or using the workflow):

```bash
# Known good baseline
./test-commit.sh 1b3f43746891a3fabdd0d2528595a1bdb4c0f26f  # v0.92.0

# Before parent prop addition
./test-commit.sh e75d42d50  # Oct 4

# Parent prop addition (PRIME SUSPECT)
./test-commit.sh 7e4d04fad  # Oct 5

# After parent prop
./test-commit.sh c8de6fe9c  # Oct 5

# Known bad
./test-commit.sh 454ee94ff30d8423520bba9488bed0e3f8d1c77b  # v0.95.0
```

### 2. Improved Bisect Script

If manual testing confirms `7e4d04fad` is the culprit, we could:
- Create a more reliable bisect script that:
  - Skips commits with build failures (exit 125)
  - Increases timeout or reduces worker count
  - Checks for actual crash error messages
  - Tests multiple times to handle intermittent issues

### 3. Investigation Focus

If `7e4d04fad` is confirmed:
- Review how `parent` circular references are handled in deserializers
- Check if Windows has different behavior with circular object graphs
- Review memory allocation patterns in generated code
- Check if V8 heap size needs adjustment for Windows
- Consider if Node.js worker thread limitations on Windows are hit

### 4. Possible Workarounds

If the parent prop feature causes issues:
- Make it opt-in with a flag
- Use WeakRef for parent pointers to avoid GC issues
- Lazy initialization of parent references
- Platform-specific code paths for Windows

## Conclusion

The automated bisect was **unreliable** due to:
- Test criteria conflating build failures with crashes
- Timeout issues with concurrent builds
- No verification of actual heap corruption

**Most likely culprit:** Commit `7e4d04fad` which added parent prop feature with massive generated deserializers and circular references.

**Action:** Need Windows machine or CI to manually test the suspect commits.
