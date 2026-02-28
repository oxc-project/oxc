# `oxc_module_graph` — Cross-Module Analysis Crate (Trait-Based)

## Context

Rolldown and similar bundlers need cross-module analysis: knowing which module imports what from which, resolving imports to exports, and linking symbols across module boundaries. Today, oxc provides excellent **per-file** analysis (`oxc_parser`, `oxc_semantic`), while Rolldown has its own tightly-coupled implementations (`EcmaView`, `SymbolRefDb`, `LinkStage`). The linter also has its own ad-hoc version (`oxc_linter::module_record`).

This crate provides **trait-based abstractions** and **algorithms** for cross-module analysis. Rolldown can implement the traits with its own types and adopt incrementally. Other tools get a batteries-included default implementation.

**Provides**: Trait interfaces + graph algorithms + import-to-export binding + default implementations.
**Does NOT provide**: Tree-shaking, CJS interop, chunk splitting — consumers layer those on top.

## Architecture: Trait-Based Design

The key insight: **separate the algorithms from the data structures**. Define traits for what the algorithms need, provide default implementations, let Rolldown plug in its own types.

All traits use **associated types** for `ModuleIdx` and `SymbolRef`, and **callback-based iteration** (`for_each_*`) instead of returning concrete collection references. This allows consumers (like Rolldown) to implement the traits directly on their own types without needing to match oxc_module_graph's exact collection or index types.

```
┌──────────────────────────────────────────────────────────────┐
│                      oxc_module_graph                         │
│                                                               │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐  │
│  │     Traits      │  │   Algorithms   │  │    Defaults    │  │
│  │                │  │                │  │                │  │
│  │ ModuleInfo     │◄─┤ bind_imports   │  │ Module         │  │
│  │ ModuleStore    │◄─┤ match_imports  │  │ ModuleGraph    │  │
│  │ SymbolGraph    │◄─┤ exec_order     │  │ SymbolRefDb    │  │
│  │ ImportMatcher  │◄─┤ find_cycles    │  │ Builder        │  │
│  │ SideEffects    │◄─┤ tla            │  │                │  │
│  │ Checker        │  │ side_effects   │  │                │  │
│  │                │  │ dynamic_exports│  │                │  │
│  └────────────────┘  └────────────────┘  └────────────────┘  │
│         ▲                                       ▲            │
└─────────┼───────────────────────────────────────┼────────────┘
          │                                       │
    Rolldown implements                    Other tools use
    traits with own types                  default impls
```

## Crate Layout

```
crates/oxc_module_graph/
  Cargo.toml
  src/
    lib.rs              -- Re-exports, crate docs

    # Core traits (the stable API contract)
    traits/
      mod.rs
      module_info.rs    -- ModuleInfo trait: read parse-time data
      symbol_graph.rs   -- SymbolGraph trait: cross-module symbol linking
      module_store.rs   -- ModuleStore trait: indexed module collection
      import_matcher.rs -- ImportMatcher trait: consumer-specific import handling
      side_effects_checker.rs -- SideEffectsChecker trait: consumer-specific side effects

    # Shared types (used by both traits and impls)
    types/
      mod.rs
      module_idx.rs     -- ModuleIdx newtype index
      symbol_ref.rs     -- SymbolRef = (ModuleIdx, SymbolId)
      module_record.rs  -- Owned ModuleRecord (converted from parser)
      import_export.rs  -- NamedImport, LocalExport, ResolvedExport, ImportKind, MatchImportKind
      error.rs          -- Error types

    # Algorithms (generic over traits)
    algo/
      mod.rs
      binding.rs        -- bind_imports_and_exports, build_resolved_exports, match_imports
      cycles.rs         -- find_cycles<M: ModuleStore>()
      exec_order.rs     -- compute_exec_order<M: ModuleStore>()
      tla.rs            -- compute_tla<M: ModuleStore>()
      side_effects.rs   -- determine_side_effects<M: ModuleStore, C: SideEffectsChecker>()
      dynamic_exports.rs -- compute_has_dynamic_exports<M: ModuleStore>()

    # Default implementations (batteries-included)
    default/
      mod.rs
      module.rs         -- Default Module struct
      graph.rs          -- Default ModuleGraph implementing ModuleStore
      symbol_db.rs      -- Default SymbolRefDb implementing SymbolGraph
      builder.rs        -- ModuleGraphBuilder (parse → semantic → resolve → BFS)

  tests/
    fixtures/           -- JS/TS fixture files
    integration.rs
```

## Core Traits

### `ModuleInfo` — Read parse-time data from a module

```rust
pub trait ModuleInfo {
    type ModuleIdx: Copy + Eq + Hash + Debug;
    type SymbolRef: Copy + Eq + Hash + Debug;

    fn module_idx(&self) -> Self::ModuleIdx;
    fn default_export_ref(&self) -> Self::SymbolRef;
    fn namespace_object_ref(&self) -> Self::SymbolRef;
    fn has_module_syntax(&self) -> bool;
    fn is_commonjs(&self) -> bool;
    fn has_top_level_await(&self) -> bool;
    fn side_effects(&self) -> Option<bool>;

    // Callback-based iteration (no concrete collection types exposed)
    fn for_each_named_export(&self, f: &mut dyn FnMut(&str, Self::SymbolRef, bool));
    fn for_each_named_import(&self, f: &mut dyn FnMut(Self::SymbolRef, &str, usize, bool));
    fn for_each_star_export(&self, f: &mut dyn FnMut(Self::ModuleIdx));
    fn for_each_esm_star_export(&self, f: &mut dyn FnMut(Self::ModuleIdx)); // default: delegates to for_each_star_export
    fn for_each_indirect_export(&self, f: &mut dyn FnMut(&str, &str, Self::ModuleIdx));
    fn for_each_import_record(&self, f: &mut dyn FnMut(usize, Option<Self::ModuleIdx>, ImportKind));

    fn import_record_count(&self) -> usize;
    fn import_record_resolved_module(&self, idx: usize) -> Option<Self::ModuleIdx>;
    fn symbol_import_info(&self, symbol: Self::SymbolRef) -> Option<(&str, usize, bool)>;
}
```

### `ModuleStore` — Indexed collection of modules

```rust
pub trait ModuleStore {
    type ModuleIdx: Copy + Eq + Hash + Debug;
    type SymbolRef: Copy + Eq + Hash + Debug;
    type Module: ModuleInfo<ModuleIdx = Self::ModuleIdx, SymbolRef = Self::SymbolRef>;

    fn module(&self, idx: Self::ModuleIdx) -> Option<&Self::Module>;
    fn modules_len(&self) -> usize;
    fn for_each_module(&self, f: &mut dyn FnMut(Self::ModuleIdx, &Self::Module));
    fn for_each_dependency(&self, idx: Self::ModuleIdx, f: &mut dyn FnMut(Self::ModuleIdx));
    fn for_each_static_dependency(&self, idx: Self::ModuleIdx, f: &mut dyn FnMut(Self::ModuleIdx));

    /// Query side-effects for any module (including externals not returned by `module()`).
    fn any_module_side_effects(&self, idx: Self::ModuleIdx) -> Option<Option<bool>> {
        self.module(idx).map(|m| m.side_effects())
    }
}
```

### `SymbolGraph` — Cross-module symbol linking

```rust
pub trait SymbolGraph {
    type ModuleIdx: Copy + Eq + Hash + Debug;
    type SymbolRef: Copy + Eq + Hash + Debug;

    fn canonical_ref_for(&self, symbol: Self::SymbolRef) -> Self::SymbolRef;
    fn link(&mut self, from: Self::SymbolRef, to: Self::SymbolRef);
    fn symbol_name(&self, symbol: Self::SymbolRef) -> &str;
    fn symbol_owner(&self, symbol: Self::SymbolRef) -> Self::ModuleIdx;
}
```

### `ImportMatcher` — Consumer-specific import handling

```rust
pub trait ImportMatcher {
    type ModuleIdx: Copy + Eq + Hash + Debug;
    type SymbolRef: Copy + Eq + Hash + Debug;

    fn on_missing_module(...) -> Option<MatchImportKind<Self::SymbolRef>>; // External modules
    fn on_before_match(...) -> Option<MatchImportKind<Self::SymbolRef>>;   // CJS short-circuit
    fn on_no_match(...) -> Option<MatchImportKind<Self::SymbolRef>>;       // Dynamic fallback
    fn on_cjs_match(...) -> Option<MatchImportKind<Self::SymbolRef>>;      // CJS export override
    fn on_resolved(...);                                                    // Post-resolution callback
}
```

### `SideEffectsChecker` — Consumer-specific side-effects logic

```rust
pub trait SideEffectsChecker {
    type ModuleIdx: Copy + Eq + Hash + Debug;

    /// Check if a star-export edge introduces side effects (wrapping, dynamic exports).
    fn star_export_has_side_effects(&self, importer: Self::ModuleIdx, importee: Self::ModuleIdx) -> bool;
}
```

## Shared Types

### `ImportKind`

```rust
pub enum ImportKind {
    Static,     // import ... from '...'
    Dynamic,    // import('...')
    Require,    // require('...')
    HotAccept,  // import.meta.hot.accept('...') — HMR-only, not a graph edge
}
```

### `MatchImportKind`

```rust
pub enum MatchImportKind<S> {
    Normal { symbol_ref: S },
    Namespace { namespace_ref: S },
    NormalAndNamespace { namespace_ref: S, alias: CompactString }, // CJS/dynamic fallback
    Ambiguous { candidates: Vec<S> },
    Cycle,
    NoMatch,
}
```

### `ResolvedExport`

```rust
pub struct ResolvedExport<S> {
    pub symbol_ref: S,
    pub potentially_ambiguous: Option<Vec<S>>,
    pub came_from_cjs: bool,
}
```

## Algorithms (Generic Over Traits)

### `build_resolved_exports` — Phase 1: resolve exports

Builds resolved exports for all modules. Initializes from local exports, then propagates star re-exports with proper shadowing, CJS semantics, and ambiguity detection.

### `match_imports` — Phase 2: match imports to exports

Matches each import to the resolved exports, calling `ImportMatcher` callbacks for consumer-specific behavior (CJS, externals, dynamic fallback). Links symbols via `SymbolGraph::link()`.

### `bind_imports_and_exports` — Combined Phase 1 + 2

Convenience function that calls `build_resolved_exports` then `match_imports` with a `DefaultImportMatcher`.

### `compute_exec_order` — DFS post-order execution sort

Returns modules in JavaScript evaluation order. Handles runtime module, static/dynamic/require edges, cycle detection with full cycle paths. Skips `HotAccept` edges.

### `compute_tla` — Top-level await propagation

Identifies modules affected by top-level `await`, propagating through static import edges only.

### `determine_side_effects` — Side-effects propagation

Propagates side-effects status through import and ESM star-export edges. Uses `any_module_side_effects()` for external modules and `SideEffectsChecker` for consumer-specific logic (wrapping, dynamic exports).

### `compute_has_dynamic_exports` — Dynamic export detection

Identifies modules whose exports are not statically analyzable (transitively `export *` from CJS or external).

### `find_cycles` — Cycle detection

DFS-based cycle detection returning all cycles as lists of module indices.

## Rolldown Integration

Rolldown implements the traits on its existing types — **no data structure changes needed**:

| Rolldown Type | Implements | Notes |
|---------------|-----------|-------|
| `NormalModule` | `ModuleInfo` | Bridges `EcmaView` fields to trait methods |
| `ModuleTable` | `ModuleStore` | `module()` returns `None` for external modules |
| `SymbolRefDb` | `SymbolGraph` | Direct delegation |
| `RolldownImportMatcher` | `ImportMatcher` | CJS interop, external modules, dynamic fallback |
| `RolldownSideEffectsChecker` | `SideEffectsChecker` | WrapKind + has_dynamic_exports checks |

### Algorithms adopted by Rolldown

| Algorithm | Replaces | Status |
|-----------|----------|--------|
| `build_resolved_exports` + `match_imports` | `bind_imports_and_exports.rs` | Adopted |
| `compute_exec_order` | `sort_modules.rs` | Adopted |
| `compute_tla` | `compute_tla.rs` | Adopted |
| `determine_side_effects` | `determine_side_effects.rs` | Adopted |

## Verification

```bash
cargo test -p oxc_module_graph        # 45 tests passing
cargo clippy -p oxc_module_graph      # Clean
just fmt
```
