# `oxc_module_graph` — Cross-Module Analysis Crate

## Context

Rolldown and similar bundlers need cross-module analysis: knowing which module imports what from which, resolving imports to exports, and linking symbols across module boundaries. Today, oxc provides excellent **per-file** analysis (`oxc_parser`, `oxc_semantic`), while Rolldown has its own tightly-coupled implementations (`EcmaView`, `SymbolRefDb`, `LinkStage`). The linter also has its own ad-hoc version (`oxc_linter::module_record`).

This crate provides a **concrete, batteries-included module graph** and **algorithms** for building module dependency graphs, resolving imports to exports, and linking symbols across module boundaries.

**Provides**: Concrete data model + graph algorithms + import-to-export binding + builder + optional hooks.
**Does NOT provide**: Tree-shaking, chunk splitting — consumers layer those on top.

## Architecture: Concrete Types with Hooks

Algorithms operate directly on `&ModuleGraph` — no trait bounds needed. Only 2 optional hook traits remain for consumer-specific behavior (import matching and side-effects checks), with sensible built-in defaults covering CJS interop, external modules, and dynamic exports.

```
┌──────────────────────────────────────────────────────────────┐
│                      oxc_module_graph                         │
│                                                               │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐  │
│  │   Data Model    │  │   Algorithms   │  │     Hooks      │  │
│  │                │  │                │  │                │  │
│  │ NormalModule   │  │ bind_imports   │  │ ImportHooks    │  │
│  │ ExternalModule │  │ match_imports  │  │ SideEffects    │  │
│  │ ModuleGraph    │  │ exec_order     │  │   Hooks        │  │
│  │ SymbolRefDb    │  │ find_cycles    │  │ LinkConfig     │  │
│  │                │  │ tla            │  │                │  │
│  │                │  │ side_effects   │  │                │  │
│  │                │  │ dynamic_exports│  │                │  │
│  └────────────────┘  └────────────────┘  └────────────────┘  │
│         ▲                                       ▲            │
└─────────┼───────────────────────────────────────┼────────────┘
          │                                       │
    Rolldown populates                  Optional hooks for
    ModuleGraph directly               consumer-specific logic
```

## Crate Layout

```
crates/oxc_module_graph/
  Cargo.toml
  src/
    lib.rs              -- Re-exports, crate docs

    # Core data model
    graph.rs            -- ModuleGraph: module storage + symbol db + link pipeline
    module.rs           -- NormalModule, ExternalModule, Module enum, SideEffects
    hooks.rs            -- ImportHooks, SideEffectsHooks, LinkConfig

    # Shared types
    types/
      mod.rs
      module_idx.rs     -- ModuleIdx newtype index
      symbol_ref.rs     -- SymbolRef = (ModuleIdx, SymbolId)
      module_record.rs  -- Owned ModuleRecord (converted from parser)
      import_export.rs  -- NamedImport, LocalExport, ResolvedExport, ImportKind, MatchImportKind
      error.rs          -- Error types

    # Algorithms (operate on &ModuleGraph directly)
    algo/
      mod.rs
      binding.rs        -- bind_imports_and_exports, build_resolved_exports, match_imports
      cycles.rs         -- find_cycles()
      exec_order.rs     -- compute_exec_order()
      tla.rs            -- compute_tla()
      side_effects.rs   -- determine_side_effects()
      dynamic_exports.rs -- compute_has_dynamic_exports()

    # Default builder (batteries-included)
    default/
      mod.rs
      builder.rs        -- ModuleGraphBuilder (parse → semantic → resolve → BFS)
      symbol_db.rs      -- SymbolRefDb (cross-module symbol database with union-find)

  tests/
    fixtures/           -- JS/TS fixture files
    integration.rs
```

## Core Data Model

### `ModuleGraph`

The central graph type. Owns modules and symbols, provides query/build/link APIs.

```rust
pub struct ModuleGraph {
    pub modules: IndexVec<ModuleIdx, Module>,
    pub symbols: SymbolRefDb,
    entries: Vec<ModuleIdx>,
    runtime: Option<ModuleIdx>,
    exec_order: Vec<ModuleIdx>,
    cycles: Vec<Vec<ModuleIdx>>,
    binding_errors: Vec<BindingError>,
}
```

Key APIs:

- **Build**: `alloc_module_idx()`, `add_normal_module()`, `add_external_module()`, `add_symbol()`, `set_entries()`
- **Query**: `module()`, `normal_module()`, `external_module()`, `normal_modules()`, `canonical_ref()`, `symbol_name()`
- **Link**: `link(&LinkConfig)` — runs the full pipeline, or call individual algorithms

### `NormalModule`

A parsed JavaScript/TypeScript module with full import/export data.

```rust
pub struct NormalModule {
    // Identity
    pub idx: ModuleIdx,
    pub path: PathBuf,

    // Parse-time data
    pub has_module_syntax: bool,
    pub is_commonjs: bool,
    pub has_top_level_await: bool,
    pub side_effects: SideEffects,

    // Import/export declarations
    pub named_exports: FxHashMap<CompactString, LocalExport>,
    pub named_imports: FxHashMap<SymbolRef, NamedImport>,
    pub import_records: Vec<ResolvedImportRecord>,
    pub star_export_entries: Vec<StarExportEntry>,
    pub indirect_export_entries: Vec<IndirectExportEntry>,
    pub default_export_ref: SymbolRef,
    pub namespace_object_ref: SymbolRef,

    // Link-time results (populated by algorithms)
    pub resolved_exports: FxHashMap<CompactString, ResolvedExport>,
    pub has_dynamic_exports: bool,
    pub is_tla_or_contains_tla: bool,
    pub propagated_side_effects: bool,
    pub exec_order: u32,
}
```

### `ExternalModule`

An unresolvable dependency (e.g., `"react"`, `"lodash"`).

```rust
pub struct ExternalModule {
    pub idx: ModuleIdx,
    pub specifier: CompactString,
    pub side_effects: SideEffects,
    pub namespace_ref: SymbolRef,
    pub exec_order: u32,
}
```

## Hooks

Only 2 optional hook traits (with default implementations that do nothing):

### `ImportHooks`

```rust
pub trait ImportHooks {
    /// Called after every import resolution.
    fn on_resolved(&mut self, importer: ModuleIdx, local_symbol: SymbolRef,
                   result: &MatchImportKind, reexport_chain: &[SymbolRef]) { ... }

    /// Called when no match found and no built-in fallback applies.
    fn on_final_no_match(&mut self, target: ModuleIdx, import_name: &str)
                         -> Option<MatchImportKind> { None }
}
```

### `SideEffectsHooks`

```rust
pub trait SideEffectsHooks {
    /// Extra side-effects check beyond the built-in has_dynamic_exports.
    fn star_export_has_extra_side_effects(&self, importer: ModuleIdx, importee: ModuleIdx) -> bool { false }
}
```

### `LinkConfig`

```rust
pub struct LinkConfig<'a> {
    pub include_dynamic_imports: bool,
    pub cjs_interop: bool,
    pub import_hooks: Option<&'a mut dyn ImportHooks>,
    pub side_effects_hooks: Option<&'a dyn SideEffectsHooks>,
}
```

Built-in behaviors (previously required trait implementations):

- **External modules**: `ExternalModule` is a first-class graph node — algorithms use `external.namespace_ref` directly
- **CJS interop**: `is_commonjs` flag + `cjs_interop` config → `NormalAndNamespace` fallback
- **Dynamic exports**: `has_dynamic_exports` flag → `NormalAndNamespace` fallback on no match

## Algorithms

All algorithms operate on `&ModuleGraph` directly:

| Algorithm                     | Purpose                                                      |
| ----------------------------- | ------------------------------------------------------------ |
| `build_resolved_exports`      | Resolve local + star re-exports into a per-module export map |
| `match_imports`               | Match each import to resolved exports, link symbols          |
| `bind_imports_and_exports`    | Combined: `build_resolved_exports` + `match_imports`         |
| `compute_exec_order`          | DFS post-order execution sort                                |
| `compute_tla`                 | Top-level await propagation through static edges             |
| `determine_side_effects`      | Side-effects propagation through import/star-export edges    |
| `compute_has_dynamic_exports` | Identify modules with non-statically-analyzable exports      |
| `find_cycles`                 | DFS-based cycle detection                                    |

## Builder

`ModuleGraphBuilder` parses files from disk and produces a complete `ModuleGraph`:

```rust
let result = ModuleGraphBuilder::new().build(&[PathBuf::from("./entry.js")]);
let graph = result.graph;
```

The builder:

- Uses `oxc_parser` + `oxc_semantic` for parsing and symbol analysis
- Uses `oxc_resolver` for ESM import resolution
- Maps semantic `SymbolId`s to graph `SymbolRef`s (real symbol IDs, not synthetic)
- Detects top-level `await` via AST visitor
- Creates `ExternalModule` nodes for unresolvable bare specifiers (e.g., `"react"`)
- Resolves star/indirect export targets to module indices

## Rolldown Integration

Rolldown populates `ModuleGraph` directly from its own parse pipeline, then calls algorithms:

```rust
let mut graph = ModuleGraph::new();
// ... populate from Rolldown's EcmaView data ...
graph.link(&LinkConfig {
    cjs_interop: true,
    import_hooks: Some(&mut rolldown_hooks),
    ..Default::default()
});
```

## Verification

```bash
cargo test -p oxc_module_graph    # 46 tests passing
cargo clippy -p oxc_module_graph  # Clean
just fmt
```
