# Eliminate manual `ctx.state.changed = true` via mutation helpers

**Status:** Design — pending user review
**Date:** 2026-05-25
**Scope:** `crates/oxc_minifier/`

## 1. Problem

`MinifierState::changed: bool` (`state.rs:33`) is set by **187** manual
`ctx.state.changed = true` lines across 16 files in `crates/oxc_minifier/src/peephole/`,
plus one reset (`ctx.state.changed = false`) at `peephole/mod.rs:165` cleared from
`enter_program`.

The flag does three jobs:

1. **Loop signal** — `Compressor::run_in_loop` (`compressor.rs:70`) re-runs the full
   peephole traversal until the flag stays `false`, capped at 10 iterations.
2. **Refresh gate** — `PeepholeOptimizations::exit_program` (`peephole/mod.rs:168-187`)
   rebuilds stale `Scoping` data (live `ReferenceId` set, `ScopeFlags::DirectEval`) via
   `LiveUsageCollector` only when the flag is set.
3. **Implicit contract** — every rewrite that mutates the AST must self-report.

Job 3 is the failure mode. Forgetting a single `= true` silently disables fixed-point
convergence: the downstream pass that _would_ have fired never runs, the AST is left
under-optimized, and the test suite catches nothing because the output is still valid JS.
The active branch `fix/minifier-mark-changed-on-dead-stmt-drop` is a recent instance of
exactly this bug class.

This spec addresses **only the manual-flag failure mode**. It does NOT change the scoping
refresh, the loop driver semantics, or `exit_program`. See §6 for what's deferred and why.

## 2. Why a narrow scope

Earlier iterations of this spec attempted a larger redesign — making scoping refresh
_incremental_ by collecting dirty references at mutation time. Two rounds of Codex
adversarial review surfaced six real issues with the incremental design, including a
release-mode correctness bug (cross-call resurrection of `ReferenceId`s preserved by
clone-with-semantic-ids rewrites) and unresolved edge cases around mid-pass reference
creation.

The lesson: the manual-flag problem and the incremental-scoping problem are separable.
This spec solves the first cleanly. The second is filed as a follow-up (§6) that needs
its own design with measurement and a separate audit of every code path that creates
references.

## 3. API

Four helpers on `TraverseCtx<'a, MinifierState<'a>>` (existing impl block at
`traverse_context/ecma_context.rs:167`):

```rust
impl<'a> TraverseCtx<'a, MinifierState<'a>> {
    /// Replace an expression slot. Marks the pass as having mutated the AST.
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

    /// Mark that the pass mutated the AST in place (operand swap, in-place field flip,
    /// collection element removal, etc.) where no slot replacement happened. Prefer the
    /// `replace_*` helpers when the mutation IS a slot replacement.
    #[inline]
    pub fn notice_change(&mut self) {
        self.state.changed = true;
    }

    /// Clear the per-pass "changed" signal. Called once at the top of each peephole
    /// traversal in `enter_program`. This is the only sanctioned way to write `false`.
    #[inline]
    pub fn reset_changed(&mut self) {
        self.state.changed = false;
    }
}
```

That's the entire surface change. `MinifierState::changed` stays as today. No `PassDirty`,
no bitset, no overflow, no walk-and-collect. The existing
`peephole/mod.rs:165 ctx.state.changed = false` becomes `ctx.reset_changed()`.

### What this is and isn't

It IS:

- A typed API for "I mutated the AST." The natural-feeling method writes the bool.
- A reduction of the failure mode from "silent missed `= true`" to "must use the helper to
  replace a slot." Reviewing for adherence is easier than scanning for missing assignments.

It is NOT:

- A change to how `Scoping` data is refreshed.
- A change to the fixed-point loop algorithm.
- A change to the cost profile of any pass (the same `LiveUsageCollector` walk runs on
  the same `changed` signal).

## 4. How each call-site shape transforms

```rust
// (A) Slot replace — most common, ~80% of sites
- *expr = new;
- ctx.state.changed = true;
+ ctx.replace_expression(expr, new);

// (B) Statement slot replace
- *stmt = new;
- ctx.state.changed = true;
+ ctx.replace_statement(stmt, new);

// (C) Collection mutation — conditional, same shape, just uses the helper
  let old_len = elems.len();
  elems.retain_mut(|e| !Self::remove_unused_expression(e, ctx));
- if elems.len() != old_len { ctx.state.changed = true; }
+ if elems.len() != old_len { ctx.notice_change(); }

// (D) In-place tweak — operand swap
  e.right = left;
  e.left = right;
- ctx.state.changed = true;
+ ctx.notice_change();

// (E) Conditional semantic gate — `dead_drop_mutates_ast`
  if dead_drop_mutates_ast(&stmt) {
-     ctx.state.changed = true;
+     ctx.notice_change();
  }
```

Row (E) is the critical preservation case: `dead_drop_mutates_ast`
(`minimize_statements.rs:22`) returns `false` when dropping a `var` with no initializer
would produce a byte-identical AST after `KeepVar` re-emits it. The gate stays at the call
site; the helper is called only in the truthy branch. A mechanical search-and-replace must
preserve the surrounding `if`.

## 5. Migration

One PR per peephole module, in dependency order. PR 1 adds the helpers. Each subsequent PR
mechanically replaces every `ctx.state.changed = true` site in its module with the
corresponding helper call. Verification per PR:

- `cargo test -p oxc_minifier`
- `cargo coverage -- minifier`
- `just minsize` (expected: zero diff)
- `verify-minifier` skill
- `cargo test -p oxc_mangler`

PR description includes the `just minsize` diff (expected empty).

The migration is _additive_ throughout: `MinifierState::changed` stays put, helpers and
legacy writes both write to it. There is no two-bit migration invariant to manage, no
intermediate state where the loop signal could break.

PR order is **smallest first**, generated from
`grep -c "ctx.state.changed = true" crates/oxc_minifier/src/peephole/*.rs`
as of the date of this spec (re-run before starting to catch drift):

| #   | File                                                                | Sites |
| --- | ------------------------------------------------------------------- | ----- |
| 1   | (PR 1 also adds helpers + `reset_changed()` migration)              | —     |
| 2   | `inline.rs`                                                         | 1     |
| 3   | `remove_unused_private_members.rs`                                  | 1     |
| 4   | `minimize_for_statement.rs`                                         | 2     |
| 5   | `peephole/mod.rs` (visitor body itself)                             | 2     |
| 6   | `minimize_logical_expression.rs`                                    | 3     |
| 7   | `minimize_not_expression.rs`                                        | 3     |
| 8   | `minimize_if_statement.rs`                                          | 5     |
| 9   | `remove_unused_declaration.rs`                                      | 5     |
| 10  | `minimize_conditions.rs`                                            | 6     |
| 11  | `minimize_expression_in_boolean_context.rs`                         | 6     |
| 12  | `replace_known_methods.rs`                                          | 6     |
| 13  | `fold_constants.rs`                                                 | 14    |
| 14  | `remove_dead_code.rs`                                               | 19    |
| 15  | `remove_unused_expression.rs`                                       | 27    |
| 16  | `substitute_alternate_syntax.rs`                                    | 43    |
| 17  | `minimize_statements.rs` (special care for `dead_drop_mutates_ast`) | 44    |
| 18  | Final lockdown PR (§5.1)                                            | —     |

Files with 0 sites that get a free pass: `convert_to_dotted_properties.rs`,
`minimize_conditional_expression.rs`, `normalize.rs`.

Total: **187 sites across 16 files**. Each per-file PR should run `grep -c` on its target
file before and after to verify zero remaining direct writes. The final lockdown PR (§5.1)
re-runs `grep -rc "state.changed = true" crates/oxc_minifier/src/` across the whole crate
and asserts zero hits.

### 5.1 Final lockdown PR

After the last call-site migration:

1. Make `MinifierState::changed` `pub(crate)` (was `pub`).
2. Add a grep-based CI check (e.g. a `tools/check_state_changed.sh` invoked from
   `just ready`) with **exactly** this rule:

   > Any `state.changed =` write in `crates/oxc_minifier/` is forbidden EXCEPT inside
   > `traverse_context/ecma_context.rs`, where it must appear only inside the bodies of
   > `replace_expression`, `replace_statement`, `notice_change`, or `reset_changed`.

   Concretely:

   ```bash
   # CI check (fails if any unauthorized write is found)
   rg -n 'state\.changed\s*=' crates/oxc_minifier/ \
     --glob '!crates/oxc_minifier/src/traverse_context/ecma_context.rs' \
     && exit 1 || exit 0
   ```

3. Confirm zero direct writes outside the helpers via the CI check above.

This bans every direct write to `state.changed` — both the `= true` mutations AND the
`= false` reset. The reset migrates to `ctx.reset_changed()` (a new helper, see §3)
_before_ the lockdown is enabled, so the CI check passes from PR 1 onward.

The check covers the original missed-change failure mode permanently: any future rewrite
that wants to signal a mutation must use a helper.

## 6. Deferred: incremental scoping refresh

The bigger redesign (collect dirty refs at mutation time, eliminate the post-pass walk)
is **deferred**, not rejected. It needs:

1. **Measurement.** Profile `exit_program`'s LiveUsageCollector walk on a representative
   bundler workload. Confirm it's actually a meaningful cost before optimizing.
2. **Audit of all reference creation paths.** Every `create_bound_reference` /
   `create_unbound_reference` call in `crates/oxc_minifier/` needs to be categorized: does
   it produce a `ReferenceId` that can later be dropped? If so, what scoping data needs
   maintenance?
3. **A solution for cross-call resurrection** (Codex high-severity finding from review
   round 2). When rewrite A drops a subtree containing `R` and rewrite B later builds a
   new subtree that preserves `R` via `clone_in_with_semantic_ids`, `R` is _live_ despite
   appearing in A's dropped subtree. The incremental design has no clean way to know.
4. **Direct-eval refresh.** Either keep the AST walk (gated more cleverly than today) or
   build a maintained per-scope direct-eval counter. The latter requires a one-time
   initialization pass and incremental updates in every relevant visitor.
5. **Unresolved reference metadata.** `root_unresolved_references` is keyed by name;
   pruning requires the name, which is lost once the AST node is dropped.

Each of these is a real design problem on its own. Bundling them with the helper refactor
was over-reach. The helper refactor in this spec is a strict prerequisite for any future
incremental work (it makes mutation observable through a controlled API), so this is a
strict-progress step.

## 7. Other rejected / deferred alternatives

### Counter on `PeepholeOptimizations` instead of `MinifierState`

Conceptually cleaner — "did this pass mutate" is naturally pass state, not program state.
But every static `Self::method(args, ctx)` call site would need to become
`self.method(args, ctx)` to access `&mut self`. ~120 signature touches purely to relocate
the bit. Rejected as ceremony for no win.

### Return-type discipline (`Rewrite<T> = Unchanged | Replace(T)`)

Most rust-idiomatic on paper. But many `exit_*` arms chain 3-7 rewrites against the same
slot (see `peephole/mod.rs:338-347`: seven calls against one `BinaryExpression`).
Atomizing each into a Result-returning unit is a much larger refactor than this design,
for the same correctness win. Rejected as worst ROI.

### Kill the fixed-point loop entirely (fixed N iterations)

The most radical answer to "do we need state for changed at all." Cap passes at 2-3, drop
the flag entirely, accept some size regression on pathological cases. **Deferred pending
measurement** of the iteration-count distribution on Test262 + a bundler workload.

### Dirty-region traversal (skip clean subtrees in iterations 2..N)

A perf optimization, not a correctness one. Defer until profiling shows iterations 2..N
spend their time in visitor dispatch rather than inside rewrite bodies.

## 8. Risks and mitigations

### 8.1 `dead_drop_mutates_ast` must keep gating

**Risk:** A naive migration converts `if dead_drop_mutates_ast(&stmt) { state.changed = true; }`
into an unconditional `ctx.notice_change()`, counting an identity-drop as a real change
and looping forever.

**Mitigation:** The `if` stays. The mechanical sweep transforms only the body of the `if`,
not the `if` itself. Reviewers must verify the gate is preserved. Existing tests in
`minimize_statements.rs` exercise the no-op path and would fail (via the
`Ran loop more than 10 times` assertion in `compressor.rs:86`) if the gate were removed.

### 8.2 Helper inlining

**Risk:** A function-call wrapping `*slot = new; bool = true` could regress hot-path perf
if not inlined.

**Mitigation:** `#[inline]` on each helper. After PR 1, verify with `cargo asm` on a
representative call site that codegen is byte-identical to the manual two-line form.
Escalate to `#[inline(always)]` if not (acceptable for leaf helpers in a hot loop).

### 8.3 New writes to `state.changed` outside helpers

**Risk:** Future contributors add `ctx.state.changed = true` in new rewrites by analogy
with surrounding code (some of which still has the legacy pattern mid-migration).

**Mitigation:** Two CI checks, both enabled in the final lockdown PR (§5.1):

- **Grep check:** any `state.changed =` write (either polarity) outside
  `ecma_context.rs` fails the build.
- **ast-grep structural check (§9 #4):** any direct `*expr = …` / `*stmt = …` in
  `crates/oxc_minifier/src/peephole/` without a documented allowlist comment fails the
  build.

The grep check catches the literal bypass. The structural check catches the subtler
bypass where someone writes `*expr = new;` then `ctx.notice_change();` — the slot
assignment bypasses the typed `replace_expression` helper but no `state.changed =` line
exists for grep to find.

## 9. Acceptance criteria

1. `replace_expression`, `replace_statement`, `notice_change`, `reset_changed` exist on
   `TraverseCtx<'a, MinifierState<'a>>`, all `#[inline]`.
2. Zero occurrences of `state.changed =` (either polarity) anywhere in
   `crates/oxc_minifier/` after the final migration PR EXCEPT inside the four helper
   bodies in `ecma_context.rs`. Enforced by the §5.1 CI check.
3. `MinifierState::changed` is `pub(crate)` (was `pub`).
4. **Structural-audit criterion (from Codex round 3):** Every `*expr = …` and `*stmt = …`
   in `crates/oxc_minifier/src/peephole/` either (a) goes through
   `ctx.replace_expression` / `ctx.replace_statement`, or (b) appears in an explicit
   allowlist documented in the final-PR description with a justification per site.
   Verified by an ast-grep rule run from CI:

   ```yaml
   id: peephole-direct-slot-assignment
   message: Direct slot assignment in peephole code must use ctx.replace_expression / ctx.replace_statement
   severity: error
   language: rust
   rule:
     any:
       - pattern: "*$E = $X" # expression slot
   files:
     - crates/oxc_minifier/src/peephole/**/*.rs
   ```

   (Allowlist mechanism: `// ast-grep-ignore: peephole-direct-slot-assignment — reason: …`
   on the line above each justified exception. The final PR description lists every
   allowlist entry.)

5. `cargo test -p oxc_minifier` passes with no expected-output changes.
6. `cargo coverage -- minifier` shows no conformance regression.
7. `just minsize` produces zero size deltas across the migration (any non-zero delta is
   explained and called out in the PR description).
8. `cargo test -p oxc_mangler` passes unchanged.
9. The `dead_drop_mutates_ast` gate (`minimize_statements.rs:22`) is preserved — verified
   by code review.
10. `cargo asm` (or equivalent inspection) on a representative `replace_expression` site
    confirms inlined codegen matches the manual two-line form.

## 10. Rollback

Pure refactor. The `exit_program` algorithm is unchanged, the fixed-point loop is
unchanged, output is bit-identical. Revert per-PR (stacked) or single-PR (sweep). No data
migration, no public API change outside the crate (helpers are `pub` on a
crate-local-typed `TraverseCtx<MinifierState>`).
