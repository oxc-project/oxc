# Minifier: live-maintained reference counts (replace dirty-set + batch retain)

Date: 2026-06-09
Status: design (approved in brainstorming)
Branch: `spec/minifier-incremental-scoping` (layers on / supersedes parts of PR #22736)

## 1. Summary

Replace the per-pass dirty accumulator (`PassDirty::dead_refs` bitset) and the
end-of-pass `Scoping::retain_resolved_references_excluding` list compaction with
a **persistent, eagerly-initialized, live-maintained per-symbol reference-count
store** held inside the minifier's existing `SymbolValues`.

The minifier never consumes reference-list _identities_ — only aggregate
_counts_ (`write == 0`, total `.count()`, "all read-only"). Today it materializes
those counts in `SymbolValues` by re-reading Scoping's reference lists every pass,
then throws them away on `reset()`. This design **stops recomputing and instead
maintains the counts incrementally** as the AST mutates, and **stops touching
Scoping's lists at all** (they are allowed to go stale; nothing reads them after
Normalize).

This is "maintain the data, not the deltas." It also restores the pre-PR
self-healing property (counts derived from the live program, not the handed-in
Scoping) — paid **once** up front instead of every pass — which _relaxes_ the
incoming-`Scoping` trust precondition the incremental-scoping refresh introduced.

## 2. Motivation

PR #22736 deleted the per-pass `LiveUsageCollector` full walk and replaced it
with a dirty `dead_refs` bitset consumed by a single `retain_resolved_references_excluding`
per pass. That is correct and fast, but it keeps two representations of truth (the
minifier-side dirty set + Scoping's lists) and trusts the incoming Scoping to match
the program (the rolldown-facing precondition).

The lists are O(n) to query and O(n²) to maintain under deletion — the documented
bundler regression (thousands of `var import_X = __toESM(require_Y())` removals each
deleting from a huge shared reference list). The dirty-set + batch retain exists to
dodge that O(n²). But the only thing the minifier actually _needs_ from those lists
is a handful of O(1) aggregates. Caching and maintaining those aggregates directly
removes both the dirty shadow and the list churn.

## 3. Goals / Non-goals

Goals:

- Maintain per-symbol reference counts incrementally; eliminate the per-pass
  recompute, the `dead_refs` bitset, and the batch list compaction.
- Derive initial counts from the live program (self-healing), once.
- Preserve correct minifier decisions; accept fresher (mid-pass) counts.

Non-goals:

- Changing `Scoping`'s storage or adding maintenance machinery to `oxc_semantic`.
- Maintaining Scoping's reference _lists_ (they go stale; that's intentional).
- Pruning unresolved references (already dropped as dead work in #22736).
- Mangler behavior (it rebuilds Scoping fresh; unaffected).

## 4. Current architecture (to be replaced)

- `SymbolValues: IndexVec<SymbolId, Option<SymbolValue>>`, `reset()` to all-`None`
  every `enter_program`. `SymbolValue` bundles per-pass value data
  (`initialized_constant`, `is_fresh_value`, `exported`, `scope_id`) **and**
  reference counts (`read/write/member_write_target_read`).
- `init_value` (lazy, from the inline pass) computes counts by iterating
  `Scoping::get_resolved_references(symbol)`.
- Mutations route through typed helpers; drops accumulate `ReferenceId`s into
  `PassDirty::dead_refs` (a `BitSet`), consumed at `exit_program` via
  `retain_resolved_references_excluding`. `debug_assert_no_over_prune` guards it.
- Three optimizations read Scoping lists directly inside the loop:
  - `remove_dead_code.rs:489` — `get_resolved_references(sym).all(|r| r.flags().is_read_only())`
  - `substitute_alternate_syntax.rs:902,907` — `get_resolved_references(id).count() != cached`
  - `substitute_alternate_syntax.rs:919` — `get_resolved_references(id).count() > 1`
- `mod.rs:120 is_symbol_mutated` prefers cached `write_references_count`, else
  falls back to `Scoping::symbol_is_mutated` (O(refs) scan).

## 5. Proposed architecture

### 5.1 The count store (inside `SymbolValues`, no new top-level store)

Split `SymbolValues` into two indexed sub-stores with distinct lifecycles:

```rust
struct ReferenceCounts {
    read: u32,                 // refs with the Read flag
    write: u32,                // refs with the Write flag
    member_write_target: u32,  // refs that are member-write targets
    total: u32,                // number of resolved reference entries
    non_read_only: u32,        // refs where !flags.is_read_only()
}

struct SymbolValues<'a> {
    counts: IndexVec<SymbolId, ReferenceCounts>, // persistent, all symbols, live-maintained
    values: IndexVec<SymbolId, Option<ValueData<'a>>>, // per-pass lazy (declarators only)
}
```

`ValueData` is the current `SymbolValue` minus the count fields. `counts` is
sized once to `symbols_len()` and **never reset**. `values` keeps its per-pass
`reset()`-to-`None` lifecycle (value data genuinely changes per pass and is only
meaningful for declarators).

Indexing by `SymbolId` (dense, fixed — no minifier pass mints symbols) **removes
the `BitSet` capacity/mid-pass-mint fragility** entirely: there is no per-reference
index to overflow.

### 5.2 Eager init via one program walk (self-healing)

On the first `enter_program` (post-Normalize), walk the live program once: for each
`IdentifierReference` with a resolved `reference_id`, look up its `symbol_id` and
`flags` from Scoping and tally into `counts`. Counts therefore reflect the **actual
program**, not Scoping's (possibly stale) lists. Subsequent `enter_program`s skip
re-init (the store is maintained, not rebuilt). The same walk routine backs the
debug assertion (§7).

Rationale for walking the program rather than reading Scoping lists: it makes the
counts robust to any upstream staleness (Normalize, or a caller-supplied Scoping),
restoring the deleted full-walk's self-healing — but paid once, not per pass.

### 5.3 Live maintenance (`DropDiff` adjusts counts)

`DropDiff` keeps its structure (two modes, `resurrect_is_noop` skip, eval
detection, `reference_id.is_none()` tolerance). Only the leaf action changes:

- MarkDead (`walk_old_*`): for each resolved ref, `counts[symbol]` is decremented
  per the ref's flags (`total -= 1`; `read/write/member/non_read_only -=` as the
  flags dictate).
- Resurrect (`resurrect_from_*`): symmetric increment.

`DropDiff` borrows `&mut SymbolValues.counts` + `&Scoping` (for `reference_id →
symbol_id` + flags). The `drop_*` helpers only decrement; `replace_*` decrement
(old) then increment (new) atomically before `*slot = new`. `notice_change` stays
count-free. Mints (`create_bound_reference`) increment when their node is walked by
a resurrect (same as a surviving clone).

### 5.4 Consumer migration (counts replace list reads)

- Count readers (`get_symbol_value(...).write_references_count`, etc.) read
  `counts[symbol]` directly (always present, no `Option`, live).
- `remove_dead_code.rs:489` `.all(is_read_only)` → `counts[sym].non_read_only == 0`
  (exact; `write == 0` would be wrong for refs that are neither read nor write).
- `substitute_alternate_syntax.rs:902,907` `.count() != cached` → `counts[id].total
!= cached`, with the cached value captured from `counts[id].total` at the same
  point it is captured today.
- `substitute_alternate_syntax.rs:919` `.count() > 1` → `counts[id].total > 1`.
- `mod.rs:120 is_symbol_mutated` → `counts[sym].write > 0` for all symbols; the
  `symbol_is_mutated` Scoping-scan fallback is removed (counts now cover every
  symbol, not just declarators). Requires confirming
  `symbol_is_mutated(sym) ⟺ write > 0` (the existing code already treats the cached
  path as equivalent).
- `normalize.rs:179` is **unchanged** — Normalize runs once before the loop on a
  fresh Scoping; it does not consume the count store.

### 5.5 `init_value` simplification

`init_value` no longer iterates Scoping lists for counts; it reads `counts[symbol]`
and populates only `ValueData` (`initialized_constant`, `is_fresh_value`,
`exported`, `scope_id`). It stays lazy/per-pass for value data.

### 5.6 Deleted

- `PassDirty` and `MinifierState::dirty` (the `dead_refs` bitset, `eval_dropped`
  moves to a tiny standalone flag on state — eval handling is retained as in #22736).
- `Scoping::retain_resolved_references_excluding` (no remaining minifier caller;
  remove unless another crate uses it).
- The per-pass count recompute path in `init_value`.
- `debug_assert_no_over_prune` (replaced by §7).

Note: `eval_dropped` and the gated `LiveDirectEvalCollector` refresh are unchanged —
direct-eval handling does not depend on reference counts.

## 6. Correctness invariants

1. **No observable intermediate state.** `walk_old_*` and `resurrect_from_*` run
   inside a single helper call, before control returns to the traversal.
   Optimizations only read counts _between_ helper calls, so they never see a
   half-applied replacement. Mid-pass drift is real but consistent at call
   boundaries (this is what "live maintenance" means; accepted).
2. **Mint/resurrect symmetry.** A reference added by a replacement value is
   counted (resurrect increment / mint); one removed is uncounted (walk_old
   decrement). Clone aliasing (`clone_in_with_semantic_ids`,
   `expression_identifier_with_reference_id`) is net-zero within its single
   `replace_expression` call, exactly as in #22736.
3. **Exact `is_read_only`.** `non_read_only` is maintained from the same flag the
   query inspects, so `non_read_only == 0 ⟺ all refs read-only`.
4. **Self-healing init.** Counts come from the live program, so a stale incoming
   Scoping (rolldown, Normalize) cannot corrupt them. This _relaxes_ the incoming-
   `Scoping` trust precondition from #22736: we no longer trust the incoming
   resolved-reference lists, only the `reference_id → symbol_id` resolution.
5. **Counts are non-negative.** Every decrement corresponds to a reference that the
   eager walk (or a prior mint) counted. The debug assertion (§7) catches any drift.

## 7. Debug assertion (replaces over-prune)

`debug_assert_counts_match` (debug builds only, at `exit_program` when any mutation
occurred): re-walk the live program tallying counts per symbol and assert they
equal the maintained `counts`. This turns the unit-test + `cargo coverage -- minifier`
corpus into a count-drift detector at zero release cost, the same way
`debug_assert_no_over_prune` did for the bitset.

## 8. Output impact + benchmark gate

- **`minsize` will likely change.** Fresher (lower) mid-pass counts let
  optimizations fire earlier; output should be equal-or-smaller and still correct.
  Re-snapshot `tasks/minsize` and review every diff for sanity (no size increases,
  no semantic change).
- **Benchmark gate.** This change only ships if single-file minifier benchmarks
  (kitchen-sink.tsx, cal.com.tsx, binder.ts, react.development.js) show it is at
  least neutral and wins on the large/bundler-shaped inputs vs the #22736 baseline.
  If it regresses without a compensating win, it is reverted (§9).

## 9. Risks & rollback

- Risk: a count-reading site whose semantics differ subtly from the maintained
  aggregate (esp. `is_read_only`, `symbol_is_mutated`). Mitigation: per-site
  migration verified in the plan; debug count-drift assertion; conformance corpus.
- Risk: a reference-dropping path that bypasses the helpers (would leave a count
  too high → missed optimization, _safe_ direction). Mitigation: the #22736 CI
  gates (`check_state_changed.sh`, ast-grep) already forbid bypasses.
- Risk: mid-pass drift changes output in an undesirable way. Mitigation: minsize
  re-snapshot review; this is the explicit, accepted tradeoff.
- Rollback: the change is a cohesive set of commits layered on #22736; reverting
  them restores the dirty-set + batch-retain design. Keep
  `retain_resolved_references_excluding` deletion as the last commit so rollback is
  a clean `git revert` range.

## 10. Relationship to PR #22736

This **supersedes** the consumption model #22736 introduced: it removes the
`dead_refs` bitset, `retain_resolved_references_excluding`, and
`debug_assert_no_over_prune`. The typed-helper discipline and the `DropDiff` walk
infrastructure from #22736 are the **prerequisite** that makes this possible and
are retained (only the leaf action and the consumer reads change). On this branch
it is a clearly-delimited follow-on commit range; it may alternatively be split into
its own PR once benchmarked.

## 11. Acceptance criteria

1. `cargo test -p oxc_minifier` green (debug; `debug_assert_counts_match` active).
2. `cargo test -p oxc_mangler -p oxc_semantic -p oxc_transformer_plugins` green.
3. `cargo coverage -- minifier` no regression vs baseline; no panic.
4. `tasks/minsize` re-snapshotted; every diff is equal-or-smaller and reviewed.
5. CI gates (`check_state_changed.sh`, ast-grep) pass.
6. `cargo clippy -p oxc_minifier --all-targets` clean.
7. Benchmark: neutral-or-better overall, wins on kitchen-sink/bundler inputs.
8. `PassDirty`, `retain_resolved_references_excluding`, `debug_assert_no_over_prune`
   removed; no remaining reader of Scoping resolved-reference lists inside the loop.
