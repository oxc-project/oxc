# Eliminate manual `state.changed` writes — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace 187 manual `ctx.state.changed = true` writes across `crates/oxc_minifier/` with four typed mutation helpers (`replace_expression`, `replace_statement`, `notice_change`, `reset_changed`), and add CI checks that prevent regressions.

**Architecture:** Pure refactor. The `MinifierState::changed: bool` field and the `LiveUsageCollector`-based `exit_program` refresh are unchanged. Helpers are thin `#[inline]` wrappers on `TraverseCtx<'a, MinifierState<'a>>` that bump `state.changed`. The migration is additive — helpers and legacy writes both target the same field, so any mid-migration state is correct.

**Tech Stack:** Rust workspace, `cargo` / `just` / `cargo coverage` / `cargo insta` / `ast-grep` (already installed per `AGENTS.md`).

**Spec:** `docs/superpowers/specs/2026-05-25-minifier-eliminate-changed-flag-design.md`

---

## File Structure

| Path                                                                                                 | Action                       | Responsibility                                                                                                                                                                  |
| ---------------------------------------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/oxc_minifier/src/traverse_context/ecma_context.rs`                                           | **Modify** (Task 1, Task 18) | Add helpers (Task 1) and tighten allowlist comments (Task 18). Existing impl block at line 167.                                                                                 |
| `crates/oxc_minifier/src/state.rs`                                                                   | **Modify** (Task 18)         | Change `MinifierState::changed: pub bool` → `pub(crate) bool`.                                                                                                                  |
| `crates/oxc_minifier/src/peephole/mod.rs`                                                            | **Modify** (Task 1, Task 5)  | Task 1: migrate `enter_program`'s `ctx.state.changed = false;` reset to `ctx.reset_changed()`. Task 5: migrate the 2 `= true` writes in `exit_statement` and `exit_expression`. |
| `crates/oxc_minifier/src/peephole/*.rs` (15 other files)                                             | **Modify** (Tasks 2-4, 6-17) | One task per file; mechanical conversion from `state.changed = true` to helper calls.                                                                                           |
| `tools/check_state_changed.sh`                                                                       | **Create** (Task 18)         | Bash script invoked from `just ready` that fails CI on any unauthorized `state.changed =` write.                                                                                |
| `justfile`                                                                                           | **Modify** (Task 18)         | Wire `check_state_changed.sh` into `just ready`.                                                                                                                                |
| `crates/oxc_minifier/sgconfig.yml` + `crates/oxc_minifier/rules/peephole-direct-slot-assignment.yml` | **Create** (Task 18)         | ast-grep configuration and rule that prevents direct `*expr = …` / `*stmt = …` in peephole code without an allowlist comment.                                                   |

---

## Reference: Transformation patterns

These five patterns cover every site. Tasks 2-17 reference this section.

### Pattern A — slot replace (expression)

```rust
- *expr = new;
- ctx.state.changed = true;
+ ctx.replace_expression(expr, new);
```

### Pattern B — slot replace (statement)

```rust
- *stmt = new;
- ctx.state.changed = true;
+ ctx.replace_statement(stmt, new);
```

### Pattern C — collection mutation

```rust
  let old_len = elems.len();
  elems.retain_mut(|e| !Self::remove_unused_expression(e, ctx));
- if elems.len() != old_len { ctx.state.changed = true; }
+ if elems.len() != old_len { ctx.notice_change(); }
```

### Pattern D — in-place tweak (operand swap, field flip)

```rust
  e.right = left;
  e.left = right;
- ctx.state.changed = true;
+ ctx.notice_change();
```

### Pattern E — conditional semantic gate (`dead_drop_mutates_ast`)

```rust
  if dead_drop_mutates_ast(&stmt) {
-     ctx.state.changed = true;
+     ctx.notice_change();
  }
```

The `if` STAYS. The mechanical sweep transforms only the body. **Verify by reading the surrounding code** before applying.

### Site classification rule

1. If the line above sets `*expr = X` or `*stmt = X`, use Pattern A or B (collapse two lines into one helper call).
2. Else, use Pattern C/D/E with `ctx.notice_change()`.
3. **Never** call `ctx.notice_change()` when `replace_expression`/`replace_statement` applies — Task 18's ast-grep rule will fail the build.

### Per-task verification commands

After applying the patterns to a file:

```bash
# Should print 0 (the file is fully migrated)
grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/<file>.rs

# Should print N (other files still have their original counts)
grep -rc "ctx.state.changed = true" crates/oxc_minifier/src/peephole/ | sort -t: -k2 -nr
```

---

## Task 1: Add the four helpers and migrate the reset (PR 1)

**Files:**

- Modify: `crates/oxc_minifier/src/traverse_context/ecma_context.rs` (extend impl block at line 167)
- Modify: `crates/oxc_minifier/src/peephole/mod.rs:165` (migrate the reset)

- [ ] **Step 1: Baseline test pass**

```bash
cargo test -p oxc_minifier
```

Expected: PASS.

- [ ] **Step 2: Record baseline `state.changed = true` count**

```bash
grep -rc "ctx.state.changed = true" crates/oxc_minifier/src/peephole/ | awk -F: '{s+=$2} END {print s}'
```

Expected: `187`. If different, the spec is stale — STOP and update the spec.

- [ ] **Step 3: Add the four helpers**

Append inside the existing `impl<'a> TraverseCtx<'a, MinifierState<'a>>` block in `crates/oxc_minifier/src/traverse_context/ecma_context.rs` (currently starts at line 167):

```rust
    /// Replace an expression slot. Marks the pass as having mutated the AST.
    ///
    /// Prefer this over a direct `*slot = new; ctx.state.changed = true;` pair —
    /// the helper is enforced by CI (see `tools/check_state_changed.sh`).
    #[inline]
    pub fn replace_expression(&mut self, slot: &mut Expression<'a>, new: Expression<'a>) {
        *slot = new;
        self.state.changed = true;
    }

    /// Replace a statement slot. Marks the pass as having mutated the AST.
    #[inline]
    pub fn replace_statement(&mut self, slot: &mut Statement<'a>, new: Statement<'a>) {
        *slot = new;
        self.state.changed = true;
    }

    /// Mark the pass as having mutated the AST in place (operand swap, in-place
    /// field flip, collection element removal, etc.) where no slot replacement
    /// happened. Prefer the `replace_*` helpers when the mutation IS a slot
    /// replacement.
    #[inline]
    pub fn notice_change(&mut self) {
        self.state.changed = true;
    }

    /// Clear the per-pass mutation signal. Called once at the top of each
    /// peephole traversal in `enter_program`. This is the only sanctioned
    /// way to write `state.changed = false`.
    #[inline]
    pub fn reset_changed(&mut self) {
        self.state.changed = false;
    }
```

- [ ] **Step 4: Migrate the false-reset**

Edit `crates/oxc_minifier/src/peephole/mod.rs:165`:

```rust
- ctx.state.changed = false;
+ ctx.reset_changed();
```

- [ ] **Step 5: Compile**

```bash
cargo build -p oxc_minifier
```

Expected: clean build.

- [ ] **Step 6: Run tests**

```bash
cargo test -p oxc_minifier
```

Expected: PASS, no snapshot diffs.

- [ ] **Step 7: Verify inlining (one-time check)**

Build the release binary and inspect a known call site. Pick any peephole call site that will be migrated in a later task (e.g. one of the two sites in `peephole/mod.rs` migrated by Task 5) and confirm the call lowers to `mov` + `mov $1, …` with no `call` to the helper body.

Tooling options (use whichever is installed in your environment):

- `cargo-show-asm`: `cargo asm --rust -p oxc_minifier "<symbol>"`
- `rustc` direct: `cargo rustc -p oxc_minifier --release -- --emit asm` and grep the output in `target/release/deps/*.s`.

Expected: helper invocations lower to inline stores. If you see `call oxc_minifier::…::replace_expression`, escalate to `#[inline(always)]` on all four helpers and re-run. Document the actual command used and the result in the PR description.

- [ ] **Step 8: Commit**

```bash
git add crates/oxc_minifier/src/traverse_context/ecma_context.rs \
        crates/oxc_minifier/src/peephole/mod.rs
git commit -m "refactor(minifier): add mutation helpers on TraverseCtx" -m "Introduces \`replace_expression\`, \`replace_statement\`, \`notice_change\`,
and \`reset_changed\` as the sanctioned API for marking a peephole pass as
having mutated the AST. Migrates the single \`state.changed = false\`
reset in \`enter_program\` to \`reset_changed()\`; all \`= true\` sites
will be migrated in subsequent PRs in this stack.

The legacy field stays \`pub\` for now so unmigrated modules can keep
writing to it directly. The final PR (\`refactor(minifier): lock down
state.changed writes\`) makes the field \`pub(crate)\` and adds the CI
check."
```

---

## Task 2: Migrate `inline.rs` (PR 2)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/inline.rs`

- [ ] **Step 1: Confirm count before**

```bash
grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/inline.rs
```

Expected: `1`.

- [ ] **Step 2: Locate the site**

```bash
grep -n "ctx.state.changed = true" crates/oxc_minifier/src/peephole/inline.rs
```

- [ ] **Step 3: Apply the appropriate transformation pattern**

Read the lines surrounding the site, classify per Reference §"Site classification rule," and apply Pattern A/B/C/D from the Reference.

- [ ] **Step 4: Confirm count after**

```bash
grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/inline.rs
```

Expected: `0`.

- [ ] **Step 5: Build + test**

```bash
cargo build -p oxc_minifier && cargo test -p oxc_minifier
```

Expected: clean build, all tests pass, no snapshot diffs.

- [ ] **Step 6: Run size snapshot check**

```bash
just minsize
git diff --stat tasks/minsize/
```

Expected: empty diff (no `tasks/minsize/` changes).

- [ ] **Step 7: Commit**

```bash
git add crates/oxc_minifier/src/peephole/inline.rs
git commit -m "refactor(minifier): migrate inline.rs to mutation helpers"
```

---

## Task 3: Migrate `remove_unused_private_members.rs` (PR 3)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/remove_unused_private_members.rs`

- [ ] **Step 1: Confirm count before**

```bash
grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/remove_unused_private_members.rs
```

Expected: `1`.

- [ ] **Step 2: Locate and classify the site**

```bash
grep -n "ctx.state.changed = true" crates/oxc_minifier/src/peephole/remove_unused_private_members.rs
```

Then read the surrounding 4 lines to classify per Reference §"Site classification rule."

- [ ] **Step 3: Apply the appropriate Reference pattern**

- [ ] **Step 4: Confirm count after**

```bash
grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/remove_unused_private_members.rs
```

Expected: `0`.

- [ ] **Step 5: Build + test**

```bash
cargo build -p oxc_minifier && cargo test -p oxc_minifier
```

- [ ] **Step 6: Size snapshot check**

```bash
just minsize && git diff --stat tasks/minsize/
```

Expected: empty diff.

- [ ] **Step 7: Commit**

```bash
git add crates/oxc_minifier/src/peephole/remove_unused_private_members.rs
git commit -m "refactor(minifier): migrate remove_unused_private_members.rs to mutation helpers"
```

---

## Task 4: Migrate `minimize_for_statement.rs` (PR 4)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/minimize_for_statement.rs`

- [ ] **Step 1: Confirm count before**

```bash
grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/minimize_for_statement.rs
```

Expected: `2`.

- [ ] **Step 2: Locate and classify all sites**

```bash
grep -n -B1 -A1 "ctx.state.changed = true" crates/oxc_minifier/src/peephole/minimize_for_statement.rs
```

- [ ] **Step 3: Apply the appropriate Reference pattern to each site**

- [ ] **Step 4: Confirm count after**

```bash
grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/minimize_for_statement.rs
```

Expected: `0`.

- [ ] **Step 5: Build + test**

```bash
cargo build -p oxc_minifier && cargo test -p oxc_minifier
```

- [ ] **Step 6: Size snapshot check**

```bash
just minsize && git diff --stat tasks/minsize/
```

Expected: empty diff.

- [ ] **Step 7: Commit**

```bash
git add crates/oxc_minifier/src/peephole/minimize_for_statement.rs
git commit -m "refactor(minifier): migrate minimize_for_statement.rs to mutation helpers"
```

---

## Task 5: Migrate `peephole/mod.rs` (PR 5)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/mod.rs`

The 2 sites in this file are inside the `exit_statement`/`exit_expression` visitor methods at lines 229 and 385. Both fit Pattern A (slot replace).

- [ ] **Step 1: Confirm count before**

```bash
grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/mod.rs
```

Expected: `2`.

- [ ] **Step 2: Locate the sites**

```bash
grep -n -B3 "ctx.state.changed = true" crates/oxc_minifier/src/peephole/mod.rs
```

- [ ] **Step 3: Apply Pattern A to both sites**

Line 229 region (currently):

```rust
                    if let Statement::IfStatement(if_stmt) = stmt
                        && let Some(folded_stmt) = Self::try_minimize_if(if_stmt, ctx)
                    {
                        *stmt = folded_stmt;
                        ctx.state.changed = true;
                    }
```

Becomes:

```rust
                    if let Statement::IfStatement(if_stmt) = stmt
                        && let Some(folded_stmt) = Self::try_minimize_if(if_stmt, ctx)
                    {
                        ctx.replace_statement(stmt, folded_stmt);
                    }
```

Line 385 region (currently):

```rust
                    if let Some(changed) = Self::minimize_conditional_expression(logical_expr, ctx)
                    {
                        *expr = changed;
                        ctx.state.changed = true;
                    }
```

Becomes:

```rust
                    if let Some(changed) = Self::minimize_conditional_expression(logical_expr, ctx)
                    {
                        ctx.replace_expression(expr, changed);
                    }
```

- [ ] **Step 4: Confirm count after**

```bash
grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/mod.rs
```

Expected: `0`.

- [ ] **Step 5: Build + test**

```bash
cargo build -p oxc_minifier && cargo test -p oxc_minifier
```

- [ ] **Step 6: Size snapshot check**

```bash
just minsize && git diff --stat tasks/minsize/
```

Expected: empty diff.

- [ ] **Step 7: Commit**

```bash
git add crates/oxc_minifier/src/peephole/mod.rs
git commit -m "refactor(minifier): migrate peephole/mod.rs to mutation helpers"
```

---

## Task 6: Migrate `minimize_logical_expression.rs` (PR 6)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/minimize_logical_expression.rs`

- [ ] **Step 1: Count before — expected `3`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns A/B/C/D to each site (classify by reading surrounding code).**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/minimize_logical_expression.rs
  git commit -m "refactor(minifier): migrate minimize_logical_expression.rs to mutation helpers"
  ```

---

## Task 7: Migrate `minimize_not_expression.rs` (PR 7)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/minimize_not_expression.rs`

- [ ] **Step 1: Count before — expected `3`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns A/B/C/D to each site.**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/minimize_not_expression.rs
  git commit -m "refactor(minifier): migrate minimize_not_expression.rs to mutation helpers"
  ```

---

## Task 8: Migrate `minimize_if_statement.rs` (PR 8)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/minimize_if_statement.rs`

- [ ] **Step 1: Count before — expected `5`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns to each site.**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/minimize_if_statement.rs
  git commit -m "refactor(minifier): migrate minimize_if_statement.rs to mutation helpers"
  ```

---

## Task 9: Migrate `remove_unused_declaration.rs` (PR 9)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/remove_unused_declaration.rs`

- [ ] **Step 1: Count before — expected `5`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns to each site.**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/remove_unused_declaration.rs
  git commit -m "refactor(minifier): migrate remove_unused_declaration.rs to mutation helpers"
  ```

---

## Task 10: Migrate `minimize_conditions.rs` (PR 10)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/minimize_conditions.rs`

- [ ] **Step 1: Count before — expected `6`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns to each site.**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/minimize_conditions.rs
  git commit -m "refactor(minifier): migrate minimize_conditions.rs to mutation helpers"
  ```

---

## Task 11: Migrate `minimize_expression_in_boolean_context.rs` (PR 11)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/minimize_expression_in_boolean_context.rs`

- [ ] **Step 1: Count before — expected `6`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns to each site.**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/minimize_expression_in_boolean_context.rs
  git commit -m "refactor(minifier): migrate minimize_expression_in_boolean_context.rs to mutation helpers"
  ```

---

## Task 12: Migrate `replace_known_methods.rs` (PR 12)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/replace_known_methods.rs`

- [ ] **Step 1: Count before — expected `6`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns to each site.**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/replace_known_methods.rs
  git commit -m "refactor(minifier): migrate replace_known_methods.rs to mutation helpers"
  ```

---

## Task 13: Migrate `fold_constants.rs` (PR 13)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/fold_constants.rs`

- [ ] **Step 1: Count before — expected `14`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns to each site. Most are Pattern A (`*expr = changed; ctx.state.changed = true;`).**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/fold_constants.rs
  git commit -m "refactor(minifier): migrate fold_constants.rs to mutation helpers"
  ```

---

## Task 14: Migrate `remove_dead_code.rs` (PR 14)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/remove_dead_code.rs`

- [ ] **Step 1: Count before — expected `19`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns to each site. Watch for Pattern C (collection mutations) since DCE often pops from statement vectors.**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/remove_dead_code.rs
  git commit -m "refactor(minifier): migrate remove_dead_code.rs to mutation helpers"
  ```

---

## Task 15: Migrate `remove_unused_expression.rs` (PR 15)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/remove_unused_expression.rs`

- [ ] **Step 1: Count before — expected `27`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns to each site. This file has a mix of Pattern A (slot replace via `*e = …`) and Pattern C (collection mutations like `sequence_expr.expressions.retain_mut(…)`).**
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/remove_unused_expression.rs
  git commit -m "refactor(minifier): migrate remove_unused_expression.rs to mutation helpers"
  ```

---

## Task 16: Migrate `substitute_alternate_syntax.rs` (PR 16)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/substitute_alternate_syntax.rs`

This is the largest single-file migration. Most sites are Pattern A; a notable few are Pattern D (the binary-operand swap around line 419).

- [ ] **Step 1: Count before — expected `43`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Apply Reference patterns to each site. Special cases to verify:**
  - The operand swap at line ~419 (`e.right = left; e.left = right;`) is Pattern D → `ctx.notice_change()`.
  - The `substitute_chain_call_expression` at line ~1305 (creates a new reference via `create_unbound_reference`, no slot replacement) is Pattern D → `ctx.notice_change()`.
  - The `substitute_is_object_and_not_null_for_left_and_right` at lines ~447 and ~470 is Pattern A despite the rebuilt `new_expr` preserving semantic IDs from the old — this is fine because the helpers do not walk for refs.
- [ ] **Step 4: Count after — expected `0`.**
- [ ] **Step 5: `cargo build -p oxc_minifier && cargo test -p oxc_minifier` — expected PASS.**
- [ ] **Step 6: `just minsize && git diff --stat tasks/minsize/` — expected empty diff.**
- [ ] **Step 7: Commit:**
  ```bash
  git add crates/oxc_minifier/src/peephole/substitute_alternate_syntax.rs
  git commit -m "refactor(minifier): migrate substitute_alternate_syntax.rs to mutation helpers"
  ```

---

## Task 17: Migrate `minimize_statements.rs` — SPECIAL CARE for `dead_drop_mutates_ast` (PR 17)

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/minimize_statements.rs`

This is the largest single-file migration and has the `dead_drop_mutates_ast` gate at line 22. **Pattern E (conditional semantic gate) applies to one or more sites in this file.** Removing the `if dead_drop_mutates_ast(...)` wrapper would cause the peephole loop to spin forever; the existing `Ran loop more than 10 times` `debug_assert!` in `compressor.rs:86` would catch it in tests, but only after wasted iterations — easier to get it right.

- [ ] **Step 1: Count before — expected `44`.**
- [ ] **Step 2: Locate sites with `grep -n -B3 "ctx.state.changed = true"`.**
- [ ] **Step 3: Locate the `dead_drop_mutates_ast` gate**

```bash
grep -n -A3 "dead_drop_mutates_ast" crates/oxc_minifier/src/peephole/minimize_statements.rs
```

For every site INSIDE an `if dead_drop_mutates_ast(...)` block (or otherwise gated by a `dead_drop_mutates_ast` call), apply **Pattern E** — keep the `if`, replace only the body.

- [ ] **Step 4: Apply Reference patterns to all 44 sites**

Most sites in this file are Pattern A (slot replace, e.g. line 269 region where a sequence of throw statements is merged into one). Be careful with Pattern E sites; verify each by reading the surrounding 6 lines.

- [ ] **Step 5: Count after — expected `0`.**
- [ ] **Step 6: Build + test**

```bash
cargo build -p oxc_minifier && cargo test -p oxc_minifier
```

Expected: PASS. If a test fails with `Ran loop more than 10 times`, a `dead_drop_mutates_ast` gate was inadvertently removed — revisit step 4.

- [ ] **Step 7: Size snapshot check**

```bash
just minsize && git diff --stat tasks/minsize/
```

Expected: empty diff.

- [ ] **Step 8: Commit:**

```bash
git add crates/oxc_minifier/src/peephole/minimize_statements.rs
git commit -m "refactor(minifier): migrate minimize_statements.rs to mutation helpers" -m "Preserves the \`dead_drop_mutates_ast\` gate at line 22 — dropping
that gate would let the peephole fixed-point loop spin forever on
\`var x;\` declarations that \`KeepVar\` re-emits unchanged."
```

---

## Task 18: Final lockdown — make field `pub(crate)` and add CI checks (PR 18)

**Files:**

- Modify: `crates/oxc_minifier/src/state.rs:33` (`pub changed: bool` → `pub(crate) changed: bool`)
- Create: `tools/check_state_changed.sh`
- Modify: `justfile` (wire the check into `just ready`)
- Create: `crates/oxc_minifier/sgconfig.yml`
- Create: `crates/oxc_minifier/rules/peephole-direct-slot-assignment.yml`

- [ ] **Step 1: Verify zero unauthorized writes**

```bash
rg -n 'state\.changed\s*=' crates/oxc_minifier/ \
  --glob '!crates/oxc_minifier/src/traverse_context/ecma_context.rs'
```

Expected: **no output**. If any lines print, a previous task missed a site — go back to the task that owns that file and complete it before continuing.

- [ ] **Step 2: Tighten field visibility**

Edit `crates/oxc_minifier/src/state.rs` line 33:

```rust
- pub changed: bool,
+ pub(crate) changed: bool,
```

- [ ] **Step 3: Add the grep CI check**

Create `tools/check_state_changed.sh`:

```bash
#!/usr/bin/env bash
# Fails if any `state.changed =` write exists in crates/oxc_minifier/
# outside the four sanctioned helpers in ecma_context.rs.
#
# See: docs/superpowers/specs/2026-05-25-minifier-eliminate-changed-flag-design.md
set -euo pipefail

violations=$(rg -n 'state\.changed\s*=' crates/oxc_minifier/ \
    --glob '!crates/oxc_minifier/src/traverse_context/ecma_context.rs' \
    || true)

if [ -n "$violations" ]; then
    echo "ERROR: Unauthorized state.changed writes detected." >&2
    echo "       Use ctx.replace_expression / ctx.replace_statement /" >&2
    echo "       ctx.notice_change / ctx.reset_changed instead." >&2
    echo "" >&2
    echo "$violations" >&2
    exit 1
fi
```

Make it executable:

```bash
chmod +x tools/check_state_changed.sh
```

- [ ] **Step 4: Verify the check passes**

```bash
./tools/check_state_changed.sh && echo OK
```

Expected: `OK`.

- [ ] **Step 5: Wire into `just ready`**

Read the existing `just ready` recipe:

```bash
grep -n -A20 "^ready:" justfile
```

Add a new line invoking the check at the start of the recipe. Concrete edit (insert after `ready:` declaration, before the first existing step):

```makefile
ready:
    ./tools/check_state_changed.sh
    # ... existing steps unchanged ...
```

Then run:

```bash
just ready
```

Expected: full ready pipeline passes including the new check.

- [ ] **Step 6: Add the ast-grep structural check**

Create `crates/oxc_minifier/sgconfig.yml`:

```yaml
ruleDirs:
  - rules
```

Create `crates/oxc_minifier/rules/peephole-direct-slot-assignment.yml`:

```yaml
id: peephole-direct-slot-assignment
language: rust
severity: error
message: |
  Direct slot assignment in peephole code must use ctx.replace_expression
  or ctx.replace_statement instead of a raw `*slot = …; ctx.notice_change();`
  pair. The typed helper is enforced by CI; see
  docs/superpowers/specs/2026-05-25-minifier-eliminate-changed-flag-design.md.

  To explicitly allowlist a justified exception, add a comment on the line
  above the assignment:
    // ast-grep-ignore: peephole-direct-slot-assignment — reason: <why>

rule:
  any:
    - pattern: "*$SLOT = $VALUE"

files:
  - src/peephole/**/*.rs
```

- [ ] **Step 7: Verify the ast-grep check passes on current code**

```bash
cd crates/oxc_minifier && ast-grep scan && cd ../..
```

Expected: zero violations. If any are flagged, they are real bypass attempts hiding in the codebase — investigate, decide whether to use `replace_*` or add the allowlist comment with a written justification, then re-run.

- [ ] **Step 8: Wire the ast-grep check into `just ready`**

Edit the `ready` recipe in `justfile` again to add the ast-grep step:

```makefile
ready:
    ./tools/check_state_changed.sh
    cd crates/oxc_minifier && ast-grep scan
    # ... existing steps unchanged ...
```

Then run:

```bash
just ready
```

Expected: full pipeline passes.

- [ ] **Step 9: Full regression suite**

```bash
cargo test -p oxc_minifier
cargo test -p oxc_mangler
cargo coverage -- minifier
just minsize && git diff --stat tasks/minsize/
```

Expected: all pass, no size diffs.

- [ ] **Step 10: Commit:**

```bash
git add crates/oxc_minifier/src/state.rs \
        tools/check_state_changed.sh \
        justfile \
        crates/oxc_minifier/sgconfig.yml \
        crates/oxc_minifier/rules/peephole-direct-slot-assignment.yml
git commit -m "refactor(minifier)!: lock down state.changed writes" -m "After 17 prior PRs migrated every \`state.changed = true\` site to the
typed mutation helpers, this PR:

  - Makes \`MinifierState::changed\` \`pub(crate)\` (was \`pub\`).
  - Adds \`tools/check_state_changed.sh\` to fail CI on any future
    \`state.changed =\` write outside the four helper bodies in
    \`ecma_context.rs\`.
  - Adds an ast-grep rule that catches the structural bypass
    \`*slot = new; ctx.notice_change();\` — where no \`state.changed\`
    line exists for grep to find but the typed helper was skipped.

Both checks are wired into \`just ready\`. The bug class on the branch
\`fix/minifier-mark-changed-on-dead-stmt-drop\` becomes impossible: the
only sanctioned way to signal a mutation is through a helper.

BREAKING CHANGE: \`MinifierState::changed\` is no longer \`pub\`.
External crates that read or write the field directly must use the
helper accessors or stop reading the field."
```

---

## Final verification (after all 18 PRs land)

Independent of any single PR, after the stack is fully merged:

- [ ] **Whole-crate verification**

```bash
# Zero direct writes anywhere in the minifier crate
rg -n 'state\.changed\s*=' crates/oxc_minifier/ \
  --glob '!crates/oxc_minifier/src/traverse_context/ecma_context.rs'
# Expected: no output.

# The four helpers exist as expected
rg -n 'pub fn (replace_expression|replace_statement|notice_change|reset_changed)' \
   crates/oxc_minifier/src/traverse_context/ecma_context.rs
# Expected: 4 lines.

# Final tests
cargo test -p oxc_minifier
cargo test -p oxc_mangler
cargo coverage -- minifier
just minsize && git diff --stat tasks/minsize/
just ready
```

All expected to pass with no diffs.

---

## Notes for the executing engineer

- **Conventional Commits required.** `.github/workflows/pr.yml` validates PR titles against `^(build|chore|ci|docs|feat|fix|perf|refactor|release|revert|style|test)\(scope\)!?: .+`. All commit subjects above already match.
- **Each per-file task should produce a clean PR with zero `minsize` deltas.** If a delta appears, the migration accidentally changed semantics — revert, re-classify the site, and try again.
- **The `dead_drop_mutates_ast` gate (Task 17 Step 3) is the only correctness-sensitive case.** Every other transformation is mechanical.
- **Use Graphite for the stack.** See the `graphite-pr` skill in this environment.
- **Branch base:** all PRs stack on top of `spec/minifier-eliminate-changed-flag` (which currently holds only the spec doc). The first implementation PR (Task 1) is a child of that branch.
