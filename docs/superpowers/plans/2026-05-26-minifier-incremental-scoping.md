# Incremental scoping refresh + mutation counter — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `exit_program` `LiveUsageCollector` post-pass walk with per-helper incremental dead-ref tracking; replace `state.changed: bool` with `mutations: u64` driven by snapshot-compare; delete the bool, the reset helper, and the collector.

**Architecture:** Each mutation helper walks both the dropped subtree and the replacement subtree, accumulating `dead = walk(old) − walk(new)` into a per-pass `PassDirty` set. `drop_*` helpers walk only the dropped subtree. Direct-eval refresh stays as a gated full-walk (only runs when a `eval(...)` call was actually dropped this pass). `exit_program` consumes the dirty data directly with no walk over live refs.

**Tech Stack:** Rust workspace, `cargo` / `just` / `cargo coverage` / `cargo insta` / `ast-grep` / `oxc_ast_visit`.

**Spec:** `docs/superpowers/specs/2026-05-26-minifier-incremental-scoping-refresh-design.md`

**Branch:** `spec/minifier-incremental-scoping` (already created, stacked on `spec/minifier-eliminate-changed-flag`).

**Ship as:** ONE PR with 5 sequenced commits (per user direction). Each commit is individually revertable.

---

## File Structure

| Path                                                               | Action                          | Responsibility                                                                                                                                                                                                                                                                                                  |
| ------------------------------------------------------------------ | ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/oxc_minifier/src/peephole/convert_to_dotted_properties.rs` | **Modify** (Commit 1)           | Refactor `convert_to_dotted_properties` to take `&mut TraverseCtx<'a>` and use `replace_expression`. Removes 2 silent-bypass sites.                                                                                                                                                                             |
| `crates/oxc_minifier/src/peephole/minimize_statements.rs`          | **Modify** (Commit 1, Commit 2) | Commit 1: refactor `substitute_single_use_symbol_in_expression` / `..._from_declarators` / `..._in_statement` to take `&mut TraverseCtx`, use `replace_expression` at line 1387, drop the caller-side `if changed { ctx.notice_change() }` chain. Commit 2: migrate Pattern C/D drop sites to `drop_*` helpers. |
| `crates/oxc_minifier/src/peephole/mod.rs`                          | **Modify** (Commit 1, Commit 5) | Commit 1: update `exit_member_expression` caller site for the new signature. Commit 5: rewrite `exit_program` to consume `dirty.*`; delete `LiveUsageCollector`; add `LiveDirectEvalCollector`.                                                                                                                 |
| `crates/oxc_minifier/src/peephole/*.rs`                            | **Modify** (Commit 2)           | Migrate every Pattern C/D site that drops references via Option-clear, `retain_mut`-drop, `pop`/`drain`/`truncate`/`splice`, or Class field `take()`.                                                                                                                                                           |
| `crates/oxc_minifier/src/traverse_context/ecma_context.rs`         | **Modify** (Commits 2, 3, 4)    | Commit 2: add `drop_expression` + `drop_statement` helpers. Commit 3: remove `reset_changed`. Commit 4: route the walking helpers through `DropDiff`.                                                                                                                                                           |
| `crates/oxc_minifier/src/state.rs`                                 | **Modify** (Commits 3, 4, 5)    | Commit 3: add `mutations: u64`. Commit 4: add `dirty: PassDirty<'a>`. Commit 5: remove `changed: bool`.                                                                                                                                                                                                         |
| `crates/oxc_minifier/src/compressor.rs`                            | **Modify** (Commit 3)           | Switch `run_in_loop` to snapshot-compare on `state.mutations`.                                                                                                                                                                                                                                                  |
| `crates/oxc_minifier/src/traverse_context/drop_diff.rs`            | **Create** (Commit 4)           | `DropDiff` collector; `walk_old_*` / `resurrect_from_*` methods built on `oxc_ast_visit::Visit`.                                                                                                                                                                                                                |
| `crates/oxc_semantic/src/scoping.rs`                               | **Modify** (Commit 5)           | Add `retain_resolved_references_excluding(&FxHashSet<ReferenceId>)`. Add `remove_unresolved_reference(name)` if not present.                                                                                                                                                                                    |

---

## Commit 1: Latent-bug fixes

**Goal:** Remove the two `convert_to_dotted_properties.rs` silent-bypass sites and the
`minimize_statements.rs:1387` caller-tracked bypass. After this commit, EVERY peephole
mutation goes through a typed helper. `just minsize` IS expected to show non-zero
deltas; the deltas correspond to optimizations that the missed signals were silently
suppressing.

### Task 1.1: Refactor `convert_to_dotted_properties` to `&mut TraverseCtx`

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/convert_to_dotted_properties.rs`
- Modify: `crates/oxc_minifier/src/peephole/mod.rs:517` (the one call site)

- [ ] **Step 1: Baseline tests**

```bash
cargo test -p oxc_minifier
```

Expected: PASS.

- [ ] **Step 2: Change the function signature and body**

Edit `crates/oxc_minifier/src/peephole/convert_to_dotted_properties.rs`:

```rust
use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_syntax::identifier::is_identifier_name_patched;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Converts property accesses from quoted string or bracket access syntax to dot or unquoted string
    /// syntax, where possible. Dot syntax is more compact.
    ///
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/ConvertToDottedProperties.java>
    ///
    /// `foo['bar']` -> `foo.bar`
    /// `foo?.['bar']` -> `foo?.bar`
    pub fn convert_to_dotted_properties(
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let MemberExpression::ComputedMemberExpression(e) = expr else { return };
        let Expression::StringLiteral(s) = &e.expression else { return };
        if is_identifier_name_patched(&s.value) {
            let property = ctx.ast.identifier_name(s.span, s.value);
            let new_member = ctx.ast.alloc_static_member_expression(
                e.span,
                e.object.take_in(ctx.ast),
                property,
                e.optional,
            );
            // The whole `MemberExpression` slot is replaced. Use the typed helper
            // path: build the new wrapped value first, then go through the helper
            // for the inner enum slot. Since `MemberExpression` is an enum (not
            // Expression/Statement/AssignmentTargetProperty/PropertyKey), there's
            // no `replace_member_expression` helper — fall back to direct
            // assignment + `notice_change()`, mirroring the way the existing
            // `*prop` / `*key` sites in `substitute_alternate_syntax.rs` are
            // handled (see PR 18 lockdown's Category C allowlist).
            *expr = MemberExpression::StaticMemberExpression(new_member);
            ctx.notice_change();
            return;
        }
        let v = s.value.as_str();
        if e.optional {
            return;
        }
        if let Some(n) = TraverseCtx::string_to_equivalent_number_value(v) {
            let new_expr =
                ctx.ast.expression_numeric_literal(s.span, n, None, NumberBase::Decimal);
            ctx.replace_expression(&mut e.expression, new_expr);
        }
    }
}
```

Two changes:

- Signature: `&TraverseCtx<'a>` → `&mut TraverseCtx<'a>`.
- Line 26-32 (was): keep the direct `*expr = …` for the enum slot (no typed helper
  exists for `MemberExpression` enum) but follow it with `ctx.notice_change()` so the
  change is tracked. Remove the stale `// ast-grep-ignore` block.
- Line 40-44 (was): `e.expression = …` is a field assignment to an `Expression<'a>`
  slot — use `replace_expression(&mut e.expression, …)`. Remove the stale
  `// ast-grep-ignore` block.

- [ ] **Step 3: Update the one caller in `peephole/mod.rs`**

Read line 517: `Self::convert_to_dotted_properties(expr, ctx);`. The enclosing
`exit_member_expression` already has `&mut TraverseCtx<'a>`, so no change needed at
the call site — Rust's reborrow rules handle the conversion. Verify by attempting to
build.

- [ ] **Step 4: Build**

```bash
cargo build -p oxc_minifier
```

Expected: clean build. If you get a borrow error at the call site, add an explicit
reborrow: `Self::convert_to_dotted_properties(expr, &mut *ctx);`.

- [ ] **Step 5: Run tests + minsize**

```bash
cargo test -p oxc_minifier
just minsize
git diff --stat tasks/minsize/
```

Expected: tests PASS. `just minsize` may report non-zero diff in `tasks/minsize/` —
that's the latent-bug fix surfacing as a correct optimization. Capture the diff (`git
diff tasks/minsize/ > /tmp/minsize-1.1.diff`) for the commit body.

- [ ] **Step 6: Run the existing CI gates**

```bash
./tools/check_state_changed.sh && cd crates/oxc_minifier && ast-grep scan && cd ../..
```

Expected: both pass. The ast-grep rule should now report zero violations from
`convert_to_dotted_properties.rs` (the deref-write at line 26 is now followed by
`notice_change`; the field-write at line 44 went through `replace_expression`).

### Task 1.2: Refactor `substitute_single_use_symbol_*` family to `&mut TraverseCtx`

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/minimize_statements.rs`

Three functions form a recursive group that all take `&TraverseCtx<'a>` today:

- `substitute_single_use_symbol_in_statement` (line 1231)
- `substitute_single_use_symbol_in_expression_from_declarators` (line 1305)
- `substitute_single_use_symbol_in_expression` (line 1367)

There are 10 caller sites at lines 420, 494, 640, 702, 855, 914, 952, 973, 1052, 1140
(`grep -n "substitute_single_use_symbol_in_statement" crates/oxc_minifier/src/peephole/minimize_statements.rs`).
All callers ALREADY have `&mut TraverseCtx` in scope.

- [ ] **Step 1: Change all 3 function signatures**

For each of the 3 functions, change `ctx: &TraverseCtx<'a>` → `ctx: &mut TraverseCtx<'a>`.

- [ ] **Step 2: Update each function's body to use `replace_expression` at line 1387**

At line 1374-1391 (current state with allowlist comments), change:

```rust
match target_expr {
    Expression::Identifier(id) => {
        if id.name == search_for {
            let target_span = target_expr.span();
            *target_expr = replacement.take_in(ctx.ast);
            *target_expr.span_mut() = target_span;
            return Some(true);
        }
        // ... rest unchanged
    }
    // ... rest unchanged
}
```

Into:

```rust
match target_expr {
    Expression::Identifier(id) => {
        if id.name == search_for {
            let target_span = target_expr.span();
            let mut new_expr = replacement.take_in(ctx.ast);
            *new_expr.span_mut() = target_span;
            ctx.replace_expression(target_expr, new_expr);
            return Some(true);
        }
        // ... rest unchanged
    }
    // ... rest unchanged
}
```

Apply the span FIRST to the new_expr (not after the replace), so the `*target_expr.span_mut()` write becomes `*new_expr.span_mut()` on the still-owned value — eliminating the second allowlisted deref-write.

- [ ] **Step 3: Update recursive calls inside the 3 functions**

Inside the 3 functions, any recursive call passes `ctx`. With `&mut ctx`, these calls
now reborrow automatically. Build and let the borrow checker tell you which call sites
need an explicit `&mut *ctx` reborrow.

- [ ] **Step 4: Update 10 caller sites**

For each call to `substitute_single_use_symbol_in_statement` at lines 420, 494, 640,
702, 855, 914, 952, 973, 1052, 1140, **remove the now-redundant** `if changed { ctx.notice_change(); }` pattern. The helper at line 1387 now bumps `state.changed` directly via `replace_expression`, so the caller's bool propagation is no longer the change signal.

But wait — the bool return value is _also_ used to control caller control flow
(short-circuiting subsequent substitutions). So the return value stays; only the
`if changed { ctx.notice_change(); }` line is removed.

Concretely at line 420-428 (current):

```rust
let changed = Self::substitute_single_use_symbol_in_statement(
    first_decl_init,
    result,
    ctx,
    false,
);
if changed {
    ctx.notice_change();
}
```

Becomes:

```rust
// changed return value used purely for control flow; helper already bumped the counter
let _ = Self::substitute_single_use_symbol_in_statement(
    first_decl_init,
    result,
    ctx,
    false,
);
```

If `changed` IS actually used for control flow at any of the 10 sites (read the
surrounding code at each site), keep the binding and remove only the `notice_change`
line.

- [ ] **Step 5: Build**

```bash
cargo build -p oxc_minifier
```

Expected: clean build. If borrow-checker fights you persistently, the alternative
fallback is to keep the `&TraverseCtx<'a>` signature and instead require callers to
walk the dropped subtree before calling the function. See "If Task 1.2 escalation"
below.

- [ ] **Step 6: Run tests + minsize**

```bash
cargo test -p oxc_minifier
just minsize
git diff --stat tasks/minsize/
```

Expected: tests PASS. `just minsize` may show additional non-zero diff — this is the
`minimize_statements.rs:1387` fix surfacing. Capture as `/tmp/minsize-1.2.diff`.

- [ ] **Step 7: Run CI gates**

```bash
./tools/check_state_changed.sh && cd crates/oxc_minifier && ast-grep scan && cd ../..
```

Expected: both pass. The ast-grep rule should now report zero violations from
`minimize_statements.rs:1387` (was previously allowlisted).

#### If Task 1.2 escalation needed

If the borrow checker cascade is intractable, FALL BACK to option (b) from spec §6.2:
keep `&TraverseCtx<'a>` immutable, but require callers to call
`ctx.drop_expression(target_expr)` BEFORE calling `substitute_single_use_symbol_in_statement`
when they have evidence the call may rewrite. This is more invasive at call sites but
keeps the helper inversion. Document the choice in the commit body.

### Task 1.3: Commit

- [ ] **Step 1: Verify combined diff**

```bash
git status --short
git diff --stat
```

Expected: only the files mentioned in Tasks 1.1 + 1.2.

- [ ] **Step 2: Run final tests**

```bash
cargo test -p oxc_minifier
cargo test -p oxc_mangler
cargo coverage -- minifier
just minsize
```

Expected: tests pass, coverage no regression, `just minsize` produces the documented
deltas.

- [ ] **Step 3: Commit**

```bash
git add crates/oxc_minifier/src/peephole/convert_to_dotted_properties.rs \
        crates/oxc_minifier/src/peephole/minimize_statements.rs
git commit -m "fix(minifier): close two latent silent-mutation bypasses" -m "$(cat <<'EOF'
The prior helper-migration PR deferred two silent-bypass sites because
fixing them would change minified output. The follow-up incremental
scoping refresh design (docs/superpowers/specs/2026-05-26-...)
requires every reference-dropping mutation to go through a typed
helper, so the deferral is no longer tenable.

Two refactors:

  - \`convert_to_dotted_properties\` (convert_to_dotted_properties.rs)
    now takes \`&mut TraverseCtx\`. The field-write at line 40 goes
    through \`replace_expression\`; the enum-slot write at line 26 stays
    as \`*expr = …\` (no typed helper for the \`MemberExpression\` enum)
    but is now followed by \`notice_change()\`.

  - \`substitute_single_use_symbol_*\` family (minimize_statements.rs)
    refactored to take \`&mut TraverseCtx\`. The slot write at line 1387
    goes through \`replace_expression\`. Callers' redundant
    \`if changed { ctx.notice_change(); }\` removed (the helper bumps
    the counter directly).

\`just minsize\` deltas in this commit are EXPECTED — they reflect
correct optimizations that the missed signals were silently
suppressing. See /tmp/minsize-1.1.diff and /tmp/minsize-1.2.diff
for the per-snapshot diffs.
EOF
)"
```

(Inline the captured `git diff tasks/minsize/` excerpts directly in the commit body
where the `EOF`-quoted heredoc says "see /tmp/...".)

---

## Commit 2: Add `drop_*` helpers + migrate Pattern C/D drop sites

**Goal:** Add `drop_expression` and `drop_statement` helpers. Migrate every site that
silently drops references via Option-clear, `retain_mut`-drop, `pop`/`drain`/`truncate`/
`splice`, or Class field `take()`. After this commit, every reference-dropping mutation
goes through a typed helper. `just minsize` MUST be zero deltas (live-program walk in
`exit_program` was authoritative; this is pure refactor).

### Task 2.1: Add `drop_expression` and `drop_statement` helpers

**Files:**

- Modify: `crates/oxc_minifier/src/traverse_context/ecma_context.rs`

- [ ] **Step 1: Add the two helpers**

In the existing `impl<'a> TraverseCtx<'a, MinifierState<'a>>` block, ADD after the
existing helpers:

```rust
    /// Mark an expression subtree as about to be dropped (popped from a collection,
    /// taken out of an Option, etc.). For now, only bumps `state.changed`; in a
    /// later commit this becomes the entry point for `DropDiff` walks that feed
    /// per-pass dead-ref tracking.
    #[inline]
    pub fn drop_expression(&mut self, _expr: &Expression<'a>) {
        self.state.changed = true;
    }

    /// Mark a statement subtree as about to be dropped. Same contract as
    /// `drop_expression`.
    #[inline]
    pub fn drop_statement(&mut self, _stmt: &Statement<'a>) {
        self.state.changed = true;
    }
```

Note: the `_expr` / `_stmt` parameter is unused for now; commit 4 will populate it.
The reason for accepting it now is so the call sites in commit 2 land in their final
shape — commit 4 only needs to change the helper bodies, not the call sites.

- [ ] **Step 2: Build**

```bash
cargo build -p oxc_minifier
```

Expected: clean (helpers compile; no callers yet).

### Task 2.2: Find all Pattern C/D drop sites

- [ ] **Step 1: Audit `notice_change()` sites for dropping shapes**

```bash
grep -n "ctx.notice_change()" crates/oxc_minifier/src/peephole/*.rs > /tmp/notice_change_sites.txt
wc -l /tmp/notice_change_sites.txt
```

Expected: ~30-40 lines. (Each line is one site; total count varies.)

- [ ] **Step 2: Classify each site**

For each site, read 5-10 lines of surrounding context and classify into one of:

| Category                             | Pattern                                                                       | Migration                                                       |
| ------------------------------------ | ----------------------------------------------------------------------------- | --------------------------------------------------------------- |
| Operand swap                         | `e.right = left; e.left = right;`                                             | KEEP `notice_change()` — no drop                                |
| Operator/bool/span flip              | `e.operator = …`, `*computed = false`, `*span_mut() = …`                      | KEEP — no drop                                                  |
| Option-clear                         | `field = None`, `field.take()` on `Option<Expression>`/`Option<Statement>`    | MIGRATE → `drop_expression`/`drop_statement` before the drop    |
| Collection retain-mut                | `vec.retain_mut(\|e\| !remove_unused_expression(e, ctx))` + length check      | MIGRATE → `drop_expression` inside the predicate                |
| Collection pop/drain/truncate/splice | `vec.pop()`, `vec.drain(…)`, etc. that returns a dropped Expression/Statement | MIGRATE → `drop_*` on the returned value before letting it fall |
| Class field take                     | `super_class.take()`, `def.value.take()`                                      | MIGRATE → `drop_expression` before the take                     |

Save the classification to `/tmp/c2-classification.md` for the commit body.

- [ ] **Step 3: Verify the audit is exhaustive**

```bash
# Should match the count from step 1
wc -l /tmp/c2-classification.md
```

### Task 2.3: Migrate Pattern C/D drop sites

For each MIGRATE-class site, apply the corresponding pattern from spec §6.7. The
specific transformations:

**Pattern: Option-clear**

```rust
- field = None;
- ctx.notice_change();
+ if let Some(old) = field.take() {
+     ctx.drop_expression(&old);  // or drop_statement
+ }
```

**Pattern: retain_mut with drop-predicate**

```rust
  let old_len = vec.len();
- vec.retain_mut(|e| !remove_unused_expression(e, ctx));
- if vec.len() != old_len { ctx.notice_change(); }
+ vec.retain_mut(|e| {
+     if remove_unused_expression(e, ctx) {
+         ctx.drop_expression(e);
+         false
+     } else {
+         true
+     }
+ });
```

**Pattern: pop/drain/truncate/splice of Statement**

```rust
- let dropped = stmts.pop().unwrap();
- ctx.notice_change();
+ let dropped = stmts.pop().unwrap();
+ ctx.drop_statement(&dropped);
```

**Pattern: Class field take()**

```rust
- e.super_class.take();
- ctx.notice_change();
+ if let Some(old) = e.super_class.take() {
+     ctx.drop_expression(&old);
+ }
```

- [ ] **Step 1: Apply patterns to all MIGRATE-class sites**

Work file-by-file. For each file, after applying:

```bash
cargo build -p oxc_minifier
```

Should compile cleanly. Fix any borrow-checker friction by extracting locals (the
pattern from PR 6/PR 10/etc. migrations).

- [ ] **Step 2: Verify no MIGRATE sites missed**

Re-run the classification audit and confirm zero MIGRATE-class sites remain on
`notice_change()`:

```bash
grep -n "ctx.notice_change()" crates/oxc_minifier/src/peephole/*.rs
```

For every remaining `notice_change()`, the surrounding code must clearly show NO
subtree drop (operand swap / operator flip / bool/span field / etc.).

### Task 2.4: Verify and commit

- [ ] **Step 1: Full test suite**

```bash
cargo test -p oxc_minifier
cargo test -p oxc_mangler
```

Expected: PASS unchanged.

- [ ] **Step 2: minsize MUST be zero diff**

```bash
just minsize
git diff --stat tasks/minsize/
```

Expected: EMPTY. If any delta, the migration changed semantics — investigate before
committing. Likely cause: a Pattern C/D site that needed Pattern A/B treatment
instead. Re-classify.

- [ ] **Step 3: CI gates**

```bash
./tools/check_state_changed.sh && cd crates/oxc_minifier && ast-grep scan && cd ../..
```

Expected: both pass.

- [ ] **Step 4: Commit**

```bash
git add crates/oxc_minifier/src/traverse_context/ecma_context.rs \
        crates/oxc_minifier/src/peephole/*.rs
git commit -m "refactor(minifier): add drop_* helpers + migrate Pattern C/D drop sites" -m "$(cat <<'EOF'
Adds \`drop_expression\` and \`drop_statement\` helpers — for now they
just bump \`state.changed\`, matching every other walking helper. Their
purpose is to give callers an explicit way to mark an AST subtree as
about-to-be-dropped, so a future commit can teach them to walk for
dead references at the same time.

Migrates every Pattern C/D site that previously dropped a subtree via
\`field = None\`, \`retain_mut\` predicate, \`pop()\`/\`drain()\`/\`truncate()\`/
\`splice()\`, or Class field \`take()\` to call the new helpers BEFORE
the drop. Remaining \`notice_change()\` sites are all operand swaps,
operator-only flips, or bool/span field tweaks — no AST subtree drops.

\`just minsize\` is zero-delta (live-program walk in exit_program is
authoritative on output today). See /tmp/c2-classification.md for the
per-site classification audit.
EOF
)"
```

---

## Commit 3: Counter + remove `reset_changed`

**Goal:** Add `MinifierState::mutations: u64`, run it in parallel with `changed: bool`
(all helpers bump both), switch loop driver to snapshot-compare, remove
`reset_changed()`. Pure refactor — `just minsize` MUST be zero deltas.

### Task 3.1: Add `mutations: u64` field

**Files:**

- Modify: `crates/oxc_minifier/src/state.rs`

- [ ] **Step 1: Add the field**

In `crates/oxc_minifier/src/state.rs`, add inside `MinifierState<'a>`:

```rust
    /// Monotonic mutation counter. Bumped by every helper call.
    /// Together with `changed: bool` during this transition commit;
    /// `changed` is removed in commit 5.
    pub(crate) mutations: u64,
```

Initialize to `0` in `MinifierState::new`.

- [ ] **Step 2: Update all helpers to bump both**

In `crates/oxc_minifier/src/traverse_context/ecma_context.rs`, in each of the 6
walking helpers (`replace_expression`, `replace_statement`,
`replace_assignment_target_property`, `replace_property_key`, `drop_expression`,
`drop_statement`) and `notice_change`, add `self.state.mutations += 1;` after the
existing `self.state.changed = true;`.

Example for `replace_expression`:

```rust
    #[inline]
    pub fn replace_expression(&mut self, slot: &mut Expression<'a>, new: Expression<'a>) {
        *slot = new;
        self.state.changed = true;
        self.state.mutations += 1;
    }
```

- [ ] **Step 3: Build + test**

```bash
cargo build -p oxc_minifier && cargo test -p oxc_minifier
```

Expected: clean, all 504+ tests pass.

### Task 3.2: Switch loop driver to snapshot-compare

**Files:**

- Modify: `crates/oxc_minifier/src/compressor.rs`

- [ ] **Step 1: Read current `run_in_loop`**

```bash
grep -n -A20 "fn run_in_loop" crates/oxc_minifier/src/compressor.rs
```

- [ ] **Step 2: Rewrite to use snapshot-compare**

Current shape (uses `ctx.state.changed`):

```rust
loop {
    PeepholeOptimizations.run_once(program, ctx);
    if !ctx.state().changed { break; }
    // …max-iteration guard…
}
```

Becomes:

```rust
loop {
    let snapshot = ctx.state().mutations;
    PeepholeOptimizations.run_once(program, ctx);
    if ctx.state().mutations == snapshot { break; }
    // …max-iteration guard unchanged…
}
```

- [ ] **Step 3: Build + test**

```bash
cargo build -p oxc_minifier && cargo test -p oxc_minifier
```

Expected: clean, all tests pass.

### Task 3.3: Remove `reset_changed` and the `enter_program` reset

**Files:**

- Modify: `crates/oxc_minifier/src/traverse_context/ecma_context.rs`
- Modify: `crates/oxc_minifier/src/peephole/mod.rs:165`

- [ ] **Step 1: Remove the `enter_program` call**

In `crates/oxc_minifier/src/peephole/mod.rs:165`, remove:

```rust
- ctx.reset_changed();
```

(`mutations` is monotonic; no reset needed. `changed` is reset to `false` in
`exit_program` as part of the existing flow — actually verify this with `grep "changed
= false" crates/oxc_minifier/src/`. If `exit_program` doesn't reset it, KEEP a manual
`ctx.state.changed = false;` reset inside `enter_program` temporarily — the bool is
removed entirely in commit 5.)

Actually re-reading the original code: `enter_program` set `changed = false` via the
old direct write, then commit 1 (Task 1) of the prior migration introduced
`reset_changed()`. Now we remove `reset_changed()` AND the `changed = false` reset
itself — the bool is kept only because `LiveUsageCollector` reads it. Verify what
reads `changed`:

```bash
grep -n "state.changed" crates/oxc_minifier/src/peephole/mod.rs
```

If the only read is the `if ctx.state.changed` guard around `LiveUsageCollector`,
keep ONE manual `ctx.state.changed = false;` inside `enter_program` for the duration
of commits 3 and 4. Remove it in commit 5 alongside the bool.

- [ ] **Step 2: Remove the `reset_changed` helper**

In `crates/oxc_minifier/src/traverse_context/ecma_context.rs`, delete the
`reset_changed` method body:

```rust
-    #[inline]
-    pub fn reset_changed(&mut self) {
-        self.state.changed = false;
-    }
```

- [ ] **Step 3: Build + test**

```bash
cargo build -p oxc_minifier && cargo test -p oxc_minifier
```

Expected: clean. If anything else called `reset_changed()`, the build fails — fix the
caller.

### Task 3.4: Verify and commit

- [ ] **Step 1: Full test suite + minsize**

```bash
cargo test -p oxc_minifier
just minsize
git diff --stat tasks/minsize/
```

Expected: tests pass, minsize EMPTY diff.

- [ ] **Step 2: CI gates**

```bash
./tools/check_state_changed.sh && cd crates/oxc_minifier && ast-grep scan && cd ../..
```

Expected: pass.

- [ ] **Step 3: Commit**

```bash
git add crates/oxc_minifier/src/state.rs \
        crates/oxc_minifier/src/traverse_context/ecma_context.rs \
        crates/oxc_minifier/src/compressor.rs \
        crates/oxc_minifier/src/peephole/mod.rs
git commit -m "refactor(minifier): replace state.changed bool with mutations counter" -m "Adds \`MinifierState::mutations: u64\` alongside the existing \`changed:
bool\`. All helpers bump both — \`changed\` stays so \`LiveUsageCollector\`
keeps working (deleted in commit 5). The fixed-point loop now drives
off snapshot-compare on \`mutations\` instead of \`changed\`, and
\`reset_changed()\` is removed (the counter is monotonic across the
session, no reset needed).

Pure refactor: \`just minsize\` is zero-delta."
```

---

## Commit 4: DropDiff infrastructure (no-op observable)

**Goal:** Add `PassDirty` struct, `DropDiff` collector, the walk-and-diff methods on
each helper. Do NOT yet consume in `exit_program` — `LiveUsageCollector` keeps running
authoritatively. This commit is a no-op observable; it builds the data without
consuming it. `just minsize` MUST be zero deltas.

### Task 4.1: Add `PassDirty` struct to `MinifierState`

**Files:**

- Modify: `crates/oxc_minifier/src/state.rs`

- [ ] **Step 1: Add the struct and field**

In `crates/oxc_minifier/src/state.rs`, ADD:

```rust
use oxc_syntax::{reference::ReferenceId, symbol::SymbolId};
use oxc_str::Atom;
use rustc_hash::FxHashSet;

/// Per-pass dirty data accumulated by walking-helper calls. Consumed by
/// `exit_program` (in commit 5) and reset there.
pub struct PassDirty<'a> {
    /// `ReferenceId`s whose AST node has been removed and not re-installed
    /// in any later mutation this pass.
    pub(crate) dead_refs: FxHashSet<ReferenceId>,

    /// Names of unresolved references whose last AST occurrence has been
    /// removed. Pruning `Scoping::root_unresolved_references` is name-keyed
    /// (and a name can have many references); confirming the prune is safe
    /// requires a small walk in `exit_program`.
    pub(crate) dead_unresolved: FxHashSet<Atom<'a>>,

    /// At least one direct `eval(...)` call was dropped this pass. Gates
    /// the small `LiveDirectEvalCollector` walk at `exit_program`.
    pub(crate) eval_dropped: bool,
}

impl<'a> PassDirty<'a> {
    pub fn new() -> Self {
        Self {
            dead_refs: FxHashSet::default(),
            dead_unresolved: FxHashSet::default(),
            eval_dropped: false,
        }
    }

    pub fn reset(&mut self) {
        self.dead_refs.clear();
        self.dead_unresolved.clear();
        self.eval_dropped = false;
    }

    pub fn is_empty(&self) -> bool {
        self.dead_refs.is_empty() && self.dead_unresolved.is_empty() && !self.eval_dropped
    }
}

impl<'a> Default for PassDirty<'a> {
    fn default() -> Self {
        Self::new()
    }
}
```

Add the field to `MinifierState`:

```rust
    pub(crate) dirty: PassDirty<'a>,
```

Initialize in `MinifierState::new`: `dirty: PassDirty::new(),`.

- [ ] **Step 2: Build**

```bash
cargo build -p oxc_minifier
```

Expected: clean.

### Task 4.2: Add `DropDiff` collector

**Files:**

- Create: `crates/oxc_minifier/src/traverse_context/drop_diff.rs`
- Modify: `crates/oxc_minifier/src/traverse_context/mod.rs` (add `mod drop_diff;`)

- [ ] **Step 1: Create the file**

`crates/oxc_minifier/src/traverse_context/drop_diff.rs`:

```rust
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_semantic::Scoping;
use oxc_str::Atom;

use crate::state::PassDirty;

/// Walks AST subtrees collecting `IdentifierReference`s and direct `eval(...)`
/// calls, updating the per-pass `PassDirty` accumulator.
///
/// Two distinct walk modes:
///
/// - `walk_old_*` — invoked on a subtree being dropped or replaced. Every
///   reference found is ADDED to `dirty.dead_refs` (or `dead_unresolved`).
///   Every direct eval call sets `dirty.eval_dropped = true`.
///
/// - `resurrect_from_*` — invoked on the replacement value during a
///   `replace_*` helper call. Every reference found is REMOVED from
///   `dirty.dead_refs` (or `dead_unresolved`). Handles within-call and
///   cross-call ReferenceId preservation via `clone_in_with_semantic_ids`.
pub(crate) struct DropDiff<'a, 's> {
    dirty: &'s mut PassDirty<'a>,
    scoping: &'s Scoping,
    mode: DropDiffMode,
}

#[derive(Clone, Copy)]
enum DropDiffMode {
    /// Add visited refs to the dirty set.
    MarkDead,
    /// Remove visited refs from the dirty set.
    Resurrect,
}

impl<'a, 's> DropDiff<'a, 's> {
    pub(crate) fn new(dirty: &'s mut PassDirty<'a>, scoping: &'s Scoping) -> Self {
        Self { dirty, scoping, mode: DropDiffMode::MarkDead }
    }

    pub(crate) fn walk_old_expression(mut self, expr: &Expression<'a>) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_expression(expr);
        self
    }

    pub(crate) fn walk_old_statement(mut self, stmt: &Statement<'a>) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_statement(stmt);
        self
    }

    pub(crate) fn walk_old_assignment_target_property(
        mut self,
        prop: &AssignmentTargetProperty<'a>,
    ) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_assignment_target_property(prop);
        self
    }

    pub(crate) fn walk_old_property_key(mut self, key: &PropertyKey<'a>) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_property_key(key);
        self
    }

    pub(crate) fn resurrect_from_expression(mut self, expr: &Expression<'a>) -> Self {
        self.mode = DropDiffMode::Resurrect;
        self.visit_expression(expr);
        self
    }

    pub(crate) fn resurrect_from_statement(mut self, stmt: &Statement<'a>) -> Self {
        self.mode = DropDiffMode::Resurrect;
        self.visit_statement(stmt);
        self
    }

    pub(crate) fn resurrect_from_assignment_target_property(
        mut self,
        prop: &AssignmentTargetProperty<'a>,
    ) -> Self {
        self.mode = DropDiffMode::Resurrect;
        self.visit_assignment_target_property(prop);
        self
    }

    pub(crate) fn resurrect_from_property_key(mut self, key: &PropertyKey<'a>) -> Self {
        self.mode = DropDiffMode::Resurrect;
        self.visit_property_key(key);
        self
    }
}

impl<'a> Visit<'a> for DropDiff<'a, '_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        let reference_id = it.reference_id();
        let resolved = self.scoping.get_reference(reference_id).symbol_id().is_some();

        match (self.mode, resolved) {
            (DropDiffMode::MarkDead, true) => {
                self.dirty.dead_refs.insert(reference_id);
            }
            (DropDiffMode::MarkDead, false) => {
                self.dirty.dead_unresolved.insert(it.name);
            }
            (DropDiffMode::Resurrect, true) => {
                self.dirty.dead_refs.remove(&reference_id);
            }
            (DropDiffMode::Resurrect, false) => {
                // Don't aggressively remove unresolved names — a name with
                // many references shouldn't be removed from `dead_unresolved`
                // just because one occurrence survives in `new`. The exit_program
                // prune walk handles this correctly via the per-name confirmation.
                //
                // (See spec §5 prune_unresolved_refs.)
            }
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if matches!(self.mode, DropDiffMode::MarkDead)
            && !it.optional
            && let Some(ident) = it.callee.get_identifier_reference()
            && ident.name == "eval"
        {
            self.dirty.eval_dropped = true;
        }
        // Recurse — eval may be nested inside another call's arguments.
        walk_call_expression(self, it);
    }
}
```

- [ ] **Step 2: Wire into the module**

In `crates/oxc_minifier/src/traverse_context/mod.rs`, ADD near the other `mod`
declarations:

```rust
mod drop_diff;
pub(crate) use drop_diff::DropDiff;
```

- [ ] **Step 3: Build**

```bash
cargo build -p oxc_minifier
```

Expected: clean — the file compiles but no callers yet.

### Task 4.3: Wire `DropDiff` into the helpers

**Files:**

- Modify: `crates/oxc_minifier/src/traverse_context/ecma_context.rs`

- [ ] **Step 1: Add a private `dirty_diff` accessor**

In the existing `impl<'a> TraverseCtx<'a, MinifierState<'a>>` block, ADD:

```rust
    /// Construct a `DropDiff` borrowing the per-pass dirty accumulator and
    /// the current scoping snapshot. Used by walking helpers.
    #[inline]
    fn dirty_diff(&mut self) -> DropDiff<'a, '_> {
        // Field-level borrowing: state and scoping are disjoint fields of self.
        DropDiff::new(&mut self.state.dirty, self.scoping.scoping())
    }
```

(Verify that `self.scoping.scoping()` returns `&Scoping` — check `TraverseScoping` API.
If the method is named differently, use the actual name.)

- [ ] **Step 2: Update each walking helper to call `dirty_diff()`**

For each of the 6 walking helpers, INSERT the walk-and-diff call. Example for
`replace_expression`:

```rust
    #[inline]
    pub fn replace_expression(&mut self, slot: &mut Expression<'a>, new: Expression<'a>) {
        self.dirty_diff().walk_old_expression(slot).resurrect_from_expression(&new);
        *slot = new;
        self.state.changed = true;
        self.state.mutations += 1;
    }
```

For `drop_expression` / `drop_statement` (no `new`):

```rust
    #[inline]
    pub fn drop_expression(&mut self, expr: &Expression<'a>) {
        self.dirty_diff().walk_old_expression(expr);
        self.state.changed = true;
        self.state.mutations += 1;
    }
```

Apply the same shape to all 4 `replace_*` and both `drop_*`. `notice_change` stays
unchanged (no walk).

- [ ] **Step 3: Build**

```bash
cargo build -p oxc_minifier
```

Expected: clean. If borrow-checker complains about simultaneous borrows of
`self.state` and `self.scoping`, use a destructure pattern:

```rust
        let TraverseCtx { state, scoping, .. } = self;
        DropDiff::new(&mut state.dirty, scoping.scoping())
            .walk_old_expression(slot)
            .resurrect_from_expression(&new);
        *slot = new;
        state.changed = true;
        state.mutations += 1;
```

### Task 4.4: Reset `PassDirty` in `enter_program`

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/mod.rs`

- [ ] **Step 1: Add the reset call**

In `enter_program` (near where the old `ctx.reset_changed()` was, before it was
removed in commit 3):

```rust
    fn enter_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        ctx.state.symbol_values.reset();
        ctx.state.proto_write_symbols.clear();
        ctx.state.dirty.reset();
        ctx.state.changed = false;  // still needed; removed in commit 5
    }
```

### Task 4.5: Verify and commit

- [ ] **Step 1: Full test suite + minsize**

```bash
cargo test -p oxc_minifier
just minsize
git diff --stat tasks/minsize/
```

Expected: tests pass, minsize EMPTY diff. (This commit is no-op observable —
`LiveUsageCollector` still authoritative; `PassDirty` data is computed but unused.)

- [ ] **Step 2: Verify `PassDirty` is actually being populated**

Add a debug print temporarily and rerun a test:

```rust
// In exit_program, before the LiveUsageCollector walk:
eprintln!("[debug] dirty.dead_refs.len() = {}", ctx.state.dirty.dead_refs.len());
```

Run any minifier test with a non-trivial mutation. Confirm non-zero counts in stderr.
REMOVE the debug print before committing.

- [ ] **Step 3: CI gates**

```bash
./tools/check_state_changed.sh && cd crates/oxc_minifier && ast-grep scan && cd ../..
```

Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add crates/oxc_minifier/src/state.rs \
        crates/oxc_minifier/src/traverse_context/ \
        crates/oxc_minifier/src/peephole/mod.rs
git commit -m "refactor(minifier): add DropDiff infrastructure (no-op observable)" -m "Adds \`PassDirty\` to \`MinifierState\` and a private \`DropDiff\`
collector at \`traverse_context/drop_diff.rs\`. Each walking helper
(\`replace_*\`, \`drop_*\`) now invokes \`DropDiff::walk_old_*\` on the
dropped/replaced subtree; \`replace_*\` additionally invoke
\`resurrect_from_*\` on the new value.

\`LiveUsageCollector\` still runs in \`exit_program\` and remains
authoritative — this commit builds the dirty data without consuming
it. The switchover happens in commit 5.

Pure refactor: \`just minsize\` is zero-delta."
```

---

## Commit 5: Switch `exit_program` consumer + delete `LiveUsageCollector`

**Goal:** Add `Scoping::retain_resolved_references_excluding`. Rewrite `exit_program`
to consume `dirty.*` directly. Delete `LiveUsageCollector` and `MinifierState::changed`.
This is the load-bearing commit. `just minsize` MUST be zero deltas.

### Task 5.1: Add `Scoping::retain_resolved_references_excluding` to oxc_semantic

**Files:**

- Modify: `crates/oxc_semantic/src/scoping.rs`

- [ ] **Step 1: Find the existing `retain_resolved_references`**

```bash
grep -n "fn retain_resolved_references\|fn delete_resolved_reference" crates/oxc_semantic/src/scoping.rs
```

- [ ] **Step 2: Add the new method**

Near the existing `retain_resolved_references`, add:

```rust
    /// Remove every `ReferenceId` in `excluded` from each symbol's
    /// resolved-references list. O(total_references) in the worst case;
    /// short-circuits when the per-symbol list is empty or has no
    /// excluded entries.
    pub fn retain_resolved_references_excluding(
        &mut self,
        excluded: &FxHashSet<ReferenceId>,
    ) {
        if excluded.is_empty() {
            return;
        }
        for refs in self.resolved_references.iter_mut() {
            refs.retain(|r| !excluded.contains(r));
        }
    }
```

(The exact field name `resolved_references` may differ — adapt to the actual `Scoping`
struct's field names.)

- [ ] **Step 3: Build + test the semantic crate**

```bash
cargo build -p oxc_semantic && cargo test -p oxc_semantic
```

Expected: clean, no test regressions.

### Task 5.2: Add `Scoping::remove_unresolved_reference` if not present

- [ ] **Step 1: Check if it exists**

```bash
grep -n "fn remove_unresolved_reference\|fn delete_unresolved" crates/oxc_semantic/src/scoping.rs
```

If present, skip to Task 5.3. Otherwise:

- [ ] **Step 2: Add the method**

Near the unresolved-references accessors:

```rust
    /// Remove all references for `name` from the root-unresolved set.
    pub fn remove_unresolved_reference(&mut self, name: &str) {
        self.root_unresolved_references.remove(name);
    }
```

(Adapt to the actual field name.)

### Task 5.3: Rewrite `exit_program` and create `LiveDirectEvalCollector`

**Files:**

- Modify: `crates/oxc_minifier/src/peephole/mod.rs`

- [ ] **Step 1: Add `LiveDirectEvalCollector` and `NamedRefCollector`**

Replace the existing `LiveUsageCollector` block in `peephole/mod.rs:568-603` with:

```rust
/// Walks the live program to find scopes containing direct `eval(...)` calls.
/// Used by `exit_program` only when at least one direct eval call was dropped
/// this pass (gated via `PassDirty::eval_dropped`).
struct LiveDirectEvalCollector<'s> {
    scoping: &'s Scoping,
    scopes: FxHashSet<ScopeId>,
}

impl<'s> LiveDirectEvalCollector<'s> {
    fn new(scoping: &'s Scoping) -> Self {
        Self { scoping, scopes: FxHashSet::default() }
    }
}

impl<'a> Visit<'a> for LiveDirectEvalCollector<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if !it.optional
            && let Some(ident) = it.callee.get_identifier_reference()
            && ident.name == "eval"
        {
            let scope_id = self.scoping.get_reference(ident.reference_id()).scope_id();
            self.scopes.insert(scope_id);
        }
        walk_call_expression(self, it);
    }
}

/// Walks the live program to find which names in `candidates` still appear
/// in any `IdentifierReference`. Used by `prune_unresolved_refs` to confirm
/// it's safe to remove a name from `root_unresolved_references`.
struct NamedRefCollector<'a, 's> {
    candidates: &'s FxHashSet<Atom<'a>>,
    survivors: &'s mut FxHashSet<Atom<'a>>,
}

impl<'a> Visit<'a> for NamedRefCollector<'a, '_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if self.candidates.contains(&it.name) {
            self.survivors.insert(it.name);
        }
    }
}
```

- [ ] **Step 2: Rewrite `exit_program`**

Replace the existing `exit_program` body with:

```rust
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let dirty = &ctx.state.dirty;

        // (1) Resolved references — direct consumption, no walk.
        if !dirty.dead_refs.is_empty() {
            ctx.scoping_mut().retain_resolved_references_excluding(&dirty.dead_refs);
        }

        // (2) Unresolved references — gated confirmation walk by name.
        if !dirty.dead_unresolved.is_empty() {
            Self::prune_unresolved_refs(ctx, program);
        }

        // (3) Direct-eval — gated full walk only when an eval was dropped.
        if dirty.eval_dropped {
            let scoping = ctx.scoping();
            let mut live = LiveDirectEvalCollector::new(scoping);
            live.visit_program(program);
            let scopes = live.scopes;
            Self::refresh_direct_eval_flags(ctx.scoping_mut(), &scopes);
        }

        // Reset for next pass.
        ctx.state.dirty.reset();

        debug_assert!(ctx.state.dce || ctx.state.class_symbols_stack.is_exhausted());
    }
```

- [ ] **Step 3: Add `prune_unresolved_refs` helper**

In the `impl<'a> PeepholeOptimizations` block near `refresh_direct_eval_flags`, ADD:

```rust
    fn prune_unresolved_refs(ctx: &mut TraverseCtx<'a>, program: &Program<'a>) {
        let candidates = &ctx.state.dirty.dead_unresolved;
        let mut survivors = FxHashSet::default();
        {
            let mut collector = NamedRefCollector { candidates, survivors: &mut survivors };
            collector.visit_program(program);
        }
        // Reborrow scoping after the immutable visit completes.
        let scoping = ctx.scoping_mut();
        for name in candidates.iter() {
            if !survivors.contains(name) {
                scoping.remove_unresolved_reference(name);
            }
        }
    }
```

(Borrow-checker note: `candidates` borrows immutably from `ctx`, the visit reads
`program`, and `remove_unresolved_reference` needs `&mut scoping`. The pattern above
isolates the immutable phase before reborrowing mutably. If the borrow checker
complains, materialize `candidates` into an owned `Vec<Atom<'a>>` before the mutable
phase.)

### Task 5.4: Delete `LiveUsageCollector` and `MinifierState::changed`

**Files:**

- Modify: `crates/oxc_minifier/src/state.rs`
- Modify: `crates/oxc_minifier/src/peephole/mod.rs`

- [ ] **Step 1: Delete `MinifierState::changed`**

In `crates/oxc_minifier/src/state.rs`, remove the `changed: bool` field. Update
`MinifierState::new` accordingly.

- [ ] **Step 2: Remove `self.state.changed = true` from all 7 helpers**

In `crates/oxc_minifier/src/traverse_context/ecma_context.rs`, remove the
`self.state.changed = true;` line from each of the 7 helpers (`replace_expression`,
`replace_statement`, `replace_assignment_target_property`, `replace_property_key`,
`drop_expression`, `drop_statement`, `notice_change`). The `self.state.mutations += 1;`
stays.

- [ ] **Step 3: Remove the manual reset in `enter_program`**

In `crates/oxc_minifier/src/peephole/mod.rs::enter_program`, remove:

```rust
- ctx.state.changed = false;
```

- [ ] **Step 4: Delete the old `LiveUsageCollector`**

Already replaced in Task 5.3 by `LiveDirectEvalCollector` and `NamedRefCollector`. Verify
no references remain:

```bash
grep -n "LiveUsageCollector" crates/oxc_minifier/
```

Expected: zero matches. If any remain, remove them.

- [ ] **Step 5: Build**

```bash
cargo build -p oxc_minifier
```

Expected: clean. Any error means a leftover read of `state.changed` somewhere — find
and remove.

### Task 5.5: Verify and commit

- [ ] **Step 1: Full test suite**

```bash
cargo test -p oxc_minifier
cargo test -p oxc_mangler
cargo coverage -- minifier
```

Expected: PASS unchanged.

- [ ] **Step 2: minsize MUST be zero diff**

```bash
just minsize
git diff --stat tasks/minsize/
```

Expected: EMPTY. If any delta:

- The `DropDiff` walks are missing some path. Investigate which optimization is now
  firing differently (or not firing).
- Common cause: a Pattern C/D site missed in commit 2 that's actually dropping
  references not currently tracked.
- Fall back to keeping `LiveUsageCollector` and the dirty refresh side-by-side for
  debug: compare `dirty.dead_refs` against `!live_set` from the collector and look
  for missing IDs.

- [ ] **Step 3: CI gates**

```bash
./tools/check_state_changed.sh && cd crates/oxc_minifier && ast-grep scan && cd ../..
```

Expected: BOTH pass. The grep gate becomes trivially satisfied (the field no longer
exists). The ast-grep rule still meaningful.

- [ ] **Step 4: Commit**

```bash
git add crates/oxc_minifier/src/state.rs \
        crates/oxc_minifier/src/traverse_context/ecma_context.rs \
        crates/oxc_minifier/src/peephole/mod.rs \
        crates/oxc_semantic/src/scoping.rs
git commit -m "refactor(minifier)!: incremental scoping refresh, delete LiveUsageCollector" -m "$(cat <<'EOF'
Switches \`exit_program\` from running \`LiveUsageCollector\` over the
whole program to consuming the per-pass \`PassDirty\` data accumulated
by the mutation helpers. Three consumers:

  - Resolved references: \`Scoping::retain_resolved_references_excluding\`
    (new method) prunes \`dirty.dead_refs\` directly.

  - Unresolved references: \`prune_unresolved_refs\` walks the program
    once per pass for the small set of \`dirty.dead_unresolved\` names,
    confirming each is gone before pruning from
    \`root_unresolved_references\`.

  - Direct eval: \`LiveDirectEvalCollector\` (renamed/shrunk from the
    old collector, eval-only) walks the program ONLY when
    \`dirty.eval_dropped\` is set. Same algorithm as before, smaller
    domain.

\`LiveUsageCollector\` deleted. \`MinifierState::changed\` removed.
Helpers no longer write the bool; \`mutations: u64\` is the sole
mutation signal.

BREAKING CHANGE: \`MinifierState::changed\` (\`pub(crate)\`) is removed
from the crate's internal API. \`Scoping::retain_resolved_references_excluding\`
is a new \`pub\` method on \`oxc_semantic\`'s \`Scoping\`.

\`just minsize\` is zero-delta — the dirty data tracks every reference
the old live walk would have found dead.
EOF
)"
```

---

## Final verification (after all 5 commits)

- [ ] **Whole-stack verification**

```bash
# 5 commits since the spec branch tip
git log --oneline spec/minifier-eliminate-changed-flag..HEAD

# Zero MinifierState::changed references
rg 'state\.changed|MinifierState::changed' crates/oxc_minifier/
# Expected: zero matches.

# Zero LiveUsageCollector references
rg 'LiveUsageCollector' crates/oxc_minifier/
# Expected: zero matches.

# Helpers all present and inline
rg -n 'pub fn (replace_expression|replace_statement|replace_assignment_target_property|replace_property_key|drop_expression|drop_statement|notice_change)' crates/oxc_minifier/src/traverse_context/ecma_context.rs
# Expected: 7 matches. No `reset_changed`.

# Full regression
cargo test -p oxc_minifier
cargo test -p oxc_mangler
cargo coverage -- minifier
just minsize && git diff --stat tasks/minsize/
just ready
```

All expected to pass.

---

## Notes for the executing engineer

- **Stack base.** The branch `spec/minifier-incremental-scoping` is already created
  off `spec/minifier-eliminate-changed-flag`. Verify with `git branch --show-current`
  before each commit.

- **`just minsize` is the load-bearing test.** Commits 2-5 must produce zero deltas.
  If any delta appears, do NOT commit — diagnose the missing tracking site first.

- **Borrow-checker fights.** The helpers borrow both `state` (mut) and `scoping`
  (immutable). Field-level borrow on `TraverseCtx` works (its fields are disjoint),
  but if you hit friction, the destructure pattern at Task 4.3 Step 3 is the escape.

- **Test #28 (the deferred latent bugs) is closed by Commit 1.** Mark it complete
  after Commit 1 lands.

- **PR description.** Pull material from the 5 commit bodies. Lead with: "Incremental
  scoping refresh + mutation counter. Replaces the post-pass `LiveUsageCollector`
  walk with per-helper dead-ref tracking. 1 latent-bug fix → 4 pure-refactor commits
  → 1 load-bearing switchover. See `docs/superpowers/specs/2026-05-26-...` for the
  full design."
