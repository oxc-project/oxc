---
name: insta-snapshots
description: Review and update Oxc Insta snapshots non-interactively after expected output changes or snapshot test failures.
---

# Insta snapshots

Run the affected test to generate pending snapshots. Use `cargo insta pending-snapshots` and read the old `.snap` and new `.snap.new` outputs to decide whether each change matches the intended behavior.

Use the CLI instead of the interactive `cargo insta review` UI or manually editing expected output:

```bash
cargo insta pending-snapshots
cargo insta accept --snapshot <snapshot_name>
```

Scope acceptance to the reviewed snapshots from this task; check `cargo insta accept --help` if a name or package filter is ambiguous. Avoid workspace-wide acceptance or rejection when unrelated pending snapshots exist. Fix unexpected output in the implementation before accepting it.

After accepting, inspect the snapshot diff and rerun the affected test. Finish with the intended `.snap` updates and no task-generated `.snap.new` files left pending. Updating snapshots does not itself call for a commit.

Snapshots record expected output, including successful output in some crates. Linter diagnostic snapshots are under `crates/oxc_linter/src/snapshots/`; other crates have their own locations. Conformance `.snap.md` reports use a separate update mechanism; inspect the producing test harness rather than treating them as Insta pending snapshots.
