//! # oxc_module_graph
//!
//! Cross-module analysis for the Oxc toolchain.
//!
//! Provides **trait-based abstractions** and **algorithms** for building module
//! dependency graphs, resolving imports to exports, and linking symbols across
//! module boundaries.
//!
//! **Provides:** trait interfaces, graph algorithms, import-to-export binding,
//! default implementations.
//!
//! **Does not provide:** tree-shaking, CommonJS interop, chunk splitting —
//! consumers layer those on top.
//!
//! # Motivation
//!
//! Rolldown and similar bundlers need cross-module analysis: knowing which
//! module imports what from which, resolving imports to exports, and linking
//! symbols across module boundaries. Today, Oxc provides excellent per-file
//! analysis (`oxc_parser`, `oxc_semantic`), while Rolldown has its own tightly
//! coupled implementations (`EcmaView`, `SymbolRefDb`, `LinkStage`). The linter
//! also has an ad-hoc version (`oxc_linter::module_record`).
//!
//! This crate separates **algorithms from data structures** via traits, so
//! Rolldown can implement the traits with its own types and adopt
//! incrementally, while other tools get a batteries-included default
//! implementation.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │                    oxc_module_graph                       │
//! │                                                          │
//! │  ┌──────────────┐  ┌───────────────┐  ┌──────────────┐  │
//! │  │    Traits     │  │  Algorithms   │  │   Defaults   │  │
//! │  │              │  │               │  │              │  │
//! │  │ ModuleInfo   │◄─┤ bind_imports  │  │ Module       │  │
//! │  │ ModuleStore  │◄─┤ topo_sort     │  │ ModuleGraph  │  │
//! │  │ SymbolGraph  │◄─┤ find_cycles   │  │ SymbolRefDb  │  │
//! │  │              │  │               │  │ Builder      │  │
//! │  └──────────────┘  └───────────────┘  └──────────────┘  │
//! │         ▲                                    ▲           │
//! └─────────┼────────────────────────────────────┼───────────┘
//!           │                                    │
//!     Rolldown implements                 Other tools use
//!     traits with own types               default impls
//! ```
//!
//! # Module layout
//!
//! - [`traits`] — The stable API contract: [`ModuleInfo`], [`ModuleStore`],
//!   [`SymbolGraph`].
//! - [`types`] — Shared data types used by both traits and implementations:
//!   [`ModuleIdx`], [`SymbolRef`], [`NamedImport`], [`LocalExport`],
//!   [`ResolvedExport`], [`ImportEdge`], [`ModuleRecord`], etc.
//! - [`algo`] — Algorithms generic over the traits:
//!   [`bind_imports_and_exports`], [`topological_sort`], [`find_cycles`].
//! - [`default`] — Batteries-included implementations: [`default::Module`],
//!   [`default::DefaultModuleGraph`], [`default::SymbolRefDb`],
//!   [`default::ModuleGraphBuilder`].
//!
//! # Core traits
//!
//! ## `ModuleInfo` — read parse-time data from a module
//!
//! Read-only access to a module's import/export declarations.  Rolldown
//! implements this on `NormalModule`/`EcmaView`; the default implementation
//! uses [`default::Module`].
//!
//! ```rust,ignore
//! pub trait ModuleInfo {
//!     fn module_idx(&self) -> ModuleIdx;
//!     fn named_exports(&self) -> &FxHashMap<CompactString, LocalExport>;
//!     fn named_imports(&self) -> &FxHashMap<SymbolRef, NamedImport>;
//!     fn import_records(&self) -> &[ResolvedImportRecord];
//!     fn default_export_ref(&self) -> SymbolRef;
//!     fn namespace_object_ref(&self) -> SymbolRef;
//!     fn star_export_entries(&self) -> &[StarExportEntry];
//!     fn indirect_export_entries(&self) -> &[IndirectExportEntry];
//!     fn has_module_syntax(&self) -> bool;
//! }
//! ```
//!
//! ## `ModuleStore` — indexed collection of modules
//!
//! A collection of modules, indexed by `ModuleIdx`.  Rolldown implements this
//! on `ModuleTable`; the default implementation uses
//! [`default::DefaultModuleGraph`].
//!
//! ```rust,ignore
//! pub trait ModuleStore {
//!     type Module: ModuleInfo;
//!     fn module(&self, idx: ModuleIdx) -> &Self::Module;
//!     fn module_mut(&mut self, idx: ModuleIdx) -> &mut Self::Module;
//!     fn modules_len(&self) -> usize;
//!     fn iter_modules(&self) -> impl Iterator<Item = (ModuleIdx, &Self::Module)>;
//!     fn dependencies(&self, idx: ModuleIdx) -> &[ImportEdge];
//! }
//! ```
//!
//! ## `SymbolGraph` — cross-module symbol linking
//!
//! Mutable symbol linking across modules using a union-find style pattern.
//! Rolldown implements this on `SymbolRefDb`; the default implementation uses
//! [`default::SymbolRefDb`].
//!
//! ```rust,ignore
//! pub trait SymbolGraph {
//!     fn canonical_ref_for(&self, symbol: SymbolRef) -> SymbolRef;
//!     fn link(&mut self, from: SymbolRef, to: SymbolRef);
//!     fn symbol_name(&self, symbol: SymbolRef) -> &str;
//! }
//! ```
//!
//! # Algorithms
//!
//! All algorithms are generic over the traits, so they work with both the
//! default implementations and custom consumer types.
//!
//! ## `bind_imports_and_exports`
//!
//! The main cross-module linking algorithm.  Resolves all imports to exports
//! and links symbols via [`SymbolGraph::link`].
//!
//! Operates in three phases:
//!
//! 1. **Initialize resolved exports** — for each module, collect its local
//!    exports into a `resolved_exports` map.
//! 2. **Propagate star re-exports** — DFS with cycle detection merges
//!    `export *` targets into each module's resolved exports.  Explicit local
//!    exports shadow star re-exports.  When the same name arrives from
//!    multiple star sources at equal precedence, it is marked as potentially
//!    ambiguous.  `export { x } from './foo'` (indirect re-exports) are also
//!    resolved in this phase.
//! 3. **Match imports and link symbols** — for each import in every module,
//!    look up the imported name in the target module's resolved exports and
//!    call `SymbolGraph::link` to wire the local symbol to the canonical
//!    export symbol.  Namespace imports (`import * as ns`) are linked to the
//!    target's namespace object.  Unresolved and ambiguous imports are
//!    collected as [`algo::BindingError`]s.
//!
//! ```rust,ignore
//! let errors = bind_imports_and_exports(&graph, &mut symbols);
//! ```
//!
//! ## `topological_sort`
//!
//! Kahn's algorithm over the reachable subgraph from the given entry points.
//! Returns `Some(ordered)` for a DAG, `None` if cycles exist.
//!
//! ```rust,ignore
//! let order = topological_sort(&graph, &[entry_idx]);
//! ```
//!
//! ## `find_cycles`
//!
//! DFS-based cycle detection.  Returns all cycles as lists of `ModuleIdx`,
//! each reported once.
//!
//! ```rust,ignore
//! let cycles = find_cycles(&graph);
//! ```
//!
//! # Default implementations
//!
//! For tools that do not need to plug in custom types, the [`default`] module
//! provides concrete implementations of all three traits:
//!
//! - [`default::Module`] — stores all import/export data plus the file path.
//! - [`default::DefaultModuleGraph`] — an `IndexVec<ModuleIdx, Module>`.
//! - [`default::SymbolRefDb`] — per-module symbol tables with union-find
//!   linking for `canonical_ref_for`.
//! - [`default::ModuleGraphBuilder`] — BFS from entry points using
//!   `oxc_parser` + `oxc_semantic` + `oxc_resolver`.
//!
//! ```rust,ignore
//! use oxc_module_graph::default::ModuleGraphBuilder;
//!
//! let result = ModuleGraphBuilder::new().build(&[entry_path]);
//! let graph = result.graph;    // DefaultModuleGraph
//! let symbols = result.symbols; // SymbolRefDb
//! ```
//!
//! # Shared types
//!
//! [`types::ModuleIdx`] is a `u32` newtype index (via `oxc_index`).
//!
//! [`types::SymbolRef`] is a `(ModuleIdx, SymbolId)` pair that uniquely
//! identifies a symbol across the entire module graph.
//!
//! [`types::ModuleRecord`] is an owned copy of the parser's arena-allocated
//! `oxc_syntax::module_record::ModuleRecord<'a>`, with `From` conversions for
//! all sub-types.  This follows the same pattern as
//! `oxc_linter::module_record`.  Rolldown does not need this; it has its own
//! `EcmaView`.
//!
//! Other shared types: [`NamedImport`], [`LocalExport`], [`ResolvedExport`],
//! [`ResolvedImportRecord`], [`ImportEdge`], [`StarExportEntry`],
//! [`IndirectExportEntry`], [`MatchImportKind`].
//!
//! # How Rolldown adopts this
//!
//! Rolldown implements the traits on its existing types — no data structure
//! changes needed:
//!
//! ```rust,ignore
//! // In Rolldown's codebase:
//! impl ModuleInfo for NormalModule {
//!     fn module_idx(&self) -> ModuleIdx { self.idx.into() }
//!     fn named_exports(&self) -> ... { &self.ecma_view.named_exports }
//!     fn named_imports(&self) -> ... { &self.ecma_view.named_imports }
//!     // ...
//! }
//!
//! impl ModuleStore for LinkStageOutput {
//!     type Module = Module;
//!     fn module(&self, idx: ModuleIdx) -> &Module { &self.module_table[idx] }
//!     // ...
//! }
//!
//! impl SymbolGraph for SymbolRefDb {
//!     fn canonical_ref_for(&self, s: SymbolRef) -> SymbolRef {
//!         self.canonical_ref_for(s)
//!     }
//!     fn link(&mut self, from: SymbolRef, to: SymbolRef) {
//!         self.link(from, to)
//!     }
//!     // ...
//! }
//!
//! // Then call the shared algorithm:
//! let errors = oxc_module_graph::bind_imports_and_exports(
//!     &module_table,
//!     &mut symbol_db,
//! );
//! ```
//!
//! # Related crates in Oxc
//!
//! | Crate | Relationship |
//! |-------|-------------|
//! | `oxc_syntax::module_record` | Arena-allocated `ModuleRecord<'a>` — source for `types::ModuleRecord` conversion |
//! | `oxc_linter::module_record` | Owned `ModuleRecord` + `From` impls — same pattern as our `types::ModuleRecord` |
//! | `oxc_parser` | Produces the `Program` and `ModuleRecord` consumed by `ModuleGraphBuilder` |
//! | `oxc_semantic` | Produces `Scoping` (symbol table) consumed by `ModuleGraphBuilder` |
//! | `oxc_resolver` | Resolves specifiers to file paths in `ModuleGraphBuilder` |

pub mod algo;
pub mod default;
pub mod traits;
pub mod types;

// Re-export core types at crate root for convenience.
pub use traits::{ModuleInfo, ModuleStore, SymbolGraph};
pub use types::{ModuleIdx, SymbolRef};

// Re-export algorithms at crate root.
pub use algo::{bind_imports_and_exports, find_cycles, topological_sort};
