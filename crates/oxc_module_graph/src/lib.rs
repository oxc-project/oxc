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
//! # Trait design
//!
//! All three traits use **associated types** for `ModuleIdx` and `SymbolRef`,
//! and **callback-based iteration** (`for_each_*`) instead of returning
//! concrete collection references.  This allows consumers (like Rolldown) to
//! implement the traits directly on their own types without needing to match
//! oxc_module_graph's exact collection or index types.
//!
//! ## `ModuleInfo` — read parse-time data from a module
//!
//! ```rust,ignore
//! pub trait ModuleInfo {
//!     type ModuleIdx: Copy + Eq + Hash + Debug;
//!     type SymbolRef: Copy + Eq + Hash + Debug;
//!     fn module_idx(&self) -> Self::ModuleIdx;
//!     fn default_export_ref(&self) -> Self::SymbolRef;
//!     fn namespace_object_ref(&self) -> Self::SymbolRef;
//!     fn has_module_syntax(&self) -> bool;
//!     fn for_each_named_export(&self, f: &mut dyn FnMut(&str, Self::SymbolRef, bool));
//!     fn for_each_named_import(&self, f: &mut dyn FnMut(Self::SymbolRef, &str, usize, bool));
//!     fn import_record_count(&self) -> usize;
//!     fn import_record_resolved_module(&self, idx: usize) -> Option<Self::ModuleIdx>;
//!     fn for_each_star_export(&self, f: &mut dyn FnMut(Self::ModuleIdx));
//!     fn for_each_indirect_export(&self, f: &mut dyn FnMut(&str, &str, Self::ModuleIdx));
//! }
//! ```
//!
//! ## `ModuleStore` — indexed collection of modules
//!
//! ```rust,ignore
//! pub trait ModuleStore {
//!     type ModuleIdx: Copy + Eq + Hash + Debug;
//!     type SymbolRef: Copy + Eq + Hash + Debug;
//!     type Module: ModuleInfo<ModuleIdx = Self::ModuleIdx, SymbolRef = Self::SymbolRef>;
//!     fn module(&self, idx: Self::ModuleIdx) -> Option<&Self::Module>;
//!     fn modules_len(&self) -> usize;
//!     fn for_each_module(&self, f: &mut dyn FnMut(Self::ModuleIdx, &Self::Module));
//!     fn for_each_dependency(&self, idx: Self::ModuleIdx, f: &mut dyn FnMut(Self::ModuleIdx));
//! }
//! ```
//!
//! ## `SymbolGraph` — cross-module symbol linking
//!
//! ```rust,ignore
//! pub trait SymbolGraph {
//!     type SymbolRef: Copy + Eq + Hash + Debug;
//!     fn canonical_ref_for(&self, symbol: Self::SymbolRef) -> Self::SymbolRef;
//!     fn link(&mut self, from: Self::SymbolRef, to: Self::SymbolRef);
//!     fn symbol_name(&self, symbol: Self::SymbolRef) -> &str;
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
//! and links symbols via [`SymbolGraph::link`].  Returns a [`algo::BindingResult`]
//! containing resolved exports per module and any binding errors.
//!
//! ```rust,ignore
//! let result = bind_imports_and_exports(&graph, &mut symbols);
//! // result.resolved_exports: FxHashMap<ModuleIdx, FxHashMap<String, ResolvedExport>>
//! // result.errors: Vec<BindingError>
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
//! DFS-based cycle detection.  Returns all cycles as lists of module indices,
//! each reported once.
//!
//! ```rust,ignore
//! let cycles = find_cycles(&graph);
//! ```
//!
//! ## `compute_has_dynamic_exports`
//!
//! Identifies modules whose exports are not statically analyzable because they
//! (transitively) `export *` from a CommonJS or external module.
//!
//! ```rust,ignore
//! let dynamic_modules = compute_has_dynamic_exports(&graph);
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

pub mod algo;
pub mod default;
pub mod traits;
pub mod types;

// Re-export core traits at crate root for convenience.
pub use traits::{DefaultImportMatcher, ImportMatcher, ModuleInfo, ModuleStore, SymbolGraph};

// Re-export algorithms at crate root.
pub use algo::{
    BindingError, BindingResult, bind_imports_and_exports, build_resolved_exports,
    compute_has_dynamic_exports, find_cycles, match_imports, topological_sort,
};
