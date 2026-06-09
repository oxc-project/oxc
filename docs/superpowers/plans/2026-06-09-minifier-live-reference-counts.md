# Minifier Live-Maintained Reference Counts — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the minifier's per-pass dead-ref bitset + batch Scoping-list compaction with a persistent, eagerly-initialized, live-maintained per-symbol reference-count store inside `SymbolValues`.

**Architecture:** Counts are derived from the live program once (self-healing), then maintained incrementally by the existing `DropDiff` walk (decrement on drop, increment on resurrect/mint). Consumers read O(1) counts instead of iterating Scoping reference lists; Scoping's lists are left untouched. Build it as a _shadow_ first (validated by a debug assertion across the test corpus), then switch consumers, then delete the old machinery.

**Tech Stack:** Rust, `oxc_minifier`, `oxc_semantic::Scoping`, `oxc_index::IndexVec`, `cargo test` / `cargo coverage` / `just minsize`.

Spec: `docs/superpowers/specs/2026-06-09-minifier-live-reference-counts-design.md`

---

## File Structure

- `crates/oxc_minifier/src/symbol_value.rs` — add `ReferenceCounts`; restructure `SymbolValues` into `counts` (persistent) + `values` (per-pass). Add accessors + `init_counts_from_program`.
- `crates/oxc_minifier/src/traverse_context/drop_diff.rs` — `DropDiff` borrows the count store; leaf action decrements/increments counts by reference flags.
- `crates/oxc_minifier/src/traverse_context/ecma_context.rs` — `dirty_diff()` borrows counts; `init_value` drops the count recompute; `tracked_constant_for_reference_id` reads counts.
- `crates/oxc_minifier/src/peephole/mod.rs` — `enter_program` eager-inits counts once; `exit_program` runs `debug_assert_counts_match`; later, the over-prune assert + retain are removed; `is_symbol_mutated` reads counts.
- `crates/oxc_minifier/src/peephole/{remove_dead_code,substitute_alternate_syntax,remove_unused_expression,inline}.rs` — migrate count/list reads.
- `crates/oxc_minifier/src/state.rs` — later: remove `PassDirty`/`dirty`; keep a standalone `eval_dropped` flag.
- `crates/oxc_semantic/src/scoping.rs` — later: remove `retain_resolved_references_excluding`.

Key invariant to preserve at every migration site: **`get_symbol_value(...)` returning `Some` is a semantic gate for _value data_ (declarator-tracked), not for counts.** Counts are always present. Keep the value-data `Option` gate where it exists; only move count reads to the count store.

---

## Task 1: Add `ReferenceCounts` + restructure `SymbolValues` (shadow, unconsumed)

**Files:**

- Modify: `crates/oxc_minifier/src/symbol_value.rs`
- Modify: `crates/oxc_minifier/src/traverse_context/ecma_context.rs:254-303` (`init_value` writes value-data only; counts written via a separate path)
- Test: `crates/oxc_minifier/src/symbol_value.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Write the failing unit test for `ReferenceCounts` add/sub**

In `symbol_value.rs`, add at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::ReferenceCounts;
    use oxc_syntax::reference::ReferenceFlags;

    #[test]
    fn reference_counts_add_sub_by_flags() {
        let mut c = ReferenceCounts::default();
        c.add(ReferenceFlags::read());            // read-only ref
        c.add(ReferenceFlags::write());           // write ref
        c.add(ReferenceFlags::read() | ReferenceFlags::write()); // read+write
        assert_eq!(c.total, 3);
        assert_eq!(c.read, 2);
        assert_eq!(c.write, 2);
        assert_eq!(c.non_read_only, 2); // the write and the read+write are not read-only
        c.sub(ReferenceFlags::write());
        assert_eq!(c.total, 2);
        assert_eq!(c.write, 1);
        assert_eq!(c.non_read_only, 1);
    }
}
```

- [ ] **Step 2: Run it to confirm it fails to compile** (type/methods absent)

Run: `cargo test -p oxc_minifier --lib reference_counts_add_sub_by_flags`
Expected: compile error (`ReferenceCounts` not found).

- [ ] **Step 3: Add the `ReferenceCounts` type**

In `symbol_value.rs`, add imports `use oxc_syntax::reference::ReferenceFlags;` and:

```rust
/// Per-symbol reference aggregate, maintained incrementally by the peephole
/// helpers. Replaces re-reading `Scoping::get_resolved_references` each pass.
#[derive(Debug, Default, Clone, Copy)]
pub struct ReferenceCounts {
    pub read: u32,
    pub write: u32,
    pub member_write_target: u32,
    /// Number of reference entries (a ref counts once regardless of flags).
    pub total: u32,
    /// Number of refs where `!flags.is_read_only()`. Lets the pure-function
    /// check ask "are all refs read-only?" as `non_read_only == 0` (exact —
    /// `write == 0` is wrong for refs that are neither read nor write).
    pub non_read_only: u32,
}

impl ReferenceCounts {
    #[inline]
    pub fn add(&mut self, flags: ReferenceFlags) {
        self.total += 1;
        if flags.is_read() { self.read += 1; }
        if flags.is_write() { self.write += 1; }
        if flags.is_member_write_target() { self.member_write_target += 1; }
        if !flags.is_read_only() { self.non_read_only += 1; }
    }

    #[inline]
    pub fn sub(&mut self, flags: ReferenceFlags) {
        self.total -= 1;
        if flags.is_read() { self.read -= 1; }
        if flags.is_write() { self.write -= 1; }
        if flags.is_member_write_target() { self.member_write_target -= 1; }
        if !flags.is_read_only() { self.non_read_only -= 1; }
    }
}
```

(If `ReferenceFlags` lacks `read()`/`write()` constructors, adapt the test to build flags via the available API; verify with `cargo doc`/source for `oxc_syntax::reference::ReferenceFlags`.)

- [ ] **Step 4: Run the test to confirm it passes**

Run: `cargo test -p oxc_minifier --lib reference_counts_add_sub_by_flags`
Expected: PASS.

- [ ] **Step 5: Restructure `SymbolValues` to hold persistent `counts` + per-pass `values`**

Rename the count fields out of `SymbolValue` (it becomes value-data only) and split the store:

```rust
#[derive(Debug)]
pub struct SymbolValue<'a> {
    pub initialized_constant: Option<ConstantValue<'a>>,
    pub exported: bool,
    pub is_fresh_value: bool,
    pub scope_id: ScopeId,
}

#[derive(Debug)]
pub struct SymbolValues<'a> {
    /// Persistent, never reset; maintained live by the peephole helpers.
    counts: IndexVec<SymbolId, ReferenceCounts>,
    /// Per-pass value data; reset to `None` each `enter_program`.
    values: IndexVec<SymbolId, Option<SymbolValue<'a>>>,
    counts_initialized: bool,
}

impl<'a> SymbolValues<'a> {
    pub(crate) fn new(len: usize) -> Self {
        let mut values = IndexVec::with_capacity(len);
        values.resize_with(len, || None);
        let mut counts = IndexVec::with_capacity(len);
        counts.resize_with(len, ReferenceCounts::default);
        Self { counts, values, counts_initialized: false }
    }

    /// Reset only the per-pass value data. Counts persist.
    pub fn reset(&mut self) {
        for slot in &mut self.values { *slot = None; }
    }

    #[inline]
    pub fn init_value(&mut self, symbol_id: SymbolId, value: SymbolValue<'a>) {
        self.values[symbol_id] = Some(value);
    }

    #[inline]
    pub fn get_symbol_value(&self, symbol_id: SymbolId) -> Option<&SymbolValue<'a>> {
        self.values.get(symbol_id)?.as_ref()
    }

    /// Always-present reference counts for a symbol.
    #[inline]
    pub fn reference_counts(&self, symbol_id: SymbolId) -> ReferenceCounts {
        self.counts[symbol_id]
    }

    /// Mutable access for the maintaining helpers.
    #[inline]
    pub(crate) fn counts_mut(&mut self) -> &mut IndexVec<SymbolId, ReferenceCounts> {
        &mut self.counts
    }

    pub(crate) fn counts_initialized(&self) -> bool { self.counts_initialized }
    pub(crate) fn mark_counts_initialized(&mut self) { self.counts_initialized = true; }
}
```

- [ ] **Step 6: Update `init_value` in `ecma_context.rs` to write value-data only**

In `ecma_context.rs`, `init_value` (lines ~254-303): delete the `for r in self.scoping().get_resolved_references(symbol_id)` count loop and the three `*_references_count` locals; construct `SymbolValue { initialized_constant, exported, is_fresh_value, scope_id }`. Leave the `DirectEval` gating of `initialized_constant` and `exported`/`scope_id` derivation unchanged.

- [ ] **Step 7: Fix all count reads that referenced the moved fields so the crate compiles**

The compiler will flag every `symbol_value.read_references_count` / `.write_references_count` / `.member_write_target_read_count`. For Task 1, temporarily point them at the count store so the crate builds (they get their final form in Task 3): e.g. replace `sv.write_references_count` with `ctx.state.symbol_values.reference_counts(symbol_id).write` at each site. (Sites: `ecma_context.rs:195`, `mod.rs:122`, `remove_unused_expression.rs:730`, `inline.rs:134,138`.)

- [ ] **Step 8: Build + run minifier tests**

Run: `cargo test -p oxc_minifier`
Expected: all pass (counts still computed lazily — see note). minsize unaffected.

> Note: at this point counts are NOT yet maintained or eagerly initialized — `reference_counts()` returns zeros. That is wrong, but it is corrected in Task 1b before any consumer relies on it. To keep steps green, do Task 1b in the SAME commit as Task 1.

### Task 1b: eager-init counts from the live program (same commit as Task 1)

**Files:** `symbol_value.rs`, `crates/oxc_minifier/src/peephole/mod.rs`

- [ ] **Step 1: Add `init_counts_from_program`**

In `symbol_value.rs`:

```rust
impl<'a> SymbolValues<'a> {
    /// Tally reference counts from the live program (self-healing — does not
    /// trust Scoping's reference lists). Called once, post-Normalize.
    pub fn init_counts_from_program(&mut self, program: &Program<'a>, scoping: &Scoping) {
        use oxc_ast_visit::Visit;
        struct Counter<'b> { counts: &'b mut IndexVec<SymbolId, ReferenceCounts>, scoping: &'b Scoping }
        impl<'a, 'b> Visit<'a> for Counter<'b> {
            fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
                let Some(reference_id) = it.reference_id.get() else { return };
                let reference = self.scoping.get_reference(reference_id);
                if let Some(symbol_id) = reference.symbol_id() {
                    self.counts[symbol_id].add(reference.flags());
                }
            }
        }
        for slot in &mut self.counts { *slot = ReferenceCounts::default(); }
        Counter { counts: &mut self.counts, scoping }.visit_program(program);
        self.counts_initialized = true;
    }
}
```

Add the needed `use` for `Program`, `IdentifierReference` (`oxc_ast::ast::*`).

- [ ] **Step 2: Call it once at first `enter_program`**

In `mod.rs` `enter_program`, replace the `dirty.init(...)` line WITH (keep `dirty.init` too for now — shadow phase):

```rust
if !ctx.state.symbol_values.counts_initialized() {
    let TraverseCtx { state, scoping, .. } = ctx;
    state.symbol_values.init_counts_from_program(_program, scoping.scoping());
}
```

(Adjust borrow split to match the existing `enter_program` body; `_program` is the `&mut Program` param — rename to `program`.)

- [ ] **Step 3: Build + test**

Run: `cargo test -p oxc_minifier`
Expected: all pass. Counts now hold correct pass-start values; consumers (Task 1 Step 7) read them. minsize zero-delta (counts equal what the lazy recompute produced at pass start, and nothing yet mutates them mid-pass differently — but see Task 2/3).

- [ ] **Step 4: Commit**

```bash
git add crates/oxc_minifier/src/symbol_value.rs crates/oxc_minifier/src/traverse_context/ecma_context.rs crates/oxc_minifier/src/peephole/mod.rs crates/oxc_minifier/src/peephole/remove_unused_expression.rs crates/oxc_minifier/src/peephole/inline.rs
git commit -m "refactor(minifier): add eager-initialized reference-count store"
```

---

## Task 2: Live maintenance in `DropDiff` + count-drift debug assertion (shadow)

**Files:**

- Modify: `crates/oxc_minifier/src/traverse_context/drop_diff.rs`
- Modify: `crates/oxc_minifier/src/traverse_context/ecma_context.rs:368-372` (`dirty_diff`)
- Modify: `crates/oxc_minifier/src/peephole/mod.rs` (`exit_program`)

- [ ] **Step 1: Point `DropDiff` at the count store**

In `drop_diff.rs`, change the struct to borrow counts instead of (in addition to, this phase) `PassDirty`. For the shadow phase keep `PassDirty` working too, so borrow both via the accessor. Simplest: have `DropDiff` hold `&mut IndexVec<SymbolId, ReferenceCounts>` + `&Scoping` and keep the existing `dead_refs` updates in a parallel `DropDiff` OR fold both. Recommended: fold — `DropDiff` updates BOTH `dead_refs` (existing) and `counts` (new) this phase, so `debug_assert_counts_match` and `debug_assert_no_over_prune` both validate.

Update `visit_identifier_reference`:

```rust
fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
    let Some(reference_id) = it.reference_id.get() else { return };
    let reference = self.scoping.get_reference(reference_id);
    let Some(symbol_id) = reference.symbol_id() else { return }; // unresolved: untracked
    let flags = reference.flags();
    match self.mode {
        DropDiffMode::MarkDead => {
            self.marked = true;
            self.counts[symbol_id].sub(flags);
            // (shadow phase) keep: self.dead_refs.set_bit(reference_id.index());
        }
        DropDiffMode::Resurrect => {
            self.counts[symbol_id].add(flags);
            // (shadow phase) keep: self.dead_refs.unset_bit(reference_id.index());
        }
    }
}
```

Keep `visit_call_expression` (eval detection) unchanged. Keep `resurrect_is_noop`.

- [ ] **Step 2: Update `dirty_diff()` borrow split**

In `ecma_context.rs`:

```rust
fn dirty_diff(&mut self) -> DropDiff<'a, '_> {
    let TraverseCtx { state, scoping, .. } = self;
    DropDiff::new(state.symbol_values.counts_mut(), &mut state.dirty, scoping.scoping())
}
```

(Adjust `DropDiff::new` signature to take both during the shadow phase.)

- [ ] **Step 3: Add `debug_assert_counts_match` and call it in `exit_program`**

In `mod.rs`, add (debug-only):

```rust
#[cfg(debug_assertions)]
fn debug_assert_counts_match(program: &Program<'a>, scoping: &Scoping, counts: &oxc_index::IndexVec<SymbolId, ReferenceCounts>) {
    let mut fresh: oxc_index::IndexVec<SymbolId, ReferenceCounts> = counts.iter().map(|_| ReferenceCounts::default()).collect();
    // re-tally from the live program
    struct C<'b> { fresh: &'b mut oxc_index::IndexVec<SymbolId, ReferenceCounts>, scoping: &'b Scoping }
    impl<'a, 'b> Visit<'a> for C<'b> {
        fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
            if let Some(id) = it.reference_id.get() {
                let r = self.scoping.get_reference(id);
                if let Some(sym) = r.symbol_id() { self.fresh[sym].add(r.flags()); }
            }
        }
    }
    C { fresh: &mut fresh, scoping }.visit_program(program);
    for (sym, c) in counts.iter_enumerated() {
        let f = &fresh[sym];
        assert!(
            c.total == f.total && c.read == f.read && c.write == f.write
                && c.member_write_target == f.member_write_target && c.non_read_only == f.non_read_only,
            "reference-count drift for symbol {sym:?}: maintained {c:?} != live {f:?}",
        );
    }
}
```

Call it at the top of `exit_program` (debug only), before the existing dead_refs handling:

```rust
#[cfg(debug_assertions)]
{
    let TraverseCtx { state, scoping, .. } = ctx;
    Self::debug_assert_counts_match(program, scoping.scoping(), state.symbol_values.counts_ref());
}
```

(Add a small `pub(crate) fn counts_ref(&self) -> &IndexVec<...>` accessor on `SymbolValues` for the assertion.)

- [ ] **Step 4: Build + test (the assertion validates maintenance across the whole corpus)**

Run: `cargo test -p oxc_minifier`
Expected: all pass. If `debug_assert_counts_match` fires, the maintenance has a gap (likely a drop path not routed through a helper, or a mint not incremented) — fix before proceeding. This is the load-bearing validation step.

- [ ] **Step 5: Run conformance with assertion active**

Run: `cargo coverage -- minifier 2>&1 | tail -20`
Expected: no panic, pass counts unchanged vs baseline.

- [ ] **Step 6: Commit**

```bash
git add crates/oxc_minifier/src/traverse_context/drop_diff.rs crates/oxc_minifier/src/traverse_context/ecma_context.rs crates/oxc_minifier/src/peephole/mod.rs crates/oxc_minifier/src/symbol_value.rs
git commit -m "refactor(minifier): maintain reference counts in DropDiff (shadow, asserted)"
```

---

## Task 3: Switch consumers to the count store; migrate the 3 list-readers

**Files:** `ecma_context.rs`, `mod.rs`, `remove_unused_expression.rs`, `inline.rs`, `remove_dead_code.rs`, `substitute_alternate_syntax.rs`

For each site below, the count comes from `ctx.state.symbol_values.reference_counts(symbol_id)`; the value-data `Option` gate (where present) stays.

- [ ] **Step 1: `ecma_context.rs:187-197` `tracked_constant_for_reference_id`**

Read count from store; keep `initialized_constant` from value data:

```rust
fn tracked_constant_for_reference_id(&self, reference_id: ReferenceId) -> Option<&ConstantValue<'a>> {
    let symbol_id = self.scoping().get_reference(reference_id).symbol_id()?;
    if self.state.symbol_values.reference_counts(symbol_id).write != 0 { return None; }
    self.state.symbol_values.get_symbol_value(symbol_id)?.initialized_constant.as_ref()
}
```

- [ ] **Step 2: `mod.rs:120 is_symbol_mutated` — drop the Scoping fallback**

```rust
fn is_symbol_mutated(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
    ctx.state.symbol_values.reference_counts(symbol_id).write > 0
}
```

Update the doc comment (no more fallback). If `Scoping::symbol_is_mutated` has no other caller, leave it (other crates may use it).

- [ ] **Step 3: `remove_unused_expression.rs:723-732`**

Keep value-data gate for `exported`; read `read` from store:

```rust
let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else { return false };
if symbol_value.exported { return false; }
if ctx.state.symbol_values.reference_counts(symbol_id).read > 0 { return false; }
```

- [ ] **Step 4: `inline.rs:130-138`**

```rust
let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else { return; };
if ctx.state.symbol_values.reference_counts(symbol_id).write > 0 { return; }
let Some(cv) = &symbol_value.initialized_constant else { return; };
if ctx.state.symbol_values.reference_counts(symbol_id).read == 1
    || match cv { /* unchanged */ }
```

(Avoid a borrow conflict: copy `let counts = ctx.state.symbol_values.reference_counts(symbol_id);` before binding `symbol_value`, then use `counts.write` / `counts.read`.)

- [ ] **Step 5: `remove_dead_code.rs:489` (list-reader → `non_read_only`)**

```rust
if ctx.state.symbol_values.reference_counts(symbol_id).non_read_only == 0 {
```

- [ ] **Step 6: `substitute_alternate_syntax.rs:902,907,919` (list-reader → `total`)**

```rust
// :902
ctx.state.symbol_values.reference_counts(id).total != e_ref_count
// :907
if ctx.state.symbol_values.reference_counts(a_id_symbol_id).total != a_ref_count {
// :919
(ctx.state.symbol_values.reference_counts(de_id_symbol_id).total > 1)
```

(The `e_ref_count`/`a_ref_count` expectation accumulators at lines 696-881 are unchanged.)

- [ ] **Step 7: Build + run tests; expect possible output changes**

Run: `cargo test -p oxc_minifier`
Expected: PASS, OR a small number of `test(...)` cases whose expected output shifted (fresher mid-pass counts let an optimization fire earlier). For each failure: confirm the new output is equal-or-smaller and semantically equivalent, then update the expected string. Do NOT update a test whose output got _larger_ or changed meaning — that signals a real bug; stop and investigate.

- [ ] **Step 8: Re-snapshot minsize and review**

Run: `just minsize` then `git diff -- tasks/minsize`
Expected: equal-or-smaller sizes only. Review each diff hunk. Commit the snapshot.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "refactor(minifier): read reference counts from the maintained store"
```

---

## Task 4: Make value-data count-free; confirm `init_value` no longer touches lists

**Files:** `ecma_context.rs`, any remaining readers.

- [ ] **Step 1: Confirm no remaining reference to the old count fields**

Run: `rg -n "read_references_count|write_references_count|member_write_target_read_count" crates/oxc_minifier/src`
Expected: only matches inside `ReferenceCounts` (the new `read`/`write`/`member_write_target` fields) and the store — none on the old `SymbolValue`. Fix any stragglers.

- [ ] **Step 2: Confirm `init_value` has no `get_resolved_references` loop**

Run: `rg -n "get_resolved_references" crates/oxc_minifier/src/traverse_context/ecma_context.rs`
Expected: no matches.

- [ ] **Step 3: Build + test**

Run: `cargo test -p oxc_minifier`
Expected: all pass; minsize unchanged vs Task 3.

- [ ] **Step 4: Commit (if any changes)**

```bash
git add -A && git commit -m "refactor(minifier): drop reference counts from per-pass value data"
```

---

## Task 5: Delete the dead-ref bitset, batch retain, and over-prune assertion

**Files:** `state.rs`, `drop_diff.rs`, `ecma_context.rs`, `mod.rs`, `crates/oxc_semantic/src/scoping.rs`

- [ ] **Step 1: Remove `dead_refs` from `DropDiff` and the `PassDirty` plumbing**

In `drop_diff.rs`, delete the `dead_refs` field + its `set_bit`/`unset_bit` lines (keep the count `sub`/`add`). `DropDiff::new` now takes only `(&mut counts, &Scoping)`. Keep `eval_dropped` handling — relocate `eval_dropped` from `PassDirty` to a standalone `MinifierState` field (`pub(crate) eval_dropped: bool`), reset in `enter_program`.

- [ ] **Step 2: Update `state.rs`**

Remove `PassDirty` and `MinifierState::dirty`; add `pub(crate) eval_dropped: bool` (init `false`). Remove the `BitSet` import if now unused.

- [ ] **Step 3: Update `mod.rs` `enter_program` / `exit_program`**

`enter_program`: remove `dirty.init(...)`; reset `eval_dropped = false`; keep the once-only `init_counts_from_program`. `exit_program`: remove the `dead_refs.is_empty()` block, `debug_assert_no_over_prune`, and `retain_resolved_references_excluding`. Keep the `eval_dropped`-gated `LiveDirectEvalCollector` refresh and the `debug_assert_counts_match` call. Delete the `debug_assert_no_over_prune` fn.

- [ ] **Step 4: Remove the now-unused Scoping method**

Run: `rg -n "retain_resolved_references_excluding" crates`
If only the definition remains, delete it from `crates/oxc_semantic/src/scoping.rs`.

- [ ] **Step 5: Build + full test + gates**

Run:

```bash
cargo test -p oxc_minifier
cargo clippy -p oxc_minifier --all-targets
./tools/check_state_changed.sh
(cd crates/oxc_minifier && ast-grep scan)
```

Expected: all green; minsize unchanged vs Task 3 (pure deletion of redundant machinery).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(minifier)!: remove dead-ref bitset and batch list compaction"
```

---

## Task 6: Benchmark gate + final verification

- [ ] **Step 1: Adjacent crates + conformance**

Run:

```bash
cargo test -p oxc_mangler -p oxc_semantic -p oxc_transformer_plugins
cargo coverage -- minifier 2>&1 | tail -20
```

Expected: green; no conformance regression; no panic (count-drift assertion active under coverage profile).

- [ ] **Step 2: Single-file minifier benchmark vs baseline**

Run the minifier benchmark on `kitchen-sink.tsx`, `cal.com.tsx`, `binder.ts`, `react.development.js` (use the repo's bench harness, e.g. `cargo bench -p oxc_minifier` or the tasks/benchmark setup). Compare to the pre-Task-1 commit.
Expected (gate): neutral-or-better overall; a win on `kitchen-sink.tsx`/bundler-shaped inputs. If it regresses without a compensating win, `git revert` the Task 1-5 range (spec §9) and stop.

- [ ] **Step 3: Update the spec status + note results**

Mark the spec `Status: implemented`, record benchmark numbers.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/specs/2026-06-09-minifier-live-reference-counts-design.md
git commit -m "docs(minifier): record live-reference-counts benchmark results"
```

---

## Self-Review notes (for the executor)

- **Borrow conflicts** are the most likely friction (counts live on `state.symbol_values`, and helpers also touch `state.dirty`/`scoping`). Use field-level destructuring (`let TraverseCtx { state, scoping, .. } = self;`) as the existing `dirty_diff()` does.
- **`is_read_only` exactness**: only `non_read_only == 0` is correct for `remove_dead_code.rs:489`; do not substitute `write == 0`.
- **Do not update a test whose output grew or changed meaning** in Task 3 Step 7 — that is a real bug, not an expected re-snapshot.
- **The count-drift assertion is the safety net.** If it never fires across `cargo test` + `cargo coverage`, the maintenance is sound; if it fires, a drop/mint path is unrouted — fix the path, don't weaken the assertion.
