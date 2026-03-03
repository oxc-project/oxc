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
│  │                │  │ exports_kind   │  │                │  │
│  │                │  │ wrapping       │  │                │  │
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
      import_export.rs  -- NamedImport, LocalExport, ResolvedExport, ImportKind, MatchImportKind, ExportsKind, WrapKind, ImportRecordMeta, NamespaceAlias
      error.rs          -- Error types

    # Algorithms (operate on &ModuleGraph directly)
    algo/
      mod.rs
      binding.rs        -- bind_imports_and_exports, build_resolved_exports, match_imports
      cycles.rs         -- find_cycles()
      exec_order.rs     -- compute_exec_order()
      exports_kind.rs   -- determine_module_exports_kind()
      tla.rs            -- compute_tla()
      side_effects.rs   -- determine_side_effects()
      dynamic_exports.rs -- compute_has_dynamic_exports()
      wrapping.rs       -- wrap_modules()
      safely_merge_cjs_ns.rs -- determine_safely_merge_cjs_ns()

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
    pub exports_kind: ExportsKind,      // Esm | CommonJs | None (replaces is_commonjs: bool)
    pub has_top_level_await: bool,
    pub side_effects: SideEffects,
    pub has_lazy_export: bool,          // Rolldown: skip ESM classification on static import
    pub execution_order_sensitive: bool, // Forces wrapping even with on-demand optimization

    // Import/export declarations
    pub named_exports: FxHashMap<CompactString, LocalExport>,
    pub named_imports: FxHashMap<SymbolRef, NamedImport>,
    pub import_records: Vec<ResolvedImportRecord>,
    pub star_export_entries: Vec<StarExportEntry>,
    pub indirect_export_entries: Vec<IndirectExportEntry>,
    pub default_export_ref: SymbolRef,
    pub namespace_object_ref: SymbolRef,

    // Link-time results (populated by algorithms)
    pub wrap_kind: WrapKind,            // None | Cjs | Esm
    pub original_wrap_kind: WrapKind,   // Initial wrap_kind before propagation
    pub wrapper_ref: Option<SymbolRef>, // e.g., require_foo or init_foo
    pub required_by_other_module: bool, // Imported via require() by another module
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
- **CJS interop**: `exports_kind.is_commonjs()` + `cjs_interop` config → `NormalAndNamespace` fallback
- **Dynamic exports**: `has_dynamic_exports` flag → `NormalAndNamespace` fallback on no match

## Algorithms

All algorithms operate on `&ModuleGraph` directly:

| Algorithm                       | Purpose                                                           |
| ------------------------------- | ----------------------------------------------------------------- |
| `build_resolved_exports`        | Resolve local + star re-exports into a per-module export map      |
| `match_imports`                 | Match each import to resolved exports, link symbols               |
| `bind_imports_and_exports`      | Combined: `build_resolved_exports` + `match_imports`              |
| `compute_exec_order`            | DFS post-order execution sort                                     |
| `compute_tla`                   | Top-level await propagation through static edges                  |
| `determine_side_effects`        | Side-effects propagation through import/star-export edges         |
| `compute_has_dynamic_exports`   | Identify modules with non-statically-analyzable exports           |
| `determine_module_exports_kind` | Classify module export format (ESM/CJS) and mark initial wrapping |
| `wrap_modules`                  | Propagate wrapping through deps and create wrapper symbols        |
| `determine_safely_merge_cjs_ns` | Identify ESM→CJS imports whose namespace refs can be merged       |
| `find_cycles`                   | DFS-based cycle detection                                         |

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

Rolldown populates `ModuleGraph` directly from its own parse pipeline, then calls algorithms.

**Simple mode** — call `graph.link()` which runs the full pipeline:

```rust
let mut graph = ModuleGraph::new();
// ... populate from Rolldown's EcmaView data ...
graph.link(&mut LinkConfig {
    cjs_interop: true,
    import_hooks: Some(&mut rolldown_hooks),
    ..Default::default()
});
```

**Step-by-step mode** — call individual algorithms with interleaved consumer steps.
This is Rolldown's actual workflow since it needs to run `determine_module_exports_kind`
and `wrap_modules` between algorithm calls.

### Step-by-step Walkthrough

#### Phase 1: Populate

```rust
let mut graph = ModuleGraph::new();

// For each module parsed by Rolldown:
let idx = graph.alloc_module_idx();
// Copy symbols from Rolldown's scoping into SymbolRefDb:
let sym_ref = graph.add_symbol(idx, name);
// Build NormalModule from EcmaView data:
graph.add_normal_module(NormalModule {
    idx,
    path: ecma_view.source.path.clone(),
    has_module_syntax: ecma_view.has_module_syntax,
    exports_kind: ecma_view.exports_kind,  // ExportsKind::Esm | CommonJs | None
    named_exports: /* convert from EcmaView */,
    named_imports: /* convert from EcmaView */,
    import_records: /* convert resolved records, each with namespace_ref + meta */,
    ..Default::default()
});
// External modules:
graph.add_external_module(ExternalModule { idx, specifier, namespace_ref, .. });
graph.set_entries(entry_indices);
```

#### Phase 2: Step-by-step Link

```rust
use oxc_module_graph::*;

// 1. Execution order
let config = LinkConfig { include_dynamic_imports: false, ..Default::default() };
let exec = compute_exec_order(&graph, &config);
for (i, &idx) in exec.sorted.iter().enumerate() {
    match graph.module_mut(idx) {
        Module::Normal(m) => m.exec_order = i as u32,
        Module::External(m) => m.exec_order = i as u32,
    }
}

// 2. TLA propagation
let tla = compute_tla(&graph);
for &idx in &tla {
    graph.normal_module_mut(idx).map(|m| m.is_tla_or_contains_tla = true);
}

// 3. Determine module exports kind
let ek_config = ExportsKindConfig {
    dynamic_imports_as_require: false,
    wrap_cjs_entries: true,
};
let ek_result = determine_module_exports_kind(&graph, &ek_config);
for (&idx, &kind) in &ek_result.exports_kind_updates {
    graph.normal_module_mut(idx).map(|m| m.exports_kind = kind);
}
for (&idx, &wrap) in &ek_result.wrap_kind_updates {
    graph.normal_module_mut(idx).map(|m| m.wrap_kind = wrap);
}

// 4. Wrap modules
let wrap_config = WrapModulesConfig {
    on_demand_wrapping: false,
    strict_execution_order: false,
    skip_symbol_creation: false,
};
let wrap_result = wrap_modules(&mut graph, &wrap_config);
for (&idx, &wrap) in &wrap_result.wrap_kind_updates {
    graph.normal_module_mut(idx).map(|m| m.wrap_kind = wrap);
}
for (&idx, &wrapper) in &wrap_result.wrapper_refs {
    graph.normal_module_mut(idx).map(|m| m.wrapper_ref = Some(wrapper));
}
for (&idx, &orig) in &wrap_result.original_wrap_kinds {
    graph.normal_module_mut(idx).map(|m| m.original_wrap_kind = orig);
}
for &idx in &wrap_result.required_by_other_module {
    graph.normal_module_mut(idx).map(|m| m.required_by_other_module = true);
}

// 5. Dynamic exports (runs after exports_kind is finalized)
let dynamic = compute_has_dynamic_exports(&graph);
for &idx in &dynamic {
    graph.normal_module_mut(idx).map(|m| m.has_dynamic_exports = true);
}

// 6. Resolved exports
let resolved = build_resolved_exports(&graph);
for (idx, exports) in resolved {
    graph.normal_module_mut(idx).map(|m| m.resolved_exports = exports);
}

// 7. Match imports (with hooks)
let mut hooks = RolldownImportHooks { /* ... */ };
let mut config = LinkConfig {
    cjs_interop: true,
    import_hooks: Some(&mut hooks),
    ..Default::default()
};
let (errors, links) = match_imports_collect(&graph, &mut config);
// Apply links to Rolldown's SymbolRefDb (or graph.symbols):
for (from, to) in links {
    rolldown_symbol_db.link(from, to);
}

// 8. Side effects
let se = determine_side_effects(&graph, &config);
for (idx, has) in se {
    graph.normal_module_mut(idx).map(|m| m.propagated_side_effects = has);
}
```

#### Phase 3: Extract Results

Rolldown reads results directly from `NormalModule` fields (`exec_order`,
`exports_kind`, `wrap_kind`, `original_wrap_kind`, `wrapper_ref`,
`required_by_other_module`, `resolved_exports`, `has_dynamic_exports`,
`is_tla_or_contains_tla`, `propagated_side_effects`)
and from the link pairs returned by `match_imports_collect`.

Rolldown can also call `determine_safely_merge_cjs_ns(&graph)` to identify
ESM imports of CJS modules whose namespace refs can be merged into a single
representative symbol (optimization for ESM→CJS interop).

## Verification

```bash
cargo test -p oxc_module_graph    # 79 tests passing
cargo clippy -p oxc_module_graph  # Clean
just fmt
```
