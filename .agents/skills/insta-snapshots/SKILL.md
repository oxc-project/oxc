---
name: insta-snapshots
description: Guide for working with and updating insta snapshot tests in Oxc without terminal interaction.
---

This skill guides you through working with [insta](https://insta.rs) snapshot tests in the Oxc codebase without requiring terminal interaction.

## What are Insta Snapshots?

Insta is a snapshot testing library for Rust. Oxc uses it extensively for:

- Linter rule tests (`crates/oxc_linter/src/snapshots/`)
- Semantic analysis tests (`crates/oxc_semantic/tests/integration/snapshots/`)
- Other crate-specific snapshot tests

Snapshots track **expected test outputs** (often failures or errors). When code changes, new snapshots are generated as `.snap.new` files for review.

## Running Tests and Generating Snapshots

### Run tests for a specific crate:

```bash
cargo test -p <crate_name>
```

This generates `.snap.new` files if test outputs have changed.

## Reviewing Snapshots Non-Interactively

**IMPORTANT**: Avoid using `cargo insta review` (the interactive terminal UI). Instead, follow these steps:

### 1. List all pending snapshots

```bash
cargo insta pending-snapshots
# Or for workspace-wide:
cargo insta pending-snapshots --workspace
```

This shows all `.snap.new` files waiting for review.

### 2. Read the snapshot files directly

New snapshots are stored as `.snap.new` files next to their corresponding `.snap` files. You can use `cargo insta pending-snapshots` to view the changes to the snapshot files.

### 3. Accept or reject changes

**Accept all pending snapshots:**

```bash
cargo insta accept
# Or workspace-wide:
cargo insta accept --workspace
```

**Accept specific snapshot(s):**

```bash
cargo insta accept --snapshot <snapshot_name>
```

**Reject all pending snapshots:**

```bash
cargo insta reject
# Or workspace-wide:
cargo insta reject --workspace
```

This deletes all `.snap.new` files.

### 4. Verify the changes

After accepting, the `.snap.new` files become `.snap` files. Check with git:

```bash
git diff <path/to/snapshots/>
```

## Common Workflows

### After fixing a bug or adding new snapshot tests:

1. Run tests: `cargo test -p oxc_linter` (or relevant crate)
2. Check pending: `cargo insta pending-snapshots`
3. Read new snapshots to verify they match expected behavior
4. Accept if correct: `cargo insta accept`
5. Commit the updated `.snap` files

### Working with specific test files:

1. Run: `cargo test -p oxc_linter specific_test_name`
2. Read: `cargo insta pending-snapshots` - look for `specific_test_name.snap.new`
3. Accept: `cargo insta accept -p oxc_linter`

## Snapshot File Formats

- `.snap` - Current expected output
- `.snap.new` - New output from recent test run (pending review), do not commit if these files are present

Note that `.snap.md` files are from Vitest, and **not** Insta. They are used in conformance tests, and are not handled by the instructions in this skill.

## Tips

- **Always review snapshots manually** before accepting - don't blindly accept all changes
- Snapshot files are **checked into git** - they're part of the test suite
- Large snapshot changes may indicate breaking changes or bugs
- Use `git diff` after accepting to see what actually changed
- Use the CLI commands. **DO NOT** just copy-paste or manually edit `.snap` files.

## Example: Complete Workflow

```bash
# 1. Make a code change to a linter rule
# 2. Run tests
cargo test -p oxc_linter

# 3. See what changed
cargo insta pending-snapshots

# 4. Review specific snapshot (using Read tool)
# Read: crates/oxc_linter/src/snapshots/some_test.snap.new

# 5. Accept if correct
cargo insta accept -p oxc_linter

# 6. Verify with git
git diff crates/oxc_linter/src/snapshots/

# 7. Run tests again to ensure everything passes
cargo test -p oxc_linter
```

---

**When to use this skill**: When tests fail due to snapshot mismatches or after changing code that affects test outputs.
