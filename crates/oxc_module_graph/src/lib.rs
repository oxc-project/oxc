//! # oxc_module_graph
//!
//! Cross-module analysis for the Oxc toolchain.
//!
//! Provides a **concrete, batteries-included module graph** and **algorithms**
//! for building module dependency graphs, resolving imports to exports, and
//! linking symbols across module boundaries.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │                    oxc_module_graph                       │
//! │                                                          │
//! │  ┌──────────────┐  ┌───────────────┐  ┌──────────────┐  │
//! │  │  Data Model   │  │  Algorithms   │  │    Hooks     │  │
//! │  │              │  │               │  │              │  │
//! │  │ NormalModule │  │ bind_imports  │  │ ImportHooks  │  │
//! │  │ ExtModule    │  │ exec_order    │  │ SideEffects  │  │
//! │  │ ModuleGraph  │  │ find_cycles   │  │   Hooks      │  │
//! │  │ SymbolRefDb  │  │ tla           │  │ LinkConfig   │  │
//! │  │              │  │ side_effects  │  │              │  │
//! │  └──────────────┘  └───────────────┘  └──────────────┘  │
//! │         ▲                                    ▲           │
//! └─────────┼────────────────────────────────────┼───────────┘
//!           │                                    │
//!     Rolldown populates                 Optional hooks for
//!     ModuleGraph directly               consumer-specific logic
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use oxc_module_graph::{ModuleGraph, NormalModule, bind_imports_and_exports};
//!
//! // Build phase: populate the graph
//! let mut graph = ModuleGraph::new();
//! // ... add modules, set entries ...
//!
//! // Link phase: run all algorithms
//! graph.link(&LinkConfig::default());
//!
//! // Query results directly on modules
//! for module in graph.normal_modules() {
//!     println!("{}: exec_order={}", module.path.display(), module.exec_order);
//! }
//! ```
//!
//! # Algorithms
//!
//! All algorithms operate on `&ModuleGraph` directly — no trait bounds needed.
//!
//! - [`bind_imports_and_exports`] — Resolve imports to exports + link symbols
//! - [`compute_exec_order`] — DFS post-order execution sort
//! - [`compute_tla`] — Top-level await propagation
//! - [`determine_side_effects`] — Side-effects propagation
//! - [`find_cycles`] — Cycle detection
//! - [`compute_has_dynamic_exports`] — Dynamic export detection
//! - [`build_resolved_exports`] — Build resolved exports map
//!
//! # Hooks
//!
//! Only 2 optional hook traits with default implementations:
//! - [`ImportHooks`] — Consumer-specific import matching behavior
//! - [`SideEffectsHooks`] — Consumer-specific side-effects checks

pub mod algo;
pub mod default;
pub mod graph;
pub mod hooks;
pub mod module;
pub mod types;

// Re-export core types at crate root.
pub use graph::ModuleGraph;
pub use hooks::{ImportHooks, LinkConfig, SideEffectsHooks};
pub use module::{ExternalModule, Module, NormalModule, SideEffects};

// Re-export algorithms at crate root.
pub use algo::{
    BindingError, ExecOrderConfig, ExecOrderResult, bind_imports_and_exports,
    build_resolved_exports, compute_exec_order, compute_has_dynamic_exports, compute_tla,
    determine_side_effects, find_cycles, match_imports,
};
