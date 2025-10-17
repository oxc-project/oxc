# GitHub Actions Guide for Bisecting Issue #14732

## Problem

The workflow files are on the `cam-debug-14732` branch, but GitHub Actions can only run workflows from the default branch (`main`) or the branch being tested needs to have the workflow file.

## Solution: Manual Bisect Using GitHub Actions

Since we can't automatically trigger workflows on old commits, we'll manually test key commits by creating test branches.

### Step 1: Test v0.95.0 (Expected: BAD/crash)

```bash
# Already created: test-v0.95.0 branch
```

Go to: https://github.com/oxc-project/oxc/actions/workflows/debug-14732.yml

Click "Run workflow" →
- Branch: `test-v0.95.0`
- Runs: `3`
- Click "Run workflow"

Expected result: **FAIL** (crashes with heap corruption)

### Step 2: Test v0.92.0 (Expected: GOOD/no crash)

```bash
# Already created: test-v0.92.0 branch
```

Go to: https://github.com/oxc-project/oxc/actions/workflows/debug-14732.yml

Click "Run workflow" →
- Branch: `test-v0.92.0`
- Runs: `3`
- Click "Run workflow"

Expected result: **PASS** (no crashes)

### Step 3: Binary Search

Once we confirm v0.92.0 is GOOD and v0.95.0 is BAD, we can manually bisect:

1. Find the midpoint commit between them
2. Create a test branch with that commit + test files
3. Run the workflow
4. Based on result, narrow the range
5. Repeat until we find the exact breaking commit

## Creating Test Branches

To test a specific commit:

```bash
# Checkout the commit
git checkout <commit-sha>

# Cherry-pick the test files
git cherry-pick cam-debug-14732

# Push to a test branch
git push origin HEAD:refs/heads/test-<name> --force
```

Then go to GitHub Actions and run the workflow on that branch.

## Commits Between v0.92.0 and v0.95.0

Key releases to test:
- v0.92.0: `1b3f43746` (GOOD baseline)
- v0.93.0: `aa0689fe3`
- v0.94.0: `f88f5f459`
- v0.95.0: `454ee94ff` (BAD baseline)

Suggested bisect order:
1. Test v0.93.0 (midpoint between v0.92.0 and v0.94.0)
2. Test v0.94.0 (if v0.93.0 is GOOD, test v0.94.0)
3. Based on results, narrow to specific commit range
4. Test individual commits in that range

## Alternative: Run Locally on Windows

If you have access to a Windows machine:

```bash
git checkout <commit-to-test>
cd napi/parser
pnpm install
pnpm build --features allocator --release
node test-worker-main.mjs
```

If it crashes with heap corruption → BAD
If all workers complete → GOOD
