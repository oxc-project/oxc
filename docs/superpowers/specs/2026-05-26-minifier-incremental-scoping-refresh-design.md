# Incremental scoping refresh + mutation counter

**Status:** Design — pending user review
**Date:** 2026-05-26
**Scope:** `crates/oxc_minifier/`
**Stacks on:** `docs/superpowers/specs/2026-05-25-minifier-eliminate-changed-flag-design.md`

## 1. Problem

Two follow-ups from the helper-migration spec become tractable now that all AST mutations
go through 6 typed helpers (`replace_expression`, `replace_statement`,
`replace_assignment_target_property`, `replace_property_key`, `notice_change`,
`reset_changed`) on `TraverseCtx<'a, MinifierState<'a>>`.

### 1.1 `exit_program` does a full-AST walk on every changed pass

`PeepholeOptimizations::exit_program` (`peephole/mod.rs:168-187`) runs
`LiveUsageCollector` to scan the live program for surviving `IdentifierReference`s and
direct `eval(...)` calls whenever `state.changed` is set. The walk is O(program size) and
runs once per fixed-point iteration that mutated anything. Its single purpose is to feed
`scoping.retain_resolved_references(&live)` and `refresh_direct_eval_flags(...)` so the
`Scoping` data stays consistent with the rewritten AST.

The walk exists because rewrites *destroy information* (which subtree was dropped, what
references it contained), and `exit_program` *reconstructs* that information from the
live AST. The dropped subtree is hot in cache at the moment of the rewrite; cold by
`exit_program`. With helpers as the chokepoint for every mutation, the reconstruction
becomes unnecessary — the helper can collect dirtiness at the mutation site.

### 1.2 `state.changed: bool` is a coarse signal

A bool answers "did anything change since the last reset?" The fixed-point loop driver
needs that signal, but so does the proposed incremental refresh (to decide whether to
walk dropped subtrees at all). Two consumers, each needing a snapshot-compare, with one
shared bit currently being reset between them.

Replacing the bool with a monotonic `u64` counter lets each consumer snapshot independently
and compare without coordination. As a side benefit, the dedicated `reset_changed()`
helper becomes obsolete.

## 2. Goals

1. Eliminate the `LiveUsageCollector` post-pass walk over `IdentifierReference`s.
2. Reduce the direct-eval refresh from "every changed pass" to "only when an eval was
   actually dropped this pass."
3. Replace `state.changed: bool` with `state.mutations: u64` driven by snapshot compare.
4. Preserve current minified output exactly, EXCEPT for the latent-bug fixes in PR 1
   (Task #28 + `minimize_statements.rs:1387`) which by design enable previously-skipped
   optimizations. Success criteria: `cargo test -p oxc_minifier` passes, `cargo coverage
   -- minifier` shows no conformance regression, `just minsize` produces zero deltas in
   PRs 2-5 (any delta is investigated and must trace to a fix in PR 1).

Explicitly NOT goals:
- Measured wall-clock speedup. The user has chosen to ship the architectural win even if
  benchmarks show wash or marginal regression (see §6 risk #3 for fallback).
- Per-rule telemetry (deferred — separate spec if ever needed).
- Killing the fixed-point loop (deferred).
- Dirty-region traversal (skip clean subtrees in iterations 2..N) (deferred).

## 3. Approach

**Per-helper walk-old-and-new with set-difference.**

Each `replace_*` helper walks both `*slot` and `new`, accumulates
`dead = walk(old) − walk(new)` into a per-pass `PassDirty` structure. Refs that appear in
`new` are also REMOVED from the accumulator on every helper call. This handles:

- **Within-call preservation** — the case Codex flagged in v3 review:
  `substitute_is_object_and_not_null_for_left_and_right` rebuilds `new_expr` from
  `clone_in_with_semantic_ids` calls. The same `ReferenceId`s appear in both `*slot` and
  `new` simultaneously. The walk of `*slot` would mark them dead; the walk of `new`
  removes them, net live.
- **Cross-call preservation** — same `ReferenceId` appears in helper A's dropped
  subtree (marked dead) and later helper B's replacement value (the "remove from
  accumulator" step on B's `new` resurrects it). This case requires
  `clone_in_with_semantic_ids` to share IDs across distinct AST positions; rare in
  practice but the design handles it for free.

`drop_*` helpers walk only the dropped subtree; everything found is dead.

Direct-eval refresh stays as a gated full-walk: a single `eval_dropped: bool` flag flipped
when any dropped subtree contains a direct `eval(...)` call. At `exit_program`, if the
flag is set, run a small eval-only walk (`LiveDirectEvalCollector`) and refresh
`DirectEval` scope flags. The gate fires rarely because most programs don't drop eval
calls.

### Rejected alternatives

**B — walk old only, callers annotate preserved refs via `ctx.preserve_reference(id)`.**
Re-introduces the silent-failure mode the helper migration just eliminated; rejected.

**C — record dirty subtree spans, defer walks to `exit_program`.** Spans don't uniquely
identify subtrees in the AST. Still walks. Complex queue processing. Rejected.

## 4. API changes

### 4.1 `MinifierState`

```rust
pub struct MinifierState<'a> {
    // existing fields ...

    /// Monotonic mutation counter. Bumped exactly once per helper call.
    /// Replaces `changed: bool`. The loop driver snapshots before each pass
    /// and compares; `exit_program` snapshots at `enter_program` and compares.
    pub(crate) mutations: u64,

    /// Per-pass dirty data accumulated by helpers. Consumed and reset by
    /// `exit_program`.
    pub(crate) dirty: PassDirty<'a>,
}

pub struct PassDirty<'a> {
    /// `ReferenceId`s whose AST node has been removed and not re-installed in
    /// any later mutation this pass. Consumed by `exit_program` to prune
    /// `Scoping`'s resolved-references list.
    pub(crate) dead_refs: FxHashSet<ReferenceId>,

    /// Names of unresolved references whose last AST occurrence has been
    /// removed. Consumed by `exit_program` (with a confirmation walk) to
    /// prune `Scoping::root_unresolved_references`.
    pub(crate) dead_unresolved: FxHashSet<Atom<'a>>,

    /// At least one direct `eval(...)` call was dropped this pass.
    /// Gates the small `LiveDirectEvalCollector` walk at `exit_program`.
    pub(crate) eval_dropped: bool,
}
```

`MinifierState::changed: bool` is removed.

### 4.2 Helpers on `TraverseCtx`

Final API has **7 helpers**: 4 existing `replace_*` + 2 NEW `drop_*` + the existing
`notice_change`. The `reset_changed()` helper is REMOVED. The 6 walking helpers all
update `state.dirty` via a private `DropDiff` collector; `notice_change` only bumps the
counter.

```rust
impl<'a> TraverseCtx<'a, MinifierState<'a>> {
    #[inline]
    pub fn replace_expression(&mut self, slot: &mut Expression<'a>, new: Expression<'a>) {
        // walk old slot for refs/evals, then walk `new` to remove any refs that
        // survive into the replacement (cross-call resurrection)
        self.dirty_diff().walk_old_expression(slot).resurrect_from_expression(&new);
        *slot = new;
        self.state.mutations += 1;
    }

    #[inline]
    pub fn replace_statement(&mut self, slot: &mut Statement<'a>, new: Statement<'a>) { /* … */ }

    #[inline]
    pub fn replace_assignment_target_property(
        &mut self,
        slot: &mut AssignmentTargetProperty<'a>,
        new: AssignmentTargetProperty<'a>,
    ) { /* … */ }

    #[inline]
    pub fn replace_property_key(&mut self, slot: &mut PropertyKey<'a>, new: PropertyKey<'a>) { /* … */ }

    // NEW in this spec — added because helpers must walk every dropped subtree,
    // including those popped from collections or dropped without a replacement value.
    #[inline]
    pub fn drop_expression(&mut self, expr: &Expression<'a>) {
        self.dirty_diff().walk_old_expression(expr);
        self.state.mutations += 1;
    }

    #[inline]
    pub fn drop_statement(&mut self, stmt: &Statement<'a>) {
        self.dirty_diff().walk_old_statement(stmt);
        self.state.mutations += 1;
    }

    #[inline]
    pub fn notice_change(&mut self) {
        self.state.mutations += 1;
    }

    // `reset_changed()` is REMOVED — `mutations` is monotonic across the session,
    // snapshot-compare drives the loop and the exit_program refresh decision.
}
```

`dirty_diff()` is a private accessor that returns a `DropDiff` collector with the
required borrows of `state.dirty` (mutable) and `scoping` (immutable). Implementation
detail: the borrow split is handled via field-level borrowing on `TraverseCtx` (its
fields are disjoint, so NLL permits the simultaneous borrows).

`DropDiff` is a private type (likely a new file `traverse_context/drop_diff.rs`). Its job
is to walk AST subtrees collecting `IdentifierReference`s and direct `eval(...)` calls,
updating the per-pass `PassDirty` accumulator. It uses `oxc_ast_visit::Visit` like the
existing `LiveUsageCollector`. The exact method chain (`walk_old_*` vs
`resurrect_from_*`) controls whether refs are ADDED to or REMOVED from `dead_refs` —
walks of dropped subtrees add; walks of replacement values remove.

The 4 `walk_old_*` / `resurrect_from_*` method pairs (one per slot type) are mechanical
but verbose; their existence is the price of providing typed helpers for 4 distinct AST
slot types.

### 4.3 Loop driver

`crates/oxc_minifier/src/compressor.rs::run_in_loop`:

```rust
loop {
    let snapshot = ctx.state().mutations;
    PeepholeOptimizations.run_once(program, ctx);
    if ctx.state().mutations == snapshot {
        break;
    }
    // existing max-iteration guard unchanged
}
```

The `enter_program` reset is removed (was `ctx.reset_changed()` after the migration). The
counter is monotonic across the entire session; only snapshot comparisons matter.

`exit_program` no longer needs a guard at its start — see §5.

## 5. `exit_program` redesign

```rust
fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
    let dirty = &ctx.state.dirty;

    // (1) Resolved references — direct consumption, no walk.
    if !dirty.dead_refs.is_empty() {
        ctx.scoping_mut().retain_resolved_references_excluding(&dirty.dead_refs);
    }

    // (2) Unresolved references — gated confirmation walk by name.
    if !dirty.dead_unresolved.is_empty() {
        Self::prune_unresolved_refs(
            ctx.scoping_mut(),
            &dirty.dead_unresolved,
            program,
        );
    }

    // (3) Direct-eval — gated full walk only when an eval was dropped.
    if dirty.eval_dropped {
        let mut live = LiveDirectEvalCollector::new();
        live.visit_program(program);
        Self::refresh_direct_eval_flags(ctx.scoping_mut(), &live.scopes);
    }

    // Reset for next pass.
    ctx.state.dirty.reset();

    debug_assert!(ctx.state.dce || ctx.state.class_symbols_stack.is_exhausted());
}
```

`LiveUsageCollector` is **deleted**. Its two responsibilities split:

- The `IdentifierReference` walk is distributed across the helpers.
- The direct-eval-call walk becomes `LiveDirectEvalCollector` (eval-only, smaller),
  invoked conditionally.

`retain_resolved_references_excluding(&dead_refs)` is a new method on `Scoping` if one
doesn't already exist; the existing `retain_resolved_references(&live_set)` would require
us to invert dead_refs into live_refs, which negates the locality benefit. Adding the
"exclude" variant is straightforward — it iterates each symbol's reference list and drops
entries whose ID is in the dead set. See §6.4 for cross-crate scope.

`prune_unresolved_refs` is a new method on `PeepholeOptimizations`:

```rust
fn prune_unresolved_refs(
    scoping: &mut Scoping,
    candidates: &FxHashSet<Atom<'a>>,
    program: &Program<'a>,
) {
    // For each candidate name, scan the live program for surviving
    // IdentifierReferences with that name. If none, prune the entry from
    // root_unresolved_references.
    let mut survivors = FxHashSet::<Atom<'a>>::default();
    let mut collector = NamedRefCollector { candidates, survivors: &mut survivors };
    collector.visit_program(program);
    for name in candidates.iter() {
        if !survivors.contains(name) {
            scoping.remove_unresolved_reference(name);
        }
    }
}
```

This is the one piece of state we cannot make fully local. The unresolved set is keyed by
name and a name can have many references; pruning a name requires confirming no surviving
reference uses it. The walk is bounded by `candidates.len()` (typically very small —
empty for most passes) but scans the program once when non-empty.

## 6. Risks and limitations

### 6.1 Prerequisite: fix `convert_to_dotted_properties.rs:26, 40`

Two latent bugs deferred in PR 18 (Task #28) bypass the helpers: their enclosing
functions take `&TraverseCtx` (immutable) and mutate an AST slot directly without going
through the API. Today this only means a fixed-point pass might not re-run; with
incremental refresh, the dropped subtree is invisible to the helpers, so dead references
leak into `Scoping` and cause downstream consumers (mangler, codegen) to see stale data.

**Must fix before this design ships.** Two options:
- **(a)** Refactor both function signatures to `&mut TraverseCtx` and use
  `replace_expression`. Cleanest.
- **(b)** Propagate a bool return + caller helper-calls (the `substitute_single_use_*`
  pattern at `minimize_statements.rs:1387`). More invasive.

Recommend **(a)** unless the signature change cascades through many callers.

This work is added to Task #28 as a hard prerequisite.

### 6.2 Prerequisite: refactor `minimize_statements.rs:1387`

`substitute_single_use_symbol_in_expression` takes `&TraverseCtx` (immutable) and
propagates a bool return; callers translate to `ctx.notice_change()`. With incremental
refresh, the dropped subtree at line 1387 (`*target_expr = replacement.take_in(...)`) is
not walked, so its references leak.

Fix: refactor the function to take `&mut TraverseCtx` and call `replace_expression`
directly. Caller-side `if changed { ctx.notice_change(); }` becomes redundant and is
removed.

This is mechanical and contained; tracked as a sub-task of this spec's implementation.

### 6.3 Performance unknown — `FxHashSet` vs `BitSet` for `dead_refs`

The existing `LiveUsageCollector` uses `BitSet` documented at `peephole/mod.rs:571` to
drop insert/contains cost from ~25 to ~5 cycles. `PassDirty::dead_refs` is
`FxHashSet<ReferenceId>` because we need `remove` for cross-call resurrection (BitSet
doesn't support deletion).

Mitigation tiers if benchmarks regress (in order of cost):
1. **Profile.** Confirm `FxHashSet` insert is actually hot.
2. **Hybrid structure.** `BitSet` for IDs known at pass start (most cases),
   `FxHashSet` overflow for IDs created mid-pass. Resurrection works by tombstoning bits.
3. **Lazy init.** Allocate `dead_refs` only on first dirty mutation (skip the
   O(`references_len`) bitset zero-fill on no-op passes).
4. **Defer the refresh** — fall back to the gated `LiveUsageCollector` walk for the
   resolved-ref case if (1-3) don't recover the regression.

Per §2, we ship the architectural design even at marginal regression. Mitigations are
follow-ups only if needed.

### 6.4 Cross-crate scope: `retain_resolved_references_excluding`

If `Scoping` doesn't expose this method, it lives in `oxc_semantic` and needs a small
public addition. Acceptable per the migration's existing precedent (`Scoping` already
exposes `retain_resolved_references`, `delete_resolved_reference`, etc.). Inverting
dead-refs into live-refs in the minifier instead would force walking all symbols' ref
lists redundantly.

### 6.5 Allocator concerns

`PassDirty::dead_refs` and `dead_unresolved` are `FxHashSet`s allocated on the heap, not
in the `oxc_allocator` arena. Per-pass allocation cost is bounded by O(mutations per
pass). The existing `MinifierState` already holds heap allocations (`concat_scratch:
String`, `pure_functions: FxHashMap<...>`, etc.), so this is consistent.

### 6.6 Walk cost per helper call

Adds O(size of old + size of new) per `replace_*` call. Total per-pass cost is bounded by
O(total mutated region size), same big-O as today's single LiveUsageCollector walk over
the whole program. Cache locality is better (walks happen on hot subtrees) but per-call
overhead is real. Per §2 we accept this.

### 6.7 Prerequisite: migrate Pattern C/D sites that silently drop references

The prior migration deliberately left a class of mutations using `notice_change()` because
no slot-typed helper applied:

- **`Option<Expression>` / `Option<Statement>` field clears** — `for_stmt.init = None`,
  `for_stmt.update = None`, `for_stmt.test = None`, `stmt.argument = None`, `decl.init = None`,
  `*func.id = None` and `*class.id = None` (×4 in `substitute_alternate_syntax.rs`),
  `import_decl.specifiers = None` (in `remove_unused_declaration.rs`).
- **Collection mutations that drop elements** — `array_expr.elements.retain_mut(|e| !remove_unused_expression(e, ctx))`,
  `sequence_expr.expressions.retain_mut(...)`, `stmts.pop()` / `stmts.drain(...)` /
  `stmts.truncate(...)` / `stmts.splice(...)` in `minimize_statements.rs` and
  `remove_dead_code.rs`.
- **Class field `take()`** — `super_class.take()` and `def.value.take()` inside
  `remove_unused_expression.rs::remove_unused_class`.

Today these are correct because `LiveUsageCollector` walks the live program after the
pass and finds what survived; anything missing is dead. After this design deletes
`LiveUsageCollector`, these silent drops leak dead references into `Scoping` (resolved
and unresolved both), corrupting downstream mangler/codegen reads.

**Must fix before this design ships.** Migration pattern for each shape:

```rust
// Option<Expression> clear:
- field = None;
+ if let Some(old) = field.take() {
+     ctx.drop_expression(&old);
+ }

// retain_mut with predicate that decides to drop:
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
+ // notice_change is now redundant — drop_expression already bumped the counter

// stmts.pop() of a Statement:
- let dropped = stmts.pop().unwrap();
- ctx.notice_change();
+ let dropped = stmts.pop().unwrap();
+ ctx.drop_statement(&dropped);

// Class field take():
- e.super_class.take();
- ctx.notice_change();
+ if let Some(old) = e.super_class.take() {
+     ctx.drop_expression(&old);
+ }
```

**Scope:** Audit all `notice_change()` call sites in `crates/oxc_minifier/src/peephole/`
(grep for `ctx.notice_change` after the prior migration lands). Categorize each:
- **Drops nothing** (operand swap, operator-only tweak, bool/span field flip) → keep as
  `notice_change()`.
- **Drops a subtree** (Option clear, collection element removal, take()) → migrate to
  `ctx.drop_expression(&dropped)` / `ctx.drop_statement(&dropped)` BEFORE the actual
  drop; remove the redundant `notice_change()` (the `drop_*` helper bumps the counter).

This migration is mechanical but touches every Pattern C/D site that drops references.
Rough estimate: 15-25 sites across 5-7 files. Added to PR 1 of the migration plan
(§8).

## 7. Acceptance criteria

1. `MinifierState::changed` field is REMOVED.
2. `MinifierState::mutations: u64` and `MinifierState::dirty: PassDirty<'a>` are present
   with `pub(crate)` visibility.
3. `TraverseCtx::reset_changed()` is REMOVED. `compressor.rs::run_in_loop` uses
   snapshot-compare on `state.mutations`.
4. Final API has 7 helpers. The 6 walking helpers (`replace_expression`,
   `replace_statement`, `replace_assignment_target_property`, `replace_property_key`,
   `drop_expression`, `drop_statement`) call `DropDiff::walk_old_*` on the
   dropped/replaced subtree; the 4 `replace_*` variants additionally call
   `resurrect_from_*` on `&new`. The 7th helper `notice_change` only bumps the counter.
   `drop_expression` and `drop_statement` are NEW (introduced by this spec — the prior
   migration only had `replace_*` variants).
5. `LiveUsageCollector` is deleted. `LiveDirectEvalCollector` (eval-only) replaces it.
6. `peephole/mod.rs::exit_program` consumes `dirty.dead_refs`, `dirty.dead_unresolved`,
   and `dirty.eval_dropped` directly. Each consumer has its own early-exit when the
   relevant collection is empty.
7. `Scoping::retain_resolved_references_excluding` exists (added to `oxc_semantic` if not
   already present).
8. `convert_to_dotted_properties.rs:26, 40` no longer bypass the helpers (Task #28
   resolved as prerequisite).
9. `minimize_statements.rs:1387` no longer caller-tracked; the enclosing function takes
   `&mut TraverseCtx` and uses `replace_expression` directly.
9a. All Pattern C/D sites that drop references (§6.7) are migrated to use
    `ctx.drop_expression` / `ctx.drop_statement` BEFORE the drop. After PR 2 of the
    migration plan, `grep` for `notice_change()` returns only sites where no AST subtree
    is being dropped (operand swap, operator-only tweak, bool/span flip). Verified by
    audit.
10. `cargo test -p oxc_minifier` passes with no expected-output changes.
11. `cargo coverage -- minifier` shows no conformance regression.
12. `just minsize` produces zero size deltas (any non-zero delta is investigated; the
    expected outcome is identical output).
13. `cargo test -p oxc_mangler` passes unchanged.
14. CI gates from the prior PR still pass (`./tools/check_state_changed.sh`, ast-grep
    rule). `state.changed = …` writes remain zero in the crate; this spec removes the
    field entirely so the gate becomes trivially satisfied.

## 8. Migration strategy

The work decomposes into 5 stacked PRs in order:

1. **PR 1: Latent-bug fixes (Task #28 + `minimize_statements.rs:1387`).** Fix the two
   `convert_to_dotted_properties.rs:26, 40` silent bypasses and refactor
   `substitute_single_use_symbol_in_expression` to take `&mut TraverseCtx`. **`just
   minsize` is expected to show non-zero deltas here** — these are the latent bugs
   surfacing as correct optimizations that previously didn't fire. Document each diff in
   the PR description with the underlying optimization that's now firing.

2. **PR 2: Add `drop_expression` / `drop_statement` helpers and migrate Pattern C/D drop
   sites (§6.7).** Add the two new helpers (bumping `state.changed` for now). Migrate
   every site that silently drops references via Option-clear, collection mutation, or
   `take()`. After this PR, every reference-dropping mutation goes through a helper.
   `just minsize` MUST be zero deltas here — these migrations are pure refactor (helpers
   bump the same `state.changed` bool the silent sites previously skipped, but the
   live-program walk in `exit_program` was authoritative so output is unchanged).

3. **PR 3: Counter + remove `reset_changed`.** Add `MinifierState::mutations: u64`, keep
   `changed: bool` in parallel (helpers bump both). Loop driver switches to
   snapshot-compare. `reset_changed()` removed. `just minsize` MUST be zero deltas —
   pure refactor.

4. **PR 4: DropDiff infrastructure.** Add `PassDirty` struct, `DropDiff` collector, the
   walk-and-diff methods on each helper. Do NOT yet consume in `exit_program` —
   `LiveUsageCollector` continues to run authoritatively. This PR is a no-op observable;
   it builds the data without consuming it. `just minsize` MUST be zero deltas.

5. **PR 5: Switch `exit_program` consumer and delete `LiveUsageCollector`.** Add
   `Scoping::retain_resolved_references_excluding`. Rewrite `exit_program` to consume
   `dirty.*`. Delete `LiveUsageCollector` and the bool field. This is the load-bearing
   PR — if anything regresses, the bug is here. `just minsize` MUST be zero deltas; any
   non-zero delta means PassDirty under-tracked something.

Each PR includes `cargo test`, `cargo coverage -- minifier`, `just minsize`, and the
existing CI gates. Per the prior migration's discipline, mid-stack PRs should keep the
build green. PR 1 is the only PR where `just minsize` deltas are expected.

## 9. Rollback

- **PR 1 (latent-bug fixes)** changes minified output by enabling previously-skipped
  optimizations. Should NOT be reverted; if a specific minsize diff is judged
  unacceptable, file as a separate bug investigation rather than rolling back the helper
  contract restoration.
- **PR 2 (Pattern C/D drop migrations)** is pure refactor (output bit-identical). Revert
  if borrow-checker friction or test failure surfaces, but no semantic risk.
- **PR 3 (counter + remove `reset_changed`)** is pure refactor. Safe to revert.
- **PR 4 (DropDiff infrastructure)** is no-op observable (data built but not consumed).
  Safe to revert in isolation.
- **PR 5 (switch consumer, delete LiveUsageCollector)** is the only PR with semantic
  risk. Revert PR 5 alone restores `LiveUsageCollector` and the pre-existing
  `exit_program` flow; PRs 1-4 stay harmless.

## 10. Out of scope

- Per-rule mutation telemetry (user explicitly chose global counter in design Q1).
- Killing the fixed-point loop / capping iterations.
- Dirty-region traversal in iterations 2..N.
- Audit of `normalize.rs`, `mangler/`, or other crates for similar patterns.
- Replacing `notice_change` with finer-grained helpers (operand swap vs collection
  mutation vs etc.).
- Measured speedup. Architectural correctness is the success bar.
