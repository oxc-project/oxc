# oxc_module_graph Performance & Adoption Refactor

## Context

The ROLLDOWN_ADOPTION_PLAN.md defines the end-state architecture where Rolldown deletes `oxc_bridge` and uses `oxc_module_graph::ModuleGraph` as the sole source of truth. For that to work well, `oxc_module_graph` itself needs to be faster and easier to integrate. This plan covers the concrete changes to make in `oxc_module_graph` right now to prepare for that adoption.

**Key problems today:**

1. Rolldown maintains TWO parallel SymbolRefDbs — the root cause of all integration friction
2. The graph is reconstructed from Rolldown's ModuleTable then freed after linking — pure waste
3. Binding algorithm creates ~36MB of temporary allocations per link pass (500K FxHashSet + Vec allocations)
4. No `flatten_all_chains()` — immutable `canonical_ref` traverses full chains in generate-stage hot loops
5. `NormalModule` construction requires 25 fields of boilerplate
6. `ImportHooks::on_resolved` lacks context Rolldown needs (record_idx, imported_name, target_module)

**Goal:** Make `oxc_module_graph` fast enough and ergonomic enough that Rolldown can use it as THE canonical graph without performance regression.

---

## Stage 0: Baseline Benchmarks

**Goal**: Establish performance baselines before any changes
**Status**: Complete

Add criterion benchmarks:

- `bench_canonical_ref` — chain traversal at depths 1, 5, 10, 50
- `bench_link` — 500K link operations
- `bench_flatten` — flatten_all_chains on 500K symbols
- `bench_build_resolved_exports` — 1000 modules, star-export chains
- `bench_match_imports` — 1000 modules, 10 imports each

---

## Stage 1: SymbolRefDb Hot Path Optimization

**Goal**: Improve cache locality and reduce allocations in symbol resolution
**Files**: `src/default/symbol_db.rs`, `src/graph.rs`
**Status**: Complete

### 1a. Split hot/cold data

Separate `links` (hot) from `names` (cold) in SymbolRefDb.

### 1b. String to CompactString for symbol names

Change `IndexVec<SymbolId, String>` to `IndexVec<SymbolId, CompactString>`.

### 1c. Re-add `flatten_all_chains()`

Full path compression so all `canonical_ref_for()` calls are O(1) post-flatten.

### 1d. Bulk symbol initialization

`init_module_symbols()` replaces N calls with one allocation.

### 1e. Early-exit in `link()`

Skip no-op links (from == to, already linked).

---

## Stage 2: Algorithm Allocation Reduction

**Goal**: Eliminate ~36MB of temporary allocations in binding phase
**Files**: `src/algo/binding.rs`
**Status**: Complete

### 2a. Hoist per-import allocations in `match_imports_collect`

Reuse FxHashSet and Vec across iterations.

### 2b. Hoist per-module allocation in `build_resolved_exports`

Reuse visited set across modules.

### 2c. Pre-size with capacity hints

---

## Stage 3: API Ergonomics

**Goal**: Reduce integration boilerplate for Rolldown
**Files**: `src/module.rs`, `src/hooks.rs`, `src/algo/*.rs`, `src/graph.rs`
**Status**: Complete

### 3a. `NormalModule::new()` constructor

4-field constructor with defaults for everything else.

### 3b. Algorithm result `apply()` methods

Add `apply(self, graph: &mut ModuleGraph)` to result types.

### 3c. Richer `ImportHooks::on_resolved` context

Replace individual parameters with `ImportResolutionContext` struct.

---

## Stage 4: Structural Improvements

**Goal**: Reduce memory usage
**Files**: `src/types/import_export.rs`, `src/module.rs`
**Status**: Complete

### 4a. Slim `ResolvedExport`

Box the rare `potentially_ambiguous` field.

### 4b. Box `NormalModule` in `Module` enum

Shrink enum from ~400+ bytes to ~80 bytes.

---

## Implementation Order

```
Stage 0: Baseline benchmarks   → measure before anything changes
Stage 1 + 2 (in parallel)      → highest performance impact
Stage 1 depends on Stage 0 (baselines)
Stage 2 depends on Stage 0 (baselines)
Stage 3                         → unblocks Rolldown integration
Stage 4                         → structural polish
Re-run benchmarks               → verify gains
```

## Verification

After each stage:

1. `cargo test -p oxc_module_graph` — all existing tests pass
2. `cargo check -p oxc_module_graph` — no warnings
3. Benchmark comparison against baseline

After all stages: 4. `just ready` — full project checks pass 5. Benchmark suite shows cumulative improvement

## Expected Cumulative Results

| Metric                     | Before         | After                                             |
| -------------------------- | -------------- | ------------------------------------------------- |
| canonical_ref post-link    | O(chain_depth) | O(1) after flatten                                |
| Binding temp memory        | ~36MB          | ~0 (reused)                                       |
| ResolvedExport size        | 40 bytes       | 24 bytes                                          |
| Module enum size           | ~400+ bytes    | ~80 bytes                                         |
| Symbol name heap allocs    | 1 per symbol   | 0 for names <= 24 chars                           |
| NormalModule constructor   | 25 lines       | 4 lines                                           |
| graph.link() body          | ~100 lines     | ~30 lines                                         |
| ImportHooks context fields | 4              | 7 (adds record_idx, imported_name, target_module) |
