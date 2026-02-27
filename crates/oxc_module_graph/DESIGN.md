# `oxc_module_graph` — Cross-Module Analysis Crate (Trait-Based)

## Context

Rolldown and similar bundlers need cross-module analysis: knowing which module imports what from which, resolving imports to exports, and linking symbols across module boundaries. Today, oxc provides excellent **per-file** analysis (`oxc_parser`, `oxc_semantic`), while Rolldown has its own tightly-coupled implementations (`EcmaView`, `SymbolRefDb`, `LinkStage`). The linter also has its own ad-hoc version (`oxc_linter::module_record`).

This crate provides **trait-based abstractions** and **algorithms** for cross-module analysis. Rolldown can implement the traits with its own types and adopt incrementally. Other tools get a batteries-included default implementation.

**Provides**: Trait interfaces + graph algorithms + import-to-export binding + default implementations.
**Does NOT provide**: Tree-shaking, CJS interop, chunk splitting — consumers layer those on top.

## Architecture: Trait-Based Design

The key insight: **separate the algorithms from the data structures**. Define traits for what the algorithms need, provide default implementations, let Rolldown plug in its own types.

```
┌─────────────────────────────────────────────────────┐
│                 oxc_module_graph                     │
│                                                      │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │   Traits     │  │  Algorithms  │  │  Defaults   │  │
│  │             │  │              │  │             │  │
│  │ ModuleInfo  │◄─┤ bind_imports │  │ Module      │  │
│  │ SymbolGraph │◄─┤ topo_sort    │  │ ModuleGraph │  │
│  │ ModuleStore │◄─┤ find_cycles  │  │ SymbolRefDb │  │
│  │             │  │              │  │ Builder     │  │
│  └─────────────┘  └──────────────┘  └────────────┘  │
│         ▲                                  ▲         │
└─────────┼──────────────────────────────────┼─────────┘
          │                                  │
    Rolldown implements               Other tools use
    traits with own types             default impls
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

    # Shared types (used by both traits and impls)
    types/
      mod.rs
      module_idx.rs     -- ModuleIdx newtype index
      symbol_ref.rs     -- SymbolRef = (ModuleIdx, SymbolId)
      module_record.rs  -- Owned ModuleRecord (converted from parser)
      import_export.rs  -- NamedImport, LocalExport, ResolvedExport, ImportEdge
      error.rs          -- Error types

    # Algorithms (generic over traits)
    algo/
      mod.rs
      binding.rs        -- bind_imports_and_exports<M: ModuleStore, S: SymbolGraph>()
      topo_sort.rs      -- topological_sort<M: ModuleStore>()
      cycles.rs         -- find_cycles<M: ModuleStore>()

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
/// Read-only access to a module's import/export declarations.
/// Rolldown implements this on NormalModule/EcmaView.
pub trait ModuleInfo {
    fn module_idx(&self) -> ModuleIdx;

    /// All named exports declared by this module.
    fn named_exports(&self) -> &FxHashMap<CompactStr, LocalExport>;

    /// All named imports consumed by this module.
    fn named_imports(&self) -> &FxHashMap<SymbolRef, NamedImport>;

    /// Import records (after resolution, contains target ModuleIdx).
    fn import_records(&self) -> &[ResolvedImportRecord];

    /// The SymbolRef for this module's default export.
    fn default_export_ref(&self) -> SymbolRef;

    /// The SymbolRef for this module's namespace object.
    fn namespace_object_ref(&self) -> SymbolRef;

    /// Star export entries (for `export * from './foo'`).
    fn star_export_entries(&self) -> &[StarExportEntry];

    /// Indirect export entries (for `export { x } from './foo'`).
    fn indirect_export_entries(&self) -> &[IndirectExportEntry];

    /// Whether this module has ESM syntax.
    fn has_module_syntax(&self) -> bool;
}
```

### `ModuleStore` — Indexed collection of modules

```rust
/// A collection of modules, indexed by ModuleIdx.
/// Rolldown implements this on ModuleTable.
pub trait ModuleStore {
    type Module: ModuleInfo;

    fn module(&self, idx: ModuleIdx) -> &Self::Module;
    fn module_mut(&mut self, idx: ModuleIdx) -> &mut Self::Module;
    fn modules_len(&self) -> usize;
    fn iter_modules(&self) -> impl Iterator<Item = (ModuleIdx, &Self::Module)>;

    /// Import edges: which modules does `idx` import from?
    fn dependencies(&self, idx: ModuleIdx) -> &[ImportEdge];
}
```

### `SymbolGraph` — Cross-module symbol linking

```rust
/// Mutable symbol linking across modules.
/// Rolldown implements this on SymbolRefDb.
pub trait SymbolGraph {
    /// Follow link chains to find the canonical symbol.
    fn canonical_ref_for(&self, symbol: SymbolRef) -> SymbolRef;

    /// Link `from` to resolve to `to`.
    fn link(&mut self, from: SymbolRef, to: SymbolRef);

    /// Get the name of a symbol.
    fn symbol_name(&self, symbol: SymbolRef) -> &str;
}
```

## Shared Types

### `ModuleIdx`
```rust
oxc_index::define_index_type! {
    pub struct ModuleIdx = u32;
}
```

### `SymbolRef`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolRef {
    pub owner: ModuleIdx,
    pub symbol: SymbolId,
}
```

### Import/Export Types
```rust
pub struct NamedImport {
    pub imported_name: CompactStr,     // "foo" in `import { foo }`
    pub local_symbol: SymbolRef,       // Local binding symbol
    pub record_idx: ImportRecordIdx,   // Points to import record
    pub is_type: bool,
}

pub struct LocalExport {
    pub exported_name: CompactStr,
    pub local_symbol: SymbolRef,
}

pub struct ResolvedExport {
    pub symbol_ref: SymbolRef,
    pub potentially_ambiguous: Option<Vec<SymbolRef>>,
}

pub struct ResolvedImportRecord {
    pub specifier: CompactStr,
    pub resolved_module: Option<ModuleIdx>,
    pub kind: ImportKind,  // Static, Dynamic, Require
}

pub struct ImportEdge {
    pub specifier: CompactStr,
    pub target: ModuleIdx,
    pub is_type: bool,
}

pub enum MatchImportKind {
    Normal { symbol_ref: SymbolRef },
    Namespace { namespace_ref: SymbolRef },
    Ambiguous { candidates: Vec<SymbolRef> },
    Cycle,
    NoMatch,
}
```

### `ModuleRecord` (owned)
Copied from `oxc_linter::module_record` pattern — owned version of `oxc_syntax::module_record::ModuleRecord<'a>` with `From` conversions. Used by default implementations. Rolldown doesn't need this; it has its own `EcmaView`.

## Algorithms (Generic Over Traits)

```rust
/// Resolve all imports to exports across the module graph.
/// Populates resolved_exports on each module and links symbols.
pub fn bind_imports_and_exports<S, M>(store: &M, symbols: &mut S) -> Vec<BindingError>
where
    S: SymbolGraph,
    M: ModuleStore,
{
    // Phase 1: Build resolved_exports from local + indirect exports
    // Phase 2: Propagate star re-exports (merge, detect ambiguity)
    // Phase 3: Match each import to target's resolved exports
    //          Link symbols via SymbolGraph::link()
}

/// Topological sort of module dependencies (Kahn's algorithm).
pub fn topological_sort<M: ModuleStore>(store: &M, entries: &[ModuleIdx]) -> Option<Vec<ModuleIdx>>;

/// Find all cycles in the module graph (DFS with on-stack tracking).
pub fn find_cycles<M: ModuleStore>(store: &M) -> Vec<Vec<ModuleIdx>>;
```

## How Rolldown Adopts This

Rolldown implements the traits on its existing types — **no data structure changes needed**:

```rust
// In Rolldown's codebase:
impl ModuleInfo for NormalModule {
    fn module_idx(&self) -> ModuleIdx { self.idx.into() }
    fn named_exports(&self) -> ... { &self.ecma_view.named_exports }
    fn named_imports(&self) -> ... { &self.ecma_view.named_imports }
    // ...
}

impl ModuleStore for LinkStageOutput {
    type Module = Module;
    fn module(&self, idx: ModuleIdx) -> &Module { &self.module_table[idx] }
    // ...
}

impl SymbolGraph for SymbolRefDb {
    fn canonical_ref_for(&self, s: SymbolRef) -> SymbolRef { self.canonical_ref_for(s) }
    fn link(&mut self, from: SymbolRef, to: SymbolRef) { self.link(from, to) }
    // ...
}

// Then call the shared algorithm:
let errors = oxc_module_graph::bind_imports_and_exports(&module_table, &mut symbol_db);
```

## Key Files to Reuse/Reference

| File | Why |
|------|-----|
| `oxc/crates/oxc_syntax/src/module_record.rs` | Arena `ModuleRecord<'a>` — source for owned conversion |
| `oxc/crates/oxc_linter/src/module_record.rs` | Owned `ModuleRecord` + `From` impls — same pattern |
| `oxc/crates/oxc_linter/src/module_graph_visitor.rs` | Graph traversal reference |
| `oxc/crates/oxc_linter/src/service/runtime.rs` | Resolver setup + parallel graph building |
| `oxc/crates/oxc_syntax/src/symbol.rs` | `SymbolId` index type pattern |
| `oxc/crates/oxc_semantic/src/scoping.rs` | `Scoping` type for default impl |
| `rolldown/crates/rolldown_common/src/types/symbol_ref_db.rs` | Rolldown's SymbolRefDb design |
| `rolldown/crates/rolldown/src/stages/link_stage/bind_imports_and_exports.rs` | Binding algorithm reference |

## Implementation Stages

### Stage 1: Core Types + Traits — Complete
**Goal**: Define `ModuleIdx`, `SymbolRef`, shared import/export types, and the 3 core traits (`ModuleInfo`, `ModuleStore`, `SymbolGraph`).

### Stage 2: Default Implementations — Complete
**Goal**: `Module`, `ModuleGraph`, `SymbolRefDb` — concrete types implementing the traits. Owned `ModuleRecord` with `From` conversions.

### Stage 3: Graph Builder + Resolution — Complete
**Goal**: `ModuleGraphBuilder` using `oxc_parser` + `oxc_semantic` + `oxc_resolver`. BFS from entry points.

### Stage 4: Binding Algorithm — Complete
**Goal**: `bind_imports_and_exports()` generic over traits. Named/default/namespace imports, re-exports, star re-exports, ambiguity detection.

### Stage 5: Graph Algorithms + Polish — Complete
**Goal**: `topological_sort()`, `find_cycles()`, docs, integration tests. 17 tests passing.

## Verification

```bash
cargo test -p oxc_module_graph        # 17 tests passing
cargo clippy -p oxc_module_graph      # Clean
just fmt
```
