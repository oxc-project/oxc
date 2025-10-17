# Next Steps for Issue #14732 Investigation

## Current Status

✅ **Completed:**
- Created worker thread reproduction test
- Built automated bisect workflow for GitHub Actions
- Identified bisect workflow limitations
- Analyzed commits and found prime suspect

❌ **Automated bisect unreliable:** Identified wrong commit due to build failures being marked as "bad"

## Prime Suspect

**Commit `7e4d04fad` - "feat(napi/parser): add option to add `parent` prop to AST nodes"**
- Adds massive generated deserializers with circular references
- Date: Oct 5, 2025 (between v0.92.0 and v0.95.0)
- Changes: 4 new ~6000-7000 line deserializer files

## Immediate Actions Required

### Option 1: Manual Testing on Windows (RECOMMENDED)

Need access to Windows machine to test:

```bash
# Clone and setup
git clone https://github.com/Boshen/oxc.git
cd oxc
git checkout 7e4d04fad

# Build and test
cd napi/parser
pnpm install --ignore-scripts
pnpm build --features allocator --release

# Copy test files from cam-debug-14732 branch
# Then run: node test-worker-main.mjs
```

If it crashes with heap corruption → We found it!
If it works → Test next suspect commit

### Option 2: Trigger Manual Workflow Tests

Create a new workflow that tests specific commits:

```bash
# Modify bisect workflow to test single commits
# Trigger with specific commit SHAs:
# - 1b3f43746891a3fabdd0d2528595a1bdb4c0f26f (v0.92.0 - should pass)
# - 7e4d04fad (suspect - might fail)
# - 454ee94ff30d8423520bba9488bed0e3f8d1c77b (v0.95.0 - should fail)
```

### Option 3: Binary Search with Fewer Commits

Test commits in this order:
1. `e75d42d50` (Oct 4, before parent prop)
2. `7e4d04fad` (Oct 5, parent prop added) ← **TEST THIS FIRST**
3. `c8de6fe9c` (Oct 5, after parent prop)

## If Commit 7e4d04fad is Confirmed

### Investigation Steps

1. **Review generated code:**
   - Check `napi/parser/generated/deserialize/*_parent.js`
   - Look for memory allocation patterns
   - Check circular reference handling

2. **Test variations:**
   - Does it crash without `parent` option?
   - Does it crash with fewer workers (8? 4?)?
   - Is it deterministic or race condition?

3. **Compare with working version:**
   - `git diff e75d42d50..7e4d04fad -- crates/oxc_ast/src/serialize/`
   - Review serialization logic changes

### Potential Fixes

1. **Use WeakRef for parent pointers:**
   ```javascript
   // Instead of: node.parent = parent
   node.parent = new WeakRef(parent)
   ```

2. **Lazy initialization:**
   ```javascript
   // Don't set parent during deserialization
   // Set on first access via getter
   ```

3. **Platform-specific code:**
   ```javascript
   if (process.platform === 'win32') {
     // Skip parent references on Windows
   }
   ```

4. **Adjust Node.js heap size:**
   ```bash
   NODE_OPTIONS="--max-old-space-size=4096" node test-worker-main.mjs
   ```

5. **Feature flag:**
   ```javascript
   // Make parent prop opt-in
   parseSync(code, { addParent: false })
   ```

## Alternative: If Not Commit 7e4d04fad

Test other suspects:
- `e75d42d50` - preserveParens changes
- `68c0252ff` - deserializer refactoring
- Commits touching allocator: `f37b211d1`, `7a1c33948`

## Files to Reference

- `BISECT-ANALYSIS.md` - Detailed analysis of bisect results
- `REPRO-14732.md` - Original reproduction steps
- `.github/workflows/bisect-auto-14732.yml` - Automated bisect workflow
- `napi/parser/test-worker-main.mjs` - Test spawning 16 workers
- `napi/parser/test-worker.mjs` - Worker importing parser

## Decision Point

**Need Windows testing to proceed.** Without it, we're just speculating.

Options:
1. Ask issue reporter (@Boshen) to test specific commits
2. Use GitHub Actions Windows runner with manual workflow
3. Rent Windows VM for testing
4. Ask community with Windows machines to help

**Estimated time to resolution:** 1-2 hours with Windows access
