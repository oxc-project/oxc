# Rolldown Adoption Plan for `oxc_module_graph`

## Summary

This document defines the refactor needed to remove Rolldown's `oxc_bridge` and
adopt `oxc_module_graph` as the canonical link-time graph kernel, while keeping
`oxc_module_graph` generic and free of Rolldown-specific logic.

The only viable end state is:

- `oxc_module_graph::ModuleGraph` owns all generic link-time graph state.
- Rolldown keeps all bundler-specific behavior and heavy payloads in sidecars.
- Scan stage populates the canonical graph directly.
- Link stage operates on the canonical graph directly.
- No graph reconstruction or `ModuleIdx` / `SymbolRef` conversion layer remains.

This refactor should be accepted only if it preserves or improves performance.

## Constraints

- `oxc_module_graph` is a shared crate for multiple tools.
- Rolldown-specific semantics must remain in Rolldown.
- `oxc_module_graph` may gain only generic, tool-agnostic APIs and performance
  improvements.
- No remapping layer for parser-produced `SymbolId` is acceptable in the final
  design.
- Temporary migration adapters are allowed, but `ModuleTable` and `oxc_bridge`
  must not remain as permanent parallel abstractions.

## Goals

- Delete Rolldown's `oxc_bridge`.
- Eliminate temporary `ModuleGraph` reconstruction in link stage.
- Make `oxc_module_graph::ModuleGraph` the sole source of truth for link-time
  graph state.
- Preserve all Rolldown-specific import/link behavior in Rolldown-owned code.
- Reach parity or better performance versus the current Rolldown linker path.

## Non-Goals

- Do not move tree-shaking, chunking, rendering, plugin behavior, or HMR into
  `oxc_module_graph`.
- Do not make `oxc_module_graph` generic over large bundler payloads.
- Do not move Rolldown-specific diagnostics formatting or output semantics into
  `oxc_module_graph`.

## Target Architecture

Rolldown should introduce a new internal kernel wrapper:

```rust
pub struct LinkKernel {
    pub graph: oxc_module_graph::ModuleGraph,
    pub module_payloads: IndexVec<ModuleIdx, Option<ModulePayload>>,
    pub symbol_extras: IndexVec<ModuleIdx, Option<SymbolExtrasForModule>>,
}
```

Where:

- `graph` holds only generic link-relevant data.
- `module_payloads` holds Rolldown-only heavy module data.
- `symbol_extras` holds Rolldown-only per-symbol metadata.

### Canonical ownership in `oxc_module_graph`

The canonical graph should own:

- `ModuleIdx` allocation
- `SymbolRef` identity
- import records
- named imports
- named exports
- indirect exports
- star exports
- `default_export_ref`
- `namespace_object_ref`
- `has_module_syntax`
- `exports_kind`
- `has_top_level_await`
- `has_lazy_export`
- `execution_order_sensitive`
- `side_effects`
- `wrap_kind`
- `original_wrap_kind`
- `wrapper_ref`
- `required_by_other_module`
- `resolved_exports`
- `has_dynamic_exports`
- `is_tla_or_contains_tla`
- `propagated_side_effects`
- `exec_order`
- cross-module symbol linking and canonicalization

### Rolldown sidecars

Rolldown should keep these outside `oxc_module_graph`:

- source text
- AST and `EcmaView` payload not needed for generic linking
- `ModuleId`, `StableModuleId`, display/debug names
- sourcemap chain
- render mutations and codegen state
- HMR state
- tree-shaking-only metadata
- chunk assignment
- plugin-specific bookkeeping
- `namespace_alias`
- symbol flags
- facade-symbol provenance
- `normal_symbol_exports_chain_map`
- external import namespace/binding mergers
- shimmed missing export maps
- Rolldown diagnostics aggregation

## Required Generic Changes in `oxc_module_graph`

These changes are allowed because they are generic and improve the shared kernel.

### 1. Upgrade `SymbolRefDb` for performance

The current `default::SymbolRefDb` is too minimal to be the canonical hot path.

Required changes:

- add path halving or full path compression for canonicalization
- add a mutable fast path similar to Rolldown's current `find_mut`
- optionally add union-by-rank or union-by-size if canonical stability is
  preserved
- keep storage dense and module-local

### 2. Support parser-produced symbol slots directly

The canonical DB must adopt existing semantic `SymbolId`s without remapping.

Add APIs equivalent to:

```rust
pub fn ensure_module_symbol_capacity(&mut self, module: ModuleIdx, len: usize);
pub fn set_symbol_name(&mut self, module: ModuleIdx, symbol: SymbolId, name: String);
pub fn init_symbol_self_link(&mut self, module: ModuleIdx, symbol: SymbolId);
```

`add_symbol` can remain for synthetic linker-created symbols.

### 3. Support generic synthetic symbol allocation

Add a generic helper:

```rust
pub fn alloc_synthetic_symbol(&mut self, module: ModuleIdx, name: String) -> SymbolRef;
```

This supports wrappers, synthetic namespaces, and shim-like linker-created
symbols without embedding consumer-specific policy.

### 4. Tighten direct graph-build APIs

`ModuleGraph` should support direct scan-stage construction cleanly:

- retain `alloc_module_idx`
- add `reserve_modules`
- keep `add_normal_module`
- keep `add_external_module`
- allow efficient placeholder replacement

### 5. Keep hooks narrow and generic

The existing hook design is directionally correct.

Allowed changes:

- extend `ImportHooks::on_resolved` only if more generic context is necessary
- add a minimal generic unresolved-import callback only if current hooks are
  insufficient

Not allowed:

- Rolldown-specific CJS output behavior
- chunking/render semantics
- plugin-specific policy

## Required Rolldown Refactor

## 1. Introduce `LinkKernel`

Add `LinkKernel` as the canonical scan-to-link handoff object.

`NormalizedScanStageOutput` should eventually change from:

```rust
pub struct NormalizedScanStageOutput {
    pub module_table: ModuleTable,
    pub symbol_ref_db: SymbolRefDb,
    ...
}
```

To:

```rust
pub struct NormalizedScanStageOutput {
    pub link_kernel: LinkKernel,
    ...
}
```

During migration, temporary compatibility fields are acceptable, but the final
state must use `link_kernel` as the source of truth.

## 2. Split Rolldown module data into graph core vs payload

Rolldown's current `NormalModule` is too heavy to be the canonical graph node.

Refactor it into:

- graph-core data that populates `oxc_module_graph::NormalModule`
- a `ModulePayload` sidecar for all non-generic data

This split should happen in scan-stage normalization, not by rebuilding from a
full Rolldown module later in link stage.

## 3. Replace canonical linking ownership in Rolldown `SymbolRefDb`

Rolldown's current `SymbolRefDb` combines:

- AST scoping
- symbol flags
- namespace alias state
- chunk metadata
- cross-module linking

Only the last responsibility belongs in the canonical graph.

Final state:

- `oxc_module_graph::SymbolRefDb` owns canonical linking
- Rolldown keeps `SymbolExtrasForModule` for all extra symbol data

`SymbolExtrasForModule` should own:

- `AstScopes`
- `flags`
- `namespace_alias`
- `chunk_idx`
- facade-symbol bookkeeping
- debug-only metadata

## 4. Make scan stage populate the canonical graph directly

This is the structural turning point.

For each module in scan stage:

- allocate `ModuleIdx` from `link_kernel.graph`
- initialize canonical symbol slots in `graph.symbols` from semantic analysis
- build `oxc_module_graph::NormalModule` or `ExternalModule` directly
- store heavy Rolldown data in `module_payloads`
- store local symbol metadata in `symbol_extras`

Critical rule:

- there must be exactly one `ModuleIdx` and `SymbolRef` universe

The bridge disappears only when scan stage creates the canonical graph directly.

## 5. Refactor `LinkStage` to own `LinkKernel`

`LinkStage` should move from:

- `module_table`
- `symbols`

To:

- `link_kernel`

Final shape:

```rust
pub struct LinkStage<'a> {
    pub link_kernel: LinkKernel,
    pub entries: FxIndexMap<ModuleIdx, Vec<EntryPoint>>,
    pub sorted_modules: Vec<ModuleIdx>,
    pub metas: LinkingMetadataVec,
    pub warnings: Vec<BuildDiagnostic>,
    pub errors: Vec<BuildDiagnostic>,
    pub ast_table: IndexEcmaAst,
    pub options: &'a SharedOptions,
    ...
}
```

Access rules:

- use `link_kernel.graph` for canonical graph facts
- use `link_kernel.module_payloads` for Rolldown-only module payload
- use `link_kernel.symbol_extras` for Rolldown-only symbol metadata

## 6. Keep Rolldown-specific linking semantics in Rolldown

Rolldown-specific logic must remain outside `oxc_module_graph`, even after
adoption.

This includes:

- namespace alias creation
- shim missing exports behavior
- `normal_symbol_exports_chain_map`
- external binding/namespace merging
- member-expression resolution
- CJS-specific output-shape adjustments
- Rolldown-specific diagnostics

### `bind_imports_and_exports` target state

Rolldown should still use shared generic algorithms:

- `build_resolved_exports`
- `match_imports_collect`

But they must run against the live canonical graph:

```rust
build_resolved_exports(&self.link_kernel.graph);
match_imports_collect(&self.link_kernel.graph, &mut config);
```

Rolldown's `ImportHooks` implementation remains responsible for collecting and
applying:

- namespace alias writes
- shim requests
- Rolldown diagnostics
- side maps like `normal_symbol_exports_chain_map`

The difference is ownership, not semantics: no temporary graph reconstruction,
no conversion layer.

## Migration Phases

## Phase 0: Baseline and measurement

Before major structural changes:

- record current timings for:
  - scan stage
  - `sort_modules`
  - `determine_module_exports_kind`
  - `wrap_modules`
  - `bind_imports_and_exports`
  - total link stage
- preserve a representative benchmark corpus:
  - large ESM graph
  - mixed ESM/CJS graph
  - many external modules
  - deep re-export chains
  - high symbol-count graphs

Acceptance gate:

- baseline numbers are captured and rerunnable

## Phase 1: Strengthen `oxc_module_graph`

Land only generic kernel changes first:

- `SymbolRefDb` performance upgrades
- parser-symbol adoption APIs
- synthetic symbol allocation API
- minimal generic hook expansion if truly required

Acceptance gate:

- `cargo test -p oxc_module_graph` passes
- microbenchmarks for symbol linking are at least parity

## Phase 2: Introduce `LinkKernel` in Rolldown

Add `LinkKernel` without changing final behavior yet.

Temporary state:

- construct `LinkKernel`
- keep legacy `ModuleTable` / legacy `SymbolRefDb` as compatibility layers
- assert identity equivalence where practical

Acceptance gate:

- no behavior change
- compatibility assertions hold on fixture coverage

## Phase 3: Make scan stage emit the canonical kernel

Shift scan stage to populate `LinkKernel.graph` directly.

Temporary compatibility is still allowed, but:

- legacy `ModuleTable` may be derived from canonical data, never the reverse
- canonical symbol linking must stop being owned by the legacy Rolldown DB

Acceptance gate:

- scan output is canonicalized around `link_kernel`
- no symbol remap layer exists

## Phase 4: Port graph-oriented link passes

Port these stages first:

- `sort_modules`
- `compute_tla`
- `determine_module_exports_kind`
- `determine_safely_merge_cjs_ns`
- `wrap_modules`
- `determine_side_effects`

Acceptance gate:

- these stages read/write graph facts only through `link_kernel.graph`

## Phase 5: Port `bind_imports_and_exports` and delete `oxc_bridge`

This is the main milestone.

Refactor `bind_imports_and_exports` to:

- use `build_resolved_exports(&self.link_kernel.graph)`
- use `match_imports_collect(&self.link_kernel.graph, &mut config)`
- apply symbol links directly into the canonical graph
- write Rolldown-only side effects into Rolldown metadata/sidecars

Then delete:

- `crates/rolldown/src/stages/link_stage/oxc_bridge.rs`

Acceptance gate:

- no temporary `ModuleGraph` is built
- current targeted CJS, shim, and external-binding fixtures still pass

## Phase 6: Port downstream symbol consumers

Port all downstream users of canonical symbol state to the new split:

- export creation
- reference collection
- cross-module optimization
- inclusion logic
- dependency patching
- member-expression resolution

Acceptance gate:

- no production code reads canonical links from legacy Rolldown `SymbolRefDb`

## Phase 7: Delete legacy duplicated structures

Remove migration-only compatibility layers:

- `ModuleTable` from the link pipeline
- cross-module linking ownership from Rolldown `SymbolRefDb`
- conversion helpers and compatibility adapters

Acceptance gate:

- no production link-stage code depends on `ModuleTable`
- no production link-stage code depends on legacy canonical symbol-link APIs

## Internal API Changes

## `oxc_module_graph`

Add:

- `SymbolRefDb::ensure_module_symbol_capacity`
- `SymbolRefDb::set_symbol_name`
- `SymbolRefDb::init_symbol_self_link`
- `SymbolRefDb::alloc_synthetic_symbol`
- mutable compressed canonicalization API

Possibly adjust:

- `ImportHooks::on_resolved` signature, but only for generic context

Do not add:

- Rolldown-specific enums or callbacks
- rendering/chunk/plugin concepts

## Rolldown

Add:

- `LinkKernel`
- `ModulePayload`
- `SymbolExtrasForModule`

Change:

- `NormalizedScanStageOutput` to carry `link_kernel`
- `LinkStage` to own `link_kernel`

Delete by the end:

- link-stage dependency on `ModuleTable`
- canonical-link ownership in Rolldown `SymbolRefDb`
- `oxc_bridge`

## Testing

## `oxc_module_graph` tests

Add or expand tests for:

- symbol slot adoption from existing `SymbolId`
- path compression/halving correctness
- synthetic symbol allocation
- canonicalization stability after repeated links
- import/export resolution across direct, indirect, star, and ambiguous exports

## Rolldown integration coverage

Required fixture scenarios:

- missing export errors
- `shim_missing_exports`
- CJS namespace/default imports
- `NormalAndNamespace` alias retention
- safely merged CJS namespace reexports
- external namespace imports in ESM output
- deep re-export chains across mixed module types
- member-expression resolution through namespace chains

## Required validation commands

- `cargo test -p oxc_module_graph`
- `cargo check -p rolldown -p rolldown_common`
- targeted Rolldown integration fixtures during migration
- full `cargo test -p rolldown --test integration_rolldown` before final cleanup

## Performance Acceptance Criteria

This refactor is only worth landing if it is neutral or positive on performance.

Minimum acceptance:

- `bind_imports_and_exports` is not slower than baseline beyond noise
- total link-stage time is at least parity
- peak memory does not regress materially after duplicate graph state is removed
- symbol canonicalization throughput is at least parity with the current Rolldown
  implementation

Expected wins:

- no temporary graph construction
- no conversion of `ModuleIdx` / `SymbolRef`
- less duplicated link-time state
- better locality for shared graph algorithms

## Explicit Assumptions

- `oxc_module_graph` may evolve, but only with generic, tool-agnostic changes.
- Rolldown-specific logic must remain in Rolldown-owned code.
- `ModuleGraph` will become the sole canonical owner of link-time graph state.
- parser-produced `SymbolId` remains the canonical symbol identity.
- temporary compatibility layers are acceptable only during migration.
- final cleanup must delete `oxc_bridge` and remove `ModuleTable` from the
  link pipeline.

## Final Acceptance Criteria

The migration is complete only when all of the following are true:

- `oxc_bridge` is deleted
- scan stage builds the canonical graph directly
- link stage uses `LinkKernel.graph` as the sole graph truth
- Rolldown-specific logic remains outside `oxc_module_graph`
- no symbol-ID remapping layer exists
- `ModuleTable` is removed from the link pipeline
- `cargo test -p oxc_module_graph` passes
- full Rolldown integration coverage passes
- benchmarks are at least parity, with expected wins in link-stage overhead
