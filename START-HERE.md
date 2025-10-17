# Start Here - Bisect Issue #14732

## 🚀 RECOMMENDED: Automated Bisect (One Click!)

### ⭐ Use the Automated Bisect Workflow

**This workflow automatically finds the breaking commit using `git bisect run`**

1. Go to: https://github.com/oxc-project/oxc/actions/workflows/bisect-auto-14732.yml

2. Click "Run workflow" dropdown:
   - **Use workflow from:** `cam-debug-14732` (or `main` if merged)
   - **good_commit:** `1b3f43746` (v0.92.0 - leave default)
   - **bad_commit:** `454ee94ff` (v0.95.0 - leave default)
   - Click green "Run workflow" button

3. Wait ~30-90 minutes - it will test ~10-15 commits automatically

4. Check the "Summary" tab to see the **First Bad Commit** 🎯

**That's it!** The workflow will automatically:
- Test each commit in the range
- Build oxc-parser with the allocator feature
- Run the worker thread test
- Identify the exact breaking commit

---

## Alternative: Manual Testing (Step by Step)

If you want to test specific commits manually or the automated bisect fails:

The GitHub Actions workflows are ready to use. Follow these steps:

### 1. Test v0.95.0 (Expected: CRASH)

**URL:** https://github.com/oxc-project/oxc/actions/workflows/debug-14732.yml

Click "Run workflow" dropdown:
- **Use workflow from:** `test-v0.95.0`
- **commit:** leave empty
- **runs:** `3`
- Click green "Run workflow" button

Expected result: ❌ **FAIL** - Process crashes with heap corruption

---

### 2. Test v0.92.0 (Expected: SUCCESS)

**URL:** https://github.com/oxc-project/oxc/actions/workflows/debug-14732.yml

Click "Run workflow" dropdown:
- **Use workflow from:** `test-v0.92.0`
- **commit:** leave empty
- **runs:** `3`
- Click green "Run workflow" button

Expected result: ✅ **PASS** - All workers complete successfully

---

### 3. Test v0.93.0 (Result determines next step)

**URL:** https://github.com/oxc-project/oxc/actions/workflows/debug-14732.yml

Click "Run workflow" dropdown:
- **Use workflow from:** `test-crates_v0.93.0`
- **commit:** leave empty
- **runs:** `3`
- Click green "Run workflow" button

If PASS → Breaking change is between v0.93.0 and v0.94.0
If FAIL → Breaking change is between v0.92.0 and v0.93.0

---

### 4. Test v0.94.0 (Based on v0.93.0 result)

**URL:** https://github.com/oxc-project/oxc/actions/workflows/debug-14732.yml

Click "Run workflow" dropdown:
- **Use workflow from:** `test-crates_v0.94.0`
- **commit:** leave empty
- **runs:** `3`
- Click green "Run workflow" button

---

## Interpreting Results

### ✅ PASS (GOOD commit)
- All workers complete successfully
- "All workers done!" message appears
- Exit code: 0
- No crash or timeout

### ❌ FAIL (BAD commit)
- Process crashes mid-execution
- Workers don't all complete
- Exit code: non-zero
- May see heap corruption error

---

## After Testing

Once you identify which version broke (v0.93.0 or v0.94.0), you can:

1. **Check the commit range:**
   ```bash
   # Between v0.92.0 and v0.93.0
   git log --oneline 1b3f43746..aa0689fe3

   # Between v0.93.0 and v0.94.0
   git log --oneline aa0689fe3..f88f5f459

   # Between v0.94.0 and v0.95.0
   git log --oneline f88f5f459..454ee94ff
   ```

2. **Create more test branches** for commits in the narrowed range

3. **Investigate the breaking commit** once found

---

## Alternative: Local Testing on Windows

If you have access to a Windows machine:

```bash
git checkout <commit-to-test>
cd napi/parser
pnpm install
pnpm build --features allocator --release
node test-worker-main.mjs
```

Watch for crashes or "All workers done!" message.

---

## Documentation

- **REPRO-14732.md** - Detailed reproduction information
- **GITHUB-ACTIONS-GUIDE.md** - Detailed GitHub Actions guide
- **SUMMARY.md** - Investigation summary

---

## Test Branches Available

- `test-v0.92.0` - v0.92.0 + test files (GOOD baseline)
- `test-crates_v0.93.0` - v0.93.0 + test files
- `test-crates_v0.94.0` - v0.94.0 + test files
- `test-v0.95.0` - v0.95.0 + test files (BAD baseline)

All branches have the workflow files and test scripts included.
