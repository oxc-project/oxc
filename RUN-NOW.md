# ▶️ Run the Automated Bisect NOW

## No merge needed! Run directly from the branch.

---

## 🚀 Quick Start (3 Steps)

### Step 1: Go to Actions
**URL:** https://github.com/oxc-project/oxc/actions

### Step 2: Find the Workflow
In the left sidebar, click: **"Auto Bisect Issue 14732"**

(If you don't see it yet, wait 1-2 minutes for GitHub to index the workflow, then refresh)

### Step 3: Run It!
1. Click the **"Run workflow"** dropdown button (top right)
2. In **"Use workflow from"** dropdown: Select **`cam-debug-14732`** ⚠️ (NOT main!)
3. Leave defaults:
   - good_commit: `1b3f43746`
   - bad_commit: `454ee94ff`
4. Click the green **"Run workflow"** button

---

## ⏱️ What Happens Next

- The workflow runs on Windows (windows-latest)
- Takes ~30-90 minutes
- Tests ~10-15 commits automatically
- Uses `git bisect run` to find the breaking commit
- Outputs result in the Summary tab

---

## 📊 How to Check Results

1. Go to: https://github.com/oxc-project/oxc/actions/workflows/bisect-auto-14732.yml
2. Click on the running workflow (top of the list)
3. Wait for it to complete
4. Click the **"Summary"** tab
5. See the **"First Bad Commit"** 🎯

---

## 🔗 Direct Links

- **Workflow file:** https://github.com/oxc-project/oxc/blob/cam-debug-14732/.github/workflows/bisect-auto-14732.yml
- **Actions page:** https://github.com/oxc-project/oxc/actions
- **This branch:** https://github.com/oxc-project/oxc/tree/cam-debug-14732

---

## ❓ Troubleshooting

**Q: I don't see "Auto Bisect Issue 14732" in the sidebar**

A: Wait 1-2 minutes after the push, then refresh. GitHub needs to index the workflow.

**Q: Can I run this multiple times?**

A: Yes! Each run is independent.

**Q: Can I test a different commit range?**

A: Yes! When running the workflow, change the good_commit and bad_commit values.

**Q: What if the workflow fails?**

A: Check the logs in the workflow run. May need to adjust timeouts or handle build failures.

---

## 🎯 Expected Result

The workflow will output something like:

```
First bad commit: abc123def456...

commit abc123def456...
Author: Someone
Date: Mon Oct 14 2025

    feat(napi): some change that broke things
    
    - Changed X
    - Modified Y
```

Then you can investigate that specific commit to understand what broke!

---

**Ready to run!** No merge required. 🚀
