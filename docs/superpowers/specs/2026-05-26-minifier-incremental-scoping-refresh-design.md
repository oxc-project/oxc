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
4. Preserve current minified output exactly. Success criteria: `cargo test -p oxc_minifier`
   passes, `cargo coverage -- minifier` shows no conformance regression, `just minsize`
   produces zero deltas.

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
`new` are also REMOVED from the accumulator on every helper call — this handles the
cross-call resurrection case that prior spec iterations got wrong (rewrite A drops a
subtree containing ref X, then rewrite B builds a subtree that preserves ref X via
`clone_in_with_semantic_ids`; the second walk re-marks X as live).

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
10. `cargo test -p oxc_minifier` passes with no expected-output changes.
11. `cargo coverage -- minifier` shows no conformance regression.
12. `just minsize` produces zero size deltas (any non-zero delta is investigated; the
    expected outcome is identical output).
13. `cargo test -p oxc_mangler` passes unchanged.
14. CI gates from the prior PR still pass (`./tools/check_state_changed.sh`, ast-grep
    rule). `state.changed = …` writes remain zero in the crate; this spec removes the
    field entirely so the gate becomes trivially satisfied.

## 8. Migration strategy

The work decomposes into 4 stacked PRs in order:

1. **PR 1: Prerequisites.** Fix `convert_to_dotted_properties.rs:26, 40` (Task #28) and
   refactor `minimize_statements.rs:1387`. After this, EVERY peephole mutation goes
   through a helper. `just minsize` may have non-zero deltas here — these are the latent
   bugs surfacing as correct optimizations that previously didn't fire. Document each in
   the PR description.

2. **PR 2: Counter + remove `reset_changed`.** Add `MinifierState::mutations: u64`, keep
   `changed: bool` in parallel (helpers bump both). Loop driver switches to
   snapshot-compare. `reset_changed()` removed. Verify behavior unchanged.

3. **PR 3: DropDiff infrastructure.** Add `PassDirty` struct, `DropDiff` collector, the
   walk-and-diff methods on each helper. Don't yet consume in `exit_program` —
   `LiveUsageCollector` continues to run authoritatively. This PR is a no-op observable;
   it builds the data without consuming it.

4. **PR 4: Switch `exit_program` consumer and delete `LiveUsageCollector`.** Add
   `Scoping::retain_resolved_references_excluding`. Rewrite `exit_program` to consume
   `dirty.*`. Delete `LiveUsageCollector` and the bool field. This is the load-bearing
   PR — if `just minsize` regresses, the bug is here.

Each PR includes `cargo test`, `cargo coverage -- minifier`, `just minsize`, and the
existing CI gates. Per the prior migration's discipline, mid-stack PRs should keep the
build green.

## 9. Rollback

PR 4 is the only PR with semantic risk. Revert PR 4 alone restores the previous
`LiveUsageCollector` behavior; the helper-instrumentation in PR 3 stays harmless. PR 1's
fixes are independent and should not be reverted (they remove latent bugs). PR 2 is a
pure refactor with no semantic impact.

## 10. Out of scope

- Per-rule mutation telemetry (user explicitly chose global counter in design Q1).
- Killing the fixed-point loop / capping iterations.
- Dirty-region traversal in iterations 2..N.
- Audit of `normalize.rs`, `mangler/`, or other crates for similar patterns.
- Replacing `notice_change` with finer-grained helpers (operand swap vs collection
  mutation vs etc.).
- Measured speedup. Architectural correctness is the success bar.
